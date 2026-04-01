#!/usr/bin/env python3
from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
PACKAGE_DIR = Path(__file__).resolve().parents[1] / "src" / "ctara_dhruv"


def shared_library_name() -> str:
    if sys.platform == "darwin":
        return "libdhruv_ffi_c.dylib"
    if sys.platform == "win32":
        return "dhruv_ffi_c.dll"
    return "libdhruv_ffi_c.so"


def main() -> int:
    cargo = shutil.which("cargo")
    if cargo is None:
        raise SystemExit(
            "cargo not found in PATH; install Rust toolchain before building Python wheels"
        )

    subprocess.run(
        [
            cargo,
            "build",
            "-p",
            "dhruv_ffi_c",
            "--release",
            "--manifest-path",
            str(ROOT / "Cargo.toml"),
        ],
        check=True,
        cwd=ROOT,
    )

    lib_name = shared_library_name()
    built_lib = ROOT / "target" / "release" / lib_name
    if not built_lib.is_file():
        raise SystemExit(f"expected built library at {built_lib}")

    PACKAGE_DIR.mkdir(parents=True, exist_ok=True)
    bundled_lib = PACKAGE_DIR / lib_name
    shutil.copy2(built_lib, bundled_lib)
    print(bundled_lib)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
