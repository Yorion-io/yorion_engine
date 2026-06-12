# Swift Integration Guide

## Overview

This guide shows how to integrate the YorionEngine Rust library into your macOS Swift/SwiftUI application using automatically generated Swift bindings via UniFFI.

## Installation

### Option 1: Swift Package Manager (Recommended)

Add to your `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/Yorion-io/yorion_engine", from: "0.1.0")
]
```

Or in Xcode:
1. File → Add Package Dependencies
2. Enter repository URL
3. Select version/branch
4. Add to your target

### Option 2: Local XCFramework

1. Download the latest release from GitHub Releases
2. Extract `YorionCore.xcframework.zip`
3. Drag `YorionCore.xcframework` into your Xcode project
4. Ensure it's added to "Frameworks, Libraries, and Embedded Content"

### Option 3: Build from Source

```bash
git clone https://github.com/Yorion-io/yorion_engine
cd YorionCore/00_core

# Build universal binary
./scripts/build-macos.sh

# Generate Swift bindings and XCFramework
./scripts/generate-swift-bindings.sh

# The XCFramework is now at dist/YorionCore.xcframework
```

## Usage

### Basic Date Conversion

```swift
import YorionCore

// Create the calendar engine
let engine = createEngine()

// Convert Gregorian to BS
let gregorianDate = GregorianDate(year: 2024, month: 2, day: 8)
if let bsDate = try? engine.gregorianToBs(date: gregorianDate) {
    print("BS Date: \\(bsDate.year)/\\(bsDate.month)/\\(bsDate.day)")
    // Output: BS Date: 2080/10/26
}

// Convert BS to Gregorian
let bsDate = BsDate(year: 2080, month: 1, day: 1)
if let adDate = try? engine.bsToGregorian(date: bsDate) {
    print("Gregorian: \\(adDate.year)-\\(adDate.month)-\\(adDate.day)")
    // Output: Gregorian: 2023-4-14
}
```

### Astronomical Calculations

```swift
// Get Tithi (lunar day) for a date
let location = Location(latitude: 27.7172, longitude: 85.3240) // Kathmandu
let date = GregorianDate(year: 2024, month: 2, day: 8)

if let tithi = try? engine.getTithi(date: date) {
    print("Tithi: \\(tithi.name)")
    print("Paksha: \\(tithi.paksha)")
}

// Get comprehensive astronomical info
if let astroInfo = try? engine.getDailyAstroInfo(date: date, location: location) {
    print("Tithi: \\(astroInfo.tithi.name)")
    print("Sun Sign: \\(astroInfo.sunSign)")
    print("Moon Sign: \\(astroInfo.moonSign)")
    print("Nakshatra: \\(astroInfo.nakshatra)")
}

// Get sunrise/sunset times
if let sunrise = try? engine.getSunrise(date: date, location: location) {
    print("Sunrise: \\(sunrise.hour):\\(sunrise.minute)")
}

if let sunset = try? engine.getSunset(date: date, location: location) {
    print("Sunset: \\(sunset.hour):\\(sunset.minute)")
}
```

### SwiftUI Integration

```swift
import SwiftUI
import YorionCore

struct CalendarView: View {
    @State private var bsDate: BsDate?
    @State private var tithiName: String = ""
    
    private let engine = createEngine()
    private let location = Location(latitude: 27.7172, longitude: 85.3240)
    
    var body: some View {
        VStack(spacing: 20) {
            Text("BS Calendar")
                .font(.largeTitle)
                .bold()
            
            if let date = bsDate {
                VStack {
                    Text("\\(date.year)/\\(date.month)/\\(date.day)")
                        .font(.title)
                    
                    if !tithiName.isEmpty {
                        Text("Tithi: \\(tithiName)")
                            .font(.headline)
                    }
                }
            }
            
            Button("Get Today's BS Date") {
                updateDate()
            }
            .buttonStyle(.borderedProminent)
        }
        .padding()
        .onAppear {
            updateDate()
        }
    }
    
    private func updateDate() {
        let now = Date()
        let calendar = Calendar.current
        let components = calendar.dateComponents([.year, .month, .day], from: now)
        
        let gregorian = GregorianDate(
            year: Int32(components.year!),
            month: UInt32(components.month!),
            day: UInt32(components.day!)
        )
        
        // Convert to BS
        if let bs = try? engine.gregorianToBs(date: gregorian) {
            bsDate = bs
        }
        
        // Get Tithi
        if let tithi = try? engine.getTithi(date: gregorian) {
            tithiName = engine.getTithiName(tithi: tithi, lang: .english)
        }
    }
}
```

### Month Calendar View

```swift
// Get full month calendar with astronomical data
let monthData = try? engine.getMonthCalendar(
    year: 2080,
    month: 10,
    location: location
)

if let data = monthData {
    print("Month: \\(data.month), Year: \\(data.year)")
    print("Days in month: \\(data.daysInMonth)")
    print("Starts on day: \\(data.startDayOfWeek)")
    
    for day in data.days {
        print("BS: \\(day.bsDay) - Tithi: \\(day.tithi.name)")
    }
}
```

### Recurrence Rules

```swift
// Generate BS recurring instances
let bsRule = BsRecurrenceRule(
    frequency: .monthly,
    interval: 1,
    byMonthDay: [1, 15],  // 1st and 15th of each month
    byMonth: nil,
    count: 12
)

let startDate = BsDate(year: 2080, month: 1, day: 1)
let endDate = BsDate(year: 2080, month: 12, day: 30)

if let instances = try? engine.generateBsInstances(
    rule: bsRule,
    start: startDate,
    end: endDate
) {
    for date in instances {
        print("\\(date.year)/\\(date.month)/\\(date.day)")
    }
}

// Generate Tithi-based recurring instances
let tithiRule = TithiRecurrenceRule(
    tithis: [11],  // Ekadashi (11th day)
    paksha: nil,   // Both Shukla and Krishna paksha
    count: 24
)

if let tithiInstances = try? engine.generateTithiInstances(
    eventId: "ekadashi-2080",
    title: "Ekadashi",
    rule: tithiRule,
    start: startDate,
    end: endDate,
    version: CalendarVersion(version: "v1", isOfficial: true),
    location: location
) {
    for instance in tithiInstances {
        print("\\(instance.title): \\(instance.bsDate.year)/\\(instance.bsDate.month)/\\(instance.bsDate.day)")
    }
}
```

### Localization

```swift
// Format dates in Nepali
let formattedDate = engine.formatBsDate(
    date: bsDate,
    pattern: "YYYY MMMM DD",
    lang: .nepali
)
print(formattedDate)  // २०८० बैशाख १

// Get names in different languages
let tithiNameEnglish = engine.getTithiName(tithi: tithi, lang: .english)
let tithiNameNepali = engine.getTithiName(tithi: tithi, lang: .nepali)

let zodiacNameEnglish = engine.getZodiacName(zodiac: .aries, lang: .english)
let zodiacNameNepali = engine.getZodiacName(zodiac: .aries, lang: .nepali)
```

## Error Handling

All conversion and calculation methods can throw errors. Always use `try?` or `do-catch`:

```swift
do {
    let bsDate = try engine.gregorianToBs(date: gregorianDate)
    print("Success: \\(bsDate)")
} catch let error as YorionError {
    switch error {
    case .invalidDate(let msg):
        print("Invalid date: \\(msg)")
    case .outOfRange(let msg):
        print("Out of range: \\(msg)")
    case .conversionError(let msg):
        print("Conversion failed: \\(msg)")
    case .astronomicalError(let msg):
        print("Astronomical calculation failed: \\(msg)")
    case .invalidRecurrenceRule(let msg):
        print("Invalid recurrence rule: \\(msg)")
    }
}
```

## Performance Tips

1. **Reuse the engine**: Create one `CalendarEngine` instance and reuse it
2. **Batch operations**: Use `getMonthCalendar()` instead of individual date conversions
3. **Cache results**: Astronomical calculations are CPU-intensive, cache when possible

## Platform Support

- ✅ macOS 12.0+
- ✅ iOS 15.0+
- ✅ Universal Binary (Intel + Apple Silicon)

## Building for App Store

The XCFramework is ready for App Store submission:

1. Archive your app in Xcode
2. The framework will be automatically included
3. Validate and submit as usual

## Building for GitHub DMG Release

For universal DMG distribution:

1. Build your app with the XCFramework included
2. Create DMG using your preferred tool (e.g., `create-dmg`)
3. The universal binary will work on both Intel and Apple Silicon Macs

## Troubleshooting

### "Module not found" error

Ensure the XCFramework is added to your target's "Frameworks, Libraries, and Embedded Content" section.

### Linker errors

Make sure you're building for the correct architecture. The framework supports both x86_64 and arm64.

### Swift version mismatch

This package requires Swift 5.9+. Update your Xcode if needed.

## License

MIT OR Apache-2.0

## Support

For issues and questions, please open an issue on GitHub.
