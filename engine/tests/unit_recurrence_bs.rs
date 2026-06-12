// BS Recurrence Tests
// Verifying BS recurrence behavior covering all frequencies and filters
use yorion_engine::domain::bs_date::{BsDate, BsMonth};
use yorion_engine::domain::recurrence::{BsFrequency, BsRecurrenceRule};
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

// ============================================================================
// Scenario coverage: UNTIL, INTERVAL (monthly/yearly), ordering, clamping
// ============================================================================

#[test]
fn test_bs_daily_until_stops_at_until() {
    let generator = create_generator();

    // Daily from Baisakh 1, UNTIL Baisakh 10 → 10 instances, last == UNTIL.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2080, 1, 10).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_until(until);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 2, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 10);
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(*instances.last().unwrap(), until);
}

#[test]
fn test_bs_monthly_interval_every_2_months() {
    let generator = create_generator();

    // Every 2 months on the 1st, 4 times: Baisakh, Ashadh, Bhadra, Kartik (months 1,3,5,7).
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_interval(2)
        .with_count(4);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2081, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 4);
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(instances[1], BsDate::new(2080, 3, 1).unwrap());
    assert_eq!(instances[2], BsDate::new(2080, 5, 1).unwrap());
    assert_eq!(instances[3], BsDate::new(2080, 7, 1).unwrap());
}

#[test]
fn test_bs_yearly_interval_every_2_years() {
    let generator = create_generator();

    // Baisakh 1 every 2 years: 2080, 2082, 2084.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Yearly, anchor)
        .with_interval(2)
        .with_count(3);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2090, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(instances[1], BsDate::new(2082, 1, 1).unwrap());
    assert_eq!(instances[2], BsDate::new(2084, 1, 1).unwrap());
}

#[test]
fn test_bs_monthly_bymonthday_out_of_order_is_ascending() {
    let generator = create_generator();

    // BYMONTHDAY listed descending (15,1). Within each month the engine must
    // emit them ASCENDING (1st then 15th), and COUNT must pick the right ones.
    // Exposes the candidate-ordering bug if days are pushed in rule-list order.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month_day(vec![15, 1])
        .with_count(4);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2081, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 4);
    // Baisakh 1, Baisakh 15, Jestha 1, Jestha 15 — strictly ascending.
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(instances[1], BsDate::new(2080, 1, 15).unwrap());
    assert_eq!(instances[2], BsDate::new(2080, 2, 1).unwrap());
    assert_eq!(instances[3], BsDate::new(2080, 2, 15).unwrap());

    // Defensive: whole output must be sorted ascending.
    let mut sorted = instances.clone();
    sorted.sort();
    assert_eq!(instances, sorted, "instances must be in ascending date order");
}

#[test]
fn test_bs_daily_bymonthday_sentinel_32_is_last_day() {
    let generator = create_generator();

    // Daily freq filtered to BYMONTHDAY=32 (last-day sentinel). Should yield the
    // last day of each month in range. Exposes the Daily no-clamping bug
    // (strict equality day==32 never matches a real day).
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_by_month_day(vec![32]);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 3, 1).unwrap())
        .unwrap();

    // At least Baisakh-end and Jestha-end within the window.
    assert!(
        instances.len() >= 2,
        "expected at least 2 month-end instances, got {}",
        instances.len()
    );
    // Every instance must be the last day of its month (day >= 29 and no later
    // day exists in that month).
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));
    for d in &instances {
        let month_days = conversion.calendar().get_month_days(d.year, d.month).unwrap();
        assert_eq!(
            d.day, month_days,
            "instance {} is not the last day ({}) of its month",
            d, month_days
        );
    }
}

// ============================================================================
// A2: collision after clamp must be deduped within a frame
// ============================================================================

#[test]
fn test_bs_monthly_bymonthday_collision_after_clamp_is_deduped() {
    let generator = create_generator();

    // BS 2080 Poush (month 9) has 29 days. BYMONTHDAY=30,31 both clamp onto day 29.
    // Without dedup this frame would emit day 29 TWICE; with A2 it emits it once.
    let anchor = BsDate::new(2080, 9, 1).unwrap();
    let rule =
        BsRecurrenceRule::new(BsFrequency::Monthly, anchor).with_by_month_day(vec![30, 31]);

    // Window: just the single Poush month.
    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 9, 29).unwrap())
        .unwrap();

    assert_eq!(
        instances,
        vec![BsDate::new(2080, 9, 29).unwrap()],
        "BYMONTHDAY=30,31 in a 29-day month must yield exactly one (deduped) instance"
    );
}

// ============================================================================
// BYMONTH filter: Monthly rule restricted to specific months
// ============================================================================

#[test]
fn test_bs_monthly_with_bymonth_filter() {
    let generator = create_generator();

    // Monthly on the 1st, restricted to Baisakh (month 1) and Bhadra (month 5) only.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month(vec![BsMonth::Baisakh, BsMonth::Bhadra])
        .with_count(4);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2082, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 4);
    for d in &instances {
        assert!(
            d.month == BsMonth::Baisakh || d.month == BsMonth::Bhadra,
            "instance {} must be in Baisakh or Bhadra",
            d
        );
    }
    // Must interleave years: 2080 Baisakh, 2080 Bhadra, 2081 Baisakh, 2081 Bhadra
    assert_eq!(instances[0], BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(instances[1], BsDate::new(2080, 5, 1).unwrap());
    assert_eq!(instances[2], BsDate::new(2081, 1, 1).unwrap());
    assert_eq!(instances[3], BsDate::new(2081, 5, 1).unwrap());
}

// ============================================================================
// Year boundary: daily recurrence spanning Chaitra→Baisakh produces no gap
// ============================================================================

#[test]
fn test_bs_daily_spans_year_boundary_without_gap() {
    let generator = create_generator();
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));

    // Start 5 days before the end of BS 2081 Chaitra and run 10 days.
    let chaitra_days = conversion
        .calendar()
        .get_month_days(2081, BsMonth::Chaitra)
        .unwrap();
    let start = BsDate::new(2081, 12, chaitra_days - 4).unwrap();
    let anchor = start;
    let end = BsDate::new(2082, 1, 10).unwrap();

    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_count(10);
    let instances = generator.generate_bs_instances(&rule, anchor, end).unwrap();

    assert_eq!(instances.len(), 10, "daily COUNT=10 must yield exactly 10");

    // Verify consecutive AD dates across the BS year boundary.
    let mut prev_ad: Option<NaiveDate> = None;
    for bs in &instances {
        let ad = conversion.bs_to_gregorian(*bs).unwrap();
        if let Some(p) = prev_ad {
            assert_eq!(
                (ad - p).num_days(),
                1,
                "gap at year boundary: {} -> {}",
                p,
                ad
            );
        }
        prev_ad = Some(ad);
    }
}

// ============================================================================
// Conversion fidelity: BS→AD day-of-week must be correct (underpins BYDAY)
// ============================================================================

#[test]
fn test_bs_to_ad_day_of_week_fidelity() {
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));

    // Known anchors (BS date, expected Gregorian date, expected weekday).
    // 2080 Baisakh 1 == 2023-04-14 (Friday) — relied on by the weekly BYDAY tests.
    let cases: &[(BsDate, NaiveDate, Weekday)] = &[
        (
            BsDate::new(2080, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 4, 14).unwrap(),
            Weekday::Fri,
        ),
        (
            BsDate::new(2080, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2023, 4, 16).unwrap(),
            Weekday::Sun,
        ),
    ];

    for (bs, expected_ad, expected_wd) in cases {
        let ad = conversion.bs_to_gregorian(*bs).unwrap();
        assert_eq!(ad, *expected_ad, "BS {} converted to wrong AD date", bs);
        assert_eq!(
            ad.weekday(),
            *expected_wd,
            "BS {} ({}) has wrong weekday",
            bs,
            ad
        );
    }
}

// ============================================================================
// Single-weekday BYDAY filter
// ============================================================================

#[test]
fn test_bs_weekly_single_byday() {
    let generator = create_generator();

    // Every Saturday (6) from 2080 Baisakh 1.
    // 2080-01-01 = 2023-04-14 (Friday) → first Saturday is Baisakh 2.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Weekly, anchor)
        .with_by_day(vec![6])
        .with_count(3);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2080, 2, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 3);
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));
    for d in &instances {
        let ad = conversion.bs_to_gregorian(*d).unwrap();
        assert_eq!(
            ad.weekday(),
            Weekday::Sat,
            "instance {} (AD {}) is not a Saturday",
            d,
            ad
        );
    }
}

// ============================================================================
// BYMONTH + BYMONTHDAY combined filter
// ============================================================================

#[test]
fn test_bs_monthly_bymonth_and_bymonthday_combined() {
    let generator = create_generator();

    // Monthly on the 1st, only in Baisakh (1) and Ashadh (3).
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month(vec![BsMonth::Baisakh, BsMonth::Ashadh])
        .with_by_month_day(vec![1])
        .with_count(4);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2082, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 4);
    for d in &instances {
        assert!(
            d.month == BsMonth::Baisakh || d.month == BsMonth::Ashadh,
            "instance {} must be in Baisakh or Ashadh",
            d
        );
        assert_eq!(d.day, 1, "instance {} must be on the 1st", d);
    }
}

// ============================================================================
// COUNT terminates before UNTIL
// ============================================================================

#[test]
fn test_bs_count_terminates_before_until() {
    let generator = create_generator();

    // COUNT=3 with a far-future UNTIL: COUNT must stop after 3 regardless.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2085, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_count(3)
        .with_until(until);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2090, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 3, "COUNT=3 must stop after 3 instances");
    assert!(
        *instances.last().unwrap() <= until,
        "last instance must not exceed UNTIL"
    );
}

// ============================================================================
// UNTIL terminates before COUNT
// ============================================================================

#[test]
fn test_bs_until_terminates_before_count() {
    let generator = create_generator();

    // COUNT=100 but UNTIL after 2 months: only 2 instances.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2080, 2, 28).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_count(100)
        .with_until(until);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2090, 1, 1).unwrap())
        .unwrap();

    assert!(
        instances.len() <= 2,
        "UNTIL bounds must stop expansion even when COUNT is large, got {}",
        instances.len()
    );
    for d in &instances {
        assert!(d <= &until, "instance {} exceeds UNTIL {}", d, until);
    }
}

// ============================================================================
// INTERVAL=3 weekly
// ============================================================================

#[test]
fn test_bs_weekly_interval_3() {
    let generator = create_generator();

    // Every 3 weeks, 3 occurrences → 0, 21, 42 days from anchor.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Weekly, anchor)
        .with_interval(3)
        .with_count(3);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2081, 1, 1).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 3);
    let provider = TestCalendarProvider::new();
    let conversion = ConversionService::new(Arc::new(provider));
    let ad0 = conversion.bs_to_gregorian(instances[0]).unwrap();
    let ad1 = conversion.bs_to_gregorian(instances[1]).unwrap();
    let ad2 = conversion.bs_to_gregorian(instances[2]).unwrap();
    assert_eq!((ad1 - ad0).num_days(), 21, "second instance must be 21 AD days after first");
    assert_eq!((ad2 - ad1).num_days(), 21, "third instance must be 21 AD days after second");
}

// ============================================================================
// Yearly with no filter: anchor day recurs exactly once per year
// ============================================================================

#[test]
fn test_bs_yearly_no_filter_repeats_anchor_day() {
    let generator = create_generator();

    // Yearly on Baisakh 14, no BYMONTH/BYMONTHDAY override.
    let anchor = BsDate::new(2080, 1, 14).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Yearly, anchor).with_count(4);

    let instances = generator
        .generate_bs_instances(&rule, anchor, BsDate::new(2084, 1, 30).unwrap())
        .unwrap();

    assert_eq!(instances.len(), 4);
    for d in &instances {
        assert_eq!(d.month, BsMonth::Baisakh, "month must stay Baisakh");
        assert_eq!(d.day, 14, "day must stay 14");
    }
    assert_eq!(instances[0].year, 2080);
    assert_eq!(instances[1].year, 2081);
    assert_eq!(instances[2].year, 2082);
    assert_eq!(instances[3].year, 2083);
}
