#!/usr/bin/env bash
set -euo pipefail

# This script generates Swift bindings using uniffi-bindgen.
# In CI, we use 'cargo binstall uniffi_bindgen' for speed.
# Locally, it falls back to 'cargo run' if not installed.

echo "🔧 Generating Swift bindings..."

mkdir -p dist/swift

# We use 'cargo run' from the workspace. 
# We avoid '--release' because LTO makes it extremely slow to compile the tool.
# Debug mode is fast enough for binding generation.
BINDGEN_CMD="cargo run --manifest-path engine/Cargo.toml --features uniffi-bindings --bin uniffi-bindgen --"

# Generate Swift bindings
$BINDGEN_CMD generate engine/src/uniffi.udl \
    --language swift \
    --out-dir dist/swift \
    --no-format

echo "✅ Swift bindings generated in dist/swift"
