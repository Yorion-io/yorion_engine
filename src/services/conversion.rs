use crate::domain::bs_date::{BsDate, BsMonth};
use crate::error::{BsCalendarError, Result};
use crate::ports::CalendarProvider;
use chrono::{Datelike, Duration, NaiveDate};
use std::sync::Arc;

/// BS ↔ Gregorian conversion service
///
/// Uses anchor-point algorithm: each BS year has a known 1st Baisakh date in Gregorian calendar.
/// Conversion is done by calculating day offsets from these anchor points.
pub struct ConversionService {
    calendar: Arc<dyn CalendarProvider>,
}

impl ConversionService {
    /// Create a new conversion service with a calendar provider
    pub fn new(calendar: Arc<dyn CalendarProvider>) -> Self {
        ConversionService { calendar }
    }

    /// Get a reference to the calendar provider
    pub fn calendar(&self) -> &Arc<dyn CalendarProvider> {
        &self.calendar
    }

    /// Convert Gregorian date to BS date
    ///
    /// Algorithm:
    /// 1. Approximate BS year (AD year + 57)
    /// 2. Get anchor point (1st Baisakh) for that year
    /// 3. Calculate day difference
    /// 4. If negative, use previous BS year
    /// 5. Count forward through months to find exact BS date
    pub fn gregorian_to_bs(&self, gregorian: NaiveDate) -> Result<BsDate> {
        let approximate_bs_year = (gregorian.year() + 57) as u16;

        // Try approximate year first
        if let Ok(bs_date) = self.try_convert_with_year(gregorian, approximate_bs_year) {
            return Ok(bs_date);
        }

        // If that fails, try previous year (date might be before 1st Baisakh)
        let prev_year = approximate_bs_year - 1;
        if let Ok(bs_date) = self.try_convert_with_year(gregorian, prev_year) {
            return Ok(bs_date);
        }

        Err(BsCalendarError::ConversionError(format!(
            "Could not convert Gregorian date {} to BS",
            gregorian
        )))
    }

    /// Try to convert using a specific BS year
    fn try_convert_with_year(&self, gregorian: NaiveDate, bs_year: u16) -> Result<BsDate> {
        let anchor = self.calendar.get_first_baisakh(bs_year)?;
        let days_diff = (gregorian - anchor).num_days();

        // If date is before anchor, it belongs to previous year
        if days_diff < 0 {
            return Err(BsCalendarError::ConversionError(
                "Date is before anchor".to_string(),
            ));
        }

        // Count forward from 1st Baisakh
        let year_months = self.calendar.get_year_months(bs_year)?;
        let mut remaining_days = days_diff as i32;
        let mut month_idx = 0;

        // Find the month
        while month_idx < 12 {
            let days_in_month = year_months[month_idx] as i32;

            if remaining_days < days_in_month {
                // Found the month
                let month = BsMonth::from_u8((month_idx + 1) as u8)?;
                let day = (remaining_days + 1) as u8; // +1 because days are 1-indexed

                return BsDate::from_parts(bs_year, month, day);
            }

            remaining_days -= days_in_month;
            month_idx += 1;
        }

        // If we've gone past all 12 months, date belongs to next year
        Err(BsCalendarError::ConversionError(
            "Date is after year end".to_string(),
        ))
    }

    /// Convert BS date to Gregorian date
    ///
    /// Algorithm:
    /// 1. Get anchor point (1st Baisakh) for the BS year
    /// 2. Count days from 1st Baisakh to target date
    /// 3. Add those days to the anchor date
    pub fn bs_to_gregorian(&self, bs_date: BsDate) -> Result<NaiveDate> {
        // Validate day against actual month length
        let actual_days = self.calendar.get_month_days(bs_date.year, bs_date.month)?;
        if bs_date.day > actual_days {
            return Err(BsCalendarError::InvalidDay(
                bs_date.day,
                bs_date.month.to_u8(),
            ));
        }

        let anchor = self.calendar.get_first_baisakh(bs_date.year)?;
        let year_months = self.calendar.get_year_months(bs_date.year)?;

        // Count days from 1st Baisakh to target date
        let mut day_count = 0i64;

        // Add full months before target month
        for month_idx in 0..(bs_date.month.to_u8() - 1) {
            day_count += year_months[month_idx as usize] as i64;
        }

        // Add days in target month (minus 1 because we start from day 1)
        day_count += (bs_date.day - 1) as i64;

        // Add days to anchor
        anchor
            .checked_add_signed(Duration::days(day_count))
            .ok_or_else(|| {
                BsCalendarError::ConversionError(format!(
                    "Date arithmetic overflow for BS date {}",
                    bs_date
                ))
            })
    }

    /// Validate a BS date against calendar data
    pub fn validate_bs_date(&self, bs_date: BsDate) -> Result<()> {
        if !self.calendar.has_year(bs_date.year) {
            return Err(BsCalendarError::CalendarDataNotFound(bs_date.year));
        }

        let actual_days = self.calendar.get_month_days(bs_date.year, bs_date.month)?;
        if bs_date.day > actual_days || bs_date.day < 1 {
            return Err(BsCalendarError::InvalidDay(
                bs_date.day,
                bs_date.month.to_u8(),
            ));
        }

        Ok(())
    }

    /// Clamp a BS date to valid range (useful for recurrence rules)
    /// Example: 2080 Chaitra 32 → 2080 Chaitra 30 (if Chaitra has 30 days)
    pub fn clamp_bs_date(&self, year: u16, month: BsMonth, day: u8) -> Result<BsDate> {
        let actual_days = self.calendar.get_month_days(year, month)?;
        let clamped_day = day.min(actual_days).max(1);

        BsDate::from_parts(year, month, clamped_day)
    }
}
