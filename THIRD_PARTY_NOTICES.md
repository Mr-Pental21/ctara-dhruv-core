# Third-Party Notices

This repository distributes original project source code plus dependencies
resolved through the Rust ecosystem and (optionally) language bindings.

## Dependency Inventory

- Rust dependencies are pinned in `Cargo.lock` and governed by `deny.toml`.
- Binding dependencies (when present) must be lockfile-based and are scanned by
  `scripts/ci/license_gate.sh`.

## License Policy

Allowed licenses are defined in `LICENSE_POLICY.md`:

- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Zlib

Denylisted/restricted licenses are rejected in CI.

## Attribution Workflow

When dependency sets change:

1. Update lockfiles (`Cargo.lock`, wrapper lockfiles).
2. Re-run license checks (`scripts/ci/license_gate.sh`).
3. Update this file with any new third-party attribution requirements.

## Python Bindings Dependencies

The `ctara-dhruv` Python package (`bindings/python-open/`) uses the following
runtime dependencies, pinned in `requirements.lock.txt`:

| Package    | Version | License      | Purpose                        |
|------------|---------|--------------|--------------------------------|
| cffi       | 1.17.1  | MIT          | C FFI interface (ABI/dlopen)   |
| pycparser  | 2.22    | BSD-3-Clause | C header parser (cffi dep)     |

Both licenses are on the project allowlist.

## Go Bindings Dependencies

The Go wrapper (`bindings/go-open/`) currently uses only the Go standard
library and does not introduce third-party Go module dependencies.

## Node Bindings Dependencies

The Node wrapper (`bindings/node-open/`) currently uses only Node.js built-in
modules and an in-repo native Node-API addon implementation. It does not
introduce third-party npm runtime dependencies.

## Elixir Bindings Dependencies

The Elixir wrapper (`bindings/elixir-open/`) pins Hex dependencies in
`mix.lock` and currently resolves to:

| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| rustler | 0.37.3  | MIT OR Apache-2.0 | Rustler NIF compile/load integration |
| jason   | 1.4.4   | Apache-2.0 | Rustler Hex dependency |

Both licenses are on the project allowlist. The native Rust crate
`dhruv_elixir_nif` is part of the root Cargo workspace, so its Rust dependency
licenses are tracked through the shared `Cargo.lock` and `cargo deny` policy.

## Bundled Data

Kernel/data files in `kernels/` are sourced from public NAIF/JPL resources and
tracked via lock manifests with checksums and provenance references.
