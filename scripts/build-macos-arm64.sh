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

echo "🔧 Generating Swift bindings..."
# Generate Bindings (using the same cargo run invocation which is convenient here)
# We use x86 or arm64 runner, but 'cargo run' will compile for HOST architecture to run the tool.
# This might double-compile if host != target, but usually on macos-latest (arm64/x86) it's fine.
cargo run --release --manifest-path engine/Cargo.toml \
    --features uniffi-bindings \
    --bin uniffi-bindgen \
    -- generate engine/src/uniffi.udl --language swift --out-dir dist/swift --no-format

# Artifacts:
# - engine/target/aarch64-apple-darwin/release/libbs_calendar_core.a
# - dist/swift/bs_calendar_core.swift

mkdir -p dist/aarch64
cp engine/target/aarch64-apple-darwin/release/libbs_calendar_core.a dist/aarch64/

