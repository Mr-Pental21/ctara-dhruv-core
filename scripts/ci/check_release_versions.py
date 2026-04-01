#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]


def cargo_workspace_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as handle:
        data = tomllib.load(handle)
    return data["workspace"]["package"]["version"]


def cargo_package_version(path: pathlib.Path) -> str:
    with path.open("rb") as handle:
        data = tomllib.load(handle)
    version = data["package"]["version"]
    if isinstance(version, dict) and version.get("workspace"):
        return cargo_workspace_version()
    return version


def pyproject_version() -> str:
    with (ROOT / "bindings" / "python-open" / "pyproject.toml").open("rb") as handle:
        data = tomllib.load(handle)
    return data["project"]["version"]


def package_json_version() -> str:
    with (ROOT / "bindings" / "node-open" / "package.json").open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    return data["version"]


def mix_version() -> str:
    text = (ROOT / "bindings" / "elixir-open" / "mix.exs").read_text(encoding="utf-8")
    match = re.search(r'version:\s*"([^"]+)"', text)
    if not match:
        raise SystemExit("Unable to find version in bindings/elixir-open/mix.exs")
    return match.group(1)


def expected_version_from_ref(tag_ref: str | None, explicit_version: str | None) -> str:
    if explicit_version:
        return explicit_version
    if not tag_ref:
        raise SystemExit("Either --version or --github-ref must be provided")
    prefix = "refs/tags/v"
    if not tag_ref.startswith(prefix):
        raise SystemExit(
            f"Expected a release tag ref starting with {prefix!r}, got {tag_ref!r}"
        )
    return tag_ref[len(prefix) :]


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify all public surface versions match the release tag.")
    parser.add_argument("--github-ref", help="Git ref, typically refs/tags/vX.Y.Z")
    parser.add_argument("--version", help="Explicit expected version")
    args = parser.parse_args()

    expected = expected_version_from_ref(args.github_ref, args.version)

    versions = {
        "workspace": cargo_workspace_version(),
        "dhruv_rs": cargo_package_version(ROOT / "crates" / "dhruv_rs" / "Cargo.toml"),
        "dhruv_ffi_c": cargo_package_version(ROOT / "crates" / "dhruv_ffi_c" / "Cargo.toml"),
        "dhruv_cli": cargo_package_version(ROOT / "crates" / "dhruv_cli" / "Cargo.toml"),
        "python": pyproject_version(),
        "node": package_json_version(),
        "elixir": mix_version(),
    }

    mismatches = {
        surface: version for surface, version in versions.items() if version != expected
    }
    if mismatches:
        print("Release version mismatch detected:", file=sys.stderr)
        for surface, version in mismatches.items():
            print(
                f"- {surface}: expected {expected}, found {version}",
                file=sys.stderr,
            )
        return 1

    print(f"All release versions match v{expected}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
