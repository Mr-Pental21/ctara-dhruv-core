# Contributing & Licensing

The authoritative policy files live at the repository root; this chapter is
the map.

## Clean-room rules (read before contributing)

This library is a **clean-room implementation**. Contributors must not read
GPL/AGPL/LGPL-licensed astrology source code (or any denylisted source)
while working on dhruv, and every substantial algorithm needs a provenance
record listing its conceptual sources.

- `LICENSE_POLICY.md` — allowed licenses (MIT/Apache-2.0/BSD/ISC/Zlib),
  denylist, source-intake rules, the no-taint rule.
- `CLEAN_ROOM_RECORD_TEMPLATE.md` — the record template; existing records
  are indexed in the [Internal Records Index](internal_records.md).
- `CONTRIBUTING.md` — the PR checklist: license review, third-party notice
  updates, clean-room declaration.

## Licensing

The workspace is a mix of MIT (workspace default) and Apache-2.0
(per-crate overrides); the root `LICENSE` file is the Apache-2.0 text.
Automated enforcement: `deny.toml` + `scripts/ci/license_gate.sh` run in CI
across all five ecosystems (Rust, Node, Python, Go, Elixir).

> **Note for maintainers:** the per-crate license fields, the workspace
> default, and the root LICENSE file do not currently tell one consistent
> story (MIT vs Apache-2.0). Worth unifying — or documenting the split
> deliberately — before wider publication.

## Development conventions

- Every public feature must land on **all** surfaces (C ABI, `dhruv_rs`,
  CLI, Python/Go/Node/Elixir wrappers) with docs updated in the same change
  (`docs/` + `docs/end_user/`).
- One entry point per feature; variation via request/config types.
- Unit tests in `src/` are for pure logic only; anything touching kernel or
  data files goes in crate `tests/` and must skip gracefully when the files
  are absent.
- Naming: crates `dhruv_*`, C symbols `dhruv_*` / constants `DHRUV_*`,
  Elixir modules `CtaraDhruv.*`.
- After pushing, refresh local artifacts with
  `scripts/ci/build_local_native_binaries.sh` so downstream consumers can
  sync.
