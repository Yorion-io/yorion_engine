<div align="center">
  <img src="assets/logo.svg" alt="Yorion Engine" width="120" />

  # YorionEngine

  **The Nepali calendar engine that does more than convert dates.**

  Accurate Bikram Sambat conversion, real panchanga (Hindu almanac) calculation, and a festival recurrence system, in one platform-agnostic Rust library.

  [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
  [![Build](https://img.shields.io/github/actions/workflow/status/Yorion-io/yorion_engine/ci.yml)](https://github.com/Yorion-io/yorion_engine/actions)

</div>

---

## Why this exists

Date conversion between Bikram Sambat and Gregorian is a solved problem. There are dozens of good libraries in JavaScript, Python, Dart, .NET, and more, and if all you need is `2081-01-01 → 2024-04-13`, use one of those.

What none of them do is the part that actually makes a Nepali calendar app hard:

1. **Panchanga.** Tithi, nakshatra, yoga, karana, sunrise and sunset for a real location. Every patro app needs this and most either reimplement it badly or scrape it from a website at runtime.
2. **Festival recurrence.** "Every Ekadashi", "Baisakh 1 every year", "the second Friday of Shrawan". These are lunar and solar rules that standard calendar tooling (RFC 5545 RRULE) simply cannot express.

YorionEngine is built around those two problems. Conversion is included because you need it, but it is the floor, not the headline. The panchanga and recurrence layers are the reason this library is worth depending on.

There is open-source panchanga code out there, but it is Python, tied to the Swiss Ephemeris, tuned to South Indian conventions, and not wired to the BS civil calendar. There is, as far as I can tell, no embeddable, permissively licensed, Nepali-tuned engine that ties all three layers together and compiles to web and mobile. That gap is what this fills.

---

## What you get

| Layer | What it does | Who else does this |
| --- | --- | --- |
| **BS-RRULE recurrence** | Solar (BS), lunar (panchanga), and Gregorian recurrence rules. 53 Nepal festivals encoded, 208 verified tests. | Nobody, that I have found. |
| **Panchanga** | All five angas (tithi, vara, nakshatra, yoga, karana), Lahiri ayanamsa, sunrise/sunset and zodiac for any lat/lon. | Python-only, India-tuned, not BS-integrated. |
| **BS ↔ AD conversion** | BS 1975–2100 (AD 1918–2044), constant-time year lookup + fixed 12-step month scan, validated round-trips. | Many mature options, this matches them. |

All three from one Rust codebase that targets native, WASM (browser and Node.js), and Apple today, with Android, Flutter, and React Native bindings on the roadmap.

---

## The recurrence system (BS-RRULE v2.0)

This is the piece worth your attention. BS-RRULE is a strict superset of RFC 5545 that adds `X-` extension parameters for the Bikram Sambat calendar and the Hindu panchanga cycle. Every rule starts with `X-CALENDAR=<FAMILY>`:

| Family | Anchored in |
| --- | --- |
| `X-CALENDAR=BS` | The BS solar calendar |
| `X-CALENDAR=PANCHANGA` | Tithi / paksha (lunar) |
| `X-CALENDAR=AD` | Standard Gregorian RFC 5545 |

```text
# Nepali New Year, Baisakh 1 every year
X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20810101;BYMONTH=1;BYMONTHDAY=1

# Every Ekadashi, both pakshas
X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE
```

The spec is language and platform independent, so anyone can implement a conformant parser from the document alone. Ready-to-use rules for 53 Nepal festivals ship with the library.

**[Full specification → docs/bs-rrule-spec.md](docs/bs-rrule-spec.md)** · **[Festival reference → docs/festivals.md](docs/festivals.md)**

---

## Why a library, not an API

- **Offline.** Embedded apps do calendar math with no network.
- **Deterministic.** Same input, same output, every time.
- **Zero runtime cost.** Calendar data is baked in at compile time; conversion is constant-time.
- **One core, many targets.** Write the logic once in Rust, ship it everywhere.

---

## Quick start

### Rust

```toml
[dependencies]
yorion_engine = { git = "https://github.com/Yorion-io/yorion_engine", tag = "v0.2.0" }
```

```rust
use yorion_engine::prelude::*;
use chrono::NaiveDate;

let engine = CalendarEngine::new();

// Conversion
let bs = engine.gregorian_to_bs(NaiveDate::from_ymd_opt(2024, 4, 13).unwrap())?;
println!("{}-{}-{}", bs.year, bs.month_u8(), bs.day); // 2081-1-1

// Daily panchanga
let info = engine.get_daily_astro_info(
    NaiveDate::from_ymd_opt(2024, 4, 13).unwrap(),
    Location::kathmandu(),
)?;
println!("Tithi: {}  Nakshatra: {}", info.tithi, info.nakshatra);
```

Recurrence walkthrough: **[docs/recurring-events.md](docs/recurring-events.md)**

### WASM (browser / Node.js)

```js
import init, { gregorian_to_bs, get_month_calendar_with_location }
  from './dist/wasm/bundler/yorion_engine.js';

await init();
const bs  = gregorian_to_bs(2024, 4, 13);          // { year: 2081, month: 1, day: 1 }
const loc = new Location(27.7, 85.3, 'Kathmandu', 345);
const cal = get_month_calendar_with_location(2081, 1, loc);
```

Full TypeScript definitions are included. See **[docs/consuming-wasm.md](docs/consuming-wasm.md)**.

---

## Targets

| Target | Status | Mechanism |
| --- | --- | --- |
| Native Rust | Shipping | `crate` |
| WASM (web, bundler, Node.js) | Shipping | `wasm-bindgen` |
| macOS (arm64 + x86_64) | Shipping | UniFFI → XCFramework |
| iOS (device + simulator) | Planned | UniFFI → XCFramework |
| Android / Kotlin | Planned | UniFFI → `.aar` |
| Flutter / Dart | Planned | `flutter_rust_bridge` |
| React Native | Planned | `uniffi-bindgen-react-native` |
| Linux / Windows | Planned | `cdylib` |

The engine already compiles to every target above. The planned rows are missing only CI packaging scripts. Contributions welcome.

Pre-built WASM and Apple artifacts ship with every [release](https://github.com/Yorion-io/yorion_engine/releases).

---

## Accuracy and maintenance (read this before depending on it)

Two coverage limits that do **not** move together:

| Layer | Covered through | Self-extending? |
| --- | --- | --- |
| BS ↔ AD conversion | BS 2100 | Yes, fixed table |
| Tithi (panchanga) accuracy | BS 2083 | **No, needs a yearly update** |

Conversion is a fixed, validated table out to BS 2100. Tithi values are astronomical output corrected against the official Nepali Panchanga almanac, which currently stops at **BS 2083**. Past that year, tithis are raw astronomical output with no almanac correction.

There is exactly one recurring maintenance task: when a new official patro is published, add its reference almanac, regenerate the overrides, and bump `TITHI_VERIFIED_THROUGH_BS`. Full runbook in [CONTRIBUTING.md](CONTRIBUTING.md#yearly-maintenance-refreshing-tithi-data-when-a-new-almanac-is-published). I am calling this out up front so it is a known quantity, not a surprise.

Calendar data is derived from the [`nepali-datetime`](https://github.com/opensource-nepal/nepali_datetime) reference table and spot-checked against Hamro Patro. Every year's month lengths are checked against its anchor gap at both build time and test time.

---

## Architecture

Ports and adapters (hexagonal). `CalendarEngine` is the single public entry point, everything beneath it is an implementation detail. Full walkthrough in [docs/architecture.md](docs/architecture.md).

---

## Contributing

Build setup and test commands are in [CONTRIBUTING.md](CONTRIBUTING.md). This project uses [Conventional Commits](https://www.conventionalcommits.org/).

```bash
cd engine && cargo test --all-features
```

If you maintain a Nepali calendar package and want an accurate panchanga or festival source under the hood, open an issue. That is exactly the use case this is built for.

---

## License

MIT OR Apache-2.0
