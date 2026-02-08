//! Property-based tests for the BS Calendar Core
//!
//! These tests use proptest to verify invariants across many random inputs.

use bs_calendar_core::core_api::CalendarEngine;
use bs_calendar_core::domain::{BsDate, BsMonth};
use chrono::NaiveDate;
use proptest::prelude::*;

/// Test that BS to AD to BS conversion is a perfect roundtrip for VALID dates
#[test]
fn test_roundtrip_bs_to_ad_to_bs() {
    let engine = CalendarEngine::new();

    proptest!(|(
        year in 2000u16..2090,
        month in 1u8..=12,
        day in 1u8..=31
    )| {
        // BsDate::new only validates 1-32 range, not actual month days
        // So we need to check if conversion succeeds
        if let Ok(bs_original) = BsDate::new(year, month, day) {
            if let Ok(ad) = engine.bs_to_gregorian(bs_original) {
                let bs_back = engine.gregorian_to_bs(ad)?;
                prop_assert_eq!(bs_original, bs_back,
                    "Roundtrip failed: {} -> {} -> {}", bs_original, ad, bs_back);
            }
            // If conversion fails, it means the day is invalid for that month
            // This is expected behavior
        }
    });
}

/// Test that AD to BS to AD conversion is a perfect roundtrip
#[test]
fn test_roundtrip_ad_to_bs_to_ad() {
    let engine = CalendarEngine::new();

    proptest!(|(
        year in 1943i32..2033,
        month in 1u32..=12,
        day in 1u32..=28  // Use 28 to avoid month-end edge cases
    )| {
        if let Some(ad_original) = NaiveDate::from_ymd_opt(year, month, day) {
            if let Ok(bs) = engine.gregorian_to_bs(ad_original) {
                let ad_back = engine.bs_to_gregorian(bs)?;
                prop_assert_eq!(ad_original, ad_back,
                    "Roundtrip failed: {} -> {} -> {}", ad_original, bs, ad_back);
            }
        }
    });
}

/// Test that BS dates with valid day range (1-32) can be created
/// but conversion may fail for days beyond actual month length
#[test]
fn test_bs_date_creation_and_conversion() {
    proptest!(|(
        year in 2000u16..2090,
        month in 1u8..=12,
        day in 1u8..=31
    )| {
        match BsDate::new(year, month, day) {
            Ok(bs_date) => {
                // BsDate creation succeeded (day is 1-32)
                // Conversion may still fail if day > actual month days
                let engine = CalendarEngine::new();
                let _ = engine.bs_to_gregorian(bs_date);
                // We don't assert here because some valid BsDate instances
                // may have days beyond the actual month length
            }
            Err(_) => {
                // Only fails if day is 0 or > 32, or month is invalid
                prop_assert!(day == 0 || day > 32 || month == 0 || month > 12);
            }
        }
    });
}

/// Test that converted dates maintain chronological order
#[test]
fn test_chronological_order() {
    let engine = CalendarEngine::new();

    proptest!(|(
        year in 2000u16..2089,
        month in 1u8..=12,
        day in 1u8..=15  // Use smaller day to ensure we can add 1
    )| {
        if let Ok(bs1) = BsDate::new(year, month, day) {
            if let Ok(bs2) = BsDate::new(year, month, day + 1) {
                // Only test if both conversions succeed
                if let (Ok(ad1), Ok(ad2)) = (
                    engine.bs_to_gregorian(bs1),
                    engine.bs_to_gregorian(bs2)
                ) {
                    prop_assert!(ad1 < ad2,
                        "Chronological order violated: {} ({}) should be before {} ({})",
                        bs1, ad1, bs2, ad2);
                }
            }
        }
    });
}

/// Test that month boundaries are handled correctly
#[test]
fn test_month_boundaries() {
    let engine = CalendarEngine::new();

    for year in 2000..2090 {
        for month in 1..=12 {
            let bs_month = BsMonth::from_u8(month).unwrap();
            let days_in_month = engine.calendar().get_month_days(year, bs_month).unwrap();

            // First day of month should be valid
            let first_day = BsDate::new(year, month, 1).unwrap();
            assert!(engine.bs_to_gregorian(first_day).is_ok());

            // Last day of month should be valid
            let last_day = BsDate::new(year, month, days_in_month).unwrap();
            assert!(engine.bs_to_gregorian(last_day).is_ok());

            // Day beyond actual month length should fail conversion
            // (but BsDate creation may succeed if <= 32)
            if days_in_month < 32 {
                let invalid_day = BsDate::new(year, month, days_in_month + 1);
                if let Ok(date) = invalid_day {
                    // BsDate creation succeeded, but conversion should fail
                    assert!(
                        engine.bs_to_gregorian(date).is_err(),
                        "Month {}/{} has {} days but day {} converted successfully",
                        year,
                        month,
                        days_in_month,
                        days_in_month + 1
                    );
                }
            }
        }
    }
}

/// Test that year boundaries are handled correctly
#[test]
fn test_year_boundaries() {
    let engine = CalendarEngine::new();

    // Test first supported year (2000 BS)
    let first_bs = BsDate::new(2000, 1, 1).unwrap();
    assert!(engine.bs_to_gregorian(first_bs).is_ok());

    // Test last supported year (2089 BS)
    let last_bs = BsDate::new(2089, 12, 30).unwrap();
    assert!(engine.bs_to_gregorian(last_bs).is_ok());

    // Note: BsDate::new doesn't validate year range, only day (1-32) and month (1-12)
    // Year validation happens during conversion
    let high_year = BsDate::new(2100, 1, 1).unwrap();
    assert!(
        engine.bs_to_gregorian(high_year).is_err(),
        "Year 2100 should fail conversion"
    );
}

/// Test that astronomical calculations don't panic
#[test]
fn test_astronomical_calculations_no_panic() {
    let engine = CalendarEngine::new();

    proptest!(|(
        year in 2000i32..2033,
        month in 1u32..=12,
        day in 1u32..=28
    )| {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            // These should not panic, even if they return errors
            let _ = engine.get_tithi(date);
            let _ = engine.get_sun_zodiac(date);
            let _ = engine.get_moon_zodiac(date);
            let _ = engine.get_nakshatra(date);
        }
    });
}
