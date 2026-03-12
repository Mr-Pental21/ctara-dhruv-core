# Elixir Wrapper (`elixir-open`)

Open-source Elixir bindings for `ctara-dhruv-core`, implemented as a Rustler
NIF that calls the in-repo Rust crates directly.

## Status

- OTP app: `:ctara_dhruv`
- Binding strategy: source-built Rustler NIF (`native/dhruv_elixir_nif`)
- Package root: `bindings/elixir-open`
- Build mode: local source build only, no precompiled NIFs

## Prerequisites

- Elixir 1.19+
- Erlang/OTP 28+
- Rust toolchain (`cargo`)

## Build

From `bindings/elixir-open`:

```bash
mix deps.get
mix test
```

Rustler compiles the NIF automatically during Mix compilation.

## Test

```bash
mix test
```

The ExUnit suite runs wrapper smoke coverage across the native families. Tests
that require SPK/LSK/EOP/tara data skip gracefully when those files are absent.

## Quickstart

```elixir
alias CtaraDhruv.{Engine, Ephemeris, Time}

{:ok, engine} =
  Engine.new(%{
    spk_paths: ["/abs/path/to/de442s.bsp"],
    lsk_path: "/abs/path/to/naif0012.tls",
    cache_capacity: 64,
    strict_validation: false
  })

{:ok, state} =
  Ephemeris.query(engine, %{
    target: :mars,
    observer: :solar_system_barycenter,
    frame: :eclip_j2000,
    epoch_tdb_jd: 2_451_545.0
  })

{:ok, jd} =
  Time.utc_to_jd_tdb(engine, %{
    utc: %{year: 2024, month: 1, day: 1, hour: 12, minute: 0, second: 0.0}
  })

IO.inspect(state)
IO.inspect(jd)

:ok = Engine.close(engine)
```

## Coverage

Public modules included in this wrapper:

- `CtaraDhruv.Engine`
- `CtaraDhruv.Ephemeris`
- `CtaraDhruv.Time`
- `CtaraDhruv.Vedic`
- `CtaraDhruv.Panchang`
- `CtaraDhruv.Search`
- `CtaraDhruv.Jyotish`
- `CtaraDhruv.Dasha`
- `CtaraDhruv.Tara`

Each module returns `{:ok, value}` or
`{:error, %CtaraDhruv.Error{kind, message, details}}`. The only long-lived
wrapper-owned struct is `%CtaraDhruv.Engine{}`.

## Notes

- The wrapper keeps the NIF boundary private in `CtaraDhruv.Native`.
- Most results are returned as atom-keyed maps and lists rather than large
  Elixir struct graphs.
- The default tara catalog is the embedded Rust catalog; loading a JSON catalog
  from disk is optional.
