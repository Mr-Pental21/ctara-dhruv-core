# CLI Upagraha Configuration

Dhruv CLI supports the following time-based upagraha options:

- `--gulika-point start|middle|end`
- `--maandi-point start|middle|end`
- `--other-upagraha-point start|middle|end`
- `--gulika-planet rahu|saturn`
- `--maandi-planet rahu|saturn`

Default behavior:

- Gulika: `rahu` + `start`
- Maandi: `rahu` + `end`
- Other time-based upagrahas: `start`

Standalone upagrahas:

```bash
cargo run -p dhruv_cli -- upagrahas \
  --date 2026-03-17T15:06:19Z \
  --lat 12.9716 \
  --lon 77.5946 \
  --eop /path/to/finals2000A.all \
  --gulika-point middle \
  --gulika-planet saturn \
  --maandi-point end \
  --maandi-planet rahu \
  --other-upagraha-point start
```

Core bindus:

```bash
cargo run -p dhruv_cli -- core-bindus \
  --date 2026-03-17T15:06:19Z \
  --lat 12.9716 \
  --lon 77.5946 \
  --eop /path/to/finals2000A.all \
  --gulika-point start \
  --gulika-planet saturn \
  --maandi-point middle \
  --maandi-planet saturn \
  --other-upagraha-point middle
```

Full kundali:

```bash
cargo run -p dhruv_cli -- kundali \
  --date 2026-03-17T15:06:19Z \
  --lat 12.9716 \
  --lon 77.5946 \
  --eop /path/to/finals2000A.all \
  --include-upagrahas \
  --include-bindus \
  --gulika-point middle \
  --gulika-planet saturn \
  --maandi-point end \
  --maandi-planet rahu \
  --other-upagraha-point end
```
