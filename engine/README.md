# BS Calendar Core

A robust, platform-agnostic Rust library for the Bikram Sambat (BS) calendar system. This library handles date conversions, tithi (lunar day) calculations, and complex recurrence rules. Designed with clean architecture (Ports and Adapters) and compiled to WebAssembly for web applications.

## 🚀 Key Features

*   **Accurate Conversions**: Convert between Bikram Sambat (BS) and Gregorian (AD) dates with high precision using historical anchor points.
*   **Tithi Support**: Calculate Tithis (lunar days), Pakshas (lunar fortnights), and handle lunar events.
*   **Recurrence Rules**: Powerful recurrence engine (similar to iCal RRULE) for both solar (BS dates) and lunar (Tithi-based) events.
*   **WASM Ready**: Compiled to WebAssembly for use in browsers, bundlers, and Node.js.
*   **TypeScript Support**: Full TypeScript definitions included.
*   **Extensible Architecture**: Uses Ports and Adapters to allow swapping data sources and time providers.

## 🏗️ Architecture Overview

The project is structured into four main layers to ensure separation of concerns:

### 1. Domain (`src/domain`)
Contains the core data structures and logic of the calendar system. These are pure Rust structs/enums with no external side effects.
*   **`BsDate`**: Represents a specific date in the BS calendar.
*   **`Tithi` / `Paksha`**: Represents specific lunar moments.
*   **`BsRecurrenceRule`**: Defines rules for repeating events (e.g., "Every year on Baisakh 1st").

### 2. Ports (`src/ports`)
Defines the usage interfaces (Traits) that the core system needs. This allows us to potentially swap out implementations later.
*   **`CalendarProvider`**: Trait for fetching calendar data (days in month, etc.).
*   **`TimeProvider`**: Trait for getting the current system time.

### 3. Adapters (`src/adapters`)
Concrete implementations of the ports.
*   **`StaticCalendarProvider`**: Embeds calendar data directly into the binary (fast, no file I/O).

### 4. Services (`src/services`)
High-level business logic that coordinates usage of the domain and ports.
*   **`ConversionService`**: Handles `BS ↔ Gregorian` conversions.
*   **`AstronomicalService`**: High-precision astronomical calculations for specific Tithi timings.
*   **`InstanceGenerator`**: Expands recurrence rules into actual event occurrences.

## 🛠️ Usage

### Prerequisites
*   [Rust installed](https://www.rust-lang.org/tools/install) (latest stable version).
*   For WASM builds: `wasm-pack`

### Basic Date Conversion

To convert dates, you use the `ConversionService`.

```rust
use yorion_engine::prelude::*;
use std::sync::Arc;

fn main() -> Result<()> {
    // 1. Initialize the calendar data provider
    let provider = StaticCalendarProvider::new();
    
    // 2. Create the conversion service
    let service = ConversionService::new(Arc::new(provider));

    // 3. Convert BS to Gregorian
    let bs_date = BsDate::new(2080, 1, 1)?; // Baisakh 1, 2080
    let gregorian = service.bs_to_gregorian(bs_date)?;
    println!("BS: {} = AD: {}", bs_date, gregorian);

    Ok(())
}
```

### Building WASM

```bash
# Build for all WASM targets (web, bundler, nodejs)
cd ..
./scripts/build-wasm.sh

# Output will be in dist/wasm/
```

## 📚 Examples

The project comes with several examples to help you understand how to use the library. You can find them in the `examples/` directory.

To run an example, use the command:
```bash
cargo run --example <example_name>
```

### 1. Basic Conversion (`examples/basic_conversion.rs`)
Demonstrates simple conversion between BS and Gregorian dates.
```bash
cargo run --example basic_conversion
```

### 2. Today's Date (`examples/today.rs`)
Shows how to get the current date in the BS calendar.
```bash
cargo run --example today
```

### 3. Recurring Events (`examples/recurring_events.rs`)
A complex example showing how to generate repeating events (like birthdays or festivals) over a period of time.
```bash
cargo run --example recurring_events
```

### 4. Calendar Generation (`examples/generate_calendar_multi.rs`)
Generates calendar data for multiple years, useful for verifying data integrity or generating UI inputs.
```bash
cargo run --example generate_calendar_multi
```

## 📂 Project Structure

```
.
├── Cargo.toml          # Dependencies and project metadata
├── examples/           # Ready-to-run example code
├── src/
│   ├── lib.rs          # Library entry point
│   ├── domain/         # Core types (BsDate, Tithi, etc.)
│   ├── services/       # Business logic (Conversion, etc.)
│   ├── ports/          # Traits (CalendarProvider, etc.)
│   ├── adapters/       # Implementations (StaticProvider, etc.)
│   └── wasm.rs         # WASM bindings
└── data/               # Raw calendar data (JSON/CSV)
```

## 🧪 Testing

```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test test_bs_to_gregorian
```
