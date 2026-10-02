#!/usr/bin/env bash
# Reclaim disk from Rust build artifacts without paying a full cold
# rebuild every time. Run periodically (cron/launchd) or by hand.
#
#   scripts/cargo-target-hygiene.sh [days]     # default 30
#
# - Installs cargo-sweep on demand and removes artifacts not touched in
#   <days> (stale fingerprints from old dependency versions).
# - Drops incremental caches, which are safe to delete anytime.
# - Prints sizes before/after.
set -euo pipefail

DAYS="${1:-30}"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

human() { du -sh "$1" 2>/dev/null | cut -f1 || echo '?'; }

echo "before: $(human "$TARGET_DIR")"
if [[ -d "$TARGET_DIR/debug/incremental" ]]; then
    rm -rf "$TARGET_DIR/debug/incremental"
fi
if [[ -d "$TARGET_DIR/release/incremental" ]]; then
    rm -rf "$TARGET_DIR/release/incremental"
fi

if ! command -v cargo-sweep >/dev/null 2>&1; then
    cargo install cargo-sweep --locked 2>/dev/null || true
fi
if command -v cargo-sweep >/dev/null 2>&1; then
    cargo sweep --time "$DAYS" 2>/dev/null || true
fi

echo "after:  $(human "$TARGET_DIR")"
