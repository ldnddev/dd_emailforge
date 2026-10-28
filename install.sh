#!/usr/bin/env bash
# install.sh — install dd_emailforge from a GitHub Release package, or
# build from source when run inside a clone.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash -s -- uninstall
#   ./install.sh                # clone: cargo build --release and install
#   ./install.sh install        # same as above
#   ./install.sh --from-release # download the package for this machine
#   ./install.sh --from-source  # always cargo-build (clones the repo if needed)
#   ./install.sh uninstall      # remove the binary + theme + empty dirs
#   ./install.sh --help
#
# Override defaults via env vars:
#   PREFIX=$HOME/.local                       # binary lives at $PREFIX/bin/dd_emailforge
#   CONFIG_DIR=${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev
#   VERSION=latest                            # or v0.9.7 / 0.9.7
#   REPO=ldnddev/dd_emailforge
#   DOWNLOAD_BASE=https://…/download          # optional mirror (also accepts file://)
#   GITHUB_TOKEN=…                            # optional; private repos / higher rate limits
#
# Re-run safe: existing themes are left alone on install; the binary is overwritten.

set -euo pipefail

# ---- config -----------------------------------------------------------------
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev}"
BIN_NAME="dd_emailforge"
THEME_FILE="dd_emailforge_theme.yml"
LIBRARY_DIR="$CONFIG_DIR/dd_emailforge/templates"
REPO="${REPO:-ldnddev/dd_emailforge}"
VERSION="${VERSION:-latest}"
DEFAULT_BRANCH="${DEFAULT_BRANCH:-main}"

# ---- pretty -----------------------------------------------------------------
if [ -t 1 ]; then
    cyan()   { printf '\033[36m%s\033[0m\n' "$*"; }
    green()  { printf '\033[32m%s\033[0m\n' "$*"; }
    yellow() { printf '\033[33m%s\033[0m\n' "$*"; }
    red()    { printf '\033[31m%s\033[0m\n' "$*" >&2; }
else
    cyan()   { printf '%s\n' "$*"; }
    green()  { printf '%s\n' "$*"; }
    yellow() { printf '%s\n' "$*"; }
    red()    { printf '%s\n' "$*" >&2; }
fi

require() {
    command -v "$1" >/dev/null 2>&1 || { red "Required command not found: $1"; exit 1; }
}

usage() {
    cat <<'EOF'
install.sh — install dd_emailforge from a GitHub Release package, or
build from source when run inside a clone.

Usage:
  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash -s -- uninstall
  ./install.sh                # clone: cargo build --release and install
  ./install.sh install        # same as above
  ./install.sh --from-release # download the package for this machine
  ./install.sh --from-source  # always cargo-build (clones the repo if needed)
  ./install.sh uninstall      # remove the binary + theme + empty dirs
  ./install.sh --help

Override defaults via env vars:
  PREFIX=$HOME/.local                       # binary lives at $PREFIX/bin/dd_emailforge
  CONFIG_DIR=${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev
  VERSION=latest                            # or v0.9.7 / 0.9.7
  REPO=ldnddev/dd_emailforge
  DOWNLOAD_BASE=https://…/download          # optional mirror (also accepts file://)
  GITHUB_TOKEN=…                            # optional; private repos / higher rate limits

Re-run safe: existing themes are left alone on install; the binary is overwritten.
EOF
}

normalize_version() {
    local v="$1"
    case "$v" in
        latest|"") printf 'latest\n' ;;
        v*)        printf '%s\n' "$v" ;;
        *)         printf 'v%s\n' "$v" ;;
    esac
}

# Rust target triple that matches the GitHub Release archives.
detect_target() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"
    case "$os" in
        linux)  os="unknown-linux-musl" ;;
        darwin) os="apple-darwin" ;;
        *)
            red "Unsupported OS: $(uname -s). Supported: Linux, macOS."
            exit 1
            ;;
    esac
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            red "Unsupported architecture: $arch. Supported: x86_64, aarch64."
            exit 1
            ;;
    esac
    printf '%s-%s\n' "$arch" "$os"
}

# When piped (`curl | bash`), BASH_SOURCE is empty — there is no local clone.
find_repo_root() {
    local src="${BASH_SOURCE[0]-}"
    if [ -n "$src" ] && [ -f "$src" ]; then
        local dir
        dir="$(cd "$(dirname "$src")" && pwd)"
        if [ -f "$dir/Cargo.toml" ]; then
            printf '%s\n' "$dir"
            return 0
        fi
    fi
    return 1
}

curl_get() {
    local dest="$1" url="$2"
    if [ -n "${GITHUB_TOKEN:-}" ]; then
        curl -fsSL --retry 3 --retry-delay 1 \
            -H "Authorization: Bearer ${GITHUB_TOKEN}" \
            -H "Accept: application/octet-stream" \
            -o "$dest" "$url"
    else
        curl -fsSL --retry 3 --retry-delay 1 -o "$dest" "$url"
    fi
}

release_base_url() {
    if [ -n "${DOWNLOAD_BASE:-}" ]; then
        printf '%s\n' "${DOWNLOAD_BASE%/}"
        return 0
    fi
    local ver
    ver="$(normalize_version "$VERSION")"
    if [ "$ver" = latest ]; then
        printf 'https://github.com/%s/releases/latest/download\n' "$REPO"
    else
        printf 'https://github.com/%s/releases/download/%s\n' "$REPO" "$ver"
    fi
}

# Verify $archive against the sha256 line that names it. A missing tool, a
# missing line, or a mismatch aborts. `sha256sum -c` is not used: a checksum
# file can name other paths, and those must not be hashed or written.
verify_sha256() {
    local dir="$1" archive="$2" sumfile="$3"
    local expected actual expected_l actual_l
    expected="$(awk -v name="$archive" '
        NF >= 2 {
            hash = $1
            file = $2
            sub(/^\*/, "", file)
            if (file == name) {
                print hash
                exit
            }
        }
    ' "$dir/$sumfile" | tr -d "\r")"
    if [ "$(printf '%s' "$expected" | wc -c | tr -d '[:space:]')" -ne 64 ] \
        || printf '%s' "$expected" | grep -q '[^0-9A-Fa-f]'; then
        red "Checksum file has no sha256 line for $archive — aborting."
        return 1
    fi
    if command -v sha256sum >/dev/null 2>&1; then
        actual="$(sha256sum "$dir/$archive" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        actual="$(shasum -a 256 "$dir/$archive" | awk '{print $1}')"
    else
        red "No sha256sum or shasum found — refusing to install $archive without verification."
        return 1
    fi
    expected_l="$(printf '%s' "$expected" | tr '[:upper:]' '[:lower:]')"
    actual_l="$(printf '%s' "$actual" | tr '[:upper:]' '[:lower:]')"
    if [ "$actual_l" != "$expected_l" ]; then
        red "Checksum mismatch for $archive — aborting."
        return 1
    fi
}

# Release archives are flat: the binary, the theme, and LICENSE. Anything else
# (absolute paths, "..", extra files, directories) is rejected before extract.
extract_release_archive() {
    local archive="$1" dest="$2"
    local member
    local wanted=""
    while IFS= read -r member; do
        member="${member%$'\r'}"
        [ -n "$member" ] || continue
        case "$member" in
            "$BIN_NAME"|"$THEME_FILE"|LICENSE) ;;
            *)
                red "Refusing archive member '$member'"
                return 1
                ;;
        esac
        wanted="$wanted $member"
    done < <(tar -tzf "$archive")
    case " $wanted " in
        *" $BIN_NAME "*) ;;
        *)
            red "Archive did not contain $BIN_NAME"
            return 1
            ;;
    esac
    # shellcheck disable=SC2086
    tar -xzf "$archive" -C "$dest" -- $wanted
    if [ -L "$dest/$BIN_NAME" ] || [ ! -f "$dest/$BIN_NAME" ]; then
        red "Archive $BIN_NAME is not a regular file — aborting."
        return 1
    fi
}

install_file() {
    local mode="$1" src="$2" dst="$3"
    if command -v install >/dev/null 2>&1; then
        install -m "$mode" "$src" "$dst"
    else
        cp "$src" "$dst"
        chmod "$mode" "$dst"
    fi
}

install_payload() {
    local src_bin="$1"
    local src_theme="${2:-}"

    [ -f "$src_bin" ] || { red "Binary not found at $src_bin"; exit 1; }
    chmod +x "$src_bin" 2>/dev/null || true

    mkdir -p "$BIN_DIR"
    install_file 0755 "$src_bin" "$BIN_DIR/$BIN_NAME"
    green "Installed $BIN_NAME → $BIN_DIR/$BIN_NAME"

    mkdir -p "$CONFIG_DIR"
    local theme_dst="$CONFIG_DIR/$THEME_FILE"
    if [ -f "$theme_dst" ]; then
        yellow "Theme already exists at $theme_dst — leaving it alone."
    elif [ -n "$src_theme" ] && [ -f "$src_theme" ]; then
        install_file 0644 "$src_theme" "$theme_dst"
        green "Installed default theme → $theme_dst"
    else
        yellow "No $THEME_FILE in the package — using the binary's built-in theme."
    fi

    mkdir -p "$LIBRARY_DIR"
    green "Template library dir → $LIBRARY_DIR"
}

path_note() {
    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            yellow ""
            yellow "Note: $BIN_DIR is not on \$PATH. Add it to your shell rc:"
            yellow "    export PATH=\"$BIN_DIR:\$PATH\""
            ;;
    esac
    green ""
    green "Done. Try:  $BIN_NAME"
}

# Download, verify, and install a matching GitHub Release archive into $1.
# Returns 1 if the archive is missing (caller may fall back to source).
install_prebuilt_into() {
    local tmp="$1"
    local target archive url sumfile sum_url src_bin src_theme
    target="$(detect_target)"
    archive="${BIN_NAME}-${target}.tar.gz"
    sumfile="${BIN_NAME}-${target}.sha256"
    url="$(release_base_url)/${archive}"
    # taiki-e/upload-rust-binary-action names checksums $bin-$target.sha256
    sum_url="$(release_base_url)/${sumfile}"

    cyan "Looking for $archive ($(normalize_version "$VERSION"))…"

    if ! curl_get "$tmp/$archive" "$url"; then
        yellow "No prebuilt package at $url"
        return 1
    fi

    if ! curl_get "$tmp/$sumfile" "$sum_url"; then
        red "No checksum file at $sum_url — refusing to install without verification."
        exit 1
    fi
    verify_sha256 "$tmp" "$archive" "$sumfile" || exit 1

    if ! extract_release_archive "$tmp/$archive" "$tmp"; then
        red "Failed to extract $archive"
        exit 1
    fi

    src_bin="$tmp/$BIN_NAME"
    src_theme=""
    if [ -f "$tmp/$THEME_FILE" ] && [ ! -L "$tmp/$THEME_FILE" ]; then
        src_theme="$tmp/$THEME_FILE"
    fi

    cyan "Installing $BIN_NAME for $target…"
    install_payload "$src_bin" "$src_theme"
    return 0
}

# install_prebuilt_into calls `exit` when the checksum or archive is bad, so
# this has to be global: a local would already be gone when the EXIT trap runs.
PREBUILT_TMP=""

cleanup_prebuilt_tmp() {
    if [ -n "${PREBUILT_TMP:-}" ]; then
        rm -rf "$PREBUILT_TMP"
        PREBUILT_TMP=""
    fi
}

# Returns 0 if a matching GitHub Release archive was downloaded and installed.
# A missing archive returns 1 so auto mode can build from source. A bad
# checksum or archive still exits 1, and the download directory is removed.
try_install_prebuilt() {
    require curl
    require tar
    require mktemp
    require uname

    PREBUILT_TMP="$(mktemp -d)"
    trap cleanup_prebuilt_tmp EXIT
    if install_prebuilt_into "$PREBUILT_TMP"; then
        cleanup_prebuilt_tmp
        trap - EXIT
        return 0
    fi
    cleanup_prebuilt_tmp
    trap - EXIT
    return 1
}

clone_source() {
    require git
    require mktemp
    local ver workdir ref
    ver="$(normalize_version "$VERSION")"
    if [ "$ver" = latest ]; then
        ref="${REF:-$DEFAULT_BRANCH}"
    else
        ref="${REF:-$ver}"
    fi
    workdir="$(mktemp -d)"
    cyan "Cloning $REPO ($ref)…"
    git clone --depth 1 --branch "$ref" "https://github.com/${REPO}.git" "$workdir/src"
    printf '%s\n' "$workdir/src"
}

build_from_source() {
    local repo_root="$1"
    require cargo

    [ -f "$repo_root/Cargo.toml" ] || {
        red "No Cargo.toml in $repo_root — cannot build from source."
        exit 1
    }

    cyan "Building $BIN_NAME (release)…"
    (cd "$repo_root" && cargo build --release)

    local src_bin="$repo_root/target/release/$BIN_NAME"
    [ -x "$src_bin" ] || { red "Build did not produce $src_bin"; exit 1; }

    local src_theme=""
    [ -f "$repo_root/$THEME_FILE" ] && src_theme="$repo_root/$THEME_FILE"

    install_payload "$src_bin" "$src_theme"
}

# ---- subcommands ------------------------------------------------------------
do_install() {
    local mode="$1" repo_root="" clone_root=""
    repo_root="$(find_repo_root || true)"

    if [ "$mode" = auto ]; then
        if [ -n "$repo_root" ]; then
            mode=source
        else
            mode=release
        fi
    fi

    if [ "$mode" = release ]; then
        if try_install_prebuilt; then
            path_note
            return 0
        fi
        if [ "$1" = release ]; then
            red "No matching package for $(detect_target) at $(release_base_url)/"
            red "Retry with --from-source (needs Rust) or set VERSION to a published tag."
            exit 1
        fi
        yellow "Falling back to a source build (needs Rust / cargo)…"
        mode=source
    fi

    if [ -z "$repo_root" ]; then
        clone_root="$(clone_source)"
        repo_root="$clone_root"
        CLONE_CLEANUP="$(dirname "$clone_root")"
        trap 'rm -rf "${CLONE_CLEANUP:-}"' EXIT
    fi

    build_from_source "$repo_root"
    if [ -n "${CLONE_CLEANUP:-}" ]; then
        rm -rf "$CLONE_CLEANUP"
        trap - EXIT
        CLONE_CLEANUP=""
    fi
    path_note
}

do_uninstall() {
    local bin_dst="$BIN_DIR/$BIN_NAME"
    local theme_dst="$CONFIG_DIR/$THEME_FILE"
    local app_dir="$CONFIG_DIR/dd_emailforge"
    local removed_any=0

    if [ -f "$bin_dst" ] || [ -L "$bin_dst" ]; then
        rm -f "$bin_dst"
        green "Removed $bin_dst"
        removed_any=1
    else
        yellow "No binary at $bin_dst"
    fi

    if [ -f "$theme_dst" ] || [ -L "$theme_dst" ]; then
        rm -f "$theme_dst"
        green "Removed $theme_dst"
        removed_any=1
    else
        yellow "No theme at $theme_dst"
    fi

    if [ -d "$LIBRARY_DIR" ] && [ -z "$(ls -A "$LIBRARY_DIR")" ]; then
        rmdir "$LIBRARY_DIR"
        green "Removed empty $LIBRARY_DIR"
    fi
    if [ -d "$app_dir" ] && [ -z "$(ls -A "$app_dir")" ]; then
        rmdir "$app_dir"
        green "Removed empty $app_dir"
    fi

    # Remove config dir only if empty (don't clobber other tools' files)
    if [ -d "$CONFIG_DIR" ] && [ -z "$(ls -A "$CONFIG_DIR")" ]; then
        rmdir "$CONFIG_DIR"
        green "Removed empty $CONFIG_DIR"
    fi

    if [ "$removed_any" -eq 0 ]; then
        yellow "Nothing to uninstall."
    else
        green ""
        green "Uninstall complete."
    fi
}

# ---- dispatch ---------------------------------------------------------------
cmd="install"
mode="auto"

while [ "$#" -gt 0 ]; do
    case "$1" in
        install)              cmd="install" ;;
        uninstall|remove)     cmd="uninstall" ;;
        --from-release|--prebuilt) mode="release" ;;
        --from-source)        mode="source" ;;
        -h|--help|help)       usage; exit 0 ;;
        *)
            red "Unknown command: $1"
            usage
            exit 1
            ;;
    esac
    shift
done

case "$cmd" in
    install)          do_install "$mode" ;;
    uninstall)        do_uninstall ;;
    *) red "Unknown command: $cmd"; usage; exit 1 ;;
esac
