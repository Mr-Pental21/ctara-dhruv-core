#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

ALLOWED_NODE_LICENSES="MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;Zlib"
ALLOWED_LICENSES_CSV="MIT,Apache-2.0,BSD-2-Clause,BSD-3-Clause,ISC,Zlib"
PYTHON_BIN="${PYTHON_BIN:-$(command -v python3.12 || command -v python3.11 || command -v python3 || command -v python || true)}"

if [[ -z "${PYTHON_BIN}" ]]; then
  echo "ERROR: python3/python is required for license scanning." >&2
  exit 1
fi

scan_rust() {
  if [[ -f "Cargo.toml" ]]; then
    echo "==> Running cargo-deny license check"
    cargo deny check licenses
  else
    echo "==> Skipping cargo-deny (no Cargo.toml in repository root)"
  fi
}

scan_node_wrapper() {
  local wrapper_dir="$1"
  echo "==> Scanning Node wrapper licenses in ${wrapper_dir}"

  if [[ ! -f "${wrapper_dir}/package.json" ]]; then
    return 0
  fi

  if [[ ! -f "${wrapper_dir}/package-lock.json" && ! -f "${wrapper_dir}/npm-shrinkwrap.json" ]]; then
    echo "ERROR: ${wrapper_dir} has package.json but no npm lockfile."
    echo "Use npm lockfiles so CI can run deterministic license scans."
    return 1
  fi

  (
    cd "${wrapper_dir}"
    npm ci --ignore-scripts
    npx --yes license-checker-rseidelsohn --production --onlyAllow "${ALLOWED_NODE_LICENSES}"
  )
}

scan_python_wrapper() {
  local wrapper_dir="$1"
  local requirements_file=""

  if [[ -f "${wrapper_dir}/requirements.lock.txt" ]]; then
    requirements_file="${wrapper_dir}/requirements.lock.txt"
  elif [[ -f "${wrapper_dir}/requirements.txt" ]]; then
    requirements_file="${wrapper_dir}/requirements.txt"
  elif [[ -f "${wrapper_dir}/pyproject.toml" ]]; then
    echo "ERROR: ${wrapper_dir} has pyproject.toml but no requirements lock file."
    echo "Add requirements.lock.txt (or requirements.txt) for deterministic license scanning."
    return 1
  else
    return 0
  fi

  echo "==> Scanning Python wrapper licenses in ${wrapper_dir}"
  local venv_dir
  local venv_python
  local venv_pip_licenses
  venv_dir="$(mktemp -d)"

  "${PYTHON_BIN}" -m venv "${venv_dir}"
  venv_python="${venv_dir}/bin/python"
  venv_pip_licenses="${venv_dir}/bin/pip-licenses"
  "${venv_python}" -m pip install --upgrade pip >/dev/null
  "${venv_python}" -m pip install pip-licenses >/dev/null
  "${venv_python}" -m pip install -r "${requirements_file}" >/dev/null

  local license_json
  license_json="$(mktemp)"
  "${venv_pip_licenses}" --format=json --with-license-file > "${license_json}"

  "${venv_python}" - "${license_json}" <<'PY'
import json
import re
import sys

path = sys.argv[1]
deny_re = re.compile(r"\b(agpl|gpl|lgpl|sspl|busl|bsl)\b", re.IGNORECASE)
ambiguous_re = re.compile(r"\b(unknown|proprietary|custom|other)\b", re.IGNORECASE)
allowed_re = re.compile(r"(mit|apache|bsd|isc|zlib)", re.IGNORECASE)

with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)

violations = []
for pkg in data:
    name = pkg.get("Name", "<unknown>")
    license_text = (pkg.get("License") or "").strip()

    if not license_text:
        violations.append(f"{name}: missing license")
        continue
    if deny_re.search(license_text):
        violations.append(f"{name}: denylisted license ({license_text})")
        continue
    if ambiguous_re.search(license_text):
        violations.append(f"{name}: ambiguous or non-approved license ({license_text})")
        continue
    if not allowed_re.search(license_text):
        violations.append(f"{name}: license not in allowlist ({license_text})")

if violations:
    print("Python license policy violations detected:")
    for v in violations:
        print(f"- {v}")
    sys.exit(1)
PY

  rm -f "${license_json}"
  rm -rf "${venv_dir}"
}

scan_go_wrapper() {
  local wrapper_dir="$1"
  local gomod="${wrapper_dir}/go.mod"

  if [[ ! -f "${gomod}" ]]; then
    return 0
  fi

  echo "==> Scanning Go wrapper licenses in ${wrapper_dir}"

  # Deterministic policy for now: stdlib-only Go wrapper.
  # If/when third-party modules are added, this gate must be expanded
  # with module-level license extraction and allowlist checks.
  if grep -Eq '^[[:space:]]*require[[:space:]]' "${gomod}"; then
    echo "ERROR: ${wrapper_dir} declares Go module dependencies."
    echo "Current policy for go-open requires stdlib-only dependencies."
    return 1
  fi
}

scan_elixir_wrapper() {
  local wrapper_dir="$1"

  if [[ ! -f "${wrapper_dir}/mix.exs" ]]; then
    return 0
  fi

  echo "==> Scanning Elixir wrapper licenses in ${wrapper_dir}"

  if ! command -v mix >/dev/null 2>&1; then
    echo "ERROR: ${wrapper_dir} has mix.exs but mix is unavailable."
    echo "Install Elixir/OTP so Hex dependencies can be resolved and scanned."
    return 1
  fi

  if [[ ! -f "${wrapper_dir}/mix.lock" ]]; then
    echo "ERROR: ${wrapper_dir} has mix.exs but no mix.lock."
    echo "Use mix.lock so CI can run deterministic license scans."
    return 1
  fi

  (
    cd "${wrapper_dir}"
    mix local.hex --force >/dev/null
    mix local.rebar --force >/dev/null
    mix deps.get >/dev/null
  )

  local dep_list
  dep_list="$(mktemp)"

  elixir --eval '
    path = hd(System.argv())
    {lock, _binding} = Code.eval_file(path)

    Enum.each(lock, fn {name, entry} ->
      case entry do
        {:hex, package, version, _checksum, _build_tools, _deps, repo, _inner_checksum} ->
          IO.puts("#{name}\t#{package}\t#{version}\t#{repo}")

        other ->
          IO.puts(:stderr, "ERROR: non-Hex dependency in mix.lock for #{inspect(name)}: #{inspect(other)}")
          System.halt(2)
      end
    end)
  ' "${wrapper_dir}/mix.lock" > "${dep_list}"

  "${PYTHON_BIN}" - "${dep_list}" "${ALLOWED_LICENSES_CSV}" <<'PY'
import json
import sys
import urllib.error
import urllib.parse
import urllib.request

deps_path = sys.argv[1]
allowed = {item.strip() for item in sys.argv[2].split(",") if item.strip()}

with open(deps_path, "r", encoding="utf-8") as handle:
    rows = [line.strip().split("\t") for line in handle if line.strip()]

violations = []
seen = set()

def fetch_json(url):
    req = urllib.request.Request(url, headers={"User-Agent": "ctara-dhruv-license-gate"})
    with urllib.request.urlopen(req) as response:
        return json.load(response)

for row in rows:
    if len(row) != 4:
        violations.append(f"malformed mix.lock entry: {row!r}")
        continue

    dep_name, package_name, version, repo = row
    key = (package_name, version)
    if key in seen:
        continue
    seen.add(key)

    if repo != "hexpm":
        violations.append(
            f"{dep_name}: unsupported Hex repo '{repo}' in mix.lock; only hexpm is supported"
        )
        continue

    package_url = f"https://hex.pm/api/packages/{urllib.parse.quote(package_name)}"
    release_url = f"{package_url}/releases/{urllib.parse.quote(version)}"

    try:
        package_data = fetch_json(package_url)
        release_data = fetch_json(release_url)
    except urllib.error.HTTPError as exc:
        violations.append(f"{package_name} {version}: failed to fetch Hex metadata ({exc})")
        continue
    except urllib.error.URLError as exc:
        violations.append(f"{package_name} {version}: failed to reach Hex API ({exc})")
        continue

    releases = package_data.get("releases") or []
    if not any(release.get("version") == version for release in releases):
        violations.append(f"{package_name} {version}: version not present in Hex package metadata")
        continue

    release_package_url = release_data.get("package_url")
    if release_package_url and release_package_url.rstrip("/") != package_data.get("url", "").rstrip("/"):
        violations.append(f"{package_name} {version}: release/package metadata mismatch")
        continue

    release_licenses = ((release_data.get("meta") or {}).get("licenses")) or []
    package_licenses = ((package_data.get("meta") or {}).get("licenses")) or []
    licenses = release_licenses or package_licenses

    if not licenses:
        violations.append(f"{package_name} {version}: missing license metadata in Hex API")
        continue

    disallowed = [license_name for license_name in licenses if license_name not in allowed]
    if disallowed:
        violations.append(
            f"{package_name} {version}: non-allowlisted license(s) {', '.join(disallowed)}"
        )

if violations:
    print("Elixir license policy violations detected:")
    for violation in violations:
        print(f"- {violation}")
    sys.exit(1)
PY

  rm -f "${dep_list}"
}

scan_wrappers() {
  if [[ ! -d "bindings" ]]; then
    echo "==> Skipping wrapper scans (no bindings directory)"
    return 0
  fi

  local wrapper
  while IFS= read -r wrapper; do
    scan_node_wrapper "${wrapper}"
    scan_python_wrapper "${wrapper}"
    scan_go_wrapper "${wrapper}"
    scan_elixir_wrapper "${wrapper}"
  done < <(find bindings -mindepth 1 -maxdepth 1 -type d | sort)
}

scan_rust
scan_wrappers

echo "==> License gate checks passed"
