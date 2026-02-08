#![allow(dead_code, unused_imports)]

// Test helpers and utilities
use bs_calendar_core::core_api::CalendarEngine;
use bs_calendar_core::domain::{BsDate, BsMonth};
use chrono::NaiveDate;

// Re-export StaticCalendarProvider as TestCalendarProvider for service tests
pub use bs_calendar_core::adapters::StaticCalendarProvider as TestCalendarProvider;

/// Create a sample BS date for testing (2080-01-15)
pub fn sample_bs_date() -> BsDate {
    BsDate::new(2080, 1, 15).unwrap()
}

/// Create a sample Gregorian date for testing (2023-04-28)
pub fn sample_gregorian_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2023, 4, 28).unwrap()
}

/// Get a test calendar engine
pub fn test_engine() -> CalendarEngine {
    CalendarEngine::new()
}

/// Assert two BS dates are equal with helpful error message
#[track_caller]
pub fn assert_bs_date_eq(actual: BsDate, expected: BsDate) {
    assert_eq!(
        actual, expected,
        "BS Date mismatch:\n  Expected: {}\n  Actual: {}",
        expected, actual
    );
}

/// Assert roundtrip BS → AD → BS conversion works
#[track_caller]
pub fn assert_roundtrip_bs_ad_bs(bs: BsDate) {
    let engine = test_engine();
    let ad = engine
        .bs_to_gregorian(bs)
        .expect("BS to AD conversion failed");
    let bs_back = engine
        .gregorian_to_bs(ad)
        .expect("AD to BS conversion failed");
    assert_bs_date_eq(bs_back, bs);
}

/// Assert roundtrip AD → BS → AD conversion works
#[track_caller]
pub fn assert_roundtrip_ad_bs_ad(ad: NaiveDate) {
    let engine = test_engine();
    let bs = engine
        .gregorian_to_bs(ad)
        .expect("AD to BS conversion failed");
    let ad_back = engine
        .bs_to_gregorian(bs)
        .expect("BS to AD conversion failed");
    assert_eq!(
        ad_back, ad,
        "AD Date mismatch:\n  Expected: {}\n  Actual: {}",
        ad, ad_back
    );
}

/// Create a valid BS date for a specific year/month/day
pub fn bs_date(year: u16, month: u8, day: u8) -> BsDate {
    BsDate::new(year, month, day).unwrap()
}

/// Create a BS month from u8
pub fn bs_month(month: u8) -> BsMonth {
    BsMonth::from_u8(month).unwrap()
}
