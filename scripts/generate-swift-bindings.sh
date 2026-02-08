#!/usr/bin/env bash
set -euo pipefail

# This script generates Swift bindings using uniffi-bindgen.
# In CI, we use 'cargo binstall uniffi_bindgen' for speed.
# Locally, it falls back to 'cargo run' if not installed.

echo "🔧 Generating Swift bindings..."

mkdir -p dist/swift

# Check if uniffi-bindgen is in PATH, otherwise use cargo run
if command -v uniffi-bindgen &> /dev/null; then
    BINDGEN_CMD="uniffi-bindgen"
else
    echo "💡 uniffi-bindgen not found in PATH, falling back to cargo run (this will be slower local-only behavior)..."
    BINDGEN_CMD="cargo run --release --manifest-path engine/Cargo.toml --features uniffi-bindings --bin uniffi-bindgen --"
fi

# Generate Swift bindings
$BINDGEN_CMD generate engine/src/uniffi.udl \
    --language swift \
    --out-dir dist/swift \
    --no-format

echo "✅ Swift bindings generated in dist/swift"
