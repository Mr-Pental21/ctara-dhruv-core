# Rust Library End-User Docs

`dhruv_rs` provides a Rust-friendly surface over the core crates.
It includes:

- `DhruvContext` and request/operation APIs
- search operations (`conjunction`, `motion`, `sankranti`, `lunar_phase`,
  `grahan`) — the conjunction/motion/sankranti requests track any
  `TransitBody` (plain bodies plus Rahu/Ketu), and the sankranti operation
  is a general rashi-ingress search for any body (Sun = classical
  sankranti)
- canonical request/context-driven jyotish operations such as `upagraha_op`,
  `avastha_op`, `full_kundali`, and `gochar_events`
- re-exported config/result types used by end users

Amsha variation control is shared across the high-level jyotish surfaces:
standalone shadbala, vimsopaka, balas, and avastha calls accept
`AmshaSelectionConfig`, and `full_kundali(...).amshas` returns the resolved
union of explicit and internally required amshas.

Start here:

- [Rust Reference](./reference.md)
- [Upagraha Configuration](./upagraha_configuration.md)
- `cargo add dhruv_rs` from the unified crates.io release stream

Deeper internal reference:

- [`docs/rust_wrapper.md`](../../rust_wrapper.md)
- [`docs/release_distribution.md`](../../release_distribution.md)
