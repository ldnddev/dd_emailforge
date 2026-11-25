#!/usr/bin/env bash
# install.sh — install dd_emailforge from a GitHub Release package, or
# build from source when run inside a clone.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash -s -- uninstall
#   ./install.sh                # clone: ask terminal, window, or both, then build
#   ./install.sh install        # same as above
#   ./install.sh --tui          # terminal binary only
#   ./install.sh --gui          # window only (does not replace the theme)
#   ./install.sh --both         # terminal and window
#   ./install.sh --from-release # download the terminal package for this machine
#   ./install.sh --from-source  # always cargo-build (clones the repo if needed)
#   ./install.sh uninstall      # ask which binaries to remove; leave the theme file
#   ./install.sh --help
#
#   INSTALL_GUI=1               # same as --both when no --tui/--gui/--both is passed
#
# Override defaults via env vars:
#   PREFIX=$HOME/.local                       # binary lives at $PREFIX/bin/dd_emailforge
#   CONFIG_DIR=${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev
#   XDG_DATA_HOME=$HOME/.local/share          # Linux launcher dir is $XDG_DATA_HOME/applications
#   VERSION=latest                            # or v0.9.10 / 0.9.10
#   REPO=ldnddev/dd_emailforge
#   DOWNLOAD_BASE=https://…/download          # optional mirror (also accepts file://)
#   GITHUB_TOKEN=…                            # optional; private repos / higher rate limits
#
# On Linux, install writes a desktop launcher for each binary it installs.
# The terminal launcher sets Terminal=true. The window launcher sets
# Terminal=false. Pass the same PREFIX, BIN_DIR, and XDG_DATA_HOME to
# uninstall that you used to install.
# Re-run safe: existing themes are left alone on install; the chosen binary
# and launcher are overwritten. Uninstall removes only that binary and its
# desktop file. It leaves the theme file, saved templates, and the
# last-opened path. The window needs Vulkan and Wayland or X11.

set -euo pipefail

# ---- config -----------------------------------------------------------------
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev}"
BIN_NAME="dd_emailforge"
GUI_NAME="dd_emailforge-gui"
THEME_FILE="dd_emailforge_theme.yml"
GUI_WMCLASS="dev.ldnddev.dd_emailforge"
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

# ldnddev wordmark, half-block art of the white-on-black mark.
# Printed only on a terminal so a captured log stays plain text.
print_mark() {
    [ -t 1 ] || return 0
    printf '\033[40;97m'
    cat <<'EOF'

█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█
█  ██                ██                         ██  █
█  ██                ██                         ██  █
█  ██         ▄▄▄▄▄▄ ██     ▄▄▄▄▄▄       ▄▄▄▄▄▄ ██  █
█  ██       ▄██▀▀▀▀▀███  ▄██████████▄  ▄██▀▀▀▀▀███  █
█  ██      ██▀       ██  ██        ██ ██▀       ██  █
█  ██      ██        ██  ██        ██ ██        ██  █
█  ██      ▀█▄       ██  ██        ██ ▀█▄       ██  █
█  ▀█▄▄▄▄▄  ▀██▄▄▄▄██▀   ██        ██  ▀██▄▄▄▄██▀   █
█   ▀▀▀▀▀▀    ▀▀▀▀▀▀     ▀▀        ▀▀    ▀▀▀▀▀▀     █
█                                                   █
█                     ██                            █
█                     ██                            █
█              ▄▄▄▄▄▄ ██     ▄▄▄▄▄▄    ▄▄      ▄▄▄  █
█            ▄██▀▀▀▀▀███   ██▀▀▀▀███   ██     ▄██   █
█           ██▀       ██  ██    ██▀ ▄▄ ██    ▄██    █
█           ██        ██  ██  ▄██   ██ ██    ██     █
█           ▀█▄       ██  ██▄██▀   ▄█▀ ██  ▄██      █
█  ▄▄▄▄▄▄▄▄  ▀██▄▄▄▄██▀    ▀███▄▄▄██▀  ██▄██▀       █
█  ▀▀▀▀▀▀▀▀    ▀▀▀▀▀▀        ▀▀▀▀▀▀    ▀▀▀▀         █
▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
EOF
    printf '\033[0m\n'
}

# The device node is mode rw-rw-rw, so -r/-w stay true with no controlling
# terminal. Opening it is the check that matters.
have_tty() {
    { : <>/dev/tty; } 2>/dev/null
}

# Read 1, 2, or 3 from the terminal. stdout is tui, gui, or both.
# $1 is the verb shown in the question. $2 is the default number.
# Returns 1 when the terminal cannot be used. main captures stdout, and a
# failed redirect in that capture does not abort under set -e, so every
# /dev/tty failure has to be an explicit status.
ask_choice() {
    local verb=$1 default=$2 answer choice
    while true; do
        {
            printf '%s which program?\n' "$verb"
            printf '%s\n' "  1) terminal (dd_emailforge)"
            printf '%s\n' "  2) desktop window (dd_emailforge-gui)"
            printf '%s\n' "  3) both"
            printf 'Choice [%s]: ' "$default"
        } >/dev/tty 2>/dev/null || return 1
        if ! IFS= read -r answer </dev/tty 2>/dev/null; then
            printf '\n' >/dev/tty 2>/dev/null || true
            answer=$default
        fi
        choice=$(printf '%s' "$answer" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')
        case "$choice" in
            '') choice=$default ;;
        esac
        case "$choice" in
            1|tui|terminal) printf '%s\n' tui; return 0 ;;
            2|gui|window) printf '%s\n' gui; return 0 ;;
            3|both) printf '%s\n' both; return 0 ;;
            *) printf '%s\n' "Enter 1, 2, or 3." >/dev/tty 2>/dev/null || return 1 ;;
        esac
    done
}

# stdout is tui, gui, or both. Notices go to stderr so a capture stays one word.
choose_target() {
    local action=$1 explicit=$2
    if [ -n "$explicit" ]; then
        printf '%s\n' "$explicit"
        return 0
    fi
    if have_tty; then
        if [ "$action" = uninstall ]; then
            ask_choice "Remove" 3 && return 0
        else
            ask_choice "Install" 1 && return 0
        fi
    fi
    if [ "$action" = uninstall ]; then
        printf '%s\n' "No terminal for a prompt. Removing both programs." >&2
        printf '%s\n' both
    else
        printf '%s\n' "No terminal for a prompt. Installing the terminal app." >&2
        printf '%s\n' tui
    fi
}

usage() {
    cat <<'EOF'
install.sh — install dd_emailforge from a GitHub Release package, or
build from source when run inside a clone.

Usage:
  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_emailforge/main/install.sh | bash -s -- uninstall
  ./install.sh                # clone: ask terminal, window, or both
  ./install.sh install        # same as above
  ./install.sh --tui          # terminal binary only
  ./install.sh --gui          # window only
  ./install.sh --both         # terminal and window
  ./install.sh --from-release # download the terminal package for this machine
  ./install.sh --from-source  # always cargo-build (clones the repo if needed)
  ./install.sh uninstall      # ask which binaries to remove; leave the theme file
  ./install.sh --help

INSTALL_GUI=1 is the same as --both when --tui, --gui, and --both are absent.
With no terminal to answer, install selects the terminal app and uninstall
removes both.

Override defaults via env vars:
  PREFIX=$HOME/.local                       # binary lives at $PREFIX/bin/dd_emailforge
  CONFIG_DIR=${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev
  XDG_DATA_HOME=$HOME/.local/share          # Linux launcher dir is $XDG_DATA_HOME/applications
  VERSION=latest                            # or v0.9.10 / 0.9.10
  REPO=ldnddev/dd_emailforge
  DOWNLOAD_BASE=https://…/download          # optional mirror (also accepts file://)
  GITHUB_TOKEN=…                            # optional; private repos / higher rate limits

On Linux, install writes a desktop launcher for each binary it installs.
The window needs Vulkan and Wayland or X11. Pass the same
PREFIX, BIN_DIR, and XDG_DATA_HOME to uninstall that you used to install.
Uninstall removes the chosen binary and the launcher. It leaves
dd_emailforge_theme.yml, saved templates, and the last-opened path in place.

Re-run safe: existing themes are left alone on install; the binary and
launcher are overwritten.
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

# Desktop menus search XDG_DATA_HOME, then the system data dirs.
# A prefix under $HOME still publishes the launcher where the session looks.
apps_dir() {
    if [ -n "${XDG_DATA_HOME:-}" ]; then
        printf '%s/applications\n' "$XDG_DATA_HOME"
        return
    fi
    case "$PREFIX" in
        "$HOME"|"$HOME"/*)
            printf '%s/.local/share/applications\n' "$HOME"
            ;;
        *)
            printf '%s/share/applications\n' "$PREFIX"
            ;;
    esac
}

# Quote one desktop-entry value. Field codes treat % specially, so Exec
# doubles a literal percent. TryExec does not, so pass 0 for that key.
desktop_quote() {
    local s="$1" escape_percent="${2:-1}"
    case "$s" in
        *$'\n'*)
            red "Desktop path contains a newline"
            exit 1
            ;;
    esac
    s=${s//\\/\\\\}
    s=${s//\"/\\\"}
    s=${s//\`/\\\`}
    s=${s//\$/\\\$}
    if [ "$escape_percent" = 1 ]; then
        s=${s//%/%%}
    fi
    printf '"%s"' "$s"
}

write_desktop() {
    local dest="$1" bin="$2" name="$3" comment="$4" terminal="$5" startup="$6" wmclass="${7:-}"
    local exec_q try_q
    if [ -d "$dest" ]; then
        red "Refusing to overwrite directory $dest"
        exit 1
    fi
    exec_q="$(desktop_quote "$bin" 1)"
    try_q="$(desktop_quote "$bin" 0)"
    {
        printf '%s\n' "[Desktop Entry]"
        printf '%s\n' "Type=Application"
        printf '%s\n' "Version=1.0"
        printf '%s\n' "Name=${name}"
        printf '%s\n' "GenericName=Email template builder"
        printf '%s\n' "Comment=${comment}"
        printf '%s\n' "Exec=${exec_q}"
        printf '%s\n' "TryExec=${try_q}"
        printf '%s\n' "Icon=internet-mail"
        printf '%s\n' "Terminal=${terminal}"
        printf '%s\n' "Categories=Office;"
        printf '%s\n' "Keywords=email;mjml;template;"
        printf '%s\n' "StartupNotify=${startup}"
        if [ -n "$wmclass" ]; then
            printf '%s\n' "StartupWMClass=${wmclass}"
        fi
    } >"$dest"
    chmod 644 "$dest"
}

refresh_desktop_database() {
    local dir="$1"
    if [ -d "$dir" ] && command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$dir" >/dev/null 2>&1 || true
    fi
}

# Linux menus get one launcher per binary still present. Other systems skip
# this; the binary install is unchanged.
install_launchers() {
    local os dir
    os="$(uname -s)"
    if [ "$os" != "Linux" ]; then
        yellow "Desktop launchers are written on Linux. Skipped on $os."
        return 0
    fi
    dir="$(apps_dir)"
    mkdir -p "$dir"
    if [ -x "$BIN_DIR/$BIN_NAME" ]; then
        write_desktop \
            "$dir/${BIN_NAME}.desktop" \
            "$BIN_DIR/$BIN_NAME" \
            "dd_emailforge" \
            "Edit MJML email templates in the terminal" \
            "true" \
            "false"
        green "Launcher → $dir/${BIN_NAME}.desktop"
    fi
    if [ -x "$BIN_DIR/$GUI_NAME" ]; then
        write_desktop \
            "$dir/${GUI_NAME}.desktop" \
            "$BIN_DIR/$GUI_NAME" \
            "dd_emailforge GUI" \
            "Edit MJML email templates in a window" \
            "false" \
            "true" \
            "$GUI_WMCLASS"
        green "Launcher → $dir/${GUI_NAME}.desktop"
    fi
    refresh_desktop_database "$dir"
}

install_binary() {
    local src_bin="$1" name="$2"
    [ -f "$src_bin" ] || { red "Binary not found at $src_bin"; exit 1; }
    chmod +x "$src_bin" 2>/dev/null || true
    mkdir -p "$BIN_DIR"
    install_file 0755 "$src_bin" "$BIN_DIR/$name"
    green "Installed $name → $BIN_DIR/$name"
}

# Copy the theme only when it is missing. The window install does not call this.
install_theme_if_missing() {
    local src_theme="${1:-}"
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

install_payload() {
    local src_bin="$1"
    local src_theme="${2:-}"
    install_binary "$src_bin" "$BIN_NAME"
    install_theme_if_missing "$src_theme"
}

built_bin() {
    local repo_root="$1" name="$2"
    if [ -n "${CARGO_TARGET_DIR:-}" ]; then
        printf '%s/release/%s\n' "$CARGO_TARGET_DIR" "$name"
    else
        printf '%s/target/release/%s\n' "$repo_root" "$name"
    fi
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
    case "${1:-tui}" in
        gui)  green "Done. Try:  $GUI_NAME" ;;
        both) green "Done. Try:  $BIN_NAME    or    $GUI_NAME" ;;
        *)    green "Done. Try:  $BIN_NAME" ;;
    esac
    if [ "${1:-tui}" != tui ]; then
        green "The window needs Vulkan and Wayland or X11."
    fi
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
    cyan "Cloning $REPO ($ref)…" >&2
    git clone --depth 1 --branch "$ref" "https://github.com/${REPO}.git" "$workdir/src"
    printf '%s\n' "$workdir/src"
}

build_tui_from_source() {
    local repo_root="$1" src_bin src_theme
    require cargo
    [ -f "$repo_root/Cargo.toml" ] || {
        red "No Cargo.toml in $repo_root — cannot build from source."
        exit 1
    }
    cyan "Building $BIN_NAME (release)…"
    (cd "$repo_root" && cargo build --release -p dd_emailforge)
    src_bin="$(built_bin "$repo_root" "$BIN_NAME")"
    [ -x "$src_bin" ] || { red "Build did not produce $src_bin"; exit 1; }
    src_theme=""
    [ -f "$repo_root/$THEME_FILE" ] && src_theme="$repo_root/$THEME_FILE"
    install_binary "$src_bin" "$BIN_NAME"
    install_theme_if_missing "$src_theme"
}

build_gui_from_source() {
    local repo_root="$1" src_bin
    require cargo
    [ -f "$repo_root/Cargo.toml" ] || {
        red "No Cargo.toml in $repo_root — cannot build from source."
        exit 1
    }
    cyan "Building $GUI_NAME (release)…"
    cyan "The window needs Vulkan and Wayland or X11."
    (cd "$repo_root" && cargo build --release -p dd_emailforge_gui)
    src_bin="$(built_bin "$repo_root" "$GUI_NAME")"
    [ -x "$src_bin" ] || { red "Build did not produce $src_bin"; exit 1; }
    install_binary "$src_bin" "$GUI_NAME"
}

# Window-only does not copy or replace the theme. tui and both copy it only
# when the destination file is missing.
build_from_source() {
    local repo_root="$1" target="$2"
    case "$target" in
        tui)  build_tui_from_source "$repo_root" ;;
        gui)  build_gui_from_source "$repo_root" ;;
        both)
            build_tui_from_source "$repo_root"
            build_gui_from_source "$repo_root"
            ;;
        *)
            red "Unknown install target: $target"
            exit 1
            ;;
    esac
}

# ---- subcommands ------------------------------------------------------------
take_source_tree() {
    local repo_root="$1"
    if [ -n "$repo_root" ]; then
        printf '%s\n' "$repo_root"
        return 0
    fi
    clone_source
}

drop_clone() {
    if [ -n "${CLONE_CLEANUP:-}" ]; then
        rm -rf "$CLONE_CLEANUP"
        trap - EXIT
        CLONE_CLEANUP=""
    fi
}

do_install() {
    local mode="$1" target="$2" repo_root="" clone_root=""
    repo_root="$(find_repo_root || true)"

    if [ "$mode" = auto ]; then
        if [ -n "$repo_root" ]; then
            mode=source
        else
            mode=release
        fi
    fi

    # Release archives are the terminal app. The window is always built here.
    if [ "$mode" = release ] && [ "$target" = gui ]; then
        yellow "The release package is the terminal app. Building the window from source."
        mode=source
    fi

    if [ "$mode" = release ]; then
        if try_install_prebuilt; then
            if [ "$target" = both ]; then
                yellow "The release package is the terminal app. Building the window from source."
                clone_root="$(take_source_tree "$repo_root")"
                if [ "$clone_root" != "$repo_root" ]; then
                    repo_root="$clone_root"
                    CLONE_CLEANUP="$(dirname "$clone_root")"
                    trap 'rm -rf "${CLONE_CLEANUP:-}"' EXIT
                fi
                build_gui_from_source "$repo_root"
                drop_clone
            fi
            install_launchers
            path_note "$target"
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

    build_from_source "$repo_root" "$target"
    drop_clone
    install_launchers
    path_note "$target"
}

do_uninstall() {
    local target="${1:-both}"
    local theme_dst="$CONFIG_DIR/$THEME_FILE"
    local removed_any=0
    local apps=""

    remove_path() {
        local path="$1"
        if [ -f "$path" ] || [ -L "$path" ]; then
            rm -f "$path"
            green "Removed $path"
            removed_any=1
        else
            yellow "No file at $path"
        fi
    }

    case "$target" in
        tui) remove_path "$BIN_DIR/$BIN_NAME" ;;
        gui) remove_path "$BIN_DIR/$GUI_NAME" ;;
        both)
            remove_path "$BIN_DIR/$BIN_NAME"
            remove_path "$BIN_DIR/$GUI_NAME"
            ;;
        *)
            red "Unknown uninstall target: $target"
            exit 1
            ;;
    esac

    if [ "$(uname -s)" = "Linux" ]; then
        apps="$(apps_dir)"
        case "$target" in
            tui) remove_path "$apps/${BIN_NAME}.desktop" ;;
            gui) remove_path "$apps/${GUI_NAME}.desktop" ;;
            both)
                remove_path "$apps/${BIN_NAME}.desktop"
                remove_path "$apps/${GUI_NAME}.desktop"
                ;;
        esac
        refresh_desktop_database "$apps"
    fi

    if [ "$removed_any" -eq 0 ]; then
        yellow "Nothing to uninstall."
    else
        green ""
        green "Uninstall complete."
    fi
    yellow "Left $theme_dst in place."
    yellow "Left saved templates and the last-opened path in place."
}

# ---- dispatch ---------------------------------------------------------------
main() {
    print_mark
    local cmd="install"
    local mode="auto"
    local target=""
    while [ "$#" -gt 0 ]; do
        case "$1" in
            install) cmd="install" ;;
            uninstall|remove|--uninstall) cmd="uninstall" ;;
            --from-release|--prebuilt) mode="release" ;;
            --from-source) mode="source" ;;
            --tui|--gui|--both)
                if [ -n "$target" ]; then
                    red "Pass only one of --tui, --gui, or --both."
                    exit 1
                fi
                target="${1#--}"
                ;;
            -h|--help|help) usage; exit 0 ;;
            *)
                red "Unknown command: $1"
                usage
                exit 1
                ;;
        esac
        shift
    done
    if [ -z "$target" ] && [ "${INSTALL_GUI:-}" = "1" ]; then
        target=both
    fi
    target="$(choose_target "$cmd" "$target")"
    case "$cmd" in
        install) do_install "$mode" "$target" ;;
        uninstall) do_uninstall "$target" ;;
        *) red "Unknown command: $cmd"; usage; exit 1 ;;
    esac
}

# `curl | bash` reads this file from stdin. BASH_SOURCE is unset then, and
# `set -u` aborts on a bare ${BASH_SOURCE[0]}. An empty source means the
# file is being executed, not sourced.
if [[ -z "${BASH_SOURCE[0]:-}" || "${BASH_SOURCE[0]:-}" == "$0" ]]; then
    main "$@"
fi
