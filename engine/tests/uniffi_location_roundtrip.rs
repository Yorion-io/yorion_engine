//! Regression: the FFI `Location` -> `CoreLocation` conversion used to drop
//! `follow_nepal_social_calendar`, because it built through `CoreLocation::new`
//! which hard-codes that field to `false`.
//!
//! `StaticHolidayProvider::get_holiday` returns `None` when the flag is false,
//! so *every* UniFFI consumer (Swift, Kotlin, the Flutter plugin) saw
//! `is_holiday: false` on every day of every month regardless of what it passed.
//! The reverse conversion always preserved the flag, which is why the asymmetry
//! went unnoticed.
#![cfg(feature = "uniffi-bindings")]

use yorion_engine::uniffi_bindings::{create_engine, Location};

fn kathmandu() -> Location {
    Location {
        latitude: 27.7172,
        longitude: 85.3240,
        name: "Kathmandu".to_string(),
        timezone_offset_mins: 345,
        follow_nepal_social_calendar: true,
    }
}

#[test]
fn ffi_month_calendar_reports_gazette_holidays() {
    let engine = create_engine();
    // Bhadra 2083 spans 2026-08-17..09-16 and contains Janai Poornima
    // (2026-08-28) and Gaijatra (08-29) in data/holidays.csv.
    let data = engine
        .get_month_calendar(2083, 5, kathmandu())
        .expect("month calendar");

    let holidays: Vec<_> = data.days.iter().filter(|d| d.is_holiday).collect();
    assert!(
        !holidays.is_empty(),
        "Bhadra 2083 has gazette holidays; got none — the social-calendar flag \
         was probably dropped crossing the FFI boundary again"
    );

    let janai = data
        .days
        .iter()
        .find(|d| {
            d.gregorian_date.year == 2026
                && d.gregorian_date.month == 8
                && d.gregorian_date.day == 28
        })
        .expect("2026-08-28 is in Bhadra 2083");
    assert!(janai.is_holiday, "2026-08-28 is Janai Poornima");
    assert_eq!(janai.holiday_name, "Janai Poornima");
}

#[test]
fn ffi_respects_a_location_that_does_not_follow_the_nepal_calendar() {
    let engine = create_engine();
    let mut abroad = kathmandu();
    abroad.follow_nepal_social_calendar = false;

    let data = engine
        .get_month_calendar(2083, 5, abroad)
        .expect("month calendar");
    assert!(
        data.days.iter().all(|d| !d.is_holiday),
        "gazette holidays must not apply when the caller opts out"
    );
}

#[test]
fn ffi_weekend_flags_are_independent_of_the_holiday_table() {
    let engine = create_engine();
    let data = engine
        .get_month_calendar(2083, 5, kathmandu())
        .expect("month calendar");

    // Weekends are computed, not gazette data, so they survive either setting.
    assert!(
        data.days.iter().any(|d| d.is_weekend),
        "a 31-day month always contains weekends"
    );
}
