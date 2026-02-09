#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCK_FILE="${ROOT_DIR}/kernels/manifest/de442s.lock"
DEST_DIR="${ROOT_DIR}/kernels/data"

usage() {
  cat <<'EOF'
Usage: fetch_kernels.sh [--lock-file <path>] [--dest-dir <path>]

Downloads kernel files from a lock file and verifies MD5 checksums.
Lock format: name|url|md5|checksum_source
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --lock-file)
      LOCK_FILE="$2"
      shift 2
      ;;
    --dest-dir)
      DEST_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ! -f "$LOCK_FILE" ]]; then
  echo "Lock file not found: $LOCK_FILE" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi

md5_for_file() {
  local file_path="$1"
  if command -v md5sum >/dev/null 2>&1; then
    md5sum "$file_path" | awk '{print tolower($1)}'
    return
  fi
  if command -v md5 >/dev/null 2>&1; then
    md5 -q "$file_path" | tr '[:upper:]' '[:lower:]'
    return
  fi
  echo "No MD5 tool found (md5sum or md5)." >&2
  return 1
}

mkdir -p "$DEST_DIR"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

echo "Using lock file: $LOCK_FILE"
echo "Destination dir: $DEST_DIR"

while IFS='|' read -r name url md5_expected checksum_source; do
  if [[ -z "${name}" || "${name:0:1}" == "#" ]]; then
    continue
  fi

  if [[ -z "${url}" || -z "${md5_expected}" ]]; then
    echo "Invalid lock entry for ${name}" >&2
    exit 1
  fi

  target_path="${DEST_DIR}/${name}"
  tmp_path="${tmp_dir}/${name}"

  echo "Downloading ${name}"
  curl -fsSL --retry 3 --proto '=https' --tlsv1.2 "${url}" -o "${tmp_path}"

  md5_actual="$(md5_for_file "${tmp_path}")"
  md5_expected="$(echo "${md5_expected}" | tr '[:upper:]' '[:lower:]')"
  if [[ "${md5_actual}" != "${md5_expected}" ]]; then
    echo "Checksum mismatch for ${name}" >&2
    echo "Expected: ${md5_expected}" >&2
    echo "Actual:   ${md5_actual}" >&2
    if [[ -n "${checksum_source}" ]]; then
      echo "Checksum source: ${checksum_source}" >&2
    fi
    exit 1
  fi

  mv "${tmp_path}" "${target_path}"
  echo "Verified ${name} (${md5_actual})"
done < "${LOCK_FILE}"

echo "Kernel download and verification complete."
