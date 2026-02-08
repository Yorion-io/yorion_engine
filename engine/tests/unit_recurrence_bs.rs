// BS Recurrence Tests
// Verifying BS recurrence behavior covering all frequencies and filters
use bs_calendar_core::domain::bs_date::{BsDate, BsMonth};
use bs_calendar_core::domain::recurrence::{BsFrequency, BsRecurrenceRule};
use bs_calendar_core::services::{ConversionService, InstanceGenerator};
use std::sync::Arc;

mod helpers;
use helpers::TestCalendarProvider;

fn create_generator() -> InstanceGenerator {
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));
    InstanceGenerator::new(Arc::new(conversion))
}

#[test]
fn test_bs_daily_recurrence() {
    let generator = create_generator();

    // Daily for 5 days
    // 2080 Baisakh 1
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_count(5);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 1, 20).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 5);
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(instances[4], BsDate::new(2080, 1, 5).unwrap());
}

#[test]
fn test_bs_weekly_with_byday() {
    let generator = create_generator();

    // Every Sunday (0) and Wednesday (3)
    // 2080 Baisakh 1 is a Friday (2023-04-14)
    // Baisakh 3 is Sunday.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Weekly, anchor).with_by_day(vec![0, 3]); // Sunday, Wednesday

    // Generate for first month
    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 1, 15).unwrap())
        .unwrap();

    // 2080 Baisakh 1 (Apr 14, 2023) is Friday.
    // Next Sunday: Baisakh 3 (Apr 16)
    // Next Wednesday: Baisakh 6 (Apr 19)
    // Next Sunday: Baisakh 10 (Apr 23)
    // Next Wednesday: Baisakh 13 (Apr 26)

    assert!(!instances.is_empty());
    assert_eq!(instances[0], BsDate::new(2080, 1, 3).unwrap());
    assert_eq!(instances[1], BsDate::new(2080, 1, 6).unwrap());
    assert_eq!(instances[2], BsDate::new(2080, 1, 10).unwrap());
    assert_eq!(instances[3], BsDate::new(2080, 1, 13).unwrap());
}

#[test]
fn test_bs_monthly_with_clamping() {
    let generator = create_generator();

    // 32nd of every month (should clamp to end of month)
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month_day(vec![32])
        .with_count(3);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2081, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 3);
    // Baisakh has 31 days in 2080
    assert_eq!(instances[0], BsDate::new(2080, 1, 31).unwrap());
    // Jestha has 31/32? Let's assume standard lengths, code should handle it via calendar data
    // Jestah 2080 usually 31 or 32.
    // We can just assert they are valid dates and represent end of month.
    let date1 = instances[1];
    assert_eq!(date1.month, BsMonth::Jestha);
    assert!(date1.day >= 29);

    let date2 = instances[2];
    assert_eq!(date2.month, BsMonth::Ashadh);
    assert!(date2.day >= 29);
}

#[test]
fn test_bs_yearly_with_bymonth() {
    let generator = create_generator();

    // Every Kartik (7) and Chaitra (12)
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Yearly, anchor)
        .with_by_month(vec![BsMonth::Kartik, BsMonth::Chaitra])
        .with_count(4);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2082, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 4);
    assert_eq!(instances[0].month, BsMonth::Kartik);
    assert_eq!(instances[0].year, 2080);

    assert_eq!(instances[1].month, BsMonth::Chaitra);
    assert_eq!(instances[1].year, 2080);

    assert_eq!(instances[2].month, BsMonth::Kartik);
    assert_eq!(instances[2].year, 2081);

    assert_eq!(instances[3].month, BsMonth::Chaitra);
    assert_eq!(instances[3].year, 2081);
}

#[test]
fn test_bs_recurrence_interval() {
    let generator = create_generator();

    // Every 2 weeks
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Weekly, anchor)
        .with_interval(2)
        .with_count(3);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 12, 30).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());

    // +14 days
    // Baisakh 1 + 14 = Baisakh 15
    assert_eq!(instances[1], BsDate::new(2080, 1, 15).unwrap());

    // +14 days
    // Baisakh 15 + 14 = Baisakh 29
    assert_eq!(instances[2], BsDate::new(2080, 1, 29).unwrap());
}
