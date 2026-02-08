#!/usr/bin/env bash
set -euo pipefail

# Build Flutter bindings
# Outputs to dist/flutter/

echo "🦀 Building Flutter bindings..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create output directory
DIST_DIR="dist/flutter"
mkdir -p "$DIST_DIR"

# Change to engine directory
cd engine

# Check if flutter_rust_bridge_codegen is installed
if ! command -v flutter_rust_bridge_codegen &> /dev/null; then
    echo -e "${YELLOW}⚠️  flutter_rust_bridge_codegen not found${NC}"
    echo "Install with: cargo install flutter_rust_bridge_codegen"
    echo "Skipping Flutter build..."
    exit 0
fi

# Generate Dart bindings
echo -e "${BLUE}Generating Dart bindings...${NC}"
flutter_rust_bridge_codegen \
    --rust-input src/lib.rs \
    --dart-output "../$DIST_DIR/lib/bridge_generated.dart" \
    --dart-decl-output "../$DIST_DIR/lib/bridge_definitions.dart"

# Build for different platforms
echo -e "${BLUE}Building native libraries...${NC}"

# macOS (both architectures)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Building for macOS (x86_64)..."
    cargo build --release --target x86_64-apple-darwin --features flutter
    
    echo "Building for macOS (aarch64)..."
    cargo build --release --target aarch64-apple-darwin --features flutter
    
    # Create universal binary
    mkdir -p "../$DIST_DIR/macos"
    lipo -create \
        target/x86_64-apple-darwin/release/libbs_calendar_core.dylib \
        target/aarch64-apple-darwin/release/libbs_calendar_core.dylib \
        -output "../$DIST_DIR/macos/libbs_calendar_core.dylib"
    
    echo "Building for iOS simulator..."
    cargo build --release --target aarch64-apple-ios-sim --features flutter || echo "iOS sim target not available"
    
    echo "Building for iOS device..."
    cargo build --release --target aarch64-apple-ios --features flutter || echo "iOS target not available"
fi

# Android (requires NDK)
if command -v cargo-ndk &> /dev/null; then
    echo "Building for Android..."
    mkdir -p "../$DIST_DIR/android/jniLibs"
    
    cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 \
        -o "../$DIST_DIR/android/jniLibs" \
        build --release --features flutter
else
    echo -e "${YELLOW}⚠️  cargo-ndk not found, skipping Android build${NC}"
    echo "Install with: cargo install cargo-ndk"
fi

# Create pubspec.yaml template
cat > "../$DIST_DIR/pubspec.yaml" << 'EOF'
name: bs_calendar_core
description: Bikram Sambat calendar engine with tithi support
version: 0.1.0

environment:
  sdk: '>=3.0.0 <4.0.0'

dependencies:
  flutter:
    sdk: flutter
  ffi: ^2.0.0
  flutter_rust_bridge: ^2.0.0

# To use this package, download the release artifacts and place them in your project
# Then add this as a path dependency in your pubspec.yaml:
# dependencies:
#   bs_calendar_core:
#     path: ./path/to/this/directory
EOF

# Create README
cat > "../$DIST_DIR/README.md" << 'EOF'
# BS Calendar Core - Flutter Bindings

This directory contains Flutter/Dart bindings for the BS Calendar Core library.

## Installation

1. Download the release artifacts for your target platform
2. Extract to your Flutter project (e.g., `packages/bs_calendar_core/`)
3. Add to your `pubspec.yaml`:

```yaml
dependencies:
  bs_calendar_core:
    path: ./packages/bs_calendar_core
```

## Platform Support

- ✅ macOS (Universal binary: x86_64 + aarch64)
- ✅ iOS (Device + Simulator)
- ✅ Android (arm64-v8a, armeabi-v7a, x86_64, x86)
- ⚠️  Linux (build on Linux host)
- ⚠️  Windows (build on Windows host)

## Usage

```dart
import 'package:bs_calendar_core/bridge_generated.dart';

// Use the library
```
EOF

echo -e "${GREEN}✅ Flutter build complete!${NC}"
echo "Output: $DIST_DIR"
ls -lh "../$DIST_DIR"
