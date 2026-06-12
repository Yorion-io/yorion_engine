use crate::domain::bs_date::{BsDate, BsMonth};
use crate::domain::recurrence::{
    AdRecurrenceRule, BsFrequency, BsRecurrenceRule, Recurrence, TithiRecurrenceRule,
};
use crate::domain::tithi::{Paksha, Tithi};
use crate::error::{BsCalendarError, RRuleRejectReason, Result};
use std::collections::HashMap;

/// RRULE parser for converting between RRULE strings and recurrence rule structs
///
/// Supports standard RFC 5545 RRULE format with custom extensions:
/// - X-CALENDAR=BS - Indicates BS calendar mode
/// - X-TITHI=<name> - For tithi-based recurrence
/// - X-PAKSHA=SHUKLA|KRISHNA - Paksha filter
/// - X-BYLUNARMONTH=<months> - Lunar month filter
/// - X-SKIPADHIK=TRUE|FALSE - Skip adhik months
pub struct RRuleParser;

impl RRuleParser {
    /// Parse an RRULE string into a map of key-value pairs
    pub fn parse_params(rrule: &str) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();

        for part in rrule.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((key, value)) = part.split_once('=') {
                params.insert(key.to_uppercase(), value.to_string());
            } else {
                // V1: any parameter lacks `=`.
                return Err(BsCalendarError::rrule_rejected(
                    RRuleRejectReason::V1,
                    format!("Invalid parameter format: {}", part),
                ));
            }
        }

        Ok(params)
    }

    /// Parse RRULE string and return appropriate Recurrence enum
    pub fn parse(rrule: &str) -> Result<Recurrence> {
        let params = Self::parse_params(rrule)?;

        // BS-RRULE v2.0: family is selected by the single `X-CALENDAR` discriminator
        // (PANCHANGA | BS | AD). The value is case-insensitive (spec §2.2).
        let calendar = params
            .get("X-CALENDAR")
            .map(|v| v.trim().to_uppercase());

        // Legacy (v1.0) fallback: a tithi rule used to be detected by the mere
        // presence of `X-TITHI` (usually alongside `X-CALENDAR=BS`). Accept those
        // strings so already-stored rules keep parsing as the lunar family.
        let has_tithi = params.contains_key("X-TITHI");

        match calendar.as_deref() {
            Some("PANCHANGA") => Ok(Recurrence::Tithi(Self::parse_tithi_rrule(rrule)?)),
            Some("BS") if has_tithi => {
                // Legacy tithi string: X-CALENDAR=BS together with X-TITHI.
                Ok(Recurrence::Tithi(Self::parse_tithi_rrule(rrule)?))
            }
            Some("BS") => Ok(Recurrence::Bs(Self::parse_bs_rrule(rrule)?)),
            // No (or non-PANCHANGA) X-CALENDAR but X-TITHI present → legacy tithi.
            _ if has_tithi => Ok(Recurrence::Tithi(Self::parse_tithi_rrule(rrule)?)),
            _ => Ok(Recurrence::Ad(Self::parse_ad_rrule(rrule)?)),
        }
    }

    /// Parse frequency from RRULE
    fn parse_frequency(freq_str: &str) -> Result<BsFrequency> {
        match freq_str.to_uppercase().as_str() {
            "DAILY" => Ok(BsFrequency::Daily),
            "WEEKLY" => Ok(BsFrequency::Weekly),
            "MONTHLY" => Ok(BsFrequency::Monthly),
            "YEARLY" => Ok(BsFrequency::Yearly),
            // V3: FREQ is not one of the four defined frequencies.
            _ => Err(BsCalendarError::rrule_rejected(
                RRuleRejectReason::V3,
                format!("Invalid frequency: {}", freq_str),
            )),
        }
    }

    /// Parse BS date from DTSTART/UNTIL format (YYYYMMDD).
    ///
    /// V5: the value must be exactly 8 digits and its month must be 1–12.
    fn parse_bs_date(date_str: &str) -> Result<BsDate> {
        let v5 = |detail: &str| {
            BsCalendarError::rrule_rejected(RRuleRejectReason::V5, detail.to_string())
        };

        if date_str.len() != 8 || !date_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(v5(&format!("Date must be exactly 8 digits: {}", date_str)));
        }

        let year: u16 = date_str[0..4]
            .parse()
            .map_err(|_| v5("Invalid year"))?;
        let month: u8 = date_str[4..6]
            .parse()
            .map_err(|_| v5("Invalid month"))?;
        let day: u8 = date_str[6..8]
            .parse()
            .map_err(|_| v5("Invalid day"))?;

        if !(1..=12).contains(&month) {
            return Err(v5(&format!("Month outside 1-12: {}", month)));
        }

        // Out-of-range year/day are still spec-coded as V5 (malformed date token).
        BsDate::new(year, month, day).map_err(|_| {
            v5(&format!("Invalid date: {:04}{:02}{:02}", year, month, day))
        })
    }

    /// Parse comma-separated list of months (BYMONTH / X-BYLUNARMONTH).
    ///
    /// V7: a value outside 1–12 (or non-numeric).
    fn parse_months(months_str: &str) -> Result<Vec<BsMonth>> {
        months_str
            .split(',')
            .map(|m| {
                let month_num: u8 = m.trim().parse().map_err(|_| {
                    BsCalendarError::rrule_rejected(
                        RRuleRejectReason::V7,
                        format!("Invalid month value: {}", m.trim()),
                    )
                })?;
                BsMonth::try_from(month_num).map_err(|_| {
                    BsCalendarError::rrule_rejected(
                        RRuleRejectReason::V7,
                        format!("Month outside 1-12: {}", month_num),
                    )
                })
            })
            .collect()
    }

    /// Parse comma-separated list of month days
    fn parse_month_days(days_str: &str) -> Result<Vec<u8>> {
        days_str
            .split(',')
            .map(|d| {
                d.trim()
                    .parse::<u8>()
                    .map_err(|_| BsCalendarError::InvalidRRule("Invalid day".to_string()))
            })
            .collect()
    }

    /// Parse comma-separated list of week days (MO, TU, etc.)
    fn parse_week_days(days_str: &str) -> Result<Vec<u8>> {
        days_str
            .split(',')
            .map(|d| match d.trim().to_uppercase().as_str() {
                "SU" => Ok(0),
                "MO" => Ok(1),
                "TU" => Ok(2),
                "WE" => Ok(3),
                "TH" => Ok(4),
                "FR" => Ok(5),
                "SA" => Ok(6),
                // V8: BYDAY contains a token that is not a defined weekday code.
                _ => Err(BsCalendarError::rrule_rejected(
                    RRuleRejectReason::V8,
                    format!("Invalid weekday: {}", d),
                )),
            })
            .collect()
    }

    /// Convert week day number (0-6) to string (SU, MO, etc.)
    fn week_day_to_string(day: u8) -> &'static str {
        match day {
            0 => "SU",
            1 => "MO",
            2 => "TU",
            3 => "WE",
            4 => "TH",
            5 => "FR",
            6 => "SA",
            _ => "SU",
        }
    }

    /// Parse Tithi from string.
    ///
    /// V9: an unrecognized tithi name.
    fn parse_tithi(tithi_str: &str) -> Result<Tithi> {
        Tithi::from_name(tithi_str).ok_or_else(|| {
            BsCalendarError::rrule_rejected(
                RRuleRejectReason::V9,
                format!("Invalid tithi: {}", tithi_str),
            )
        })
    }

    /// Parse comma-separated list of Tithis
    fn parse_tithis(tithis_str: &str) -> Result<Vec<Tithi>> {
        tithis_str.split(',').map(Self::parse_tithi).collect()
    }

    /// Parse Paksha from string
    fn parse_paksha(paksha_str: &str) -> Result<Paksha> {
        match paksha_str.to_uppercase().as_str() {
            "SHUKLA" => Ok(Paksha::Shukla),
            "KRISHNA" => Ok(Paksha::Krishna),
            // V10: X-PAKSHA value other than SHUKLA/KRISHNA.
            _ => Err(BsCalendarError::rrule_rejected(
                RRuleRejectReason::V10,
                format!("Invalid paksha: {}", paksha_str),
            )),
        }
    }

    /// Parse an optional positive `u16` parameter (e.g. INTERVAL).
    ///
    /// V6: the parameter is present but not a positive integer (zero or
    /// non-numeric both fail).
    fn parse_positive_u16(params: &HashMap<String, String>, key: &str) -> Result<Option<u16>> {
        match params.get(key) {
            None => Ok(None),
            Some(raw) => {
                let n: u16 = raw.trim().parse().map_err(|_| {
                    BsCalendarError::rrule_rejected(
                        RRuleRejectReason::V6,
                        format!("{} must be a positive integer: {}", key, raw),
                    )
                })?;
                if n == 0 {
                    return Err(BsCalendarError::rrule_rejected(
                        RRuleRejectReason::V6,
                        format!("{} must be a positive integer: {}", key, raw),
                    ));
                }
                Ok(Some(n))
            }
        }
    }

    /// Parse an optional positive `u32` parameter (e.g. COUNT).
    ///
    /// V6: the parameter is present but not a positive integer (zero or
    /// non-numeric both fail).
    fn parse_positive_u32(params: &HashMap<String, String>, key: &str) -> Result<Option<u32>> {
        match params.get(key) {
            None => Ok(None),
            Some(raw) => {
                let n: u32 = raw.trim().parse().map_err(|_| {
                    BsCalendarError::rrule_rejected(
                        RRuleRejectReason::V6,
                        format!("{} must be a positive integer: {}", key, raw),
                    )
                })?;
                if n == 0 {
                    return Err(BsCalendarError::rrule_rejected(
                        RRuleRejectReason::V6,
                        format!("{} must be a positive integer: {}", key, raw),
                    ));
                }
                Ok(Some(n))
            }
        }
    }

    /// Parse RRULE string into BsRecurrenceRule
    pub fn parse_bs_rrule(rrule: &str) -> Result<BsRecurrenceRule> {
        let params = Self::parse_params(rrule)?;

        // Required: FREQ (V2) and DTSTART (V4)
        let freq_str = params.get("FREQ").ok_or_else(|| {
            BsCalendarError::rrule_rejected(RRuleRejectReason::V2, "Missing FREQ")
        })?;
        let frequency = Self::parse_frequency(freq_str)?;

        let dtstart_str = params.get("DTSTART").ok_or_else(|| {
            BsCalendarError::rrule_rejected(RRuleRejectReason::V4, "Missing DTSTART")
        })?;
        let anchor = Self::parse_bs_date(dtstart_str)?;

        // Optional parameters. INTERVAL/COUNT must be positive integers (V6).
        let interval = Self::parse_positive_u16(&params, "INTERVAL")?.unwrap_or(1);
        let count = Self::parse_positive_u32(&params, "COUNT")?;

        let until = params
            .get("UNTIL")
            .map(|s| Self::parse_bs_date(s))
            .transpose()?;

        let by_month = params
            .get("BYMONTH")
            .map(|s| Self::parse_months(s))
            .transpose()?;

        let by_month_day = params
            .get("BYMONTHDAY")
            .map(|s| Self::parse_month_days(s))
            .transpose()?;

        let by_day = params
            .get("BYDAY")
            .map(|s| Self::parse_week_days(s))
            .transpose()?;

        Ok(BsRecurrenceRule {
            frequency,
            interval,
            anchor,
            by_month,
            by_month_day,
            by_day,
            count,
            until,
        })
    }

    /// Convert BsRecurrenceRule to RRULE string
    pub fn bs_to_rrule(rule: &BsRecurrenceRule) -> String {
        // Canonical order (BS-RRULE v2.0 §7): the X-CALENDAR discriminator leads.
        let mut parts = vec![
            "X-CALENDAR=BS".to_string(),
            format!("FREQ={}", Self::frequency_to_string(&rule.frequency)),
            format!(
                "DTSTART={:04}{:02}{:02}",
                rule.anchor.year,
                rule.anchor.month_u8(),
                rule.anchor.day
            ),
        ];

        if rule.interval != 1 {
            parts.push(format!("INTERVAL={}", rule.interval));
        }

        if let Some(count) = rule.count {
            parts.push(format!("COUNT={}", count));
        }

        if let Some(until) = &rule.until {
            parts.push(format!(
                "UNTIL={:04}{:02}{:02}",
                until.year,
                until.month_u8(),
                until.day
            ));
        }

        if let Some(months) = &rule.by_month {
            let months_str = months
                .iter()
                .map(|m| (*m as u8).to_string())
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("BYMONTH={}", months_str));
        }

        if let Some(days) = &rule.by_month_day {
            let days_str = days
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("BYMONTHDAY={}", days_str));
        }

        if let Some(days) = &rule.by_day {
            let days_str = days
                .iter()
                .map(|&d| Self::week_day_to_string(d))
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("BYDAY={}", days_str));
        }

        parts.join(";")
    }

    /// Parse RRULE string into AdRecurrenceRule
    pub fn parse_ad_rrule(rrule: &str) -> Result<AdRecurrenceRule> {
        // Just wrap the string, the struct validates it on creation/usage
        AdRecurrenceRule::new(rrule.to_string())
    }

    /// Convert AdRecurrenceRule to RRULE string
    pub fn ad_to_rrule(rule: &AdRecurrenceRule) -> String {
        rule.to_rrule()
    }
    /// Parse RRULE string into TithiRecurrenceRule
    pub fn parse_tithi_rrule(rrule: &str) -> Result<TithiRecurrenceRule> {
        let params = Self::parse_params(rrule)?;

        // Required: X-TITHI (V9 — missing/empty) and DTSTART (V4).
        let tithis_str = params.get("X-TITHI").filter(|s| !s.trim().is_empty()).ok_or_else(|| {
            BsCalendarError::rrule_rejected(RRuleRejectReason::V9, "Missing X-TITHI")
        })?;
        let by_tithi = Self::parse_tithis(tithis_str)?;

        let dtstart_str = params.get("DTSTART").ok_or_else(|| {
            BsCalendarError::rrule_rejected(RRuleRejectReason::V4, "Missing DTSTART")
        })?;
        let anchor = Self::parse_bs_date(dtstart_str)?;

        // Optional parameters
        let paksha_filter = params
            .get("X-PAKSHA")
            .map(|s| Self::parse_paksha(s))
            .transpose()?;

        // COUNT must be a positive integer (V6).
        let count = Self::parse_positive_u32(&params, "COUNT")?;

        let until = params
            .get("UNTIL")
            .map(|s| Self::parse_bs_date(s))
            .transpose()?;

        let by_month = params
            .get("BYMONTH")
            .map(|s| Self::parse_months(s))
            .transpose()?;

        let by_lunar_month = params
            .get("X-BYLUNARMONTH")
            .map(|s| Self::parse_months(s))
            .transpose()?;

        let skip_adhik = params
            .get("X-SKIPADHIK")
            .map(|s| {
                let v = s.trim().to_uppercase();
                v == "TRUE" || v == "1" || v == "YES"
            })
            .unwrap_or(true);

        let take_first = if let Some(s) = params.get("X-TAKE") {
            let v = s.trim().to_uppercase();
            if v != "FIRST" {
                return Err(BsCalendarError::rrule_rejected(
                    RRuleRejectReason::V11,
                    format!("X-TAKE must be FIRST, got: {}", s.trim()),
                ));
            }
            true
        } else {
            false
        };

        Ok(TithiRecurrenceRule {
            by_tithi,
            paksha_filter,
            anchor,
            count,
            until,
            by_month,
            by_lunar_month,
            skip_adhik,
            take_first,
        })
    }

    /// Convert TithiRecurrenceRule to RRULE string
    pub fn tithi_to_rrule(rule: &TithiRecurrenceRule) -> String {
        let tithis_str = rule
            .by_tithi
            .iter()
            .map(|t| t.name().to_uppercase())
            .collect::<Vec<_>>()
            .join(",");

        // Canonical order (BS-RRULE v2.0 §7): the X-CALENDAR discriminator leads.
        let mut parts = vec![
            "X-CALENDAR=PANCHANGA".to_string(),
            "FREQ=MONTHLY".to_string(),
            format!(
                "DTSTART={:04}{:02}{:02}",
                rule.anchor.year,
                rule.anchor.month_u8(),
                rule.anchor.day
            ),
            format!("X-TITHI={}", tithis_str),
        ];

        if let Some(paksha) = &rule.paksha_filter {
            parts.push(format!("X-PAKSHA={}", Self::paksha_to_string(paksha)));
        }

        if let Some(count) = rule.count {
            parts.push(format!("COUNT={}", count));
        }

        if let Some(until) = &rule.until {
            parts.push(format!(
                "UNTIL={:04}{:02}{:02}",
                until.year,
                until.month_u8(),
                until.day
            ));
        }

        if let Some(months) = &rule.by_month {
            let months_str = months
                .iter()
                .map(|m| (*m as u8).to_string())
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("BYMONTH={}", months_str));
        }

        if let Some(lunar_months) = &rule.by_lunar_month {
            let months_str = lunar_months
                .iter()
                .map(|m| (*m as u8).to_string())
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("X-BYLUNARMONTH={}", months_str));
        }

        parts.push(format!(
            "X-SKIPADHIK={}",
            if rule.skip_adhik { "TRUE" } else { "FALSE" }
        ));

        if rule.take_first {
            parts.push("X-TAKE=FIRST".to_string());
        }

        parts.join(";")
    }

    fn frequency_to_string(freq: &BsFrequency) -> &'static str {
        match freq {
            BsFrequency::Daily => "DAILY",
            BsFrequency::Weekly => "WEEKLY",
            BsFrequency::Monthly => "MONTHLY",
            BsFrequency::Yearly => "YEARLY",
        }
    }

    fn paksha_to_string(paksha: &Paksha) -> &'static str {
        match paksha {
            Paksha::Shukla => "SHUKLA",
            Paksha::Krishna => "KRISHNA",
        }
    }
}
