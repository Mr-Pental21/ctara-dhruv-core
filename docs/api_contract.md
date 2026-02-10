# API Contract (Draft)

## Principles
- Stable, explicit input and output types.
- Deterministic behavior for equivalent normalized inputs.
- Explicit error codes/messages across all public boundaries.

## Core Rust API (Draft)
```rust
pub struct EngineConfig;
pub struct Engine;

pub struct Query {
    pub body: Body,
    pub observer: Observer,
    pub frame: Frame,
    pub epoch: Epoch,
}

pub struct StateVector {
    pub position_km: [f64; 3],
    pub velocity_km_s: [f64; 3],
}

impl Engine {
    pub fn new(config: EngineConfig) -> Result<Self, EngineError>;
    pub fn query(&self, query: Query) -> Result<StateVector, EngineError>;
}
```

## ABI Principles (Draft)
- No panics across FFI boundaries.
- Versioned ABI surface (`DHRUV_API_VERSION`).
- Caller-allocated buffers or explicit release functions.
- Numeric units and frames are explicit in function names or structs.

## Compatibility
- Pre-1.0: iterate quickly but avoid unnecessary public API churn.
- 1.0+: semver policy for API/ABI compatibility.
