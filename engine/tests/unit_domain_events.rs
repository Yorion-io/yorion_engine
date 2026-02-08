// Domain Event Tests
use bs_calendar_core::domain::bs_date::BsDate;
use bs_calendar_core::domain::event::{CalendarVersion, Event, EventInstance};
use bs_calendar_core::domain::recurrence::{BsFrequency, Recurrence};
use bs_calendar_core::domain::tithi::Tithi;

mod helpers;

#[test]
fn test_create_event_instance() {
    let bs_date = BsDate::new(2080, 1, 15).unwrap();
    let version = CalendarVersion::official("2080".to_string());

    let instance = EventInstance::new(
        "evt-1".to_string(),
        bs_date,
        "Test Event".to_string(),
        version.clone(),
    );

    assert_eq!(instance.id, "evt-1");
    assert_eq!(instance.bs_date, bs_date);
    assert_eq!(instance.title, "Test Event");
    assert_eq!(instance.calendar_version, version);
    assert!(!instance.is_exception);
    assert!(!instance.is_cancelled);
}

#[test]
fn test_recurring_instance() {
    let bs_date = BsDate::new(2080, 1, 15).unwrap();
    let version = CalendarVersion::official("2080".to_string());

    let instance = EventInstance::from_recurrence(
        "evt-1-occ-1".to_string(),
        bs_date,
        "Recurring Event".to_string(),
        version,
        "evt-1".to_string(),
    );

    assert_eq!(instance.parent_event_id, Some("evt-1".to_string()));
}

#[test]
fn test_tithi_instance() {
    let bs_date = BsDate::new(2080, 1, 15).unwrap();
    let version = CalendarVersion::official("2080".to_string());

    let instance = EventInstance::new(
        "evt-1".to_string(),
        bs_date,
        "Ekadashi".to_string(),
        version,
    )
    .with_tithi(Tithi::ShuklaEkadashi);

    assert_eq!(instance.tithi, Some(Tithi::ShuklaEkadashi));
}

#[test]
fn test_exception_instance() {
    let bs_date = BsDate::new(2080, 1, 15).unwrap();
    let original_date = BsDate::new(2080, 1, 14).unwrap();
    let version = CalendarVersion::official("2080".to_string());

    let instance = EventInstance::new(
        "evt-1".to_string(),
        bs_date,
        "Moved Event".to_string(),
        version,
    )
    .as_exception(original_date);

    assert!(instance.is_exception);
    assert_eq!(instance.original_date, Some(original_date));
}

#[test]
fn test_needs_reconciliation() {
    let bs_date = BsDate::new(2080, 1, 15).unwrap();
    let projected_version = CalendarVersion::projected("2080-proj".to_string());
    let official_version = CalendarVersion::official("2080".to_string());

    let instance = EventInstance::new(
        "evt-1".to_string(),
        bs_date,
        "Event".to_string(),
        projected_version.clone(),
    );

    assert!(instance.needs_reconciliation(&official_version));
    assert!(!instance.needs_reconciliation(&projected_version));
}

#[test]
fn test_deserialize_bs_rrule() {
    let json = r#"{
        "id": "evt-1",
        "title": "Monthly Event",
        "recurrence": "FREQ=MONTHLY;DTSTART=20800101;BYMONTHDAY=1,15;X-CALENDAR=BS"
    }"#;

    let event: Event = serde_json::from_str(json).unwrap();
    assert_eq!(event.id, "evt-1");

    match event.recurrence {
        Recurrence::Bs(rule) => {
            assert_eq!(rule.frequency, BsFrequency::Monthly);
            assert_eq!(rule.anchor.year, 2080);
        }
        _ => panic!("Expected BS recurrence"),
    }
}

#[test]
fn test_deserialize_tithi_rrule() {
    let json = r#"{
        "id": "evt-2",
        "title": "Every Ekadashi",
        "recurrence": "FREQ=MONTHLY;DTSTART=20800101;X-TITHI=SHUKLA EKADASHI;X-SKIPADHIK=TRUE;X-CALENDAR=BS"
    }"#;

    let event: Event = serde_json::from_str(json).unwrap();

    match event.recurrence {
        Recurrence::Tithi(rule) => {
            assert_eq!(rule.by_tithi[0], Tithi::ShuklaEkadashi);
            assert!(rule.skip_adhik);
        }
        _ => panic!("Expected Tithi recurrence"),
    }
}
