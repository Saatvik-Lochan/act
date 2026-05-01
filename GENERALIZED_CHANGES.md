# Generalized Changes for Generator Backport

This document restates the **ISA-parameterized** target-artifact changes in the form we want to implement in `generators/backend`.

## 1. Alpha-space bridge operators in the e-graph IR

### 1.1. Generate alpha operators from data models
- Source of truth: `data_models`.
- Emit one alpha `TensorOp` variant for every data model:
  - `d0` becomes `AlphaHBM(Id)`.
  - `dN` becomes `AlphaD<N>(Id)`.
- These are generated into `egraph.rs`; they are not handwritten into target artifacts.
- Concrete generator changes:
  - pass `data_models` into `generate_egraph_rs_file(...)`
  - derive alpha variant names/op names in `ir_egraph_rs_generator.py`

### 1.2. Generate normal e-graph support for alpha operators
- Add separate `ALPHA_*` placeholders in `generic/src/ir/egraph.rs`; do not reuse `ISA_*` placeholders because alpha nodes are `Id`-shaped, not `[Id; N]` instruction-shaped.
- Generate alpha arms for:
  - enum variants
  - `TensorOp::num_children`
  - `Language::children`
  - `Language::children_mut`
  - `FromOp`
  - `Display`
- Alpha nodes carry no metadata and need no `set_metadata` arm.
- Parser/display names should follow the generator’s normal emitted-op style, e.g. `alpha-hbm`, `alpha-d1`.

### 1.3. Generate alpha tensor-analysis pass-through
- In `Analysis<TensorOp> for TensorInfo`, generated alpha nodes should forward child metadata:
  - `AlphaX(child) => egraph[*child].data.clone()`
- Emit this as a combined match arm over all generated alpha variants.
- `DetectedConst` handling remains as today.

## 2. Alpha-aware IR->ISA rewrite text generation

### 2.1. Expose instruction buffers to rewrite generation
- Extend `InstructionMetadata` in `taidl/accelerator.py` with:
  - `instr_inputs`
  - `instr_outputs`
- Populate those fields from each `Instruction`.
- Update `rhs_size` to account for alpha-wrapped RHS:
  - old: `len(instr_inputs) + 1`
  - new: `len(instr_inputs) + 2`

### 2.2. Wrap LHS parameters for textual rewrite rules
- File: `generators/backend/isel_ir2isa_rewrites_txt_generator.py`.
- This affects only the emitted `ir2isa_rewrites.txt` pattern strings.
- Pass parameter-index-to-alpha-op information into the txt generator’s `RewriteRuleVisitor`.
- When visiting `parameter(n)`:
  - still allocate/store the raw variable in `parameter_vars` for RHS use
  - set `variable_definitions[%In]` to `(alpha-for-input-buffer ?var)` for LHS string construction
- Result: the textual LHS pattern contains alpha wrappers, while the textual RHS ISA operands still use raw variables.
- This visitor does not generate Rust shape/precondition code; that is handled separately in section 3.1.
- Note: section 2.2 and section 3.1 use different visitor classes, but they parse the same semantics and must stay conceptually aligned: 2.2 expands the textual pattern, while 3.1 expands the Rust helper assignment/eclass model.

### 2.3. Wrap RHS result in `generate_rewrite_rule(...)`
- Build the raw RHS ISA node exactly as before, using raw `parameter_vars`.
- Then wrap that raw RHS with the alpha op for `metadata.instr_outputs[0][0]`.
- Example shape:
  - old: `(gemm ?a ?b)`
  - new: `(alpha-d2 (gemm ?a ?b))`

## 3. Alpha-aware `ir2isa_rewrites.rs` helper generation

### 3.1. Expand LHS assignment model for generated Rust helpers
- File: `generators/backend/isel_ir2isa_rewrites_rs_generator.py`.
- This affects only the generated Rust helper functions in `ir2isa_rewrites.rs`, especially `precond_*` LHS eclass counts and shape checks.
- This file has a separate `RustGeneratorVisitor`; it is not the same visitor as section 2.2.
- In `RustGeneratorVisitor`, when visiting `parameter(n)`, create two assignment records:
  - raw parameter assignment
  - alpha-wrapped parameter assignment with the same dtype/shape
- Later semantic uses should refer to the alpha assignment record.
- Result: generated `precond_*` functions expect/check both the raw parameter eclass and the alpha parameter eclass from the expanded textual LHS pattern.

### 3.2. Place RHS metadata on the ISA node, not the alpha node
- Current metadata-with-attrs template writes metadata to `last_mut()`.
- After alpha wrapping, `last()` is the alpha wrapper; the ISA node is second-last.
- Change the metadata-with-attrs template to write to `rhs_metadata[len - 2]`.
- No-attr metadata generation remains unchanged except for the larger `rhs_size`.

### 3.3. Set shapes on both RHS ISA node and alpha wrapper
- Current shape-setting template sets shape only on `rhs_eclasses.last()`.
- After alpha wrapping:
  - second-last RHS eclass is the ISA node
  - last RHS eclass is the alpha wrapper
- Change the shape-setting template to set the same `TensorInfo` on both second-last and last.
- This requires no new template arguments; both positions can be computed from `rhs_eclasses.len()`.

## 4. Explicit non-changes

### 4.1. No alpha changes to `buffer.rs`
- The SmoothE extraction path does not use `buffer_assignment`.
- Alpha nodes do not need buffer assignments.

### 4.2. No alpha changes to `applier.rs`
- `applier.rs` already derives RHS eclass order from the actual RHS pattern.
- It only requires metadata vector length to match RHS length.
- Returning `rhs_eclasses.last()` is now correct because the alpha wrapper is the RHS root.

## 5. Non-ISA-parameterized changes to backport in the same pass

### 5.1. SmoothE dependency and extraction scaffolding
- Add `egraph-serialize` to `generators/backend/generic/Cargo.toml` and refresh `Cargo.lock`.
- Add `generators/backend/generic/cpp/malloc/.gitignore` with `build/`.
- Backport SmoothE/alpha-root extraction changes into the generic backend:
  - `generic/src/isel/extractor/lib.rs`
  - `generic/src/isel/extractor/mod.rs`
  - new `generic/src/isel/extractor/smoothe.rs`
  - new `generic/src/isel/extractor/alpha.rs`
  - `generic/src/isel/extractor/utils.rs`

### 5.2. HBM tagging for external parameters
- Backport initializer change into `generic/src/isel/initializer/lib.rs`.
- Parameters should be wrapped with generated `AlphaHBM(...)` before reshape, using the global convention that external inputs/outputs live in HBM.

### 5.3. Alpha injectivity hook
- Backport alpha injectivity enforcement into the generic backend:
  - new `generic/src/isel/rewrites/alpha.rs` with `ALPHA_*` placeholders
  - update `generic/src/isel/rewrites/mod.rs`
  - update `generic/src/pipeline.rs`
- Use placeholder-based generation, not whole-file generation.
- Fill the injectivity match/bucket logic from `data_models`, matching the generated alpha variants.

## 6. Implementation order

1. Backport non-ISA generic changes:
   - SmoothE/alpha extractor files
   - extractor utility additions
   - HBM parameter tagging
   - Cargo dependency / lockfile
   - `cpp/malloc/.gitignore`
2. Add shared alpha helper utilities in `generators/backend/alpha_utils.py`.
3. Generate alpha e-graph operators/support/analysis from `data_models`.
4. Add placeholder-based alpha injectivity generation from `data_models`.
5. Generate alpha-aware `ir2isa_rewrites.txt`.
6. Generate matching alpha-aware `ir2isa_rewrites.rs` helpers.
7. Regenerate QKV and compare against the current target artifact changes.
