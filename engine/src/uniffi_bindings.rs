//! UniFFI bindings for BS Calendar Core
//!
//! This module provides a thin wrapper around the core API for UniFFI
//! to generate Swift, Kotlin, and Python bindings.

use crate::core_api::{CalendarDay as CoreCalendarDay, CalendarEngine as CoreEngine};
use crate::domain::recurrence::{
    AdRecurrenceRule as CoreAdRecurrenceRule, BsFrequency as CoreBsFrequency,
    BsRecurrenceRule as CoreBsRecurrenceRule, TithiRecurrenceRule as CoreTithiRecurrenceRule,
};
use crate::domain::tithi::{Location as CoreLocation, Paksha as CorePaksha, Tithi as CoreTithi};
use crate::domain::zodiac::{
    DailyAstroInfo as CoreDailyAstroInfo, Nakshatra as CoreNakshatra, ZodiacSign as CoreZodiacSign,
};
use crate::domain::{
    BsDate as CoreBsDate, CalendarVersion as CoreCalendarVersion,
    EventInstance as CoreEventInstance, Language as CoreLanguage,
};
use crate::error::BsCalendarError as CoreError;
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};

// uniffi::include_scaffolding!("uniffi");

// ============================================================================
// Data Structures (UDL Dictionaries/Enums)
// ============================================================================

#[derive(Debug, Clone)]
pub struct BsDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl From<CoreBsDate> for BsDate {
    fn from(d: CoreBsDate) -> Self {
        BsDate {
            year: d.year,
            month: d.month as u8, // Convert enum to u8
            day: d.day,
        }
    }
}

impl TryFrom<BsDate> for CoreBsDate {
    type Error = BsCalendarError;
    fn try_from(d: BsDate) -> Result<Self, Self::Error> {
        CoreBsDate::new(d.year, d.month, d.day).map_err(|_| BsCalendarError::InvalidDate)
    }
}

#[derive(Debug, Clone)]
pub struct GregorianDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl From<NaiveDate> for GregorianDate {
    fn from(d: NaiveDate) -> Self {
        GregorianDate {
            year: d.year(),
            month: d.month(),
            day: d.day(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub name: String,
    pub timezone_offset_mins: i32,
    pub follow_nepal_social_calendar: bool,
}

impl From<Location> for CoreLocation {
    fn from(l: Location) -> Self {
        CoreLocation::new(l.latitude, l.longitude, l.name, l.timezone_offset_mins)
    }
}

impl From<CoreLocation> for Location {
    fn from(l: CoreLocation) -> Self {
        Location {
            latitude: l.latitude,
            longitude: l.longitude,
            name: l.name.clone(),
            timezone_offset_mins: l.timezone_offset_mins,
            follow_nepal_social_calendar: l.follow_nepal_social_calendar,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeOfDay {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl From<NaiveTime> for TimeOfDay {
    fn from(t: NaiveTime) -> Self {
        TimeOfDay {
            hour: t.hour(),
            minute: t.minute(),
            second: t.second(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Nepali,
}

impl From<Language> for CoreLanguage {
    fn from(l: Language) -> Self {
        match l {
            Language::English => CoreLanguage::English,
            Language::Nepali => CoreLanguage::Nepali,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tithi {
    pub number: u8,
    pub name: String,
    pub paksha: Paksha,
}

impl From<CoreTithi> for Tithi {
    fn from(t: CoreTithi) -> Self {
        Tithi {
            number: t.day_in_paksha(),
            name: t.name().to_string(),
            paksha: t.paksha().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DailyAstroInfo {
    pub tithi: Tithi,
    pub sun_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
    pub nakshatra: Nakshatra,
    pub is_overridden: bool,
}

impl From<CoreDailyAstroInfo> for DailyAstroInfo {
    fn from(info: CoreDailyAstroInfo) -> Self {
        DailyAstroInfo {
            tithi: info.tithi.into(),
            sun_sign: info.sun_sign.into(),
            moon_sign: info.moon_sign.into(),
            nakshatra: info.nakshatra.into(),
            is_overridden: info.is_overridden,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalendarDay {
    pub bs_year: u16,
    pub bs_month: u8,
    pub bs_day: u8,
    pub gregorian_date: GregorianDate,
    pub day_of_week: u8,
    pub tithi: Tithi,
    pub sun_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
    pub nakshatra: Nakshatra,
    pub is_overridden: bool,
}

impl From<CoreCalendarDay> for CalendarDay {
    fn from(d: CoreCalendarDay) -> Self {
        CalendarDay {
            bs_year: d.bs_year,
            bs_month: d.bs_month, // already u8 in CoreCalendarDay
            bs_day: d.bs_day,
            gregorian_date: d.gregorian_date.into(),
            day_of_week: d.day_of_week, // core_api::CalendarDay uses u8 directly
            tithi: d.tithi.into(),
            sun_sign: d.sun_sign.into(),
            moon_sign: d.moon_sign.into(),
            nakshatra: d.nakshatra.into(),
            is_overridden: d.is_overridden,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonthCalendarData {
    pub days: Vec<CalendarDay>,
    pub year: u16,
    pub month: u8,
    pub days_in_month: u8,
    pub start_day_of_week: u8,
}

// ... (Error Mapping and straightforward Data Structures remain the same until Recurrence Rules)

// [Skipping unchanged structs/enums up to BsRecurrenceRule for brevity in replacement]
// ...

// uniffi::include_scaffolding!("uniffi");

#[derive(Debug, Clone)]
pub struct BsRecurrenceRule {
    pub frequency: BsFrequency,
    pub interval: u32,
    pub by_month_day: Option<Vec<u8>>,
    pub by_month: Option<Vec<u8>>,
    pub count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BsFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl From<CoreZodiacSign> for ZodiacSign {
    fn from(sign: CoreZodiacSign) -> Self {
        match sign {
            CoreZodiacSign::Mesh => ZodiacSign::Aries,
            CoreZodiacSign::Vrishabh => ZodiacSign::Taurus,
            CoreZodiacSign::Mithun => ZodiacSign::Gemini,
            CoreZodiacSign::Karka => ZodiacSign::Cancer,
            CoreZodiacSign::Simha => ZodiacSign::Leo,
            CoreZodiacSign::Kanya => ZodiacSign::Virgo,
            CoreZodiacSign::Tula => ZodiacSign::Libra,
            CoreZodiacSign::Vrishchik => ZodiacSign::Scorpio,
            CoreZodiacSign::Dhanu => ZodiacSign::Sagittarius,
            CoreZodiacSign::Makar => ZodiacSign::Capricorn,
            CoreZodiacSign::Kumbha => ZodiacSign::Aquarius,
            CoreZodiacSign::Meen => ZodiacSign::Pisces,
        }
    }
}

// Ensure the reverse mapping if needed (not currently used but good to have)
// Actually, we only implemented From<Core> for external.
// Do we need From<External> for Core? Only if inputs take ZodiacSign.
// get_daily_astro_info returns ZodiacSign, so we just need From<Core>.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nakshatra {
    Ashwini,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Mula,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishtha,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

impl From<CoreNakshatra> for Nakshatra {
    fn from(value: CoreNakshatra) -> Self {
        match value {
            CoreNakshatra::Ashwini => Nakshatra::Ashwini,
            CoreNakshatra::Bharani => Nakshatra::Bharani,
            CoreNakshatra::Krittika => Nakshatra::Krittika,
            CoreNakshatra::Rohini => Nakshatra::Rohini,
            CoreNakshatra::Mrigashira => Nakshatra::Mrigashira,
            CoreNakshatra::Ardra => Nakshatra::Ardra,
            CoreNakshatra::Punarvasu => Nakshatra::Punarvasu,
            CoreNakshatra::Pushya => Nakshatra::Pushya,
            CoreNakshatra::Ashlesha => Nakshatra::Ashlesha,
            CoreNakshatra::Magha => Nakshatra::Magha,
            CoreNakshatra::PurvaPhalguni => Nakshatra::PurvaPhalguni,
            CoreNakshatra::UttaraPhalguni => Nakshatra::UttaraPhalguni,
            CoreNakshatra::Hasta => Nakshatra::Hasta,
            CoreNakshatra::Chitra => Nakshatra::Chitra,
            CoreNakshatra::Swati => Nakshatra::Swati,
            CoreNakshatra::Vishakha => Nakshatra::Vishakha,
            CoreNakshatra::Anuradha => Nakshatra::Anuradha,
            CoreNakshatra::Jyeshtha => Nakshatra::Jyeshtha,
            CoreNakshatra::Mula => Nakshatra::Mula,
            CoreNakshatra::PurvaAshadha => Nakshatra::PurvaAshadha,
            CoreNakshatra::UttaraAshadha => Nakshatra::UttaraAshadha,
            CoreNakshatra::Shravana => Nakshatra::Shravana,
            CoreNakshatra::Dhanishtha => Nakshatra::Dhanishtha,
            CoreNakshatra::Shatabhisha => Nakshatra::Shatabhisha,
            CoreNakshatra::PurvaBhadrapada => Nakshatra::PurvaBhadrapada,
            CoreNakshatra::UttaraBhadrapada => Nakshatra::UttaraBhadrapada,
            CoreNakshatra::Revati => Nakshatra::Revati,
        }
    }
}

// Reverse mapping
impl From<Nakshatra> for CoreNakshatra {
    fn from(value: Nakshatra) -> Self {
        match value {
            Nakshatra::Ashwini => CoreNakshatra::Ashwini,
            Nakshatra::Bharani => CoreNakshatra::Bharani,
            Nakshatra::Krittika => CoreNakshatra::Krittika,
            Nakshatra::Rohini => CoreNakshatra::Rohini,
            Nakshatra::Mrigashira => CoreNakshatra::Mrigashira,
            Nakshatra::Ardra => CoreNakshatra::Ardra,
            Nakshatra::Punarvasu => CoreNakshatra::Punarvasu,
            Nakshatra::Pushya => CoreNakshatra::Pushya,
            Nakshatra::Ashlesha => CoreNakshatra::Ashlesha,
            Nakshatra::Magha => CoreNakshatra::Magha,
            Nakshatra::PurvaPhalguni => CoreNakshatra::PurvaPhalguni,
            Nakshatra::UttaraPhalguni => CoreNakshatra::UttaraPhalguni,
            Nakshatra::Hasta => CoreNakshatra::Hasta,
            Nakshatra::Chitra => CoreNakshatra::Chitra,
            Nakshatra::Swati => CoreNakshatra::Swati,
            Nakshatra::Vishakha => CoreNakshatra::Vishakha,
            Nakshatra::Anuradha => CoreNakshatra::Anuradha,
            Nakshatra::Jyeshtha => CoreNakshatra::Jyeshtha,
            Nakshatra::Mula => CoreNakshatra::Mula,
            Nakshatra::PurvaAshadha => CoreNakshatra::PurvaAshadha,
            Nakshatra::UttaraAshadha => CoreNakshatra::UttaraAshadha,
            Nakshatra::Shravana => CoreNakshatra::Shravana,
            Nakshatra::Dhanishtha => CoreNakshatra::Dhanishtha,
            Nakshatra::Shatabhisha => CoreNakshatra::Shatabhisha,
            Nakshatra::PurvaBhadrapada => CoreNakshatra::PurvaBhadrapada,
            Nakshatra::UttaraBhadrapada => CoreNakshatra::UttaraBhadrapada,
            Nakshatra::Revati => CoreNakshatra::Revati,
        }
    }
}

// Ensure Zodiac reverse mapping
impl From<ZodiacSign> for CoreZodiacSign {
    fn from(sign: ZodiacSign) -> Self {
        match sign {
            ZodiacSign::Aries => CoreZodiacSign::Mesh,
            ZodiacSign::Taurus => CoreZodiacSign::Vrishabh,
            ZodiacSign::Gemini => CoreZodiacSign::Mithun,
            ZodiacSign::Cancer => CoreZodiacSign::Karka,
            ZodiacSign::Leo => CoreZodiacSign::Simha,
            ZodiacSign::Virgo => CoreZodiacSign::Kanya,
            ZodiacSign::Libra => CoreZodiacSign::Tula,
            ZodiacSign::Scorpio => CoreZodiacSign::Vrishchik,
            ZodiacSign::Sagittarius => CoreZodiacSign::Dhanu,
            ZodiacSign::Capricorn => CoreZodiacSign::Makar,
            ZodiacSign::Aquarius => CoreZodiacSign::Kumbha,
            ZodiacSign::Pisces => CoreZodiacSign::Meen,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdRecurrenceRule {
    pub frequency: AdFrequency,
    pub interval: u32,
    pub by_month_day: Option<Vec<u8>>,
    pub by_month: Option<Vec<u8>>,
    pub count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone)]
pub struct TithiRecurrenceRule {
    pub tithis: Vec<u8>,
    pub paksha: Option<Paksha>,
    pub anchor: BsDate,
    pub count: Option<u32>,
    pub until: Option<BsDate>,
    pub by_month: Option<Vec<u8>>,
    pub by_lunar_month: Option<Vec<u8>>,
    pub skip_adhik: bool,
    pub take_first: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paksha {
    Shukla,
    Krishna,
}

impl From<CorePaksha> for Paksha {
    fn from(paksha: CorePaksha) -> Self {
        match paksha {
            CorePaksha::Shukla => Paksha::Shukla,
            CorePaksha::Krishna => Paksha::Krishna,
        }
    }
}

impl From<Paksha> for CorePaksha {
    fn from(paksha: Paksha) -> Self {
        match paksha {
            Paksha::Shukla => CorePaksha::Shukla,
            Paksha::Krishna => CorePaksha::Krishna,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalendarVersion {
    pub version: String,
    pub is_official: bool,
}

impl From<CalendarVersion> for CoreCalendarVersion {
    fn from(v: CalendarVersion) -> Self {
        if v.is_official {
            CoreCalendarVersion::official(v.version)
        } else {
            CoreCalendarVersion::new(v.version, false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventInstance {
    pub id: String,
    pub bs_date: BsDate,
    pub ad_date: GregorianDate,
    pub title: String,
    pub version: CalendarVersion,
    pub parent_event_id: Option<String>,
}

impl From<CoreEventInstance> for EventInstance {
    fn from(inst: CoreEventInstance) -> Self {
        EventInstance {
            id: inst.id,
            bs_date: inst.bs_date.into(),
            ad_date: inst.ad_date.into(),
            title: inst.title,
            version: CalendarVersion {
                version: inst.calendar_version.version.clone(),
                is_official: inst.calendar_version.is_official,
            },
            parent_event_id: inst.parent_event_id,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BsCalendarError {
    #[error("Invalid date")]
    InvalidDate,
    #[error("Date out of supported range")]
    OutOfRange,
    #[error("Date conversion failed")]
    ConversionError,
    #[error("Astronomical calculation failed")]
    AstronomicalError,
    #[error("Invalid recurrence rule")]
    InvalidRecurrenceRule,
}

impl TryFrom<GregorianDate> for NaiveDate {
    type Error = BsCalendarError;

    fn try_from(date: GregorianDate) -> Result<Self, Self::Error> {
        NaiveDate::from_ymd_opt(date.year, date.month, date.day).ok_or(BsCalendarError::InvalidDate)
    }
}

impl From<CoreError> for BsCalendarError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::InvalidYear(_) | CoreError::CalendarDataNotFound(_) => {
                BsCalendarError::OutOfRange
            }
            CoreError::InvalidMonth(_)
            | CoreError::InvalidDay(_, _)
            | CoreError::InvalidDate(_) => BsCalendarError::InvalidDate,
            CoreError::ConversionError(_) | CoreError::InternalError(_) => {
                BsCalendarError::ConversionError
            }
            CoreError::AstronomicalError(_) => BsCalendarError::AstronomicalError,
            CoreError::InvalidRecurrenceRule(_)
            | CoreError::InvalidRRule(_)
            | CoreError::RRuleRejected { .. }
            | CoreError::UnsupportedRRuleFeature(_) => BsCalendarError::InvalidRecurrenceRule,
        }
    }
}

// ============================================================================
// Main Engine Interface
// ============================================================================

pub struct CalendarEngine {
    core: CoreEngine,
}

impl CalendarEngine {
    pub fn create_engine() -> std::sync::Arc<CalendarEngine> {
        std::sync::Arc::new(CalendarEngine {
            core: CoreEngine::new(),
        })
    }

    pub fn bs_to_gregorian(&self, date: BsDate) -> Result<GregorianDate, BsCalendarError> {
        let core_date: CoreBsDate = date.try_into()?;
        let ad_date = self.core.bs_to_gregorian(core_date)?;
        Ok(ad_date.into())
    }

    pub fn gregorian_to_bs(&self, date: GregorianDate) -> Result<BsDate, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let bs_date = self.core.gregorian_to_bs(core_date)?;
        Ok(bs_date.into())
    }

    pub fn get_tithi(&self, date: GregorianDate) -> Result<Tithi, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let tithi = self.core.get_tithi(core_date)?;
        Ok(tithi.into())
    }

    pub fn get_sun_zodiac(&self, date: GregorianDate) -> Result<ZodiacSign, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        // Core returns ZodiacSign directly, so wrap in Ok
        let sign = self.core.get_sun_zodiac(core_date);
        Ok(sign.into())
    }

    pub fn get_moon_zodiac(&self, date: GregorianDate) -> Result<ZodiacSign, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let sign = self.core.get_moon_zodiac(core_date);
        Ok(sign.into())
    }

    pub fn get_nakshatra(&self, date: GregorianDate) -> Result<Nakshatra, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let nakshatra = self.core.get_nakshatra(core_date);
        Ok(nakshatra.into())
    }

    pub fn get_daily_astro_info(
        &self,
        date: GregorianDate,
        location: Location,
    ) -> Result<DailyAstroInfo, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let core_loc: CoreLocation = location.into();
        let info = self.core.get_daily_astro_info(core_date, core_loc)?;
        Ok(info.into())
    }

    pub fn get_sunrise(
        &self,
        date: GregorianDate,
        location: Location,
    ) -> Result<TimeOfDay, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let core_loc: CoreLocation = location.into();
        let time = self.core.get_sunrise(core_date, core_loc)?;
        Ok(time.into())
    }

    pub fn get_sunset(
        &self,
        date: GregorianDate,
        location: Location,
    ) -> Result<TimeOfDay, BsCalendarError> {
        let core_date: NaiveDate = date.try_into()?;
        let core_loc: CoreLocation = location.into();
        let time = self.core.get_sunset(core_date, core_loc)?;
        Ok(time.into())
    }

    // ... (Date conversion and simple getters remain the same)

    pub fn generate_bs_instances(
        &self,
        rule: BsRecurrenceRule,
        start: BsDate,
        end: BsDate,
    ) -> Result<Vec<BsDate>, BsCalendarError> {
        let core_start: CoreBsDate = start.try_into()?;
        let core_end: CoreBsDate = end.try_into()?;

        // Manual conversion to CoreBsRecurrenceRule
        let core_freq = match rule.frequency {
            BsFrequency::Daily => CoreBsFrequency::Daily,
            BsFrequency::Weekly => CoreBsFrequency::Weekly,
            BsFrequency::Monthly => CoreBsFrequency::Monthly,
            BsFrequency::Yearly => CoreBsFrequency::Yearly,
        };

        let by_month = rule.by_month.map(|months| {
            months
                .into_iter()
                .filter_map(|m| crate::domain::bs_date::BsMonth::from_u8(m).ok())
                .collect()
        });

        let core_rule = CoreBsRecurrenceRule {
            frequency: core_freq,
            interval: rule.interval as u16, // Careful with truncation, but u16 is 65535 which is plenty
            anchor: core_start.clone(),     // Use start date as anchor
            by_month: by_month,
            by_month_day: rule.by_month_day,
            by_day: None, // Not exposed in UniFFI yet
            count: rule.count,
            until: None, // We use 'end' param for range generation usually, OR specifically for rule termination?
                         // generate_bs_instances takes explicit 'end' date which caps generation.
        };

        let results = self
            .core
            .generate_bs_instances(&core_rule, core_start, core_end)?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub fn generate_ad_instances(
        &self,
        rule: AdRecurrenceRule,
        start: GregorianDate,
        end: GregorianDate,
    ) -> Result<Vec<GregorianDate>, BsCalendarError> {
        let start_date: NaiveDate = start.clone().try_into()?;
        let end_date: NaiveDate = end.try_into()?;

        // Construct RRULE string
        let freq_str = match rule.frequency {
            AdFrequency::Daily => "DAILY",
            AdFrequency::Weekly => "WEEKLY",
            AdFrequency::Monthly => "MONTHLY",
            AdFrequency::Yearly => "YEARLY",
        };

        let mut rrule_parts = vec![format!("FREQ={}", freq_str)];
        if rule.interval > 1 {
            rrule_parts.push(format!("INTERVAL={}", rule.interval));
        }
        if let Some(count) = rule.count {
            rrule_parts.push(format!("COUNT={}", count));
        }
        if let Some(by_month) = rule.by_month {
            let s: Vec<String> = by_month.iter().map(|n| n.to_string()).collect();
            rrule_parts.push(format!("BYMONTH={}", s.join(",")));
        }
        if let Some(by_month_day) = rule.by_month_day {
            let s: Vec<String> = by_month_day.iter().map(|n| n.to_string()).collect();
            rrule_parts.push(format!("BYMONTHDAY={}", s.join(",")));
        }

        // Note: DTSTART is handled by the engine/rrule crate logic typically, or we prepend it?
        // CoreAdRecurrenceRule takes a raw string.
        // Usually RRULE string doesn't include DTSTART if passed separately, but rrule crate might need it.
        // However, Core's generate_ad_instances takes Start/End dates separately.

        let rrule_string = rrule_parts.join(";");
        let core_rule = CoreAdRecurrenceRule::new(rrule_string)
            .map_err(|_| BsCalendarError::InvalidRecurrenceRule)?;

        let results = self
            .core
            .generate_ad_instances(&core_rule, start_date, end_date)?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub fn generate_tithi_instances(
        &self,
        event_id: String,
        title: String,
        rule: TithiRecurrenceRule,
        start: BsDate,
        end: BsDate,
        version: CalendarVersion,
        location: Location,
    ) -> Result<Vec<EventInstance>, BsCalendarError> {
        let core_start: CoreBsDate = start.try_into()?;
        let core_end: CoreBsDate = end.try_into()?;
        let core_version: CoreCalendarVersion = version.into();
        let core_location: CoreLocation = location.into();

        let core_anchor: CoreBsDate = rule.anchor.try_into()?;

        let tithis: Vec<CoreTithi> = rule
            .tithis
            .iter()
            .filter_map(|&d| CoreTithi::from_paksha_day(CorePaksha::Shukla, d).ok())
            .collect();

        let paksha_filter = rule.paksha.map(CorePaksha::from);

        let until = rule
            .until
            .map(|d| CoreBsDate::try_from(d))
            .transpose()
            .map_err(|_| BsCalendarError::InvalidDate)?;

        let by_month = rule.by_month.map(|ms| {
            ms.iter()
                .filter_map(|&m| crate::domain::bs_date::BsMonth::from_u8(m).ok())
                .collect()
        });

        let by_lunar_month = rule.by_lunar_month.map(|ms| {
            ms.iter()
                .filter_map(|&m| crate::domain::bs_date::BsMonth::from_u8(m).ok())
                .collect()
        });

        let core_rule = CoreTithiRecurrenceRule {
            by_tithi: tithis,
            paksha_filter,
            anchor: core_anchor,
            count: rule.count,
            until,
            by_month,
            by_lunar_month,
            skip_adhik: rule.skip_adhik,
            take_first: rule.take_first,
        };

        let results = self.core.generate_tithi_instances(
            &event_id,
            &title,
            &core_rule,
            core_start,
            core_end,
            core_version,
            core_location,
        )?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    // ... (Methods)

    pub fn get_month_calendar(
        &self,
        year: u16,
        month: u8,
        location: Location,
    ) -> Result<MonthCalendarData, BsCalendarError> {
        let core_location: CoreLocation = location.into();
        let result = self.core.get_month_calendar(year, month, core_location)?;

        Ok(MonthCalendarData {
            days: result.days.into_iter().map(Into::into).collect(),
            year: result.year,
            month: result.month,
            days_in_month: result.days_in_month,
            start_day_of_week: result.start_day_of_week,
        })
    }

    pub fn format_bs_date(&self, date: BsDate, pattern: String, lang: Language) -> Result<String, BsCalendarError> {
        let core_date: CoreBsDate = date.try_into().map_err(|_| BsCalendarError::InvalidDate)?;
        let core_lang: CoreLanguage = lang.into();
        Ok(self.core.format_bs_date(core_date, &pattern, core_lang))
    }

    pub fn get_tithi_name(&self, tithi: Tithi, lang: Language) -> String {
        // Reconstruct CoreTithi from our Tithi
        let core_paksha: CorePaksha = tithi.paksha.into();
        // Use from_paksha_day
        let core_tithi =
            CoreTithi::from_paksha_day(core_paksha, tithi.number).unwrap_or(CoreTithi::Purnima);
        // fallback?
        let core_lang: CoreLanguage = lang.into();
        self.core.get_tithi_name(core_tithi, core_lang)
    }

    pub fn get_zodiac_name(&self, zodiac: ZodiacSign, lang: Language) -> String {
        let core_zodiac: CoreZodiacSign = zodiac.into(); // Need From<ZodiacSign> for CoreZodiacSign
        let core_lang: CoreLanguage = lang.into();
        self.core.get_zodiac_name(core_zodiac, core_lang)
    }

    pub fn get_nakshatra_name(&self, nakshatra: Nakshatra, lang: Language) -> String {
        let core_nakshatra: CoreNakshatra = nakshatra.into(); // Need From<Nakshatra> for CoreNakshatra
        let core_lang: CoreLanguage = lang.into();
        self.core.get_nakshatra_name(core_nakshatra, core_lang)
    }
}

// ============================================================================
// Public API Functions
// ============================================================================

pub fn create_engine() -> std::sync::Arc<CalendarEngine> {
    CalendarEngine::create_engine()
}
