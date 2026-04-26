#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/generate.sh [ISA_SPEC.py]

Run a TAIDL ISA spec from the repository root. The spec controls what is
produced, e.g. oracle/backend generation via qkv.generate_oracle() /
qkv.generate_backend().

Default ISA_SPEC.py: QKV.py
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SPEC="${1:-QKV.py}"

cd "${REPO_ROOT}"

if [[ ! -f "${SPEC}" ]]; then
  echo "error: ISA spec not found: ${SPEC}" >&2
  exit 1
fi

python "${SPEC}"
