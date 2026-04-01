#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-}"
if [[ -z "${MODE}" ]]; then
  echo "usage: $0 <package|publish>" >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

CRATES=(
  jpl_kernel
  dhruv_time
  dhruv_frames
  dhruv_core
  dhruv_vedic_math
  dhruv_tara
  dhruv_vedic_engine
  dhruv_vedic_base
  dhruv_search
  dhruv_vedic_ops
  dhruv_config
  dhruv_rs
)

case "${MODE}" in
  package)
    for crate in "${CRATES[@]}"; do
      cargo package -p "${crate}" --allow-dirty --list >/dev/null
    done
    ;;
  publish)
    if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
      echo "CARGO_REGISTRY_TOKEN must be set for publish mode" >&2
      exit 1
    fi
    for crate in "${CRATES[@]}"; do
      cargo publish -p "${crate}" --locked --token "${CARGO_REGISTRY_TOKEN}"
      sleep 30
    done
    ;;
  *)
    echo "unsupported mode: ${MODE}" >&2
    exit 1
    ;;
esac
