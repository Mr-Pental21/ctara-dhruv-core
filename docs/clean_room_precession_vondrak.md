# Clean-Room Record: Precession Model Migration (IAU 2006 -> Vondrak 2011)

## Subsystem

- Name: `dhruv_frames::precession` and model-aware ayanamsha propagation
- Owner: AI agent + project maintainers
- Date: 2026-02-21

## Scope

- What is being implemented:
  - A dual-model precession API surface (`Iau2006`, `Vondrak2011`) with model selection threaded through dependent crates.
  - Migration of default precession behavior to `Vondrak2011` while keeping explicit `Iau2006` access for reproducibility.
  - Model-threading through ayanamsha legacy and anchor-relative code paths.
- Public API surface impacted:
  - `dhruv_frames::precession::*_with_model`
  - `dhruv_frames::PrecessionModel`, `dhruv_frames::DEFAULT_PRECESSION_MODEL`
  - `dhruv_vedic_base::ayanamsha_*_with_model`

## Conceptual Sources

- Paper/spec/public-domain source URL:
  - https://www.aanda.org/articles/aa/abs/2011/10/aa17274-11/aa17274-11.html
  - https://iers-conventions.obspm.fr/chapter5.php
- License/status:
  - IAU/IERS standards and peer-reviewed paper references (conceptual use).
- What concept or formula was used:
  - IAU 2006 ecliptic precession polynomial/angles remain the default backend.
  - Vondrak 2011 long-term model equations for `p_A`, `P_A`, and `Q_A` are implemented from the paper's published coefficient tables.
  - `pi_A` and `Pi_A` are derived from `P_A` and `Q_A` via:
    - `pi_A = asin(sqrt(P_A^2 + Q_A^2))`
    - `Pi_A = atan2(P_A, Q_A)`

## Explicitly Excluded Sources

- Denylisted projects reviewed: `None`
- Source-available/proprietary projects reviewed: `None`

## Data Provenance

- Tables/constants/datasets used:
  - Vondrak 2011 Table 1 coefficients:
    - periodic terms `P_j`, `A_Pj`, `B_Pj`, `A_Qj`, `B_Qj`, `C_j`, `D_j`
    - polynomial terms for `P_A`, `Q_A`, and `p_A`
- Source URL:
  - https://www.aanda.org/articles/aa/abs/2011/10/aa17274-11/aa17274-11.html
  - https://numpy2erfa.readthedocs.io/en/latest/api/erfa.ltpequ.html
- License/status:
  - Peer-reviewed paper (formula source) and ERFA documentation (BSD-3-Clause project docs).
- Evidence this source is public domain or allowlisted:
  - A&A paper provides the canonical published model.
  - ERFA project is BSD-3-Clause (allowlisted), and only formula/table values were used.

## Implementation Notes

- Key algorithm choices:
  - Keep model-specific entry points for downstream selection (`*_with_model` APIs).
  - Set wrapper default to `DEFAULT_PRECESSION_MODEL = Vondrak2011`.
  - Reuse existing Euler-rotation path by swapping in model-specific `p_A`, `pi_A`, and `Pi_A`.
- Numerical assumptions:
  - Vondrak periodic terms are evaluated directly in Julian centuries and added to polynomial terms in arcseconds.
  - Arcsecond series are converted to radians before trigonometric derivations.
- Edge cases handled:
  - `t=0` identity behavior preserved for both model paths.

## Accepted Epoch Range And Expectations

- Epoch parameter: Julian centuries since J2000 (`t`).
- Supported operational range for this phase: Vondrak long-term behavior from the published model tables (century input domain).
- Expectation:
  - small deltas near J2000,
  - increasing divergence from IAU 2006 at large `|t|`,
  - wrapper APIs now default to `Vondrak2011`, with explicit `Iau2006` still available.

## Validation

- Black-box references used (I/O comparison only):
  - Existing unit tests for precession and ayanamsha behavior.
- Golden test vectors added:
  - model-delta checks verifying `Iau2006` and `Vondrak2011` differ at long epochs
  - model-specific round-trip checks for ecliptic precession transforms
- Error tolerance used:
  - machine precision for round-trip and wrapper parity checks
  - explicit non-zero model-delta checks (`> 1e-6` or tighter as appropriate)

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from denylisted/source-available code.
- Name: Codex (GPT-5)
- Date: 2026-02-21
