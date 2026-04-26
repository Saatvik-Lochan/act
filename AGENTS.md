# AGENTS.md

## Repository overview
This is the top-level ACT repository. It ties together:
- `taidl/`: the ISA DSL and language docs
- `generators/backend/`: backend generator source
- `generators/oracle/`: oracle generator source
- `targets/QKV/`: generated backend/oracle artifacts for the QKV example
- `tutorials/asplos26/`: the main usage guide and exercises

For most tasks in this repo, the source of truth is the TAIDL spec in `QKV.py` and the tutorial flow in `tutorials/asplos26/README.md`.

## What to focus on here
Prioritize the backend + e-graph pipeline over the oracle side unless the task explicitly asks for oracle changes.

Backend/e-graph flow to keep in mind:
1. TAIDL ISA spec (`QKV.py`) defines buffers, instructions, and HLO semantics.
2. Backend generation emits the e-graph rewrite tables and allocator inputs under `targets/QKV/backend/`.
3. Phase 1 in Rust does HLO parsing, e-graph rewriting, extraction, and instruction selection.
4. Phase 2 in C++ does memory allocation / constraint solving / code emission.
5. Final assembly is written under `backends/QKV` or a path you pass via `--output`.

## Tutorial usage pattern
The ASPLOS 2026 tutorial under `tutorials/asplos26/` is the best guide for how the repo is used:
- Exercise 1: write/update the TAIDL ISA
- Exercise 2: use the generated kernel API
- Exercise 3: generate backend + compile HLO to assembly
- Exercise 4: modify the ISA and observe the regenerated stack

Typical commands are run from `tutorials/asplos26/`:
- `./copy.sh exercise1|exercise2|exercise3`
- `./docker.sh --sim`
- `./docker.sh --compile`

## Backend/e-graph code map
Key backend files in `targets/QKV/backend/`:
- `src/ir/`: e-graph language, tensor metadata, buffer mapping, IR helpers
- `src/isel/initializer/`: HLO parsing into the e-graph
- `src/isel/rewrites/`: generated rewrite rules (`ir2ir_rewrites.*`, `ir2isa_rewrites.*`)
- `src/isel/extractor/`: fast/slow extraction logic
- `src/malloc/`: Rust-to-C++ bridge for allocation/codegen
- `cpp/malloc/`: scheduler, constraint solver, parser, emitter, and instruction model
- `build.rs`: builds the C++ allocator and links OR-Tools

## Editing guidance
- For the current work, it is OK to edit `targets/QKV/backend/` and its build artifacts directly to iterate faster; just treat those edits as temporary and plan to backport the final behavior into the generator later.
- If you add or rename ISA operations, update the e-graph language and backend mappings together:
  - `src/ir/egraph.rs` (`TensorOp` and parsing/display)
  - `src/ir/buffer.rs` (buffer assignment)
  - rewrite tables in `src/isel/rewrites/*.txt`
  - C++ instruction definitions in `cpp/malloc/include/instructions.h`
  - parser/emitter logic if the assembly format changes
- Keep the backend and e-graph representations in sync; mismatches usually show up as missing rewrites, extraction failures, or allocator/parser errors.

## Generated artifacts
For fast iteration, build artifacts in `targets/QKV/backend/` are fair game right now, including:
- `targets/QKV/backend/target/`
- `targets/QKV/backend/cpp/malloc/build/`
- other local build logs / compiled binaries

When the change is stable, backport it to the generator source instead of keeping only the generated copy.

## Practical debugging order
When something breaks, debug in this order:
1. ISA spec / HLO syntax (`QKV.py`, tutorial HLO)
2. HLO parsing into the e-graph
3. Rewrite coverage and rule matching
4. Extraction / instruction selection
5. Memory allocation / C++ backend
6. Only then inspect oracle behavior if the task needs runtime validation

## Keep in mind
- The oracle is useful for correctness checks, but this repository’s core compiler story is the backend and e-graph lowering pipeline.
- Generated backend behavior is driven by the ISA spec; when in doubt, regenerate from `QKV.py` and compare the diffs under `targets/QKV/backend/`.
