#!/usr/bin/env bash
# Recapture tutorial screenshots from the live TUI renderer.
#
# Usage (from repo root or anywhere):
#   ./docs/capture.sh
#
# Requires: cargo, and a Chromium/Chrome binary for PNG output.
# Optional: EMAILFORGE_TUTORIAL_SHOTS, CHROME

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
shot_dir="${EMAILFORGE_TUTORIAL_SHOTS:-$root/docs/images}"
mkdir -p "$shot_dir"

chrome="${CHROME:-}"
if [ -z "$chrome" ]; then
    for c in chromium google-chrome chromium-browser google-chrome-stable; do
        if command -v "$c" >/dev/null 2>&1; then
            chrome="$c"
            break
        fi
    done
fi
if [ -z "$chrome" ]; then
    echo "No Chromium/Chrome on PATH. Set CHROME to the binary." >&2
    exit 1
fi

echo "Writing HTML frames via cargo test…"
(
    cd "$root"
    EMAILFORGE_TUTORIAL_SHOTS="$shot_dir" cargo test --offline --quiet \
        tui::tutorial_shots::capture_tutorial_frames -- --ignored --nocapture --exact
)

shopt -s nullglob
frames=("$shot_dir"/_frame-*.html)
if [ ${#frames[@]} -eq 0 ]; then
    echo "No _frame-*.html written to $shot_dir" >&2
    exit 1
fi

for html in "${frames[@]}"; do
    base="$(basename "$html")"
    name="${base#_frame-}"
    name="${name%.html}"
    png="$shot_dir/${name}.png"
    echo "Screenshot $base → $(basename "$png")"
    "$chrome" \
        --headless=new \
        --disable-gpu \
        --hide-scrollbars \
        --force-device-scale-factor=1 \
        --window-size=900,590 \
        --allow-file-access-from-files \
        --screenshot="$png" \
        "file://$html" >/dev/null 2>&1
    rm -f "$html"
done

echo "Done. PNGs in $shot_dir"
ls -1 "$shot_dir"/*.png
