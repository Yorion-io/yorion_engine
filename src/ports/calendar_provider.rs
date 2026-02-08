use chrono::NaiveDate;
use crate::domain::bs_date::BsMonth;
use crate::error::Result;

/// Calendar data provider trait - abstraction for accessing BS calendar data
pub trait CalendarProvider: Send + Sync {
    /// Get the number of days in a specific BS month
    fn get_month_days(&self, year: u16, month: BsMonth) -> Result<u8>;
    
    /// Get the Gregorian date for 1st Baisakh of a BS year (anchor point)
    fn get_first_baisakh(&self, year: u16) -> Result<NaiveDate>;
    
    /// Get all month days for a BS year
    fn get_year_months(&self, year: u16) -> Result<[u8; 12]>;
    
    /// Check if calendar data exists for a year
    fn has_year(&self, year: u16) -> bool;
    
    /// Get the calendar version identifier
    fn version(&self) -> &str;
    
    /// Check if this is official or projected data
    fn is_official(&self) -> bool;
}
