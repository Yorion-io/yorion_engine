#!/usr/bin/env bash
set -euo pipefail

# Build WASM binaries for web, bundler, and nodejs targets
# Outputs to dist/wasm/

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

# Install wasm-pack if not available
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    cargo install wasm-pack
fi

# Build for web target
echo -e "${BLUE}Building for web target...${NC}"
wasm-pack build \
    --target web \
    --out-dir "../$DIST_DIR/web" \
    --release \
    --features wasm \
    --no-default-features

# Build for bundler target (webpack, rollup, etc.)
echo -e "${BLUE}Building for bundler target...${NC}"
wasm-pack build \
    --target bundler \
    --out-dir "../$DIST_DIR/bundler" \
    --release \
    --features wasm \
    --no-default-features

# Build for nodejs target
echo -e "${BLUE}Building for nodejs target...${NC}"
wasm-pack build \
    --target nodejs \
    --out-dir "../$DIST_DIR/nodejs" \
    --release \
    --features wasm \
    --no-default-features

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
import init, { BsCalendarCore } from './web/bs_calendar_core.js';

await init();
// Use the library
```

### Bundler (webpack, vite, etc.)
```javascript
import { BsCalendarCore } from './bundler/bs_calendar_core.js';
// Use the library
```

### Node.js
```javascript
const { BsCalendarCore } = require('./nodejs/bs_calendar_core.js');
// Use the library
```

## TypeScript

TypeScript definitions are included in each target directory.
EOF

echo -e "${GREEN}✅ WASM build complete!${NC}"
echo "Output: $DIST_DIR"
ls -lh "../$DIST_DIR"
