use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use crate::domain::tithi::Location;
use crate::error::Result;

/// Time provider trait - abstraction for time operations
/// 
/// Allows dependency injection of time sources for testing and different platforms
pub trait TimeProvider: Send + Sync {
    /// Get current UTC time
    fn now_utc(&self) -> DateTime<Utc>;
    
    /// Calculate sunrise time for a given date and location
    /// Returns the local time of sunrise
    fn sunrise_time(&self, date: NaiveDate, location: Location) -> Result<NaiveTime>;
    
    /// Calculate sunset time for a given date and location
    /// Returns the local time of sunset
    fn sunset_time(&self, date: NaiveDate, location: Location) -> Result<NaiveTime>;
}

/// Location provider trait - abstraction for location data
pub trait LocationProvider: Send + Sync {
    /// Get location by identifier
    fn get_location(&self, id: &str) -> Result<Location>;
    
    /// Get default location (Kathmandu)
    fn default_location(&self) -> Location {
        Location::KATHMANDU
    }
}
