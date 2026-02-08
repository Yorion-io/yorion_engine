#!/usr/bin/env bash
set -euo pipefail

# Unified release script
# Builds all targets and creates release archives

echo "🚀 Building release artifacts..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get version from Cargo.toml or git tag
VERSION=${1:-$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")}
VERSION=${VERSION#v} # Remove 'v' prefix

echo -e "${BLUE}Building version: ${VERSION}${NC}"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist
mkdir -p dist/archives

# Run all build scripts
echo -e "${BLUE}Running build scripts...${NC}"

# WASM
if [ -f "scripts/build-wasm.sh" ]; then
    echo -e "${BLUE}Building WASM...${NC}"
    bash scripts/build-wasm.sh || echo -e "${RED}❌ WASM build failed${NC}"
else
    echo -e "${YELLOW}⚠️  WASM build script not found${NC}"
fi

# Flutter
if [ -f "scripts/build-flutter.sh" ]; then
    echo -e "${BLUE}Building Flutter...${NC}"
    bash scripts/build-flutter.sh || echo -e "${YELLOW}⚠️  Flutter build skipped${NC}"
else
    echo -e "${YELLOW}⚠️  Flutter build script not found${NC}"
fi

# Native
if [ -f "scripts/build-native.sh" ]; then
    echo -e "${BLUE}Building Native...${NC}"
    bash scripts/build-native.sh || echo -e "${RED}❌ Native build failed${NC}"
else
    echo -e "${YELLOW}⚠️  Native build script not found${NC}"
fi

# Create archives
echo -e "${BLUE}Creating release archives...${NC}"

cd dist

# WASM archives
if [ -d "wasm" ]; then
    tar -czf "archives/bs_calendar_core-wasm-${VERSION}.tar.gz" wasm/
    echo -e "${GREEN}✅ Created WASM archive${NC}"
fi

# Flutter archives
if [ -d "flutter" ]; then
    tar -czf "archives/bs_calendar_core-flutter-${VERSION}.tar.gz" flutter/
    echo -e "${GREEN}✅ Created Flutter archive${NC}"
fi

# Native archives (per platform)
if [ -d "native" ]; then
    for platform_dir in native/*/; do
        platform=$(basename "$platform_dir")
        tar -czf "archives/bs_calendar_core-native-${platform}-${VERSION}.tar.gz" -C native "$platform"
        echo -e "${GREEN}✅ Created Native archive: ${platform}${NC}"
    done
fi

# Generate checksums
echo -e "${BLUE}Generating checksums...${NC}"
cd archives
shasum -a 256 *.tar.gz > SHA256SUMS
cd ../..

# Create release notes template
cat > dist/RELEASE_NOTES.md << EOF
# Release ${VERSION}

## Downloads

### WASM
- \`bs_calendar_core-wasm-${VERSION}.tar.gz\` - WASM bindings for web, bundler, and Node.js

### Flutter
- \`bs_calendar_core-flutter-${VERSION}.tar.gz\` - Flutter/Dart bindings with native libraries

### Native (C FFI)
- \`bs_calendar_core-native-macos-universal-${VERSION}.tar.gz\` - macOS universal binary
- \`bs_calendar_core-native-linux-x86_64-${VERSION}.tar.gz\` - Linux x86_64
- \`bs_calendar_core-native-linux-aarch64-${VERSION}.tar.gz\` - Linux ARM64
- \`bs_calendar_core-native-windows-x86_64-${VERSION}.tar.gz\` - Windows x86_64

## Checksums

See \`SHA256SUMS\` for file checksums.

## Changes

<!-- Changelog will be inserted here by CI -->
EOF

echo -e "${GREEN}✅ Release build complete!${NC}"
echo ""
echo "📦 Release artifacts:"
ls -lh dist/archives/
echo ""
echo "📝 Release notes: dist/RELEASE_NOTES.md"
echo "🔐 Checksums: dist/archives/SHA256SUMS"
