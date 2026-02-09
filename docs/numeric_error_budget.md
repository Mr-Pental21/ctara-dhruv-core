# Numeric Error Budget (Draft)

## Validation Targets
- Reference systems: JPL Horizons snapshots and CSPICE spot checks.
- Initial outputs: Cartesian position (km) and velocity (km/s).

## Draft Tolerance Buckets
- Position error tolerance: TBD (km) per body class.
- Velocity error tolerance: TBD (km/s) per body class.
- Angle-derived tolerance: TBD (arcseconds).

## Error Sources To Track
- Kernel interpolation rounding.
- Time-scale conversion errors (UTC -> TT/TDB).
- Frame transformation accumulation.
- Floating-point behavior across targets.

## CI Policy (Target)
- Golden fixtures are versioned and immutable.
- Any tolerance change requires explicit review and rationale.
- Cross-platform consistency checks run in CI matrix.
