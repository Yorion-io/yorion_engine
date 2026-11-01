//! Date-level RFC 5545 recurrence for the AD (Gregorian) family.
//!
//! The engine works in calendar dates, so this expander parses an iCalendar
//! recurrence (`DTSTART`, `RRULE`, `RDATE`, `EXDATE`) and returns the dates it
//! falls on. It replaces the `rrule` crate, which pulled in a regex engine and
//! the whole IANA time-zone database only to have every result truncated to a
//! date. A differential test (`rrule` is kept as a dev-dependency) checks that
//! both produce the same dates on thousands of random rules.
//!
//! Supported: `FREQ=DAILY|WEEKLY|MONTHLY|YEARLY`, `INTERVAL`, `COUNT`, `UNTIL`,
//! `BYMONTH`, `BYMONTHDAY`, `BYYEARDAY`, `BYDAY` (with ordinals for MONTHLY and
//! YEARLY), `BYSETPOS`, `WKST`, several `RRULE`s, `RDATE` and `EXDATE`.
//!
//! Deliberately not supported, with a clear error instead: sub-daily
//! frequencies, more than one value in `BYHOUR`/`BYMINUTE`/`BYSECOND` (both
//! would only repeat the same date), `BYWEEKNO`, `EXRULE` and `RDATE` periods.
//!
//! One intentional difference from those libraries: a `BYDAY` list that mixes
//! plain and numbered days (`BYDAY=-1FR,TH`) matches either, as RFC 5545 lists
//! each entry as a day ("the last Friday, and every Thursday"). rrule.js,
//! python-dateutil and the `rrule` crate require both, which yields no dates here.
//!
//! Dates are taken as written. A `DTSTART;TZID=Asia/Kathmandu:20240101T000000`
//! occurs on 1 January, the date in its own zone, without converting through
//! UTC. That is what a calendar user expects, and it fixes the `rrule`-based
//! code dropping such occurrences at the edge of a query window.

use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};

use crate::error::{BsCalendarError, Result};

/// A parsed recurrence set, ready to expand over a date window.
#[derive(Debug, Clone, PartialEq)]
pub struct AdRecurrenceSet {
    dtstart: Option<DateValue>,
    rules: Vec<Rule>,
    rdates: Vec<DateValue>,
    exdates: Vec<DateValue>,
}

/// A DATE or DATE-TIME value. `time` is `None` for a DATE (`VALUE=DATE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateValue {
    date: NaiveDate,
    time: Option<NaiveTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq)]
struct Rule {
    freq: Freq,
    interval: u32,
    count: Option<u32>,
    until: Option<DateValue>,
    by_month: Vec<u32>,
    by_month_day: Vec<i32>,
    by_year_day: Vec<i32>,
    by_day: Vec<(Option<i32>, Weekday)>,
    by_set_pos: Vec<i32>,
    wkst: Weekday,
}

fn invalid(msg: impl Into<String>) -> BsCalendarError {
    BsCalendarError::InvalidRecurrenceRule(msg.into())
}

impl AdRecurrenceSet {
    /// Parses an iCalendar recurrence. Accepts full content lines
    /// (`DTSTART:…\nRRULE:…`) or a bare rule (`FREQ=WEEKLY;BYDAY=MO`), which may
    /// carry an inline `DTSTART=YYYYMMDD` and `X-` parameters such as `X-CALENDAR=AD`.
    pub fn parse(input: &str) -> Result<Self> {
        let mut set = AdRecurrenceSet {
            dtstart: None,
            rules: Vec::new(),
            rdates: Vec::new(),
            exdates: Vec::new(),
        };
        let mut inline_dtstart = None;

        for line in unfold(input) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let upper = line.to_ascii_uppercase();
            let property = ["DTSTART", "RRULE", "EXDATE", "RDATE", "EXRULE"]
                .into_iter()
                .find(|p| {
                    upper.starts_with(p)
                        && matches!(upper.as_bytes().get(p.len()), Some(b':') | Some(b';'))
                });

            let Some(property) = property else {
                // A bare RRULE value.
                let (rule, dtstart) = parse_rule(line)?;
                set.rules.push(rule);
                inline_dtstart = inline_dtstart.or(dtstart);
                continue;
            };

            let colon = line
                .find(':')
                .ok_or_else(|| invalid(format!("{property} has no value")))?;
            let params = line[property.len()..colon].to_ascii_uppercase();
            let value = &line[colon + 1..];
            if params.contains("VALUE=PERIOD") {
                return Err(invalid("RDATE periods are not supported"));
            }

            match property {
                "DTSTART" => {
                    if set.dtstart.is_some() {
                        return Err(invalid("more than one DTSTART"));
                    }
                    set.dtstart = Some(parse_date_value(value)?);
                }
                "RRULE" => {
                    let (rule, dtstart) = parse_rule(value)?;
                    set.rules.push(rule);
                    inline_dtstart = inline_dtstart.or(dtstart);
                }
                "RDATE" => {
                    for v in value.split(',') {
                        set.rdates.push(parse_date_value(v)?);
                    }
                }
                "EXDATE" => {
                    for v in value.split(',') {
                        set.exdates.push(parse_date_value(v)?);
                    }
                }
                _ => {
                    return Err(invalid(
                        "EXRULE is not supported (it was removed from iCalendar in RFC 5545)",
                    ))
                }
            }
        }

        if set.dtstart.is_none() {
            set.dtstart = inline_dtstart;
        }
        if set.rules.is_empty() && set.rdates.is_empty() {
            return Err(invalid("no RRULE or RDATE"));
        }
        Ok(set)
    }

    /// The dates this recurrence falls on within `[start, end]`, ascending and
    /// without duplicates.
    ///
    /// Without a `DTSTART` the series is anchored at `start`, so a bare rule such
    /// as `FREQ=WEEKLY;BYDAY=MO` lists the Mondays in the window.
    pub fn dates_between(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<NaiveDate>> {
        if end < start {
            return Ok(Vec::new());
        }
        let dtstart = self.dtstart.unwrap_or(DateValue {
            date: start,
            time: None,
        });

        let mut dates = Vec::new();
        for rule in &self.rules {
            expand_rule(rule, dtstart, end, &mut |date| {
                if date >= start && !self.is_excluded(date, dtstart.time) {
                    dates.push(date);
                }
            })?;
        }
        for rdate in &self.rdates {
            let date = rdate.date;
            if date >= start && date <= end && !self.is_excluded(date, rdate.time.or(dtstart.time))
            {
                dates.push(date);
            }
        }

        dates.sort_unstable();
        dates.dedup();
        Ok(dates)
    }

    /// EXDATE without a time removes the whole date; with a time it removes the
    /// occurrence at exactly that time of day.
    fn is_excluded(&self, date: NaiveDate, time: Option<NaiveTime>) -> bool {
        self.exdates.iter().any(|ex| {
            ex.date == date
                && match ex.time {
                    None => true,
                    Some(t) => t == time.unwrap_or(NaiveTime::MIN),
                }
        })
    }
}

/// Joins folded content lines (a line starting with a space or tab continues the previous one).
fn unfold(input: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in input.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = raw.strip_prefix(' ').or_else(|| raw.strip_prefix('\t')) {
            if let Some(last) = lines.last_mut() {
                last.push_str(rest);
                continue;
            }
        }
        lines.push(raw.to_string());
    }
    lines
}

/// `YYYYMMDD`, `YYYYMMDDTHHMMSS` or `YYYYMMDDTHHMMSSZ`.
fn parse_date_value(value: &str) -> Result<DateValue> {
    let v = value.trim();
    let bad = || invalid(format!("invalid date value '{v}'"));
    let digits = |s: &str| -> Result<u32> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad());
        }
        s.parse().map_err(|_| bad())
    };

    let (date_part, time_part) = match v.find(['T', 't']) {
        Some(i) => (&v[..i], Some(&v[i + 1..])),
        None => (v, None),
    };
    if date_part.len() != 8 {
        return Err(bad());
    }
    let date = NaiveDate::from_ymd_opt(
        digits(&date_part[..4])? as i32,
        digits(&date_part[4..6])?,
        digits(&date_part[6..8])?,
    )
    .ok_or_else(bad)?;

    let time = match time_part {
        None => None,
        Some(t) => {
            let t = t.strip_suffix(['Z', 'z']).unwrap_or(t);
            if t.len() != 6 {
                return Err(bad());
            }
            Some(
                NaiveTime::from_hms_opt(digits(&t[..2])?, digits(&t[2..4])?, digits(&t[4..6])?)
                    .ok_or_else(bad)?,
            )
        }
    };
    Ok(DateValue { date, time })
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    Some(match s {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return None,
    })
}

fn parse_int_list(
    name: &str,
    value: &str,
    min: i32,
    max: i32,
    allow_negative: bool,
) -> Result<Vec<i32>> {
    value
        .split(',')
        .map(|item| {
            let n: i32 = item
                .trim()
                .parse()
                .map_err(|_| invalid(format!("{name}: '{item}' is not a number")))?;
            let in_range =
                (min..=max).contains(&n) || (allow_negative && (-max..=-min).contains(&n));
            if !in_range {
                return Err(invalid(format!("{name}: {n} is out of range")));
            }
            Ok(n)
        })
        .collect()
}

/// Parses the `NAME=VALUE;…` list of an RRULE. Also returns an inline `DTSTART=`
/// (a BS-RRULE convenience) when present.
fn parse_rule(value: &str) -> Result<(Rule, Option<DateValue>)> {
    let mut freq = None;
    let mut rule = Rule {
        freq: Freq::Daily,
        interval: 1,
        count: None,
        until: None,
        by_month: Vec::new(),
        by_month_day: Vec::new(),
        by_year_day: Vec::new(),
        by_day: Vec::new(),
        by_set_pos: Vec::new(),
        wkst: Weekday::Mon,
    };
    let mut dtstart = None;
    let mut seen = Vec::new();

    for part in value.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (name, val) = part
            .split_once('=')
            .ok_or_else(|| invalid(format!("'{part}' is not NAME=VALUE")))?;
        let name = name.trim().to_ascii_uppercase();
        let val = val.trim().to_ascii_uppercase();
        if name.starts_with("X-") {
            continue; // extension parameters, e.g. X-CALENDAR=AD
        }
        if seen.contains(&name) {
            return Err(invalid(format!("{name} appears more than once")));
        }
        seen.push(name.clone());

        match name.as_str() {
            "FREQ" => {
                freq = Some(match val.as_str() {
                    "DAILY" => Freq::Daily,
                    "WEEKLY" => Freq::Weekly,
                    "MONTHLY" => Freq::Monthly,
                    "YEARLY" => Freq::Yearly,
                    "HOURLY" | "MINUTELY" | "SECONDLY" => {
                        return Err(invalid(format!(
                            "FREQ={val} is not supported: recurrences are expanded to dates"
                        )))
                    }
                    _ => return Err(invalid(format!("unknown FREQ '{val}'"))),
                })
            }
            "INTERVAL" => {
                rule.interval = val
                    .parse()
                    .ok()
                    .filter(|n| *n >= 1)
                    .ok_or_else(|| invalid("INTERVAL must be a positive integer"))?
            }
            "COUNT" => {
                rule.count = Some(
                    val.parse()
                        .ok()
                        .filter(|n| *n >= 1)
                        .ok_or_else(|| invalid("COUNT must be a positive integer"))?,
                )
            }
            "UNTIL" => rule.until = Some(parse_date_value(&val)?),
            "DTSTART" => dtstart = Some(parse_date_value(&val)?),
            "BYMONTH" => {
                rule.by_month = parse_int_list("BYMONTH", &val, 1, 12, false)?
                    .into_iter()
                    .map(|m| m as u32)
                    .collect()
            }
            "BYMONTHDAY" => rule.by_month_day = parse_int_list("BYMONTHDAY", &val, 1, 31, true)?,
            "BYYEARDAY" => rule.by_year_day = parse_int_list("BYYEARDAY", &val, 1, 366, true)?,
            "BYSETPOS" => rule.by_set_pos = parse_int_list("BYSETPOS", &val, 1, 366, true)?,
            "BYDAY" => {
                for item in val.split(',').map(str::trim) {
                    let split = item
                        .len()
                        .checked_sub(2)
                        .ok_or_else(|| invalid(format!("BYDAY: invalid '{item}'")))?;
                    let wd = parse_weekday(&item[split..])
                        .ok_or_else(|| invalid(format!("BYDAY: invalid '{item}'")))?;
                    let ordinal = match &item[..split] {
                        "" => None,
                        n => {
                            let n: i32 = n
                                .parse()
                                .map_err(|_| invalid(format!("BYDAY: invalid '{item}'")))?;
                            if n == 0 || n.abs() > 53 {
                                return Err(invalid(format!("BYDAY: ordinal {n} is out of range")));
                            }
                            Some(n)
                        }
                    };
                    rule.by_day.push((ordinal, wd));
                }
            }
            "WKST" => {
                rule.wkst =
                    parse_weekday(&val).ok_or_else(|| invalid(format!("WKST: invalid '{val}'")))?
            }
            "BYHOUR" | "BYMINUTE" | "BYSECOND" => {
                // One value only sets the time of day, which does not move the date.
                if val.contains(',') {
                    return Err(invalid(format!("{name} with several values is not supported: recurrences are expanded to dates")));
                }
            }
            "BYWEEKNO" => return Err(invalid("BYWEEKNO is not supported")),
            _ => return Err(invalid(format!("unknown RRULE part '{name}'"))),
        }
    }

    rule.freq = freq.ok_or_else(|| invalid("FREQ is required"))?;
    if rule.by_day.iter().any(|(ord, _)| ord.is_some())
        && !matches!(rule.freq, Freq::Monthly | Freq::Yearly)
    {
        return Err(invalid(
            "BYDAY ordinals (e.g. 2MO) are only valid with FREQ=MONTHLY or YEARLY",
        ));
    }
    if !rule.by_set_pos.is_empty()
        && rule.by_month.is_empty()
        && rule.by_month_day.is_empty()
        && rule.by_year_day.is_empty()
        && rule.by_day.is_empty()
    {
        return Err(invalid("BYSETPOS needs another BYxxx part"));
    }
    Ok((rule, dtstart))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .map_or(28, |d| d.day())
}

fn days_in_year(year: i32) -> u32 {
    if NaiveDate::from_ymd_opt(year, 2, 29).is_some() {
        366
    } else {
        365
    }
}

/// Walks the rule's periods from DTSTART and calls `emit` for each occurrence
/// date up to `end`, honouring COUNT and UNTIL.
fn expand_rule(
    rule: &Rule,
    dtstart: DateValue,
    end: NaiveDate,
    emit: &mut dyn FnMut(NaiveDate),
) -> Result<()> {
    // RFC 5545 §3.3.10: with no BYxxx that picks days, the day comes from DTSTART.
    let mut rule = rule.clone();
    if rule.by_year_day.is_empty() && rule.by_month_day.is_empty() && rule.by_day.is_empty() {
        match rule.freq {
            Freq::Yearly => {
                if rule.by_month.is_empty() {
                    rule.by_month = vec![dtstart.date.month()];
                }
                rule.by_month_day = vec![dtstart.date.day() as i32];
            }
            Freq::Monthly => rule.by_month_day = vec![dtstart.date.day() as i32],
            Freq::Weekly => rule.by_day = vec![(None, dtstart.date.weekday())],
            Freq::Daily => {}
        }
    }

    let start_time = dtstart.time.unwrap_or(NaiveTime::MIN);
    let past_until = |date: NaiveDate| match rule.until {
        None => false,
        Some(DateValue {
            date: until,
            time: None,
        }) => date > until,
        // A DATE-TIME UNTIL bounds the occurrence's start instant (date + DTSTART time).
        Some(DateValue {
            date: until,
            time: Some(t),
        }) => date.and_time(start_time) > until.and_time(t),
    };
    let last_date = match rule.until {
        Some(u) if u.date < end => u.date,
        _ => end,
    };

    let interval = rule.interval as i64;
    let mut emitted = 0u32;
    let d0 = dtstart.date;

    for k in 0i64.. {
        let (period_start, candidates) = match rule.freq {
            Freq::Daily => {
                let day = d0 + Duration::days(k * interval);
                (day, vec![day])
            }
            Freq::Weekly => {
                let offset = (7 + d0.weekday().num_days_from_monday() as i64
                    - rule.wkst.num_days_from_monday() as i64)
                    % 7;
                let first = d0 - Duration::days(offset) + Duration::weeks(k * interval);
                // The first week starts at DTSTART itself, so BYSETPOS counts from
                // there (as rrule.js, python-dateutil and the rrule crate do).
                let from = if k == 0 { offset } else { 0 };
                (
                    first,
                    (from..7).map(|i| first + Duration::days(i)).collect(),
                )
            }
            Freq::Monthly => {
                let months = d0.month0() as i64 + k * interval;
                let year = d0.year() + (months / 12) as i32;
                let month = (months % 12) as u32 + 1;
                let first = NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| invalid("date out of range"))?;
                (
                    first,
                    (0..days_in_month(year, month))
                        .map(|i| first + Duration::days(i as i64))
                        .collect(),
                )
            }
            Freq::Yearly => {
                let year = d0.year() + (k * interval) as i32;
                let first = NaiveDate::from_ymd_opt(year, 1, 1)
                    .ok_or_else(|| invalid("date out of range"))?;
                let candidates = if rule.by_month.is_empty() {
                    (0..days_in_year(year))
                        .map(|i| first + Duration::days(i as i64))
                        .collect()
                } else {
                    // Only the listed months can match; skip scanning the rest of the year.
                    let mut months = rule.by_month.clone();
                    months.sort_unstable();
                    months.dedup();
                    months
                        .into_iter()
                        .filter_map(|m| {
                            NaiveDate::from_ymd_opt(year, m, 1).map(|f| (f, days_in_month(year, m)))
                        })
                        .flat_map(|(f, len)| (0..len).map(move |i| f + Duration::days(i as i64)))
                        .collect()
                };
                (first, candidates)
            }
        };
        if period_start > last_date {
            break;
        }

        let mut matched: Vec<NaiveDate> = candidates
            .into_iter()
            .filter(|d| day_matches(&rule, *d))
            .collect();
        if !rule.by_set_pos.is_empty() {
            matched = select_positions(&matched, &rule.by_set_pos);
        }

        for date in matched {
            if date < d0 {
                continue;
            }
            if date > end || past_until(date) {
                return Ok(());
            }
            emit(date);
            emitted += 1;
            if rule.count == Some(emitted) {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Applies every BYxxx filter (they combine with AND) to one candidate day.
fn day_matches(rule: &Rule, d: NaiveDate) -> bool {
    if !rule.by_month.is_empty() && !rule.by_month.contains(&d.month()) {
        return false;
    }
    if !rule.by_year_day.is_empty() {
        let len = days_in_year(d.year()) as i32;
        let n = d.ordinal() as i32;
        if !rule
            .by_year_day
            .iter()
            .any(|&y| if y > 0 { n == y } else { n == len + 1 + y })
        {
            return false;
        }
    }
    if !rule.by_month_day.is_empty() {
        let len = days_in_month(d.year(), d.month()) as i32;
        let n = d.day() as i32;
        if !rule
            .by_month_day
            .iter()
            .any(|&m| if m > 0 { n == m } else { n == len + 1 + m })
        {
            return false;
        }
    }
    if !rule.by_day.is_empty() {
        // Ordinals count within the month for MONTHLY, or YEARLY with BYMONTH;
        // otherwise within the year.
        let month_scope = rule.freq == Freq::Monthly || !rule.by_month.is_empty();
        let (scope_first, scope_last) = if month_scope {
            (
                d.with_day(1),
                d.with_day(days_in_month(d.year(), d.month())),
            )
        } else {
            (d.with_ordinal(1), d.with_ordinal(days_in_year(d.year())))
        };
        let (Some(first), Some(last)) = (scope_first, scope_last) else {
            return false;
        };
        let ok = rule.by_day.iter().any(|&(ordinal, wd)| {
            wd == d.weekday()
                && match ordinal {
                    None => true,
                    Some(n) if n > 0 => (d - first).num_days() / 7 + 1 == n as i64,
                    Some(n) => (last - d).num_days() / 7 + 1 == -n as i64,
                }
        });
        if !ok {
            return false;
        }
    }
    true
}

/// BYSETPOS: picks the nth (or nth-from-last) matches within a period.
fn select_positions(matched: &[NaiveDate], positions: &[i32]) -> Vec<NaiveDate> {
    let len = matched.len() as i32;
    let mut out: Vec<NaiveDate> = positions
        .iter()
        .filter_map(|&p| {
            let idx = if p > 0 { p - 1 } else { len + p };
            (0..len).contains(&idx).then(|| matched[idx as usize])
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn dates(rule: &str, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        AdRecurrenceSet::parse(rule)
            .unwrap()
            .dates_between(from, to)
            .unwrap()
    }

    #[test]
    fn bare_rule_without_dtstart_anchors_at_window_start() {
        let got = dates("FREQ=WEEKLY;BYDAY=MO", d(2024, 1, 1), d(2024, 1, 31));
        assert_eq!(
            got,
            vec![
                d(2024, 1, 1),
                d(2024, 1, 8),
                d(2024, 1, 15),
                d(2024, 1, 22),
                d(2024, 1, 29)
            ]
        );
        let got = dates(
            "X-CALENDAR=AD;FREQ=YEARLY;BYMONTH=1;BYMONTHDAY=1",
            d(2024, 1, 1),
            d(2026, 12, 31),
        );
        assert_eq!(got, vec![d(2024, 1, 1), d(2025, 1, 1), d(2026, 1, 1)]);
    }

    #[test]
    fn tzid_dates_are_local_and_not_shifted_through_utc() {
        let got = dates(
            "DTSTART;TZID=Asia/Kathmandu:20240101T000000\nRRULE:FREQ=DAILY;COUNT=2",
            d(2024, 1, 1),
            d(2024, 12, 31),
        );
        assert_eq!(got, vec![d(2024, 1, 1), d(2024, 1, 2)]);
    }

    #[test]
    fn single_byhour_is_accepted_and_several_are_rejected() {
        let got = dates(
            "DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;BYHOUR=9;COUNT=2",
            d(2024, 1, 1),
            d(2024, 1, 31),
        );
        assert_eq!(got, vec![d(2024, 1, 1), d(2024, 1, 2)]);
        assert!(
            AdRecurrenceSet::parse("DTSTART:20240101T000000Z\nRRULE:FREQ=DAILY;BYHOUR=9,17")
                .is_err()
        );
        assert!(AdRecurrenceSet::parse("FREQ=HOURLY").is_err());
    }

    #[test]
    fn last_friday_and_setpos() {
        let got = dates(
            "DTSTART:20240101T000000Z\nRRULE:FREQ=MONTHLY;BYDAY=-1FR;COUNT=3",
            d(2024, 1, 1),
            d(2024, 12, 31),
        );
        assert_eq!(got, vec![d(2024, 1, 26), d(2024, 2, 23), d(2024, 3, 29)]);
        // Last weekday of the month.
        let got = dates(
            "DTSTART:20240101\nRRULE:FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1;COUNT=2",
            d(2024, 1, 1),
            d(2024, 12, 31),
        );
        assert_eq!(got, vec![d(2024, 1, 31), d(2024, 2, 29)]);
    }

    #[test]
    fn exdate_rdate_and_folding() {
        let rule = "DTSTART:20240101T000000Z\r\nRRULE:FREQ=DAILY;\r\n COUNT=5\r\nEXDATE:20240103T000000Z\r\nRDATE;VALUE=DATE:20240110";
        assert_eq!(
            dates(rule, d(2024, 1, 1), d(2024, 1, 31)),
            vec![
                d(2024, 1, 1),
                d(2024, 1, 2),
                d(2024, 1, 4),
                d(2024, 1, 5),
                d(2024, 1, 10)
            ]
        );
    }

    #[test]
    fn rejects_malformed_rules() {
        for bad in [
            "",
            "FREQ=DAILY;INTERVAL=0",
            "FREQ=WEEKLY;BYDAY=1MO",
            "FREQ=MONTHLY;BYMONTHDAY=32",
            "FREQ=DAILY;BYSETPOS=1",
            "FREQ=YEARLY;BYWEEKNO=1",
            "FREQ=DAILY;FOO=1",
            "DTSTART:2024\nRRULE:FREQ=DAILY",
            "COUNT=3",
        ] {
            assert!(
                AdRecurrenceSet::parse(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }
}
