#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run.sh PYTHON_FILE [ARGS...]

Run a Python harness from the repository root. This is intentionally generic:
use it for simulator/test drivers that import generated oracle/backend artifacts.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || $# -lt 1 ]]; then
  usage
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PY_FILE="$1"
shift

cd "${REPO_ROOT}"

if [[ ! -f "${PY_FILE}" ]]; then
  echo "error: Python file not found: ${PY_FILE}" >&2
  exit 1
fi

python "${PY_FILE}" "$@"
