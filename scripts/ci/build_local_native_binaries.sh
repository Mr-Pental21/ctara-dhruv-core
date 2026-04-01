#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

VERSION="$(python3 - <<'PY'
import tomllib
from pathlib import Path
with (Path('Cargo.toml')).open('rb') as handle:
    data = tomllib.load(handle)
print(data['workspace']['package']['version'])
PY
)"

echo "Building optimized local binaries for version ${VERSION}"

cargo build -p dhruv_cli --release
cargo build -p dhruv_ffi_c --release

python3 bindings/python-open/scripts/prepare_native_lib.py

cargo build --release --manifest-path bindings/elixir-open/native/dhruv_elixir_nif/Cargo.toml
case "$(uname -s)" in
  Darwin)
    ELIXIR_NIF_NAME="libdhruv_elixir_nif.dylib"
    ELIXIR_DST="bindings/elixir-open/priv/native/dhruv_elixir_nif.so"
    ;;
  Linux)
    ELIXIR_NIF_NAME="libdhruv_elixir_nif.so"
    ELIXIR_DST="bindings/elixir-open/priv/native/dhruv_elixir_nif.so"
    ;;
  *)
    ELIXIR_NIF_NAME="dhruv_elixir_nif.dll"
    ELIXIR_DST="bindings/elixir-open/priv/native/dhruv_elixir_nif.dll"
    ;;
esac
cp "target/release/${ELIXIR_NIF_NAME}" "${ELIXIR_DST}"

mkdir -p dist/local
python3 scripts/ci/package_release_asset.py \
  --kind cli \
  --platform "$(python3 - <<'PY'
import platform
system = platform.system()
print('macos' if system == 'Darwin' else 'windows' if system == 'Windows' else 'linux')
PY
)" \
  --arch "$(python3 - <<'PY'
import platform
machine = platform.machine().lower()
print('arm64' if machine in ('arm64', 'aarch64') else 'x64')
PY
)" \
  --version "${VERSION}" \
  --input-root target/release \
  --output-dir dist/local

python3 scripts/ci/package_release_asset.py \
  --kind cabi \
  --platform "$(python3 - <<'PY'
import platform
system = platform.system()
print('macos' if system == 'Darwin' else 'windows' if system == 'Windows' else 'linux')
PY
)" \
  --arch "$(python3 - <<'PY'
import platform
machine = platform.machine().lower()
print('arm64' if machine in ('arm64', 'aarch64') else 'x64')
PY
)" \
  --version "${VERSION}" \
  --input-root target/release \
  --output-dir dist/local

echo "Optimized local binaries updated:"
echo "  CLI: target/release/dhruv_cli"
echo "  C ABI: target/release/$(python3 - <<'PY'
import platform
system = platform.system()
if system == 'Darwin':
    print('libdhruv_ffi_c.dylib')
elif system == 'Windows':
    print('dhruv_ffi_c.dll')
else:
    print('libdhruv_ffi_c.so')
PY
)"
echo "  Python bundled lib: bindings/python-open/src/ctara_dhruv/$(python3 - <<'PY'
import platform
system = platform.system()
if system == 'Darwin':
    print('libdhruv_ffi_c.dylib')
elif system == 'Windows':
    print('dhruv_ffi_c.dll')
else:
    print('libdhruv_ffi_c.so')
PY
)"
echo "  Elixir NIF: ${ELIXIR_DST}"
echo "  Local release zips: dist/local/"
