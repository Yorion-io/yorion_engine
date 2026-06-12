use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::event::Event;
use yorion_engine::domain::{BsDate, BsMonth, Location};
use yorion_engine::prelude::*;
use serde_json::json;

#[test]
fn test_end_to_end_event_generation_from_json() {
    let events_json = json!([
        {
            "id": "evt-bs",
            "title": "Monthly BS Event",
            "recurrence": "X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20800101;BYMONTHDAY=1"
        },
        {
            "id": "evt-tithi",
            "title": "Ekadashi Event",
            "recurrence": "X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20800101;X-TITHI=SHUKLA EKADASHI"
        }
    ]);

    let events: Vec<Event> =
        serde_json::from_str(&events_json.to_string()).expect("Failed to parse JSON");
    assert_eq!(events.len(), 2);

    let location = Location::kathmandu();

    let engine = CalendarEngine::new();

    let year = 2080;
    let month = 11; // Falgun
    let bs_month = BsMonth::from_u8(month).unwrap();
    let start_date = BsDate::new(year, month, 1).unwrap();
    let days_in_month = engine.calendar().get_month_days(year, bs_month).unwrap();
    let end_date = BsDate::new(year, month, days_in_month).unwrap();

    let all_instances = engine
        .generate_event_instances(events, start_date, end_date, location)
        .unwrap();

    let bs_instance = all_instances
        .iter()
        .find(|i| i.parent_event_id.as_deref() == Some("evt-bs"));
    assert!(bs_instance.is_some(), "Should have generated BS event instance");
    assert_eq!(bs_instance.unwrap().bs_date.day, 1, "BS event should be on day 1");

    let tithi_instance = all_instances
        .iter()
        .find(|i| i.parent_event_id.as_deref() == Some("evt-tithi"));
    assert!(
        tithi_instance.is_some(),
        "Should have generated Tithi event instance"
    );
}

#[test]
fn test_direct_service_construction() {
    // Verify that direct service construction (without CalendarEngine) still works
    // for callers who need finer control.
    let provider = std::sync::Arc::new(StaticCalendarProvider::new());
    let conversion_service = std::sync::Arc::new(ConversionService::new(provider));
    let override_provider = Box::new(StaticTithiOverrideProvider::new());
    let astronomical_service =
        std::sync::Arc::new(AstronomicalService::with_overrides(override_provider));

    let instance_generator = InstanceGenerator::new(conversion_service.clone());
    let tithi_generator =
        TithiInstanceGenerator::new(conversion_service.clone(), astronomical_service);

    let rule: BsRecurrenceRule = BsRecurrenceRule::from_rrule(
        "X-CALENDAR=BS;FREQ=MONTHLY;DTSTART=20800101;BYMONTHDAY=1",
    )
    .unwrap();

    let start = BsDate::new(2080, 11, 1).unwrap();
    let end = BsDate::new(2080, 11, 30).unwrap();
    let instances = instance_generator.generate_bs_instances(&rule, start, end).unwrap();
    assert!(!instances.is_empty());

    // Suppress unused warning — tithi_generator construction is the point of this test
    let _ = tithi_generator;
}
