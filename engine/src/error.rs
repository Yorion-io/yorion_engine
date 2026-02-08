use thiserror::Error;

#[derive(Error, Debug)]
pub enum BsCalendarError {
    #[error("Invalid BS year: {0}")]
    InvalidYear(u16),

    #[error("Invalid BS month: {0}")]
    InvalidMonth(u8),

    #[error("Invalid BS day: {0} for month {1}")]
    InvalidDay(u8, u8),

    #[error("Calendar data not found for year {0}")]
    CalendarDataNotFound(u16),

    #[error("Date conversion failed: {0}")]
    ConversionError(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Invalid recurrence rule: {0}")]
    InvalidRecurrenceRule(String),

    #[error("Invalid RRULE format: {0}")]
    InvalidRRule(String),

    #[error("Unsupported RRULE feature: {0}")]
    UnsupportedRRuleFeature(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Astronomical calculation failed: {0}")]
    AstronomicalError(String),
}

pub type Result<T> = std::result::Result<T, BsCalendarError>;
