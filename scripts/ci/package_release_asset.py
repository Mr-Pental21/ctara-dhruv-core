#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import pathlib
import shutil
import tempfile
import zipfile


ROOT = pathlib.Path(__file__).resolve().parents[2]


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cli_binary_name(platform: str) -> str:
    return "dhruv_cli.exe" if platform == "windows" else "dhruv_cli"


def shared_library_name(platform: str) -> str:
    if platform == "windows":
        return "dhruv_ffi_c.dll"
    if platform == "macos":
        return "libdhruv_ffi_c.dylib"
    return "libdhruv_ffi_c.so"


def static_library_candidates(platform: str) -> list[str]:
    if platform == "windows":
        return ["dhruv_ffi_c.lib"]
    return ["libdhruv_ffi_c.a"]


def package_cli(args: argparse.Namespace) -> pathlib.Path:
    binary = pathlib.Path(args.input_root) / cli_binary_name(args.platform)
    if not binary.exists():
        raise SystemExit(f"Missing CLI binary: {binary}")

    archive_name = f"dhruv_cli-v{args.version}-{args.platform}-{args.arch}.zip"
    archive_path = pathlib.Path(args.output_dir) / archive_name

    with tempfile.TemporaryDirectory() as temp_dir:
        staging = pathlib.Path(temp_dir)
        bin_dir = staging / "bin"
        bin_dir.mkdir(parents=True)
        shutil.copy2(binary, bin_dir / binary.name)
        manifest = staging / "SHA256SUMS.txt"
        manifest.write_text(
            f"{sha256(bin_dir / binary.name)}  bin/{binary.name}\n",
            encoding="utf-8",
        )
        make_zip(staging, archive_path)

    return archive_path


def package_cabi(args: argparse.Namespace) -> pathlib.Path:
    input_root = pathlib.Path(args.input_root)
    include_dir = ROOT / "crates" / "dhruv_ffi_c" / "include"
    shared = input_root / shared_library_name(args.platform)
    if not shared.exists():
        raise SystemExit(f"Missing shared library: {shared}")

    archive_name = f"dhruv_c_abi-v{args.version}-{args.platform}-{args.arch}.zip"
    archive_path = pathlib.Path(args.output_dir) / archive_name

    with tempfile.TemporaryDirectory() as temp_dir:
        staging = pathlib.Path(temp_dir)
        staged_include = staging / "include"
        staged_lib = staging / "lib"
        staged_include.mkdir(parents=True)
        staged_lib.mkdir(parents=True)

        shutil.copy2(include_dir / "dhruv.h", staged_include / "dhruv.h")
        copied = [staged_include / "dhruv.h"]

        shutil.copy2(shared, staged_lib / shared.name)
        copied.append(staged_lib / shared.name)

        for candidate in static_library_candidates(args.platform):
            src = input_root / candidate
            if src.exists():
                shutil.copy2(src, staged_lib / src.name)
                copied.append(staged_lib / src.name)

        manifest = staging / "SHA256SUMS.txt"
        lines = [f"{sha256(path)}  {path.relative_to(staging).as_posix()}" for path in copied]
        manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")
        make_zip(staging, archive_path)

    return archive_path


def make_zip(staging: pathlib.Path, output_path: pathlib.Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(staging.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(staging))


def main() -> int:
    parser = argparse.ArgumentParser(description="Package CLI/C ABI release artifacts.")
    parser.add_argument("--kind", choices=["cli", "cabi"], required=True)
    parser.add_argument("--platform", choices=["linux", "macos", "windows"], required=True)
    parser.add_argument("--arch", choices=["x64", "arm64"], required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--input-root", required=True)
    parser.add_argument("--output-dir", required=True)
    args = parser.parse_args()

    if args.kind == "cli":
        archive = package_cli(args)
    else:
        archive = package_cabi(args)

    print(archive)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
