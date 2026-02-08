// Services Instance Generator Tests
use bs_calendar_core::domain::bs_date::{BsDate, BsMonth};
use bs_calendar_core::domain::recurrence::{BsFrequency, BsRecurrenceRule};
use bs_calendar_core::services::{ConversionService, InstanceGenerator};
use std::sync::Arc;

mod helpers;
use helpers::TestCalendarProvider;

fn create_test_generator() -> InstanceGenerator {
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));
    InstanceGenerator::new(Arc::new(conversion))
}

#[test]
fn test_daily_recurrence() {
    let generator = create_test_generator();

    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_count(5);

    let start = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2080, 1, 10).unwrap();

    let instances = generator.generate_bs_instances(&rule, start, end).unwrap();
    assert_eq!(instances.len(), 5);
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(instances[4], BsDate::new(2080, 1, 5).unwrap());
}

#[test]
fn test_monthly_recurrence() {
    let generator = create_test_generator();

    let anchor = BsDate::new(2080, 1, 15).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor).with_count(3);

    let start = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2080, 12, 30).unwrap();

    let instances = generator.generate_bs_instances(&rule, start, end).unwrap();
    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0].month, BsMonth::Baisakh);
    assert_eq!(instances[1].month, BsMonth::Jestha);
    assert_eq!(instances[2].month, BsMonth::Ashadh);
}

#[test]
fn test_yearly_recurrence() {
    let generator = create_test_generator();

    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Yearly, anchor).with_count(3);

    let start = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2083, 1, 1).unwrap();

    let instances = generator.generate_bs_instances(&rule, start, end).unwrap();
    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0].year, 2080);
    assert_eq!(instances[1].year, 2081);
    assert_eq!(instances[2].year, 2082);
}

#[test]
fn test_bymonth_filter() {
    let generator = create_test_generator();

    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month(vec![BsMonth::Baisakh, BsMonth::Ashadh])
        .with_count(4);

    let start = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2081, 12, 30).unwrap();

    let instances = generator.generate_bs_instances(&rule, start, end).unwrap();

    // Should only get Baisakh and Ashadh months
    for instance in &instances {
        assert!(instance.month == BsMonth::Baisakh || instance.month == BsMonth::Ashadh);
    }
}

#[test]
fn test_bymonthday_filter_with_clamping() {
    let generator = create_test_generator();

    // Request day 31, which should clamp to actual month days
    let anchor = BsDate::new(2080, 1, 31).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month_day(vec![31])
        .with_count(3);

    let start = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2080, 12, 30).unwrap();

    let instances = generator.generate_bs_instances(&rule, start, end).unwrap();

    // Each instance should be on day 31 or clamped to month end
    for instance in &instances {
        assert!(instance.day >= 29); // At least close to end of month
    }
}
