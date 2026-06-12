// Custom BS/Tithi RRULE String Parser Tests
//
// These tests exercise `RRuleParser::parse` (the top-level auto-detecting entry
// point used across the WASM boundary) and the BS/Tithi parsers directly, using
// the exact RRULE strings produced by the TypeScript UI (`BsRecurrenceBuilder.svelte`)
// and the backend. The goal is to keep the Rust engine, the WASM clients, and the
// NestJS backend IN SYNC on the custom RRULE grammar:
//   - X-CALENDAR=BS         marks a BS-calendar rule
//   - X-TITHI=<name[,name]> marks a tithi rule (takes precedence over X-CALENDAR)
//   - X-PAKSHA=SHUKLA|KRISHNA
//   - X-BYLUNARMONTH=<m,..>
//   - X-SKIPADHIK=TRUE|FALSE
//
// Where the TS UI and the Rust parser currently DISAGREE, the test is named with a
// `desync_` prefix and documents the observed engine behavior so the divergence is
// visible and tracked rather than silently passing.

use yorion_engine::domain::bs_date::BsDate;
use yorion_engine::domain::recurrence::rrule_parser::RRuleParser;
use yorion_engine::domain::recurrence::{BsFrequency, Recurrence};
use yorion_engine::domain::tithi::{Paksha, Tithi};

// ============================================================================
// Top-level auto-detection: RRuleParser::parse
// ============================================================================

#[test]
fn parse_detects_ad_when_no_markers() {
    // Standard RFC 5545 with no custom markers -> AD
    let rrule = "FREQ=WEEKLY;BYDAY=MO,WE,FR";
    match RRuleParser::parse(rrule).unwrap() {
        Recurrence::Ad(rule) => assert_eq!(rule.rrule, rrule),
        other => panic!("expected Ad, got {:?}", other),
    }
}

#[test]
fn parse_detects_bs_when_x_calendar_present() {
    let rrule = "FREQ=YEARLY;DTSTART=20810101;X-CALENDAR=BS";
    match RRuleParser::parse(rrule).unwrap() {
        Recurrence::Bs(rule) => {
            assert_eq!(rule.frequency, BsFrequency::Yearly);
            assert_eq!(rule.anchor, BsDate::new(2081, 1, 1).unwrap());
        }
        other => panic!("expected Bs, got {:?}", other),
    }
}

#[test]
fn parse_detects_tithi_when_x_tithi_present() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=PURNIMA;X-CALENDAR=BS";
    match RRuleParser::parse(rrule).unwrap() {
        Recurrence::Tithi(rule) => assert_eq!(rule.by_tithi[0], Tithi::Purnima),
        other => panic!("expected Tithi, got {:?}", other),
    }
}

#[test]
fn parse_tithi_takes_precedence_over_x_calendar() {
    // Both X-TITHI and X-CALENDAR=BS present -> must route to Tithi, not Bs.
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-CALENDAR=BS";
    assert!(matches!(
        RRuleParser::parse(rrule).unwrap(),
        Recurrence::Tithi(_)
    ));
}

#[test]
fn parse_keys_are_case_insensitive() {
    // parse_params upper-cases keys; values for FREQ are also upper-cased.
    let rrule = "freq=yearly;dtstart=20810101;x-calendar=BS";
    assert!(matches!(
        RRuleParser::parse(rrule).unwrap(),
        Recurrence::Bs(_)
    ));
}

// ============================================================================
// BS rule parsing — all four FREQ values
// ============================================================================

#[test]
fn parse_bs_freq_daily() {
    let rule = RRuleParser::parse_bs_rrule("FREQ=DAILY;DTSTART=20810101;X-CALENDAR=BS").unwrap();
    assert_eq!(rule.frequency, BsFrequency::Daily);
}

#[test]
fn parse_bs_freq_weekly() {
    let rule = RRuleParser::parse_bs_rrule("FREQ=WEEKLY;DTSTART=20810101;X-CALENDAR=BS").unwrap();
    assert_eq!(rule.frequency, BsFrequency::Weekly);
}

#[test]
fn parse_bs_freq_monthly() {
    let rule =
        RRuleParser::parse_bs_rrule("FREQ=MONTHLY;DTSTART=20810101;X-CALENDAR=BS").unwrap();
    assert_eq!(rule.frequency, BsFrequency::Monthly);
}

#[test]
fn parse_bs_freq_yearly() {
    let rule = RRuleParser::parse_bs_rrule("FREQ=YEARLY;DTSTART=20810101;X-CALENDAR=BS").unwrap();
    assert_eq!(rule.frequency, BsFrequency::Yearly);
}

// ============================================================================
// BS rule parsing — all optional parameters
// ============================================================================

#[test]
fn parse_bs_full_filter_set() {
    // The richest BS rule the TS "custom_bs" editor can emit.
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;INTERVAL=2;BYMONTH=1,5,9;BYMONTHDAY=1,15;BYDAY=MO,WE;COUNT=10;X-CALENDAR=BS";
    let rule = RRuleParser::parse_bs_rrule(rrule).unwrap();

    assert_eq!(rule.frequency, BsFrequency::Monthly);
    assert_eq!(rule.interval, 2);
    assert_eq!(rule.count, Some(10));
    assert_eq!(rule.by_month.as_ref().unwrap().len(), 3);
    assert_eq!(rule.by_month_day, Some(vec![1, 15]));
    assert_eq!(rule.by_day, Some(vec![1, 3])); // MO=1, WE=3
}

#[test]
fn parse_bs_until() {
    let rrule = "FREQ=YEARLY;DTSTART=20810101;UNTIL=20850101;X-CALENDAR=BS";
    let rule = RRuleParser::parse_bs_rrule(rrule).unwrap();
    assert_eq!(rule.until, Some(BsDate::new(2085, 1, 1).unwrap()));
}

#[test]
fn parse_bs_interval_defaults_to_1_when_absent() {
    let rule =
        RRuleParser::parse_bs_rrule("FREQ=DAILY;DTSTART=20810101;X-CALENDAR=BS").unwrap();
    assert_eq!(rule.interval, 1);
}

#[test]
fn parse_bs_bymonth_single_value() {
    let rule =
        RRuleParser::parse_bs_rrule("FREQ=YEARLY;DTSTART=20810101;BYMONTH=7;X-CALENDAR=BS")
            .unwrap();
    assert_eq!(rule.by_month.as_ref().unwrap().len(), 1);
}

#[test]
fn parse_bs_byday_all_7_weekday_codes() {
    // All valid two-letter weekday tokens must be accepted.
    let rrule = "FREQ=WEEKLY;DTSTART=20810101;BYDAY=SU,MO,TU,WE,TH,FR,SA;X-CALENDAR=BS";
    let rule = RRuleParser::parse_bs_rrule(rrule).unwrap();
    assert_eq!(rule.by_day, Some(vec![0, 1, 2, 3, 4, 5, 6]));
}

#[test]
fn parse_bs_until_invalid_format_errors_v5() {
    // UNTIL must be YYYYMMDD; ISO-style rejected with V5.
    let rrule = "FREQ=YEARLY;DTSTART=20810101;UNTIL=2085-01-01;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V5)
    );
}

// ============================================================================
// BS rule parsing — rejection codes (V1–V8)
// ============================================================================

#[test]
fn parse_bs_v1_param_missing_equals() {
    // A token without `=` must be rejected with V1.
    let rrule = "FREQ=DAILY;NOEQUALS;DTSTART=20810101;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V1)
    );
}

#[test]
fn parse_bs_missing_freq_errors_v2() {
    let rrule = "DTSTART=20810101;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V2)
    );
}

#[test]
fn parse_bs_bad_freq_errors_v3() {
    let rrule = "FREQ=FORTNIGHTLY;DTSTART=20810101;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V3)
    );
}

#[test]
fn parse_bs_missing_dtstart_errors_v4() {
    let rrule = "FREQ=YEARLY;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V4)
    );
}

#[test]
fn parse_bs_dtstart_must_be_8_digits_v5() {
    let rrule = "FREQ=YEARLY;DTSTART=2081-01-01;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V5)
    );
}

#[test]
fn parse_bs_interval_zero_errors_v6() {
    let rrule = "FREQ=DAILY;DTSTART=20810101;INTERVAL=0;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V6)
    );
}

#[test]
fn parse_bs_count_zero_errors_v6() {
    let rrule = "FREQ=DAILY;DTSTART=20810101;COUNT=0;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V6)
    );
}

#[test]
fn parse_bs_bymonth_zero_errors_v7() {
    // 0 is outside 1–12.
    let rrule = "FREQ=YEARLY;DTSTART=20810101;BYMONTH=0;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V7)
    );
}

#[test]
fn parse_bs_invalid_month_errors_v7() {
    let rrule = "FREQ=YEARLY;DTSTART=20810101;BYMONTH=13;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V7)
    );
}

#[test]
fn parse_bs_byday_invalid_token_errors_v8() {
    let rrule = "FREQ=WEEKLY;DTSTART=20810101;BYDAY=XX;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V8)
    );
}

// ============================================================================
// BS rule parsing — additional parameter coverage
// ============================================================================

#[test]
fn parse_bs_bymonthday_stored_correctly() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;BYMONTHDAY=1,15,32;X-CALENDAR=BS";
    let rule = RRuleParser::parse_bs_rrule(rrule).unwrap();
    assert_eq!(rule.by_month_day, Some(vec![1, 15, 32]));
}

#[test]
fn parse_bs_dtstart_month_zero_errors_v5() {
    // Month 00 is outside 1–12.
    let rrule = "FREQ=YEARLY;DTSTART=20810001;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V5)
    );
}

#[test]
fn parse_bs_interval_non_numeric_errors_v6() {
    let rrule = "FREQ=DAILY;DTSTART=20810101;INTERVAL=abc;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V6)
    );
}

#[test]
fn parse_bs_count_non_numeric_errors_v6() {
    let rrule = "FREQ=DAILY;DTSTART=20810101;COUNT=abc;X-CALENDAR=BS";
    let err = RRuleParser::parse_bs_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V6)
    );
}

// ============================================================================
// Roundtrip: struct -> rrule string -> struct (BS)
// ============================================================================

#[test]
fn bs_roundtrip_preserves_all_fields() {
    use yorion_engine::domain::bs_date::BsMonth;
    use yorion_engine::domain::recurrence::BsRecurrenceRule;

    let original = BsRecurrenceRule::new(BsFrequency::Monthly, BsDate::new(2081, 1, 1).unwrap())
        .with_interval(3)
        .with_by_month(vec![BsMonth::Baisakh, BsMonth::Kartik])
        .with_by_month_day(vec![1, 15, 30])
        .with_count(12);

    let rrule = RRuleParser::bs_to_rrule(&original);
    assert!(rrule.contains("X-CALENDAR=BS"));

    let parsed = RRuleParser::parse_bs_rrule(&rrule).unwrap();
    assert_eq!(parsed.frequency, original.frequency);
    assert_eq!(parsed.interval, original.interval);
    assert_eq!(parsed.by_month, original.by_month);
    assert_eq!(parsed.by_month_day, original.by_month_day);
    assert_eq!(parsed.count, original.count);
}

#[test]
fn bs_roundtrip_with_byday_and_until() {
    use yorion_engine::domain::recurrence::BsRecurrenceRule;

    let original = BsRecurrenceRule::new(BsFrequency::Weekly, BsDate::new(2081, 1, 1).unwrap())
        .with_by_day(vec![0, 5]) // SU, FR
        .with_until(BsDate::new(2082, 6, 15).unwrap());

    let rrule = RRuleParser::bs_to_rrule(&original);
    assert!(rrule.contains("BYDAY="));
    assert!(rrule.contains("UNTIL="));

    let parsed = RRuleParser::parse_bs_rrule(&rrule).unwrap();
    assert_eq!(parsed.by_day, original.by_day);
    assert_eq!(parsed.until, original.until);
    assert_eq!(parsed.count, None);
}

#[test]
fn bs_to_rrule_omits_interval_when_default() {
    use yorion_engine::domain::recurrence::BsRecurrenceRule;

    // interval=1 is the default and must not appear in the serialised string.
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, BsDate::new(2081, 1, 1).unwrap());
    assert_eq!(rule.interval, 1);
    let rrule = RRuleParser::bs_to_rrule(&rule);
    assert!(
        !rrule.contains("INTERVAL="),
        "INTERVAL=1 must be suppressed: {rrule}"
    );
}

// ============================================================================
// Tithi rule parsing — names produced by the TS UI
//
// NOTE: the inputs below use the v1.0 legacy form (X-TITHI present with
// X-CALENDAR=BS, discriminator not first). They drive parse_tithi_rrule
// directly to exercise tithi-name/param parsing, and are intentionally kept in
// legacy form to prove v2.0 still accepts v1.0 strings. Producers now emit
// canonical X-CALENDAR=PANCHANGA-first.
// ============================================================================

#[test]
fn parse_tithi_purnima_bare_name() {
    // TS preset 'purnima' emits X-TITHI=Purnima (CamelCase).
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_tithi, vec![Tithi::Purnima]);
}

#[test]
fn parse_tithi_amavasya_bare_name() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Amavasya;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_tithi, vec![Tithi::Amavasya]);
}

#[test]
fn parse_tithi_ekadashi_bare_defaults_to_shukla() {
    // Documented behavior: bare EKADASHI resolves to Shukla variant.
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Ekadashi;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_tithi, vec![Tithi::ShuklaEkadashi]);
}

#[test]
fn parse_tithi_with_paksha_filter() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Ekadashi;X-PAKSHA=SHUKLA;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.paksha_filter, Some(Paksha::Shukla));
}

#[test]
fn parse_tithi_with_bylunarmonth() {
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-BYLUNARMONTH=1,5,8;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_lunar_month.as_ref().unwrap().len(), 3);
}

#[test]
fn parse_tithi_missing_x_tithi_errors() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-PAKSHA=SHUKLA;X-CALENDAR=BS";
    assert!(RRuleParser::parse_tithi_rrule(rrule).is_err());
}

#[test]
fn parse_tithi_skipadhik_true_word() {
    // Backend/serializer form: X-SKIPADHIK=TRUE.
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-SKIPADHIK=TRUE;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(rule.skip_adhik);
}

#[test]
fn parse_tithi_skipadhik_false_word() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-SKIPADHIK=FALSE;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(!rule.skip_adhik);
}

#[test]
fn parse_tithi_skipadhik_defaults_true_when_absent() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(rule.skip_adhik);
}

// ============================================================================
// Tithi roundtrip: struct -> rrule string -> struct
// ============================================================================

#[test]
fn tithi_roundtrip_preserves_fields() {
    use yorion_engine::domain::recurrence::TithiRecurrenceRule;

    let original = TithiRecurrenceRule::purnima(BsDate::new(2081, 1, 1).unwrap()).with_count(6);
    let rrule = RRuleParser::tithi_to_rrule(&original);

    // Serializer emits the PANCHANGA discriminator (v2.0) and X-SKIPADHIK explicitly.
    assert!(rrule.contains("X-CALENDAR=PANCHANGA"));
    assert!(rrule.contains("X-SKIPADHIK="));

    let parsed = RRuleParser::parse_tithi_rrule(&rrule).unwrap();
    assert_eq!(parsed.by_tithi[0].name(), original.by_tithi[0].name());
    assert_eq!(parsed.anchor, original.anchor);
    assert_eq!(parsed.count, original.count);
    assert_eq!(parsed.skip_adhik, original.skip_adhik);
}

// ============================================================================
// PANCHANGA rule parsing — parameter coverage
// ============================================================================

#[test]
fn parse_tithi_paksha_krishna() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Ashtami;X-PAKSHA=KRISHNA;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.paksha_filter, Some(Paksha::Krishna));
}

#[test]
fn parse_tithi_paksha_invalid_errors_v10() {
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-PAKSHA=WAXING;X-CALENDAR=BS";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V10)
    );
}

#[test]
fn parse_tithi_unknown_name_errors_v9() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Dashera;X-CALENDAR=BS";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V9)
    );
}

#[test]
fn parse_tithi_empty_x_tithi_errors_v9() {
    // X-TITHI present but value is empty (e.g. "X-TITHI=").
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=;X-CALENDAR=BS";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V9)
    );
}

#[test]
fn parse_tithi_missing_dtstart_errors_v4() {
    let rrule = "FREQ=MONTHLY;X-TITHI=Purnima;X-CALENDAR=PANCHANGA";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V4)
    );
}

#[test]
fn parse_tithi_count_zero_errors_v6() {
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;COUNT=0;X-CALENDAR=PANCHANGA";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V6)
    );
}

#[test]
fn parse_tithi_bylunarmonth_zero_errors_v7() {
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-BYLUNARMONTH=0;X-CALENDAR=BS";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V7)
    );
}

#[test]
fn parse_tithi_bylunarmonth_13_errors_v7() {
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-BYLUNARMONTH=13;X-CALENDAR=BS";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(yorion_engine::error::RRuleRejectReason::V7)
    );
}

#[test]
fn parse_tithi_multiple_x_tithi_values() {
    // Comma-separated list: both tithis must be stored.
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima,Amavasya;X-CALENDAR=PANCHANGA";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_tithi.len(), 2);
    assert!(rule.by_tithi.contains(&Tithi::Purnima));
    assert!(rule.by_tithi.contains(&Tithi::Amavasya));
}

#[test]
fn parse_tithi_until_stored() {
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;UNTIL=20830101;X-CALENDAR=PANCHANGA";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.until, Some(BsDate::new(2083, 1, 1).unwrap()));
}

#[test]
fn parse_tithi_bymonth_solar_stored() {
    // BYMONTH is the solar-month filter (distinct from X-BYLUNARMONTH).
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Ekadashi;BYMONTH=1,9;X-CALENDAR=PANCHANGA";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_month.as_ref().unwrap().len(), 2);
    assert!(rule.by_lunar_month.is_none());
}

// ============================================================================
// PANCHANGA roundtrip — full field set
// ============================================================================

#[test]
fn tithi_roundtrip_full_field_set() {
    use yorion_engine::domain::bs_date::BsMonth;
    use yorion_engine::domain::recurrence::TithiRecurrenceRule;

    let original = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaEkadashi, Tithi::KrishnaEkadashi],
        Paksha::Shukla,
        BsDate::new(2081, 1, 1).unwrap(),
    )
    .with_count(6)
    .with_until(BsDate::new(2083, 6, 1).unwrap())
    .with_by_month(vec![BsMonth::Baisakh, BsMonth::Kartik])
    .with_by_lunar_month(vec![BsMonth::Shrawan])
    .with_skip_adhik(false);

    let rrule = RRuleParser::tithi_to_rrule(&original);

    assert!(rrule.contains("X-CALENDAR=PANCHANGA"));
    assert!(rrule.contains("X-TITHI="));
    assert!(rrule.contains("X-PAKSHA=SHUKLA"));
    assert!(rrule.contains("COUNT=6"));
    assert!(rrule.contains("UNTIL="));
    assert!(rrule.contains("BYMONTH="));
    assert!(rrule.contains("X-BYLUNARMONTH="));
    assert!(rrule.contains("X-SKIPADHIK=FALSE"));

    let parsed = RRuleParser::parse_tithi_rrule(&rrule).unwrap();
    assert_eq!(parsed.paksha_filter, original.paksha_filter);
    assert_eq!(parsed.count, original.count);
    assert_eq!(parsed.until, original.until);
    assert_eq!(
        parsed.by_month.as_ref().map(|v| v.len()),
        original.by_month.as_ref().map(|v| v.len())
    );
    assert_eq!(
        parsed.by_lunar_month.as_ref().map(|v| v.len()),
        original.by_lunar_month.as_ref().map(|v| v.len())
    );
    assert_eq!(parsed.skip_adhik, original.skip_adhik);
}

// ============================================================================
// CROSS-BOUNDARY PARITY
//
// These assert that the engine accepts the exact RRULE strings produced by the
// TypeScript UI (shared/packages/ui/src/components/events/BsRecurrenceBuilder.svelte).
// Previously the engine diverged here; the parser now accepts both forms.
//
// NOTE: these inputs are the v1.0 legacy form (X-CALENDAR=BS, not first). They
// remain in legacy form on purpose to guard backward-compat parsing; v2.0
// producers emit canonical X-CALENDAR=PANCHANGA-first.
// ============================================================================

#[test]
fn parity_skipadhik_numeric_one_is_true() {
    // TS UI emits `X-SKIPADHIK=1` (see buildRrule: `parts.push('X-SKIPADHIK=1')`).
    // Engine now treats "1"/"TRUE"/"YES" as true, matching UI intent ('skip adhik').
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-SKIPADHIK=1;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(
        rule.skip_adhik,
        "X-SKIPADHIK=1 from the TS UI must parse as skip_adhik=true"
    );
}

#[test]
fn parity_skipadhik_zero_is_false() {
    let rrule = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-SKIPADHIK=0;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(!rule.skip_adhik);
}

#[test]
fn parity_bare_paksha_qualified_tithi_name_parses() {
    // TS UI "Fulpati" preset emits X-TITHI=Saptami (bare, paksha given separately
    // via X-PAKSHA). The bare day-name now resolves (to the Shukla enum variant),
    // and the paksha filter is carried independently.
    let rrule =
        "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Saptami;X-PAKSHA=SHUKLA;BYMONTH=6;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert_eq!(rule.by_tithi, vec![Tithi::ShuklaSaptami]);
    assert_eq!(rule.paksha_filter, Some(Paksha::Shukla));
    assert_eq!(rule.by_month.as_ref().unwrap().len(), 1);
}

// ============================================================================
// BS-RRULE v2.0: X-CALENDAR is the single family discriminator
//
// X-CALENDAR=PANCHANGA selects the lunar (tithi) family. X-TITHI is no longer the
// detector — it only carries which tithi names. Legacy v1.0 strings (X-TITHI with
// X-CALENDAR=BS, or X-TITHI alone) must still resolve to the lunar family.
// ============================================================================

#[test]
fn parse_x_calendar_panchanga_selects_tithi_family() {
    let rrule = "X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima";
    match RRuleParser::parse(rrule).unwrap() {
        Recurrence::Tithi(rule) => assert_eq!(rule.by_tithi, vec![Tithi::Purnima]),
        other => panic!("expected Tithi family, got {:?}", other),
    }
}

#[test]
fn parse_legacy_tithi_without_panchanga_still_tithi() {
    // v1.0 compat: X-TITHI present with X-CALENDAR=BS → lunar family.
    let with_bs = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Ekadashi;X-CALENDAR=BS";
    assert!(matches!(
        RRuleParser::parse(with_bs).unwrap(),
        Recurrence::Tithi(_)
    ));

    // v1.0 compat: bare X-TITHI with no X-CALENDAR → lunar family.
    let bare = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Ekadashi";
    assert!(matches!(
        RRuleParser::parse(bare).unwrap(),
        Recurrence::Tithi(_)
    ));
}

#[test]
fn tithi_serializes_x_calendar_panchanga_first() {
    use yorion_engine::domain::recurrence::TithiRecurrenceRule;

    let original = TithiRecurrenceRule::purnima(BsDate::new(2081, 1, 1).unwrap());
    let rrule = RRuleParser::tithi_to_rrule(&original);

    assert!(rrule.contains("X-CALENDAR=PANCHANGA"));
    assert!(!rrule.contains("X-CALENDAR=BS"));
    // Canonical order: the discriminator leads the string.
    assert!(rrule.starts_with("X-CALENDAR=PANCHANGA"));
}

#[test]
fn tithi_legacy_roundtrips_to_canonical_panchanga() {
    // A legacy string parses, and re-serializing yields canonical PANCHANGA output.
    let legacy = "FREQ=MONTHLY;DTSTART=20810101;X-TITHI=Purnima;X-CALENDAR=BS";
    let rule = RRuleParser::parse_tithi_rrule(legacy).unwrap();
    let reserialized = RRuleParser::tithi_to_rrule(&rule);
    assert!(reserialized.starts_with("X-CALENDAR=PANCHANGA"));
    assert!(!reserialized.contains("X-CALENDAR=BS"));
}

#[test]
fn parity_all_bare_tithi_day_names_parse() {
    // Every bare day-name the TS `BsTithiName` union can emit must resolve.
    // (Note: UI spells the 6th tithi "Shashthi"; engine enum is "Shashti".)
    let names = [
        "Pratipada",
        "Dvitiya",
        "Tritiya",
        "Chaturthi",
        "Panchami",
        "Shashthi",
        "Saptami",
        "Ashtami",
        "Navami",
        "Dashami",
        "Ekadashi",
        "Dwadashi",
        "Trayodashi",
        "Chaturdashi",
        "Purnima",
        "Amavasya",
    ];
    for name in names {
        let rrule = format!(
            "FREQ=MONTHLY;DTSTART=20810101;X-TITHI={};X-CALENDAR=BS",
            name
        );
        assert!(
            RRuleParser::parse_tithi_rrule(&rrule).is_ok(),
            "bare tithi name '{}' from the TS UI must parse",
            name
        );
    }
}

// ============================================================================
// X-TAKE=FIRST — parser / serializer / roundtrip
// ============================================================================

#[test]
fn x_take_first_parses_take_first_true() {
    let rrule = "X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20730101;X-TITHI=SHUKLADASHAMI;X-PAKSHA=SHUKLA;BYMONTH=6,7;X-SKIPADHIK=TRUE;X-TAKE=FIRST";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(rule.take_first, "X-TAKE=FIRST must set take_first=true");
}

#[test]
fn x_take_first_case_insensitive() {
    let rrule = "X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-TAKE=first";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(rule.take_first, "x-take=first (lowercase) must be accepted");
}

#[test]
fn x_take_absent_means_take_first_false() {
    let rrule = "X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI";
    let rule = RRuleParser::parse_tithi_rrule(rrule).unwrap();
    assert!(!rule.take_first, "absent X-TAKE must default to take_first=false");
}

#[test]
fn x_take_invalid_value_rejected_v11() {
    use yorion_engine::error::RRuleRejectReason;

    let rrule = "X-CALENDAR=PANCHANGA;FREQ=MONTHLY;DTSTART=20810101;X-TITHI=EKADASHI;X-TAKE=LAST";
    let err = RRuleParser::parse_tithi_rrule(rrule).unwrap_err();
    assert_eq!(
        err.reject_reason(),
        Some(RRuleRejectReason::V11),
        "X-TAKE=LAST must be rejected with V11"
    );
}

#[test]
fn x_take_first_serializes_and_roundtrips() {
    use yorion_engine::domain::bs_date::BsMonth;
    use yorion_engine::domain::recurrence::TithiRecurrenceRule;
    use yorion_engine::domain::tithi::{Paksha, Tithi};

    let anchor = BsDate::new(2073, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaDashami],
        Paksha::Shukla,
        anchor,
    )
    .with_by_month(vec![BsMonth::Ashwin, BsMonth::Kartik])
    .with_take_first(true);

    let s = RRuleParser::tithi_to_rrule(&rule);
    assert!(s.contains("X-TAKE=FIRST"), "serialized string must contain X-TAKE=FIRST");

    // Roundtrip: parse the serialized string back
    let reparsed = RRuleParser::parse_tithi_rrule(&s).unwrap();
    assert!(reparsed.take_first, "reparsed rule must have take_first=true");
    assert_eq!(reparsed.paksha_filter, rule.paksha_filter);
    assert_eq!(reparsed.by_tithi, rule.by_tithi);
    assert_eq!(reparsed.by_month, rule.by_month);
}

#[test]
fn x_take_first_false_not_emitted_in_serialization() {
    use yorion_engine::domain::recurrence::TithiRecurrenceRule;
    let anchor = BsDate::new(2081, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::purnima(anchor); // take_first defaults to false
    let s = RRuleParser::tithi_to_rrule(&rule);
    assert!(!s.contains("X-TAKE"), "X-TAKE must not appear when take_first=false");
}
