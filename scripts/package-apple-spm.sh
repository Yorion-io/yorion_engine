#!/usr/bin/env bash
set -euo pipefail

# Builds the XCFramework that the Swift package (root Package.swift) downloads
# as a binary target. Covers macOS (arm64 + x86_64), iOS devices (arm64) and the
# iOS simulator (arm64 + x86_64). macOS only: needs Xcode for lipo/xcodebuild.
#
#   ./scripts/generate-swift-bindings.sh   # writes dist/swift/
#   ./scripts/package-apple-spm.sh          # writes dist/spm/yorion_engineFFI.xcframework.zip
#
# The zip's SHA-256 is what Package.swift pins (scripts/update-swift-package.sh).

cd "$(dirname "$0")/.."

ENGINE_DIR="engine"
LIB="libyorion_engine.a"
OUT="dist/spm"
FRAMEWORK="yorion_engineFFI.xcframework"

export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-13.0}"

if [[ ! -f dist/swift/yorion_engineFFI.h || ! -f dist/swift/yorion_engineFFI.modulemap ]]; then
  echo "❌ dist/swift is missing the FFI header or modulemap. Run scripts/generate-swift-bindings.sh first." >&2
  exit 1
fi

TARGETS=(aarch64-apple-darwin x86_64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios)
rustup target add "${TARGETS[@]}"
for target in "${TARGETS[@]}"; do
  echo "🔨 $target"
  cargo build --release --manifest-path "$ENGINE_DIR/Cargo.toml" \
    --features uniffi-bindings --target "$target"
done

rm -rf "$OUT"
mkdir -p "$OUT/macos" "$OUT/ios-sim" "$OUT/headers"

# One fat library per platform: xcodebuild rejects two slices for the same platform.
lipo -create \
  "$ENGINE_DIR/target/aarch64-apple-darwin/release/$LIB" \
  "$ENGINE_DIR/target/x86_64-apple-darwin/release/$LIB" \
  -output "$OUT/macos/$LIB"
lipo -create \
  "$ENGINE_DIR/target/aarch64-apple-ios-sim/release/$LIB" \
  "$ENGINE_DIR/target/x86_64-apple-ios/release/$LIB" \
  -output "$OUT/ios-sim/$LIB"

# SwiftPM finds the C module through Headers/module.modulemap inside each slice.
cp dist/swift/yorion_engineFFI.h "$OUT/headers/"
cp dist/swift/yorion_engineFFI.modulemap "$OUT/headers/module.modulemap"

xcodebuild -create-xcframework \
  -library "$OUT/macos/$LIB" -headers "$OUT/headers" \
  -library "$ENGINE_DIR/target/aarch64-apple-ios/release/$LIB" -headers "$OUT/headers" \
  -library "$OUT/ios-sim/$LIB" -headers "$OUT/headers" \
  -output "$OUT/$FRAMEWORK"

( cd "$OUT" && ditto -c -k --sequesterRsrc --keepParent "$FRAMEWORK" "$FRAMEWORK.zip" )
rm -rf "$OUT/macos" "$OUT/ios-sim" "$OUT/headers"

echo "✅ $OUT/$FRAMEWORK.zip"
echo "   checksum: $(shasum -a 256 "$OUT/$FRAMEWORK.zip" | cut -d' ' -f1)"
