use crate::domain::tithi::Location;
use crate::ports::{Holiday, HolidayProvider};
use chrono::{Datelike, NaiveDate};

// Auto-generated holiday data: HOLIDAYS: [((i32, u8, u8), &str, &str); N]
include!(concat!(env!("OUT_DIR"), "/holidays_data.rs"));

/// Static holiday provider backed by the baked gazette table (zero-cost at
/// runtime). Nepal-only: like tithi overrides, it applies solely when the
/// location follows the Nepal social calendar.
#[derive(Debug, Clone, Copy)]
pub struct StaticHolidayProvider {}

impl StaticHolidayProvider {
    pub const fn new() -> Self {
        StaticHolidayProvider {}
    }
}

impl Default for StaticHolidayProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl HolidayProvider for StaticHolidayProvider {
    fn get_holiday(&self, date: NaiveDate, location: &Location) -> Option<Holiday> {
        // Gazette holidays are specific to Nepal's official calendar.
        if !location.follow_nepal_social_calendar {
            return None;
        }

        let key = (date.year(), date.month() as u8, date.day() as u8);
        HOLIDAYS
            .iter()
            .find(|(d, _, _)| *d == key)
            .map(|(_, name_en, name_ne)| Holiday {
                name_en,
                name_ne,
            })
    }
}
