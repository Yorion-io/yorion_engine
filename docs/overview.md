**The Nepali calendar engine that does more than convert dates.**

Accurate Bikram Sambat conversion, real panchanga (Hindu almanac) calculation, and a festival recurrence system, in one platform-agnostic Rust library.

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
| **BS ↔ AD conversion** | BS 1975–2100 (AD 1918–2044), O(1) table lookup, validated round-trips. | Many mature options, this matches them. |

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

**[Full specification → bs-rrule-spec.md](bs-rrule-spec.md)** · **[Festival reference → festivals.md](festivals.md)**

---

## Why a library, not an API

- **Offline.** Embedded apps do calendar math with no network.
- **Deterministic.** Same input, same output, every time.
- **Zero runtime cost.** Calendar data is baked in at compile time, lookups are O(1).
- **One core, many targets.** Write the logic once in Rust, ship it everywhere.

---

## Accuracy and maintenance (read this before depending on it)

Two coverage limits that do **not** move together:

| Layer | Covered through | Self-extending? |
| --- | --- | --- |
| BS ↔ AD conversion | BS 2100 | Yes, fixed table |
| Tithi (panchanga) accuracy | BS 2083 | **No, needs a yearly update** |

Conversion is a fixed, validated table out to BS 2100. Tithi values are astronomical output corrected against the official Nepali Panchanga almanac, which currently stops at **BS 2083**. Past that year, tithis are raw astronomical output with no almanac correction.

There is exactly one recurring maintenance task: when a new official patro is published, add its reference almanac, regenerate the overrides, and bump `TITHI_VERIFIED_THROUGH_BS`. Full runbook in CONTRIBUTING.md. I am calling this out up front so it is a known quantity, not a surprise.

Calendar data is derived from the [`nepali-datetime`](https://github.com/opensource-nepal/nepali_datetime) reference table and spot-checked against Hamro Patro. Every year's month lengths are checked against its anchor gap at both build time and test time.

---

## Architecture

Ports and adapters (hexagonal). `CalendarEngine` is the single public entry point, everything beneath it is an implementation detail. Full walkthrough in [architecture.md](architecture.md).
