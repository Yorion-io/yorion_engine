use crate::domain::tithi::{Location, Tithi};
use crate::ports::TithiOverrideProvider;
use chrono::{Datelike, NaiveDate};

// Auto-generated override data
include!(concat!(env!("OUT_DIR"), "/tithi_overrides_data.rs"));

/// Static Tithi override provider (zero-cost at runtime)
#[derive(Debug, Clone, Copy)]
pub struct StaticTithiOverrideProvider;

impl StaticTithiOverrideProvider {
    /// Create a new instance
    pub const fn new() -> Self {
        StaticTithiOverrideProvider
    }
}

impl TithiOverrideProvider for StaticTithiOverrideProvider {
    fn get_override(&self, date: NaiveDate, location: &Location) -> Option<Tithi> {
        // These overrides are specific to Nepal's social/religious calendar (official)
        if !location.follow_nepal_social_calendar {
            return None;
        }

        let key = (date.year(), date.month() as u8, date.day() as u8);
        TITHI_OVERRIDES
            .iter()
            .find(|(d, _)| *d == key)
            .map(|(_, t)| *t)
    }
}
