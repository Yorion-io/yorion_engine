use crate::domain::tithi::Location;
use crate::error::Result;
use crate::ports::TimeProvider;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};

/// System time provider using chrono and sun calculations
pub struct SystemTimeProvider;

impl SystemTimeProvider {
    pub fn new() -> Self {
        SystemTimeProvider
    }

    /// Calculate sunrise time using accurate astronomical algorithms
    fn calculate_sunrise(&self, date: NaiveDate, location: Location) -> Result<NaiveTime> {
        let astro = crate::services::AstronomicalService::new();
        astro.get_sunrise(date, location)
    }

    /// Calculate sunset time using accurate astronomical algorithms
    fn calculate_sunset(&self, date: NaiveDate, location: Location) -> Result<NaiveTime> {
        let astro = crate::services::AstronomicalService::new();
        astro.get_sunset(date, location)
    }
}

impl Default for SystemTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeProvider for SystemTimeProvider {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn sunrise_time(&self, date: NaiveDate, location: Location) -> Result<NaiveTime> {
        self.calculate_sunrise(date, location)
    }

    fn sunset_time(&self, date: NaiveDate, location: Location) -> Result<NaiveTime> {
        self.calculate_sunset(date, location)
    }
}
