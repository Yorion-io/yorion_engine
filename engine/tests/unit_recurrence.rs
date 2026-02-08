// Domain Recurrence Tests (BS Rules, RRule Parser, Tithi Rules)
use bs_calendar_core::domain::bs_date::BsDate;
use bs_calendar_core::domain::recurrence::rrule_parser::RRuleParser;
use bs_calendar_core::domain::recurrence::{
    AdRecurrenceRule, BsFrequency, BsRecurrenceRule, TithiRecurrenceRule,
};
use bs_calendar_core::domain::tithi::{Paksha, Tithi};

mod helpers;

// ===== BS Rules Tests =====

#[test]
fn test_create_basic_rule() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor);

    assert_eq!(rule.frequency, BsFrequency::Daily);
    assert_eq!(rule.interval, 1);
    assert_eq!(rule.anchor, anchor);
    assert!(rule.validate().is_ok());
}

#[test]
fn test_bs_rule_builder_pattern() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2081, 1, 1).unwrap();

    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_interval(2)
        .with_by_month_day(vec![1, 15])
        .with_count(10)
        .with_until(until);

    assert_eq!(rule.interval, 2);
    assert_eq!(rule.by_month_day, Some(vec![1, 15]));
    assert_eq!(rule.count, Some(10));
    assert_eq!(rule.until, Some(until));
    assert!(rule.validate().is_ok());
}

#[test]
fn test_invalid_interval() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_interval(0);

    // Interval should be clamped to 1
    assert_eq!(rule.interval, 1);
}

#[test]
fn test_invalid_month_day() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor).with_by_month_day(vec![33]);

    assert!(rule.validate().is_err());
}

#[test]
fn test_invalid_until() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2079, 1, 1).unwrap(); // Before anchor

    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_until(until);

    assert!(rule.validate().is_err());
}

// ===== RRule Parser Tests =====

#[test]
fn test_parse_bs_daily_rrule() {
    let rrule = "FREQ=DAILY;DTSTART=20800101;COUNT=10;X-CALENDAR=BS";
    let rule = RRuleParser::parse_bs_rrule(rrule).unwrap();

    assert_eq!(rule.frequency, BsFrequency::Daily);
    assert_eq!(rule.anchor, BsDate::new(2080, 1, 1).unwrap());
    assert_eq!(rule.count, Some(10));
    assert_eq!(rule.interval, 1);
}

#[test]
fn test_parse_bs_monthly_with_filters() {
    let rrule = "FREQ=MONTHLY;DTSTART=20800101;BYMONTH=1,5,9;BYMONTHDAY=1,15;X-CALENDAR=BS";
    let rule = RRuleParser::parse_bs_rrule(rrule).unwrap();

    assert_eq!(rule.frequency, BsFrequency::Monthly);
    assert!(rule.by_month.is_some());
    assert!(rule.by_month_day.is_some());
}

#[test]
fn test_parse_ad_rrule() {
    let rrule = "DTSTART:20240115T000000Z\nRRULE:FREQ=WEEKLY;INTERVAL=2;COUNT=10";
    let rule = RRuleParser::parse_ad_rrule(rrule).unwrap();

    // Should contain the original string
    assert_eq!(rule.rrule, rrule);
}

#[test]
fn test_parse_tithi_rrule() {
    let rrule = "FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;X-SKIPADHIK=TRUE;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();

    // EKADASHI without paksha defaults to Shukla Ekadashi
    assert_eq!(rule.by_tithi[0], Tithi::ShuklaEkadashi);
    assert_eq!(rule.skip_adhik, true);
}

#[test]
fn test_parse_tithi_with_paksha() {
    let rrule = "FREQ=MONTHLY;DTSTART=20800101;X-TITHI=EKADASHI;X-PAKSHA=SHUKLA;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();

    assert_eq!(rule.paksha_filter, Some(Paksha::Shukla));
}

#[test]
fn test_bs_to_rrule_roundtrip() {
    let original = BsRecurrenceRule::new(BsFrequency::Monthly, BsDate::new(2080, 1, 1).unwrap())
        .with_interval(2)
        .with_count(10);

    let rrule = RRuleParser::bs_to_rrule(&original);
    let parsed = RRuleParser::parse_bs_rrule(&rrule).unwrap();

    assert_eq!(parsed.frequency, original.frequency);
    assert_eq!(parsed.anchor, original.anchor);
    assert_eq!(parsed.interval, original.interval);
    assert_eq!(parsed.count, original.count);
}

#[test]
fn test_ad_to_rrule_roundtrip() {
    let rrule_str = "DTSTART:20240115T000000Z\nRRULE:FREQ=WEEKLY;INTERVAL=1";
    let original = AdRecurrenceRule::new(rrule_str.to_string()).unwrap();

    let rrule = RRuleParser::ad_to_rrule(&original);
    let parsed = RRuleParser::parse_ad_rrule(&rrule).unwrap();

    assert_eq!(parsed.rrule, original.rrule);
}

#[test]
fn test_tithi_to_rrule_roundtrip() {
    let original = TithiRecurrenceRule::ekadashi(BsDate::new(2080, 1, 1).unwrap()).with_count(12);

    let rrule = RRuleParser::tithi_to_rrule(&original);
    let parsed = RRuleParser::parse_tithi_rrule(&rrule).unwrap();

    assert_eq!(parsed.by_tithi[0].name(), original.by_tithi[0].name());
    assert_eq!(parsed.anchor, original.anchor);
    assert_eq!(parsed.count, original.count);
}

// ===== Tithi Rules Tests =====

#[test]
fn test_ekadashi_rule() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor);

    // Should match both Shukla and Krishna Ekadashi
    assert!(rule.matches_tithi(Tithi::ShuklaEkadashi));
    assert!(rule.matches_tithi(Tithi::KrishnaEkadashi));
    assert!(!rule.matches_tithi(Tithi::Purnima));
}

#[test]
fn test_purnima_rule() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::purnima(anchor);

    assert!(rule.matches_tithi(Tithi::Purnima));
    assert!(!rule.matches_tithi(Tithi::Amavasya));
    assert!(!rule.matches_tithi(Tithi::ShuklaEkadashi));
}

#[test]
fn test_paksha_filter() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule =
        TithiRecurrenceRule::with_paksha(vec![Tithi::ShuklaEkadashi], Paksha::Shukla, anchor);

    // Should only match Shukla Ekadashi
    assert!(rule.matches_tithi(Tithi::ShuklaEkadashi));
    assert!(!rule.matches_tithi(Tithi::KrishnaEkadashi));
}

#[test]
fn test_tithi_rule_builder_pattern() {
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let until = BsDate::new(2081, 1, 1).unwrap();

    let rule = TithiRecurrenceRule::ekadashi(anchor)
        .with_count(10)
        .with_until(until);

    assert_eq!(rule.count, Some(10));
    assert_eq!(rule.until, Some(until));
    assert!(rule.validate().is_ok());
}
