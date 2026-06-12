// BS-RRULE v2.0 conformance harness.
//
// Executes the portable test vectors in 00_docs/bs-rrule-test-vectors.json against
// the engine's RRuleParser. These vectors assert STRING-LEVEL parsing only:
// family detection (X-CALENDAR discriminator), parameter extraction, and which
// strings a conformant parser must reject. They do NOT assert expanded dates.
//
// NOTE on reject reasons: the spec assigns reason IDs (V1, V2, …) to rejections,
// but the parser currently returns a generic InvalidRRule without those codes, so
// this harness asserts *that* a rejection occurs (and reports the expected V-code
// in the failure message) rather than asserting the specific code. Tightening this
// would require the parser to carry reason IDs — flagged as optional follow-up.

use yorion_engine::domain::recurrence::{BsFrequency, Recurrence};
use yorion_engine::domain::recurrence::RRuleParser;
use serde::Deserialize;

const VECTORS_JSON: &str = include_str!("data/bs-rrule-test-vectors.json");

#[derive(Debug, Deserialize)]
struct VectorFile {
    vectors: Vec<Vector>,
}

#[derive(Debug, Deserialize)]
struct Vector {
    id: u32,
    input: String,
    #[allow(dead_code)]
    intent: String,
    expected: Expected,
}

#[derive(Debug, Deserialize)]
struct Expected {
    outcome: String,
    family: Option<String>,
    #[serde(default)]
    params: serde_json::Value,
    #[serde(rename = "rejectReason")]
    reject_reason: Option<String>,
}

/// Map a parsed Recurrence to its family token (matching the JSON vocabulary).
fn family_of(rec: &Recurrence) -> &'static str {
    match rec {
        Recurrence::Bs(_) => "BS",
        Recurrence::Tithi(_) => "PANCHANGA",
        Recurrence::Ad(_) => "AD",
    }
}

fn freq_token(f: &BsFrequency) -> &'static str {
    match f {
        BsFrequency::Daily => "DAILY",
        BsFrequency::Weekly => "WEEKLY",
        BsFrequency::Monthly => "MONTHLY",
        BsFrequency::Yearly => "YEARLY",
    }
}

/// Check the pinned params for a parsed rule. Returns Err(reason) on first mismatch.
fn check_params(rec: &Recurrence, params: &serde_json::Value) -> Result<(), String> {
    let obj = match params.as_object() {
        Some(o) => o,
        None => return Ok(()), // no params pinned
    };

    for (key, want) in obj {
        match key.as_str() {
            "freq" => {
                let got = match rec {
                    Recurrence::Bs(r) => freq_token(&r.frequency).to_string(),
                    // PANCHANGA is monthly by construction; AD freq isn't surfaced
                    // as a BsFrequency, so only assert for BS rules.
                    _ => continue,
                };
                let want = want.as_str().unwrap_or_default();
                if got != want {
                    return Err(format!("freq: want {want}, got {got}"));
                }
            }
            "dtstart" => {
                let anchor = match rec {
                    Recurrence::Bs(r) => Some(r.anchor),
                    Recurrence::Tithi(r) => Some(r.anchor),
                    _ => None,
                };
                if let Some(a) = anchor {
                    let got = format!("{:04}{:02}{:02}", a.year, a.month_u8(), a.day);
                    let want = want.as_str().unwrap_or_default();
                    if got != want {
                        return Err(format!("dtstart: want {want}, got {got}"));
                    }
                }
            }
            "count" => {
                let got = match rec {
                    Recurrence::Bs(r) => r.count,
                    Recurrence::Tithi(r) => r.count,
                    _ => None,
                };
                let want = want.as_u64().map(|v| v as u32);
                if got != want {
                    return Err(format!("count: want {want:?}, got {got:?}"));
                }
            }
            "byMonthDay" => {
                if let Recurrence::Bs(r) = rec {
                    let got: Vec<u64> = r
                        .by_month_day
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|d| d as u64)
                        .collect();
                    let want: Vec<u64> = want
                        .as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_u64()).collect())
                        .unwrap_or_default();
                    if got != want {
                        return Err(format!("byMonthDay: want {want:?}, got {got:?}"));
                    }
                }
            }
            "byDay" => {
                // BS family stores weekdays as u8 (0=SU..6=SA); AD wraps the raw
                // string. Assert for BS by mapping tokens; for AD, check substring.
                let want_tokens: Vec<String> = want
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_uppercase()))
                            .collect()
                    })
                    .unwrap_or_default();
                match rec {
                    Recurrence::Bs(r) => {
                        let got: Vec<String> = r
                            .by_day
                            .clone()
                            .unwrap_or_default()
                            .into_iter()
                            .map(wd_token)
                            .collect();
                        if got != want_tokens {
                            return Err(format!("byDay: want {want_tokens:?}, got {got:?}"));
                        }
                    }
                    Recurrence::Ad(r) => {
                        for t in &want_tokens {
                            if !r.rrule.to_uppercase().contains(t.as_str()) {
                                return Err(format!("byDay: AD rrule missing {t}"));
                            }
                        }
                    }
                    _ => {}
                }
            }
            "byTithi" => {
                if let Recurrence::Tithi(r) = rec {
                    let got: Vec<String> =
                        r.by_tithi.iter().map(|t| t.name().to_uppercase().replace(' ', "")).collect();
                    let want: Vec<String> = want
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_uppercase().replace(' ', "")))
                                .collect()
                        })
                        .unwrap_or_default();
                    // Compare as sets of normalized names; vectors may use bare day
                    // names (e.g. EKADASHI) that expand to two pakshas.
                    if !tithi_matches(&got, &want) {
                        return Err(format!("byTithi: want {want:?}, got {got:?}"));
                    }
                }
            }
            "paksha" => {
                if let Recurrence::Tithi(r) = rec {
                    let got = r
                        .paksha_filter
                        .map(|p| format!("{p:?}").to_uppercase())
                        .unwrap_or_default();
                    let want = want.as_str().unwrap_or_default().to_uppercase();
                    if got != want {
                        return Err(format!("paksha: want {want}, got {got}"));
                    }
                }
            }
            "skipAdhik" => {
                if let Recurrence::Tithi(r) = rec {
                    let want = want.as_bool().unwrap_or(true);
                    if r.skip_adhik != want {
                        return Err(format!(
                            "skipAdhik: want {want}, got {}",
                            r.skip_adhik
                        ));
                    }
                }
            }
            "byLunarMonth" => {
                if let Recurrence::Tithi(r) = rec {
                    let got: Vec<u64> = r
                        .by_lunar_month
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|m| m as u64)
                        .collect();
                    let want: Vec<u64> = want
                        .as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_u64()).collect())
                        .unwrap_or_default();
                    if got != want {
                        return Err(format!("byLunarMonth: want {want:?}, got {got:?}"));
                    }
                }
            }
            _ => {} // unrecognized pinned key — ignore
        }
    }
    Ok(())
}

fn wd_token(d: u8) -> String {
    match d {
        0 => "SU",
        1 => "MO",
        2 => "TU",
        3 => "WE",
        4 => "TH",
        5 => "FR",
        6 => "SA",
        _ => "??",
    }
    .to_string()
}

/// A bare tithi day-name (e.g. EKADASHI) expands to its two paksha-qualified
/// variants. Accept either an exact set match or the "every paksha" expansion.
fn tithi_matches(got: &[String], want: &[String]) -> bool {
    if got == want {
        return true;
    }
    // Each wanted bare name must be covered by the got set (which may carry the
    // SHUKLA/KRISHNA prefixes), or vice versa.
    want.iter().all(|w| {
        got.iter()
            .any(|g| g == w || g.ends_with(w.as_str()) || w.ends_with(g.as_str()))
    })
}

#[test]
fn conformance_all_vectors() {
    let file: VectorFile =
        serde_json::from_str(VECTORS_JSON).expect("test vectors JSON must deserialize");

    let mut failures: Vec<String> = Vec::new();

    for v in &file.vectors {
        let result = RRuleParser::parse(&v.input);

        match v.expected.outcome.as_str() {
            "reject" => match result {
                Ok(_) => failures.push(format!(
                    "vector {}: expected REJECT ({}) but parsed OK — input: {}",
                    v.id,
                    v.expected.reject_reason.as_deref().unwrap_or("?"),
                    v.input
                )),
                Err(e) => {
                    // If the vector pins a V-code, assert the parser returned exactly it.
                    if let Some(want) = &v.expected.reject_reason {
                        let got = e.reject_reason().map(|r| r.code());
                        if got != Some(want.as_str()) {
                            failures.push(format!(
                                "vector {}: reject reason want {want}, got {} — input: {}",
                                v.id,
                                got.unwrap_or("<none>"),
                                v.input
                            ));
                        }
                    }
                }
            },
            "parse" => match result {
                Err(e) => failures.push(format!(
                    "vector {}: expected PARSE but rejected ({e:?}) — input: {}",
                    v.id, v.input
                )),
                Ok(rec) => {
                    if let Some(want_family) = &v.expected.family {
                        let got_family = family_of(&rec);
                        if got_family != want_family.to_uppercase() {
                            failures.push(format!(
                                "vector {}: family want {want_family}, got {got_family} — input: {}",
                                v.id, v.input
                            ));
                            continue;
                        }
                    }
                    if let Err(reason) = check_params(&rec, &v.expected.params) {
                        failures.push(format!(
                            "vector {}: param mismatch [{reason}] — input: {}",
                            v.id, v.input
                        ));
                    }
                }
            },
            other => failures.push(format!("vector {}: unknown outcome {other}", v.id)),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} conformance vectors failed:\n{}",
        failures.len(),
        file.vectors.len(),
        failures.join("\n")
    );
}
