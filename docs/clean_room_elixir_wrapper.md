# Clean-Room Implementation Record

## Subsystem

- Name: Elixir wrapper (`bindings/elixir-open`) via Rustler NIF
- Owner: Codex
- Date: 2026-03-12

## Scope

- What is being implemented:
  - Elixir bindings for `ctara-dhruv-core` using a Rustler NIF crate
  - Public Elixir modules for engine, ephemeris, time, vedic, panchang, search,
    jyotish, dasha, and tara families
  - Mix tests, lockfile/license integration, and GitHub Actions wiring for the
    Elixir wrapper
- Public API surface impacted:
  - Added new Elixir wrapper package at `bindings/elixir-open`

## Conceptual Sources

- Paper/spec/public-domain source URL:
  - In-repo Rust crate APIs under `crates/dhruv_*`
  - In-repo search/time/vedic public types and operation APIs
  - Rustler public documentation and Hex package metadata
- License/status:
  - Project-owned repository artifacts
  - Rustler and Hex package metadata under allowlisted open-source licenses
- What concept or formula was used:
  - FFI/NIF boundary mapping, resource lifecycle, and DTO conversion patterns
    only

## Explicitly Excluded Sources

- Denylisted projects reviewed: `None`
- Source-available/proprietary projects reviewed: `None`

## Data Provenance

- Tables/constants/datasets used:
  - No new astronomical datasets or copied tables introduced by the wrapper
- Source URL:
  - N/A
- License/status:
  - N/A
- Evidence this source is public domain or allowlisted:
  - Wrapper consumes existing project computations and optional user-provided
    kernel/catalog files only

## Implementation Notes

- Key algorithm choices:
  - Direct Rust crate calls rather than going through the C ABI
  - One Rustler resource storing engine/config/EOP/time-policy/tara-catalog
    state
  - Elixir-side normalization and error shaping to keep the public API idiomatic
- Numerical assumptions:
  - No new numerical models added
- Edge cases handled:
  - Closed engine resource detection
  - Missing EOP/config/catalog error paths
  - Kernel-dependent tests skip when required files are absent

## Validation

- Black-box references used (I/O comparison only):
  - Existing in-repo Rust APIs through smoke tests
- Golden test vectors added:
  - None; wrapper validation is smoke/unit focused
- Error tolerance used:
  - N/A for wrapper layer

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from denylisted/source-available code.
- Name: Codex
- Date: 2026-03-12
