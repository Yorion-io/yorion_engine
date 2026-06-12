# macOS Swift Integration - Quick Reference

## Build Commands

```bash
# Build universal binary (Intel + Apple Silicon)
./scripts/build-macos.sh

# Generate Swift bindings and XCFramework
./scripts/generate-swift-bindings.sh

# Build WASM (existing workflow)
./scripts/build-wasm.sh
```

## Output Locations

- **Universal Binary**: `target/universal-apple-darwin/release/libyorion_engine.a`
- **Swift Bindings**: `dist/swift/yorion_engine.swift`
- **XCFramework**: `dist/YorionCore.xcframework/`
- **WASM**: `dist/wasm/`

## Swift Integration

### Add to Xcode Project

1. **Via Swift Package Manager**:
   - File → Add Package Dependencies
   - Enter repository URL
   - Select version

2. **Via Local XCFramework**:
   - Drag `dist/YorionCore.xcframework` into project
   - Add to "Frameworks, Libraries, and Embedded Content"

### Basic Usage

```swift
import YorionCore

let engine = createEngine()

// Convert dates
let gregorian = GregorianDate(year: 2024, month: 2, day: 8)
let bs = try? engine.gregorianToBs(date: gregorian)

// Get astronomical info
let location = Location(latitude: 27.7172, longitude: 85.3240)
let tithi = try? engine.getTithi(date: gregorian)
let astroInfo = try? engine.getDailyAstroInfo(date: gregorian, location: location)
```

## Architecture

```
┌─────────────────────────────────────┐
│   macOS Swift/SwiftUI App           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Swift Bindings (Auto-generated)   │
│   yorion_engine.swift             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   UniFFI C FFI Layer                │
│   (Thin, Auto-generated)             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Rust Core Library                 │
│   yorion_engine                   │
│   (Universal Binary: x86_64 + arm64) │
└─────────────────────────────────────┘
```

## Features

- ✅ **Automatic Binding Generation**: UniFFI generates Swift code from UDL
- ✅ **Universal Binary**: Single binary for Intel + Apple Silicon
- ✅ **Type Safety**: Full type conversion between Rust and Swift
- ✅ **Zero Runtime Overhead**: Static library linking
- ✅ **App Store Ready**: Standard XCFramework format
- ✅ **Future-Proof**: Easy to add new functions

## CI/CD Integration

Add to `.github/workflows/release.yml`:

```yaml
build-macos:
  runs-on: macos-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: x86_64-apple-darwin,aarch64-apple-darwin
    - run: cargo install uniffi-bindgen
    - run: ./scripts/build-macos.sh
    - run: ./scripts/generate-swift-bindings.sh
    - run: |
        cd dist
        zip -r YorionCore.xcframework.zip YorionCore.xcframework
    - uses: softprops/action-gh-release@v1
      with:
        files: dist/YorionCore.xcframework.zip
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `uniffi-bindgen not found` | Run `cargo install uniffi-bindgen` |
| `Module not found` in Xcode | Add XCFramework to target's frameworks |
| Build fails on feature flags | Use `--features uniffi-bindings --no-default-features` |
| Linker errors | Ensure building for correct architecture (x86_64/arm64) |

## Documentation

- **Full Guide**: [Swift Integration Guide](./swift-integration.md)
- **WASM Setup**: [Setup Guide](./setup.md)
