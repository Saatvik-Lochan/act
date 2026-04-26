#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/docker.sh --sim|--compile|--setup [-- COMMAND...]

Launch the ACT Docker environment from the repository root. If COMMAND is
provided, it is executed non-interactively inside the container; otherwise an
interactive shell is opened.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ARCH="$(uname -m)"
MODE=""
CMD=()

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
  echo "Pulling ACT Docker images..."
  if [[ "${ARCH}" == "aarch64" || "${ARCH}" == "arm64" ]]; then
    docker pull devanshdvj/act-tutorials:asplos26-arm64
    docker pull devanshdvj/act-tutorials:asplos26-amd64
  else
    docker pull devanshdvj/act-tutorials:asplos26-amd64
  fi
  exit 0
fi

PLATFORM_FLAG=()
if [[ "${ARCH}" == "aarch64" || "${ARCH}" == "arm64" ]]; then
  if [[ "${MODE}" == "sim" ]]; then
    IMAGE_NAME="devanshdvj/act-tutorials:asplos26-arm64"
  else
    IMAGE_NAME="devanshdvj/act-tutorials:asplos26-amd64"
    PLATFORM_FLAG=(--platform linux/amd64)
  fi
else
  IMAGE_NAME="devanshdvj/act-tutorials:asplos26-amd64"
fi

if ! docker image inspect "${IMAGE_NAME}" >/dev/null 2>&1; then
  docker pull "${IMAGE_NAME}"
fi

DOCKER_ARGS=(-it --rm)
if [[ ${#CMD[@]} -gt 0 ]]; then
  DOCKER_ARGS=(-i --rm)
fi

CONTAINER_NAME="act-$(whoami)-${MODE}"

docker run "${DOCKER_ARGS[@]}" \
  --name "${CONTAINER_NAME}" \
  "${PLATFORM_FLAG[@]}" \
  -v "${REPO_ROOT}:/workspace:rw" \
  -w "/workspace" \
  -e HOST_UID="$(id -u)" \
  -e HOST_GID="$(id -g)" \
  "${IMAGE_NAME}" \
  "${CMD[@]}"
