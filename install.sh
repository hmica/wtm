#!/usr/bin/env bash
# Build wtm in release mode and install it to ~/.cargo/bin.
set -euo pipefail

cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found — install Rust from https://rustup.rs" >&2
    exit 1
fi

echo "Installing wtm..."
cargo install --path . --force

echo
echo "Done. wtm installed to $(command -v wtm 2>/dev/null || echo "$HOME/.cargo/bin/wtm")"
echo "Make sure ~/.cargo/bin is on your PATH."
