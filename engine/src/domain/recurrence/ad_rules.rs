use crate::error::{BsCalendarError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// AD recurrence rule wrapper around standard RFC 5545 string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdRecurrenceRule {
    /// The raw RRULE string (e.g., "FREQ=WEEKLY;BYDAY=MO")
    pub rrule: String,
}

impl AdRecurrenceRule {
    pub fn new(rrule: String) -> Result<Self> {
        let rule = Self { rrule };
        rule.validate()?;
        Ok(rule)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate by attempting to parse with rrule crate
        // We use UTC as the timezone for validation purposes to match our NaiveDate usage
        let rrule_str = if !self.rrule.to_uppercase().starts_with("RRULE:")
            && !self.rrule.to_uppercase().contains("DTSTART")
        {
            // Basic validation for fragments, though we expect full strings usually
            // If it's just a fragment, we might not be able to validate fully without DTSTART
            return Ok(());
        } else {
            &self.rrule
        };

        // We accept flexible parsing here, strict validation happens during generation
        if rrule::RRuleSet::from_str(rrule_str).is_err() {
            return Err(BsCalendarError::InvalidRecurrenceRule(
                "Invalid AD recurrence rule format".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert to RRULE string
    pub fn to_rrule(&self) -> String {
        self.rrule.clone()
    }
}

impl Serialize for AdRecurrenceRule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.rrule)
    }
}

impl<'de> Deserialize<'de> for AdRecurrenceRule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rrule = String::deserialize(deserializer)?;
        // We allow deserializing directly into the struct, validation happens on use or explicit check
        Ok(AdRecurrenceRule { rrule })
    }
}
