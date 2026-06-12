// AD Recurrence Tests
// Verifying standard RFC 5545 behavior via the InstanceGenerator
use yorion_engine::domain::recurrence::AdRecurrenceRule;
use yorion_engine::services::{ConversionService, InstanceGenerator};
use chrono::{Datelike, NaiveDate, Weekday};
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

// ============================================================================
// Scenario coverage: INTERVAL for non-weekly frequencies
// ============================================================================

#[test]
fn test_ad_monthly_interval_every_2_months() {
    let generator = create_generator();

    // Every 2 months on the 10th, 4 times.
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240110T000000Z\nRRULE:FREQ=MONTHLY;INTERVAL=2;COUNT=4".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 4);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 10).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2024, 3, 10).unwrap());
    assert_eq!(instances[2], NaiveDate::from_ymd_opt(2024, 5, 10).unwrap());
    assert_eq!(instances[3], NaiveDate::from_ymd_opt(2024, 7, 10).unwrap());
}

#[test]
fn test_ad_yearly_interval_every_2_years() {
    let generator = create_generator();

    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=YEARLY;INTERVAL=2;COUNT=3".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2032, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    assert_eq!(instances[2], NaiveDate::from_ymd_opt(2028, 1, 1).unwrap());
}

// ============================================================================
// Scenario coverage: BYMONTHDAY and BYMONTH+BYDAY combinations
// ============================================================================

#[test]
fn test_ad_monthly_bymonthday() {
    let generator = create_generator();

    // 1st and 15th of every month.
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=MONTHLY;BYMONTHDAY=1,15;COUNT=4".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 4);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
    assert_eq!(instances[2], NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    assert_eq!(instances[3], NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());
}

#[test]
fn test_ad_yearly_bymonth_byday_combo() {
    let generator = create_generator();

    // Every Friday in June and December.
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=YEARLY;BYMONTH=6,12;BYDAY=FR".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    // June 2024 Fridays: 7,14,21,28 (4). December 2024 Fridays: 6,13,20,27 (4).
    assert_eq!(instances.len(), 8);
    assert!(instances
        .iter()
        .all(|d| d.month() == 6 || d.month() == 12));
    assert!(instances.iter().all(|d| d.weekday() == Weekday::Fri));
}

// ============================================================================
// Scenario coverage: leap-day, COUNT-vs-UNTIL, empty range, range-start offset
// ============================================================================

#[test]
fn test_ad_yearly_leap_day_feb29_only_on_leap_years() {
    let generator = create_generator();

    // Yearly on Feb 29: only leap years actually have it (RFC 5545: invalid dates skipped).
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240229T000000Z\nRRULE:FREQ=YEARLY;COUNT=2".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2032, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    // 2024 and 2028 are leap years; 2025-2027 are skipped.
    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0], NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    assert_eq!(instances[1], NaiveDate::from_ymd_opt(2028, 2, 29).unwrap());
}

#[test]
fn test_ad_count_takes_effect_before_until() {
    let generator = create_generator();

    // COUNT=3 with a far-future UNTIL: COUNT bounds first → 3 instances.
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;COUNT=3;UNTIL=20240131T000000Z".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances.last().unwrap(), &NaiveDate::from_ymd_opt(2024, 1, 3).unwrap());
}

#[test]
fn test_ad_empty_when_range_outside_occurrences() {
    let generator = create_generator();

    // Weekly Mondays in Jan, but query a March window → empty.
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=4".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert!(instances.is_empty());
}

#[test]
fn test_ad_range_start_after_dtstart_skips_earlier() {
    let generator = create_generator();

    // Daily from Jan 1, but query starting Jan 10 → first instance is Jan 10.
    let rule = AdRecurrenceRule::new(
        "DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;COUNT=30".to_string(),
    )
    .unwrap();
    let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

    let instances = generator.generate_ad_instances(&rule, start, end).unwrap();

    assert_eq!(instances.first().unwrap(), &NaiveDate::from_ymd_opt(2024, 1, 10).unwrap());
    assert_eq!(instances.last().unwrap(), &NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
}
