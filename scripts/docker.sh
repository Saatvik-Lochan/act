#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/docker.sh --sim|--compile|--setup [-- COMMAND...]

Launch the locally built ACT Docker environment from the repository root.
--sim and --compile currently use the same core ACT image; the flags are kept
for compatibility with existing root-level script usage.

If COMMAND is provided, it is executed non-interactively inside the container;
otherwise an interactive shell is opened.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ARCH="$(uname -m)"
MODE=""
CMD=()

case "${ARCH}" in
  x86_64)
    IMAGE_NAME="devanshdvj/act:latest-amd64"
    ;;
  arm64|aarch64)
    IMAGE_NAME="devanshdvj/act:latest-arm64"
    ;;
  *)
    echo "error: unsupported architecture: ${ARCH}" >&2
    exit 1
    ;;
esac

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --sim)
      MODE="sim"
      shift
      ;;
    --compile)
      MODE="compile"
      shift
      ;;
    --setup)
      MODE="setup"
      shift
      ;;
    --)
      shift
      CMD=("$@")
      break
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "${MODE}" ]]; then
  echo "error: choose --sim, --compile, or --setup" >&2
  usage >&2
  exit 1
fi

if [[ "${MODE}" == "setup" ]]; then
  if docker image inspect "${IMAGE_NAME}" >/dev/null 2>&1; then
    echo "Found local ACT image: ${IMAGE_NAME}"
    exit 0
  fi
  echo "error: local ACT image not found: ${IMAGE_NAME}" >&2
  echo "Build it with: docker/build.sh" >&2
  exit 1
fi

if ! docker image inspect "${IMAGE_NAME}" >/dev/null 2>&1; then
  echo "error: local ACT image not found: ${IMAGE_NAME}" >&2
  echo "Build it first with: docker/build.sh" >&2
  exit 1
fi

DOCKER_ARGS=(-it --rm)
if [[ ${#CMD[@]} -gt 0 ]]; then
  DOCKER_ARGS=(-i --rm)
fi

CONTAINER_NAME="act-rm-$(id -un)-${MODE}"

docker run "${DOCKER_ARGS[@]}" \
  --gpus all \
  --name "${CONTAINER_NAME}" \
  -v "${REPO_ROOT}:/workspace:rw" \
  -w "/workspace" \
  -e HOST_UID="$(id -u)" \
  -e HOST_GID="$(id -g)" \
  "${IMAGE_NAME}" \
  "${CMD[@]}"
