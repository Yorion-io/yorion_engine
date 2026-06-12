// Core API tests — engine construction, event-instance generation, exception flagging.
// Pure conversion tests live in unit_services_conversion.rs.
use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::event::Event;
use yorion_engine::domain::tithi::Location;

mod helpers;
use helpers::*;

#[test]
fn test_engine_creation() {
    let engine = CalendarEngine::new();
    drop(engine);
}

#[test]
fn test_event_instances_carry_matching_ad_date() {
    // Every generated EventInstance must carry an ad_date equal to
    // bs_to_gregorian(bs_date). Use a BS-recurring rule (deterministic, no astro).
    let engine = test_engine();

    let json = r#"{
        "id": "evt-1",
        "title": "Monthly BS Event",
        "recurrence": "X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20800101;BYMONTHDAY=1"
    }"#;
    let event: Event = serde_json::from_str(json).unwrap();

    let start = bs_date(2080, 1, 1);
    let end = bs_date(2080, 6, 1);

    let instances = engine
        .generate_event_instances(vec![event], start, end, Location::kathmandu())
        .unwrap();

    assert!(!instances.is_empty(), "expected at least one instance");
    for inst in instances {
        let expected_ad = engine.bs_to_gregorian(inst.bs_date).unwrap();
        assert_eq!(
            inst.ad_date, expected_ad,
            "ad_date must equal bs_to_gregorian(bs_date) for {}",
            inst.bs_date
        );
    }
}

#[test]
fn test_event_instances_flag_calendar_clamp_as_exception() {
    // A1: when the calendar clamps a non-existent target day onto the last valid
    // day, the resulting EventInstance must be flagged is_exception=true with
    // original_date = the intended (un-clamped) BS date. Non-clamped instances must
    // stay is_exception=false / original_date=None.
    //
    // BYMONTHDAY=30 across BS 2080:
    //   - Bhadra (month 8) has 30 days → day 30 exists → NOT clamped.
    //   - Poush  (month 9) has 29 days → day 30 clamps to 29 → flagged.
    let engine = test_engine();

    let json = r#"{
        "id": "evt-clamp",
        "title": "Monthly 30th",
        "recurrence": "X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20800801;BYMONTHDAY=30"
    }"#;
    let event: Event = serde_json::from_str(json).unwrap();

    let start = bs_date(2080, 8, 1);
    // Poush has only 29 days, so the range end is its last real day.
    let end = bs_date(2080, 9, 29);

    let instances = engine
        .generate_event_instances(vec![event], start, end, Location::kathmandu())
        .unwrap();

    let bhadra = instances
        .iter()
        .find(|i| i.bs_date == bs_date(2080, 8, 30))
        .expect("expected a Bhadra-30 instance");
    assert!(
        !bhadra.is_exception,
        "Bhadra 30 exists → must not be flagged as an exception"
    );
    assert_eq!(
        bhadra.original_date, None,
        "non-clamped instance must have no original_date"
    );

    // Poush has 29 days, so day 30 clamps onto day 29.
    let poush = instances
        .iter()
        .find(|i| i.bs_date == bs_date(2080, 9, 29))
        .expect("expected a clamped Poush-29 instance");
    assert!(
        poush.is_exception,
        "Poush 29 is a clamp of the intended day 30 → must be flagged"
    );
    assert_eq!(
        poush.original_date,
        Some(bs_date(2080, 9, 30)),
        "clamped instance must record the intended (un-clamped) BS date"
    );
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
