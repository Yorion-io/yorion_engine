#!/usr/bin/env bash
set -euo pipefail

echo "🔨 Building for x86_64 (Intel)..."

# Ensure target
rustup target add x86_64-apple-darwin

# Build
cargo build --release \
    --target x86_64-apple-darwin \
    --features uniffi-bindings \
    --no-default-features \
    --manifest-path engine/Cargo.toml

# The artifact is at engine/target/x86_64-apple-darwin/release/libbs_calendar_core.a
mkdir -p dist/x86_64
cp engine/target/x86_64-apple-darwin/release/libbs_calendar_core.a dist/x86_64/
strip -x dist/x86_64/libbs_calendar_core.a

