# BS-RRULE Specification v2.0

> A platform- and language-agnostic recurrence rule format for the Bikram Sambat (BS) calendar and the Hindu panchanga (lunar tithi) cycle.
>
> Status: **Stable** · Spec version: **2.0** · Last updated: 2026-06-05

---

## 0. Changelog / Migration (v1.0 → v2.0)

**v2.0 is a breaking (MAJOR) change** to family detection and canonical serialization. The motivation:
with every occurrence now self-describing in both calendars (each carries both a BS and an AD date), the
v1.0 precedence-ladder detection (`X-TITHI` → `X-CALENDAR=BS` → AD) was implicit and fragile. v2.0 makes
**`X-CALENDAR` the single, explicit calendar-family discriminator**.

| | v1.0 | v2.0 |
|---|------|------|
| Family detection | precedence ladder: `X-TITHI` present → TITHI; else `X-CALENDAR=BS` → BS; else AD | single switch on `X-CALENDAR`: `PANCHANGA` → PANCHANGA, `BS` → BS, `AD`/absent → AD |
| `X-CALENDAR` values | `BS` only | `AD` \| `BS` \| `PANCHANGA` |
| Lunar family name | **TITHI** | **PANCHANGA** (the Hindu lunisolar almanac) |
| `X-TITHI` role | detector **and** tithi-name carrier | tithi-name carrier **only** (still REQUIRED within the PANCHANGA family) |
| Canonical order | `X-CALENDAR` emitted **last** | `X-CALENDAR` emitted **first** |
| Lunar rule serialization | `FREQ=…;…;X-TITHI=PURNIMA;X-SKIPADHIK=TRUE;X-CALENDAR=BS` | `X-CALENDAR=PANCHANGA;FREQ=…;…;X-TITHI=PURNIMA;X-SKIPADHIK=TRUE` |

**Backward compatibility:** v2.0 parsers MUST still accept legacy v1.0 lunar rules that carry `X-TITHI`
with `X-CALENDAR=BS` (or no `X-CALENDAR`) and resolve them to the **PANCHANGA** family (Rule D1, §3). No
migration of persisted RRULE strings is required. Producers always emit `X-CALENDAR=PANCHANGA`.

---

## 1. Introduction

### 1.1 Purpose

[RFC 5545](https://www.rfc-editor.org/rfc/rfc5545) defines `RRULE` for the Gregorian (AD) calendar only. It has no way to express recurrence anchored to the **Bikram Sambat** solar calendar (Nepal's official calendar) or to the **tithi** (Hindu lunar day) cycle that drives most Nepali/Hindu religious observances.

This document defines **BS-RRULE**: a strict superset of RFC 5545 `RRULE` that adds a small set of `X-` extension parameters. A BS-RRULE string is a valid RRULE-style parameter list and degrades gracefully — a consumer that does not understand the extensions can still read the standard parts.

The goal is **standardization**: any developer, in any language, on any platform, can implement a conformant parser and producer from this document alone, without reading a reference implementation.

### 1.2 Notational conventions

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

- **AD** — Anno Domini / Gregorian calendar.
- **BS** — Bikram Sambat calendar. BS year ≈ AD year + 56 or 57.
- **Tithi** — a lunar day; one of 30 per lunar month (15 per paksha).
- **Paksha** — a lunar fortnight: *Shukla* (waxing) or *Krishna* (waning).
- **Adhik maas** — an intercalary ("leap") lunar month inserted to realign the lunar and solar years.
- **Producer** — software that serializes a rule to a BS-RRULE string.
- **Consumer / Parser** — software that reads a BS-RRULE string.
- **Expander** — software that, given a rule and a date window, produces the concrete occurrence dates.

### 1.3 Scope

This spec defines the **string grammar, parameter semantics, family detection, and validation rules**. It does **not** mandate:

- A specific BS↔AD conversion algorithm or ephemeris (implementations supply their own; see §9).
- A transport, storage, or API shape.
- How occurrences are pushed to third-party calendars.

Two implementations are **interoperable** at the string level: a BS-RRULE produced by one MUST be parseable by another. They are **astronomically equivalent** only if they also share a conversion source (§9).

---

## 2. The character grammar

A BS-RRULE is a single line of `KEY=VALUE` pairs separated by semicolons. It reuses RFC 5545's `RECUR` syntax and adds `X-` parameters.

### 2.1 ABNF

```abnf
bs-rrule    = param *( ";" param )

param       = key "=" value
key         = 1*( ALPHA / "-" )        ; e.g. FREQ, X-TITHI
value       = 1*( %x20-3A / %x3C-7E )  ; any printable char except ";" (%x3B)

; Standard RFC 5545 parameters recognized by this spec:
;   FREQ, DTSTART, INTERVAL, COUNT, UNTIL, BYMONTH, BYMONTHDAY, BYDAY
; Extension parameters defined here:
;   X-CALENDAR, X-TITHI, X-PAKSHA, X-BYLUNARMONTH, X-SKIPADHIK, X-TAKE

freq        = "DAILY" / "WEEKLY" / "MONTHLY" / "YEARLY"
bsdate      = 4DIGIT 2DIGIT 2DIGIT     ; YYYYMMDD, interpreted as a BS date
weekday     = "SU" / "MO" / "TU" / "WE" / "TH" / "FR" / "SA"
month       = 1*2DIGIT                 ; 1..12
monthday    = 1*2DIGIT                 ; 1..32
boolean     = "TRUE" / "FALSE" / "1" / "0" / "YES" / "NO"
```

### 2.2 Lexical rules

1. Parameters are separated by `;`. A value MUST NOT contain `;`.
2. Each parameter is a `KEY=VALUE` pair split on the **first** `=`.
3. **Keys are case-insensitive.** A parser MUST uppercase keys before matching. (`x-tithi` ≡ `X-TITHI`.)
4. **Enumerated values are case-insensitive** (`FREQ`, `X-PAKSHA`, `X-CALENDAR`, `X-SKIPADHIK`, weekday codes, tithi names). A parser MUST uppercase them before matching.
5. Leading/trailing whitespace around a whole parameter, and around list items, SHOULD be trimmed by parsers.
6. Empty segments (e.g. a trailing `;`) MUST be ignored.
7. A parameter with no `=` is a syntax error and the parser MUST reject the whole string.
8. Parameter **order is not significant**. Producers SHOULD emit in the canonical order of §7; consumers MUST NOT depend on order.
9. Duplicate keys are undefined; a parser MAY take the last occurrence but SHOULD reject. Producers MUST NOT emit duplicates.

---

## 3. Calendar families

Every BS-RRULE belongs to exactly one of three **families**. The family is selected by the single
**`X-CALENDAR` discriminator**:

| `X-CALENDAR` value | Family | Meaning |
|--------------------|--------|---------|
| `PANCHANGA` | **PANCHANGA** | Recurs on lunar days (Ekadashi, Purnima, …) per the Hindu panchanga. |
| `BS` | **BS** | Recurs on fixed BS solar-calendar dates. |
| `AD` or absent | **AD** | Plain RFC 5545 Gregorian recurrence. |

> **Rule D1 (detection):** A consumer MUST select the family by switching on the (uppercased, trimmed)
> `X-CALENDAR` value: `PANCHANGA` → PANCHANGA, `BS` → BS, `AD` or absent → AD.
>
> **Legacy fallback (v1.0 compatibility):** if `X-CALENDAR` is absent or is `BS`, **and** `X-TITHI` is
> present, the rule MUST be resolved to the **PANCHANGA** family. This keeps already-stored v1.0 lunar
> rules parseable. Producers MUST NOT rely on this path — they always emit `X-CALENDAR=PANCHANGA`.

```
X-CALENDAR == PANCHANGA?                 ── yes ──▶ PANCHANGA
   │ no
X-CALENDAR == BS?  ── yes ──▶  has X-TITHI? ── yes ──▶ PANCHANGA (legacy)
   │ no                          │ no
   │                                        └────────▶ BS
X-CALENDAR absent + has X-TITHI? ── yes ──▶ PANCHANGA (legacy)
   │ no
                                          AD
```

---

## 4. Common parameters

### 4.1 FREQ (required for BS and AD families)

`FREQ = DAILY / WEEKLY / MONTHLY / YEARLY`. Case-insensitive.

- **BS** and **AD** families: `FREQ` is **REQUIRED**.
- **PANCHANGA** family: `FREQ` is **OPTIONAL and ignored**; tithi recurrence is inherently monthly-lunar. Producers SHOULD emit `FREQ=MONTHLY` for readability.

### 4.2 DTSTART (the anchor)

`DTSTART = YYYYMMDD`, an 8-digit **BS** date (for BS and PANCHANGA families).

- **REQUIRED** for BS and PANCHANGA families.
- The value is the **anchor**: the first candidate occurrence and the phase reference for `INTERVAL`.
- It MUST be exactly 8 digits. A non-8-digit value MUST be rejected.
- `YYYY`, `MM`, `DD` are parsed as base-10 integers. `MM` MUST be 1–12. `DD` MUST be valid for that BS month (BS months have 29–32 days; validation of the day against the month length is implementation-defined but RECOMMENDED).
- For the **AD** family, `DTSTART` (if present) follows RFC 5545 (it MAY carry a time/zone); this spec does not constrain it.

> **Note:** Unlike RFC 5545, the BS/PANCHANGA `DTSTART` carries **no time component**. Time-of-day is a property of the *event*, not of the recurrence rule (§8).

### 4.3 INTERVAL

`INTERVAL = positive integer`, default `1`.

- Applies to BS and AD families. For BS: every Nth day/week/month/year relative to the anchor.
- For the PANCHANGA family, `INTERVAL` is **OPTIONAL**; if omitted it is `1` (every lunar month). Implementations MAY support it as "every Nth qualifying lunar month".

### 4.4 COUNT and UNTIL

- `COUNT = positive integer` — stop after N occurrences.
- `UNTIL = YYYYMMDD` — a **BS** date (for BS/PANCHANGA families); stop after this date (inclusive).
- `COUNT` and `UNTIL` are both OPTIONAL. They SHOULD NOT both appear; if both do, a consumer MUST treat `UNTIL` and `COUNT` each as a cap and stop at whichever is reached first.
- If neither is present, the rule is **unbounded** (expanders apply their own window; §8).

### 4.5 BYMONTH

`BYMONTH = month *( "," month )` — a list of **BS solar** months (1–12) to which occurrences are restricted.

- OPTIONAL for BS and PANCHANGA families.
- Filters generated candidates: an occurrence is kept only if its BS month is in the list.

### 4.6 BYMONTHDAY (BS family)

`BYMONTHDAY = monthday *( "," monthday )` — a list of BS days-of-month (1–32).

- OPTIONAL, BS family only.
- `32` is permitted as a sentinel meaning "last day of the month"; expanders MUST clamp it to the actual month length.

### 4.7 BYDAY (BS / AD family)

`BYDAY = weekday *( "," weekday )` where weekday ∈ `SU MO TU WE TH FR SA`.

- OPTIONAL.
- Weekday numbering, where an implementation uses integers: `SU=0, MO=1, TU=2, WE=3, TH=4, FR=5, SA=6`.
- Note: this spec's BS `BYDAY` does **not** support the ordinal prefix form (`2MO`, `-1FR`) of RFC 5545. Only bare weekday codes are defined. Producers MUST NOT emit ordinal prefixes; consumers MAY reject them.

---

## 5. BS family parameters

The BS family expresses recurrence on the **solar** Bikram Sambat calendar — "the 1st of every BS month", "1 Baishakh every year", etc.

**Required:** `FREQ`, `DTSTART`, `X-CALENDAR=BS`.
**Optional:** `INTERVAL`, `COUNT`, `UNTIL`, `BYMONTH`, `BYMONTHDAY`, `BYDAY`.

### 5.1 X-CALENDAR

`X-CALENDAR = "AD" / "BS" / "PANCHANGA"` (case-insensitive). This is the **single calendar-family
discriminator** (§3):

- `X-CALENDAR=BS` selects the **BS** solar family.
- `X-CALENDAR=PANCHANGA` selects the **PANCHANGA** lunar family (§6).
- `X-CALENDAR=AD`, or the parameter being absent, selects the **AD** family.

Any other value is invalid; a consumer SHOULD reject it (or, lacking strictness, treat it as AD). For
backward compatibility, an absent or `BS`-valued `X-CALENDAR` combined with a present `X-TITHI` resolves
to PANCHANGA (Rule D1 legacy fallback, §3).

### 5.2 BS expansion semantics

Given the anchor and `FREQ`/`INTERVAL`, candidate BS dates are generated on the BS calendar, then filtered by any `BYMONTH`/`BYMONTHDAY`/`BYDAY`, then truncated by `COUNT`/`UNTIL`. Each surviving BS date is converted to AD for display/sync (§9).

---

## 6. PANCHANGA family parameters

The PANCHANGA family expresses recurrence on the **lunar** day cycle — "every Ekadashi", "every Purnima", "Shukla Chaturdashi in Kartik". It is selected by `X-CALENDAR=PANCHANGA` (§3). Within the family, `X-TITHI` is **REQUIRED** — it carries *which* tithi names recur, but it is no longer the family *detector*.

**Required:** `X-CALENDAR=PANCHANGA`, `X-TITHI`, `DTSTART`.
**Optional:** `X-PAKSHA`, `BYMONTH`, `X-BYLUNARMONTH`, `X-SKIPADHIK`, `X-TAKE`, `COUNT`, `UNTIL`. (`FREQ` ignored — see §4.1.)

### 6.1 X-TITHI

`X-TITHI = tithi-name *( "," tithi-name )`.

A non-empty, comma-separated list of tithi names. An occurrence is generated on each day matching **any** listed tithi. Names are case-insensitive. See §6.5 for the full accepted-name table.

If any name in the list is unrecognized, the parser MUST reject the whole rule.

### 6.2 X-PAKSHA

`X-PAKSHA = "SHUKLA" / "KRISHNA"` (case-insensitive). OPTIONAL.

Restricts matches to the waxing (Shukla) or waning (Krishna) fortnight. Interaction with `X-TITHI`:

- A **bare** tithi name (e.g. `EKADASHI`) is paksha-neutral. Combined with `X-PAKSHA=KRISHNA` it matches only Krishna Ekadashi.
- A **paksha-qualified** name (e.g. `KRISHNA EKADASHI`) already fixes the paksha. If `X-PAKSHA` is also present and **conflicts**, the qualified name's paksha takes precedence; producers SHOULD NOT emit a conflicting combination.
- `PURNIMA` is intrinsically Shukla day 15; `AMAVASYA` is intrinsically Krishna day 15. `X-PAKSHA` SHOULD be omitted with these and MUST NOT change them.

### 6.3 X-BYLUNARMONTH

`X-BYLUNARMONTH = month *( "," month )` — a list of **lunar** months (1–12) to which occurrences are restricted.

OPTIONAL, PANCHANGA family only. Distinct from `BYMONTH`, which filters by **solar** BS month. Both MAY be present and are ANDed.

### 6.4 X-SKIPADHIK

`X-SKIPADHIK = boolean` — whether to skip occurrences that fall in an **adhik maas** (intercalary lunar month).

- OPTIONAL. **Default is `TRUE`** when the parameter is absent.
- Accepted truthy values (case-insensitive): `TRUE`, `1`, `YES`. Any other value (including `FALSE`, `0`, `NO`, or an unrecognized token) is treated as **false**.
- `TRUE`: occurrences in an adhik maas are omitted (the common religious convention).
- `FALSE`: occurrences in an adhik maas are kept.

> **Rule T1 (skipadhik default):** A consumer MUST default `skip_adhik = TRUE` when `X-SKIPADHIK` is absent. This is a deliberate divergence from typical boolean defaults and is required for interoperability.

### 6.5 X-TAKE

`X-TAKE = "FIRST"` (case-insensitive). OPTIONAL, PANCHANGA family only.

When present with the value `FIRST`, within each BS **yearly** cycle the expander keeps only the **first** qualifying occurrence and discards all later occurrences in the same BS year.

**Use case — Bijaya Dashami (Dashain).** The festival always falls on the first Shukla Dashami on or after the Tula Sankranti (sun entering Libra, solar month Ashwin). In most years this is in solar Ashwin (BS month 6); in leap-month years it spills into Kartik (BS month 7). The rule `BYMONTH=6,7;X-TAKE=FIRST` covers both cases:

```
X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20730101;X-TITHI=SHUKLADASHAMI;X-PAKSHA=SHUKLA;BYMONTH=6,7;X-SKIPADHIK=TRUE;X-TAKE=FIRST
```

Without `X-TAKE=FIRST` the `BYMONTH=6,7` range would emit two occurrences in years when both months contain a Shukla Dashami. `X-TAKE=FIRST` retains only the earlier one, which is always the astronomically correct Bijaya Dashami.

**Only `FIRST` is defined.** Any other value MUST be rejected with code V11 (§11).

### 6.6 Accepted tithi names

Names are matched **case-insensitively** after trimming. Three forms are accepted; a conformant parser MUST accept all three.

**(a) Paksha-qualified, spaced or unspaced** — resolves to a specific tithi:

| Day | Shukla form(s) | Krishna form(s) |
|-----|----------------|------------------|
| 1 | `Shukla Pratipada` / `ShuklaPratipada` | `Krishna Pratipada` / `KrishnaPratipada` |
| 2 | `Shukla Dwitiya` / `ShuklaDwitiya` | `Krishna Dwitiya` / `KrishnaDwitiya` |
| 3 | `Shukla Tritiya` | `Krishna Tritiya` |
| 4 | `Shukla Chaturthi` | `Krishna Chaturthi` |
| 5 | `Shukla Panchami` | `Krishna Panchami` |
| 6 | `Shukla Shashti` | `Krishna Shashti` |
| 7 | `Shukla Saptami` | `Krishna Saptami` |
| 8 | `Shukla Ashtami` | `Krishna Ashtami` |
| 9 | `Shukla Navami` | `Krishna Navami` |
| 10 | `Shukla Dashami` | `Krishna Dashami` |
| 11 | `Shukla Ekadashi` | `Krishna Ekadashi` |
| 12 | `Shukla Dwadashi` | `Krishna Dwadashi` |
| 13 | `Shukla Trayodashi` | `Krishna Trayodashi` |
| 14 | `Shukla Chaturdashi` | `Krishna Chaturdashi` |

*(For each spaced form, the unspaced concatenation — e.g. `ShuklaEkadashi` — is also accepted.)*

**(b) Bare day-names** — paksha-neutral; combine with `X-PAKSHA` to pin the fortnight:

`Pratipada`, `Dwitiya` (alias `Dvitiya`), `Tritiya`, `Chaturthi`, `Panchami`, `Shashti` (alias `Shashthi`), `Saptami`, `Ashtami`, `Navami`, `Dashami`, `Ekadashi`, `Dwadashi`, `Trayodashi`, `Chaturdashi`.

> A bare name with no `X-PAKSHA` is, by convention, resolved to its **Shukla** variant for the purpose of obtaining a concrete enum value, but expanders SHOULD treat a bare name as matching **both** pakshas unless `X-PAKSHA` narrows it. Producers that want "both fortnights" SHOULD emit the bare name without `X-PAKSHA`.

**(c) Special full-moon / new-moon names** — intrinsically fixed:

| Name | Aliases (substring match) | Resolves to |
|------|---------------------------|-------------|
| `Purnima` | any value containing `PURNIMA` | Shukla day 15 (full moon) |
| `Amavasya` | any value containing `AMAVASYA` | Krishna day 15 (new moon) |

---

## 7. Canonical serialization

When **producing** a BS-RRULE, implementations SHOULD emit parameters in this order so that output is stable and diff-friendly. Parsers MUST NOT require it.

The `X-CALENDAR` discriminator is emitted **first** so the family is unambiguous from the leading token.

**BS family:**
```
X-CALENDAR=BS ; FREQ ; DTSTART ; [INTERVAL] ; [COUNT] ; [UNTIL] ; [BYMONTH] ; [BYMONTHDAY] ; [BYDAY]
```

**PANCHANGA family:**
```
X-CALENDAR=PANCHANGA ; FREQ=MONTHLY ; DTSTART ; X-TITHI ; [X-PAKSHA] ; [COUNT] ; [UNTIL] ; [BYMONTH] ; [X-BYLUNARMONTH] ; X-SKIPADHIK ; [X-TAKE]
```

Producer conventions:

- `X-CALENDAR` SHOULD be emitted **first** (`X-CALENDAR=BS` / `X-CALENDAR=PANCHANGA`).
- `INTERVAL` SHOULD be omitted when it equals `1`.
- Tithi names SHOULD be emitted in `UPPERCASE`.
- `X-SKIPADHIK` SHOULD always be emitted explicitly (`TRUE`/`FALSE`) by panchanga producers, even though it defaults to `TRUE`, to avoid ambiguity.

---

## 8. Relationship to event time

A BS-RRULE describes **which dates** an event recurs on. It deliberately carries **no time-of-day, duration, or timezone**. Those are properties of the event the rule is attached to.

Recommended event-level fields (informative, not part of the rule string):

| Field | Meaning |
|-------|---------|
| `isAllDay` | whether the event has a time-of-day |
| `startTime` | `"HH:MM"` local clock time |
| `durationMinutes` | length of the event |
| `timezone` | IANA zone (default `Asia/Kathmandu`, UTC+05:45) |

When converting an expanded BS/AD date to an absolute instant, the local clock time is interpreted in the event's timezone. For the default Nepal zone this is **UTC+05:45**.

Rules are **unbounded by default**; an expander MUST be given (or assume) a finite window and MUST NOT attempt to materialize an infinite series. A rolling forward window (e.g. N months ahead of "today") is RECOMMENDED.

---

## 9. Conversion source and reproducibility

Two parsers will agree on the **meaning** of a BS-RRULE, but the **actual dates** depend on:

1. The **BS↔AD conversion table** (BS month lengths vary year to year and are published, not computable by a simple formula).
2. The **tithi/paksha ephemeris** (astronomical model + reference location/timezone for sunrise-based tithi assignment).

Therefore, for reproducible results, an implementation MUST document:

- The BS calendar data range it supports (e.g. BS 2000–2090).
- The conversion data source/version.
- The reference location and timezone used for tithi computation (commonly Kathmandu, 27.7172°N 85.3240°E, UTC+05:45).

Two implementations are **astronomically interoperable** only when these agree. The string format alone guarantees **syntactic** interoperability.

---

## 10. Relationship to RFC 7529 (RSCALE) — informative

[RFC 7529](https://www.rfc-editor.org/rfc/rfc7529.html) ("Non-Gregorian Recurrence Rules") is the only existing IETF standard for calendar-aware recurrence. This section explains why BS-RRULE does **not** build on it. It is **informative**, not normative — BS-RRULE borrows no parameters from RFC 7529.

### 10.1 Why RFC 7529 does not fit

RFC 7529 adds an `RSCALE=<calendar>` part whose value is a Unicode CLDR calendar identifier (e.g. `RSCALE=HEBREW`, `RSCALE=CHINESE`), keeps `DTSTART` Gregorian, and applies the calendar internally. It cannot serve either of our needs:

1. **No Bikram Sambat identifier.** CLDR's `indian` calendar is the **Saka** national calendar, not Vikram/Bikram Sambat. There is no CLDR identifier for BS, so `RSCALE` has no value that names our solar calendar.
2. **No tithi model.** RSCALE performs year/month/day arithmetic on whole calendars. A tithi is an astronomical lunar-day whose Gregorian date is location/sunrise-dependent and can skip or repeat. No `FREQ`/`BY*` combination expresses "every Ekadashi" — lunar-day recurrence is out of scope for RFC 7529 by design.

Because neither gap can be closed within RFC 7529, BS-RRULE remains a self-contained `X-`-extension of plain RFC 5545. Defining custom `X-` parameters is the standards-compliant way to carry values that no registry covers; we do not claim RFC 7529 conformance.

### 10.2 Mapping table (for readers familiar with RFC 7529)

| Concept | RFC 7529 | BS-RRULE |
|---------|----------|----------|
| Calendar selector | `RSCALE=<cldr-id>` | `X-CALENDAR=AD\|BS\|PANCHANGA` (no CLDR id exists for BS) |
| Anchor encoding | Gregorian `DTSTART` | BS-encoded `DTSTART` (BS month lengths are tabular, not formula-derived) |
| Lunar leap month | `L` suffix on `BYMONTH` (e.g. `5L`) | `X-SKIPADHIK` boolean (adhik maas; tithi rules carry no `BYMONTH` to suffix) |
| Lunar-day recurrence | not expressible | `X-TITHI` / `X-PAKSHA` / `X-BYLUNARMONTH` |
| Invalid day-of-month | `SKIP=OMIT\|BACKWARD\|FORWARD` (default OMIT) | `BYMONTHDAY=32` "last-day" sentinel (§4.6); otherwise invalid days are omitted, matching RFC 5545's implicit default |
| iCalendar `CALSCALE` | stays `GREGORIAN` | n/a — BS-RRULE standardizes the rule string only, not a VCALENDAR object |

> **On invalid dates:** the problem RFC 7529 names `SKIP` (a recurrence landing on a day that does not exist in a given month) is universal, not RFC 7529-specific. Base RFC 5545 already resolves it by silently omitting such occurrences. BS-RRULE keeps that omit-by-default behavior and offers the `BYMONTHDAY=32` sentinel for the common "last day of the month" case. BS-RRULE intentionally does **not** introduce a `SKIP` keyword.

### 10.3 Reserved tokens

`RSCALE` and `SKIP` are **reserved** identifiers in BS-RRULE v2.0: a consumer MUST NOT treat them as selecting a family or altering behavior, but SHOULD preserve them if round-tripping unknown parameters. This leaves room for a future major version to define an interoperable RFC 7529 profile should a CLDR identifier for Bikram Sambat ever be registered.

---

## 11. Validation rules (normative summary)

A conformant parser MUST reject a BS-RRULE when:

- **V1** — any parameter lacks `=` (§2.2.7).
- **V2** — family is BS or AD and `FREQ` is missing (§4.1).
- **V3** — `FREQ` value is not one of the four defined frequencies (§4.1).
- **V4** — family is BS or PANCHANGA and `DTSTART` is missing (§4.2).
- **V5** — `DTSTART` or `UNTIL` is not exactly 8 digits, or its month is outside 1–12 (§4.2).
- **V6** — `INTERVAL` or `COUNT` is present but not a positive integer (§4.3–4.4).
- **V7** — `BYMONTH` / `X-BYLUNARMONTH` contains a value outside 1–12 (§4.5, §6.3).
- **V8** — `BYDAY` contains a token that is not a defined weekday code (§4.7).
- **V9** — family is PANCHANGA and `X-TITHI` is missing, empty, or contains an unrecognized name (§6.1, §6.5).
- **V10** — `X-PAKSHA` is present with a value other than `SHUKLA`/`KRISHNA` (§6.2).
- **V11** — `X-TAKE` is present with a value other than `FIRST` (§6.5).

A parser SHOULD warn (but MAY accept) when:

- **W1** — both `COUNT` and `UNTIL` are present (§4.4).
- **W2** — `X-PAKSHA` conflicts with a paksha-qualified `X-TITHI` name (§6.2).
- **W3** — `X-SKIPADHIK` has an unrecognized value (treated as false; §6.4).

---

## 12. Worked examples

All examples assume an anchor in BS year 2081.

### 12.1 BS family

| Intent | BS-RRULE |
|--------|----------|
| 1 Baishakh, every BS year | `X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20810101` |
| The 5th of every BS month | `X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20810105;BYMONTHDAY=5` |
| Every 3rd BS month from anchor | `X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20810101;INTERVAL=3` |
| Last day of each BS month | `X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20810101;BYMONTHDAY=32` |
| 1 Baishakh, next 5 years only | `X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20810101;COUNT=5` |
| Every Saturday (BS-anchored) | `X-CALENDAR=BS;FREQ=WEEKLY;DTSTART=20810101;BYDAY=SA` |

### 12.2 PANCHANGA family

| Intent | BS-RRULE |
|--------|----------|
| Every Ekadashi (both pakshas) | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE` |
| Shukla Ekadashi only | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-PAKSHA=SHUKLA;X-SKIPADHIK=TRUE` |
| Every Purnima | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=PURNIMA;X-SKIPADHIK=TRUE` |
| Every Amavasya | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=AMAVASYA;X-SKIPADHIK=TRUE` |
| Krishna Chaturdashi in Kartik (lunar m.7) | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=CHATURDASHI;X-PAKSHA=KRISHNA;X-BYLUNARMONTH=7;X-SKIPADHIK=TRUE` |
| Ekadashi incl. adhik maas | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=FALSE` |
| Either Ashtami or Navami | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=ASHTAMI,NAVAMI;X-SKIPADHIK=TRUE` |
| Bijaya Dashami (Dashain) — first Shukla Dashami on/after Tula Sankranti | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20730101;X-TITHI=SHUKLADASHAMI;X-PAKSHA=SHUKLA;BYMONTH=6,7;X-SKIPADHIK=TRUE;X-TAKE=FIRST` |

### 12.3 AD family (RFC 5545 passthrough)

| Intent | BS-RRULE |
|--------|----------|
| Every Mon/Wed/Fri | `FREQ=WEEKLY;BYDAY=MO,WE,FR` |
| Monthly on the 15th (Gregorian) | `FREQ=MONTHLY;BYMONTHDAY=15` |

---

## 13. Reference test vectors

A conformant implementation SHOULD pass these. `→` denotes the detected family; `✗` denotes a rejection.

> A machine-readable version of these vectors (plus more) ships alongside this spec as [`bs-rrule-test-vectors.json`](./bs-rrule-test-vectors.json). Use it as a portable conformance fixture in any language.

| # | Input | Expected |
|---|-------|----------|
| 1 | `X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20810101` | → BS |
| 2 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=PURNIMA` | → PANCHANGA |
| 3 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI` | → PANCHANGA |
| 4 | `FREQ=WEEKLY;BYDAY=MO,WE,FR` | → AD |
| 5 | `x-calendar=bs;freq=yearly;dtstart=20810101` | → BS (case-insensitive) |
| 6 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Saptami` | → PANCHANGA, tithi = Saptami |
| 7 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=1` | skip_adhik = **true** |
| 8 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-SKIPADHIK=0` | skip_adhik = **false** |
| 9 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI` | skip_adhik = **true** (default) |
| 10 | `X-CALENDAR=BS;DTSTART=20810101` | ✗ missing FREQ (BS family) |
| 11 | `X-CALENDAR=BS;FREQ=YEARLY` | ✗ missing DTSTART |
| 12 | `X-CALENDAR=BS;FREQ=FORTNIGHTLY;DTSTART=20810101` | ✗ invalid FREQ |
| 13 | `X-CALENDAR=BS;FREQ=YEARLY;DTSTART=2081011` | ✗ DTSTART not 8 digits |
| 14 | `X-CALENDAR=BS;FREQ=YEARLY;DTSTART=20811301` | ✗ month 13 out of range |
| 15 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Bogus` | ✗ unknown tithi |
| 16 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-PAKSHA=WANING` | ✗ invalid paksha |
| 17 | `FREQ=MONTHLY;DTSTART=20810101;X-TITHI=PURNIMA;X-CALENDAR=BS` | → PANCHANGA (legacy v1.0: X-TITHI + X-CALENDAR=BS) |
| 18 | `FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI` | → PANCHANGA (legacy v1.0: X-TITHI, no X-CALENDAR) |
| 19 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20730101;X-TITHI=SHUKLADASHAMI;X-PAKSHA=SHUKLA;BYMONTH=6,7;X-SKIPADHIK=TRUE;X-TAKE=FIRST` | → PANCHANGA, take_first = **true** |
| 20 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-TAKE=LAST` | ✗ V11 — invalid X-TAKE value |
| 21 | `X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-TAKE=first` | → PANCHANGA, take_first = **true** (case-insensitive) |

---

## 14. Versioning

- This is **BS-RRULE v2.0**.
- **v2.0 is a MAJOR bump from v1.0** (§0). It changes family detection from a precedence ladder to a single `X-CALENDAR` switch, adds the `PANCHANGA` and `AD` values to `X-CALENDAR`, renames the lunar family `TITHI` → `PANCHANGA`, and moves `X-CALENDAR` to the front of the canonical order. These are all breaking changes to detection/serialization, hence the major version.
- **Compatibility:** v2.0 parsers MUST accept legacy v1.0 lunar rules (`X-TITHI` with `X-CALENDAR=BS` or absent) and resolve them to PANCHANGA (§3 Rule D1). No migration of persisted RRULE strings is required.
- New optional `X-` parameters MAY be added in minor versions without breaking parsers (unknown `X-` params SHOULD be ignored by consumers, except where they trigger family detection).
- Changes to detection, required parameters, or default values constitute a **major** version bump.
- A future `X-SPEC-VERSION` parameter MAY be introduced for explicit negotiation; v2.0 consumers MUST ignore it if present.

---

## 15. Implementation checklist

To build a conformant implementation from scratch:

- [ ] **Tokenizer** — split on `;`, then first `=`; uppercase keys; trim; ignore empty segments; reject params without `=`.
- [ ] **Family detector** — switch on `X-CALENDAR` (§3): PANCHANGA → PANCHANGA, BS → BS, AD/absent → AD; with the legacy fallback (X-TITHI + X-CALENDAR=BS/absent → PANCHANGA).
- [ ] **BS parser** — require FREQ + DTSTART; parse optional INTERVAL/COUNT/UNTIL/BYMONTH/BYMONTHDAY/BYDAY.
- [ ] **PANCHANGA parser** — require X-TITHI + DTSTART; parse X-PAKSHA/BYMONTH/X-BYLUNARMONTH/X-SKIPADHIK (default TRUE)/X-TAKE (FIRST only; reject others with V11).
- [ ] **Tithi name table** — accept all three name forms (§6.5), case-insensitive.
- [ ] **Validator** — enforce V1–V11 (§11).
- [ ] **Producer** — emit canonical order (§7) with X-CALENDAR first; omit INTERVAL=1; uppercase tithi names; emit X-CALENDAR=BS for BS and X-CALENDAR=PANCHANGA for PANCHANGA.
- [ ] **Conversion source** — document BS data range, source/version, and tithi reference location (§9).
- [ ] **Test vectors** — pass §13 / `bs-rrule-test-vectors.json`.
- [ ] **Round-trip** — `parse(produce(rule)) == rule` for every supported rule.
```
