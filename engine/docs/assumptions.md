# Engine Assumptions

This document catalogues every assumption the library makes — algorithmic, calendrical, astronomical, and behavioural. A developer integrating this library or extending it should read this before filing a bug or changing core logic.

---

## 1. Calendar Data

### 1.1 Supported range: BS 2000–2090

All BS ↔ AD conversion depends on a pre-computed table of anchor points (first day of each BS year in Gregorian) and the 12 month lengths per year. That table covers **BS 2000–2090 only** (AD 1943–2033). Any date outside this range throws `YorionError::CalendarDataNotFound`.

Source: `data/bs_calendar_data.json`, embedded at compile time via `build.rs`.

### 1.2 Month lengths are tabular, not formulaic

There is no mathematical formula for BS month lengths. They vary year to year (typically 29–32 days) based on official government publication. The library treats the embedded table as ground truth. Any error in the table propagates silently into every conversion, recurrence, and astronomical calculation.

### 1.3 BS year ≈ AD year + 57

The conversion service seeds its search with `bs_year ≈ ad_year + 57`. The actual offset drifts slightly around the New Year boundary (before/after 1st Baisakh), so the algorithm also tries `bs_year - 1` as a fallback. The +57 approximation is never used directly as a result — only as a starting point for the anchor-point lookup.

Source: `services/conversion.rs:35`.

### 1.4 BYMONTHDAY=32 is the last-day sentinel

A recurrence rule of `BYMONTHDAY=32` is interpreted as "the last day of every month" because BS months never have 32 days. The value is clamped to the actual month length at expansion time. This convention is intentional but not part of the RFC 5545 standard.

Source: `services/instance_generator.rs:265`.

---

## 2. Astronomical Calculations

### 2.1 Ayanamsa fixed at 24.0°

All sidereal longitude conversions (tithi, zodiac, nakshatra) subtract a fixed ayanamsa of **24.0°** from the tropical ecliptic longitude. This is an approximation of the Lahiri ayanamsa for the present epoch.

The real Lahiri ayanamsa drifts by ~50 arcseconds per year (≈ 0.014°/year). For dates near the present the error is small; for dates near the edge of the supported range (BS 2000, i.e. AD 1943) it introduces roughly 1° of error, which may shift zodiac sign or nakshatra assignments near sign boundaries.

**This is not configurable.** If a tradition uses a different ayanamsa (Raman, Krishnamurti, etc.) the results will differ.

Source: `services/astronomical.rs:327`.

### 2.2 Tithi = one-twelfth of the moon-sun elongation cycle

Each tithi spans exactly **12°** of moon-sun elongation. Tithi index = `floor(elongation / 12°) + 1`. This is the standard Vedic definition and is mathematically correct; the approximation here is purely in the planetary positions (see §2.5).

Source: `services/astronomical.rs:127`.

### 2.3 Nakshatra divisions are equal (13°20′ each)

The library divides the ecliptic into **27 equal segments** of 360°/27 ≈ 13.333° each. Some traditions use unequal nakshatra widths. Equal divisions are used universally in software calendar implementations and differ from the unequal scheme only at nakshatra boundaries.

Source: `services/astronomical.rs:318`.

### 2.4 Amavasya search uses a 12.19°/day lunar drift rate

The iterative new-moon (Amavasya) finder advances or retreats by `(target_distance / 12.19)` days per step, clamped to [0.0001, 1.0] days to prevent overshooting. This constant is the mean moon-sun relative angular velocity in degrees per day. The actual rate varies from ~11.7° to ~13.0°/day, so the search always converges but may require up to 2000 iterations near apogee.

The search terminates when the moon-sun elongation is within **0.001°** of 0° (new moon) or 180° (full moon). Amavasya is accepted when `elongation ∉ (0.001°, 359.999°)`.

Source: `services/astronomical.rs:208–264`.

### 2.5 Planetary positions from VSOP87 (Sun) and ELP-2000/82 (Moon)

The `astro` crate implements these theories. VSOP87 is accurate to ~1 arcsecond for the Sun over the period AD 1900–2100. ELP-2000/82 is accurate to ~10 arcseconds for the Moon. Tithi assignment near a boundary (where the elongation is close to a multiple of 12°) can differ by ±1 tithi from a reference calendar using higher-precision ephemeredes.

This is the primary source of astronomical discrepancy with published Panchanga sources. The tithi override system (§6) exists to paper over these discrepancies for known dates.

### 2.6 All astronomical calculations use noon UTC as the reference moment for zodiac/nakshatra

When computing zodiac sign or nakshatra for a date (not at sunrise), the engine uses `date + 12:00:00 UTC` as the reference moment. For tithi, the reference is local sunrise (see §2.7).

Source: `core_api.rs:103, 110, 117`.

### 2.7 Tithi is determined at local sunrise

The tithi assigned to a calendar day is the tithi active at the moment of **local sunrise** at the given location. This is the standard Hindu calendar convention. If a tithi begins or ends during daylight hours, only the sunrise-tithi is recorded for that day.

Consequences:
- **A3 — Tithi Vriddhi (tithi spans two sunrises):** A tithi that is active at two consecutive sunrises produces two instances in a tithi recurrence. Both days are correct.
- **A4 — Tithi Kshaya (tithi skips a sunrise):** A tithi that is not active at any sunrise in a lunar month produces zero instances. No error is raised; the occurrence is simply absent.

Source: `services/astronomical.rs:138–149`, `services/instance_generator.rs:493–502`.

---

## 3. Location

### 3.1 Kathmandu is the hardcoded default for tithi calculations

When `CalendarEngine::get_tithi()` is called, the location defaults to `Location::KATHMANDU` (27.7172°N, 85.3240°E, +5:45 UTC). All callers of the WASM or UniFFI APIs that do not supply a location receive Kathmandu-based tithi results.

Source: `core_api.rs:95–96`, `domain/tithi.rs:411–417`.

### 3.2 Timezone is a fixed offset in minutes

The `Location` struct stores `timezone_offset_mins: i32`. There is no DST support. Kathmandu is +345 minutes (+5:45) year-round, which is correct for Nepal (Nepal does not observe DST). For locations that do observe DST the caller must supply the correct offset for the date in question — the library never adjusts it.

### 3.3 Nepal social calendar flag controls whether overrides apply

`Location.follow_nepal_social_calendar: bool` gates the tithi override table (§6). It defaults to `true` for `Location::KATHMANDU` and `false` for all other predefined and custom locations. A caller using a custom Kathmandu-area location constructed manually must set this flag to `true` explicitly to receive override-corrected tithis.

Source: `adapters/static_overrides.rs:28`, `domain/tithi.rs:449`.

---

## 4. Recurrence Rules

### 4.1 Week starts on Sunday (SU = 0)

All weekday arithmetic uses Sunday = 0, Monday = 1, …, Saturday = 6. This matches ISO weekday codes used in most iCalendar implementations but differs from ISO 8601 (which defines Monday = 1). There is no option to change this.

Source: `domain/recurrence/rrule_parser.rs:157–172`, `services/instance_generator.rs:231`.

### 4.2 FREQ=MONTHLY with BYMONTH is a *limit*, not an expand

For monthly frequency, `BYMONTH` filters out months that do not match; it does not generate additional months. For yearly frequency, `BYMONTH` *replaces* the month of the anchor date. This follows RFC 5545 §3.8.5.

### 4.3 TithiRecurrenceRule defaults skip_adhik = true

Unless explicitly set to `false`, tithi recurrence rules skip occurrences that fall in an adhik masa (intercalary lunar month). The skip is only honoured in the `TithiInstanceGenerator` path (used by `CalendarEngine`). The legacy `InstanceGenerator::generate_tithi_instances` path accepts the flag but does not enforce it.

Source: `domain/recurrence/tithi_rules.rs:49`, `services/tithi_generator.rs:149`.

### 4.4 Bare tithi name (e.g. "EKADASHI") defaults to the Shukla variant in the parser

When a tithi name is given without a paksha qualifier in the RRULE string (`X-TITHI=EKADASHI`), the parser resolves it to the Shukla variant. However, `TithiRecurrenceRule::ekadashi()` sets `paksha_filter = None`, which means both pakshas match at evaluation time. The asymmetry is in the parser only; the rule struct stores the intent correctly.

Source: `domain/tithi.rs:341–357`, `domain/recurrence/tithi_rules.rs:68–79`.

### 4.5 Adhik month detection is expensive and cached

`is_adhik_month()` requires finding the previous and next Amavasya, each an iterative search (up to 2000 iterations each). The `TithiInstanceGenerator` caches this result per lunar cycle (~29.5 days), refreshing when the current Julian Day passes the cached Amavasya JD.

Source: `services/tithi_generator.rs:65–139`.

### 4.6 AD recurrence rules are pass-through to the `rrule` crate

`AdRecurrenceRule` stores the raw RRULE string and delegates entirely to the `rrule` v0.14 crate for expansion. All RFC 5545 semantics (EXDATE, RDATE, BYYEARDAY, BYWEEKNO, etc.) are the `rrule` crate's responsibility. The BS engine does no AD-specific validation beyond checking that the string is parseable.

### 4.7 Instance generation is capped at 10,000 occurrences

`generate_bs_instances_with_clamp()` breaks the expansion loop after producing 10,000 instances. This is a safety guard against infinite rules (no COUNT, no UNTIL). It is not surfaced as an error; callers receive a silently truncated list.

Source: `services/instance_generator.rs:124–127`.

---

## 5. Day Clamping (A1 / A2)

### A1 — Non-existent BYMONTHDAY target

When a `BsRecurrenceRule` specifies `BYMONTHDAY=30` and the target month has only 29 days, the occurrence is moved to day 29. The engine signals this by returning `Some(intended_unclamped_date)` alongside the real date in `generate_bs_instances_with_clamp()`. `CalendarEngine::generate_event_instances()` stores this on the resulting `EventInstance` as `is_exception = true` and `original_date = Some(intended)`.

This is silent to callers of `generate_bs_instances()` (the non-clamp variant) — they receive only the clamped date with no indication it was adjusted.

### A2 — Two BYMONTHDAY values clamp to the same day

If `BYMONTHDAY=30,31` is used for a 29-day month, both targets clamp to day 29. The de-duplication step produces one instance, not two. The instance carries the A1 signal for the first target; the second target is silently discarded.

Source: `services/instance_generator.rs:287–295`.

---

## 6. Tithi Overrides

### 6.1 Purpose

The astronomical tithi calculation (§2.2–2.5) disagrees with published Panchanga sources on specific dates. The override table records known discrepancies and forces the correct value.

### 6.2 Source

Overrides are loaded from `data/tithi_exceptions.csv` at compile time. The CSV columns are: `id, AD_date (YYYY-MM-DD), tithi_name`. The build script generates a static Rust array `TITHI_OVERRIDES: [((i32, u8, u8), Tithi); N]`.

As of the current data set there are **176 override entries**, covering dates where the engine's astronomical output differs from the official Nepali Panchanga.

Source: `build.rs:166–222`, `adapters/static_overrides.rs`.

### 6.3 Coverage gap after BS 2083

The reference almanac covers tithi data only through BS 2083 (AD ~2027). Override entries beyond this date cannot be verified against an authoritative source. After BS 2083, tithi values come from the raw astronomical calculation with no correction applied.

### 6.4 Override is checked before astronomical calculation

When `follow_nepal_social_calendar = true`, the override table is consulted first. If an entry exists for the date, it is returned immediately without performing any astronomical computation. The `is_overridden` flag on `DailyAstroInfo` and `CalendarDay` is set to `true` in this case.

Source: `services/astronomical.rs:109–115`.

---

## 7. Data Integrity

### 7.1 Calendar data is trusted without runtime validation

The embedded `CALENDAR_DATA` array is generated from `bs_calendar_data.json` at build time with no cross-validation at runtime. A corrupt or incorrect `bs_calendar_data.json` will produce incorrect conversions silently.

### 7.2 Tithi overrides are trusted without verification

The override table is applied unconditionally when the location flag is set. There is no runtime check that the override value is plausible (e.g. that the neighbouring tithis are consistent). An incorrect override entry silently propagates into all downstream calculations.

### 7.3 Build-time generation requires correct Tithi names in the CSV

`build.rs` calls `Tithi::from_name()` for each row in `tithi_exceptions.csv`. An unrecognised tithi name causes a **build-time panic**, not a compile error. The error message names the offending row.

Source: `build.rs:198–202`.

---

## 8. API / Serialization

### 8.1 RRULE strings are the serialization format for recurrence rules

`BsRecurrenceRule` and `TithiRecurrenceRule` implement `Serialize`/`Deserialize` as plain strings in BS-RRULE v2.0 format. Any storage layer (database column, JSON field) that holds a recurrence rule holds a string. There is no binary or structured serialization.

### 8.2 BS dates in RRULE strings are always YYYYMMDD

Both `DTSTART` and `UNTIL` use the compact 8-digit format with no separators, no time component, and no timezone suffix. The date is always a BS calendar date in the BS and PANCHANGA families.

### 8.3 FREQ is always MONTHLY in PANCHANGA rules

`TithiRecurrenceRule::to_rrule()` hard-codes `FREQ=MONTHLY` in the serialized string regardless of the actual recurrence pattern. This is a placeholder; the frequency is not used by the tithi generator (which iterates day-by-day). Parsers must not infer "once per month" semantics from this field in a PANCHANGA rule.

Source: `services/tithi_generator.rs` (day-by-day loop), `domain/recurrence/rrule_parser.rs:460`.

---

## 9. Error Handling Gaps

The following cases are handled by `unwrap()` or equivalent and will panic on unexpected input:

| Location | Input | Why it is considered safe |
|---|---|---|
| `astronomical.rs:39` | `date.and_hms_opt(12, 0, 0)` | Noon on a valid `NaiveDate` is always representable |
| `astronomical.rs:40` | `.and_local_timezone(Utc)` | UTC has no ambiguous or non-existent times |
| `tithi_generator.rs:204` | `NaiveDate::from_ymd_opt(...)` | Inputs come from a JD that was derived from a valid date |
| `tithi_generator.rs:205` | `NaiveTime::from_hms_opt(...)` | Hours/minutes/seconds come from integer floor operations bounded by JD arithmetic |

These are not defensive unwraps but structural invariants. If the upstream calendar data or JD arithmetic is correct, these cannot fail.
