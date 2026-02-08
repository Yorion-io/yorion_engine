use bs_calendar_core::domain::event::{Event, EventInstance};
use bs_calendar_core::domain::recurrence::Recurrence;
use bs_calendar_core::domain::{BsDate, BsMonth, Location};
use bs_calendar_core::prelude::*;
use serde_json::json;

#[test]
fn test_end_to_end_event_generation_from_json() {
    // 1. Simulate frontend sending JSON with RRULE strings
    let events_json = json!([
        {
            "id": "evt-bs",
            "title": "Monthly BS Event",
             // Standard BS RRULE
            "recurrence": "FREQ=MONTHLY;DTSTART=20800101;BYMONTHDAY=1;X-CALENDAR=BS"
        },
        {
            "id": "evt-tithi",
            "title": "Ekadashi Event",
            // Tithi RRULE
            "recurrence": "FREQ=MONTHLY;DTSTART=20800101;X-TITHI=SHUKLA EKADASHI;X-CALENDAR=BS"
        }
    ]);

    let json_str = events_json.to_string();

    // 2. Deserialize (mimicking WASM boundary)
    let events: Vec<Event> = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert_eq!(events.len(), 2);

    // Default location for Tithi calculations
    let location = Location {
        latitude: 27.7172,
        longitude: 85.3240,
        name: "Kathmandu",
        timezone_offset_mins: 345, // +5:45
        follow_nepal_social_calendar: true,
    };

    // 3. Setup Engine (Manually since we can't access WASM's get_engine easily from integration tests without exposing it)
    // We will use the service logic directly as get_month_events does

    let provider = std::sync::Arc::new(StaticCalendarProvider::new());
    let override_provider = Box::new(StaticTithiOverrideProvider::new());
    let astronomical_service =
        std::sync::Arc::new(AstronomicalService::with_overrides(override_provider));
    let conversion_service = std::sync::Arc::new(ConversionService::new(provider));
    let time_provider = std::sync::Arc::new(bs_calendar_core::adapters::SystemTimeProvider); // Use system time for test

    let instance_generator = InstanceGenerator::new(conversion_service.clone());
    let tithi_instance_generator = TithiInstanceGenerator::new(
        conversion_service.clone(),
        astronomical_service.clone(),
        time_provider,
    );

    // 4. Generate Instances for a specific month (e.g., Falgun 2080)
    let year = 2080;
    let month = 11; // Falgun

    let bs_month = BsMonth::from_u8(month).unwrap();
    let start_date = BsDate::new(year, month, 1).unwrap();
    let days_in_month = conversion_service
        .calendar()
        .get_month_days(year, bs_month)
        .unwrap();
    let end_date = BsDate::new(year, month, days_in_month).unwrap();

    // let start_ad = conversion_service.bs_to_gregorian(start_date).unwrap();
    // let end_ad = conversion_service.bs_to_gregorian(end_date).unwrap();

    let mut all_instances = Vec::new();
    let version = bs_calendar_core::domain::CalendarVersion::official("v1".to_string());

    for event in events {
        match event.recurrence {
            Recurrence::Bs(rule) => {
                let insts = instance_generator
                    .generate_bs_instances(&rule, start_date, end_date)
                    .unwrap();
                for bs in insts {
                    all_instances.push(EventInstance::from_recurrence(
                        format!("{}-{}", event.id, bs.format()),
                        bs,
                        event.title.clone(),
                        version.clone(),
                        event.id.clone(),
                    ));
                }
            }
            Recurrence::Tithi(rule) => {
                let insts = tithi_instance_generator
                    .generate_instances(
                        &event.id,
                        &event.title,
                        &rule,
                        start_date,
                        end_date,
                        version.clone(),
                        location.clone(),
                    )
                    .unwrap();
                all_instances.extend(insts);
            }
            _ => {}
        }
    }

    // 5. Verify Results

    // Check BS Event Instance
    let bs_instance = all_instances
        .iter()
        .find(|i| i.parent_event_id.as_deref() == Some("evt-bs"));
    assert!(
        bs_instance.is_some(),
        "Should have generated BS event instance"
    );
    assert_eq!(
        bs_instance.unwrap().bs_date.day,
        1,
        "BS event should be on day 1"
    );

    // Check Tithi Event Instance (Ekadashi in Falgun 2080)
    let tithi_instance = all_instances
        .iter()
        .find(|i| i.parent_event_id.as_deref() == Some("evt-tithi"));
    assert!(
        tithi_instance.is_some(),
        "Should have generated Tithi event instance"
    );
}
