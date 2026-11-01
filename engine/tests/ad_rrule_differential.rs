//! Differential test: the built-in AD recurrence expander must produce the same
//! dates as the `rrule` crate (the implementation it replaced, now a
//! dev-dependency) on a large set of pseudo-random RFC 5545 rules.

use chrono::{Duration, NaiveDate};
use std::str::FromStr;
use yorion_engine::domain::recurrence::AdRecurrenceSet;

/// Small deterministic PRNG (xorshift64*), so failures are reproducible.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }
}

const DAYS: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];

fn nonzero(rng: &mut Rng, max: i64) -> i64 {
    let n = rng.range(1, max);
    if rng.chance(30) {
        -n
    } else {
        n
    }
}

fn list(rng: &mut Rng, mut item: impl FnMut(&mut Rng) -> String) -> String {
    let n = rng.range(1, 3);
    (0..n).map(|_| item(rng)).collect::<Vec<_>>().join(",")
}

fn fmt(dt: chrono::NaiveDateTime) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Returns (rule text, window start, window end).
fn random_case(rng: &mut Rng) -> (String, NaiveDate, NaiveDate) {
    let base = NaiveDate::from_ymd_opt(1995, 1, 1).unwrap();
    let dtstart_date = base + Duration::days(rng.range(0, 40 * 365));
    let hour = [0, 9, 23][rng.below(3) as usize];
    let dtstart = dtstart_date.and_hms_opt(hour, 0, 0).unwrap();

    let freq = ["DAILY", "WEEKLY", "MONTHLY", "YEARLY"][rng.below(4) as usize];
    let ordinals_ok = matches!(freq, "MONTHLY" | "YEARLY");
    let mut parts = vec![format!("FREQ={freq}")];
    if rng.chance(40) {
        parts.push(format!("INTERVAL={}", rng.range(1, 4)));
    }
    if rng.chance(35) {
        parts.push(format!("COUNT={}", rng.range(1, 40)));
    }
    if rng.chance(25) {
        let until =
            dtstart + Duration::days(rng.range(0, 1500)) + Duration::hours(rng.range(-12, 12));
        parts.push(format!("UNTIL={}", fmt(until)));
    }
    let mut has_by = false;
    if rng.chance(30) {
        parts.push(format!(
            "BYMONTH={}",
            list(rng, |r| r.range(1, 12).to_string())
        ));
        has_by = true;
    }
    if rng.chance(25) {
        parts.push(format!(
            "BYMONTHDAY={}",
            list(rng, |r| nonzero(r, 31).to_string())
        ));
        has_by = true;
    }
    if freq == "YEARLY" && rng.chance(15) {
        parts.push(format!(
            "BYYEARDAY={}",
            list(rng, |r| nonzero(r, 366).to_string())
        ));
        has_by = true;
    }
    if rng.chance(40) {
        // All entries numbered or none: `rrule` reads a mix such as `-4FR,TH`
        // differently (see `mixed_ordinal_byday_follows_rfc` below).
        let numbered = ordinals_ok && rng.chance(50);
        parts.push(format!(
            "BYDAY={}",
            list(rng, |r| {
                let day = DAYS[r.below(7) as usize];
                if numbered {
                    format!("{}{day}", nonzero(r, 5))
                } else {
                    day.to_string()
                }
            })
        ));
        has_by = true;
    }
    if has_by && rng.chance(20) {
        parts.push(format!(
            "BYSETPOS={}",
            list(rng, |r| nonzero(r, 5).to_string())
        ));
    }
    if rng.chance(20) {
        parts.push(format!("WKST={}", DAYS[rng.below(7) as usize]));
    }

    let mut text = format!("DTSTART:{}\nRRULE:{}", fmt(dtstart), parts.join(";"));
    if rng.chance(10) {
        let ex = dtstart + Duration::days(rng.range(0, 60));
        text.push_str(&format!("\nEXDATE:{}", fmt(ex)));
    }

    let start = dtstart_date + Duration::days(rng.range(-200, 400));
    let end = start + Duration::days(rng.range(0, 1500));
    (text, start, end)
}

fn reference(text: &str, start: NaiveDate, end: NaiveDate) -> Option<Vec<NaiveDate>> {
    let set = rrule::RRuleSet::from_str(text).ok()?;
    let start_dt = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_dt = end.and_hms_opt(23, 59, 59).unwrap().and_utc();
    let mut dates: Vec<NaiveDate> = set
        .into_iter()
        .skip_while(|dt| dt < &start_dt)
        .take_while(|dt| dt <= &end_dt)
        .map(|dt| dt.date_naive())
        .collect();
    dates.dedup();
    Some(dates)
}

/// 2,000 cases by default. For a deeper run (the 20,000 used when the expander was
/// written; the time goes into `rrule`, which grinds on rules that never match):
///     AD_DIFF_CASES=20000 cargo test --release --test ad_rrule_differential
#[test]
fn matches_rrule_crate_on_random_rules() {
    let cases: usize = std::env::var("AD_DIFF_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_000);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let (mut compared, mut reference_rejected, mut we_rejected) = (0, 0, Vec::new());

    for _ in 0..cases {
        let (text, start, end) = random_case(&mut rng);
        let Some(expected) = reference(&text, start, end) else {
            reference_rejected += 1;
            continue;
        };
        match AdRecurrenceSet::parse(&text).and_then(|s| s.dates_between(start, end)) {
            Ok(got) => {
                assert_eq!(got, expected, "\nrule:\n{text}\nwindow {start}..{end}");
                compared += 1;
            }
            Err(e) => we_rejected.push(format!("{text} -> {e}")),
        }
    }

    println!(
        "compared {compared}, rejected by rrule {reference_rejected}, rejected only by us {}",
        we_rejected.len()
    );
    assert!(
        we_rejected.is_empty(),
        "rules rrule accepts but we reject:\n{}",
        we_rejected.join("\n")
    );
    assert!(
        compared * 10 > cases * 8,
        "too few comparable cases ({compared} of {cases})"
    );
}

/// Intentional difference, so pinned here rather than compared: with a BYDAY list
/// mixing numbered and plain days, `rrule` (like rrule.js and python-dateutil)
/// requires a day to satisfy both groups, so `-4FR,TH` yields nothing. RFC 5545
/// lists each entry as a day: the fourth-from-last Friday, plus every Thursday.
#[test]
fn mixed_ordinal_byday_follows_rfc() {
    let set = AdRecurrenceSet::parse("DTSTART:20240101\nRRULE:FREQ=MONTHLY;BYDAY=-4FR,TH;COUNT=6")
        .unwrap();
    let d = |m, day| NaiveDate::from_ymd_opt(2024, m, day).unwrap();
    let got = set.dates_between(d(1, 1), d(12, 31)).unwrap();
    assert_eq!(
        got,
        vec![d(1, 4), d(1, 5), d(1, 11), d(1, 18), d(1, 25), d(2, 1)]
    );
}
