# Recurring Event Generation Guide

This guide provides a comprehensive overview of how `BsCalendarCore` handles recurring events using RRULE (RFC 5545) format with custom extensions for Bikram Sambat calendar and Tithi support.

---

## RRULE Format Overview

All recurrence rules in BsCalendarCore use the RRULE (Recurrence Rule) format from RFC 5545, with custom extensions for BS calendar and Tithi support.

### Standard RRULE Parameters

- `FREQ` - Frequency: `DAILY`, `WEEKLY`, `MONTHLY`, `YEARLY`
- `INTERVAL` - Repeat every N units (default: 1)
- `COUNT` - Maximum number of occurrences
- `UNTIL` - End date (format: YYYYMMDD)
- `DTSTART` - Start date (format: YYYYMMDD)
- `BYMONTH` - Filter by specific months (comma-separated)
- `BYMONTHDAY` - Filter by specific days of month (comma-separated)

### Custom BS Extensions

- `X-CALENDAR=BS` - Indicates BS calendar mode
- `X-TITHI=<name>` - Tithi name for lunar recurrence
- `X-PAKSHA=SHUKLA|KRISHNA` - Paksha (lunar fortnight) filter
- `X-BYLUNARMONTH=<months>` - Lunar month filter
- `X-SKIPADHIK=TRUE|FALSE` - Skip adhik (extra) months

---

## 1. BS (Bikram Sambat) Recurrence Rules

Solar recurrence operates on fixed dates within the Bikram Sambat calendar. Ideal for birthdays, anniversaries, and fixed holidays.

### Basic Examples

#### Daily Recurrence
```
FREQ=DAILY;DTSTART=20800101;COUNT=30;X-CALENDAR=BS
```
Every day starting from BS 2080/01/01 for 30 occurrences.

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Daily,
    BsDate::new(2080, 1, 1).unwrap()
).with_count(30);
```

#### Weekly Recurrence
```
FREQ=WEEKLY;DTSTART=20800101;INTERVAL=2;X-CALENDAR=BS
```
Every 2 weeks starting from BS 2080/01/01.

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Weekly,
    BsDate::new(2080, 1, 1).unwrap()
).with_interval(2);
```

#### Monthly Recurrence
```
FREQ=MONTHLY;DTSTART=20800115;UNTIL=20810115;X-CALENDAR=BS
```
Every month on the 15th, from BS 2080/01/15 until BS 2081/01/15.

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Monthly,
    BsDate::new(2080, 1, 15).unwrap()
).with_until(BsDate::new(2081, 1, 15).unwrap());
```

#### Yearly Recurrence
```
FREQ=YEARLY;DTSTART=20800101;X-CALENDAR=BS
```
Every year on Baisakh 1 (Nepali New Year).

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Yearly,
    BsDate::new(2080, 1, 1).unwrap()
);
```

### Advanced Filtering Examples

#### Specific Months
```
FREQ=MONTHLY;DTSTART=20800101;BYMONTH=1,5,9;BYMONTHDAY=1,15;X-CALENDAR=BS
```
On the 1st and 15th of Baisakh (1), Shrawan (5), and Kartik (9).

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Monthly,
    BsDate::new(2080, 1, 1).unwrap()
)
.with_by_month(vec![BsMonth::Baisakh, BsMonth::Shrawan, BsMonth::Kartik])
.with_by_month_day(vec![1, 15]);
```

#### Quarterly Events
```
FREQ=MONTHLY;DTSTART=20800101;INTERVAL=3;BYMONTHDAY=1;X-CALENDAR=BS
```
Every 3 months on the 1st day.

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Monthly,
    BsDate::new(2080, 1, 1).unwrap()
)
.with_interval(3)
.with_by_month_day(vec![1]);
```

#### Last Day of Month
```
FREQ=MONTHLY;DTSTART=20800132;X-CALENDAR=BS
```
Last day of every month (day 32 gets clamped to actual month length).

```rust
let rule = BsRecurrenceRule::new(
    BsFrequency::Monthly,
    BsDate::new(2080, 1, 32).unwrap()
);
```

### The "Clamping" Logic

Bikram Sambat months vary in length from year to year (e.g., Baisakh can be 30 or 31 days).

- **Rule**: If an event is scheduled for the 31st but the current month only has 30 days, the engine **clamps** the event to the 30th.
- **Example**: A monthly event on day 32 will always fall on the last day of every month (30, 31, or 32 depending on the month).

---

## 2. AD (Gregorian) Recurrence Rules

Standard Gregorian calendar recurrence using RFC 5545 RRULE format.

### Examples

#### Weekly Meeting
```
FREQ=WEEKLY;DTSTART=20240115;INTERVAL=1;COUNT=52
```
Every week for 52 occurrences starting January 15, 2024.

```rust
let rule = AdRecurrenceRule::new(
    AdFrequency::Weekly,
    NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
).with_count(52);
```

#### Bi-Weekly Payroll
```
FREQ=WEEKLY;DTSTART=20240101;INTERVAL=2
```
Every 2 weeks starting January 1, 2024.

#### Monthly on Specific Day
```
FREQ=MONTHLY;DTSTART=20240115;BYMONTHDAY=15
```
15th of every month.

#### Quarterly Review
```
FREQ=MONTHLY;DTSTART=20240101;INTERVAL=3;BYMONTH=1,4,7,10;BYMONTHDAY=1
```
First day of January, April, July, and October.

---

## 3. Tithi (Lunar) Recurrence Rules

Tithi recurrence is based on the angle between the Sun and the Moon. These events shift in the solar calendar every year.

### The "Udaya Tithi" Principle

In the Hindu/Nepali calendar, a day's Tithi is determined by whatever Tithi is active at the moment of **Sunrise**.
- Even if a Tithi ends at 10:00 AM, the entire day is considered to belong to the Tithi that was present at sunrise.

### Basic Tithi Examples

#### Every Ekadashi (Both Pakshas)
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Every Ekadashi (11th lunar day) in both Shukla and Krishna Paksha.

```rust
let rule = TithiRecurrenceRule::ekadashi(
    BsDate::new(2080, 1, 1).unwrap()
);
```

#### Shukla Ekadashi Only
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=SHUKLA EKADASHI;X-PAKSHA=SHUKLA;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Only Shukla Paksha Ekadashi (waxing moon).

```rust
let rule = TithiRecurrenceRule::with_paksha(
    Tithi::ShuklaEkadashi,
    Paksha::Shukla,
    BsDate::new(2080, 1, 1).unwrap()
);
```

#### Every Purnima (Full Moon)
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=PURNIMA;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Every full moon.

```rust
let rule = TithiRecurrenceRule::purnima(
    BsDate::new(2080, 1, 1).unwrap()
);
```

#### Every Amavasya (New Moon)
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=AMAVASYA;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Every new moon.

```rust
let rule = TithiRecurrenceRule::amavasya(
    BsDate::new(2080, 1, 1).unwrap()
);
```

### Advanced Tithi Examples

#### Dashain (Specific Lunar Month)
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=SHUKLA DASHAMI;X-PAKSHA=SHUKLA;X-BYLUNARMONTH=6;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Shukla Dashami in Ashwin lunar month only (Dashain).

```rust
let rule = TithiRecurrenceRule::with_paksha(
    Tithi::ShuklaDashami,
    Paksha::Shukla,
    BsDate::new(2080, 1, 1).unwrap()
)
.with_by_lunar_month(vec![BsMonth::Ashwin])
.with_skip_adhik(true);
```

#### Ekadashi in Specific Solar Months
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;BYMONTH=10,11,12;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Every Ekadashi occurring in Magh, Falgun, or Chaitra solar months.

```rust
let rule = TithiRecurrenceRule::ekadashi(
    BsDate::new(2080, 1, 1).unwrap()
)
.with_by_month(vec![BsMonth::Magh, BsMonth::Falgun, BsMonth::Chaitra]);
```

#### Limited Occurrences
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=PURNIMA;COUNT=12;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Next 12 Purnimas only.

```rust
let rule = TithiRecurrenceRule::purnima(
    BsDate::new(2080, 1, 1).unwrap()
).with_count(12);
```

#### Krishna Ashtami (Janmashtami)
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=KRISHNA ASHTAMI;X-PAKSHA=KRISHNA;X-BYLUNARMONTH=4;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Krishna Ashtami in Shrawan lunar month (Janmashtami).

```rust
let rule = TithiRecurrenceRule::with_paksha(
    Tithi::KrishnaAshtami,
    Paksha::Krishna,
    BsDate::new(2080, 1, 1).unwrap()
)
.with_by_lunar_month(vec![BsMonth::Shrawan]);
```

### Filtering & Deduping

Because lunar months don't align with solar months, a specific Tithi (like *Shukla Dashami*) can occasionally appear **twice** or **zero times** in a single BS solar month.

| Filter | Purpose | Example |
| :--- | :--- | :--- |
| `BYMONTH` | Limits search to specific **Solar** months | "Every Ekadashi in Magh" |
| `X-BYLUNARMONTH` | Limits search to specific **Astronomical** months | "Dashain" (Must be Ashwin Lunar Month) |
| `X-SKIPADHIK` | Skips occurrences in "Extra" (Intercalary) months | Ensuring festivals aren't celebrated twice in leap years |

---

## 4. Serialization Examples

All recurrence rules serialize to RRULE strings for storage and transmission.

### JSON Serialization

```rust
use serde_json;

// Create a rule
let rule = BsRecurrenceRule::new(
    BsFrequency::Monthly,
    BsDate::new(2080, 1, 15).unwrap()
).with_count(12);

// Serialize to JSON (stores as RRULE string)
let json = serde_json::to_string(&rule).unwrap();
// Output: "FREQ=MONTHLY;DTSTART=20800115;COUNT=12;X-CALENDAR=BS"

// Deserialize from JSON
let parsed: BsRecurrenceRule = serde_json::from_str(&json).unwrap();
assert_eq!(parsed, rule);
```

### Direct RRULE String Usage

```rust
// Parse from RRULE string
let rrule = "FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;X-CALENDAR=BS";
let rule = TithiRecurrenceRule::from_rrule(rrule).unwrap();

// Convert to RRULE string
let rrule_string = rule.to_rrule();
```

---

## 5. Location-Aware Tithi Generation

Tithis are calculated at the moment of local sunrise. Because sunrise happens at different times (UTC) globally:

### Example Scenarios

```rust
// 1. Official Nepal (Social Calendar)
let loc = Location::KATHMANDU; // follow_nepal_social_calendar: true

// 2. Pure Science (USA)
let loc = Location::new(40.7, -74.0, "NY", -300); // follow_nepal_social_calendar: false

// 3. Diaspora Ritual (USA coordinates, Nepal rules)
let loc = Location::with_social_calendar(40.7, -74.0, "NY", -300, true);
```

### Generating Instances

```rust
use bs_calendar_core::services::InstanceGenerator;

let generator = InstanceGenerator::new(conversion_service);

// BS recurrence
let bs_rule = BsRecurrenceRule::from_rrule(
    "FREQ=MONTHLY;DTSTART=20800101;BYMONTHDAY=1;X-CALENDAR=BS"
).unwrap();

let instances = generator.generate_bs_instances(
    &bs_rule,
    BsDate::new(2080, 1, 1).unwrap(),
    BsDate::new(2081, 1, 1).unwrap()
)?;

// Tithi recurrence
let tithi_rule = TithiRecurrenceRule::from_rrule(
    "FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;X-CALENDAR=BS"
).unwrap();

let instances = generator.generate_tithi_instances(
    &tithi_rule,
    BsDate::new(2080, 1, 1).unwrap(),
    BsDate::new(2081, 1, 1).unwrap(),
    &astro_service
)?;
```

---

## 6. Common Use Cases

### Birthday (Annual)
```
FREQ=YEARLY;DTSTART=19900515;X-CALENDAR=BS
```
Every year on Jestha 15 (birthday).

### Salary Payment (Bi-Monthly)
```
FREQ=MONTHLY;DTSTART=20240101;INTERVAL=2;BYMONTHDAY=1
```
1st day of every other month.

### Weekly Team Meeting
```
FREQ=WEEKLY;DTSTART=20240115;INTERVAL=1
```
Every week starting January 15, 2024.

### Quarterly Business Review
```
FREQ=MONTHLY;DTSTART=20240101;INTERVAL=3;BYMONTHDAY=15
```
15th day every 3 months.

### Ekadashi Fasting
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Every Ekadashi (both pakshas).

### Teej Festival
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=KRISHNA TRITIYA;X-PAKSHA=KRISHNA;X-BYLUNARMONTH=5;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Krishna Tritiya in Bhadra lunar month.

### Tihar (Laxmi Puja)
```
FREQ=MONTHLY;DTSTART=20800101;X-TITHI=AMAVASYA;X-BYLUNARMONTH=7;X-SKIPADHIK=TRUE;X-CALENDAR=BS
```
Amavasya in Kartik lunar month.

---

## 7. Performance & Reliability

Generating instances over long ranges (e.g., 50 years) is computationally expensive. We use two key strategies:

1. **Cache the Moon**: Lunar cycle data (Amavasya timings and Adhik status) is cached for 30 days, reducing heavy math by **97%**.
2. **Solar Gates**: If a `BYMONTH` filter is used, we only perform astronomical math for the relevant ~30 days of the year.

---

## 8. RRULE Reference

### Complete Parameter List

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `FREQ` | Required | Recurrence frequency | `DAILY`, `WEEKLY`, `MONTHLY`, `YEARLY` |
| `DTSTART` | Required | Start date | `20800101` |
| `INTERVAL` | Optional | Repeat every N units | `2` (every 2 weeks) |
| `COUNT` | Optional | Max occurrences | `10` |
| `UNTIL` | Optional | End date | `20810101` |
| `BYMONTH` | Optional | Solar months (1-12) | `1,5,9` |
| `BYMONTHDAY` | Optional | Days of month (1-32) | `1,15` |
| `X-CALENDAR` | Optional | Calendar type | `BS` |
| `X-TITHI` | Optional | Tithi name | `EKADASHI`, `PURNIMA` |
| `X-PAKSHA` | Optional | Paksha filter | `SHUKLA`, `KRISHNA` |
| `X-BYLUNARMONTH` | Optional | Lunar months (1-12) | `6` (Ashwin) |
| `X-SKIPADHIK` | Optional | Skip adhik months | `TRUE`, `FALSE` |

### Tithi Names

**Shukla Paksha (Waxing Moon):**
- `SHUKLA PRATIPADA`, `SHUKLA DWITIYA`, `SHUKLA TRITIYA`, `SHUKLA CHATURTHI`, `SHUKLA PANCHAMI`
- `SHUKLA SHASHTI`, `SHUKLA SAPTAMI`, `SHUKLA ASHTAMI`, `SHUKLA NAVAMI`, `SHUKLA DASHAMI`
- `SHUKLA EKADASHI`, `SHUKLA DWADASHI`, `SHUKLA TRAYODASHI`, `SHUKLA CHATURDASHI`, `PURNIMA`

**Krishna Paksha (Waning Moon):**
- `KRISHNA PRATIPADA`, `KRISHNA DWITIYA`, `KRISHNA TRITIYA`, `KRISHNA CHATURTHI`, `KRISHNA PANCHAMI`
- `KRISHNA SHASHTI`, `KRISHNA SAPTAMI`, `KRISHNA ASHTAMI`, `KRISHNA NAVAMI`, `KRISHNA DASHAMI`
- `KRISHNA EKADASHI`, `KRISHNA DWADASHI`, `KRISHNA TRAYODASHI`, `KRISHNA CHATURDASHI`, `AMAVASYA`

**Short Forms:**
- `EKADASHI` - Defaults to Shukla Ekadashi (use `X-PAKSHA` to specify)
- `PURNIMA` - Full moon
- `AMAVASYA` - New moon
