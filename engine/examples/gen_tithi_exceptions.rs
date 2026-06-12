// Throwaway generator: rebuilds data/tithi_exceptions.csv from the full set of
// engine-vs-almanac tithi divergences across the embedded reference years.
//
// For every dated row in tests/data/calendar/, it compares the engine's RAW
// (override-free) sunrise tithi against the almanac's tithi. Each divergence
// becomes one override row in the build-consumed schema:
//   BS Date,AD Date,Almanac Tithi,Almanac Paksha,Almanac Tithi Day,Generated Tithi,Generated Paksha,Generated Tithi Day
// build.rs reads col[1]=AD Date and col[2]=Almanac Tithi to pin get_tithi().
//
// Run from 05_engine/engine:  cargo run --example gen_tithi_exceptions

use yorion_engine::prelude::*;
use chrono::NaiveDate;

const YEARS: &[(u16, &str)] = &[
    (2079, include_str!("../tests/data/calendar/calendar_2079.csv")),
    (2080, include_str!("../tests/data/calendar/calendar_2080.csv")),
    (2081, include_str!("../tests/data/calendar/calendar_2081.csv")),
    (2082, include_str!("../tests/data/calendar/calendar_2082.csv")),
    (2083, include_str!("../tests/data/calendar/calendar_2083.csv")),
];

fn paksha_day(t: Tithi) -> (&'static str, u8) {
    let p = match t.paksha() {
        Paksha::Shukla => "Shukla",
        Paksha::Krishna => "Krishna",
    };
    (p, t.day_in_paksha())
}

fn main() {
    // Raw engine tithi: no override provider, so we measure the true divergences.
    let astro = AstronomicalService::new();

    let mut out = String::new();
    out.push_str("BS Date,AD Date,Almanac Tithi,Almanac Paksha,Almanac Tithi Day,Generated Tithi,Generated Paksha,Generated Tithi Day\n");

    let mut rows = 0usize;
    let mut diverged = 0usize;

    for (_year, csv) in YEARS {
        for line in csv.lines() {
            let mut cols = line.split(',');
            let bs_str = cols.next().unwrap_or("").trim();
            let ad_str = cols.next().unwrap_or("").trim();
            let tithi_str = cols.next().unwrap_or("").trim();

            if bs_str == "BS Date" || bs_str.is_empty() || tithi_str.is_empty() {
                continue;
            }

            let ad = match NaiveDate::parse_from_str(ad_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            };
            let almanac_tithi = match Tithi::from_name(tithi_str) {
                Some(t) => t,
                None => panic!("unrecognized almanac tithi name: {tithi_str:?}"),
            };

            rows += 1;

            let raw = astro
                .calculate_tithi_for_date(ad, &Location::kathmandu())
                .expect("raw tithi calc failed");

            if raw != almanac_tithi {
                diverged += 1;
                let (al_p, al_d) = paksha_day(almanac_tithi);
                let (g_p, g_d) = paksha_day(raw);
                out.push_str(&format!(
                    "{bs_str},{ad_str},{},{al_p},{al_d},{},{g_p},{g_d}\n",
                    almanac_tithi.name(),
                    raw.name(),
                ));
            }
        }
    }

    std::fs::write("data/tithi_exceptions.csv", &out)
        .expect("failed to write data/tithi_exceptions.csv");

    eprintln!("rows checked: {rows}");
    eprintln!("divergences written: {diverged}");
}
