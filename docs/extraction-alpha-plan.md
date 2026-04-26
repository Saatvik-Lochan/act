# ACT Extraction Alpha-Bridge Plan

Status: QKV generated-artifact prototype implemented; generator backport pending.

## Goal

Separate IR and ISA e-classes during extraction. IR e-classes may contain IR operators and `alpha-*` bridge operators; ISA e-classes should contain executable ISA nodes/leaves and should not point back to IR.

`alpha-D(x)` is an IR-facing, non-executable operator that exposes an ISA expression `x` whose output lives in buffer `D` as an ordinary IR value. It is value-transparent and bijective on values.

## Core rewrite shape

Old lowering:

```text
(ir-node ?a ?b) => (isa-node ?a ?b)
```

Alpha lowering:

```text
(ir-node (alpha-in ?a) (alpha-in ?b)) => (alpha-out (isa-node ?a ?b))
```

For QKV the handpatched wrappers are:

- `load_rm`, `load_cm`: `AlphaHBM -> AlphaD1`
- `store_rm`, `store_cm`: `AlphaD1 -> AlphaHBM`
- `mov`: `AlphaD2 -> AlphaD1`
- `gemm`: `AlphaD1, AlphaD1 -> AlphaD2`
- `softmax`: `AlphaD2 -> AlphaD2`

## Implemented QKV artifact changes

### E-graph language

File: `targets/QKV/backend/src/ir/egraph.rs`

- Added `TensorOp::AlphaHBM(Id)`, `TensorOp::AlphaD1(Id)`, `TensorOp::AlphaD2(Id)`.
- Pattern names are `alpha-hbm`, `alpha-d1`, `alpha-d2`.
- Alpha operators carry no metadata string; buffer identity is encoded in the variant.
- Alpha operators are non-executable and are not added to `buffer_assignment`.
- `TensorInfo::make` propagates shape/dtype/`is_const` through alpha from the child.

### Parameter initialization

File: `targets/QKV/backend/src/isel/initializer/lib.rs`

- Raw `Var` remains the HBM ISA leaf used for offsets/input tracking.
- The initializer now wraps raw parameters as:

```text
Var -> AlphaHBM(Var) -> existing reshape/bitcast typed-parameter normalization
```

This preserves existing load-rule structure while ensuring HLO parameter expressions are IR-facing alpha values.

### IR-to-ISA rewrites

Files:

- `targets/QKV/backend/src/isel/rewrites/ir2isa_rewrites.txt`
- `targets/QKV/backend/src/isel/rewrites/ir2isa_rewrites.rs`

- Lowering rules consume alpha-wrapped operands and produce alpha-wrapped ISA results.
- Generated precondition/metadata/shape helpers were handpatched to account for the extra alpha nodes.

### Alpha injectivity hook

Files:

- `targets/QKV/backend/src/isel/rewrites/alpha.rs`
- `targets/QKV/backend/src/isel/rewrites/mod.rs`
- `targets/QKV/backend/src/pipeline.rs`

During saturation, if one e-class contains multiple alpha nodes of the same kind, their children are unioned:

```text
alpha-D(x) == alpha-D(y)  =>  x == y
```

This encodes alpha's value-level bijectivity without introducing inverse nodes into ISA e-classes.

### Alpha-root extraction

Files:

- `targets/QKV/backend/src/isel/extractor/alpha.rs`
- `targets/QKV/backend/src/isel/extractor/mod.rs`
- `targets/QKV/backend/src/isel/extractor/lib.rs`

The public extractor now:

1. Finds `AlphaHBM(child)` at the HLO root e-class.
2. Treats `child` as the root ISA e-class.
3. Runs egg's `Extractor` with naive tree cost `1 + sum(child_costs)`.
4. Converts the extracted expression to `PiiGraph`.

For `RecExpr -> PiiGraph`, non-leaf extracted enodes cannot be looked up directly because their child ids are expression-local, not e-graph ids. Conversion therefore starts with leaves, then uses `map_children` to rewrite expression-local child ids to recovered e-graph ids before lookup.

If no root `AlphaHBM` exists, extraction prints a debug message and returns `vec![]`. Multiple root `AlphaHBM` nodes after injectivity enforcement are treated as an invariant violation.

## Validation flow

```bash
scripts/build-backend.sh QKV
scripts/compile.sh --input attention.hlo --output asm/compiled_qkv.py --log /tmp/log
scripts/run.sh test_qkv.py
```

`asm/` is generated output and should not be committed.

## Backport notes

Keep the QKV artifact patch simple so it can be reproduced in the backend generator.

Generator work should eventually:

- emit alpha variants for implicit HBM plus each target data model/buffer;
- derive alpha wrappers from each instruction's fixed input/output buffer signature;
- generate the alpha-aware rewrite helpers instead of handpatching `ir2isa_rewrites.rs`;
- handle tiled and untiled parameter initialization systematically;
- decide whether alpha shape propagation stays in e-graph analysis or moves into generated `set_shapes_*` logic.

## Deferred design notes

- Constants need a separate revamp. Current `DetectedConst` insertion happens during extraction, which is too late for alpha-based rewriting if constants need to participate as ISA leaves.
- ISA e-class analysis may eventually include concrete buffer information; current prototype relies on operation-level `buffer_assignment`.
- Add custom extraction costs per e-node / ISA instruction later.
- Revisit an `alpha_inv` inverse/projection-operator rewrite encoding for alpha injectivity. The current hook avoids inverse-node pollution in ISA e-classes.
