use crate::domain::bs_date::{BsDate, BsMonth};
use crate::domain::tithi::{Paksha, Tithi};
use crate::error::{BsCalendarError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Tithi recurrence rule for astronomical scheduling
///
/// Schedules events based on lunar days (tithis) rather than solar calendar dates.
/// Examples: Every Ekadashi, Every Purnima, Every Krishna Paksha Ashtami
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TithiRecurrenceRule {
    /// Specific Tithis to recur on
    pub by_tithi: Vec<Tithi>,

    /// Optional paksha filter (if None, matches both Shukla and Krishna)
    /// Useful for rules like "every Ekadashi" (both pakshas)
    pub paksha_filter: Option<Paksha>,

    /// Anchor date - the starting point for recurrence
    pub anchor: BsDate,

    /// Maximum number of occurrences (None = infinite)
    pub count: Option<u32>,

    /// End date for recurrence (None = infinite)
    pub until: Option<BsDate>,

    /// Filter by specific BS months (1-12)
    pub by_month: Option<Vec<BsMonth>>,

    /// Filter by specific lunar months (using BS month names)
    pub by_lunar_month: Option<Vec<BsMonth>>,

    /// Skip occurrences in Adhik (Extra) months
    pub skip_adhik: bool,

    /// Within each yearly cycle, keep only the first qualifying occurrence.
    /// Used for festivals like Bijaya Dashami where BYMONTH=6,7 would otherwise
    /// yield two hits per year — X-TAKE=FIRST keeps only the earliest one.
    pub take_first: bool,
}

impl TithiRecurrenceRule {
    /// Create a new tithi recurrence rule
    pub fn new(tithis: Vec<Tithi>, anchor: BsDate) -> Self {
        TithiRecurrenceRule {
            by_tithi: tithis,
            paksha_filter: None,
            anchor,
            count: None,
            until: None,
            by_month: None,
            by_lunar_month: None,
            skip_adhik: true,
            take_first: false,
        }
    }

    /// Create a rule for a specific tithi in a specific paksha
    pub fn with_paksha(tithis: Vec<Tithi>, paksha: Paksha, anchor: BsDate) -> Self {
        TithiRecurrenceRule {
            by_tithi: tithis,
            paksha_filter: Some(paksha),
            anchor,
            count: None,
            until: None,
            by_month: None,
            by_lunar_month: None,
            skip_adhik: true,
            take_first: false,
        }
    }

    /// Create a rule for Ekadashi (both pakshas)
    pub fn ekadashi(anchor: BsDate) -> Self {
        TithiRecurrenceRule {
            by_tithi: vec![Tithi::ShuklaEkadashi], // Will match both via paksha_filter = None
            paksha_filter: None,
            anchor,
            count: None,
            until: None,
            by_month: None,
            by_lunar_month: None,
            skip_adhik: true,
            take_first: false,
        }
    }

    /// Create a rule for Purnima (full moon)
    pub fn purnima(anchor: BsDate) -> Self {
        TithiRecurrenceRule::new(vec![Tithi::Purnima], anchor)
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

    /// Set month filter
    pub fn with_by_month(mut self, months: Vec<BsMonth>) -> Self {
        self.by_month = Some(months);
        self
    }

    /// Set lunar month filter
    pub fn with_by_lunar_month(mut self, months: Vec<BsMonth>) -> Self {
        self.by_lunar_month = Some(months);
        self
    }

    /// Set skip_adhik filter
    pub fn with_skip_adhik(mut self, skip: bool) -> Self {
        self.skip_adhik = skip;
        self
    }

    /// Within each BS year, keep only the first qualifying occurrence.
    pub fn with_take_first(mut self, take_first: bool) -> Self {
        self.take_first = take_first;
        self
    }

    /// Check if a tithi matches this rule
    pub fn matches_tithi(&self, tithi: Tithi) -> bool {
        // Check if any of the target tithis match
        for &target_tithi in &self.by_tithi {
            // Special case: Purnima and Amavasya are unique (no paksha variants)
            if target_tithi.is_purnima() || target_tithi.is_amavasya() {
                if tithi == target_tithi {
                    return true;
                }
                continue;
            }

            // Check if the tithi day matches
            let day_matches = tithi.day_in_paksha() == target_tithi.day_in_paksha();

            if day_matches {
                // Check paksha filter if specified
                if let Some(required_paksha) = self.paksha_filter {
                    if tithi.paksha() == required_paksha {
                        return true;
                    }
                } else {
                    // No paksha filter - matches both Shukla and Krishna
                    return true;
                }
            }
        }

        false
    }

    /// Validate the recurrence rule
    pub fn validate(&self) -> Result<()> {
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
        super::rrule_parser::RRuleParser::tithi_to_rrule(self)
    }

    /// Parse from RRULE string
    pub fn from_rrule(rrule: &str) -> Result<Self> {
        super::rrule_parser::RRuleParser::parse_tithi_rrule(rrule)
    }
}

impl Serialize for TithiRecurrenceRule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_rrule())
    }
}

impl<'de> Deserialize<'de> for TithiRecurrenceRule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rrule = String::deserialize(deserializer)?;
        Self::from_rrule(&rrule).map_err(serde::de::Error::custom)
    }
}
