#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/build-backend.sh [TARGET]

Build the generated backend at targets/<TARGET>/backend and copy the resulting
binary to backends/<TARGET>.

Default TARGET: QKV
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET="${1:-QKV}"
BACKEND_DIR="${REPO_ROOT}/targets/${TARGET}/backend"
OUT_DIR="${REPO_ROOT}/backends"

if [[ ! -d "${BACKEND_DIR}" ]]; then
  echo "error: generated backend not found: ${BACKEND_DIR}" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"

(
  cd "${BACKEND_DIR}"
  cargo build --release
)

cp "${BACKEND_DIR}/target/release/backend" "${OUT_DIR}/${TARGET}"
chmod +x "${OUT_DIR}/${TARGET}"

echo "Built backend: backends/${TARGET}"
