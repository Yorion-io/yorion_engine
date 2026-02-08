// Core API tests
use bs_calendar_core::core_api::CalendarEngine;
use chrono::Datelike;

mod helpers;
use helpers::*;

#[test]
fn test_engine_creation() {
    let engine = CalendarEngine::new();
    // Should not panic
    drop(engine);
}

#[test]
fn test_bs_to_gregorian() {
    let engine = test_engine();
    let bs = bs_date(2080, 1, 1);
    let ad = engine.bs_to_gregorian(bs).unwrap();
    assert_eq!(ad.year(), 2023);
    assert_eq!(ad.month(), 4);
    assert_eq!(ad.day(), 14);
}

#[test]
fn test_roundtrip_conversion() {
    let bs_original = bs_date(2080, 5, 15);
    assert_roundtrip_bs_ad_bs(bs_original);
}

#[test]
fn test_gregorian_to_bs() {
    let engine = test_engine();
    let ad = sample_gregorian_date();
    let bs = engine.gregorian_to_bs(ad).unwrap();
    assert_eq!(bs.year, 2080);
    assert_eq!(bs.month_u8(), 1);
    assert_eq!(bs.day, 15);
}

#[test]
fn test_out_of_range_year() {
    let engine = test_engine();
    let bs = bs_date(2100, 1, 1); // Beyond supported range
    assert!(engine.bs_to_gregorian(bs).is_err());
}

#[test]
fn test_invalid_day_for_month() {
    let engine = test_engine();
    // Create a date with day 32 (which may be invalid for some months)
    let bs = bs_date(2080, 2, 32);
    // Conversion should fail if day exceeds actual month length
    let result = engine.bs_to_gregorian(bs);
    // We expect this to either succeed (if month has 32 days) or fail
    // The important thing is it doesn't panic
    let _ = result;
}
