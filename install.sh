#!/usr/bin/env bash
set -euo pipefail

REPO="modoterra/omafetch"
BIN="omafetch"
TEMP_DIR=""

die() {
  printf 'install.sh: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Install omafetch from GitHub Releases.

Usage:
  install.sh [--prefix DIR] [--version VERSION]
  install.sh --uninstall [--prefix DIR]
  install.sh --help

Default prefix: ~/.local
Default version: latest (installs to ~/.local/bin/omafetch)

Examples:
  curl -fsSL https://raw.githubusercontent.com/modoterra/omafetch/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/modoterra/omafetch/main/install.sh | bash -s -- --prefix /usr/local
  curl -fsSL https://raw.githubusercontent.com/modoterra/omafetch/main/install.sh | bash -s -- --version 0.1.0
  curl -fsSL https://raw.githubusercontent.com/modoterra/omafetch/main/install.sh | bash -s -- --uninstall
EOF
}

normalize_version() {
  local version="${1#v}"
  if [[ ! "$version" =~ ^[0-9][0-9A-Za-z._-]*$ ]]; then
    die "invalid version: $1"
  fi
  printf '%s\n' "$version"
}

artifact_target_from() {
  local os="$1"
  local arch="$2"

  case "$os" in
    Linux) ;;
    *)
      die "prebuilt install supports Linux only (got ${os})"
      ;;
  esac

  case "$arch" in
    x86_64 | amd64)
      printf '%s\n' "x86_64-unknown-linux-gnu"
      ;;
    *)
      die "no prebuilt binary for ${arch}; build with: cargo install --git https://github.com/${REPO} --locked"
      ;;
  esac
}

artifact_target() {
  artifact_target_from "$(uname -s)" "$(uname -m)"
}

artifact_basename() {
  local version="$1"
  local target="$2"
  printf '%s\n' "${BIN}-${version}-${target}"
}

destination_path() {
  local prefix="${1%/}"
  printf '%s\n' "${prefix}/bin/${BIN}"
}

parse_args() {
  PREFIX="${PREFIX:-$HOME/.local}"
  VERSION="${VERSION:-latest}"
  ACTION="install"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --prefix)
        [[ $# -ge 2 && -n "$2" ]] || die "--prefix requires a directory"
        PREFIX="$2"
        shift 2
        ;;
      --version)
        [[ $# -ge 2 && -n "$2" ]] || die "--version requires a value"
        VERSION="$2"
        shift 2
        ;;
      --uninstall)
        ACTION="uninstall"
        shift
        ;;
      -h | --help)
        ACTION="help"
        shift
        ;;
      *)
        die "unknown option: $1"
        ;;
    esac
  done

  [[ -n "$PREFIX" ]] || die "prefix must not be empty"
}

require_command() {
  local name="$1"
  command -v "$name" >/dev/null 2>&1 || die "missing required command: ${name}"
}

resolve_version() {
  local requested="$1"
  if [[ "$requested" != "latest" ]]; then
    normalize_version "$requested"
    return
  fi

  local url tag
  url="$(
    curl -fsSL --proto '=https' --tlsv1.2 -o /dev/null -w '%{url_effective}' \
      "https://github.com/${REPO}/releases/latest"
  )" || die "could not reach GitHub releases"

  tag="${url##*/}"
  if [[ -z "$tag" || "$tag" == "latest" ]]; then
    tag="$(
      curl -fsSL --proto '=https' --tlsv1.2 \
        "https://api.github.com/repos/${REPO}/releases/latest" \
        | sed -n 's/^[[:space:]]*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' \
        | head -n1
    )" || die "could not query GitHub releases API"
  fi

  [[ -n "$tag" && "$tag" != "latest" ]] || die "could not resolve latest omafetch release"
  normalize_version "$tag"
}

download() {
  local url="$1"
  local dest="$2"
  curl -fsSL --proto '=https' --tlsv1.2 --retry 3 --retry-delay 1 -o "$dest" "$url" \
    || die "failed to download ${url}"
}

verify_checksum() {
  local archive="$1"
  local checksum_file="$2"
  local expected actual

  [[ -f "$archive" ]] || die "missing archive: ${archive}"
  [[ -f "$checksum_file" ]] || die "missing checksum: ${checksum_file}"

  expected="$(awk 'NF { print $1; exit }' "$checksum_file")"
  [[ "$expected" =~ ^[0-9a-fA-F]{64}$ ]] || die "invalid checksum file: ${checksum_file}"
  actual="$(sha256sum "$archive" | awk '{ print $1 }')"
  [[ "$expected" == "$actual" ]] || die "checksum mismatch for $(basename "$archive")"
}

register_temp_dir() {
  TEMP_DIR="$1"
  trap cleanup_temp_dir EXIT
}

cleanup_temp_dir() {
  if [[ -n "${TEMP_DIR}" && -d "${TEMP_DIR}" ]]; then
    rm -rf "${TEMP_DIR}"
  fi
  TEMP_DIR=""
}

warn_if_not_on_path() {
  local bin_dir="$1"
  case ":${PATH}:" in
    *:"${bin_dir}":*) ;;
    *)
      printf 'note: %s is not on PATH\n' "$bin_dir"
      ;;
  esac
}

about_launcher_path() {
  local prefix="${1%/}"
  printf '%s\n' "${prefix}/bin/omarchy-launch-about"
}

write_about_launcher() {
  local dest="$1"
  cat > "$dest" <<'EOF'
#!/usr/bin/env bash
# Replace Omarchy's About launcher so the stock menu item opens omafetch.
exec omarchy-launch-or-focus-tui omafetch about
EOF
  chmod 755 "$dest"
}

uninstall() {
  local dest launcher
  dest="$(destination_path "$PREFIX")"
  launcher="$(about_launcher_path "$PREFIX")"
  if [[ -e "$dest" || -L "$dest" ]]; then
    rm -f "$dest" || die "cannot remove ${dest}"
    printf 'removed %s\n' "$dest"
  else
    printf 'omafetch is not installed at %s\n' "$dest"
  fi
  if [[ -e "$launcher" || -L "$launcher" ]]; then
    rm -f "$launcher" || die "cannot remove ${launcher}"
    printf 'removed %s\n' "$launcher"
  fi
}

install_release() {
  local version target name dest work archive checksum url_base extracted

  require_command curl
  require_command tar
  require_command sha256sum
  require_command install
  require_command uname
  require_command mktemp

  version="$(resolve_version "$VERSION")"
  target="$(artifact_target)"
  name="$(artifact_basename "$version" "$target")"
  dest="$(destination_path "$PREFIX")"
  url_base="https://github.com/${REPO}/releases/download/v${version}"

  work="$(mktemp -d)" || die "could not create temporary directory"
  register_temp_dir "$work"

  archive="${work}/${name}.tar.gz"
  checksum="${archive}.sha256"
  download "${url_base}/${name}.tar.gz" "$archive"
  download "${url_base}/${name}.tar.gz.sha256" "$checksum"
  verify_checksum "$archive" "$checksum"

  tar -xzf "$archive" -C "$work"
  extracted="${work}/${name}/${BIN}"
  [[ -f "$extracted" && -x "$extracted" ]] || die "archive did not contain ${name}/${BIN}"

  mkdir -p "$(dirname "$dest")" || die "cannot create $(dirname "$dest")"
  [[ -w "$(dirname "$dest")" ]] || die "cannot write to $(dirname "$dest"); choose another --prefix or rerun with write access"
  install -Dm755 "$extracted" "$dest"
  write_about_launcher "$(about_launcher_path "$PREFIX")"

  "$dest" list >/dev/null || die "installed ${dest}, but omafetch list failed"
  warn_if_not_on_path "$(dirname "$dest")"
  printf 'installed %s %s\n' "$BIN" "$dest"
}

main() {
  parse_args "$@"
  case "$ACTION" in
    help)
      usage
      ;;
    uninstall)
      uninstall
      ;;
    install)
      install_release
      ;;
    *)
      die "internal error: unknown action ${ACTION}"
      ;;
  esac
}

# BASH_SOURCE is unset when the script is piped to bash (curl | bash).
if [[ -z "${BASH_SOURCE[0]:-}" || "${BASH_SOURCE[0]:-}" == "$0" ]]; then
  main "$@"
fi
