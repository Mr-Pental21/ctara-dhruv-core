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
  # No single export-listing tool is reliably on PATH (or prints the COFF
  # export table) across Windows runners, so accept a match from ANY
  # available tool; the stdlib-only PE parser is the guaranteed fallback.
  symbol_word="(^|[^A-Za-z0-9_])${symbol}([^A-Za-z0-9_]|\$)"
  found=""
  tried=()
  if command -v dumpbin >/dev/null 2>&1; then
    tried+=(dumpbin)
    dumpbin /exports "$artifact" 2>/dev/null | grep -E -q "$symbol_word" && found=dumpbin || true
  fi
  if [[ -z "$found" ]] && command -v llvm-readobj >/dev/null 2>&1; then
    tried+=(llvm-readobj)
    llvm-readobj --coff-exports "$artifact" 2>/dev/null | grep -E -q "$symbol_word" && found=llvm-readobj || true
  fi
  if [[ -z "$found" ]] && command -v llvm-objdump >/dev/null 2>&1; then
    tried+=(llvm-objdump)
    llvm-objdump -p "$artifact" 2>/dev/null | grep -E -q "$symbol_word" && found=llvm-objdump || true
  fi
  if [[ -z "$found" ]] && command -v objdump >/dev/null 2>&1; then
    tried+=(objdump)
    objdump -p "$artifact" 2>/dev/null | grep -E -q "$symbol_word" && found=objdump || true
  fi
  for py in python3 python; do
    if [[ -z "$found" ]] && command -v "$py" >/dev/null 2>&1; then
      tried+=("$py")
      "$py" scripts/ci/pe_exports.py "$artifact" 2>/dev/null | grep -q "^${symbol}\$" && found="$py" || true
    fi
  done
  if [[ -n "$found" ]]; then
    echo "Verified exported symbol '$symbol' in $artifact (via $found)"
  elif [[ ${#tried[@]} -eq 0 ]]; then
    echo "Symbol export check skipped for Windows artifact (no export-listing tool available)."
  else
    echo "Expected exported symbol '$symbol' not found in $artifact (tools tried: ${tried[*]})" >&2
    exit 1
  fi
else
  echo "Symbol export check skipped (nm not available)."
fi
