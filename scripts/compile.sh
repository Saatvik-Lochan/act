#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/compile.sh --input FILE.hlo --output FILE.py [--target TARGET] [--log DIR]

Compile an HLO file to generated accelerator assembly using backends/<TARGET>.

Defaults:
  TARGET: QKV
  LOG:    backend default, if omitted
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TARGET="QKV"
INPUT=""
OUTPUT=""
LOG_DIR=""

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
    -i|--input)
      INPUT="$2"
      shift 2
      ;;
    -o|--output)
      OUTPUT="$2"
      shift 2
      ;;
    -l|--log)
      LOG_DIR="$2"
      shift 2
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "${INPUT}" || -z "${OUTPUT}" ]]; then
  echo "error: --input and --output are required" >&2
  usage >&2
  exit 1
fi

cd "${REPO_ROOT}"

BACKEND="backends/${TARGET}"
if [[ ! -x "${BACKEND}" ]]; then
  echo "error: backend not found or not executable: ${BACKEND}" >&2
  echo "hint: run scripts/generate.sh or scripts/build-backend.sh ${TARGET}" >&2
  exit 1
fi

mkdir -p "$(dirname "${OUTPUT}")"

ARGS=("--input" "${INPUT}" "--output" "${OUTPUT}")
if [[ -n "${LOG_DIR}" ]]; then
  ARGS+=("--log" "${LOG_DIR}")
fi

"./${BACKEND}" "${ARGS[@]}"
