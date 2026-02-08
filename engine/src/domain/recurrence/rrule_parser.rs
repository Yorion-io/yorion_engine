use crate::domain::bs_date::{BsDate, BsMonth};
use crate::domain::recurrence::{
    AdRecurrenceRule, BsFrequency, BsRecurrenceRule, Recurrence, TithiRecurrenceRule,
};
use crate::domain::tithi::{Paksha, Tithi};
use crate::error::{BsCalendarError, Result};
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
                return Err(BsCalendarError::InvalidRRule(format!(
                    "Invalid parameter format: {}",
                    part
                )));
            }
        }

        Ok(params)
    }

    /// Parse RRULE string and return appropriate Recurrence enum
    pub fn parse(rrule: &str) -> Result<Recurrence> {
        let params = Self::parse_params(rrule)?;

        // Determine calendar type from RRULE
        let is_bs = params.get("X-CALENDAR").map(|v| v == "BS").unwrap_or(false);

        let is_tithi = params.contains_key("X-TITHI");

        // Parse into appropriate Recurrence variant
        if is_tithi {
            let rule = Self::parse_tithi_rrule(rrule)?;
            Ok(Recurrence::Tithi(rule))
        } else if is_bs {
            let rule = Self::parse_bs_rrule(rrule)?;
            Ok(Recurrence::Bs(rule))
        } else {
            let rule = Self::parse_ad_rrule(rrule)?;
            Ok(Recurrence::Ad(rule))
        }
    }

    /// Parse frequency from RRULE
    fn parse_frequency(freq_str: &str) -> Result<BsFrequency> {
        match freq_str.to_uppercase().as_str() {
            "DAILY" => Ok(BsFrequency::Daily),
            "WEEKLY" => Ok(BsFrequency::Weekly),
            "MONTHLY" => Ok(BsFrequency::Monthly),
            "YEARLY" => Ok(BsFrequency::Yearly),
            _ => Err(BsCalendarError::InvalidRRule(format!(
                "Invalid frequency: {}",
                freq_str
            ))),
        }
    }

    /// Parse BS date from DTSTART format (YYYYMMDD)
    fn parse_bs_date(date_str: &str) -> Result<BsDate> {
        if date_str.len() != 8 {
            return Err(BsCalendarError::InvalidRRule(format!(
                "Invalid date format: {}",
                date_str
            )));
        }

        let year: u16 = date_str[0..4]
            .parse()
            .map_err(|_| BsCalendarError::InvalidRRule("Invalid year".to_string()))?;
        let month: u8 = date_str[4..6]
            .parse()
            .map_err(|_| BsCalendarError::InvalidRRule("Invalid month".to_string()))?;
        let day: u8 = date_str[6..8]
            .parse()
            .map_err(|_| BsCalendarError::InvalidRRule("Invalid day".to_string()))?;

        BsDate::new(year, month, day)
    }

    /// Parse comma-separated list of months
    fn parse_months(months_str: &str) -> Result<Vec<BsMonth>> {
        months_str
            .split(',')
            .map(|m| {
                let month_num: u8 = m
                    .trim()
                    .parse()
                    .map_err(|_| BsCalendarError::InvalidRRule("Invalid month".to_string()))?;
                BsMonth::try_from(month_num).map_err(|_| {
                    BsCalendarError::InvalidRRule(format!("Invalid month: {}", month_num))
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
                _ => Err(BsCalendarError::InvalidRRule(format!(
                    "Invalid weekday: {}",
                    d
                ))),
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

    /// Parse Tithi from string
    fn parse_tithi(tithi_str: &str) -> Result<Tithi> {
        Tithi::from_name(tithi_str)
            .ok_or_else(|| BsCalendarError::InvalidRRule(format!("Invalid tithi: {}", tithi_str)))
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
            _ => Err(BsCalendarError::InvalidRRule(format!(
                "Invalid paksha: {}",
                paksha_str
            ))),
        }
    }

    /// Parse RRULE string into BsRecurrenceRule
    pub fn parse_bs_rrule(rrule: &str) -> Result<BsRecurrenceRule> {
        let params = Self::parse_params(rrule)?;

        // Required: FREQ and DTSTART
        let freq_str = params
            .get("FREQ")
            .ok_or_else(|| BsCalendarError::InvalidRRule("Missing FREQ".to_string()))?;
        let frequency = Self::parse_frequency(freq_str)?;

        let dtstart_str = params
            .get("DTSTART")
            .ok_or_else(|| BsCalendarError::InvalidRRule("Missing DTSTART".to_string()))?;
        let anchor = Self::parse_bs_date(dtstart_str)?;

        // Optional parameters
        let interval = params
            .get("INTERVAL")
            .map(|s| s.parse::<u16>())
            .transpose()
            .map_err(|_| BsCalendarError::InvalidRRule("Invalid INTERVAL".to_string()))?
            .unwrap_or(1);

        let count = params
            .get("COUNT")
            .map(|s| s.parse::<u32>())
            .transpose()
            .map_err(|_| BsCalendarError::InvalidRRule("Invalid COUNT".to_string()))?;

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
        let mut parts = vec![
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

        parts.push("X-CALENDAR=BS".to_string());

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

        // Required: X-TITHI and DTSTART
        let tithis_str = params
            .get("X-TITHI")
            .ok_or_else(|| BsCalendarError::InvalidRRule("Missing X-TITHI".to_string()))?;
        let by_tithi = Self::parse_tithis(tithis_str)?;

        let dtstart_str = params
            .get("DTSTART")
            .ok_or_else(|| BsCalendarError::InvalidRRule("Missing DTSTART".to_string()))?;
        let anchor = Self::parse_bs_date(dtstart_str)?;

        // Optional parameters
        let paksha_filter = params
            .get("X-PAKSHA")
            .map(|s| Self::parse_paksha(s))
            .transpose()?;

        let count = params
            .get("COUNT")
            .map(|s| s.parse::<u32>())
            .transpose()
            .map_err(|_| BsCalendarError::InvalidRRule("Invalid COUNT".to_string()))?;

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
            .map(|s| s.to_uppercase() == "TRUE")
            .unwrap_or(true);

        Ok(TithiRecurrenceRule {
            by_tithi,
            paksha_filter,
            anchor,
            count,
            until,
            by_month,
            by_lunar_month,
            skip_adhik,
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

        let mut parts = vec![
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
        parts.push("X-CALENDAR=BS".to_string());

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
