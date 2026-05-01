# Generalized Changes for Generator Backport

This document restates the **ISA-parameterized** changes from `TARGET-CHANGES.md` in a generalized form suitable for implementation in `generators/backend`.

## 1. Generalized alpha-space bridge operators in the e-graph IR

### 1.1. Generate alpha-space bridge operators in the emitted e-graph IR
- The generator should emit the alpha bridge operators:
  - `AlphaHBM(Id)`
  - `AlphaD1(Id)`
  - `AlphaD2(Id)`
- These should be generated into the backend artifact rather than added manually afterward.

### 1.2. Generate the e-graph support machinery for those bridge operators
For the generated alpha operators, the generator should also emit:
- 1.2.1. arity accounting in `TensorOp::num_children`
- 1.2.2. immutable/mutable child access in `Language::children` and `children_mut`
- 1.2.3. parser support in `FromOp`
- 1.2.4. pretty-printing support in `Display`

### 1.3. Generate tensor-analysis pass-through for alpha operators
- The generated analysis rule should recognize `AlphaHBM`, `AlphaD1`, and `AlphaD2`.
- Each alpha bridge node should forward its child `TensorInfo`.
- Alpha nodes should not introduce new shape/dtype/const metadata of their own.

## 2. Generalized alpha-aware IR->ISA rewrite structure

### 2.1. Generate alpha-aware rewrite rules from existing ISA rewrite rules plus buffer information
- Start from the existing generated IR->ISA rewrite rules.
- Transform those generated rules so they include alpha wrappers derived from instruction buffer information.
- This should be generated in `ir2isa_rewrites.txt`, not patched manually into the emitted target.

### 2.2. Generate alpha wrapping from instruction input/output buffers
- For each generated IR->ISA rewrite rule:
  - 2.2.1. wrap each LHS operand variable with the alpha node corresponding to that operand’s instruction input buffer
  - 2.2.2. wrap the RHS ISA node with the alpha node corresponding to the instruction output buffer
- This requires the generator to use:
  - 2.2.3. the buffer of each instruction input operand
  - 2.2.4. the buffer of the instruction output
- No additional transition inference is needed beyond the existing rewrite rule plus this buffer information.

### 2.3. Generate matching rewrite helper code for the alpha-expanded rules
- Once alpha wrappers are inserted into the generated rewrite rules, the generator must also emit matching helper code in `ir2isa_rewrites.rs`.
- Concretely:
  - 2.3.1. lhs/rhs eclass counts must match the alpha-expanded rule shape
  - 2.3.2. precondition indexing must refer to the correct eclasses after alpha nodes are inserted
  - 2.3.3. metadata placement must target the correct rhs node positions in the alpha-expanded rule
  - 2.3.4. shape propagation must assign tensor metadata to both the ISA rhs node and its enclosing alpha rhs node when both represent the same tensor value
  - 2.3.5. helper logic for this duplicated metadata/shape assignment should be generated rather than patched manually
