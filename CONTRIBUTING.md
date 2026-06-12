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

`engine/data/bs_calendar_data.json` — official Vikram Sambat month-day table for BS 2000–2090. Do not edit manually; data comes from the Government of Nepal's official calendar publication.

`engine/data/tithi_exceptions.csv` — almanac-derived tithi overrides for BS 2079–2082 (AD 2022–2026). Regenerate with:

```bash
cargo run --example gen_tithi_exceptions
```

`engine/tests/data/calendar/calendar_20{67..83}.csv` — reference almanac CSV files for ground-truth validation. Almanac tithi data is not available beyond BS 2083; tests for BS 2084+ mark astronomical calculations as unverified estimates.

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
