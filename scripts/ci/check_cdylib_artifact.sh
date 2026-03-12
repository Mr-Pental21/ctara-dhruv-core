#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

cargo build -p dhruv_ffi_c --release

os="$(uname -s)"
case "$os" in
  Linux*)
    artifact="target/release/libdhruv_ffi_c.so"
    ;;
  Darwin*)
    artifact="target/release/libdhruv_ffi_c.dylib"
    ;;
  MINGW* | MSYS* | CYGWIN* | Windows_NT)
    artifact="target/release/dhruv_ffi_c.dll"
    ;;
  *)
    echo "Unsupported OS for cdylib artifact check: $os" >&2
    exit 1
    ;;
esac

if [[ ! -f "$artifact" ]]; then
  echo "Expected cdylib artifact not found: $artifact" >&2
  echo "Contents of target/release for debugging:" >&2
  ls -la target/release >&2
  exit 1
fi

echo "Found cdylib artifact: $artifact"

symbol="dhruv_api_version"
symbol_regex="(^|[[:space:]])_?${symbol}$"

if [[ "$artifact" == *.so ]] && command -v nm >/dev/null 2>&1; then
  if ! nm -D --defined-only "$artifact" | grep -E -q "$symbol_regex"; then
    echo "Expected exported symbol '$symbol' not found in $artifact" >&2
    exit 1
  fi
  echo "Verified exported symbol '$symbol' in $artifact"
elif [[ "$artifact" == *.dylib ]] && command -v nm >/dev/null 2>&1; then
  if ! nm -gU "$artifact" | grep -E -q "$symbol_regex"; then
    echo "Expected exported symbol '$symbol' not found in $artifact" >&2
    exit 1
  fi
  echo "Verified exported symbol '$symbol' in $artifact"
elif [[ "$artifact" == *.dll ]]; then
  if command -v dumpbin >/dev/null 2>&1; then
    if ! dumpbin /exports "$artifact" | grep -E -q "\\<${symbol}\\>"; then
      echo "Expected exported symbol '$symbol' not found in $artifact" >&2
      exit 1
    fi
    echo "Verified exported symbol '$symbol' in $artifact"
  elif command -v llvm-objdump >/dev/null 2>&1; then
    if ! llvm-objdump -p "$artifact" | grep -E -q "\\<${symbol}\\>"; then
      echo "Expected exported symbol '$symbol' not found in $artifact" >&2
      exit 1
    fi
    echo "Verified exported symbol '$symbol' in $artifact"
  elif command -v objdump >/dev/null 2>&1; then
    if ! objdump -p "$artifact" | grep -E -q "\\<${symbol}\\>"; then
      echo "Expected exported symbol '$symbol' not found in $artifact" >&2
      exit 1
    fi
    echo "Verified exported symbol '$symbol' in $artifact"
  else
    echo "Symbol export check skipped for Windows artifact (no dumpbin/llvm-objdump/objdump in PATH)."
  fi
else
  echo "Symbol export check skipped (nm not available)."
fi
