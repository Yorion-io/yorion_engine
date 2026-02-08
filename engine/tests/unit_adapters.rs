// Adapter tests - Static calendar, overrides, and time providers
use bs_calendar_core::adapters::static_calendar::StaticCalendarProvider;
use bs_calendar_core::adapters::static_overrides::StaticTithiOverrideProvider;
use bs_calendar_core::adapters::system_time::SystemTimeProvider;
use bs_calendar_core::domain::bs_date::BsMonth;
use bs_calendar_core::domain::tithi::{Location, Tithi};
use bs_calendar_core::ports::{CalendarProvider, TimeProvider, TithiOverrideProvider};
use chrono::{Datelike, NaiveDate, Timelike};

mod helpers;

// ===== Static Calendar Provider Tests =====

#[test]
fn test_static_provider() {
    let provider = StaticCalendarProvider::new();

    // Test version
    assert_eq!(provider.version(), "official-2000-2090");
    assert!(provider.is_official());

    // Test has_year
    assert!(provider.has_year(2080));
    assert!(!provider.has_year(1999));

    // Test get_month_days
    assert_eq!(provider.get_month_days(2080, BsMonth::Baisakh).unwrap(), 31);
    assert_eq!(provider.get_month_days(2080, BsMonth::Jestha).unwrap(), 32);

    // Test get_first_baisakh
    let anchor = provider.get_first_baisakh(2080).unwrap();
    assert_eq!(anchor.year(), 2023);
    assert_eq!(anchor.month(), 4);
    assert_eq!(anchor.day(), 14);
}

#[test]
fn test_calendar_data_range() {
    use bs_calendar_core::adapters::static_calendar::CALENDAR_DATA;

    // Verify we have data from 2000 to 2090
    assert_eq!(CALENDAR_DATA.first().unwrap().year, 2000);
    assert_eq!(CALENDAR_DATA.last().unwrap().year, 2090);
    assert_eq!(CALENDAR_DATA.len(), 91);
}

// ===== Static Tithi Override Provider Tests =====

#[test]
fn test_static_override_provider() {
    let provider = StaticTithiOverrideProvider::new();
    let kathmandu = Location::KATHMANDU;

    // Test a known override from 2077 (2020-10-30)
    let date = NaiveDate::from_ymd_opt(2020, 10, 30).unwrap();
    let overridden = provider.get_override(date, &kathmandu);
    assert_eq!(overridden, Some(Tithi::Purnima));

    // Test a date with no override
    let date_none = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    assert!(provider.get_override(date_none, &kathmandu).is_none());

    // Test with a location that doesn't follow Nepal calendar
    let new_york = Location::NEW_YORK;
    assert!(provider.get_override(date, &new_york).is_none());
}

// ===== System Time Provider Tests =====

#[test]
fn test_sunrise_calculation() {
    let provider = SystemTimeProvider::new();
    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();
    let location = Location::KATHMANDU;

    let sunrise = provider.sunrise_time(date, location).unwrap();

    // Kathmandu sunrise in Jan should be around 6:50 AM
    assert!(sunrise.hour() >= 6 && sunrise.hour() <= 7);
}

#[test]
fn test_sunset_calculation() {
    let provider = SystemTimeProvider::new();
    let date = NaiveDate::from_ymd_opt(2026, 1, 28).unwrap();
    let location = Location::KATHMANDU;

    let sunset = provider.sunset_time(date, location).unwrap();

    // Sunset should be around 5:40 PM (17:40)
    assert!(sunset.hour() >= 17 && sunset.hour() <= 19);
}
