# YorionEngine

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Build](https://img.shields.io/github/actions/workflow/status/Yorion-io/yorion_engine/ci.yml)](https://github.com/Yorion-io/yorion_engine/actions)

A platform-agnostic Rust library for the **Bikram Sambat (BS)** calendar — the official calendar of Nepal. It provides accurate date conversion, astronomical (panchanga) calculations, and a rich recurrence-rule system for scheduling events in both solar and lunar terms.

---

## What is it?

The Gregorian calendar is not the primary calendar for roughly 30 million Nepali speakers. Their civil, religious, and agricultural life runs on **Bikram Sambat** — a solar calendar roughly 56–57 years ahead of the Gregorian year, with variable month lengths and tight coupling to Hindu astronomical cycles (tithi, nakshatra, zodiac).

No open, embeddable library existed that handled all three concerns together:

1. **Accurate BS ↔ AD conversion** — month lengths vary year-by-year and are not derivable from a formula; they require a pre-computed data table.
2. **Panchanga (Hindu almanac) data** — tithi (lunar day), paksha (fortnight), nakshatra (lunar mansion), sun/moon zodiac signs, and sunrise/sunset per location.
3. **Lunar and solar recurrence rules** — "every Ekadashi", "every Baisakh 1", "every second Friday in Shrawan" cannot be expressed in standard RFC 5545 RRULE without calendar-system extensions.

YorionEngine solves all three in one library that compiles to native code, WASM (browser + Node.js), and mobile bindings (Swift/Kotlin via UniFFI).

---

## Why a library instead of an API?

- **Offline-capable** — embedded apps (iOS, Android, desktop) need no network for calendar math.
- **Deterministic** — same input always produces the same output; no server round-trips to track.
- **Zero runtime cost** — calendar data is baked in at compile time; lookups are O(1) array accesses.
- **Cross-platform** — one Rust codebase, three output targets (native, WASM, mobile).

---

## Coverage

| Capability | Range / Detail |
|---|---|
| BS ↔ AD conversion | BS 2000–2090 (AD 1943–2033) |
| Tithi accuracy | Validated against official Nepali Panchanga almanac through BS 2083 |
| Tithi exception overrides | CSV-driven corrections baked in at build time |
| Astronomical | Sunrise/sunset + zodiac + nakshatra for any lat/lon |
| Recurrence | BS-solar, AD-solar, panchanga-lunar |

---

## Architecture

The library follows a **Ports & Adapters** (Hexagonal) layout:

```
src/
├── domain/          # Pure value types: BsDate, Tithi, BsMonth, ZodiacSign, …
│   └── recurrence/  # BsRecurrenceRule, TithiRecurrenceRule, AdRecurrenceRule
├── ports/           # Traits: CalendarProvider, TimeProvider, TithiOverrideProvider
├── adapters/        # Concrete impls of those traits (static data, system clock)
├── services/        # Business logic: conversion, astronomical, instance generation
├── core_api.rs      # Stable public façade: CalendarEngine
├── wasm.rs          # wasm-bindgen exported functions
└── uniffi_bindings.rs  # UniFFI scaffolding for Swift/Kotlin
```

`CalendarEngine` is the single entry point you depend on. Everything beneath it is an implementation detail.

---

## Getting started — Rust

Add to `Cargo.toml`:

```toml
[dependencies]
yorion_engine = { git = "https://github.com/Yorion-io/yorion_engine", tag = "v0.1.3" }
```

### Date conversion

```rust
use yorion_engine::prelude::*;
use chrono::NaiveDate;

let engine = CalendarEngine::new();

// Gregorian → BS
let ad = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
let bs = engine.gregorian_to_bs(ad)?;
println!("{}-{}-{}", bs.year, bs.month_u8(), bs.day); // 2081-1-1

// BS → Gregorian
let bs = BsDate::new(2081, 1, 1)?;
let ad = engine.bs_to_gregorian(bs)?;
println!("{}", ad); // 2024-04-13
```

### Panchanga (daily almanac)

```rust
use yorion_engine::prelude::*;
use chrono::NaiveDate;

let engine = CalendarEngine::new();
let location = Location::kathmandu();
let date = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
let info = engine.get_daily_astro_info(date, location)?;

println!("Tithi:     {}", info.tithi);
println!("Nakshatra: {}", info.nakshatra);
```

### Recurring events

```rust
use yorion_engine::prelude::*;

let engine = CalendarEngine::new();

// Every Baisakh 1 — annual solar festival
let anchor = BsDate::new(2081, 1, 1)?;
let start  = BsDate::new(2081, 1, 1)?;
let end    = BsDate::new(2086, 1, 1)?;
let rule = BsRecurrenceRule::new(BsFrequency::Yearly, anchor)
    .with_by_month(vec![BsMonth::Baisakh])
    .with_count(5);
let instances = engine.generate_bs_instances(&rule, start, end)?;

// Every Ekadashi (both pakshas)
let tithi_rule = TithiRecurrenceRule::ekadashi(BsDate::new(2081, 1, 1)?)
    .with_count(10);
let instances = engine.generate_tithi_instances(
    "my-event", "Ekadashi",
    &tithi_rule, start, end,
    CalendarVersion::official("v1".to_string()),
    Location::kathmandu(),
)?;
```

---

## Getting started — WASM (browser / Node.js)

Pre-built bundles live in `dist/wasm/` with three sub-targets.

### Bundler (Vite, Webpack, Rollup)

```javascript
import init, {
  gregorian_to_bs,
  get_tithi,
  get_month_calendar_with_location,
} from './dist/wasm/bundler/yorion_engine.js';

await init();

const bs    = gregorian_to_bs(2024, 4, 13);          // { year: 2081, month: 1, day: 1 }
const tithi = get_tithi(2024, 4, 13);                // AD (Gregorian) date → tithi
const loc   = new Location(27.7, 85.3, 'Kathmandu', 345);
const cal   = get_month_calendar_with_location(2081, 1, loc);
```

### Browser (ES Module, no bundler)

```html
<script type="module">
  import init, * as bs from './dist/wasm/web/yorion_engine.js';
  await init();
  console.log(bs.gregorian_to_bs(2024, 4, 13));
</script>
```

### Node.js

```javascript
const bs = require('./dist/wasm/nodejs/yorion_engine.js');
console.log(bs.gregorian_to_bs(2024, 4, 13));
```

Full TypeScript definitions (`.d.ts`) are included in every target directory.

---

## Native bindings — Swift (iOS / macOS)

The library exposes a UniFFI interface that generates native Swift bindings.

```bash
# Build the macOS dylib + generate Swift sources
./scripts/build-macos-arm64.sh
./scripts/generate-swift-bindings.sh
```

Then add the generated `YorionEngine.swift` and the `.dylib` / `.xcframework` to your Xcode project. See [Swift Integration Guide](./docs/swift-integration.md) for a step-by-step guide.

### Kotlin / Android

UniFFI also targets Kotlin. The `.udl` interface file at `engine/src/uniffi.udl` is the source of truth; run `uniffi-bindgen generate` with `--language kotlin` to produce the Kotlin bindings. Android packaging instructions are in progress.

---

## Building from source

### Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust | ≥ 1.70 | Core compiler |
| `wasm-pack` | latest | WASM compilation |
| `wasm-opt` | latest (via `binaryen`) | Optional WASM size reduction |
| `uniffi-bindgen` | 0.28 | Native binding generation |

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Build WASM (all three targets)
./scripts/build-wasm.sh

# Build and test native
cd engine
cargo test --all-features
```

---

## WASM binary sizes explained

Current distribution sizes (post `wasm-opt`):

| Target | `.wasm` | JS glue |
|---|---|---|
| web | ~3.3 MB | ~50 KB |
| bundler | ~3.3 MB | ~46 KB |
| nodejs | ~3.3 MB | ~48 KB |

**Why ~3.3 MB?**

The size comes primarily from three sources:

| Source | Approximate contribution |
|---|---|
| 91 years of calendar data (baked in at compile time) | ~800 KB |
| Astronomical computation code (`suncalc`, `astro` crates) | ~1.2 MB |
| Recurrence engine (`rrule` crate + BS extensions) | ~600 KB |
| Rust standard library (panic, fmt, allocator) | ~700 KB |

The binary already uses every standard Rust size-reduction flag:

```toml
[profile.release]
opt-level = "z"       # Size over speed
lto = true            # Link-time dead-code elimination
codegen-units = 1     # Single unit maximises LTO
panic = "abort"       # No unwinding tables
strip = true          # Strip debug symbols
```

**How to get a smaller build:**

1. **`wasm-opt -Oz`** — The build script calls this automatically if `binaryen` is installed. Typical saving: 10–15% on top of Rust's own stripping.

2. **Feature flags** — If you do not need astronomical calculations, you can disable the `suncalc`/`astro` dependency by gating it behind a feature flag (not yet implemented; PRs welcome). Estimated saving: ~800 KB.

3. **Strip calendar data range** — The data covers BS 2000–2090. Narrowing to e.g. 2060–2090 would shrink the embedded table. This would require a build-time environment variable to `build.rs` (not yet implemented).

4. **`wasm-bindgen` `--reference-types`** — Requires a browser with the `reference-types` proposal (universally supported since 2022); saves a small but non-trivial slice of the glue code.

For most web applications 3.3 MB is fine over HTTP/2 with gzip (compresses to ~1 MB). For edge/embedded contexts, option 2 is the highest-leverage reduction.

---

## Performance

Benchmarks run with `cargo bench` on an **Apple M5, 10-core, 16 GB RAM** (native release build, `opt-level = "z"`). WASM in Node.js is typically 2–5× slower due to the JS↔WASM boundary.

### Primitive operations

| Operation | Time | Notes |
|---|---|---|
| `bs_to_gregorian` | 76 ns | Linear scan of 91-entry static array |
| `gregorian_to_bs` | 56 ns | Same array, reverse lookup |
| `get_tithi` (override hit) | 187 ns | Hits the 176-entry correction table; skips all astronomy |
| `get_tithi` (override miss) | 200 ns | Falls through to VSOP87 + ELP-2000/82 |
| `get_sunrise` | 153 ns | suncalc only; no planet positions |
| `get_daily_astro_info` | 9.7 µs | Tithi + sun sign + moon sign + nakshatra in one pass |

### Month calendar

| Month | Days | Time |
|---|---|---|
| Shrawan (longest) | 32 | 782 µs |
| Poush (shortest) | 29 | 756 µs |

A full year render (12 months of `get_month_calendar`) costs ~9 ms.

### Tithi instance generation (`generate_tithi_instances`)

The generator walks every BS day in the requested window and evaluates each one against the rule. `BYMONTH` skips non-matching months before doing any astronomy — a single-month filter makes it ~12× faster than unfiltered.

| Rule | 1-year window | 5-year window | 10-year window |
|---|---|---|---|
| Festival with `BYMONTH` (2 months) — e.g. Bijaya Dashami | 9 ms | 40 ms | 86 ms |
| Festival with `BYMONTH` (1 month) — e.g. Shivaratri | 7.8 ms | 37 ms | 68 ms |
| Unfiltered (no `BYMONTH`) — e.g. every Purnima | 19 ms | 95 ms | 180 ms |

### BS solar instance generation (`generate_bs_instances`)

No astronomy — pure date arithmetic. ~4000× cheaper than tithi rules.

| Rule | 1-year window | 5-year window | 10-year window |
|---|---|---|---|
| Annual (e.g. Baisakh 1) | 944 ns | 1.5 µs | 2.3 µs |
| Weekly (52 instances/year) | 9.6 µs | — | — |

### Effect of `UNTIL` / `COUNT`

Without `UNTIL` or `COUNT`, the generator walks every day in whatever window the caller provides — there is no engine-side cap. Callers must provide a bounded window.

| Strategy | Time | Instances |
|---|---|---|
| No bounds, 5-year window, unfiltered | 97 ms | 62 |
| `BYMONTH` = one month, 5-year window | 26 ms | 5 |
| `COUNT=1` (stop at first hit) | 1.0 ms | 1 |
| `COUNT=12` (one year of Purnimas) | 18.6 ms | 12 |

### Practical guidance

| Context | Recommendation |
|---|---|
| Backend, one-month view, <20 events | Fine as-is (~1–8 ms per month in WASM Node.js) |
| Backend, 50+ events per month | Cache `(year, month, eventsHash) → instances`; TTL can be long (events rarely change) |
| Desktop / native (Tauri) | No concern — costs are 2–5× lower than WASM |
| Browser WASM, main thread | Keep window to one month; or move to a Web Worker for anything >16 ms |
| Notification scheduler | Pre-expand to database rows on event save; never re-expand per notification trigger |

Run the benchmarks yourself:

```bash
cd engine/engine
cargo bench --bench engine_perf
# HTML report: target/criterion/report/index.html
```

---

## Nepal festival rules

Ready-to-use BS-RRULE strings for Nepal's major festivals (Dashain, Tihar, Shivaratri,
Teej, Chhath, Buddha Purnima, and 15+ more) with verified Gregorian dates for
BS 2079–2083 are documented in:

**[docs/bs-rrule-spec.md](docs/bs-rrule-spec.md)**

Every rule in that document is backed by the `festival_ground_truth` test suite
(208 tests across 53 festivals × BS 2079–2083, validated against official Nepali Panchanga almanac data).

---

## BS-RRULE specification v2.0

Standard RFC 5545 RRULE does not know about BS months, tithi, paksha, or adhik masa. This library defines **BS-RRULE**, a strict superset of RFC 5545 for the Bikram Sambat calendar system.

The goal: any application that stores recurring event rules as strings can interoperate with this library and with future BS-calendar tools by agreeing on this format.

### Family discriminator

Every BS-RRULE string starts with `X-CALENDAR=<FAMILY>`.

| Family | Meaning |
|---|---|
| `X-CALENDAR=BS` | Solar recurrence anchored in the BS calendar |
| `X-CALENDAR=AD` | Standard RFC 5545 Gregorian recurrence (no extension needed) |
| `X-CALENDAR=PANCHANGA` | Lunar recurrence anchored in tithi/paksha |

Parsers that do not understand `X-CALENDAR` should treat the string as a plain RFC 5545 rule and ignore unknown `X-*` parameters (per iCalendar spec §3.2).

---

### Family: BS (Bikram Sambat solar)

**Required parameters:**

| Parameter | Format | Description |
|---|---|---|
| `X-CALENDAR` | `BS` | Identifies the BS family |
| `FREQ` | `DAILY\|WEEKLY\|MONTHLY\|YEARLY` | Recurrence frequency |
| `DTSTART` | `YYYYMMDD` (BS date) | Anchor date in BS |

**Optional parameters:**

| Parameter | Format | Description |
|---|---|---|
| `INTERVAL` | positive integer | Every N periods (default 1) |
| `COUNT` | positive integer | Stop after N occurrences |
| `UNTIL` | `YYYYMMDD` (BS date) | Stop on or before this BS date |
| `BYMONTH` | comma-separated 1–12 | Restrict to these BS months |
| `BYMONTHDAY` | comma-separated 1–32 | Restrict to these days of month |
| `BYDAY` | `SU,MO,TU,WE,TH,FR,SA` | Restrict to these weekdays |

**Canonical parameter order** (serializers MUST produce this order; parsers MUST accept any order):

```
X-CALENDAR;FREQ;DTSTART[;INTERVAL][;COUNT][;UNTIL][;BYMONTH][;BYMONTHDAY][;BYDAY]
```

**Examples:**

```
# Baisakh 1 every year (Nepali New Year)
X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20810101;BYMONTH=1;BYMONTHDAY=1

# Every Monday in Shrawan (month 4)
X-CALENDAR=BS;FREQ=WEEKLY;DTSTART=20810401;BYMONTH=4;BYDAY=MO

# Every 2 weeks, 10 times
X-CALENDAR=BS;FREQ=WEEKLY;DTSTART=20810101;INTERVAL=2;COUNT=10
```

---

### Family: PANCHANGA (lunar / tithi-based)

**Required parameters:**

| Parameter | Format | Description |
|---|---|---|
| `X-CALENDAR` | `PANCHANGA` | Identifies the lunar family |
| `FREQ` | `MONTHLY` (always) | One lunar cycle per month |
| `DTSTART` | `YYYYMMDD` (BS date) | Anchor date in BS |
| `X-TITHI` | comma-separated tithi names | One or more tithis to match |

**Tithi name vocabulary** (case-insensitive, upper-case canonical):

Shukla Paksha: `SHUKLAPRATIPADA` through `SHUKLACHATURDASHI`, `PURNIMA`  
Krishna Paksha: `KRISHNAPRATIPADA` through `KRISHNACHATURDASHI`, `AMAVASYA`

Shorthand names accepted: `EKADASHI` (matches both pakshas unless `X-PAKSHA` is set), `PURNIMA`, `AMAVASYA`.

**Optional parameters:**

| Parameter | Format | Description |
|---|---|---|
| `X-PAKSHA` | `SHUKLA\|KRISHNA` | Restrict to one paksha |
| `COUNT` | positive integer | Stop after N occurrences |
| `UNTIL` | `YYYYMMDD` (BS date) | Stop on or before this BS date |
| `BYMONTH` | comma-separated 1–12 | Restrict to these BS solar months |
| `X-BYLUNARMONTH` | comma-separated 1–12 | Restrict to these lunar months |
| `X-SKIPADHIK` | `TRUE\|FALSE` | Skip occurrences in adhik masa (default TRUE) |
| `X-TAKE` | `FIRST` | Within each BS year keep only the first qualifying occurrence |

**Canonical parameter order:**

```
X-CALENDAR;FREQ;DTSTART;X-TITHI[;X-PAKSHA][;COUNT][;UNTIL][;BYMONTH][;X-BYLUNARMONTH];X-SKIPADHIK[;X-TAKE]
```

**Examples:**

```
# Every Ekadashi (both pakshas), indefinitely
X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE

# Every Purnima, Shukla Paksha only, 12 times
X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=PURNIMA;X-PAKSHA=SHUKLA;COUNT=12

# Teej — Shukla Tritiya, Bhadra (month 5) only
X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810501;X-TITHI=SHUKLA TRITIYA;X-PAKSHA=SHUKLA;BYMONTH=5;X-SKIPADHIK=TRUE
```

---

### Family: AD (Gregorian passthrough)

When `X-CALENDAR=AD` (or `X-CALENDAR` is absent), the string is a plain RFC 5545 RRULE and is passed unchanged to the standard `rrule` crate. No extensions apply.

```
# Standard Gregorian weekly — no BS-RRULE extensions needed
FREQ=WEEKLY;DTSTART=20240413T000000Z;BYDAY=FR
```

---

### Validation error codes

Implementations that validate BS-RRULE strings SHOULD use these codes in error messages for interoperability:

| Code | Meaning |
|---|---|
| V1 | Malformed parameter (missing `=`) |
| V2 | Missing required `FREQ` |
| V3 | Unrecognized `FREQ` value |
| V4 | Missing required `DTSTART` |
| V5 | `DTSTART` or `UNTIL` is not a valid 8-digit BS date |
| V6 | `INTERVAL` or `COUNT` is zero or non-numeric |
| V7 | `BYMONTH` or `X-BYLUNARMONTH` value outside 1–12 |
| V8 | `BYDAY` contains an unrecognized weekday token |
| V9 | Missing or unrecognized `X-TITHI` value |
| V10 | `X-PAKSHA` value is not `SHUKLA` or `KRISHNA` |
| V11 | `X-TAKE` value is not `FIRST` |

---

### Legacy compatibility (v1.0)

Rules written before v2.0 may use `X-CALENDAR=BS` together with `X-TITHI` for tithi recurrences. The parser treats this combination as `PANCHANGA` family. Serializers MUST emit `X-CALENDAR=PANCHANGA` for new rules; parsers MUST accept the old form indefinitely.

---

## Extending the library

The library is designed to be extended at three levels:

### 1. Custom calendar data provider

Implement `CalendarProvider` to supply your own BS month-length table. The built-in provider covers BS 2000–2090; a custom provider can extend that range:

```rust
use yorion_engine::ports::CalendarProvider;
use yorion_engine::domain::BsMonth;
use yorion_engine::prelude::Result;
use chrono::NaiveDate;

struct MyProvider;
impl CalendarProvider for MyProvider {
    fn get_month_days(&self, year: u16, month: BsMonth) -> Result<u8> { … }
    fn get_first_baisakh(&self, year: u16) -> Result<NaiveDate> { … }
    fn get_year_months(&self, year: u16) -> Result<[u8; 12]> { … }
    fn has_year(&self, year: u16) -> bool { … }
    fn version(&self) -> &str { "custom-2090-2110" }
    fn is_official(&self) -> bool { false }
}

// Pass your provider to CalendarEngine via the full constructor:
// use yorion_engine::adapters::StaticTithiOverrideProvider;
// let engine = CalendarEngine::new_with_provider(
//     Arc::new(MyProvider),
//     Box::new(StaticTithiOverrideProvider::new()),
// );
```

> Note: `CalendarEngine::new()` uses the built-in static provider. A custom-provider constructor is not yet in the public API; contributions welcome.

### 2. Custom tithi overrides

Implement `TithiOverrideProvider` to supply your own correction table:

```rust
use yorion_engine::ports::TithiOverrideProvider;
use yorion_engine::domain::tithi::{Tithi, Location};
use chrono::NaiveDate;

struct MyOverrides;
impl TithiOverrideProvider for MyOverrides {
    fn get_override(&self, date: NaiveDate, location: &Location) -> Option<Tithi> {
        // Return Some(tithi) to override, None to use astronomical calculation
        None
    }
}
```

### 3. New recurrence families

Add a new variant to the `Recurrence` enum in `src/domain/recurrence/recurrence_enum.rs`, implement a `*RecurrenceRule` struct, add serialization via `RRuleParser`, and add a generator in `InstanceGenerator`. The `X-CALENDAR` discriminator in BS-RRULE v2.0 is the correct extension point for future calendar families.

---

## Contributing

The project uses conventional commits for automatic semantic versioning:

```bash
feat: add nakshatra-based recurrence
fix: correct leap year handling in BS 2083
```

Run the full test suite before submitting a PR:

```bash
cd engine
cargo test --all-features          # unit + integration tests (property tests run with the default feature set)
cargo test --test conformance_vectors  # BS-RRULE spec conformance
cargo test --test property_invariants  # proptest roundtrip invariants
```

---

## License

MIT OR Apache-2.0
