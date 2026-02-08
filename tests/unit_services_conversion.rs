// Services Conversion Tests
use bs_calendar_core::domain::bs_date::{BsDate, BsMonth};
use bs_calendar_core::services::ConversionService;
use chrono::NaiveDate;
use std::sync::Arc;

mod helpers;
use helpers::TestCalendarProvider;

fn create_test_service() -> ConversionService {
    // We use a helper provider that mimics StaticCalendarProvider behavior for tests
    // or we can use the actual one if exposed.
    // For now, let's assume we can access what we need or use a mock from helpers if needed.
    // However, looking at the inline tests, they used `StaticCalendarProvider`.
    // If `adapters` is pub, we can use it.
    // If not, we might need to rely on the helper or expose it.
    // Let's assume `bs_calendar_core::adapters::StaticCalendarProvider` is available or we use a similar mock.
    // Ideally we should use the same provider to ensure consistency.

    // Using the TestCalendarProvider from helpers for now which should cover basic needs
    // OR if we want exact parity, we might need to check if we can import StaticCalendarProvider.
    let provider = TestCalendarProvider::new();
    ConversionService::new(Arc::new(provider))
}

#[test]
fn test_bs_to_gregorian() {
    let service = create_test_service();

    // 2080 Baisakh 1 = 2023 April 14
    let bs_date = BsDate::new(2080, 1, 1).unwrap();
    let gregorian = service.bs_to_gregorian(bs_date).unwrap();
    assert_eq!(gregorian, NaiveDate::from_ymd_opt(2023, 4, 14).unwrap());

    // 2081 Baisakh 1 = 2024 April 13
    let bs_date = BsDate::new(2081, 1, 1).unwrap();
    let gregorian = service.bs_to_gregorian(bs_date).unwrap();
    assert_eq!(gregorian, NaiveDate::from_ymd_opt(2024, 4, 13).unwrap());
}

#[test]
fn test_gregorian_to_bs() {
    let service = create_test_service();

    // 2023 April 14 = 2080 Baisakh 1
    let gregorian = NaiveDate::from_ymd_opt(2023, 4, 14).unwrap();
    let bs_date = service.gregorian_to_bs(gregorian).unwrap();
    assert_eq!(bs_date.year, 2080);
    assert_eq!(bs_date.month, BsMonth::Baisakh);
    assert_eq!(bs_date.day, 1);

    // 2024 April 13 = 2081 Baisakh 1
    let gregorian = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
    let bs_date = service.gregorian_to_bs(gregorian).unwrap();
    assert_eq!(bs_date.year, 2081);
    assert_eq!(bs_date.month, BsMonth::Baisakh);
    assert_eq!(bs_date.day, 1);
}

#[test]
fn test_round_trip_conversion() {
    let service = create_test_service();

    // Test multiple dates
    let test_dates = vec![
        NaiveDate::from_ymd_opt(2023, 4, 14).unwrap(), // 2080 Baisakh 1
        NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 4, 13).unwrap(), // 2081 Baisakh 1
    ];

    for gregorian in test_dates {
        let bs_date = service.gregorian_to_bs(gregorian).unwrap();
        let back_to_gregorian = service.bs_to_gregorian(bs_date).unwrap();
        assert_eq!(
            gregorian, back_to_gregorian,
            "Round trip failed for {}",
            gregorian
        );
    }
}

#[test]
fn test_date_before_baisakh() {
    let service = create_test_service();

    // Jan 13, 2026 is before Baisakh 1, 2083 (Apr 14, 2026)
    // So it should belong to BS 2082
    let gregorian = NaiveDate::from_ymd_opt(2026, 1, 13).unwrap();
    let bs_date = service.gregorian_to_bs(gregorian).unwrap();
    assert_eq!(bs_date.year, 2082);
}

#[test]
fn test_clamp_bs_date() {
    let service = create_test_service();

    // 2080 Chaitra has 30 days, so day 32 should clamp to 30
    let clamped = service.clamp_bs_date(2080, BsMonth::Chaitra, 32).unwrap();
    assert_eq!(clamped.day, 30);

    // Day 0 should clamp to 1
    let clamped = service.clamp_bs_date(2080, BsMonth::Baisakh, 0).unwrap();
    assert_eq!(clamped.day, 1);
}
