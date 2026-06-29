// External ground-truth validation against a published Nepali Panchanga almanac.
// The reference rows are embedded locally under tests/data/calendar/ so this
// suite is self-contained.
//
// This is the only suite that checks the engine against an EXTERNAL authority
// rather than against itself or hand-pinned test values. For every dated row in
// the almanac (BS 2079–2083 — the recent years treated as actual/trusted data;
// earlier years are not validated here), it asserts:
//
//   1. CONVERSION: engine.bs_to_gregorian(bs) == the AD date the almanac lists.
//   2. TITHI:      engine.get_tithi(ad)       == the tithi the almanac lists.
//
// `get_tithi` applies the engine's tithi overrides first (data/tithi_exceptions.csv,
// which is itself derived from the almanac), so this validates the engine AS CORRECTED.
// Any mismatch the harness reports is therefore a divergence NOT yet captured as an
// override — that list is the deliverable.
//
// Policy (per design decision): EXACT match, no tolerance. Conversion mismatches
// are hard bugs (must be 0). Tithi mismatches are the candidate-override set.
//
// Run with output:  cargo test --test almanac_ground_truth -- --nocapture

use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::bs_date::BsDate;
use yorion_engine::domain::tithi::Tithi;
use chrono::NaiveDate;

/// The almanac years treated as actual/trusted data
/// (BS 2079–2083), embedded at compile time from the suite's own
/// tests/data/calendar/ copies. (year, csv_contents). A moved data dir fails
/// compilation loudly — intended.
const YEARS: &[(u16, &str)] = &[
    (2079, include_str!("data/calendar/calendar_2079.csv")),
    (2080, include_str!("data/calendar/calendar_2080.csv")),
    (2081, include_str!("data/calendar/calendar_2081.csv")),
    (2082, include_str!("data/calendar/calendar_2082.csv")),
    (2083, include_str!("data/calendar/calendar_2083.csv")),
];

/// One parsed almanac row (only the columns we validate).
struct Row {
    bs: BsDate,
    ad: NaiveDate,
    tithi: Tithi,
}

/// Parse one CSV line `BS Date,AD Date,Tithi,Paksha,Tithi Day`.
/// Returns None for the header, blank lines, or rows with an empty/unknown tithi.
fn parse_row(line: &str) -> Option<Row> {
    let mut cols = line.split(',');
    let bs_str = cols.next()?.trim();
    let ad_str = cols.next()?.trim();
    let tithi_str = cols.next()?.trim();

    // Header or trailing/blank-tithi row.
    if bs_str == "BS Date" || bs_str.is_empty() || tithi_str.is_empty() {
        return None;
    }

    // BS Date = YYYY-MM-DD
    let mut bs_parts = bs_str.split('-');
    let by: u16 = bs_parts.next()?.parse().ok()?;
    let bm: u8 = bs_parts.next()?.parse().ok()?;
    let bd: u8 = bs_parts.next()?.parse().ok()?;
    let bs = BsDate::new(by, bm, bd).ok()?;

    let ad = NaiveDate::parse_from_str(ad_str, "%Y-%m-%d").ok()?;
    let tithi = Tithi::from_name(tithi_str)
        .unwrap_or_else(|| panic!("unrecognized almanac tithi name: {:?}", tithi_str));

    Some(Row { bs, ad, tithi })
}

#[test]
fn engine_matches_almanac() {
    let engine = CalendarEngine::new();

    let mut rows_checked = 0usize;
    let mut conversion_mismatches: Vec<String> = Vec::new();
    let mut tithi_mismatches: Vec<String> = Vec::new();

    for (year, csv) in YEARS {
        for line in csv.lines() {
            let Some(row) = parse_row(line) else { continue };
            rows_checked += 1;

            // 1. CONVERSION: engine BS->AD must equal the almanac's AD date.
            let engine_ad = engine
                .bs_to_gregorian(row.bs)
                .unwrap_or_else(|e| panic!("bs_to_gregorian failed for {} ({year}): {e}", row.bs));
            if engine_ad != row.ad {
                conversion_mismatches.push(format!(
                    "BS {} : engine AD {} != almanac AD {}",
                    row.bs, engine_ad, row.ad
                ));
                // If conversion is wrong, the AD-based tithi check below is
                // meaningless for this row — use the almanac's AD so the tithi
                // check is still apples-to-apples.
            }

            // 2. TITHI: engine sunrise-tithi (with overrides) for the almanac's
            //    AD date must equal the almanac's tithi.
            let engine_tithi = engine
                .get_tithi(row.ad)
                .unwrap_or_else(|e| panic!("get_tithi failed for AD {} ({year}): {e}", row.ad));
            if engine_tithi != row.tithi {
                tithi_mismatches.push(format!(
                    "BS {} (AD {}) : engine={:?} almanac={:?}",
                    row.bs, row.ad, engine_tithi, row.tithi
                ));
            }
        }
    }

    // ---- Report (always printed; use --nocapture to see on pass) ----
    let conv = conversion_mismatches.len();
    let tit = tithi_mismatches.len();
    let pct = if rows_checked > 0 {
        100.0 * tit as f64 / rows_checked as f64
    } else {
        0.0
    };
    println!("\n===== Almanac ground-truth report =====");
    println!("  rows checked        : {rows_checked}");
    println!("  conversion mismatches: {conv}  (MUST be 0)");
    println!("  tithi mismatches     : {tit}  ({pct:.3}% of rows)");
    if conv > 0 {
        println!("\n  -- conversion mismatches --");
        for m in &conversion_mismatches {
            println!("    {m}");
        }
    }
    if tit > 0 {
        println!("\n  -- tithi mismatches (candidate overrides) --");
        for m in &tithi_mismatches {
            println!("    {m}");
        }
    }
    println!("=======================================\n");

    assert!(
        conversion_mismatches.is_empty() && tithi_mismatches.is_empty(),
        "engine diverges from almanac: {conv} conversion + {tit} tithi mismatch(es) over {rows_checked} rows (see report above)"
    );
}

// ── Guards for TITHI_VERIFIED_THROUGH_BS ────────────────────────────────────
// The verified boundary is a human-maintained assertion, deliberately NOT derived
// from the override CSV (an exception-free year contributes zero rows yet is still
// verified). These guards don't DERIVE the boundary — they assert it can never
// silently CONTRADICT the data: it must cover every year we ship corrections or
// almanac ground truth for. They fail the build/CI if someone bumps the data
// without bumping the const.

/// The canonical override source (the same file build.rs compiles into TITHI_OVERRIDES).
const TITHI_EXCEPTIONS_CSV: &str = include_str!("../data/tithi_exceptions.csv");

/// Max BS year that appears in column 0 (BS Date) of the overrides CSV.
fn max_csv_override_bs_year() -> u16 {
    TITHI_EXCEPTIONS_CSV
        .lines()
        .skip(1) // header
        .filter_map(|line| line.split(',').next())
        .filter_map(|bs| bs.split('-').next())
        .filter_map(|y| y.trim().parse::<u16>().ok())
        .max()
        .unwrap_or(0)
}

#[test]
fn verified_boundary_covers_override_data() {
    let max_override = max_csv_override_bs_year();
    assert!(
        yorion_engine::core_api::TITHI_VERIFIED_THROUGH_BS >= max_override,
        "TITHI_VERIFIED_THROUGH_BS ({}) is below the max BS year with tithi overrides ({}). \
         Override rows exist for a year not marked verified — bump the const.",
        yorion_engine::core_api::TITHI_VERIFIED_THROUGH_BS,
        max_override,
    );
}

#[test]
fn verified_boundary_covers_almanac_ground_truth() {
    let max_almanac = YEARS.iter().map(|(y, _)| *y).max().unwrap_or(0);
    assert!(
        yorion_engine::core_api::TITHI_VERIFIED_THROUGH_BS >= max_almanac,
        "TITHI_VERIFIED_THROUGH_BS ({}) is below the latest almanac-validated year ({}). \
         A year is validated against the almanac but not marked verified — bump the const.",
        yorion_engine::core_api::TITHI_VERIFIED_THROUGH_BS,
        max_almanac,
    );
}
