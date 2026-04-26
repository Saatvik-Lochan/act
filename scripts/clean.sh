#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/clean.sh [--target TARGET] [--full]

Clean generated backend build artifacts for a target.

Default behavior is conservative:
  - cargo clean in targets/<TARGET>/backend
  - remove targets/<TARGET>/backend/cpp/malloc/build

With --full, also remove:
  - backends/<TARGET>
  - targets/<TARGET>/backend
  - targets/<TARGET>/oracle

Defaults:
  TARGET: QKV
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET="QKV"
FULL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    -t|--target)
      TARGET="$2"
      shift 2
      ;;
    --full)
      FULL=1
      shift
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

TARGET_DIR="${REPO_ROOT}/targets/${TARGET}"
BACKEND_DIR="${TARGET_DIR}/backend"

if [[ -d "${BACKEND_DIR}" ]]; then
  (
    cd "${BACKEND_DIR}"
    if [[ -f Cargo.toml ]]; then
      cargo clean || true
    fi
  )
  rm -rf "${BACKEND_DIR}/cpp/malloc/build"
fi

if [[ "${FULL}" == "1" ]]; then
  rm -f "${REPO_ROOT}/backends/${TARGET}"
  rm -rf "${TARGET_DIR}/backend" "${TARGET_DIR}/oracle"
fi

echo "Cleaned target: ${TARGET}"
