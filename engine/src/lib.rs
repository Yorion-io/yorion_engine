//! BS Calendar Core Engine
//!
//! A platform-agnostic Rust library for Bikram Sambat (BS) calendar operations,
//! including BS ↔ Gregorian conversion, tithi calculations, and recurrence rules.
//!
//! # Features
//!
//! - Accurate BS ↔ Gregorian date conversion using anchor points
//! - Support for BS years 2000-2090
//! - Tithi (lunar day) calculations
//! - Recurrence rules for both BS and tithi-based scheduling
//! - Clean architecture with ports and adapters
//! - Platform-agnostic core (works on web, mobile, desktop)
//!
//! # Example
//!
//! ```rust
//! use bs_calendar_core::prelude::*;
//! use std::sync::Arc;
//!
//! // Load calendar data
//! // Load calendar data
//! let provider = StaticCalendarProvider::new();
//!
//! // Create conversion service
//! let service = ConversionService::new(Arc::new(provider));
//!
//! // Convert BS to Gregorian
//! let bs_date = BsDate::new(2080, 1, 1).unwrap();
//! let gregorian = service.bs_to_gregorian(bs_date).unwrap();
//! println!("{} = {}", bs_date, gregorian);
//!
//! // Convert Gregorian to BS
//! use chrono::NaiveDate;
//! let gregorian = NaiveDate::from_ymd_opt(2023, 4, 14).unwrap();
//! let bs_date = service.gregorian_to_bs(gregorian).unwrap();
//! println!("{} = {}", gregorian, bs_date);
//! ```

pub mod adapters;
pub mod core_api;
pub mod domain;
pub mod error;
pub mod ports;
pub mod services;
pub mod utils;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "uniffi-bindings")]
pub mod uniffi_bindings;

#[cfg(feature = "uniffi-bindings")]
use uniffi_bindings::*;

#[cfg(feature = "uniffi-bindings")]
uniffi::include_scaffolding!("uniffi");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::adapters::{
        StaticCalendarProvider, StaticTithiOverrideProvider, SystemTimeProvider,
    };
    pub use crate::domain::{
        BsDate, BsFrequency, BsMonth, BsRecurrenceRule, CalendarVersion, EventInstance, Location,
        Nakshatra, Paksha, Tithi, TithiRecurrenceRule, ZodiacSign,
    };
    pub use crate::error::{BsCalendarError, Result};
    pub use crate::ports::{
        CalendarProvider, LocationProvider, TimeProvider, TithiOverrideProvider,
    };
    pub use crate::services::{
        AstronomicalService, ConversionService, InstanceGenerator, TithiInstanceGenerator,
    };
}
