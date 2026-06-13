# 08 — Rust Calendar Engine

**Path:** `engine/`
**Language:** Rust 2021
**Outputs:** WASM (NodeJS + Web), Native (Swift/Kotlin via UniFFI)

---

## What the Engine Does

All BS calendar math is here:
1. Convert a BS date to a Gregorian date (and vice versa)
2. Calculate which Tithi (lunar day) a given date falls on
3. Expand a BS/Tithi/AD recurrence rule into a list of dates

The engine has **no network calls, no filesystem access, no side effects**. It is pure math.

---

## Architecture (Hexagonal)

```
┌─────────────────────────────────────────────────────┐
│                   core_api.rs                       │
│         Stable public interface                     │
│   CalendarEngine { conversion, astronomical,        │
│                    instance_gen, tithi_gen }        │
└──────────┬──────────────────────────────────────────┘
           │
     ┌─────▼──────────────────────────────────────┐
     │                  Services                  │
     │  ConversionService  AstronomicalService    │
     │  InstanceGenerator  TithiInstanceGenerator │
     └────────────────────┬───────────────────────┘
                          │  depends on
     ┌────────────────────▼───────────────────────┐
     │                   Ports                    │
     │  CalendarProvider  TimeProvider            │
     │  TithiOverrideProvider                     │
     └────────────────────┬───────────────────────┘
                          │  implemented by
     ┌────────────────────▼───────────────────────┐
     │                  Adapters                  │
     │  StaticCalendarProvider  (anchor table)    │
     │  SystemTimeProvider                        │
     │  StaticTithiOverrideProvider               │
     └────────────────────────────────────────────┘

     ┌────────────────────────────────────────────┐
     │                  Bindings                  │
     │  wasm.rs   (wasm-bindgen → JS)             │
     │  uniffi_bindings.rs (UniFFI → Swift/Kotlin)         │
     └────────────────────────────────────────────┘
```

---

## Domain Types

### `BsDate`
```rust
pub struct BsDate {
    pub year: u32,    // e.g. 2081
    pub month: u8,    // 1–12 (Baishakh–Chaitra)
    pub day: u8,      // 1–32 (BS months vary in length)
}
```

### `Tithi`
```rust
pub struct Tithi {
    pub number: u8,    // 1–15 within paksha
    pub paksha: Paksha, // Shukla (bright) or Krishna (dark)
    pub name: String,  // e.g. "Ekadashi"
}
```

### `EventInstance`
```rust
pub struct EventInstance {
    pub bs_date: BsDate,
    pub ad_date: NaiveDate,
    pub tithi: Option<Tithi>,
}
```

### `RecurrenceRule` (enum)
```rust
pub enum RecurrenceRule {
    BS(BsRule),          // FREQ + X-CALENDAR=BS
    Tithi(TithiRule),    // X-TITHI + X-PAKSHA
    AD(String),          // Standard RRULE string
}
```

---

## BS ↔ Gregorian Conversion

### The Problem

There is no closed-form formula for BS↔AD conversion. The BS calendar is a solar calendar but with months that vary in length (28–32 days) set each year by the Nepal government. The engine uses **anchor data**.

### Anchor Data

`src/adapters/static_calendar.rs` contains a table of anchor points, one per BS year from BS 1975 to BS 2100:

```
BS 2079 Baishakh 1  =  AD 2022 April 14
BS 2080 Baishakh 1  =  AD 2023 April 14
BS 2081 Baishakh 1  =  AD 2024 April 13
...
```

And monthly day-counts:
```
BS 2081: [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30]
         Bai Jes Ash Shr Bha Ash  Kar Mng Pou Magh Fal Chai
```

### Conversion Algorithm

**BS → AD:**
1. Find the anchor: BS year's Baishakh 1 = AD date X
2. Count days from Baishakh 1 to the target BS date (using monthly day-counts)
3. Add those days to anchor AD date X
4. Return result

**AD → BS:**
1. Binary search anchor table to find the BS year whose Baishakh 1 ≤ the target AD date
2. Count forward day-by-day until we reach the target AD date
3. Track which BS month/day we're on
4. Return BsDate

**Accuracy:** Exact (no floating point). The anchor table is authoritative.

---

## Tithi Calculation

`src/services/astronomical.rs`

A Tithi is a lunar day — 1/30th of the synodic month. There are 30 tithis in a lunar month, 15 in Shukla Paksha (waxing) and 15 in Krishna Paksha (waning).

### Algorithm

```rust
fn get_tithi(date: NaiveDate) -> Tithi:
  1. Compute Sun's ecliptic longitude at noon (using suncalc)
  2. Compute Moon's ecliptic longitude at noon
  3. elongation = moon_longitude - sun_longitude (mod 360)
  4. tithi_number = floor(elongation / 12) + 1    // 1–30
  5. if tithi_number <= 15: paksha = Shukla, number = tithi_number
     else: paksha = Krishna, number = tithi_number - 15
```

**Why noon?** Tithi transitions can happen at any time of day. Using noon gives the tithi that "rules" most of that day. Astronomically precise to within minutes.

**Location dependency:** The exact time of the Moon's position varies slightly by longitude. For most purposes (Nepal-centric calendar), a fixed longitude (Kathmandu: 85.3°E) is used.

---

## Instance Generation

### BS Recurrence (`InstanceGenerator`)

```rust
fn generate_bs_instances(rule: BsRule, start: BsDate, end: BsDate) -> Vec<BsDate>:
  match rule.freq:
    YEARLY  → iterate years from start.year to end.year:
                 find month/day in rule → emit BsDate if valid
    MONTHLY → iterate months:
                 clamp day to month length (e.g. day 32 → 29 in short month)
                 emit BsDate
    WEEKLY  → iterate weeks, check BYDAY
    DAILY   → iterate every day
  Apply COUNT and UNTIL limits
```

### Tithi Recurrence (`TithiInstanceGenerator`)

```rust
fn generate_tithi_instances(rule: TithiRule, start: BsDate, end: BsDate) -> Vec<EventInstance>:
  // For each calendar month in range:
  //   Search for the day where tithi matches rule.tithi_number + rule.paksha
  //   If found → emit EventInstance { bs_date, ad_date, tithi }
```

Tithi search is O(30) per month — check each day's tithi until a match is found. Since tithis progress 1 per ~24 hours, a simple day-by-day scan is efficient enough.

---

## Public API (`core_api.rs`)

```rust
pub struct CalendarEngine {
    conversion: Arc<ConversionService>,
    astronomical: Arc<AstronomicalService>,
    instance_gen: InstanceGenerator,
    tithi_gen: TithiInstanceGenerator,
}

impl CalendarEngine {
    pub fn new() -> Self
    pub fn bs_to_gregorian(&self, date: BsDate) -> Result<NaiveDate, EngineError>
    pub fn gregorian_to_bs(&self, date: NaiveDate) -> Result<BsDate, EngineError>
    pub fn get_tithi(&self, date: NaiveDate) -> Result<Tithi, EngineError>
    pub fn get_sun_zodiac(&self, date: NaiveDate) -> ZodiacSign
    pub fn generate_bs_instances(&self, rule: &str, start: BsDate, end: BsDate)
        -> Result<Vec<BsDate>, EngineError>
    pub fn generate_tithi_instances(&self, rule: &str, start: BsDate, end: BsDate)
        -> Result<Vec<EventInstance>, EngineError>
    pub fn get_current_bs_date(&self) -> Result<BsDate, EngineError>
    pub fn get_month_info(&self, bs_year: u32, bs_month: u8) -> Result<MonthInfo, EngineError>
}
```

---

## WASM Bindings (`src/wasm.rs`)

Uses `wasm-bindgen` to expose the Rust API to JavaScript.

```rust
#[wasm_bindgen]
pub struct CalendarEngine { ... }

#[wasm_bindgen]
impl CalendarEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CalendarEngine

    pub fn bs_to_gregorian(&self, year: u32, month: u8, day: u8) -> Result<JsValue, JsValue>
    pub fn gregorian_to_bs(&self, year: i32, month: u8, day: u8) -> Result<JsValue, JsValue>
    pub fn get_tithi(&self, year: i32, month: u8, day: u8) -> Result<JsValue, JsValue>
    pub fn generate_instances(&self, rule: &str, start_bs: JsValue, end_bs: JsValue)
        -> Result<JsValue, JsValue>
}
```

JavaScript types (`JsValue`) are serialized/deserialized via `serde_json`. TypeScript declarations are auto-generated by `wasm-bindgen`.

### Build Targets

```bash
# NodeJS target (for backend)
wasm-pack build --target nodejs --out-dir pkg/nodejs

# Web target (for browser)
wasm-pack build --target web --out-dir pkg/web

# Bundler target (for webpack/vite)
wasm-pack build --target bundler --out-dir pkg/bundler
```

---

## Native Bindings (UniFFI)

`src/uniffi_bindings.rs` uses Mozilla's UniFFI framework to generate FFI bindings for:
- **Swift** → iOS/macOS app (future)
- **Kotlin** → Android app (future)

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
uniffi = { version = "0.27", features = ["build"] }
```

The same `CalendarEngine` struct is exposed via UniFFI attributes. Mobile apps can use the same engine without re-implementing any calendar logic.

---

## Testing

```
engine/tests/
├── conversion_tests.rs      ← BS↔AD round-trip accuracy
├── tithi_tests.rs           ← Known tithis verified against published calendars
├── recurrence_tests.rs      ← Expansion output verified manually
└── edge_cases.rs            ← Month boundaries, year boundaries, leap months
```

Property tests (proptest): `∀ bs_date ∈ valid range: gregorian_to_bs(bs_to_gregorian(bs_date)) == bs_date`

---

## Distribution

The engine is distributed as a compiled WASM binary via GitHub Releases:
- `bs_calendar_core.js` — JS glue code (NodeJS target)
- `bs_calendar_core_bg.wasm` — Actual WASM binary
- `bs_calendar_core.d.ts` — TypeScript declarations

Backend and frontend both download these via:
```bash
npm run engine:download
# → downloads to dist/wasm/nodejs/ (backend) or public/wasm/ (frontend)
```

Version is controlled via `BS_ENGINE_VERSION` env var or `package.json` scripts.

---

## Performance

Benchmarks run with `cargo bench` on an **Apple M5, 10-core, 16 GB RAM** (native release build, `opt-level = "z"`). WASM in Node.js is typically 2–5× slower due to the JS↔WASM boundary.

### Primitive operations

| Operation | Time | Notes |
|---|---|---|
| `bs_to_gregorian` | 76 ns | O(1) index lookup into a 126-entry static array |
| `gregorian_to_bs` | 56 ns | Same array, reverse lookup |
| `get_tithi` (override hit) | 187 ns | Hits the 176-entry correction table; skips all astronomy |
| `get_tithi` (override miss) | 200 ns | Falls through to VSOP87 + ELP-2000/82 |
| `get_sunrise` | 153 ns | suncalc only; no planet positions |
| `get_daily_astro_info` | 9.7 µs | Tithi + sun sign + moon sign + nakshatra + yoga + karana in one pass |

### Month calendar

| Month | Days | Time |
|---|---|---|
| Shrawan (longest) | 32 | 782 µs |
| Poush (shortest) | 29 | 756 µs |

A full year render (12 months of `get_month_calendar`) costs ~9 ms.

### Tithi instance generation

The generator walks every BS day in the requested window and evaluates each one against the rule. `BYMONTH` skips non-matching months before doing any astronomy — a single-month filter makes it ~12× faster than unfiltered.

| Rule | 1-year | 5-year | 10-year |
|---|---|---|---|
| Festival with `BYMONTH` (2 months) | 9 ms | 40 ms | 86 ms |
| Festival with `BYMONTH` (1 month) | 7.8 ms | 37 ms | 68 ms |
| Unfiltered (no `BYMONTH`) | 19 ms | 95 ms | 180 ms |

### BS solar instance generation

No astronomy — pure date arithmetic. ~4000× cheaper than tithi rules.

| Rule | 1-year | 5-year | 10-year |
|---|---|---|---|
| Annual (e.g. Baisakh 1) | 944 ns | 1.5 µs | 2.3 µs |
| Weekly (52 instances/year) | 9.6 µs | — | — |

### Practical guidance

| Context | Recommendation |
|---|---|
| Backend, one-month view, <20 events | Fine as-is |
| Backend, 50+ events per month | Cache `(year, month, eventsHash) → instances` |
| Desktop / native (Tauri) | No concern — costs are 2–5× lower than WASM |
| Browser WASM, main thread | Keep window to one month; use a Web Worker for >16 ms |
| Notification scheduler | Pre-expand to database rows on event save |

```bash
cd engine && cargo bench --bench engine_perf
# HTML report: target/criterion/report/index.html
```

---

## Extending the library

### Custom calendar data provider

Implement `CalendarProvider` to supply your own BS month-length table (e.g. to extend coverage beyond BS 2100):

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
    fn version(&self) -> &str { "custom-2100-2120" }
    fn is_official(&self) -> bool { false }
}
```

> A `new_with_provider` constructor is not yet in the public API — contributions welcome.

### Custom tithi overrides

Implement `TithiOverrideProvider` to supply your own correction table:

```rust
use yorion_engine::ports::TithiOverrideProvider;
use yorion_engine::domain::tithi::{Tithi, Location};
use chrono::NaiveDate;

struct MyOverrides;
impl TithiOverrideProvider for MyOverrides {
    fn get_override(&self, date: NaiveDate, location: &Location) -> Option<Tithi> {
        None // return Some(tithi) to override astronomical calculation
    }
}
```

### New recurrence families

Add a new variant to `Recurrence` in `src/domain/recurrence/recurrence_enum.rs`, implement a `*RecurrenceRule` struct, add serialization in `RRuleParser`, and a generator in `InstanceGenerator`. The `X-CALENDAR` discriminator in BS-RRULE v2.0 is the correct extension point.
