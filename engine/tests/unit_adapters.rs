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

#[test]
fn test_static_holiday_provider() {
    use yorion_engine::adapters::static_holidays::StaticHolidayProvider;
    use yorion_engine::ports::HolidayProvider;

    let provider = StaticHolidayProvider::new();
    let kathmandu = Location::kathmandu();

    // Bijaya Dashami BS 2083-07-04 = AD 2026-10-21 (from data/holidays.csv).
    let dashami = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
    let h = provider.get_holiday(dashami, &kathmandu);
    assert!(h.is_some(), "Bijaya Dashami should be a gazette holiday");
    assert_eq!(h.unwrap().name_en, "Bijaya Dashami");
    assert!(provider.is_holiday(dashami, &kathmandu));

    // An ordinary weekday is not a gazette holiday.
    let ordinary = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
    assert!(provider.get_holiday(ordinary, &kathmandu).is_none());

    // Gazette holidays are Nepal-only — gated on follow_nepal_social_calendar.
    let new_york = Location::new_york();
    assert!(provider.get_holiday(dashami, &new_york).is_none());
}

#[test]
fn test_weekend_rule_and_sunday_gate() {
    use yorion_engine::core_api::is_weekend;

    // Saturday is always a weekend, before and after the 2082 policy.
    // 2020-01-04 and 2026-05-16 are both Saturdays (day_of_week == 6).
    let sat_old = NaiveDate::from_ymd_opt(2020, 1, 4).unwrap();
    let sat_new = NaiveDate::from_ymd_opt(2026, 5, 16).unwrap();
    assert!(is_weekend(6, sat_old));
    assert!(is_weekend(6, sat_new));

    // Sunday BEFORE 2026-04-06 is a working day; AFTER, it's a weekend.
    // 2026-03-29 (Sun, before) vs 2026-04-12 (Sun, after).
    let sun_before = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();
    let sun_after = NaiveDate::from_ymd_opt(2026, 4, 12).unwrap();
    assert!(!is_weekend(0, sun_before), "Sunday before the policy is a work day");
    assert!(is_weekend(0, sun_after), "Sunday after the policy is a weekend");

    // The gate boundary itself (2026-04-06 is a Monday, so test the first
    // affected Sunday, 2026-04-12, already covered above). A midweek day is
    // never a weekend.
    let wed = NaiveDate::from_ymd_opt(2026, 4, 8).unwrap();
    assert!(!is_weekend(3, wed));
}

#[test]
fn test_holidays_verified_boundary_covers_data() {
    // The human-maintained marker must not claim less coverage than the baked
    // data, or verified years would wrongly render as "not yet published".
    use yorion_engine::adapters::static_holidays::HOLIDAYS;
    use yorion_engine::core_api::HOLIDAYS_VERIFIED_THROUGH_BS;

    // Max AD year present in the table maps to ~ BS year - 57. The last data
    // year is BS 2083 (AD 2026/2027), so the marker must be >= 2083.
    let max_ad_year = HOLIDAYS.iter().map(|((y, _, _), _, _)| *y).max().unwrap();
    // BS 2083 spans AD 2026-04 .. 2027-04, so table reaches AD 2027.
    assert!(max_ad_year >= 2026, "expected holiday data through at least AD 2026");
    assert!(
        HOLIDAYS_VERIFIED_THROUGH_BS >= 2083,
        "marker must cover the baked data (BS 2083)"
    );
    assert!(!HOLIDAYS.is_empty(), "holiday table should not be empty");
}
