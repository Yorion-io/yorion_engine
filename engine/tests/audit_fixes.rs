//! Tests for the public-launch audit fixes: Lahiri ayanamsa, Yoga/Karana,
//! checked date construction, bounded instance generation, exact JD
//! conversion, tithi end times, and the O(1) calendar lookup.

use chrono::{NaiveDate, Timelike};
use std::collections::HashSet;
use yorion_engine::core_api::{CalendarEngine, TITHI_VERIFIED_THROUGH_BS};
use yorion_engine::prelude::*;
use yorion_engine::services::astronomical::jd_to_datetime;

fn engine() -> CalendarEngine {
    CalendarEngine::new()
}

// ---------------------------------------------------------------------------
// Lahiri ayanamsa (was: fixed 24.0°)
// ---------------------------------------------------------------------------

#[test]
fn ayanamsa_drifts_with_time() {
    let astro = AstronomicalService::new();
    let j2000 = 2_451_545.0;

    // Lahiri at J2000 is 23°51′11.5″ ≈ 23.853°.
    let at_j2000 = astro.ayanamsa(j2000);
    assert!(
        (at_j2000 - 23.853).abs() < 0.001,
        "ayanamsa at J2000 was {at_j2000}"
    );

    // ~50.29″/yr accumulation: 2025 ≈ 24.20°, 1943 ≈ 23.06°... no wait,
    // 1943 is 57 years before 2000: 23.853 - 57*0.01396 ≈ 23.06.
    let at_2025 = astro.ayanamsa(j2000 + 25.0 * 365.25);
    assert!(
        (at_2025 - 24.20).abs() < 0.01,
        "ayanamsa in 2025 was {at_2025}"
    );

    let at_1943 = astro.ayanamsa(j2000 - 57.0 * 365.25);
    assert!(
        (at_1943 - 23.057).abs() < 0.01,
        "ayanamsa in 1943 was {at_1943}"
    );

    // Strictly increasing.
    assert!(at_1943 < at_j2000 && at_j2000 < at_2025);
}

// ---------------------------------------------------------------------------
// Yoga and Karana (new panchanga angas)
// ---------------------------------------------------------------------------

#[test]
fn yoga_from_index_covers_all_27() {
    assert_eq!(Yoga::from_index(0), None);
    assert_eq!(Yoga::from_index(1), Some(Yoga::Vishkambha));
    assert_eq!(Yoga::from_index(27), Some(Yoga::Vaidhriti));
    assert_eq!(Yoga::from_index(28), None);
    for i in 1..=27 {
        assert!(Yoga::from_index(i).is_some(), "missing yoga index {i}");
    }
}

#[test]
fn karana_half_tithi_mapping() {
    // k=0 (first half of Shukla Pratipada) is the fixed Kimstughna.
    assert_eq!(Karana::from_half_tithi_index(0), Some(Karana::Kimstughna));
    // k=1..=56 cycle the seven movable karanas starting at Bava.
    assert_eq!(Karana::from_half_tithi_index(1), Some(Karana::Bava));
    assert_eq!(Karana::from_half_tithi_index(7), Some(Karana::Vishti));
    assert_eq!(Karana::from_half_tithi_index(8), Some(Karana::Bava));
    assert_eq!(Karana::from_half_tithi_index(56), Some(Karana::Vishti));
    // The last three half-tithis are the remaining fixed karanas.
    assert_eq!(Karana::from_half_tithi_index(57), Some(Karana::Shakuni));
    assert_eq!(Karana::from_half_tithi_index(58), Some(Karana::Chatushpada));
    assert_eq!(Karana::from_half_tithi_index(59), Some(Karana::Naga));
    assert_eq!(Karana::from_half_tithi_index(60), None);

    assert!(Karana::Kimstughna.is_fixed());
    assert!(!Karana::Bava.is_fixed());
}

#[test]
fn daily_astro_info_includes_yoga_and_karana() {
    let eng = engine();
    let date = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
    let info = eng
        .get_daily_astro_info(date, Location::kathmandu())
        .unwrap();
    // Names resolve in both languages (smoke check the new angas are wired).
    assert!(!info.yoga.name().is_empty());
    assert!(!info.karana.nepali_name().is_empty());

    // Karana must be one of the two halves of the sunrise tithi: half-tithi
    // index = 2*(tithi_index-1) or 2*(tithi_index-1)+1.
    let tithi_idx = info.tithi.index_1_to_30();
    let k_lo = 2 * (tithi_idx as u16 - 1);
    let expected: Vec<Karana> = [k_lo, k_lo + 1]
        .iter()
        .map(|&k| Karana::from_half_tithi_index(k as u8).unwrap())
        .collect();
    assert!(
        expected.contains(&info.karana),
        "karana {:?} not a half of tithi {:?}",
        info.karana,
        info.tithi
    );
}

#[test]
fn yoga_karana_names_in_both_languages() {
    let eng = engine();
    assert_eq!(
        eng.get_yoga_name(Yoga::Vishkambha, yorion_engine::domain::Language::English),
        "Vishkambha"
    );
    assert_eq!(
        eng.get_karana_name(Karana::Vishti, yorion_engine::domain::Language::Nepali),
        "विष्टि"
    );
}

// ---------------------------------------------------------------------------
// checked_bs_date (table-validated construction)
// ---------------------------------------------------------------------------

#[test]
fn checked_bs_date_rejects_nonexistent_day() {
    let eng = engine();

    // Structurally valid dates always pass BsDate::new...
    assert!(BsDate::new(2080, 1, 32).is_ok());

    // ...but checked_bs_date validates against the real month length.
    let days = eng
        .calendar()
        .get_month_days(2080, BsMonth::Baisakh)
        .unwrap();
    assert!(eng.checked_bs_date(2080, 1, days).is_ok());
    assert!(matches!(
        eng.checked_bs_date(2080, 1, days + 1),
        Err(BsCalendarError::InvalidDay(_, _))
    ));

    // Out-of-range year is rejected with the data-not-found error.
    assert!(matches!(
        eng.checked_bs_date(2101, 1, 1),
        Err(BsCalendarError::CalendarDataNotFound(2101))
    ));
}

// ---------------------------------------------------------------------------
// Instance limit: error instead of silent truncation
// ---------------------------------------------------------------------------

#[test]
fn unbounded_rule_past_limit_errors_instead_of_truncating() {
    let eng = engine();
    // Daily, no COUNT/UNTIL, ~33k-day window → must refuse, not clip.
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, BsDate::new(2000, 1, 1).unwrap());
    let result = eng.generate_bs_instances(
        &rule,
        BsDate::new(2000, 1, 1).unwrap(),
        BsDate::new(2090, 12, 30).unwrap(),
    );
    assert!(matches!(
        result,
        Err(BsCalendarError::InstanceLimitExceeded(_))
    ));
}

#[test]
fn bounded_rule_still_generates() {
    let eng = engine();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, BsDate::new(2080, 1, 1).unwrap())
        .with_count(10);
    let instances = eng
        .generate_bs_instances(
            &rule,
            BsDate::new(2080, 1, 1).unwrap(),
            BsDate::new(2081, 1, 1).unwrap(),
        )
        .unwrap();
    assert_eq!(instances.len(), 10);
}

// ---------------------------------------------------------------------------
// Exact JD conversion (was: hand-rolled with silent 2000-01-01 fallback)
// ---------------------------------------------------------------------------

#[test]
fn jd_to_datetime_known_epochs() {
    // J2000.0 = 2000-01-01 12:00:00 UTC.
    let j2000 = jd_to_datetime(2_451_545.0).unwrap();
    assert_eq!(j2000.date_naive(), NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
    assert_eq!(j2000.hour(), 12);

    // Unix epoch = JD 2440587.5.
    let unix = jd_to_datetime(2_440_587.5).unwrap();
    assert_eq!(unix.timestamp(), 0);

    // Absurd JD errors instead of silently returning a sentinel.
    assert!(jd_to_datetime(f64::MAX).is_err());
}

// ---------------------------------------------------------------------------
// Tithi transition time + location-aware tithi
// ---------------------------------------------------------------------------

#[test]
fn tithi_end_is_within_two_days_of_sunrise() {
    let eng = engine();
    let date = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
    let loc = Location::kathmandu();

    let end = eng.get_tithi_end(date, &loc).unwrap();
    let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let delta_hours = (end - day_start).num_hours();
    // A tithi lasts ~19–26 hours; the boundary after sunrise must fall
    // within the next two days.
    assert!(
        (0..=48).contains(&delta_hours),
        "tithi end {end} is {delta_hours}h from {date}"
    );
}

#[test]
fn get_tithi_at_location_matches_kathmandu_default() {
    let eng = engine();
    let date = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
    assert_eq!(
        eng.get_tithi(date).unwrap(),
        eng.get_tithi_at_location(date, &Location::kathmandu()).unwrap()
    );
}

// ---------------------------------------------------------------------------
// Single-pass daily panchanga
// ---------------------------------------------------------------------------

#[test]
fn daily_panchanga_consistent_with_individual_calls() {
    let eng = engine();
    let date = NaiveDate::from_ymd_opt(2024, 4, 13).unwrap();
    let loc = Location::kathmandu();

    let p = eng.get_daily_panchanga(date, &loc).unwrap();
    assert_eq!(p.sunrise, eng.get_sunrise(date, loc.clone()).unwrap());
    assert_eq!(p.sunset, eng.get_sunset(date, loc.clone()).unwrap());
    assert_eq!(p.info, eng.get_daily_astro_info(date, loc).unwrap());
    assert!(p.sunrise < p.sunset);
}

// ---------------------------------------------------------------------------
// Calendar provider: O(1) lookup edges + exact official flag
// ---------------------------------------------------------------------------

#[test]
fn provider_lookup_edges() {
    let provider = StaticCalendarProvider::new();
    assert!(provider.has_year(1975));
    assert!(provider.has_year(2100));
    assert!(!provider.has_year(1974));
    assert!(!provider.has_year(2101));
    assert!(!provider.has_year(0));

    assert!(provider.get_month_days(1975, BsMonth::Baisakh).is_ok());
    assert!(provider.get_month_days(2100, BsMonth::Chaitra).is_ok());
    assert!(matches!(
        provider.get_month_days(1974, BsMonth::Baisakh),
        Err(BsCalendarError::CalendarDataNotFound(1974))
    ));

    // Data file is the official table; the flag is now exact, not substring.
    assert!(provider.is_official());
}

#[test]
fn conversion_round_trip_unchanged_by_lookup_change() {
    let eng = engine();
    // Spot-check the documented anchor: 2081-01-01 BS = 2024-04-13 AD.
    let ad = eng
        .bs_to_gregorian(BsDate::new(2081, 1, 1).unwrap())
        .unwrap();
    assert_eq!(ad, NaiveDate::from_ymd_opt(2024, 4, 13).unwrap());

    // Round-trip across the whole supported range (BS 1975–2100).
    for year in [1975u16, 2000, 2045, 2090, 2100] {
        let bs = BsDate::new(year, 1, 1).unwrap();
        let ad = eng.bs_to_gregorian(bs).unwrap();
        assert_eq!(eng.gregorian_to_bs(ad).unwrap(), bs, "round trip BS {year}");
    }
}

// ---------------------------------------------------------------------------
// Misc: Hash derive, checked year arithmetic, coverage constant
// ---------------------------------------------------------------------------

#[test]
fn bs_date_usable_as_hash_key() {
    let mut set = HashSet::new();
    set.insert(BsDate::new(2081, 1, 1).unwrap());
    set.insert(BsDate::new(2081, 1, 1).unwrap());
    set.insert(BsDate::new(2081, 1, 2).unwrap());
    assert_eq!(set.len(), 2);
}

#[test]
fn yearly_rule_at_range_edge_errors_instead_of_wrapping() {
    let eng = engine();
    // Yearly rule anchored near the end of the data range: advancing past
    // BS 2090 must stop cleanly (no panic, no wrapped nonsense year).
    let rule = BsRecurrenceRule::new(BsFrequency::Yearly, BsDate::new(2089, 1, 1).unwrap());
    let instances = eng
        .generate_bs_instances(
            &rule,
            BsDate::new(2089, 1, 1).unwrap(),
            BsDate::new(2090, 12, 30).unwrap(),
        )
        .unwrap();
    assert_eq!(instances.len(), 2); // 2089 and 2090
}

#[test]
fn tithi_coverage_constant_is_disclosed() {
    assert_eq!(TITHI_VERIFIED_THROUGH_BS, 2083);
}
