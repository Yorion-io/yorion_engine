// Services Conversion Tests
use yorion_engine::domain::bs_date::{BsDate, BsMonth};
use yorion_engine::services::ConversionService;
use chrono::NaiveDate;
use std::sync::Arc;

mod helpers;
use helpers::TestCalendarProvider;

fn create_test_service() -> ConversionService {
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

// ============================================================================
// Year boundary: Chaitra last day → next Baisakh first day is 1 AD day apart
// ============================================================================

#[test]
fn test_chaitra_last_day_to_next_baisakh_is_one_day() {
    let service = create_test_service();

    // Last day of BS 2080 Chaitra
    let chaitra_days = service
        .calendar()
        .get_month_days(2080, BsMonth::Chaitra)
        .unwrap();
    let last_chaitra = BsDate::new(2080, 12, chaitra_days).unwrap();
    let first_baisakh = BsDate::new(2081, 1, 1).unwrap();

    let ad_chaitra = service.bs_to_gregorian(last_chaitra).unwrap();
    let ad_baisakh = service.bs_to_gregorian(first_baisakh).unwrap();

    assert_eq!(
        (ad_baisakh - ad_chaitra).num_days(),
        1,
        "last Chaitra and first Baisakh of next year must be exactly 1 AD day apart"
    );
}

// ============================================================================
// Data range boundaries: BS 1975 and BS 2100 must convert successfully
// ============================================================================

#[test]
fn test_data_range_lower_boundary() {
    let service = create_test_service();

    // First represented year is BS 1975.
    let bs = BsDate::new(1975, 1, 1).unwrap();
    let ad = service.bs_to_gregorian(bs).unwrap();
    // Round-trip must hold at the lower boundary
    let back = service.gregorian_to_bs(ad).unwrap();
    assert_eq!(back, bs, "round-trip at lower boundary BS 1975 Baisakh 1");
}

#[test]
fn test_data_range_upper_boundary() {
    let service = create_test_service();

    // Supported data covers BS 1975–2100. The last fully represented year is 2100
    // (its Baisakh 1 anchor is known, so the year is addressable).
    let bs = BsDate::new(2100, 1, 1).unwrap();
    let ad = service.bs_to_gregorian(bs).unwrap();
    let back = service.gregorian_to_bs(ad).unwrap();
    assert_eq!(back, bs, "round-trip at upper boundary BS 2100 Baisakh 1");
}

#[test]
fn test_year_beyond_range_errors() {
    let service = create_test_service();

    let bs = BsDate::new(2101, 1, 1).unwrap();
    assert!(
        service.bs_to_gregorian(bs).is_err(),
        "year 2101 is outside supported range and must error"
    );
}

// ============================================================================
// AD dates around Gregorian year boundary must map to the correct BS year
// ============================================================================

#[test]
fn test_dec_31_maps_to_correct_bs_year() {
    let service = create_test_service();

    // Dec 31 is always before Baisakh (April), so it must be in the BS year
    // that started ~April of that same Gregorian year.
    let dec31 = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    let bs = service.gregorian_to_bs(dec31).unwrap();
    // 2023-04-14 is 2080 Baisakh 1, so Dec 31 2023 is still BS 2080.
    assert_eq!(bs.year, 2080, "2023-12-31 must be BS 2080, not 2081");
}

#[test]
fn test_jan_01_maps_to_correct_bs_year() {
    let service = create_test_service();

    // Jan 1 2024 is before Baisakh 2081 (2024-04-13), so it must be BS 2080.
    let jan1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let bs = service.gregorian_to_bs(jan1).unwrap();
    assert_eq!(bs.year, 2080, "2024-01-01 must be BS 2080 (before new BS year)");
}
