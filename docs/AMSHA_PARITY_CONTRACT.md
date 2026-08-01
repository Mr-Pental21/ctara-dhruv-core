# Amsha Parity Contract

Scope: amsha-related parity only.

This document defines the canonical amsha contract used for wrapper work across:

- C ABI
- `dhruv_rs`
- CLI
- Python
- Go
- Node
- Elixir

It is the Phase 1 acceptance reference for the amsha-first parity plan in
`~/.codex/plans/2026-03-18_amsha_wrapper_cli_rs_parity_plan.md`.

## Canonical Source

The canonical external contract is the C ABI in `crates/dhruv_ffi_c`.

Primary source files:

- `crates/dhruv_ffi_c/src/lib.rs`
- `crates/dhruv_search/src/jyotish_types.rs`
- `crates/dhruv_vedic_math/src/amsha.rs`

Current ABI constants relevant to amsha:

- `DHRUV_API_VERSION = 85`
- `DHRUV_MAX_AMSHA_REQUESTS = 40`
- `DHRUV_AMSHA_POINT_FAMILY_COUNT = 10`

## Canonical Concepts

### Supported amshas

The supported amsha set is the `ALL_AMSHAS` list in `dhruv_vedic_base`:

`D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D11, D12, D15, D16, D18, D20, D21, D22, D24, D25, D27, D28, D30, D36, D40, D45, D48, D50, D54, D60, D72, D81, D108, D144`

The standard shodashavarga subset is:

`D1, D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60`

### Variations

Variation codes are amsha-scoped. The current catalog entries are:

- `D2`: `0=default`, `1=cancer-leo-only`, `2=lunar-hora`, `3=kashinath-hora`
- every other supported amsha currently exposes `0=default`

For `D2` default, odd signs map halves as `start,start+1` and even signs map
halves as `start+1,start`, where `start = (rashi * 2) % 12`.

For `D2` `lunar-hora`, each rashi is divided into 12 equal parts. Odd signs
start at Simha and advance one rashi per part through Karka; even signs start
at Karka and move backward one rashi per part through Simha. The position
inside each 1/12 sign part is scaled to 30 degrees.

For `D2` `kashinath-hora`, the Sun/Moon half is determined by the
`cancer-leo-only` split and then reassigned by natal rashi owner pair:
Surya/Chandra signs map to Simha/Karka, Budha signs to Kanya/Mithuna, Shukra
signs to Tula/Vrishabha, Mangala signs to Vrischika/Mesha, Guru signs to
Dhanu/Meena, and Shani signs to Makara/Kumbha for Sun/Moon hora respectively.

Use the amsha variation discovery helpers as the authoritative source for the
valid codes, names, labels, and defaults for any given amsha.

### Request model

Canonical request/config concepts:

- `Amsha`
- `AmshaRequest`
- `AmshaChartScope`
- `AmshaSelectionConfig`

### Output model

Canonical result concepts:

- `RashiInfo`
- `AmshaEntry`
- `AmshaPoint`
- `AmshaPointFamily`
- `AmshaChart`
- `AmshaResult`

Optional `AmshaChart` sections:

- `bhava_cusps`
- `rashi_bhava_cusps`
- `arudha_padas`
- `rashi_bhava_arudha_padas`
- `upagrahas`
- `sphutas`
- `special_lagnas`
- `outer_planets`

Grahas and lagna are always present.

### Point identity

Every `AmshaChart` section is a positional array, and every position is a
fixed, named point. `AmshaEntry` carries that identity as `point`
(`AmshaPoint { family, index }`), so a consumer never has to recover it from
array order.

`AmshaPoint::name()` gives a display name drawn from the library's existing
vocabulary (`Graha::name()`, `Upagraha::name()`, `Sphuta::name()`,
`SpecialLagna::name()`, `ArudhaPada::name()`); `AmshaPoint::key()` gives a
stable snake_case identifier.

Canonical order per family — wrappers must not renumber these:

| family | code | n | order |
|---|---|---|---|
| `Lagna` | 0 | 1 | the varga ascendant |
| `Graha` | 1 | 9 | `Graha::index()`: surya, chandra, mangal, buddh, guru, shukra, shani, rahu, ketu |
| `OuterPlanet` | 2 | 3 | uranus, neptune, pluto |
| `BhavaCusp` | 3 | 12 | index `i` is Bhava `i + 1` |
| `RashiBhavaCusp` | 4 | 12 | index `i` is Bhava `i + 1` |
| `ArudhaPada` | 5 | 12 | `ALL_ARUDHA_PADAS`: a1 (AL) .. a12 (UL) |
| `RashiBhavaArudhaPada` | 6 | 12 | `ALL_ARUDHA_PADAS` |
| `Upagraha` | 7 | 11 | `ALL_UPAGRAHAS`: gulika, maandi, kaala, mrityu, artha_prahara, yama_ghantaka, dhooma, vyatipata, parivesha, indra_chapa, upaketu |
| `Sphuta` | 8 | 16 | `ALL_SPHUTAS`, index 0 is bhrigu_bindu |
| `SpecialLagna` | 9 | 8 | `ALL_SPECIAL_LAGNAS`: bhava, hora, ghati, vighati, varnada, **sree (5)**, **pranapada (6)**, indu |

The special-lagna order is the one place where the presentation order in
`docs/clean_room_special_lagnas.md` differs from the serialised order; that
document is grouped by derivation category and its section numbers are not
indices. `dhruv_vedic_base` is the authority.

### Per-entry data

Beyond `point`, every `AmshaEntry` carries `sidereal_longitude`, `rashi`,
`rashi_index`, `dms`, `degrees_in_rashi`, `nakshatra`, `nakshatra_index`,
`pada`, and `rashi_bhava_number`.

`rashi_bhava_number` is the whole-sign bhava (1-12) counted from the varga
lagna's rashi. There is deliberately no cusp-based `bhava_number` on an amsha
entry: a varga transform is not monotonic, so the `bhava_cusps` section (D1
cusps mapped through the varga) does not form ordered house boundaries and
cannot define a cusp-based bhava inside a varga.

### Emitted shape per surface

The C ABI keeps `DhruvAmshaChart` a fixed-layout `#[repr(C)]` struct and does
**not** repeat names inside `DhruvAmshaEntry`; a point's name is a compile-time
constant of (family, index) and is queried instead:

- `dhruv_amsha_point_count(family)`
- `dhruv_amsha_point_name(family, index)`
- `dhruv_amsha_point_key(family, index)`

JSON-shaped and object-shaped wrapper surfaces (Elixir, Node, Python, Go) do
carry the resolved identity on every entry, as additive fields — `name` (the
stable key), `display_name`, `family`, `point_index`. **These sections remain
arrays on every surface.** Converting them to maps is a breaking change and is
out of contract.

## Canonical C ABI Surface

### Direct pure-amsha transforms

- `dhruv_amsha_longitude`
- `dhruv_amsha_rashi_info`
- `dhruv_amsha_longitudes`

### Date/location-backed amsha chart orchestration

- `dhruv_amsha_chart_for_date`

### Range operations (ABI v77)

- `dhruv_amsha_series` (+ `_point_count` / `_chart_count` / `_point_at` /
  `_chart_at` / `_free`) — fixed-cadence slim varga charts
- `dhruv_amsha_lagna_events` (+ `_entry_count` / `_entry_info` /
  `_segment_count` / `_segment_at` / `_meta` / `_free`) — exact varga-lagna
  rashi transitions

See `docs/C_ABI_REFERENCE.md` for the full shapes; the same amsha
code/variation validation rules below apply to their request lists.

### Full-kundali embedded amsha support

- `dhruv_full_kundali_config_default`
- `dhruv_full_kundali_for_date`
- `DhruvFullKundaliConfig.amsha_scope`
- `DhruvFullKundaliConfig.amsha_selection`
- `DhruvFullKundaliResult.amshas`

### Canonical amsha-related C structs

- `DhruvAmshaChartScope`
- `DhruvAmshaSelectionConfig`
- `DhruvAmshaEntry`
- `DhruvAmshaChart`
- `DhruvFullKundaliConfig`
- `DhruvFullKundaliResult`

## Validation Contract

All wrappers must preserve these semantics.

### Code validation

- Unknown amsha code: reject.
- Unknown variation code: reject.
- Selection count above `40`: reject.

### Variation validation

- Unknown variation code for the selected amsha: reject.

### Defaulting

- Missing variation means that amsha's default variation.
- Missing `AmshaChartScope` means all optional-section flags are false.

### Output guarantees

- Every returned amsha longitude is normalized to `[0, 360)`.
- Every returned `rashi_index` is in `0..=11`.
- In `AmshaChart`, `grahas` always has length `9`.
- In `AmshaChart`, `lagna` is always present.
- Every entry's `point` matches its position in its section, per the family
  table above.
- Every returned `nakshatra_index` is in `0..=26`, `pada` in `1..=4`, and
  `rashi_bhava_number` in `1..=12`.
- Optional sections are arrays, never maps.

## Full-Kundali Dependency Contract

Embedded amsha charts in `full_kundali` depend on the relevant root sections
already being computed.

Required dependencies:

- amsha charts in general require graha positions with lagna
- `amsha_scope.include_bhava_cusps` depends on `include_bhava_cusps`
- `amsha_scope.include_arudha_padas` depends on `include_bindus`
- `amsha_scope.include_upagrahas` depends on `include_upagrahas`
- `amsha_scope.include_sphutas` depends on `include_sphutas`
- `amsha_scope.include_special_lagnas` depends on `include_special_lagnas`

Wrappers may satisfy this either by:

1. exposing the raw config and documenting these dependencies clearly, or
2. auto-promoting the dependent root flags when amsha scope requests them.

Either approach is acceptable, but silent omission of requested amsha sections
is not.

## Wrapper Expectations

### `dhruv_rs`

Expected public surface:

- low-level pure helpers for:
  - single amsha longitude
  - batch amsha longitudes
  - amsha rashi info
- date-backed amsha chart helper
- root re-exports for the amsha type family
- `FullKundaliConfig` access to `amsha_selection` and `amsha_scope`

### CLI

Expected public surface:

- direct pure transform command: `amsha`
- date-backed chart command: `amsha-chart`
- `kundali` flags for:
  - enabling amshas
  - selecting amsha requests
  - selecting amsha scope
- printed output for optional amsha sections when requested and present

### Python

Expected public surface:

- `ctara_dhruv.amsha.amsha_longitude`
- `ctara_dhruv.amsha.amsha_rashi_info`
- `ctara_dhruv.amsha.amsha_longitudes`
- `ctara_dhruv.amsha.amsha_chart_for_date`
- `ctara_dhruv.kundali.full_kundali_config_default`
- `ctara_dhruv.kundali.full_kundali` with usable `amsha_selection` and `amsha_scope`
- extraction of all optional `AmshaChart` sections

### Go

Expected public surface:

- `AmshaLongitude`
- `AmshaRashiInfo`
- `AmshaLongitudes`
- `(*Engine).AmshaChartForDate`
- `FullKundaliConfig.AmshaSelection`
- `FullKundaliConfig.AmshaScope`
- extraction of all optional `AmshaChart` sections

### Node

Expected public surface:

- `amshaLongitude`
- `amshaRashiInfo`
- `amshaLongitudes`
- `amshaChartForDate`
- `fullKundaliConfigDefault`
- `fullKundaliForDate` with `amshaSelection` and `amshaScope`
- extraction of all optional `AmshaChart` sections

### Elixir

Expected public surface:

- `CtaraDhruv.Jyotish.amsha/2`
- `CtaraDhruv.Jyotish.full_kundali/2`
- request/config support for:
  - amsha request codes
  - variation codes
  - `amsha_scope`
  - `amsha_selection`
- result maps that expose all optional amsha sections when present

## Wrapper Checklist

Use this checklist as the acceptance gate for any wrapper claiming amsha parity.

### Shared checklist

- supports the canonical amsha code set from `ALL_AMSHAS`
- supports variation code `0`
- supports variation code `1`
- rejects unknown amsha codes
- rejects unknown variation codes for the selected amsha
- preserves the default variation behavior
- preserves the optional-section scope behavior
- preserves the canonical point order for every family
- exposes point identity: resolved per entry on JSON/object surfaces, or via
  the indexed accessors on the C ABI
- keeps optional sections as arrays
- exposes `nakshatra_index`, `pada`, and `rashi_bhava_number` per entry

### Per-wrapper checklist

#### `dhruv_rs`

- exposes pure single-longitude amsha transform
- exposes pure batch amsha transform
- exposes amsha rashi info helper
- exposes date-backed amsha chart helper
- re-exports the amsha type family
- exposes `full_kundali` amsha selection/scope config

#### CLI

- `amsha` supports longitude-only output
- `amsha` supports rashi-info output
- `amsha` supports machine-readable batch output
- `amsha-chart` supports one or more amsha requests
- `amsha-chart` supports amsha scope flags
- `kundali` supports explicit amsha selection
- `kundali` supports explicit amsha scope

#### Python

- direct amsha helpers are present
- date-backed amsha chart helper is present
- `full_kundali` config exposes `amsha_selection`
- `full_kundali` config exposes `amsha_scope`
- optional amsha chart sections are extracted

#### Go

- direct amsha helpers are present
- date-backed amsha chart helper is present
- `FullKundaliConfig` exposes `AmshaSelection`
- `FullKundaliConfig` exposes `AmshaScope`
- optional amsha chart sections are extracted

#### Node

- direct amsha helpers are present
- date-backed amsha chart helper is present
- `fullKundaliConfigDefault()` exposes `amshaSelection`
- `fullKundaliConfigDefault()` exposes `amshaScope`
- optional amsha chart sections are extracted

#### Elixir

- dedicated amsha request path accepts caller-controlled scope
- full kundali config exposes `amsha_selection`
- full kundali config exposes `amsha_scope`
- optional amsha chart sections are present in result maps

## Notes

- This document defines expected surface and behavior, not language-specific API
  style.
- Where wrapper naming differs from the C ABI, capability parity matters more
  than exact spelling.
- This document is intentionally amsha-only; it does not define parity policy
  for unrelated API families.
