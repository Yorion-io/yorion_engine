use crate::domain::tithi::Location;
use chrono::NaiveDate;

/// A Nepal government/festival gazette holiday on a specific Gregorian date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holiday {
    /// English name, e.g. "Bijaya Dashami".
    pub name_en: &'static str,
    /// Nepali (Devanagari) name.
    pub name_ne: &'static str,
}

/// Port for looking up government/festival holidays by date.
///
/// This covers the gazette list only. Weekly closures (Saturday, and Sunday
/// from the 2082 two-day-weekend policy) are computed from the day of week in
/// the engine, not provided here.
pub trait HolidayProvider: Send + Sync {
    /// The gazette holiday on `date`, if any. Implementations may restrict to
    /// Nepal via `location.follow_nepal_social_calendar`.
    fn get_holiday(&self, date: NaiveDate, location: &Location) -> Option<Holiday>;

    /// Whether `date` is a gazette holiday for `location`.
    fn is_holiday(&self, date: NaiveDate, location: &Location) -> bool {
        self.get_holiday(date, location).is_some()
    }
}
