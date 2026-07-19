#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

code_version="$(
  sed -n 's/.*DHRUV_API_VERSION: u32 = \([0-9][0-9]*\).*/\1/p' crates/dhruv_ffi_c/src/lib.rs |
    head -n1
)"
doc_version="$(
  sed -n 's/.*DHRUV_API_VERSION = \([0-9][0-9]*\).*/\1/p' docs/C_ABI_REFERENCE.md |
    head -n1
)"

if [[ -z "$code_version" || -z "$doc_version" ]]; then
  echo "Unable to parse ABI versions from code/docs" >&2
  exit 1
fi

if [[ "$code_version" != "$doc_version" ]]; then
  echo "ABI doc mismatch: code=$code_version docs=$doc_version" >&2
  exit 1
fi

# C-ABI-based wrapper READMEs state their ABI target; keep them synced too.
# (elixir-open is excluded: its NIF calls the Rust crates directly, not the C ABI.)
for readme in bindings/go-open/README.md bindings/node-open/README.md bindings/python-open/README.md; do
  readme_version="$(
    sed -n 's/.*DHRUV_API_VERSION=\([0-9][0-9]*\).*/\1/p' "$readme" | head -n1
  )"
  if [[ -z "$readme_version" ]]; then
    echo "No ABI target found in $readme" >&2
    exit 1
  fi
  if [[ "$readme_version" != "$code_version" ]]; then
    echo "ABI doc mismatch: code=$code_version $readme=$readme_version" >&2
    exit 1
  fi
done

echo "ABI docs match code (DHRUV_API_VERSION=$code_version)."
