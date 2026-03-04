# Clean-Room Record: Time Delta-T Future Transition Strategies

## Subsystem

- Name: `dhruv_time` model-agnostic future Delta-T transition framework
- Owner: `ctara-dhruv-core`
- Date: 2026-03-04

## Scope

- What is being implemented:
  - Replace dual future controls (`freeze_future_delta_at` + model selectors) with a single strategy axis.
  - Add `FutureDeltaTTransition::{LegacyTtUtcBlend, BridgeFromModernEndpoint}`.
  - Keep `LegacyTtUtcBlend` as the default frozen-compatible contract:
    - `TT-UTC = last_delta_at + DELTA_T_A`
    - diagnostics source `LskDeltaAt`
    - ignore `smh_future_family` and `future_transition_years`
  - Implement `BridgeFromModernEndpoint` as model-agnostic Delta-T bridge from
    `max(EOP prediction end, LSK end)` to asymptotic family over configurable years.
  - Preserve default continuity window: `future_transition_years = 100.0`.
- Public API surface impacted:
  - `dhruv_time::FutureDeltaTTransition`
  - `dhruv_time::TimeConversionOptions::future_delta_t_transition`
  - `dhruv_time::SmhFutureParabolaFamily`
  - CLI parsing for `--future-delta-t-transition` and `--smh-future-family`
  - runtime docs and release notes

## Conceptual Sources

- Paper/spec/public-domain source URL:
  - https://eclipse.gsfc.nasa.gov/SEcat5/deltatpoly.html
  - https://eclipse.gsfc.nasa.gov/SEhelp/deltaT2.html
- License/status:
  - NASA/GSFC public information pages (U.S. Government work).
- What concept or formula was used:
  - Stephenson-1997 long-term expression documented there:
    - `ΔT = -20 + 31*t^2`, `t = (year - 1820)/100`
  - Transition guidance toward long-term model without discontinuity.

## Explicitly Excluded Sources

- Denylisted projects reviewed: `None`
- Source-available/proprietary projects reviewed: `None`

## Data Provenance

- Tables/constants/datasets used:
  - Formula constants only: `-20`, `31`, `1820`.
- Source URL:
  - https://eclipse.gsfc.nasa.gov/SEhelp/deltaT2.html
- License/status:
  - NASA/GSFC public information page (U.S. Government work).
- Evidence this source is public domain or allowlisted:
  - NASA/GSFC web publication; no third-party code/data import performed.

## Implementation Notes

- Key algorithm choices:
  - Introduce single strategy control in `TimeConversionOptions`:
    - `LegacyTtUtcBlend` (default compatibility strategy)
    - `BridgeFromModernEndpoint` (new model-agnostic bridge)
  - Bridge at Delta-T level:
    - `T_end = modern Delta-T at anchor`
    - `F(Y) = asymptotic Delta-T for selected family`
    - `F_end = F(anchor)`
    - blend term `F(Y) + (T_end - F_end) * (1 - alpha)` over configured window.
  - Preserve anchor rule: `anchor = max(EOP prediction end, LSK end)`.
  - Add `Stephenson1997` family and evaluate formula in asymptotic future function.
- Numerical assumptions:
  - Year-fraction conversion stays as existing code path (`year_fraction_from_jd`).
  - Blend uses linear interpolation in TT-UTC space.
- Edge cases handled:
  - Legacy strategy ignores `smh_future_family` and `future_transition_years`.
  - Bridge strategy reaches pure asymptotic family after window completion.
  - No effect in pre-range or strict-LSK branches.

## Validation

- Black-box references used (I/O comparison only):
  - None required for formula introduction in this patch.
- Golden test vectors added:
  - Unit test for exact Stephenson1997 formula evaluation.
  - Policy test proving legacy strategy ignores selected future family.
  - Policy test proving full model value is reached after the 100-year blend window.
- Error tolerance used:
  - `1e-12` for direct formula identity tests.
  - `1e-6` for policy path floating-point comparisons.

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from denylisted/source-available code.
- Name: Codex (GPT-5)
- Date: 2026-03-04
