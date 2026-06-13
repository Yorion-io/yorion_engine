# Contributing

## Getting started

```bash
git clone https://github.com/Yorion-io/yorion_engine
cd yorion_engine/engine
cargo build
cargo test
```

Requirements: Rust 1.82+ (MSRV), `wasm-pack` (for WASM builds only).

## Running tests

```bash
# All tests
cargo test

# Festival ground truth (208 tests × almanac data)
cargo test --test festival_ground_truth

# BS-RRULE spec conformance
cargo test --test conformance_vectors

# Property-based roundtrip invariants
cargo test --test property_invariants

# Benchmarks (requires release build)
cargo bench
```

## Data files

`engine/data/bs_calendar_data.json` — Bikram Sambat anchor + month-length table for **BS 1975–2100** (126 years). Generated from the [`opensource-nepal/nepali_datetime`](https://github.com/opensource-nepal/nepali_datetime) reference table; do not edit by hand. Each year's 12 month lengths must sum to the gap between its 1-Baisakh anchor and the next year's anchor — `build.rs` warns and `month_lengths_match_anchor_gaps` (in `tests/property_invariants.rs`) fails if they don't.

`engine/data/tithi_exceptions.csv` — almanac-derived tithi overrides (currently 176 rows, AD 2022-04 → 2026-03, ≈ BS 2079–2082). Each row records a date where the engine's astronomical tithi disagrees with the published almanac, forcing the almanac value. Regenerate with:

```bash
cargo run --example gen_tithi_exceptions
```

`engine/tests/data/calendar/calendar_20{67..83}.csv` — reference almanac CSV files for ground-truth validation. **Tithi accuracy is gated by these files: the last one is `calendar_2083.csv`, so tithi output is verified only through BS 2083** (`core_api::TITHI_VERIFIED_THROUGH_BS`). For BS 2084+, tithi values are raw astronomical output with no almanac correction; tests for those years mark results as unverified estimates.

### Yearly maintenance: refreshing tithi data when a new almanac is published

Tithi/panchanga accuracy is **not** self-extending — it is pinned to the reference almanac files above, which currently stop at BS 2083. **Each year, when the official Nepali patro for a new BS year is published, do the following** so that year's tithis become verified rather than unverified astronomy:

1. **Add the reference almanac.** Drop the new `engine/tests/data/calendar/calendar_20XX.csv` (same column format as the existing files), transcribed from the published almanac.
2. **Regenerate the overrides.** Run `cargo run --example gen_tithi_exceptions` — it re-diffs the engine's astronomical tithi against every reference CSV and rewrites `tithi_exceptions.csv` with the new corrections.
3. **Bump the verified-through year.** Update `TITHI_VERIFIED_THROUGH_BS` in `engine/src/core_api.rs` to the new last-covered BS year.
4. **Update the docs that quote the limit:** the README *Coverage* table, `docs/assumptions.md` §6.3, and `docs/festivals.md` (which lists verified festival dates per year).
5. **Re-run `cargo test`.** The festival ground-truth and override tests now cover the new year; all must pass.

This is the calendar's only recurring data-maintenance obligation. The BS↔AD conversion table (`bs_calendar_data.json`) already covers through BS 2100 and does **not** need yearly updates.

## Building

```bash
# WASM (all three targets: web, bundler, nodejs)
./scripts/build-wasm.sh

# macOS XCFramework + Swift bindings
./scripts/build-macos-arm64.sh
./scripts/build-macos-x86.sh
./scripts/generate-swift-bindings.sh
./scripts/package-macos.sh
```

Additional tools needed for releases: `cargo install --locked cocogitto`, `cargo install git-cliff`.

## Release workflow

Push to `dev` → pre-release with `-dev` tag (WASM artifacts uploaded).  
Merge to `main` → semantic version bump, changelog, full release.

Version is auto-determined from conventional commits:
- `feat:` → minor (0.1.0 → 0.2.0)
- `fix:` → patch (0.1.0 → 0.1.1)
- `feat!:` / `BREAKING CHANGE:` → major (0.1.0 → 1.0.0)

## Commit style

This project uses [Conventional Commits](https://www.conventionalcommits.org/). Format:

```
feat: add lunar month filter for tithi rules
fix: correct sunrise calculation for DST boundaries
docs: update BS-RRULE spec with X-TAKE semantics
```

No scope tags (`feat(scope):` style is not used here).

## Pull requests

1. Fork and create a branch off `main`.
2. Add or update tests for any behavior change.
3. Run `cargo test` and `cargo clippy` — both must pass.
4. Open a PR with a description of what changed and why.

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache 2.0, matching the project license.
