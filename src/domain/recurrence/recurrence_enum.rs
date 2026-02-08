use crate::domain::recurrence::{
    AdRecurrenceRule, BsRecurrenceRule, RRuleParser, TithiRecurrenceRule,
};
use serde::{Deserialize, Deserializer, Serialize};

/// Recurrence type for events
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Recurrence {
    Once,
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
