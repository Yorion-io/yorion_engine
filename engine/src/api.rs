use crate::core_api::CalendarEngine;
use crate::domain::zodiac::DailyAstroInfo;
use crate::domain::Language;
use crate::domain::{
    BsDate, BsRecurrenceRule, EventInstance, Nakshatra, Tithi, TithiRecurrenceRule, ZodiacSign,
};
use crate::error::{BsCalendarError, Result};
use chrono::NaiveDate;
use std::sync::{OnceLock, RwLock};

// Singleton engine for Flutter
static ENGINE: OnceLock<CalendarEngine> = OnceLock::new();
static LANGUAGE: OnceLock<RwLock<Language>> = OnceLock::new();

fn get_engine() -> &'static CalendarEngine {
    ENGINE.get_or_init(CalendarEngine::new)
}

fn get_language_lock() -> &'static RwLock<Language> {
    LANGUAGE.get_or_init(|| RwLock::new(Language::default()))
}

/// Set the output language for formatting functions
pub fn set_language(lang: Language) {
    if let Ok(mut l) = get_language_lock().write() {
        *l = lang;
    }
}

/// Get the current language
pub fn get_language() -> Language {
    *get_language_lock().read().unwrap()
}

/// Format a BS date according to the current language and optional pattern
pub fn format_bs_date(date: BsDate, pattern: Option<String>) -> String {
    let engine = get_engine();
    let lang = get_language();
    let pattern = pattern.as_deref().unwrap_or("");

    engine.format_bs_date(date, pattern, lang)
}

/// Convert Bikram Sambat date to Gregorian date.
///
/// # Errors
/// Returns error if the BS date is invalid or out of range.
pub fn bs_to_gregorian(year: u16, month: u8, day: u8) -> Result<NaiveDate> {
    let engine = get_engine();
    let bs_date = BsDate::new(year, month, day)?;
    engine.bs_to_gregorian(bs_date)
}

/// Convert Gregorian date to Bikram Sambat date.
///
/// # Errors
/// Returns error if the Gregorian date is invalid or out of range.
pub fn gregorian_to_bs(year: i32, month: u32, day: u32) -> Result<BsDate> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    engine.gregorian_to_bs(gregorian)
}

/// Get the Tithi for a specific Gregorian date.
///
/// # Errors
/// Returns error if the date is invalid or tithi calculation fails.
pub fn get_tithi(year: i32, month: u32, day: u32) -> Result<Tithi> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    engine.get_tithi(gregorian)
}

/// Get the Sun's zodiac sign for a specific Gregorian date.
///
/// # Errors
/// Returns error if the date is invalid.
pub fn get_sun_zodiac(year: i32, month: u32, day: u32) -> Result<ZodiacSign> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    Ok(engine.get_sun_zodiac(gregorian))
}

/// Get the Moon's zodiac sign (Rashi) for a specific Gregorian date.
///
/// # Errors
/// Returns error if the date is invalid.
pub fn get_moon_zodiac(year: i32, month: u32, day: u32) -> Result<ZodiacSign> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    Ok(engine.get_moon_zodiac(gregorian))
}

/// Get the Nakshatra for a specific Gregorian date.
///
/// # Errors
/// Returns error if the date is invalid or nakshatra calculation fails.
pub fn get_nakshatra(year: i32, month: u32, day: u32) -> Result<Nakshatra> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    Ok(engine.get_nakshatra(gregorian))
}

/// Get complete daily astronomical information for a specific Gregorian date.
///
/// # Errors
/// Returns error if the date is invalid or astronomical calculations fail.
pub fn get_daily_astro_info(
    year: i32,
    month: u32,
    day: u32,
    location: crate::domain::tithi::Location,
) -> Result<DailyAstroInfo> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    engine.get_daily_astro_info(gregorian, location)
}

/// Get sunrise time for a specific Gregorian date and location.
///
/// # Errors
/// Returns error if the date is invalid or sunrise calculation fails.
pub fn get_sunrise_time(
    year: i32,
    month: u32,
    day: u32,
    location: crate::domain::tithi::Location,
) -> Result<chrono::NaiveTime> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    engine.get_sunrise(gregorian, location)
}

/// Get sunset time for a specific Gregorian date and location.
///
/// # Errors
/// Returns error if the date is invalid or sunset calculation fails.
pub fn get_sunset_time(
    year: i32,
    month: u32,
    day: u32,
    location: crate::domain::tithi::Location,
) -> Result<chrono::NaiveTime> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| BsCalendarError::InvalidDate("Invalid Gregorian Date".to_string()))?;
    engine.get_sunset(gregorian, location)
}

/// Generate BS recurring instances within a date range.
///
/// # Errors
/// Returns error if instance generation fails.
pub fn get_bs_recurring_instances(
    rule: &crate::domain::recurrence::BsRecurrenceRule,
    start: BsDate,
    end: BsDate,
) -> Result<Vec<BsDate>> {
    let engine = get_engine();
    engine.generate_bs_instances(rule, start, end)
}

/// Generate Tithi recurring instances within a date range.
///
/// # Errors
/// Returns error if instance generation fails.
pub fn get_tithi_recurring_instances(
    event_id: &str,
    title: &str,
    rule: &crate::domain::recurrence::TithiRecurrenceRule,
    start: BsDate,
    end: BsDate,
    version: crate::domain::event::CalendarVersion,
    location: crate::domain::tithi::Location,
) -> Result<Vec<EventInstance>> {
    let engine = get_engine();
    engine.generate_tithi_instances(event_id, title, rule, start, end, version, location)
}
