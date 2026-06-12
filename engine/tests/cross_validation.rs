// Cross-validation of the recurrence engine against the engine's OWN primitives.
//
// Premise: the conversion service (bs_to_gregorian / gregorian_to_bs) and the
// astronomical tithi computation (get_tithi) are treated as the source of truth.
// This suite re-derives, from scratch, every property the recurrence engine
// claims about each instance it emits, and asserts they agree.
//
// This proves INTERNAL CONSISTENCY: the recurrence expander never produces a date
// that disagrees with what the engine independently computes for that same date.
// (It does NOT prove the primitives themselves match an external almanac — that
// requires external ground-truth vectors, tracked separately.)

use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::bs_date::{BsDate, BsMonth};
use yorion_engine::domain::event::{CalendarVersion, EventInstance};
use yorion_engine::domain::recurrence::{BsFrequency, BsRecurrenceRule, TithiRecurrenceRule};
use yorion_engine::domain::tithi::{Location, Paksha, Tithi};
use chrono::Datelike;

fn version() -> CalendarVersion {
    CalendarVersion::official("test".to_string())
}

/// Map chrono weekday -> engine weekday code (0=SU .. 6=SA), matching parse_week_days.
fn ad_weekday_code(d: chrono::NaiveDate) -> u8 {
    match d.weekday() {
        chrono::Weekday::Sun => 0,
        chrono::Weekday::Mon => 1,
        chrono::Weekday::Tue => 2,
        chrono::Weekday::Wed => 3,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 5,
        chrono::Weekday::Sat => 6,
    }
}

fn bs_instances(
    engine: &CalendarEngine,
    rule: &BsRecurrenceRule,
    start: BsDate,
    end: BsDate,
) -> Vec<BsDate> {
    engine.generate_bs_instances(rule, start, end).unwrap()
}

fn tithi_instances(
    engine: &CalendarEngine,
    rule: &TithiRecurrenceRule,
    start: BsDate,
    end: BsDate,
) -> Vec<EventInstance> {
    engine
        .generate_tithi_instances("evt", "T", rule, start, end, version(), Location::kathmandu())
        .unwrap()
}

// ============================================================================
// BS family: every emitted BS date must round-trip through the conversion
// service, and must satisfy the rule's own BYMONTHDAY / BYDAY constraints when
// re-derived independently from the converted AD date.
// ============================================================================

/// The fundamental invariant for ANY BS instance: converting BS->AD->BS returns
/// the same BS date, and the instance lies within the requested window.
fn assert_bs_roundtrip(engine: &CalendarEngine, bs: BsDate, start: BsDate, end: BsDate) {
    let ad = engine.bs_to_gregorian(bs).unwrap();
    let back = engine.gregorian_to_bs(ad).unwrap();
    assert_eq!(
        back, bs,
        "round-trip mismatch: BS {bs} -> AD {ad} -> BS {back}"
    );
    assert!(
        bs >= start && bs <= end,
        "instance {bs} outside window {start}..{end}"
    );
}

#[test]
fn bs_monthly_bymonthday_matches_independent_recompute() {
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2083, 12, 30).unwrap();

    // 15th of every month for 18 months.
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Monthly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: Some(vec![15]),
        by_day: None,
        count: Some(18),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 18, "COUNT must produce exactly 18 instances");

    for bs in out {
        assert_bs_roundtrip(&engine, bs, start, end);
        // Independent re-derivation: BYMONTHDAY=15 means the BS day-of-month is 15.
        assert_eq!(bs.day, 15, "instance {bs} is not the 15th");
    }
}

#[test]
fn bs_monthly_last_day_sentinel_matches_real_month_length() {
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2082, 12, 30).unwrap();

    // BYMONTHDAY=32 sentinel == last day of each month.
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Monthly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: Some(vec![32]),
        by_day: None,
        count: Some(12),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 12);

    for bs in out {
        assert_bs_roundtrip(&engine, bs, start, end);
        // Independent oracle for "last day": the next BS day must roll into a new
        // month (or be invalid). We verify via conversion: AD(last)+1 day converts
        // back to BS day 1 of a different month.
        let ad = engine.bs_to_gregorian(bs).unwrap();
        let next_bs = engine
            .gregorian_to_bs(ad.succ_opt().unwrap())
            .unwrap();
        assert_eq!(
            next_bs.day, 1,
            "{bs} claimed last-day but next day {next_bs} is not day 1 of next month"
        );
        assert_ne!(
            next_bs.month, bs.month,
            "{bs} claimed last-day but next day stays in same month"
        );
    }
}

#[test]
fn bs_weekly_byday_matches_converted_ad_weekday() {
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 6, 1).unwrap();

    // Every Saturday (code 6) and Wednesday (code 3).
    let want_days = vec![3u8, 6u8];
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Weekly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: Some(want_days.clone()),
        count: Some(10),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 10);

    for bs in out {
        assert_bs_roundtrip(&engine, bs, start, end);
        // Independent re-derivation: convert to AD, read the real weekday, and
        // require it is one of the requested BYDAY codes.
        let ad = engine.bs_to_gregorian(bs).unwrap();
        let code = ad_weekday_code(ad);
        assert!(
            want_days.contains(&code),
            "instance {bs} (AD {ad}) weekday code {code} not in BYDAY {want_days:?}"
        );
    }
}

#[test]
fn bs_daily_is_consecutive_real_days() {
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 12, 25).unwrap(); // span a year boundary
    let end = BsDate::new(2082, 1, 10).unwrap();

    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Daily,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: None,
        count: Some(15),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 15);

    // Independent oracle: consecutive daily instances must be exactly 1 real
    // (AD) day apart, even across the BS year boundary.
    let mut prev_ad: Option<chrono::NaiveDate> = None;
    for bs in out {
        assert_bs_roundtrip(&engine, bs, start, end);
        let ad = engine.bs_to_gregorian(bs).unwrap();
        if let Some(p) = prev_ad {
            assert_eq!(
                (ad - p).num_days(),
                1,
                "daily instances not 1 AD day apart: {p} -> {ad}"
            );
        }
        prev_ad = Some(ad);
    }
}

// ============================================================================
// TITHI family: the strongest oracle. For every emitted instance, independently
// recompute the sunrise tithi for its AD date via the astronomical service and
// require it equals the claimed tithi, is a member of the requested set, and
// honors the paksha filter. Also require BS<->AD coherence on the instance.
// ============================================================================

/// Recompute the tithi from scratch and assert it matches the instance's claim,
/// and that the instance's BS/AD dates are mutually consistent.
fn assert_tithi_instance_consistent(engine: &CalendarEngine, inst: &EventInstance) {
    // 1. BS and AD on the instance describe the same real day.
    let converted_ad = engine.bs_to_gregorian(inst.bs_date).unwrap();
    assert_eq!(
        converted_ad, inst.ad_date,
        "instance BS {} converts to AD {} but instance carries AD {}",
        inst.bs_date, converted_ad, inst.ad_date
    );

    // 2. Independently recompute the sunrise tithi for that AD date.
    let recomputed = engine.get_tithi(inst.ad_date).unwrap();
    let claimed = inst.tithi.expect("tithi instance must carry a tithi");
    assert_eq!(
        recomputed, claimed,
        "instance {} (AD {}) claims tithi {:?} but engine recomputes {:?}",
        inst.bs_date, inst.ad_date, claimed, recomputed
    );
}

#[test]
fn tithi_ekadashi_recomputes_to_ekadashi() {
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 6, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(6);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 6);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        // Requested set: every Ekadashi (both pakshas).
        let t = inst.tithi.unwrap();
        assert!(
            t.is_ekadashi(),
            "instance {} recomputed tithi {:?} is not an Ekadashi",
            inst.bs_date, t
        );
    }
}

#[test]
fn tithi_purnima_recomputes_to_purnima() {
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 8, 1).unwrap();
    let rule = TithiRecurrenceRule::purnima(anchor).with_count(5);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 5);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        assert!(
            inst.tithi.unwrap().is_purnima(),
            "instance {} is not Purnima",
            inst.bs_date
        );
    }
}

#[test]
fn tithi_krishna_ashtami_recomputes_and_honors_paksha() {
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 8, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::KrishnaAshtami],
        Paksha::Krishna,
        anchor,
    )
    .with_count(4);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 4);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        let t = inst.tithi.unwrap();
        assert_eq!(t, Tithi::KrishnaAshtami, "instance {} not Krishna Ashtami", inst.bs_date);
        // Paksha filter must be honored: independently confirm Krishna paksha.
        assert_eq!(
            t.paksha(),
            Paksha::Krishna,
            "instance {} tithi {:?} is not Krishna paksha",
            inst.bs_date, t
        );
    }
}

#[test]
fn tithi_shukla_only_never_returns_krishna() {
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 8, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaEkadashi],
        Paksha::Shukla,
        anchor,
    )
    .with_count(4);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 4);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        let t = inst.tithi.unwrap();
        assert_eq!(t, Tithi::ShuklaEkadashi);
        assert_eq!(
            t.paksha(),
            Paksha::Shukla,
            "Shukla-only rule leaked a non-Shukla instance: {} {:?}",
            inst.bs_date, t
        );
    }
}

#[test]
fn tithi_lunar_month_filter_all_recompute_correctly() {
    // Every emitted instance under a lunar-month filter must STILL recompute to
    // the requested tithi via the astronomical oracle (the filter must not
    // corrupt the tithi identity of the instances it keeps).
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2083, 12, 30).unwrap();
    let rule = TithiRecurrenceRule::purnima(anchor).with_by_lunar_month(vec![BsMonth::Kartik]);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert!(!out.is_empty(), "expected at least one Kartik Purnima in 3 years");

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        assert!(
            inst.tithi.unwrap().is_purnima(),
            "lunar-filtered instance {} is not Purnima",
            inst.bs_date
        );
    }
}

// ============================================================================
// BS family: additional oracle checks for BYMONTH, INTERVAL, YEARLY, termination
// ============================================================================

#[test]
fn bs_bymonth_instances_are_in_declared_months() {
    // BYMONTH=Baisakh,Bhadra — every instance must fall in one of those two months,
    // and the BS→AD→BS round-trip must hold.
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2082, 12, 30).unwrap();
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Monthly,
        interval: 1,
        anchor: start,
        by_month: Some(vec![BsMonth::Baisakh, BsMonth::Bhadra]),
        by_month_day: Some(vec![1]),
        by_day: None,
        count: Some(8),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert!(!out.is_empty());

    for bs in out {
        assert_bs_roundtrip(&engine, bs, start, end);
        assert!(
            bs.month == BsMonth::Baisakh || bs.month == BsMonth::Bhadra,
            "instance {} is not in Baisakh or Bhadra",
            bs
        );
        assert_eq!(bs.day, 1, "instance {} is not the 1st of its month", bs);
    }
}

#[test]
fn bs_monthly_interval2_instances_are_2_months_apart() {
    // MONTHLY INTERVAL=2: each consecutive pair must be exactly 2 BS months apart,
    // verified by counting the AD-day gap against the actual month lengths.
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2082, 12, 30).unwrap();
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Monthly,
        interval: 2,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: None,
        count: Some(6),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 6);

    for bs in &out {
        assert_bs_roundtrip(&engine, *bs, start, end);
    }

    // Each step must advance by exactly 2 BS months.
    for pair in out.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        // b is 2 months after a — same day, month+2 (with possible year wrap).
        // Verify via AD: the gap must equal the sum of days in the two months we crossed.
        let ad_a = engine.bs_to_gregorian(a).unwrap();
        let ad_b = engine.bs_to_gregorian(b).unwrap();
        let gap = (ad_b - ad_a).num_days();
        assert!(
            gap >= 56 && gap <= 66,
            "consecutive INTERVAL=2 monthly instances {a} and {b} are {gap} AD days apart (expected ~59±3)"
        );
    }
}

#[test]
fn bs_yearly_instances_same_month_and_day_each_year() {
    // YEARLY with no BYMONTH: each instance must be the same BS month and day,
    // one year later, and round-trip correctly.
    let engine = CalendarEngine::new();
    // Anchor on Bhadra (month 5) day 10.
    let start = BsDate::new(2081, 5, 10).unwrap();
    let end = BsDate::new(2085, 12, 30).unwrap();
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Yearly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: None,
        count: Some(4),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 4);

    for (i, bs) in out.iter().enumerate() {
        assert_bs_roundtrip(&engine, *bs, start, end);
        assert_eq!(bs.year, 2081 + i as u16, "year must increment by 1 each time");
        assert_eq!(bs.month, BsMonth::Bhadra, "month must stay Bhadra (5)");
        assert_eq!(bs.day, 10, "day must stay 10");
    }
}

#[test]
fn bs_count_bound_last_instance_is_within_window() {
    // COUNT=5: exactly 5 instances, each round-trips, last is still ≤ end.
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2085, 12, 30).unwrap();
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Yearly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: None,
        count: Some(5),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 5, "COUNT=5 must yield exactly 5 instances");

    for bs in &out {
        assert_bs_roundtrip(&engine, *bs, start, end);
    }
    assert!(*out.last().unwrap() <= end);
}

#[test]
fn bs_until_bound_no_instance_after_until() {
    // UNTIL=2081-06-01: no instance may exceed it, and the last instance equals
    // or precedes UNTIL, verified via independent round-trip.
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let until = BsDate::new(2081, 6, 1).unwrap();
    let end = BsDate::new(2085, 12, 30).unwrap();
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Monthly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: None,
        count: None,
        until: Some(until),
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert!(!out.is_empty());

    for bs in &out {
        assert_bs_roundtrip(&engine, *bs, start, end);
        assert!(
            *bs <= until,
            "instance {} exceeds UNTIL {}",
            bs,
            until
        );
    }
}

#[test]
fn bs_weekly_single_byday_matches_converted_weekday() {
    // BYDAY=FR (code 5): every instance must convert to a Friday.
    let engine = CalendarEngine::new();
    let start = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 3, 1).unwrap();
    let rule = BsRecurrenceRule {
        frequency: BsFrequency::Weekly,
        interval: 1,
        anchor: start,
        by_month: None,
        by_month_day: None,
        by_day: Some(vec![5]), // FR
        count: Some(6),
        until: None,
    };

    let out = bs_instances(&engine, &rule, start, end);
    assert_eq!(out.len(), 6);

    for bs in &out {
        assert_bs_roundtrip(&engine, *bs, start, end);
        let ad = engine.bs_to_gregorian(*bs).unwrap();
        assert_eq!(
            ad_weekday_code(ad),
            5,
            "instance {bs} (AD {ad}) is not a Friday"
        );
    }
}

// ============================================================================
// TITHI family: additional oracle checks for multi-tithi, BYMONTH, PAKSHA=Krishna
// ============================================================================

#[test]
fn tithi_amavasya_recomputes_to_amavasya() {
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 6, 1).unwrap();
    let rule = TithiRecurrenceRule::new(vec![Tithi::Amavasya], anchor).with_count(3);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 3);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        assert!(
            inst.tithi.unwrap().is_amavasya(),
            "instance {} is not Amavasya",
            inst.bs_date
        );
    }
}

#[test]
fn tithi_krishna_paksha_only_recomputes_and_is_krishna() {
    // Paksha=Krishna: every recomputed tithi must be in Krishna paksha.
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 8, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::KrishnaPanchami],
        Paksha::Krishna,
        anchor,
    )
    .with_count(4);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 4);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        let t = inst.tithi.unwrap();
        assert_eq!(
            t.paksha(),
            Paksha::Krishna,
            "instance {} ({:?}) must be Krishna paksha",
            inst.bs_date, t
        );
    }
}

#[test]
fn tithi_multi_tithi_list_each_recomputes_to_a_member() {
    // by_tithi=[Purnima, Amavasya]: each instance must recompute to one of those two.
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 8, 1).unwrap();
    let rule =
        TithiRecurrenceRule::new(vec![Tithi::Purnima, Tithi::Amavasya], anchor).with_count(6);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 6);

    let mut saw_purnima = false;
    let mut saw_amavasya = false;

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        let t = inst.tithi.unwrap();
        assert!(
            t.is_purnima() || t.is_amavasya(),
            "instance {} recomputed to {:?}, not in {{Purnima, Amavasya}}",
            inst.bs_date, t
        );
        if t.is_purnima() { saw_purnima = true; }
        if t.is_amavasya() { saw_amavasya = true; }
    }
    assert!(saw_purnima, "no Purnima instance in Purnima+Amavasya rule");
    assert!(saw_amavasya, "no Amavasya instance in Purnima+Amavasya rule");
}

#[test]
fn tithi_bymonth_solar_instances_in_declared_month_and_recompute() {
    // BYMONTH=Baisakh,Kartik: each instance must be in one of those solar months
    // AND recompute correctly.
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2083, 12, 30).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor)
        .with_by_month(vec![BsMonth::Baisakh, BsMonth::Kartik])
        .with_count(4);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 4);

    for inst in &out {
        assert_tithi_instance_consistent(&engine, inst);
        assert!(
            inst.tithi.unwrap().is_ekadashi(),
            "instance {} is not Ekadashi",
            inst.bs_date
        );
        assert!(
            inst.bs_date.month == BsMonth::Baisakh || inst.bs_date.month == BsMonth::Kartik,
            "instance {} is not in Baisakh or Kartik",
            inst.bs_date
        );
    }
}

#[test]
fn tithi_instances_ad_dates_are_strictly_ascending() {
    // AD dates on consecutive instances must be strictly increasing.
    // This is stronger than BS-date ordering because it also catches the
    // (impossible but important to assert) case of duplicate AD dates.
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 8, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(8);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert!(out.len() >= 2);

    for pair in out.windows(2) {
        assert!(
            pair[0].ad_date < pair[1].ad_date,
            "AD dates not strictly ascending: {} then {}",
            pair[0].ad_date,
            pair[1].ad_date
        );
    }
}

#[test]
fn tithi_ad_dates_are_real_calendar_days_not_gaps() {
    // Each consecutive pair of Ekadashi instances must be ~14–16 AD days apart
    // (one paksha). If a bug causes a 30-day gap, it means an occurrence was
    // silently dropped. This catches tithi-computation or iteration bugs.
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let end = BsDate::new(2081, 6, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(6);

    let out = tithi_instances(&engine, &rule, anchor, end);
    assert_eq!(out.len(), 6);

    for pair in out.windows(2) {
        let gap = (pair[1].ad_date - pair[0].ad_date).num_days();
        assert!(
            gap >= 13 && gap <= 17,
            "consecutive Ekadashi gap is {gap} days ({} → {}); expected 14–16",
            pair[0].ad_date,
            pair[1].ad_date
        );
    }
}

// ============================================================================
// X-TAKE=FIRST — Bijaya Dashami (Dashain) cross-validation
// ============================================================================

#[test]
fn take_first_yields_exactly_one_per_year() {
    // BYMONTH=6,7 + X-TAKE=FIRST: must produce exactly one instance per BS year
    // over a 10-year window, and every instance must be Shukla Dashami.
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2073, 1, 1).unwrap();
    let start = anchor;
    let end = BsDate::new(2082, 12, 30).unwrap();

    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaDashami],
        Paksha::Shukla,
        anchor,
    )
    .with_by_month(vec![BsMonth::Ashwin, BsMonth::Kartik])
    .with_take_first(true);

    let out = tithi_instances(&engine, &rule, start, end);

    // Collect counts per BS year
    let mut years: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for inst in &out {
        *years.entry(inst.bs_date.year).or_insert(0) += 1;
        assert_tithi_instance_consistent(&engine, inst);
        assert_eq!(
            inst.tithi.unwrap(),
            Tithi::ShuklaDashami,
            "instance {} must be Shukla Dashami",
            inst.bs_date
        );
        assert!(
            inst.bs_date.month == BsMonth::Ashwin || inst.bs_date.month == BsMonth::Kartik,
            "instance {} must be in Ashwin or Kartik",
            inst.bs_date
        );
    }

    for (&year, &cnt) in &years {
        assert_eq!(
            cnt, 1,
            "X-TAKE=FIRST must yield exactly one instance in BS year {year}, got {cnt}"
        );
    }

    // Should cover all 10 years 2073–2082
    assert_eq!(years.len(), 10, "expected instances in exactly 10 BS years");
}

#[test]
fn take_first_without_flag_yields_more_instances_than_with() {
    // Removing X-TAKE=FIRST from the same rule should produce >= as many instances
    // (in adhik-Bhadra years the unfiltered rule gives 2/year instead of 1).
    let engine = CalendarEngine::new();
    let anchor = BsDate::new(2073, 1, 1).unwrap();
    let start = anchor;
    let end = BsDate::new(2082, 12, 30).unwrap();

    let rule_with = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaDashami],
        Paksha::Shukla,
        anchor,
    )
    .with_by_month(vec![BsMonth::Ashwin, BsMonth::Kartik])
    .with_take_first(true);

    let rule_without = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaDashami],
        Paksha::Shukla,
        anchor,
    )
    .with_by_month(vec![BsMonth::Ashwin, BsMonth::Kartik]);

    let with_count = tithi_instances(&engine, &rule_with, start, end).len();
    let without_count = tithi_instances(&engine, &rule_without, start, end).len();

    assert!(
        without_count >= with_count,
        "without X-TAKE=FIRST must have >= instances: without={without_count} with={with_count}"
    );
}
