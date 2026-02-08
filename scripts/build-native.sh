#!/usr/bin/env bash
set -euo pipefail

# Build native binaries for macOS, Linux, and Windows
# Outputs to dist/native/

echo "🦀 Building native binaries..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create output directory
DIST_DIR="dist/native"
mkdir -p "$DIST_DIR"

# Change to engine directory
cd engine

# Install cross if not available (for Linux/Windows cross-compilation)
if ! command -v cross &> /dev/null; then
    echo "📦 Installing cross..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Install cbindgen if not available (for C header generation)
if ! command -v cbindgen &> /dev/null; then
    echo "📦 Installing cbindgen..."
    cargo install cbindgen
fi

# Generate C header file
echo -e "${BLUE}Generating C header...${NC}"
cbindgen --config cbindgen.toml --crate bs_calendar_core --output "../$DIST_DIR/bs_calendar_core.h" || \
cbindgen --crate bs_calendar_core --output "../$DIST_DIR/bs_calendar_core.h"

# Build for macOS (native)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}Building for macOS (x86_64)...${NC}"
    cargo build --release --target x86_64-apple-darwin
    mkdir -p "../$DIST_DIR/macos-x86_64"
    cp target/x86_64-apple-darwin/release/libbs_calendar_core.dylib "../$DIST_DIR/macos-x86_64/"
    cp target/x86_64-apple-darwin/release/libbs_calendar_core.a "../$DIST_DIR/macos-x86_64/"
    
    echo -e "${BLUE}Building for macOS (aarch64)...${NC}"
    cargo build --release --target aarch64-apple-darwin
    mkdir -p "../$DIST_DIR/macos-aarch64"
    cp target/aarch64-apple-darwin/release/libbs_calendar_core.dylib "../$DIST_DIR/macos-aarch64/"
    cp target/aarch64-apple-darwin/release/libbs_calendar_core.a "../$DIST_DIR/macos-aarch64/"
    
    # Create universal binary
    echo -e "${BLUE}Creating universal macOS binary...${NC}"
    mkdir -p "../$DIST_DIR/macos-universal"
    lipo -create \
        target/x86_64-apple-darwin/release/libbs_calendar_core.dylib \
        target/aarch64-apple-darwin/release/libbs_calendar_core.dylib \
        -output "../$DIST_DIR/macos-universal/libbs_calendar_core.dylib"
    lipo -create \
        target/x86_64-apple-darwin/release/libbs_calendar_core.a \
        target/aarch64-apple-darwin/release/libbs_calendar_core.a \
        -output "../$DIST_DIR/macos-universal/libbs_calendar_core.a"
fi

# Build for Linux (using cross)
echo -e "${BLUE}Building for Linux (x86_64)...${NC}"
cross build --release --target x86_64-unknown-linux-gnu
mkdir -p "../$DIST_DIR/linux-x86_64"
cp target/x86_64-unknown-linux-gnu/release/libbs_calendar_core.so "../$DIST_DIR/linux-x86_64/"
cp target/x86_64-unknown-linux-gnu/release/libbs_calendar_core.a "../$DIST_DIR/linux-x86_64/"

echo -e "${BLUE}Building for Linux (aarch64)...${NC}"
cross build --release --target aarch64-unknown-linux-gnu
mkdir -p "../$DIST_DIR/linux-aarch64"
cp target/aarch64-unknown-linux-gnu/release/libbs_calendar_core.so "../$DIST_DIR/linux-aarch64/"
cp target/aarch64-unknown-linux-gnu/release/libbs_calendar_core.a "../$DIST_DIR/linux-aarch64/"

# Build for Windows (using cross)
echo -e "${BLUE}Building for Windows (x86_64)...${NC}"
cross build --release --target x86_64-pc-windows-gnu
mkdir -p "../$DIST_DIR/windows-x86_64"
cp target/x86_64-pc-windows-gnu/release/bs_calendar_core.dll "../$DIST_DIR/windows-x86_64/" || \
cp target/x86_64-pc-windows-gnu/release/libbs_calendar_core.dll "../$DIST_DIR/windows-x86_64/" || \
echo -e "${YELLOW}⚠️  Windows DLL not found${NC}"
cp target/x86_64-pc-windows-gnu/release/libbs_calendar_core.a "../$DIST_DIR/windows-x86_64/" || \
cp target/x86_64-pc-windows-gnu/release/bs_calendar_core.lib "../$DIST_DIR/windows-x86_64/" || \
echo -e "${YELLOW}⚠️  Windows static lib not found${NC}"

# Copy header to each platform directory
for dir in "$DIST_DIR"/*/; do
    cp "../$DIST_DIR/bs_calendar_core.h" "$dir"
done

# Create README
cat > "../$DIST_DIR/README.md" << 'EOF'
# BS Calendar Core - Native Bindings

This directory contains native C-compatible bindings for the BS Calendar Core library.

## Platforms

- **macos-universal/**: Universal binary for macOS (x86_64 + aarch64)
- **macos-x86_64/**: macOS Intel binary
- **macos-aarch64/**: macOS Apple Silicon binary
- **linux-x86_64/**: Linux x86_64 binary
- **linux-aarch64/**: Linux ARM64 binary
- **windows-x86_64/**: Windows x86_64 binary

## Files

Each platform directory contains:
- `bs_calendar_core.h` - C header file with function declarations
- Dynamic library (`.dylib`, `.so`, or `.dll`)
- Static library (`.a` or `.lib`)

## Usage

### C/C++

```c
#include "bs_calendar_core.h"

// Link against the library
// gcc main.c -L./macos-universal -lbs_calendar_core
```

### Python (ctypes)

```python
import ctypes

lib = ctypes.CDLL('./macos-universal/libbs_calendar_core.dylib')
# Use the library
```

### Other Languages

Any language with FFI support can use these bindings (Go, Ruby, Node.js native modules, etc.)
EOF

echo -e "${GREEN}✅ Native build complete!${NC}"
echo "Output: $DIST_DIR"
find "../$DIST_DIR" -type f -name "*.dylib" -o -name "*.so" -o -name "*.dll" -o -name "*.a" | xargs ls -lh
