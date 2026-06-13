//! Date-conversion ground-truth validation against Hamro Patro.
//!
//! Covers the 5-year window BS 2081–2085 (today is BS 2083): the past two
//! years, the current year, and the next two years. For every day Hamro Patro
//! lists, this asserts the engine converts **both directions** correctly:
//!
//!   1. BS → AD : engine.bs_to_gregorian(bs) == the AD date HP lists.
//!   2. AD → BS : engine.gregorian_to_bs(ad) == the BS date HP lists.
//!
//! The reference rows are embedded locally under tests/data/conversion/. For
//! BS 2081–2083 they are the date columns of the Hamro Patro almanac CSVs; for
//! BS 2084–2085 (which HP publishes with no tithi) they are dates-only files
//! scraped live from hamropatro.com and verified internally consistent (no AD
//! gaps, contiguous BS days). See tools/tools/scrapers/scrape_dates_v2.js.
//!
//! Policy: EXACT match, zero tolerance. Any mismatch is a hard conversion bug.
//!
//! Run with output:  cargo test --test conversion_ground_truth -- --nocapture

use chrono::NaiveDate;
use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::bs_date::BsDate;

/// Hamro-Patro-sourced BS↔AD date tables for the 5-year window, embedded at
/// compile time. A moved data dir fails compilation loudly — intended.
const YEARS: &[(u16, &str)] = &[
    (2081, include_str!("data/conversion/dates_2081.csv")),
    (2082, include_str!("data/conversion/dates_2082.csv")),
    (2083, include_str!("data/conversion/dates_2083.csv")),
    (2084, include_str!("data/conversion/dates_2084.csv")),
    (2085, include_str!("data/conversion/dates_2085.csv")),
];

/// Parse one CSV line `BS Date,AD Date` (`YYYY-MM-DD,YYYY-MM-DD`).
/// Returns None for the header and blank lines.
fn parse_row(line: &str) -> Option<(BsDate, NaiveDate)> {
    let mut cols = line.split(',');
    let bs_str = cols.next()?.trim();
    let ad_str = cols.next()?.trim();

    if bs_str == "BS Date" || bs_str.is_empty() {
        return None;
    }

    let mut bs_parts = bs_str.split('-');
    let by: u16 = bs_parts.next()?.parse().ok()?;
    let bm: u8 = bs_parts.next()?.parse().ok()?;
    let bd: u8 = bs_parts.next()?.parse().ok()?;
    let bs = BsDate::new(by, bm, bd).ok()?;

    let ad = NaiveDate::parse_from_str(ad_str, "%Y-%m-%d").ok()?;
    Some((bs, ad))
}

#[test]
fn engine_conversion_matches_hamro_patro() {
    let engine = CalendarEngine::new();

    let mut rows_checked = 0usize;
    let mut bs_to_ad_mismatches: Vec<String> = Vec::new();
    let mut ad_to_bs_mismatches: Vec<String> = Vec::new();

    for (year, csv) in YEARS {
        let mut year_rows = 0usize;
        for line in csv.lines() {
            let Some((bs, ad)) = parse_row(line) else {
                continue;
            };
            rows_checked += 1;
            year_rows += 1;

            // 1. BS -> AD
            let engine_ad = engine
                .bs_to_gregorian(bs)
                .unwrap_or_else(|e| panic!("bs_to_gregorian failed for {bs} ({year}): {e}"));
            if engine_ad != ad {
                bs_to_ad_mismatches.push(format!(
                    "BS {bs} : engine AD {engine_ad} != HP AD {ad}"
                ));
            }

            // 2. AD -> BS
            let engine_bs = engine
                .gregorian_to_bs(ad)
                .unwrap_or_else(|e| panic!("gregorian_to_bs failed for AD {ad} ({year}): {e}"));
            if engine_bs != bs {
                ad_to_bs_mismatches.push(format!(
                    "AD {ad} : engine BS {engine_bs} != HP BS {bs}"
                ));
            }
        }
        // Every year in the window must contribute a full year of days.
        assert!(
            year_rows >= 360,
            "BS {year}: only {year_rows} reference rows — data file looks truncated"
        );
    }

    let b2a = bs_to_ad_mismatches.len();
    let a2b = ad_to_bs_mismatches.len();
    println!("\n===== Conversion ground-truth report (BS 2081–2085) =====");
    println!("  rows checked         : {rows_checked}");
    println!("  BS->AD mismatches    : {b2a}  (MUST be 0)");
    println!("  AD->BS mismatches    : {a2b}  (MUST be 0)");
    if b2a > 0 {
        println!("\n  -- BS->AD mismatches --");
        for m in &bs_to_ad_mismatches {
            println!("    {m}");
        }
    }
    if a2b > 0 {
        println!("\n  -- AD->BS mismatches --");
        for m in &ad_to_bs_mismatches {
            println!("    {m}");
        }
    }
    println!("=========================================================\n");

    assert!(
        bs_to_ad_mismatches.is_empty() && ad_to_bs_mismatches.is_empty(),
        "engine diverges from Hamro Patro: {b2a} BS->AD + {a2b} AD->BS mismatch(es) over {rows_checked} rows"
    );
}
