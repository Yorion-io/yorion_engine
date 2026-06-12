# Changelog

All notable changes to `yorion_engine` are documented here.

## [0.1.3] — 2024

### Fixed
- Tithi calculation now uses sunrise time rather than noon UTC, matching Nepal's astronomical standard.
- CI: pin `wasm-bindgen` CLI to `0.2.122` to match the Cargo dependency version.

## [0.1.2] — 2024

### Fixed
- WASM build time regression.
- Swift binding generation.
- Reduced Apple archive file size.
- Release packaging.

## [0.1.1] — 2024

### Added
- UniFFI bindings for native Swift/Kotlin integration.
- Parallel macOS builds in CI.
- Local workflow testing scripts.

### Fixed
- WASM opt validation errors.
- Universal macOS binary packaging.
- Changelog generation and version bump tooling.

## [0.1.0] — 2024

### Added
- Initial release.
- BS ↔ AD date conversion for years 2000–2090.
- Panchanga calculations: tithi, nakshatra, zodiac, sunrise/sunset.
- BS-RRULE v2.0 parser and generator for solar and lunar recurrence rules.
- WASM bindings (bundler, Node.js, web targets).
- Tithi override table compiled from official Nepali Panchanga almanac data (BS 2079–2083).
