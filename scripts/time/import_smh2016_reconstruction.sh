#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <source-table-file>" >&2
  exit 2
fi

src="$1"
if [[ ! -f "$src" ]]; then
  echo "Source file not found: $src" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
dest_dir="$repo_root/kernels/data/time"
dest="$dest_dir/smh2016_reconstruction.tsv"

mkdir -p "$dest_dir"

# Normalize to a canonical numeric table while stripping comments/headers.
# Accepted input shapes:
#   1) Cubic spline rows: Ki Ki+1 a0 a1 a2 a3
#      (or with an extra leading row-index column)
#   2) Point rows: year delta_t_seconds
#
# Output shape:
#   - If spline rows are detected first: six columns Ki..a3
#   - Otherwise: two columns year delta_t_seconds
awk '
  BEGIN { OFS="\t" }
  /^[[:space:]]*$/ { next }
  /^[[:space:]]*#/ { next }
  /^[[:space:]]*\/\// { next }
  /^[[:space:]]*%/ { next }
  /^[[:space:]]*[Yy]ear([[:space:]]|,|;|$)/ { next }
  {
    gsub(/[,;]/, " ");
    n = split($0, raw, /[[:space:]]+/);
    m = 0;
    for (i = 1; i <= n; i++) {
      if (raw[i] ~ /^[-+]?[0-9]+([.][0-9]+)?$/) {
        m++;
        nums[m] = raw[i];
      }
    }
    if (m < 2) { next }

    if (mode == "") {
      if (m >= 6) mode = "segment";
      else mode = "point";
    }

    if (mode == "segment") {
      # Accept either [Ki..a3] or [row,Ki..a3]
      if (m >= 7) {
        print nums[2], nums[3], nums[4], nums[5], nums[6], nums[7];
      } else if (m >= 6) {
        print nums[1], nums[2], nums[3], nums[4], nums[5], nums[6];
      }
      next;
    }

    # point mode
    print nums[1], nums[2];
  }
' "$src" > "$dest"

line_count="$(wc -l < "$dest" | tr -d ' ')"
if [[ "$line_count" -lt 1 ]]; then
  echo "Import produced too few rows ($line_count). Aborting." >&2
  rm -f "$dest"
  exit 1
fi

col_count="$(awk 'NF>0{print NF; exit}' "$dest")"
if [[ "$col_count" != "2" && "$col_count" != "6" ]]; then
  echo "Unexpected normalized SMH column count: $col_count (expected 2 or 6)" >&2
  rm -f "$dest"
  exit 1
fi

sha="$(sha256sum "$dest" | awk '{print $1}')"

cat <<EOF
Imported SMH2016 reconstruction table:
  source: $src
  dest:   $dest
  rows:   $line_count
  cols:   $col_count
  sha256: $sha

Next step: update kernels/data/time/time_assets_manifest.json:
  - id: smh2016_reconstruction
  - status: in_repo
  - sha256: $sha
EOF
