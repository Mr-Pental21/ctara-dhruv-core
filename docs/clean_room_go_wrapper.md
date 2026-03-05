# Clean-Room Implementation Record

## Subsystem

- Name: Go wrapper (`bindings/go-open`) for `dhruv_ffi_c`
- Owner: Codex
- Date: 2026-03-05

## Scope

- What is being implemented:
  - Go bindings for `ctara-dhruv-core` via canonical C ABI (`dhruv.h`)
  - Public API package with engine/time/search/panchang/jyotish/dasha/tara wrappers
  - Go integration tests and CI wiring
- Public API surface impacted:
  - Added new Go module `bindings/go-open`

## Conceptual Sources

- Paper/spec/public-domain source URL:
  - In-repo C ABI header: `crates/dhruv_ffi_c/include/dhruv.h`
  - In-repo ABI docs: `docs/C_ABI_REFERENCE.md`
  - Go cgo documentation: https://pkg.go.dev/cmd/cgo
- License/status:
  - Project-owned repository artifacts
  - Go docs under permissive documentation terms
- What concept or formula was used:
  - FFI boundary mapping and resource lifecycle patterns only

## Explicitly Excluded Sources

- Denylisted projects reviewed: `None`
- Source-available/proprietary projects reviewed: `None`

## Data Provenance

- Tables/constants/datasets used:
  - No new external datasets introduced by wrapper
- Source URL:
  - N/A
- License/status:
  - N/A
- Evidence this source is public domain or allowlisted:
  - Wrapper only consumes existing project ABI types and runtime outputs

## Implementation Notes

- Key algorithm choices:
  - Two-layer design: `internal/cabi` (cgo bridge) and `dhruv` (idiomatic API)
  - Explicit status-code translation to Go errors
  - Resource cleanup with `Close()` plus finalizer safety net
- Numerical assumptions:
  - No new numerical models added
- Edge cases handled:
  - Null/invalid handles guarded in wrapper lifecycle
  - ABI version mismatch check exposed via `VerifyABI()`

## Validation

- Black-box references used (I/O comparison only):
  - Existing kernel-backed integration behavior via C ABI
- Golden test vectors added:
  - No new golden vectors; integration tests use repository kernels
- Error tolerance used:
  - N/A for wrapper layer

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from denylisted/source-available code.
- Name: Codex
- Date: 2026-03-05
