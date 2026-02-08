// AD Recurrence Tests
// Verifying standard RFC 5545 behavior via the InstanceGenerator
use bs_calendar_core::domain::recurrence::AdRecurrenceRule;
use bs_calendar_core::services::{ConversionService, InstanceGenerator};
use chrono::NaiveDate;
use std::sync::Arc;

mod helpers;
use helpers::TestCalendarProvider;

fn create_generator() -> InstanceGenerator {
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));
    InstanceGenerator::new(Arc::new(conversion))
}

#[test]
fn test_ad_daily_recurrence() {
    let generator = create_generator();

    // Daily for 5 days
    let rule =
        AdRecurrenceRule::new("DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;COUNT=5".to_string())
            .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

    let _instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    // Should start from start date because DTSTART is essentially implied as start by rrule crate default behavior
    // if not specified, OR we need to be careful. The rrule crate usually requires DTSTART in the string
    // or set via options. AdRecurrenceRule wraps just the string.
    // Let's verify how AdRecurrenceRule parses. It uses RRuleSet::from_str.
    // If DTSTART is missing, rrule crate might default or error.
    // Let's include DTSTART to be explicit and safe.

    let rule =
        AdRecurrenceRule::new("DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;COUNT=5".to_string())
            .unwrap();
    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 5);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_eq!(instances[4], NaiveDate::from_ymd_opt(2024, 1, 5).unwrap());
}

#[test]
fn test_ad_weekly_with_byday() {
    let generator = create_generator();

    // Every Monday and Wednesday
    // Start Jan 1 2024 (Monday)
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=4".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
    let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    // Expected: Jan 1 (Mo), Jan 3 (We), Jan 8 (Mo), Jan 10 (We)
    assert_eq!(instances.len(), 4);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2024, 1, 3).unwrap());
    assert_eq!(instances[2], NaiveDate::from_ymd_opt(2024, 1, 8).unwrap());
    assert_eq!(instances[3], NaiveDate::from_ymd_opt(2024, 1, 10).unwrap());
}

#[test]
fn test_ad_monthly_recurrence() {
    let generator = create_generator();

    // 15th of every month
    let rule =
        AdRecurrenceRule::new("DTSTART:20240115T000000Z\nRRULE:FREQ=MONTHLY;COUNT=3".to_string())
            .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());
    assert_eq!(instances[2], NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
}

#[test]
fn test_ad_yearly_recurrence() {
    let generator = create_generator();

    // Every year on Jan 1
    let rule =
        AdRecurrenceRule::new("DTSTART:20240101T000000Z\nRRULE:FREQ=YEARLY;COUNT=3".to_string())
            .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2030, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    assert_eq!(instances[2], NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
}

#[test]
fn test_ad_until_date() {
    let generator = create_generator();

    // Daily until Jan 5
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;UNTIL=20240105T000000Z".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 5);
    assert_eq!(
        instances.last().unwrap(),
        &NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()
    );
}
