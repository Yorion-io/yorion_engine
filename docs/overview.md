A platform-agnostic Rust library for the **Bikram Sambat (BS)** calendar — the official calendar of Nepal. Accurate date conversion, astronomical (panchanga) calculations, and a recurrence-rule system for scheduling events in both solar and lunar terms.

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

- **Offline-capable** — embedded apps need no network for calendar math.
- **Deterministic** — same input always produces the same output.
- **Zero runtime cost** — calendar data is baked in at compile time; lookups are O(1).
- **Cross-platform** — one Rust codebase, multiple output targets.

---

## Coverage

| Capability | Range / Detail |
|---|---|
| BS ↔ AD conversion | BS 1975–2100 (AD 1918–2044), O(1) table lookup |
| Panchanga angas | All five: tithi, vara, nakshatra, yoga, karana |
| Tithi accuracy | Validated against official Nepali Panchanga almanac through BS 2083 (`TITHI_VERIFIED_THROUGH_BS`); later years are unverified astronomical output |
| Tithi exception overrides | CSV-driven corrections baked in at build time |
| Tithi end times | `get_tithi_end` exposes the transition instant |
| Ayanamsa | Lahiri (Chitrapaksha), computed per date |
| Astronomical | Sunrise/sunset + zodiac + nakshatra + yoga + karana for any lat/lon |
| Recurrence | BS-solar, AD-solar, panchanga-lunar |

---

## Calendar data source

BS month lengths are **not derivable from a formula** — they vary year to year and must come from a published table. The embedded table in [`engine/data/bs_calendar_data.json`](engine/data/bs_calendar_data.json) covers **BS 1975–2100** and, for each year, stores:

- the **1 Baisakh anchor** — the Gregorian date of the BS new year, and
- the **12 month lengths** (Baisakh … Chaitra, each 29–32 days).

**Authoritative source.** The full table — all 126 years (BS 1975–2100) — is generated from the [`nepali-datetime`](https://github.com/opensource-nepal/nepali_datetime) reference table (Python `opensource-nepal/nepali_datetime`), spot-checked against [Hamro Patro](https://www.hamropatro.com/calendar). Each anchor is derived by cumulative day-summing from the reference's epoch (1 Baisakh 1975 = 1918-04-13); the derived anchors reproduce known dates exactly (e.g. 1 Baisakh 2081 = 2024-04-13).

**Consistency is enforced, not assumed.** Every year's 12 month lengths must sum to exactly the number of days between its anchor and the next year's anchor. This invariant is checked in two places:

- at **build time** — `build.rs` emits a compile warning for any year whose months don't match its anchor gap;
- at **test time** — `month_lengths_match_anchor_gaps` (in `tests/property_invariants.rs`) asserts it across the whole range, and a full-range BS↔AD↔BS round-trip property test guards every valid date.

When extending the table beyond BS 2100, add new years from the same reference and let those checks confirm internal consistency.

### Two separate coverage limits

Date conversion and tithi accuracy do **not** extend together:

| Layer | Covered through | Self-extending? |
|---|---|---|
| BS ↔ AD **date conversion** | BS 2100 | n/a — fixed table |
| **Tithi** (panchanga) accuracy | BS 2083 (`TITHI_VERIFIED_THROUGH_BS`) | **No — needs a yearly update** |

Tithi values come from astronomical calculation corrected by `engine/data/tithi_exceptions.csv`, which is validated against published almanac files that currently stop at BS 2083. Beyond that year, tithis are raw astronomical output with no almanac correction.

**Yearly maintenance:** when a new official patro is published, add its reference almanac CSV, regenerate the overrides, and bump `TITHI_VERIFIED_THROUGH_BS`. This is the calendar's only recurring data task — full step-by-step runbook in [CONTRIBUTING.md → Yearly maintenance](CONTRIBUTING.md#yearly-maintenance-refreshing-tithi-data-when-a-new-almanac-is-published).

---

## Architecture

Ports & Adapters (Hexagonal) layout:

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

`CalendarEngine` is the single entry point. Everything beneath it is an implementation detail. See [docs/architecture.md](docs/architecture.md) for a full walkthrough.

---

## Getting started — Rust

```toml
[dependencies]
yorion_engine = { git = "https://github.com/Yorion-io/yorion_engine", tag = "v0.1.4" }
```

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

// Daily panchanga
let info = engine.get_daily_astro_info(ad, Location::kathmandu())?;
println!("Tithi: {}  Nakshatra: {}", info.tithi, info.nakshatra);
```

For recurring events (BS-solar, tithi-based) see [docs/recurring-events.md](docs/recurring-events.md).

---

## Getting started — WASM (browser / Node.js)

```javascript
import init, { gregorian_to_bs, get_tithi, get_month_calendar_with_location }
  from './dist/wasm/bundler/yorion_engine.js';

await init();
const bs  = gregorian_to_bs(2024, 4, 13);   // { year: 2081, month: 1, day: 1 }
const loc = new Location(27.7, 85.3, 'Kathmandu', 345);
const cal = get_month_calendar_with_location(2081, 1, loc);
```

Full TypeScript definitions (`.d.ts`) are included. Browser (ES module) and Node.js targets also available. See [docs/consuming-wasm.md](docs/consuming-wasm.md).

---

## Pre-built binaries

Every [GitHub Release](https://github.com/Yorion-io/yorion_engine/releases) ships:

| Artifact | Contents | Target |
|---|---|---|
| `wasm-assets-{version}.tar.gz` | `web/`, `bundler/`, `nodejs/` WASM bundles + `.d.ts` | Browser, Node.js |
| `apple-assets-{version}.tar.gz` | `BsCalendarCore.xcframework` + `yorion_engine.swift` | macOS (arm64 + x86_64) |

For Swift / Xcode setup see [docs/swift-integration.md](docs/swift-integration.md).

### Planned targets

The engine compiles to all targets below — only the CI packaging scripts are missing. Contributions welcome.

| Target | Mechanism |
|---|---|
| iOS (device + simulator) | UniFFI → XCFramework |
| Android / Kotlin | UniFFI → `.aar` |
| Linux x86_64 | `cdylib` → `.so` |
| Windows x86_64 | `cdylib` → `.dll` |
| Flutter / Dart | `flutter_rust_bridge` |
| React Native | `uniffi-bindgen-react-native` |

---

## BS-RRULE specification v2.0

This library defines **BS-RRULE** — a strict superset of RFC 5545 that adds `X-` extension parameters for the Bikram Sambat calendar and Hindu panchanga cycle.

Every BS-RRULE string starts with `X-CALENDAR=<FAMILY>`:

| Family | Meaning |
|---|---|
| `X-CALENDAR=BS` | Solar recurrence anchored in the BS calendar |
| `X-CALENDAR=PANCHANGA` | Lunar recurrence anchored in tithi / paksha |
| `X-CALENDAR=AD` | Standard RFC 5545 Gregorian (no extensions) |

```
# Nepali New Year — Baisakh 1 every year
X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20810101;BYMONTH=1;BYMONTHDAY=1

# Every Ekadashi (both pakshas)
X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE
```

The spec is language- and platform-agnostic — any developer can implement a conformant parser from it alone. Ready-to-use rules for 53 Nepal festivals (208 verified tests) are included.

**[Full specification → docs/bs-rrule-spec.md](docs/bs-rrule-spec.md)** · **[Festival reference → docs/festivals.md](docs/festivals.md)**

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build setup and test commands. This project uses [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add nakshatra-based recurrence
fix: correct sunrise calculation at high latitudes
```

Quick test run:

```bash
cd engine && cargo test --all-features
```

---

## License

MIT OR Apache-2.0