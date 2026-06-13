#!/usr/bin/env bash
set -euo pipefail

# Build WASM binaries for web, bundler, and nodejs targets
# Outputs to dist/wasm/

export PATH="$HOME/.cargo/bin:$PATH"

echo "🦀 Building WASM binaries..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create output directory
DIST_DIR="dist/wasm"
mkdir -p "$DIST_DIR"

# Change to engine directory
cd engine

# Install wasm-bindgen-cli if not available
if ! command -v wasm-bindgen &> /dev/null; then
    echo "📦 Installing wasm-bindgen-cli..."
    if command -v cargo-binstall &> /dev/null; then
        cargo binstall --no-confirm wasm-bindgen-cli
    else
        cargo install wasm-bindgen-cli
    fi
fi

# Check for wasm-opt (part of binaryen)
if ! command -v wasm-opt &> /dev/null; then
    # Try to find wasm-opt in wasm-pack cache recursively
    WASM_OPT_CACHE=$(find "$HOME/Library/Caches/.wasm-pack" -name wasm-opt -type f 2>/dev/null | head -n 1 || true)
    if [ -n "$WASM_OPT_CACHE" ]; then
        echo "🔍 Found wasm-opt in cache: $WASM_OPT_CACHE"
        WASM_OPT_BIN="$WASM_OPT_CACHE"
    else
        echo "⚠️ wasm-opt not found in PATH or cache, binaries will not be optimized"
        echo "💡 Tip: Install binaryen (brew install binaryen) for better performance"
        WASM_OPT_BIN=""
    fi
else
    WASM_OPT_BIN="wasm-opt"
fi

# 1. Build the Rust code once for the wasm32 target
echo -e "${BLUE}Compiling Rust to WASM...${NC}"
cargo build --target wasm32-unknown-unknown --release --features wasm --no-default-features

WASM_FILE="target/wasm32-unknown-unknown/release/yorion_engine.wasm"

# Function to generate bindings and optimize
generate_target() {
    local TARGET_TYPE=$1
    local EXTRA_ARGS=$2
    
    echo -e "${BLUE}Generating bindings for $TARGET_TYPE...${NC}"
    wasm-bindgen "$WASM_FILE" \
        --target "$TARGET_TYPE" \
        --out-dir "../$DIST_DIR/$TARGET_TYPE" \
        --typescript \
        $EXTRA_ARGS

    if [ -n "$WASM_OPT_BIN" ]; then
        echo "⚡ Optimizing $TARGET_TYPE WASM..."
        "$WASM_OPT_BIN" -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
            "../$DIST_DIR/$TARGET_TYPE/yorion_engine_bg.wasm" \
            -o "../$DIST_DIR/$TARGET_TYPE/yorion_engine_bg.wasm"
    fi
}

# 2. Generate bindings for each target (very fast)
generate_target "web" ""
generate_target "bundler" ""
generate_target "nodejs" ""

# Create package info
cat > "../$DIST_DIR/README.md" << 'EOF'
# BS Calendar Core - WASM Bindings

This directory contains WASM bindings for the BS Calendar Core library.

## Targets

- **web/**: For direct browser usage with ES modules
- **bundler/**: For use with bundlers (webpack, rollup, vite, etc.)
- **nodejs/**: For Node.js environments

## Usage

### Web (ES Modules)
```javascript
import init, { BsCalendarCore } from './web/yorion_engine.js';

await init();
// Use the library
```

### Bundler (webpack, vite, etc.)
```javascript
import { BsCalendarCore } from './bundler/yorion_engine.js';
// Use the library
```

### Node.js
```javascript
const { BsCalendarCore } = require('./nodejs/yorion_engine.js');
// Use the library
```

## TypeScript

TypeScript definitions are included in each target directory.
EOF

echo -e "${GREEN}✅ WASM build complete!${NC}"
echo "Output: $DIST_DIR"
ls -lh "../$DIST_DIR"
