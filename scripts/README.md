# Scripts

Root-level helpers for the core ACT flow.

## Docker environment

```bash
scripts/docker.sh --sim       # simulator/oracle environment
scripts/docker.sh --compile   # backend/compiler environment
scripts/docker.sh --setup     # pull Docker images
```

Run one command inside Docker:

```bash
scripts/docker.sh --compile -- scripts/generate.sh QKV.py
```

## Generate from ISA spec

```bash
scripts/generate.sh QKV.py
```

Runs the TAIDL spec from the repo root. The spec decides whether oracle/backend artifacts are generated.

## Build generated backend

```bash
scripts/build-backend.sh QKV
```

Builds `targets/QKV/backend` and copies the binary to `backends/QKV`.

## Compile HLO

```bash
scripts/compile.sh --input attention.hlo --output asm/compiled_qkv.py
```

Uses `backends/QKV` by default. Override target with:

```bash
scripts/compile.sh --target QKV --input attention.hlo --output asm/compiled_qkv.py
```

## Run Python harness

```bash
scripts/run.sh test_qkv.py
```

Runs a Python simulator/test harness from the repo root.

## Clean

```bash
scripts/clean.sh --target QKV        # clean backend build artifacts
scripts/clean.sh --target QKV --full # also remove generated backend/oracle outputs
```
