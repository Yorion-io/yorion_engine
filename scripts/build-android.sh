#!/usr/bin/env bash
set -euo pipefail

# Builds the Android library (AAR) published to Maven Central as io.yorion:yorion-engine.
#
#   ./scripts/build-android.sh                 # build the AAR
#   ./scripts/build-android.sh publishToMavenLocal
#   ./scripts/build-android.sh publishToMavenCentral   # CI, with signing + Central Portal credentials
#
# Requires: Android SDK (ANDROID_HOME), the NDK (ANDROID_NDK_HOME, or installed
# under the SDK), JDK 17+, and `cargo install cargo-ndk`.

cd "$(dirname "$0")/.."

ENGINE_DIR="engine"
MODULE_DIR="android/yorion-engine"
GRADLE_TASK="${1:-assembleRelease}"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ENGINE_DIR/Cargo.toml" | head -1)"

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "❌ cargo-ndk not installed — 'cargo install cargo-ndk'" >&2
  exit 1
fi

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

JNI_OUT="$(pwd)/$MODULE_DIR/src/main/jniLibs"
rm -rf "$JNI_OUT"
# cargo-ndk only honours the crate in the current directory (see build-flutter.sh).
( cd "$ENGINE_DIR" && cargo ndk \
    -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 \
    --platform 21 \
    -o "$JNI_OUT" \
    build --release --features uniffi-bindings )

KOTLIN_OUT="$MODULE_DIR/src/main/kotlin"
rm -rf "$KOTLIN_OUT"
cargo run --manifest-path "$ENGINE_DIR/Cargo.toml" --features uniffi-cli \
  --bin uniffi-bindgen -- generate "$ENGINE_DIR/src/uniffi.udl" \
  --language kotlin --out-dir "$KOTLIN_OUT" --no-format \
  --config "$ENGINE_DIR/uniffi.toml"

( cd android && ./gradlew --no-daemon "$GRADLE_TASK" -PVERSION_NAME="$VERSION" )

echo "✅ Android $GRADLE_TASK done for io.yorion:yorion-engine:$VERSION"
ls -lh "$MODULE_DIR/build/outputs/aar/" 2>/dev/null || true
