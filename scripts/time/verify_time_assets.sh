#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="$repo_root/kernels/data/time/time_assets_manifest.json"
smh="$repo_root/kernels/data/time/smh2016_reconstruction.tsv"
finals="$repo_root/kernels/data/finals2000A.all"

if [[ ! -f "$manifest" ]]; then
  echo "Missing manifest: $manifest" >&2
  exit 1
fi

echo "Manifest: $manifest"

if [[ -f "$smh" ]]; then
  sha="$(sha256sum "$smh" | awk '{print $1}')"
  echo "SMH table present: $smh"
  echo "SMH sha256: $sha"
else
  echo "SMH table missing: $smh"
fi

if [[ -f "$finals" ]]; then
  finals_sha="$(sha256sum "$finals" | awk '{print $1}')"
  echo "finals2000A.all present: $finals"
  echo "finals2000A.all sha256: $finals_sha"
else
  echo "finals2000A.all missing: $finals"
fi

if grep -q '"id": "smh2016_reconstruction"' "$manifest"; then
  echo "Manifest contains smh2016_reconstruction entry."
else
  echo "Manifest is missing smh2016_reconstruction entry." >&2
  exit 1
fi

if grep -q '"id": "iers_c04_1962_now"' "$manifest"; then
  echo "Manifest contains iers_c04_1962_now entry."
else
  echo "Manifest is missing iers_c04_1962_now entry." >&2
  exit 1
fi

if grep -q '"id": "finals2000a_daily_extended"' "$manifest"; then
  echo "Manifest contains finals2000a_daily_extended entry."
else
  echo "Manifest is missing finals2000a_daily_extended entry." >&2
  exit 1
fi

if grep -q '"id": "finals2000a_all"' "$manifest"; then
  echo "Manifest contains finals2000a_all entry."
else
  echo "Manifest is missing finals2000a_all entry." >&2
  exit 1
fi

manifest_smh_sha="$(
  awk '
    /"id": "smh2016_reconstruction"/ { in_obj=1 }
    in_obj && /"sha256":/ {
      gsub(/[", ]/, "", $2);
      print $2;
      exit;
    }
    in_obj && /}/ { in_obj=0 }
  ' "$manifest"
)"
if [[ -f "$smh" && -n "$manifest_smh_sha" && "$manifest_smh_sha" != "null" ]]; then
  if [[ "$manifest_smh_sha" != "$sha" ]]; then
    echo "SMH sha256 mismatch: manifest=$manifest_smh_sha file=$sha" >&2
    exit 1
  fi
fi

manifest_finals_sha="$(
  awk '
    /"id": "finals2000a_all"/ { in_obj=1 }
    in_obj && /"sha256":/ {
      gsub(/[", ]/, "", $2);
      print $2;
      exit;
    }
    in_obj && /}/ { in_obj=0 }
  ' "$manifest"
)"
if [[ -f "$finals" && -n "$manifest_finals_sha" && "$manifest_finals_sha" != "null" ]]; then
  if [[ "$manifest_finals_sha" != "$finals_sha" ]]; then
    echo "finals2000A.all sha256 mismatch: manifest=$manifest_finals_sha file=$finals_sha" >&2
    exit 1
  fi
fi

echo "Time asset verification checks completed."
