#!/usr/bin/env bash
set -euo pipefail

# Builds the RELEASE engine for the Flutter plugin and vendors the artefacts
# into apps/mobile/native/yorion_engine_flutter.
#
# Why this exists: the plugin's binaries were previously built by hand, so they
# silently fell behind the engine source — the holiday table and weekend policy
# (engine commit e72fd08) were absent from the shipped .a/.so for weeks while
# the Rust source had them. Run this after any engine change that the app needs.
#
#   ./scripts/build-flutter.sh            # iOS + Android
#   ./scripts/build-flutter.sh ios        # one platform
#
# Requires: rustup targets for the platforms built, and the Android NDK
# (ANDROID_NDK_HOME) for the Android half.

cd "$(dirname "$0")/.."
ENGINE_DIR="engine"
PLUGIN_DIR="../apps/mobile/native/yorion_engine_flutter"
LIB_NAME="libyorion_engine.a"
SO_NAME="libyorion_engine.so"
WHAT="${1:-all}"

if [[ ! -d "$PLUGIN_DIR" ]]; then
  echo "❌ Flutter plugin not found at $PLUGIN_DIR" >&2
  exit 1
fi

build_ios() {
  echo "🔧 Building iOS (release)…"
  # Device, then both simulator arches — the simulator slice must be a fat
  # binary or Xcode rejects it on one of the two host architectures.
  for target in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios; do
    echo "   → $target"
    cargo build --release --manifest-path "$ENGINE_DIR/Cargo.toml" \
      --features uniffi-bindings --target "$target"
  done

  local out="dist/BsCalendarCoreIOS.xcframework"
  rm -rf "$out" dist/_sim
  mkdir -p dist/_sim

  lipo -create \
    "$ENGINE_DIR/target/aarch64-apple-ios-sim/release/$LIB_NAME" \
    "$ENGINE_DIR/target/x86_64-apple-ios/release/$LIB_NAME" \
    -output "dist/_sim/$LIB_NAME"

  # UniFFI emits the C header + modulemap alongside the Swift bindings; both
  # slices need their own copy inside the xcframework.
  local hdr="dist/_headers"
  rm -rf "$hdr"; mkdir -p "$hdr"
  cp dist/swift/yorion_engineFFI.h "$hdr/" 2>/dev/null || true
  cat > "$hdr/module.modulemap" <<'MODULEMAP'
module yorion_engineFFI {
  header "yorion_engineFFI.h"
  export *
}
MODULEMAP

  xcodebuild -create-xcframework \
    -library "$ENGINE_DIR/target/aarch64-apple-ios/release/$LIB_NAME" -headers "$hdr" \
    -library "dist/_sim/$LIB_NAME" -headers "$hdr" \
    -output "$out"

  rm -rf "$PLUGIN_DIR/ios/Frameworks/BsCalendarCoreIOS.xcframework"
  cp -R "$out" "$PLUGIN_DIR/ios/Frameworks/"
  cp dist/swift/yorion_engine.swift "$PLUGIN_DIR/ios/Classes/yorion_engine.swift"
  echo "✅ iOS vendored into $PLUGIN_DIR/ios"
}

build_android() {
  echo "🔧 Building Android (release)…"
  if ! command -v cargo-ndk >/dev/null 2>&1; then
    echo "❌ cargo-ndk not installed — 'cargo install cargo-ndk'" >&2
    return 1
  fi
  cargo ndk --manifest-path "$ENGINE_DIR/Cargo.toml" \
    -t arm64-v8a -t x86_64 \
    -o "$PLUGIN_DIR/android/src/main/jniLibs" \
    build --release --features uniffi-bindings

  cargo run --manifest-path "$ENGINE_DIR/Cargo.toml" --features uniffi-cli \
    --bin uniffi-bindgen -- generate "$ENGINE_DIR/src/uniffi.udl" \
    --language kotlin --out-dir dist/kotlin --no-format
  mkdir -p "$PLUGIN_DIR/android/src/main/kotlin/uniffi/yorion_engine"
  cp dist/kotlin/uniffi/yorion_engine/yorion_engine.kt \
    "$PLUGIN_DIR/android/src/main/kotlin/uniffi/yorion_engine/"
  echo "✅ Android vendored into $PLUGIN_DIR/android"
}

# Swift/Kotlin bindings must be regenerated from the same source as the binary,
# or the Dart bridge calls methods the .a does not export.
./scripts/generate-swift-bindings.sh

case "$WHAT" in
  ios) build_ios ;;
  android) build_android ;;
  all) build_ios; build_android || echo "⚠️  Android skipped (see above)" ;;
  *) echo "usage: $0 [ios|android|all]" >&2; exit 1 ;;
esac

echo
echo "Engine version: $(grep -m1 '^version' "$ENGINE_DIR/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
echo "Rebuild the app so the new binary is linked (flutter clean if it misbehaves)."
