# Rust Library Upagraha Configuration

These examples use the Rust wrapper surface and `TimeUpagrahaConfig`.

Default behavior:

- Gulika: `Rahu` + `Start`
- Maandi: `Rahu` + `End`
- Other time-based upagrahas: `Start`

```rust
use dhruv_rs::{
    GulikaMaandiPlanet, TimeUpagrahaConfig, TimeUpagrahaPoint,
};

let config = TimeUpagrahaConfig {
    gulika_point: TimeUpagrahaPoint::Middle,
    gulika_planet: GulikaMaandiPlanet::Saturn,
    maandi_point: TimeUpagrahaPoint::End,
    maandi_planet: GulikaMaandiPlanet::Rahu,
    other_point: TimeUpagrahaPoint::Start,
};

let upagrahas = dhruv_rs::upagrahas_with_config(
    &date,
    &eop,
    location,
    system,
    false,
    &config,
)?;
```

If you are building `BindusConfig` or `FullKundaliConfig`, pass the same
`TimeUpagrahaConfig` into those config structs so Gulika and Maandi stay
consistent across bindus and full-kundali output.
