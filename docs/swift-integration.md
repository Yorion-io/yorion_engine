# Swift Integration Guide

Integrate YorionEngine into your macOS or iOS Swift app using the pre-built XCFramework and auto-generated Swift bindings.

## Installation

1. Download `apple-assets-{version}.tar.gz` from the [latest release](https://github.com/Yorion-io/yorion_engine/releases/latest).
2. Extract and copy `BsCalendarCore.xcframework` into your project's `Frameworks/` folder.
3. Copy `yorion_engine.swift` alongside it.
4. In Xcode → target → **General → Frameworks, Libraries, and Embedded Content**: add `BsCalendarCore.xcframework`, set to **Do Not Embed**.
5. Set your bridging header:

```c
#import "Frameworks/BsCalendarCore.xcframework/macos-arm64_x86_64/Headers/yorion_engineFFI.h"
```

### Build from source

```bash
git clone https://github.com/Yorion-io/yorion_engine
cd yorion_engine
./scripts/build-macos-arm64.sh
./scripts/build-macos-x86.sh
./scripts/generate-swift-bindings.sh
./scripts/package-macos.sh
# → dist/BsCalendarCore.xcframework + dist/swift/yorion_engine.swift
```

---

## Usage

### Date conversion

```swift
import YorionCore

let engine = createEngine()

let gregorian = GregorianDate(year: 2024, month: 4, day: 13)
let bs = try? engine.gregorianToBs(date: gregorian)
// bs.year = 2081, bs.month = 1, bs.day = 1

let bsDate = BsDate(year: 2081, month: 1, day: 1)
let ad = try? engine.bsToGregorian(date: bsDate)
// ad.year = 2024, ad.month = 4, ad.day = 13
```

### Panchanga (daily almanac)

```swift
let location = Location(latitude: 27.7172, longitude: 85.3240, name: "Kathmandu", timezoneOffsetMins: 345)

if let info = try? engine.getDailyAstroInfo(date: gregorian, location: location) {
    print("Tithi: \(info.tithi)")
    print("Nakshatra: \(info.nakshatra)")
    print("Sun sign: \(info.sunSign)")
}

let sunrise = try? engine.getSunrise(date: gregorian, location: location)
let sunset  = try? engine.getSunset(date: gregorian, location: location)
```

### Month calendar

```swift
let month = try? engine.getMonthCalendar(year: 2081, month: 1, location: location)
// month.days — array of day info including bs/ad dates, tithi, nakshatra
```

### Recurring events

```swift
// Every Ekadashi
let anchor = BsDate(year: 2081, month: 1, day: 1)
let rule = TithiRecurrenceRule(
    tithis: [11],
    paksha: nil,
    anchor: anchor,
    count: 24,
    until: nil,
    byMonth: nil,
    byLunarMonth: nil,
    skipAdhik: true,
    takeFirst: false
)

let instances = try? engine.generateTithiInstances(
    eventId: "ekadashi",
    title: "Ekadashi",
    rule: rule,
    start: anchor,
    end: BsDate(year: 2083, month: 12, day: 30),
    version: CalendarVersion(version: "v1", isOfficial: true),
    location: location
)
```

### Localization

```swift
let name   = engine.getTithiName(tithi: tithi, lang: .nepali)    // नवमी
let zodiac = engine.getZodiacName(zodiac: .aries, lang: .english)
```

---

## Error handling

```swift
do {
    let bs = try engine.gregorianToBs(date: gregorian)
} catch let error as BsCalendarError {
    print("Error: \(error)")
}
```

---

## Platform support

- macOS 12.0+ (arm64 + x86_64)
- iOS 15.0+ (planned — see [Planned binary targets](../README.md#planned-binary-targets))

---

## Troubleshooting

| Issue | Fix |
|---|---|
| `Module not found` | Add XCFramework to target's Frameworks section |
| Linker errors | Confirm building for correct arch (x86_64 / arm64) |
| `uniffi-bindgen not found` | `cargo install uniffi-bindgen` |
| Build fails on feature flags | `--features uniffi-bindings --no-default-features` |
