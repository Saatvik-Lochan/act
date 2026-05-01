# TARGET Changes Summary

This document summarizes the changes in `target-changes.diff`, grouped by
whether they appear ISA-parameterized, not ISA-parameterized, or potentially
parameterized.

## ISA-parameterized

### 1. Alpha-space bridge operators in the e-graph IR
1.1. Added three new e-graph operators in `targets/QKV/backend/src/ir/egraph.rs`:
- 1.1.1. `AlphaHBM(Id)`
- 1.1.2. `AlphaD1(Id)`
- 1.1.3. `AlphaD2(Id)`
1.2. Extended the IR operator machinery for those operators:
- 1.2.1. arity accounting in `TensorOp::num_children`
- 1.2.2. immutable/mutable child access in `Language::children` and `children_mut`
- 1.2.3. parser support in `FromOp` for `alpha-hbm`, `alpha-d1`, `alpha-d2`
- 1.2.4. pretty-printing support in `Display` as `alpha_hbm`, `alpha_d1`, `alpha_d2`
1.3. Updated tensor analysis so alpha nodes transparently forward their child metadata instead of introducing new tensor metadata.

### 2. Alpha-aware IR->ISA rewrite structure
2.1. Rewrote the generated rewrite patterns in `targets/QKV/backend/src/isel/rewrites/ir2isa_rewrites.txt` so ISA lowering is expressed through alpha-space transitions rather than direct IR/ISA matching.
2.2. Concretely:
- 2.2.1. `load-rm` / `load-cm` now consume `alpha-hbm` inputs and produce `alpha-d1` values.
- 2.2.2. `store-rm` / `store-cm` now consume `alpha-d1` values and produce `alpha-hbm` values.
- 2.2.3. `mov` now converts `alpha-d2` inputs into `alpha-d1` outputs.
- 2.2.4. `gemm` now consumes `alpha-d1` inputs and produces an `alpha-d2` output.
- 2.2.5. `softmax` now consumes and produces `alpha-d2` values.
2.3. Regenerated `targets/QKV/backend/src/isel/rewrites/ir2isa_rewrites.rs` to match the new alpha-aware rule structure:
- 2.3.1. updated lhs/rhs eclass counts for all affected rules
- 2.3.2. adjusted precondition indexing because alpha wrappers add intermediate eclasses
- 2.3.3. updated metadata generation so rule parameters such as `rows` are attached to the correct rhs node
- 2.3.4. updated shape propagation so both the ISA node and its enclosing alpha node receive the same tensor metadata
- 2.3.5. factored out the repeated “set metadata on ISA rhs node plus alpha rhs node” logic into `set_two_rhs_shapes`
2.4. The affected generated rule helpers are for:
- 2.4.1. `load-rm`
- 2.4.2. `load-cm`
- 2.4.3. `store-rm`
- 2.4.4. `store-cm`
- 2.4.5. `mov`
- 2.4.6. `gemm`
- 2.4.7. `softmax`

## Not ISA-parameterized

### 3. HBM tagging of external parameters at initialization
- Updated `targets/QKV/backend/src/isel/initializer/lib.rs` so `add_parameter`
  inserts `AlphaHBM(var_node)` before the reshape that materializes a parameter
  tensor.
- This was done in both parameter initialization paths present in the file.
- The result is that external tensors enter the e-graph already tagged as
  HBM-resident.

### 4. Alpha-rooted extraction path
- Reworked `targets/QKV/backend/src/isel/extractor/lib.rs` to replace the
  previous fast/slow extraction selection with a SmoothE-based alpha-rooted
  extraction path.
- The extractor now:
  - canonicalizes the root eclass
  - searches that eclass for `AlphaHBM(child)` nodes
  - enforces the invariant that there is at most one such alpha root
  - extracts from the ISA-side child of that `AlphaHBM`
  - returns no `PiiGraph` if the root eclass has no `AlphaHBM`
- Logging was updated accordingly to report the new extractor path and timings.
- The old `inputs`/`limit` parameters are now accepted but unused in this path.

### 5. SmoothE-based extraction backend
- Added `targets/QKV/backend/src/isel/extractor/smoothe.rs` implementing a new
  extraction path backed by SmoothE.
- This file adds support for:
  - serializing the egg e-graph into `egraph-serialize` / extraction-gym JSON
    format
  - marking the chosen root eclass in the serialized e-graph
  - invoking SmoothE via `conda run -n smoothe python -m src.train --input_file
    ... --acyclic`
  - reading SmoothE’s JSON solution file
  - decoding selected enode ids of the form `<eclass>.<index>`
  - reconstructing a `PiiGraph` from those selected enodes while preserving
    tensor metadata and HBM offsets
  - detecting malformed selections, missing choices, out-of-bounds node
    selections, and cyclic selections
- Added `mod smoothe;` in `targets/QKV/backend/src/isel/extractor/mod.rs` so
  this extractor is compiled.

### 6. Alternative simple alpha extractor
- Added `targets/QKV/backend/src/isel/extractor/alpha.rs` as a simpler
  alpha-aware extractor.
- This extractor:
  - finds `AlphaHBM(child)` roots in the final eclass
  - enforces uniqueness of that alpha root
  - extracts the best `RecExpr` below the ISA-side child using a unit-cost egg
    extractor
  - converts the extracted `RecExpr` into a `PiiGraph`
  - returns no graphs if no `AlphaHBM` root exists
- Added `mod alpha;` in `targets/QKV/backend/src/isel/extractor/mod.rs` so this
  extractor module is compiled.
- Although present, this is not the extraction path currently wired into
  `extract()`.

### 7. Shared extractor utilities
- Extended `targets/QKV/backend/src/isel/extractor/utils.rs` with shared
  functionality used by the alpha extractors:
  - imported `HashMap` and `PiiGraph`
  - added `recexpr_to_pii(...)` to reconstruct a `PiiGraph` from an extracted
    `RecExpr`
  - during reconstruction, recovered the source eclass for each extracted
    enode, copied eclass analysis metadata, and attached the correct HBM offset
- Existing constant-detection and HBM-offset helper logic was otherwise
  preserved.

### 8. Build / dependency / local-build hygiene
- Added `egraph-serialize = { version = "0.3.0", features = ["serde"] }` to
  `targets/QKV/backend/Cargo.toml`.
- Updated `targets/QKV/backend/Cargo.lock` to include the new direct dependency
  and its transitive dependencies:
  - `egraph-serialize`
  - `ordered-float`
  - `rand`
  - `rand_core`
  - the necessary `serde` / `indexmap` feature-resolution changes induced by
    that dependency
- Added `targets/QKV/backend/cpp/malloc/.gitignore` with `build/` so local
  allocator build outputs remain untracked.

## Potentially parameterized

### 9. Alpha injectivity enforcement hook
- Added `targets/QKV/backend/src/isel/rewrites/alpha.rs` with
  `enforce_alpha_injectivity(...)`.
- This pass scans eclasses for multiple alpha nodes of the same space:
  - multiple `AlphaHBM(...)` nodes in one eclass
  - multiple `AlphaD1(...)` nodes in one eclass
  - multiple `AlphaD2(...)` nodes in one eclass
- When such duplicates exist, it unions their children so each alpha-space
  wrapper behaves injectively over values.
- The helper `add_child_unions(...)` collects the pairwise unions needed for
  one alpha-space bucket.
- Added `mod alpha;` and `pub use alpha::enforce_alpha_injectivity;` in
  `targets/QKV/backend/src/isel/rewrites/mod.rs` so the hook is available to
  the pipeline.
- Updated `targets/QKV/backend/src/pipeline.rs` to run this hook inside the
  rewrite runner hook, print when it changed the e-graph, and rebuild via the
  hook implementation when unions occur.
