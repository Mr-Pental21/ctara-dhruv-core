# `dhruv_rs` — Rust Convenience Wrapper

## Purpose

`dhruv_rs` provides a high-level Rust API over `dhruv_core`. It removes boilerplate by managing a global engine singleton, accepting UTC dates directly, and returning spherical coordinates.

Users only need `use dhruv_rs::*` — all core types are re-exported.

## Quick Start

```rust
use std::path::PathBuf;
use dhruv_rs::*;

// 1. Initialize once at startup
let config = EngineConfig::with_single_spk(
    PathBuf::from("kernels/data/de442s.bsp"),
    PathBuf::from("kernels/data/naif0012.tls"),
    256,
    true,
);
init(config).expect("engine init");

// 2. Query with UTC dates
let date: UtcDate = "2024-03-20T12:00:00Z".parse().unwrap();
let lon = longitude(Body::Mars, Observer::Body(Body::Earth), date).unwrap();
println!("Mars ecliptic longitude: {:.4}°", lon.to_degrees());
```

## API Reference

### Initialization

| Function | Description |
|---|---|
| `init(config: EngineConfig) -> Result<(), DhruvError>` | Initialize the global engine. Must be called once before queries. |
| `is_initialized() -> bool` | Check whether the engine has been initialized. |

### Date Input

`UtcDate` is a UTC calendar date with sub-second precision:

```rust
// Constructor
let d = UtcDate::new(2024, 3, 20, 12, 0, 0.0);

// ISO 8601 parsing (subset: YYYY-MM-DDTHH:MM:SS[.f]Z)
let d: UtcDate = "2024-03-20T12:30:45.5Z".parse().unwrap();
```

### Convenience Functions

All functions use the global engine initialized by `init()`.

| Function | Returns | Frame |
|---|---|---|
| `position(target, observer, date)` | `SphericalCoords` (lon, lat, distance) | Ecliptic J2000 |
| `position_full(target, observer, date)` | `SphericalState` (position + angular velocities) | Ecliptic J2000 |
| `longitude(target, observer, date)` | `f64` (ecliptic longitude in radians) | Ecliptic J2000 |
| `query(target, observer, frame, date)` | `StateVector` (Cartesian position + velocity) | Caller-specified |
| `query_batch(requests)` | `Vec<Result<StateVector, DhruvError>>` | Per-request |

### Error Type

```rust
pub enum DhruvError {
    NotInitialized,       // init() not called
    AlreadyInitialized,   // init() called twice
    DateParse(String),    // invalid ISO 8601 string
    Engine(EngineError),  // error from dhruv_core
    Time(TimeError),      // error from dhruv_time
}
```

## Re-exported Types

These are re-exported from `dhruv_core` and `dhruv_frames` so callers don't need direct dependencies:

- `Body`, `Observer`, `Frame`, `StateVector`, `EngineConfig`
- `SphericalCoords`, `SphericalState`

## Design Notes

- **Global singleton**: Uses `OnceLock<Engine>` — lock-free after initialization since `Engine` is `Send + Sync` and `query()` takes `&self`.
- **No external dependencies**: ISO 8601 parsing is hand-rolled for the supported subset.
- **Ecliptic J2000 default**: `position()`, `position_full()`, and `longitude()` always use ecliptic J2000, which is the standard frame for astrological and most astronomical longitude. Use `query()` for ICRF/J2000.
- **Batch memoization**: `query_batch()` delegates to `Engine::query_batch()`, sharing memoization across same-epoch queries.

## Module Structure

```
crates/dhruv_rs/
  Cargo.toml
  src/
    lib.rs              # module declarations, re-exports
    error.rs            # DhruvError enum
    date.rs             # UtcDate struct + FromStr
    global.rs           # OnceLock singleton
    convenience.rs      # position, longitude, query, query_batch
  tests/
    integration.rs      # kernel-dependent tests
```
