use yorion_engine::adapters::static_calendar::StaticCalendarProvider;
use yorion_engine::adapters::static_overrides::StaticTithiOverrideProvider;
use yorion_engine::domain::bs_date::BsMonth;
use yorion_engine::domain::tithi::{Location, Tithi};
use yorion_engine::ports::{CalendarProvider, TithiOverrideProvider};
use chrono::{Datelike, NaiveDate};

mod helpers;

// ===== Static Calendar Provider Tests =====

#[test]
fn test_static_provider() {
    let provider = StaticCalendarProvider::new();

    assert_eq!(provider.version(), "official-1975-2100");
    assert!(provider.is_official());

    assert!(provider.has_year(2080));
    assert!(provider.has_year(1975));
    assert!(provider.has_year(2100));
    assert!(!provider.has_year(1974));
    assert!(!provider.has_year(2101));

    assert_eq!(provider.get_month_days(2080, BsMonth::Baisakh).unwrap(), 31);
    assert_eq!(provider.get_month_days(2080, BsMonth::Jestha).unwrap(), 32);

    let anchor = provider.get_first_baisakh(2080).unwrap();
    assert_eq!(anchor.year(), 2023);
    assert_eq!(anchor.month(), 4);
    assert_eq!(anchor.day(), 14);
}

#[test]
fn test_calendar_data_range() {
    use yorion_engine::adapters::static_calendar::CALENDAR_DATA;

    assert_eq!(CALENDAR_DATA.first().unwrap().year, 1975);
    assert_eq!(CALENDAR_DATA.last().unwrap().year, 2100);
    assert_eq!(CALENDAR_DATA.len(), 126);
}

// ===== Static Tithi Override Provider Tests =====

#[test]
fn test_static_override_provider() {
    let provider = StaticTithiOverrideProvider::new();
    let kathmandu = Location::kathmandu();

    // BS 2079-01-11 = AD 2022-04-24: the almanac lists Krishna Ashtami where the raw
    // engine computes Krishna Navami, so it's pinned.
    let date = NaiveDate::from_ymd_opt(2022, 4, 24).unwrap();
    let overridden = provider.get_override(date, &kathmandu);
    assert_eq!(overridden, Some(Tithi::KrishnaAshtami));

    let date_none = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    assert!(provider.get_override(date_none, &kathmandu).is_none());

    let new_york = Location::new_york();
    assert!(provider.get_override(date, &new_york).is_none());
}
