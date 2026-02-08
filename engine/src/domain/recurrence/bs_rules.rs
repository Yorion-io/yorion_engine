use crate::domain::bs_date::{BsDate, BsMonth};
use crate::error::{BsCalendarError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Recurrence frequency for BS-based rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BsFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// BS recurrence rule for scheduling events in Bikram Sambat calendar
///
/// Uses RRULE (RFC 5545) format for serialization with BS-specific extensions.
/// Supports BYMONTH and BYMONTHDAY filters with proper BS semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BsRecurrenceRule {
    /// Recurrence frequency
    pub frequency: BsFrequency,

    /// Interval between occurrences (e.g., every 2 weeks)
    pub interval: u16,

    /// Anchor date - the starting point for recurrence
    pub anchor: BsDate,

    /// Filter by specific BS months (1-12)
    pub by_month: Option<Vec<BsMonth>>,

    /// Filter by specific days of month (1-32, will be clamped)
    pub by_month_day: Option<Vec<u8>>,

    /// Filter by specific days of week (0-6, 0=Sunday)
    pub by_day: Option<Vec<u8>>,

    /// Maximum number of occurrences (None = infinite)
    pub count: Option<u32>,

    /// End date for recurrence (None = infinite)
    pub until: Option<BsDate>,
}

impl BsRecurrenceRule {
    /// Create a new recurrence rule with default values
    pub fn new(frequency: BsFrequency, anchor: BsDate) -> Self {
        BsRecurrenceRule {
            frequency,
            interval: 1,
            anchor,
            by_month: None,
            by_month_day: None,
            by_day: None,
            count: None,
            until: None,
        }
    }

    /// Set interval
    pub fn with_interval(mut self, interval: u16) -> Self {
        self.interval = interval.max(1);
        self
    }

    /// Set month filter
    pub fn with_by_month(mut self, months: Vec<BsMonth>) -> Self {
        self.by_month = Some(months);
        self
    }

    /// Set month day filter
    pub fn with_by_month_day(mut self, days: Vec<u8>) -> Self {
        self.by_month_day = Some(days);
        self
    }

    /// Set week day filter (0-6, 0=Sunday)
    pub fn with_by_day(mut self, days: Vec<u8>) -> Self {
        self.by_day = Some(days);
        self
    }

    /// Set count limit
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Set until date
    pub fn with_until(mut self, until: BsDate) -> Self {
        self.until = Some(until);
        self
    }

    /// Validate the recurrence rule
    pub fn validate(&self) -> Result<()> {
        if self.interval < 1 {
            return Err(BsCalendarError::InvalidRecurrenceRule(
                "Interval must be at least 1".to_string(),
            ));
        }

        // Validate by_month_day values
        if let Some(ref days) = self.by_month_day {
            for &day in days {
                if !(1..=32).contains(&day) {
                    return Err(BsCalendarError::InvalidRecurrenceRule(format!(
                        "Invalid day of month: {}",
                        day
                    )));
                }
            }
        }

        // Validate by_day values
        if let Some(ref days) = self.by_day {
            for &day in days {
                if day > 6 {
                    return Err(BsCalendarError::InvalidRecurrenceRule(format!(
                        "Invalid day of week: {}",
                        day
                    )));
                }
            }
        }

        // Validate until is after anchor
        if let Some(until) = self.until {
            if until < self.anchor {
                return Err(BsCalendarError::InvalidRecurrenceRule(
                    "Until date must be after anchor date".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Convert to RRULE string
    pub fn to_rrule(&self) -> String {
        super::rrule_parser::RRuleParser::bs_to_rrule(self)
    }

    /// Parse from RRULE string
    pub fn from_rrule(rrule: &str) -> Result<Self> {
        super::rrule_parser::RRuleParser::parse_bs_rrule(rrule)
    }
}

impl Serialize for BsRecurrenceRule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_rrule())
    }
}

impl<'de> Deserialize<'de> for BsRecurrenceRule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rrule = String::deserialize(deserializer)?;
        Self::from_rrule(&rrule).map_err(serde::de::Error::custom)
    }
}
