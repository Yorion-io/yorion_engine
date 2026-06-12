#!/usr/bin/env bash
set -euo pipefail

echo "🔨 Building for aarch64 (Apple Silicon)..."

# Ensure target
rustup target add aarch64-apple-darwin

# Build Lib
cargo build --release \
    --target aarch64-apple-darwin \
    --features uniffi-bindings \
    --no-default-features \
    --manifest-path engine/Cargo.toml

# Artifacts:
# - engine/target/aarch64-apple-darwin/release/libyorion_engine.a
# - dist/swift/yorion_engine.swift

mkdir -p dist/aarch64
cp engine/target/aarch64-apple-darwin/release/libyorion_engine.a dist/aarch64/
strip -x dist/aarch64/libyorion_engine.a

