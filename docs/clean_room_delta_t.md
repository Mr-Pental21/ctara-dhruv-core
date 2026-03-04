# Clean-Room Record: Time Delta-T Future Strategy

## Subsystem

- Name: `dhruv_time` future Delta-T asymptotic strategy selector
- Owner: `ctara-dhruv-core`
- Date: 2026-03-04

## Scope

- What is being implemented:
  - Add an opt-in `Stephenson1997` future strategy to the existing `smh_future_family` selector.
  - Keep behavior limited to post-EOP future fallback in `hybrid-deltat` with `freeze_future_delta_at=false`.
  - Preserve the existing 100-year blend continuity behavior.
- Public API surface impacted:
  - `dhruv_time::SmhFutureParabolaFamily`
  - CLI parsing for `--smh-future-family`
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
  - Reuse existing `SmhFutureParabolaFamily` selector (no duplicate knob).
  - Add `Stephenson1997` variant and evaluate formula in asymptotic future function.
  - Keep gating unchanged so it applies only in post-EOP future fallback path.
  - Keep existing blend function and default `future_transition_years = 100.0`.
- Numerical assumptions:
  - Year-fraction conversion stays as existing code path (`year_fraction_from_jd`).
  - Blend uses linear interpolation in TT-UTC space.
- Edge cases handled:
  - No effect when future freeze is enabled.
  - No effect in pre-range or strict-LSK branches.

## Validation

- Black-box references used (I/O comparison only):
  - None required for formula introduction in this patch.
- Golden test vectors added:
  - Unit test for exact Stephenson1997 formula evaluation.
  - Policy test proving no effect when future freeze is enabled.
  - Policy test proving full model value is reached after the 100-year blend window.
- Error tolerance used:
  - `1e-12` for direct formula identity tests.
  - `1e-6` for policy path floating-point comparisons.

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from denylisted/source-available code.
- Name: Codex (GPT-5)
- Date: 2026-03-04
