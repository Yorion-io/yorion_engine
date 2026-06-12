// Tithi Recurrence Tests
//
// These exercise the PRODUCTION tithi expansion path:
//   CalendarEngine::generate_tithi_instances(...) -> Vec<EventInstance>
// which is wired to TithiInstanceGenerator and enforces sunrise-tithi,
// paksha/lunar-month filtering, adhik-maas skipping, and UNTIL.
//
// (The earlier dead-path tests that called InstanceGenerator::generate_tithi_instances
// returning Vec<BsDate> were removed: that method is not the path production uses.)
use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::bs_date::{BsDate, BsMonth};
use yorion_engine::domain::event::{CalendarVersion, EventInstance};
use yorion_engine::domain::recurrence::TithiRecurrenceRule;
use yorion_engine::domain::tithi::{Location, Paksha, Tithi};

mod helpers;

const EVENT_ID: &str = "evt";
const TITLE: &str = "Tithi Event";

fn version() -> CalendarVersion {
    CalendarVersion::official("test".to_string())
}

fn expand(rule: &TithiRecurrenceRule, start: BsDate, end: BsDate) -> Vec<EventInstance> {
    let engine = CalendarEngine::new();
    engine
        .generate_tithi_instances(
            EVENT_ID,
            TITLE,
            rule,
            start,
            end,
            version(),
            Location::kathmandu(),
        )
        .unwrap()
}

fn tithi_of(inst: &EventInstance) -> Tithi {
    inst.tithi.expect("production tithi instance must carry a tithi")
}

#[test]
fn test_recurrence_ekadashi() {
    // Every Ekadashi (11th tithi of BOTH pakshas).
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(4);
    let end = BsDate::new(2080, 3, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 4);
    for inst in &instances {
        assert!(
            tithi_of(inst).is_ekadashi(),
            "{} should be Ekadashi but is {:?}",
            inst.bs_date,
            tithi_of(inst)
        );
    }
}

#[test]
fn test_recurrence_purnima() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::purnima(anchor).with_count(2);
    let end = BsDate::new(2080, 3, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 2);
    for inst in &instances {
        assert!(
            tithi_of(inst).is_purnima(),
            "{} should be Purnima but is {:?}",
            inst.bs_date,
            tithi_of(inst)
        );
    }
}

#[test]
fn test_recurrence_krishna_ashtami() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule =
        TithiRecurrenceRule::with_paksha(vec![Tithi::KrishnaAshtami], Paksha::Krishna, anchor)
            .with_count(2);
    let end = BsDate::new(2080, 4, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 2);
    for inst in &instances {
        assert_eq!(
            tithi_of(inst),
            Tithi::KrishnaAshtami,
            "{} should be Krishna Ashtami",
            inst.bs_date
        );
    }
}

#[test]
fn test_recurrence_multiple_tithis() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaPanchami, Tithi::ShuklaDashami],
        Paksha::Shukla,
        anchor,
    )
    .with_count(4);
    let end = BsDate::new(2080, 3, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 4);
    for inst in &instances {
        let t = tithi_of(inst);
        assert!(
            t == Tithi::ShuklaPanchami || t == Tithi::ShuklaDashami,
            "{} should be Shukla Panchami or Dashami but is {:?}",
            inst.bs_date,
            t
        );
    }
}

#[test]
fn test_tithi_in_specific_bs_month() {
    // Solar BYMONTH filter: Ekadashi only in Baisakh (solar month 1).
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor)
        .with_by_month(vec![BsMonth::Baisakh])
        .with_count(2);
    let end = BsDate::new(2080, 12, 30).unwrap();

    let instances = expand(&rule, anchor, end);

    assert!(instances.len() >= 2);
    for inst in &instances {
        assert_eq!(
            inst.bs_date.month,
            BsMonth::Baisakh,
            "instance {} MUST be in solar month Baisakh",
            inst.bs_date
        );
    }
}

// ============================================================================
// Amavasya and Chaturdashi (new moon, 14th day) — coverage of less-common tithis
// ============================================================================

#[test]
fn test_recurrence_amavasya() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::Amavasya],
        Paksha::Krishna,
        anchor,
    )
    .with_count(3);
    let end = BsDate::new(2080, 6, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 3);
    for inst in &instances {
        assert!(
            tithi_of(inst).is_amavasya(),
            "{} should be Amavasya but is {:?}",
            inst.bs_date,
            tithi_of(inst)
        );
    }
}

#[test]
fn test_recurrence_krishna_chaturdashi() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::KrishnaChaturdashi],
        Paksha::Krishna,
        anchor,
    )
    .with_count(2);
    let end = BsDate::new(2080, 5, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 2);
    for inst in &instances {
        assert_eq!(
            tithi_of(inst),
            Tithi::KrishnaChaturdashi,
            "{} should be Krishna Chaturdashi",
            inst.bs_date
        );
        assert_eq!(tithi_of(inst).day_in_paksha(), 14);
    }
}

// ============================================================================
// Production-only scenarios (impossible on the dead Vec<BsDate> path)
// ============================================================================

#[test]
fn test_tithi_until_stops_at_until() {
    // UNTIL bounds the expansion: no instance may fall after UNTIL.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2080, 2, 15).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_until(until);
    let end = BsDate::new(2080, 12, 30).unwrap();

    let instances = expand(&rule, anchor, end);

    assert!(!instances.is_empty(), "expected at least one Ekadashi before UNTIL");
    for inst in &instances {
        assert!(
            inst.bs_date <= until,
            "instance {} exceeds UNTIL {}",
            inst.bs_date,
            until
        );
    }
}

#[test]
fn test_tithi_paksha_narrowing_shukla_only() {
    // Same tithi number, Shukla paksha only — must never return the Krishna one.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaEkadashi],
        Paksha::Shukla,
        anchor,
    )
    .with_count(3);
    let end = BsDate::new(2080, 6, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 3);
    for inst in &instances {
        assert_eq!(
            tithi_of(inst),
            Tithi::ShuklaEkadashi,
            "{} should be Shukla Ekadashi (never Krishna)",
            inst.bs_date
        );
    }
}

#[test]
fn test_tithi_by_lunar_month_filter_restricts_results() {
    // X-BYLUNARMONTH restricts to a lunar month. Compare a lunar-month-filtered
    // run against the unrestricted run over the same window: the filtered set must
    // be a non-strict subset (<=) and every kept instance is still the target tithi.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2081, 12, 30).unwrap();

    let unrestricted = TithiRecurrenceRule::purnima(anchor);
    let restricted = TithiRecurrenceRule::purnima(anchor)
        .with_by_lunar_month(vec![BsMonth::Kartik]);

    let all = expand(&unrestricted, anchor, end);
    let filtered = expand(&restricted, anchor, end);

    assert!(
        !all.is_empty(),
        "sanity: unrestricted Purnima should produce instances"
    );
    assert!(
        filtered.len() <= all.len(),
        "lunar-month filter must not add instances ({} > {})",
        filtered.len(),
        all.len()
    );
    for inst in &filtered {
        assert!(
            tithi_of(inst).is_purnima(),
            "{} should be Purnima",
            inst.bs_date
        );
    }
}

#[test]
fn test_tithi_skip_adhik_branch_behavior() {
    // skip_adhik default (true) vs false over a multi-year window.
    // Including adhik occurrences (skip=false) can only ADD instances, never
    // remove them, so the not-skipping count must be >= the skipping count.
    // This exercises the adhik branch without hardcoding a calendar-specific
    // adhik month.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let end = BsDate::new(2085, 12, 30).unwrap();

    let skipping = TithiRecurrenceRule::ekadashi(anchor).with_skip_adhik(true);
    let including = TithiRecurrenceRule::ekadashi(anchor).with_skip_adhik(false);

    let skipped = expand(&skipping, anchor, end);
    let included = expand(&including, anchor, end);

    assert!(!skipped.is_empty(), "sanity: expected Ekadashi instances");
    assert!(
        included.len() >= skipped.len(),
        "including adhik must not drop instances (incl {} < skip {})",
        included.len(),
        skipped.len()
    );
}

// ============================================================================
// Edge cases: COUNT=1, UNTIL before any occurrence
// ============================================================================

#[test]
fn test_tithi_count_one_yields_exactly_one() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(1);
    let end = BsDate::new(2080, 6, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 1, "COUNT=1 must yield exactly one instance");
    assert!(
        tithi_of(&instances[0]).is_ekadashi(),
        "the single instance must be an Ekadashi"
    );
}

#[test]
fn test_tithi_until_equal_to_anchor_yields_zero_or_one() {
    // UNTIL set to the anchor itself. The expansion searches from anchor onwards;
    // if Ekadashi falls on the anchor day it may yield 1, otherwise 0. Either
    // way there must be no instance AFTER the anchor.
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_until(anchor);
    let end = BsDate::new(2080, 6, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    for inst in &instances {
        assert!(
            inst.bs_date <= anchor,
            "instance {} exceeds UNTIL == anchor {}",
            inst.bs_date,
            anchor
        );
    }
}

// ============================================================================
// Krishna paksha narrowing (mirror of the Shukla-only test)
// ============================================================================

#[test]
fn test_tithi_paksha_narrowing_krishna_only() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::KrishnaAshtami],
        Paksha::Krishna,
        anchor,
    )
    .with_count(3);
    let end = BsDate::new(2080, 6, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 3);
    for inst in &instances {
        assert_eq!(
            tithi_of(inst).paksha(),
            Paksha::Krishna,
            "instance {} must be Krishna paksha",
            inst.bs_date
        );
    }
}

// ============================================================================
// Multi-tithi list (Purnima + Amavasya): both must appear
// ============================================================================

#[test]
fn test_tithi_multi_tithi_list_purnima_and_amavasya() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::new(vec![Tithi::Purnima, Tithi::Amavasya], anchor)
        .with_count(4);
    let end = BsDate::new(2080, 4, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 4);
    let has_purnima = instances.iter().any(|i| tithi_of(i).is_purnima());
    let has_amavasya = instances.iter().any(|i| tithi_of(i).is_amavasya());
    assert!(has_purnima, "expected at least one Purnima in Purnima+Amavasya rule");
    assert!(has_amavasya, "expected at least one Amavasya in Purnima+Amavasya rule");
}

// ============================================================================
// Output is in ascending bs_date order
// ============================================================================

#[test]
fn test_tithi_instances_are_ascending() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(6);
    let end = BsDate::new(2080, 6, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert!(instances.len() >= 2, "need at least 2 instances to check order");
    for pair in instances.windows(2) {
        assert!(
            pair[0].bs_date < pair[1].bs_date,
            "instances not in ascending order: {} then {}",
            pair[0].bs_date,
            pair[1].bs_date
        );
    }
}

// ============================================================================
// COUNT terminates before UNTIL (tithi)
// ============================================================================

#[test]
fn test_tithi_count_terminates_before_until() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2085, 12, 30).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor)
        .with_count(3)
        .with_until(until);
    let end = BsDate::new(2090, 1, 1).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 3, "COUNT=3 must stop after 3 even with far UNTIL");
}

// ============================================================================
// BYMONTH (solar) + tithi: both filters active simultaneously
// ============================================================================

#[test]
fn test_tithi_bymonth_solar_and_tithi_combined() {
    // Ekadashi, but only in solar months Baisakh (1) and Kartik (7).
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor)
        .with_by_month(vec![BsMonth::Baisakh, BsMonth::Kartik])
        .with_count(4);
    let end = BsDate::new(2082, 12, 30).unwrap();

    let instances = expand(&rule, anchor, end);

    assert_eq!(instances.len(), 4);
    for inst in &instances {
        assert!(
            inst.bs_date.month == BsMonth::Baisakh || inst.bs_date.month == BsMonth::Kartik,
            "instance {} must be in Baisakh or Kartik",
            inst.bs_date
        );
        assert!(
            tithi_of(inst).is_ekadashi(),
            "instance {} must be Ekadashi",
            inst.bs_date
        );
    }
}
