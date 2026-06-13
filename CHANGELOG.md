# Changelog

All notable changes to `yorion_engine` are documented here.

## [Unreleased]

### Added
- Extended BS ↔ AD conversion data to **BS 1975–2100** (was 2000–2090), 126 years, sourced from the `opensource-nepal/nepali_datetime` reference table.
- **Yoga** and **Karana** — the engine now computes all five panchanga angas (tithi, vara, nakshatra, yoga, karana). New `Yoga`/`Karana` enums, getters, and name helpers across the Rust, WASM, and UniFFI surfaces.
- `CalendarEngine::checked_bs_date` — validates a BS date against the actual month-length table (rejects e.g. day 32 in a 30-day month), unlike the structural-only `BsDate::new`.
- `CalendarEngine::get_tithi_at_location` — location-aware tithi (the existing `get_tithi` remains a Kathmandu shorthand).
- `CalendarEngine::get_tithi_end` — the UTC instant at which the sunrise tithi ends.
- `CalendarEngine::get_daily_panchanga` — daily astro info plus sunrise/sunset in a single pass.
- `TITHI_VERIFIED_THROUGH_BS` constant documenting that tithi accuracy is almanac-verified only through BS 2083.
- Build-time and test-time invariants that each year's month lengths sum to its anchor gap.

### Changed
- Ayanamsa is now the **computed Lahiri (Chitrapaksha)** value (linear drift from the J2000.0 epoch) instead of a fixed 24.0°, fixing zodiac/nakshatra misassignment near boundaries.
- BS year lookup is now O(1) (index into a contiguous table) instead of a linear scan.
- Unbounded recurrence expansion now returns `BsCalendarError::InstanceLimitExceeded` at the 10,000-occurrence safety cap instead of silently truncating.

### Fixed
- Corrected BS month lengths for years 2000, 2062, 2085, 2086, 2087, 2089, 2090 against the authoritative reference (some broke date round-trips at year boundaries).
- `BsDate` now derives `Hash`.

### Removed
- Legacy `InstanceGenerator::generate_tithi_instances` (a duplicate path that ignored `skip_adhik`); use `CalendarEngine::generate_tithi_instances`.

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
