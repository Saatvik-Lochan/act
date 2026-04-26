# ACT Extraction Alpha-Bridge Plan

Status: ready for QKV generated-artifact prototype

## Goal

Reduce extraction complexity by separating IR and ISA e-classes. Introduce an `alpha` bridge node so IR-level equivalence can point to ISA implementations without merging IR nodes and ISA nodes into the same representation space.

## Current problem

Today, rewrite rules directly union IR expressions with ISA instructions, e.g.

```text
(dot ?a ?b) => (gemm ?a ?b)
```

This means:

- IR and ISA nodes can live in the same e-class.
- ISA alternatives for different buffers can live in the same tensor e-class.
- Extractors must track requested buffer state while walking mixed IR/ISA e-classes.

## Target invariant

After the change:

- ISA e-classes contain only ISA nodes and ISA leaves.
- ISA nodes have children that are ISA e-classes.
- IR e-classes contain IR nodes and may contain `alpha-D(...)` bridge operators.
- IR e-classes can reference ISA implementations only through alpha operators.
- An alpha operator abstracts away ISA-specific details, including concrete buffer `D`, and exposes/casts the wrapped executable ISA expression as a non-executable IR expression.
- Extraction should select executable ISA subgraphs by crossing through alpha operators, but alpha itself should not be treated as an executable instruction.

## Proposed rewrite shape

Old:

```text
(ir-node ?a ?b) => (isa-node ?a ?b)
```

New:

```text
(ir-node (alpha ?a) (alpha ?b)) => (alpha (isa-node ?a ?b))
```

The `alpha` node abstracts away buffer/ISA details while presenting an IR-typed value to the IR e-class.

## Implementation principle

Keep generated-artifact changes targeted and simple. Any prototype patch in `targets/QKV/backend/` should be easy to replicate in the backend generator pipeline later. Avoid clever one-off artifact-only logic unless it is clearly temporary and documented.

## Implementation stages

### Stage 1: Add per-buffer alpha operators to QKV e-graph language

Files:

- `targets/QKV/backend/src/ir/egraph.rs`
- `targets/QKV/backend/src/ir/buffer.rs`

Tasks:

- First handpatch the QKV generated artifact with exactly the buffers it has: `TensorOp::AlphaHBM(Id)`, `TensorOp::AlphaD1(Id)`, `TensorOp::AlphaD2(Id)`.
- Alpha variants should not carry metadata strings; the buffer is encoded in the variant name.
- Add child handling, parsing, display, metadata behavior.
- Use lowercase hyphenated pattern names: `alpha-hbm`, `alpha-d1`, `alpha-d2`.
- Treat alpha as an IR-facing operator, not an executable ISA operation.
- `buffer_assignment(alpha)` should return `None`; alpha must not be considered a valid PII/executable node.
- Later generator backport should emit alpha variants automatically for implicit HBM plus each target data model/buffer.

### Stage 2: Update IR-to-ISA rewrites

Files:

- `targets/QKV/backend/src/isel/rewrites/ir2isa_rewrites.txt`
- `targets/QKV/backend/src/isel/rewrites/ir2isa_rewrites.rs`
- `targets/QKV/backend/src/isel/rewrites/applier.rs` if metadata/shapes need special handling

Tasks:

- Rewrite generated ISA lowering rules to produce `Alpha<OutBuffer>(isa(...))`.
- Rewrite rule LHSs to consume operands wrapped in `Alpha<InputBuffer>(...)`.
- For QKV artifact patching, hand-write alpha wrappers from known buffer signatures:
  - `load_rm`, `load_cm`: `AlphaHBM` input, `AlphaD1` output
  - `store_rm`, `store_cm`: `AlphaD1` input, `AlphaHBM` output
  - `mov`: `AlphaD2` input, `AlphaD1` output
  - `gemm`: `AlphaD1`, `AlphaD1` inputs, `AlphaD2` output
  - `softmax`: `AlphaD2` input, `AlphaD2` output
- Later generator backport should derive these wrappers automatically from each instruction's fixed buffer assignment.
- Preserve existing precondition/metadata/shape logic.

### Stage 3: Update initializer / parameter leaf handling

Files:

- `targets/QKV/backend/src/isel/initializer/*`

Tasks:

- Keep `Var` leaves as ISA-side HBM nodes.
- Where the initializer currently adds `TensorOp::Var(...)`, keep adding that ISA leaf first, then add `AlphaHBM(var_node)` as the HBM-facing IR expression.
- No unioning is needed for the parameter bridge: the raw `var_node` e-class remains available as the ISA leaf child.
- Preserve the existing typed-parameter normalization shape: HLO parameters should still be represented as typed IR expressions such as `bitcvt(reshape(AlphaHBM(Var)))`, so load rewrites can match.
- The initializer symbol table should map HLO parameter symbols to the final typed IR expression built over `AlphaHBM(var_node)`, not to the raw `Var` e-class.
- Do not keep extra handles beyond what the initializer already needs. The raw `Var` gets explicit analysis setup, and the alpha node's analysis data should be produced automatically by `TensorInfo::make` because alpha propagation is handled there.
- Do not revamp constants in this prototype; constants are deferred.

### Stage 4: Add alpha injectivity hook and minimal alpha-root extraction

Files:

- `targets/QKV/backend/src/isel/extractor/alpha.rs` or similar new file
- `targets/QKV/backend/src/isel/extractor/mod.rs`
- `targets/QKV/backend/src/isel/extractor/lib.rs`

Tasks:

- Add an alpha-injectivity maintenance hook during saturation: if one e-class contains multiple `AlphaD(child)` nodes of the same alpha kind, union those children because alpha is bijective on values.
- Add a new minimal extractor path rather than deeply modifying `fast.rs` / `slow.rs` first.
- Replace the public extraction path with the new alpha extractor; no fallback to old fast/slow extraction is needed for this rewrite.
- The extractor should inspect the HLO root IR e-class and find `AlphaHBM(child)` nodes.
- If more than one `AlphaHBM` exists in the root e-class after alpha injectivity enforcement, panic because this indicates an invariant violation.
- If no `AlphaHBM` exists in the root e-class, print a clear debug message and return `vec![]`; do not fallback to old extraction.
- The `child` is the root ISA e-class for full-program extraction.
- Use egg's built-in `Extractor` from that ISA root e-class after all.
- Initial cost model should be standard naive tree cost: `1 + sum(child costs)`.
- Build `PiiGraph` from the extracted expression while recovering e-graph e-class / analysis data (`TensorInfo`).
- Recovery approach: do not naively look up extracted non-leaf enodes, because their child `Id`s refer to the extracted expression, not the original e-graph. Start from extracted leaf enodes because leaves have no child `Id`s, so lookup vacuously works. Then retroactively build a map from expression-local ids to original e-graph ids. For parents, use `map_children` on enodes to translate extracted expression child ids to recovered e-graph ids before lookup.
- Later add handwritten/custom costs per e-node or ISA instruction.
- Since ISA e-classes should never point to IR e-classes, extraction should only see executable ISA ops and ISA leaves (`Var`; constants are deferred for this prototype).
- `alpha` itself must not be emitted into `.pii`.
- This is the first Cut B implementation: enough extraction shim to compile QKV while preserving the planned full extractor overhaul. Old fast/slow extractors should remain in files temporarily but unused; `lib.rs::extract` should call the alpha extractor only.
- The minimal extractor should use egg's built-in `Extractor`, not an ACT-specific first-success traversal.
- Keep the extracted expression-id to e-graph-id recovery isolated in the alpha extractor implementation.

### Stage 5: Validate on QKV attention

Commands likely needed:

```bash
scripts/build-backend.sh QKV
scripts/compile.sh --input attention.hlo --output asm/compiled_qkv.py --log /tmp/log
scripts/run.sh test_qkv.py
```

## Resolved design decisions

### D1: What exactly is the `alpha` operator?

Options:

1. `Alpha([Id; 1])` with no metadata.
2. `Alpha(String, [Id; 1])` where string stores buffer/type/shape metadata.
3. Separate alpha variants per buffer, e.g. `AlphaD1`, `AlphaD2`, `AlphaHBM`.

Decision: use separate alpha variants per concrete buffer. First implement only the QKV generated artifact variants: `AlphaHBM`, `AlphaD1`, and `AlphaD2`, each storing a single `Id` child directly, e.g. `AlphaD1(Id)`, not `[Id; 1]`. This makes e-graph rules explicit about which buffered ISA implementation is being surfaced as an IR-level value, and avoids a semantically ambiguous plain alpha. Later, the generator should emit alpha variants automatically for implicit HBM plus each target data model/buffer.

Meaning: `AlphaD(child)` is an operator that abstracts away the wrapped executable ISA expression, including its concrete buffer `D`, and exposes/casts it as a non-executable IR expression. The child should be an ISA-side e-class producing a tensor in buffer `D`.

### D2: Should `alpha` appear in PII output?

Options:

1. No, alpha is only an e-graph bridge and is stripped during extraction.
2. Yes, alpha is emitted as an explicit PII node.

Decision: no. `alpha` is not a real instruction or allocator operation, so `.pii` should stay compatible with the existing C++ parser.

### D3: How do leaves cross from IR to ISA?

Options:

1. Keep `Var` / constants as ISA-side HBM leaves and expose them to IR via `AlphaHBM(...)`.
2. Add separate explicit ISA leaf nodes distinct from IR variables/constants.
3. Special-case inputs/constants only in extraction.

Decision: for this prototype, handle parameters by keeping `Var` as an ISA-side HBM leaf and exposing it to IR with `AlphaHBM(Var(...))`. Create the raw `Var` e-class first, then create `AlphaHBM(var_node)` and use that as the child of the existing reshape/bitcast typed-parameter normalization. The HLO symbol table maps to the final typed IR expression, e.g. `bitcvt(reshape(AlphaHBM(Var)))`, while HBM offsets/inputs still refer to the raw `Var`. Constants remain conceptually ISA-side leaves exposed through `AlphaHBM`, but their implementation is deferred.

### D4: Do ISA rules require alpha-wrapped operands on the LHS?

Options:

1. Yes, all operands must be alpha-wrapped with the instruction's required input buffers: `(dot (AlphaD1 ?a) (AlphaD1 ?b)) => (AlphaD2 (gemm ?a ?b))`.
2. No, only RHS is alpha-wrapped: `(dot ?a ?b) => (AlphaD2 (gemm ?a ?b))`.

Decision: yes. Alpha wrappers are determined by each instruction's static buffer signature. This is hand-patched in the QKV generated artifact for now, and later generated automatically from the backend generator.

### D5: How much of buffer-targeted extraction remains?

Options:

1. Keep current buffer-targeted traversal initially, but cross alpha by requesting the alpha child in whatever buffer is needed.
2. Remove most target-buffer logic immediately.
3. Split into two phases: find alpha in the IR root e-class, then extract directly from the selected ISA root e-class.

Decision: prefer option 3. The introduction of alpha operators should split e-classes into IR and ISA e-classes, with ISA e-classes also split by buffer. Since ISA e-classes never point to IR e-classes, extraction can operate on the root ISA e-class reached through alpha and avoid much of the old mixed IR/ISA buffer-targeted traversal complexity. For normal full-program extraction, the selected alpha at the HLO root should be `AlphaHBM(...)`, because the final output should be in HBM.

First implementation cut is Cut B: create a minimal new alpha-root extractor. It finds `AlphaHBM(child)` in the root IR e-class, then uses egg's built-in `Extractor` from `child` as an ISA-only root with naive tree cost. Build `PiiGraph` from the extracted expression/eclasses while recovering `TensorInfo` via e-graph lookup. Avoid extensive old-extractor refactoring at this stage. This replaces the old public extraction behavior; do not fallback to old fast/slow extraction.

### D6: What happens to IR-to-IR rewrites?

Options:

1. Leave IR-to-IR rewrites unchanged; alpha participates as an IR-facing expression.
2. Block IR-to-IR rewrites from matching alpha roots.
3. Selectively allow only some IR-to-IR rules over alpha.

Decision: leave IR-to-IR rewrites unchanged for the prototype. Alpha is IR-facing, so ordinary IR rewrites should compose around it. Restrict later only if this causes blowup or bad cycles.

### D7: What about structural IR helper ops like `slice` / `concat`?

Decision: keep structural helper ops such as `slice` and `concat` as IR-side constructs for now; they should not become ISA enodes. If an ISA-side implementation needs buffer-specialized structural behavior later, address that explicitly rather than letting `ANY` helper nodes blur the IR/ISA split.

### D8: What should `buffer_assignment` return for alpha nodes?

Decision: `None`. Alpha operators are IR-facing and non-executable, so they should not be treated as PII nodes or ordinary buffer-producing ISA operations. The new extractor will explicitly unwrap alpha to reach ISA e-classes.

### D9: What names should alpha operators use in rewrite patterns?

Decision: use lowercase hyphenated pattern names matching existing style: `alpha-hbm`, `alpha-d1`, `alpha-d2`. Rewrite example: `(dot (alpha-d1 ?a) (alpha-d1 ?b)) => (alpha-d2 (gemm ?a ?b))`.

### D10: Do alpha operators carry metadata strings?

Decision: no. Alpha variants are unary operators with no metadata, e.g. `AlphaD1(Id)`. Buffer identity is encoded in the variant itself.

### D11: How should shape/dtype analysis propagate through alpha?

Options:

1. Explicitly set alpha analysis data during initializer/rewrite construction.
2. Rely on generated `set_shapes_*` functions to assign alpha root shapes.
3. Propagate shape/dtype through alpha in `TensorInfo::make`.

Decision for now: use option 3. Alpha is value-transparent, so `TensorInfo::make` should copy shape/dtype and `is_const` from the child e-class for `AlphaHBM`, `AlphaD1`, and `AlphaD2`.

Strong note: we may later shift some or all alpha shape assignment back into generated `set_shapes_*` logic for consistency with the rest of the backend generator pipeline. Revisit this during generator backport.

### D12: Which artifacts do we edit first?

Options:

1. Patch generated `targets/QKV/backend` artifacts only.
2. Patch generator first.

Decision: patch generated artifacts first for fast iteration, then backport to generator once semantics are proven.

## Implementation order

1. Add QKV alpha variants and e-graph language support (D1, D8, D9, D10).
2. Add alpha TensorInfo propagation in analysis (D11).
3. Update initializer parameter handling so HLO parameters map to `AlphaHBM(Var(...))` (D3).
4. Handpatch QKV IR-to-ISA rewrites from fixed instruction buffer signatures (D4).
5. Add alpha injectivity hook and minimal alpha-root extractor: find `AlphaHBM` at root and extract from its child ISA e-class (D5).
6. Leave IR-to-IR rewrites unchanged and keep structural helper ops IR-side (D6, D7).
7. Validate QKV compile/test.
8. Backport into generator after prototype behavior is proven.

## Decision log

- 2026-04-26: User requested plan-first workflow and direct generated-artifact iteration.
- 2026-04-26: Initial recommendation was plain `Alpha([Id; 1])`, but user rejected it as semantically wrong.
- 2026-04-26: Decided to use per-buffer alpha variants. First handpatch QKV with `AlphaHBM(Id)`, `AlphaD1(Id)`, and `AlphaD2(Id)`; later generate variants from implicit HBM plus target data models.
- 2026-04-26: Decided alpha is IR-facing, non-executable, and non-emitted.
- 2026-04-26: User clarified this change is expected to enable a complete extractor overhaul: extract from a root ISA e-class selected via alpha, with ISA e-classes never pointing back to IR e-classes.
- 2026-04-26: Decided normal full-program extraction should look for `AlphaHBM(...)` at the HLO root.
- 2026-04-26: Decided first implementation should be Cut B: add a new minimal alpha-root extractor that finds root `AlphaHBM(child)` and extracts from child as an ISA-only root using egg's built-in `Extractor` with naive tree cost.
- 2026-04-26: Decided alpha extractor should replace the public extraction path with no fallback to old fast/slow extraction. Old fast/slow files remain in place but unused.
- 2026-04-26: Re-decided to use egg's built-in extractor. Recover `TensorInfo` for `PiiGraph` by mapping extracted expression ids back to e-graph ids. Important detail: non-leaf extracted enodes cannot be naively looked up because their child `Id`s point into the extracted expression, not the e-graph. Leaf enodes can be looked up first because they have no child ids, so lookup vacuously works. Then use `map_children` to rewrite parent child ids from expression ids to recovered e-graph ids before lookup.
- 2026-04-26: Root e-class should contain at most one canonical `AlphaHBM` after alpha injectivity enforcement. Multiple root `AlphaHBM` nodes at extraction time are an invariant violation/error. Zero means print debug message and return `vec![]`.
- 2026-04-26: Decided that `Var` and constants remain ISA-side leaves, exposed to IR through `AlphaHBM`. Also noted constants need a separate revamp because current `DetectedConst` insertion happens during extraction.
- 2026-04-26: Refined HLO initializer handling: raw `Var` remains the HBM ISA leaf, `AlphaHBM(Var)` feeds existing reshape/bitcast typed-parameter normalization, the symbol table maps to the final typed IR expression, and offsets/inputs still track raw `Var`.
- 2026-04-26: Decided IR-to-ISA alpha wrappers are derived from fixed instruction input/output buffer signatures. For QKV, handpatch generated artifacts first; later backport to generator.
- 2026-04-26: Decided to leave IR-to-IR rewrites unchanged for the prototype.
- 2026-04-26: Added implementation principle: keep artifact changes targeted/simple so they can be reproduced in the backend generator.
- 2026-04-26: Decided structural helper ops such as `slice` and `concat` should remain IR-side, not ISA enodes.
- 2026-04-26: Decided alpha nodes should return `None` from `buffer_assignment`.
- 2026-04-26: Decided alpha rewrite pattern names are `alpha-hbm`, `alpha-d1`, and `alpha-d2`.
- 2026-04-26: Decided alpha operators do not carry metadata strings.
- 2026-04-26: Decided alpha should propagate shape/dtype/is_const through `TensorInfo::make` for now, with a strong note that this may move to generated `set_shapes_*` later for consistency.

## Implementation progress

- 2026-04-26: Stage 1 started in QKV artifact. Added `AlphaHBM(Id)`, `AlphaD1(Id)`, and `AlphaD2(Id)` to `targets/QKV/backend/src/ir/egraph.rs`, including child handling, parser names, and display names. Alpha remains absent from `buffer_assignment`, so it returns `None`. Also implemented alpha `TensorInfo::make` propagation so alpha copies shape/dtype/is_const from its child.
- 2026-04-26: Stage 2 started in QKV artifact. Handpatched `ir2isa_rewrites.txt` to use alpha wrappers from QKV buffer signatures, and updated `ir2isa_rewrites.rs` metadata/precondition/shape functions for the extra alpha nodes on LHS/RHS.
- 2026-04-26: Stage 3 started in QKV artifact. Updated parameter initialization so raw `Var` is wrapped with `AlphaHBM` before existing reshape/bitcast typed-parameter normalization in both tiled and untiled paths.
- 2026-04-26: Stage 4 started in QKV artifact. Added `extractor/alpha.rs`, switched public extraction to alpha-root extraction only, left old fast/slow extractors unused, and implemented `RecExpr -> PiiGraph` conversion with expression-id to e-graph-id recovery.
- 2026-04-26: Added alpha injectivity maintenance hook. During saturation, same-kind alpha nodes in the same e-class cause their children to be unioned, reflecting alpha's bijectivity on values.
- 2026-04-26: Reintroduced alpha-root panic: multiple root `AlphaHBM` nodes after alpha injectivity enforcement are treated as an invariant violation.

## Deferred design notes

- E-graph analysis data for ISA e-classes should probably include the concrete buffer as an argument/field. This may become useful once ISA e-classes are deliberately split by buffer. It is not urgent for the first QKV artifact prototype and may be ignored initially if existing `buffer_assignment` is sufficient.
- Later, write custom costs per e-node / ISA instruction to guide extraction quality.
- Revisit the `alpha_inv` inverse/projection-operator rewrite idea for expressing alpha injectivity inside the rewrite system. Current choice is a hook because naive inverse/cancellation rules can pollute ISA e-classes with inverse nodes.
- Constants need a dedicated revamp. Current `DetectedConst` nodes are introduced during extraction, which is too late for alpha-based rewriting if constants need to participate as ISA leaves. For this prototype, constants remain deferred; alpha is considered value-transparent and propagates `is_const`, but this may need adjustment during the constant revamp.
- Revisit tiled vs untiled parameter handling during generator backport. The QKV artifact patch wraps raw `Var` with `AlphaHBM` before the existing reshape/bitcast normalization in both paths, but the generator should handle this systematically.
- Backport the working QKV artifact changes into the backend generator once the prototype behavior is proven.

## Open questions

No blocking open questions for the QKV generated-artifact prototype.
