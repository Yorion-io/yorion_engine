use chrono::NaiveDate;
use crate::domain::tithi::{Tithi, Location};

/// Port for providing Tithi overrides for specific dates
pub trait TithiOverrideProvider: Send + Sync {
    /// Get the overridden Tithi for a given Gregorian date and location, if it exists
    fn get_override(&self, date: NaiveDate, location: &Location) -> Option<Tithi>;
    
    /// Check if a date and location has an override
    fn has_override(&self, date: NaiveDate, location: &Location) -> bool {
        self.get_override(date, location).is_some()
    }
}
