# Elixir Upagraha Configuration

Elixir accepts an `upagraha_config` map with string or integer enum values.

String values:

- points: `"start"`, `"middle"`, `"end"`
- planets: `"rahu"`, `"saturn"`

```elixir
%{
  op: "upagrahas",
  utc: %{year: 2026, month: 3, day: 17, hour: 15, minute: 6, second: 19.0},
  location: %{latitude_deg: 12.9716, longitude_deg: 77.5946, altitude_m: 0.0},
  upagraha_config: %{
    gulika_point: "middle",
    gulika_planet: "saturn",
    maandi_point: "end",
    maandi_planet: "rahu",
    other_point: "start"
  }
}
```
