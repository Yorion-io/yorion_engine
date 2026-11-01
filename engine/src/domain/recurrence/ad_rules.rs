use super::ad_expander::AdRecurrenceSet;
use crate::error::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

    /// Checks the rule parses as an RFC 5545 recurrence (see [`AdRecurrenceSet`]).
    pub fn validate(&self) -> Result<()> {
        self.recurrence_set().map(|_| ())
    }

    /// The parsed recurrence, ready to expand into dates.
    pub fn recurrence_set(&self) -> Result<AdRecurrenceSet> {
        AdRecurrenceSet::parse(&self.rrule)
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
