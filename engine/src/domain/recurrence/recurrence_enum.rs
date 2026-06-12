use crate::domain::recurrence::{
    AdRecurrenceRule, BsRecurrenceRule, TithiRecurrenceRule,
};
use crate::domain::recurrence::rrule_parser::RRuleParser;
use serde::{Deserialize, Deserializer, Serialize};

/// Recurrence rule for an event — BS solar, AD Gregorian, or lunar (panchanga).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Recurrence {
    Ad(AdRecurrenceRule),
    Bs(BsRecurrenceRule),
    Tithi(TithiRecurrenceRule),
}

impl<'de> Deserialize<'de> for Recurrence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rrule = String::deserialize(deserializer)?;
        RRuleParser::parse(&rrule).map_err(serde::de::Error::custom)
    }
}
