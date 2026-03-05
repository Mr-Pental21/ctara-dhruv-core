# Clean-Room Implementation Record

## Subsystem

- Name: Node wrapper (`bindings/node-open`) for `dhruv_ffi_c`
- Owner: Codex
- Date: 2026-03-05

## Scope

- What is being implemented:
  - Node.js bindings for `ctara-dhruv-core` via canonical C ABI (`dhruv.h`)
  - Native Node-API addon (`native/dhruv_node.cc`) with JS module wrappers
  - Integration tests and CI wiring for Node wrapper
- Public API surface impacted:
  - Added new Node package `bindings/node-open`

## Conceptual Sources

- Paper/spec/public-domain source URL:
  - In-repo C ABI header: `crates/dhruv_ffi_c/include/dhruv.h`
  - In-repo ABI docs: `docs/C_ABI_REFERENCE.md`
  - Node-API C header references from Node runtime (`node_api.h`, `js_native_api.h`)
- License/status:
  - Project-owned repository artifacts
  - Node runtime headers used for API interop
- What concept or formula was used:
  - FFI boundary mapping and handle lifecycle patterns only

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
  - Wrapper consumes existing project ABI types and runtime outputs only

## Implementation Notes

- Key algorithm choices:
  - Two-layer design: native addon boundary + JS public modules
  - Status-code-to-error translation at JS layer
  - Explicit `close()` lifecycle for handle-owning types
- Numerical assumptions:
  - No new numerical models added
- Edge cases handled:
  - ABI mismatch validation (`dhruv_api_version`)
  - Kernel-dependent tests skip when required files are absent

## Validation

- Black-box references used (I/O comparison only):
  - Existing C ABI behavior via smoke integration tests
- Golden test vectors added:
  - None (wrapper-layer smoke coverage only)
- Error tolerance used:
  - N/A for wrapper layer

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from denylisted/source-available code.
- Name: Codex
- Date: 2026-03-05
