//! Core API - Stable, canonical interface for the BS Calendar engine
//!
//! This module provides the stable public API that all platform bindings
//! (WASM, Flutter, etc.) should use. This ensures consistency and makes
//! testing easier.
//!
//! # Stability Guarantee
//! Functions in this module are **Tier 1 Stable** and follow semantic versioning.
//! Breaking changes will only occur in major version bumps.

use crate::domain::tithi::Location;
use crate::domain::zodiac::DailyAstroInfo;
use crate::domain::Language;
use crate::prelude::*;
use chrono::NaiveDate;
use std::sync::Arc;

/// Last BS year for which tithi output has been validated against the
/// official Nepali Panchanga almanac (and for which override corrections
/// exist). Beyond this year tithi values are raw astronomical computation
/// with no almanac verification — treat them as provisional.
pub const TITHI_VERIFIED_THROUGH_BS: u16 = 2083;

/// Core calendar engine with stable API
pub struct CalendarEngine {
    pub(crate) conversion_service: Arc<ConversionService>,
    pub(crate) astronomical_service: Arc<AstronomicalService>,
    pub(crate) instance_generator: InstanceGenerator,
    pub(crate) tithi_instance_generator: TithiInstanceGenerator,
}

impl CalendarEngine {
    pub fn new() -> Self {
        let provider = Arc::new(StaticCalendarProvider::new());
        let override_provider = Box::new(StaticTithiOverrideProvider::new());
        let astronomical_service = Arc::new(AstronomicalService::with_overrides(override_provider));
        let conversion_service = Arc::new(ConversionService::new(provider));

        CalendarEngine {
            instance_generator: InstanceGenerator::new(conversion_service.clone()),
            tithi_instance_generator: TithiInstanceGenerator::new(
                conversion_service.clone(),
                astronomical_service.clone(),
            ),
            conversion_service,
            astronomical_service,
        }
    }

    /// Convert Bikram Sambat date to Gregorian date
    ///
    /// # Stability
    /// This function is **Tier 1 Stable** and will never have breaking changes.
    ///
    /// # Errors
    /// Returns `BsCalendarError::InvalidDate` if the BS date is out of range (1975-2100).
    ///
    /// # Examples
    /// ```
    /// use yorion_engine::core_api::CalendarEngine;
    /// use yorion_engine::domain::BsDate;
    /// use chrono::Datelike;
    ///
    /// let engine = CalendarEngine::new();
    /// let bs = BsDate::new(2080, 1, 1).unwrap();
    /// let ad = engine.bs_to_gregorian(bs).unwrap();
    /// assert_eq!(ad.year(), 2023);
    /// ```
    pub fn bs_to_gregorian(&self, date: BsDate) -> Result<NaiveDate> {
        self.conversion_service.bs_to_gregorian(date)
    }

    /// Convert Gregorian date to Bikram Sambat date
    ///
    /// # Stability
    /// This function is **Tier 1 Stable** and will never have breaking changes.
    ///
    /// # Errors
    /// Returns `BsCalendarError::InvalidDate` if the Gregorian date is out of range.
    pub fn gregorian_to_bs(&self, date: NaiveDate) -> Result<BsDate> {
        self.conversion_service.gregorian_to_bs(date)
    }

    /// Create a BS date validated against the embedded calendar table.
    ///
    /// Unlike [`BsDate::new`] (structural checks only), this rejects a day
    /// that does not exist in that specific BS month (e.g. day 32 in a
    /// 30-day month) and a year outside the supported data range.
    ///
    /// # Errors
    /// Returns `BsCalendarError::CalendarDataNotFound` for an out-of-range
    /// year and `BsCalendarError::InvalidDay` for a non-existent day.
    pub fn checked_bs_date(&self, year: u16, month: u8, day: u8) -> Result<BsDate> {
        let date = BsDate::new(year, month, day)?;
        let actual_days = self.calendar().get_month_days(date.year, date.month)?;
        if date.day > actual_days {
            return Err(crate::error::BsCalendarError::InvalidDay(
                date.day,
                date.month.to_u8(),
            ));
        }
        Ok(date)
    }

    /// Get the Tithi for a specific Gregorian date at **Kathmandu**.
    ///
    /// Returns the tithi active at sunrise (the Hindu calendar convention),
    /// computed for the default Kathmandu location with Nepal social-calendar
    /// overrides applied. For any other location use
    /// [`CalendarEngine::get_tithi_at_location`].
    ///
    /// # Errors
    /// Returns error if astronomical calculation fails.
    pub fn get_tithi(&self, date: NaiveDate) -> Result<Tithi> {
        self.get_tithi_at_location(date, &Location::kathmandu())
    }

    /// Get the Tithi for a specific Gregorian date at a given location.
    ///
    /// The tithi is the one active at the moment of **local sunrise** at the
    /// supplied location.
    pub fn get_tithi_at_location(&self, date: NaiveDate, location: &Location) -> Result<Tithi> {
        self.astronomical_service
            .calculate_tithi_for_date(date, location)
    }

    /// Get the Sun's zodiac sign for a specific Gregorian date
    pub fn get_sun_zodiac(&self, date: NaiveDate) -> ZodiacSign {
        let jd = self
            .astronomical_service
            .get_julian_day(date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time").and_utc());
        self.astronomical_service.get_sun_zodiac_sign(jd)
    }

    /// Get the Moon's zodiac sign (Rashi) for a specific Gregorian date
    pub fn get_moon_zodiac(&self, date: NaiveDate) -> ZodiacSign {
        let jd = self
            .astronomical_service
            .get_julian_day(date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time").and_utc());
        self.astronomical_service.get_moon_zodiac_sign(jd)
    }

    /// Get the Nakshatra for a specific Gregorian date
    pub fn get_nakshatra(&self, date: NaiveDate) -> Nakshatra {
        let jd = self
            .astronomical_service
            .get_julian_day(date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time").and_utc());
        self.astronomical_service.get_nakshatra(jd)
    }

    /// Get the Yoga for a specific Gregorian date (computed at noon UTC)
    pub fn get_yoga(&self, date: NaiveDate) -> Yoga {
        let jd = self
            .astronomical_service
            .get_julian_day(date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time").and_utc());
        self.astronomical_service.get_yoga(jd)
    }

    /// Get the Karana for a specific Gregorian date (computed at noon UTC)
    pub fn get_karana(&self, date: NaiveDate) -> Karana {
        let jd = self
            .astronomical_service
            .get_julian_day(date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time").and_utc());
        self.astronomical_service.get_karana(jd)
    }

    /// Find when the tithi active at local sunrise of `date` ends.
    ///
    /// Published panchangas list tithi end times alongside the tithi name;
    /// this returns that boundary as a UTC instant. The start of the current
    /// tithi is the end of the previous one.
    ///
    /// # Errors
    /// Returns error if the sunrise or transition search fails.
    pub fn get_tithi_end(
        &self,
        date: NaiveDate,
        location: &Location,
    ) -> Result<chrono::DateTime<chrono::Utc>> {
        let sunrise_dt = self.local_sunrise_utc(date, location)?;
        self.astronomical_service.find_tithi_end(sunrise_dt)
    }

    /// Get comprehensive astronomical info for a specific Gregorian date
    ///
    /// Returns info calculated at sunrise (the Hindu calendar convention).
    ///
    /// # Errors
    /// Returns error if astronomical calculation fails.
    pub fn get_daily_astro_info(
        &self,
        date: NaiveDate,
        location: Location,
    ) -> Result<DailyAstroInfo> {
        self.astronomical_service
            .get_daily_astro_info_for_date(date, &location)
    }

    /// Get daily astronomical info plus sunrise/sunset in one pass.
    ///
    /// Computes sunrise once and reuses it for the sunrise-anchored panchanga
    /// calculation, instead of the two sunrise computations a caller would
    /// otherwise pay calling `get_daily_astro_info` and `get_sunrise`
    /// separately.
    ///
    /// # Errors
    /// Returns error if astronomical calculation fails.
    pub fn get_daily_panchanga(
        &self,
        date: NaiveDate,
        location: &Location,
    ) -> Result<DailyPanchanga> {
        let sunrise = self.astronomical_service.get_sunrise(date, location)?;
        let sunset = self.astronomical_service.get_sunset(date, location)?;

        let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60)
            .expect("timezone_offset_mins is within ±24h");
        let sunrise_dt = date
            .and_time(sunrise)
            .and_local_timezone(offset)
            .single()
            .expect("local sunrise time is unambiguous")
            .with_timezone(&chrono::Utc);

        let info = self
            .astronomical_service
            .get_daily_astro_info(sunrise_dt, location)?;

        Ok(DailyPanchanga {
            info,
            sunrise,
            sunset,
        })
    }

    /// UTC instant of local sunrise at the given date/location.
    fn local_sunrise_utc(
        &self,
        date: NaiveDate,
        location: &Location,
    ) -> Result<chrono::DateTime<chrono::Utc>> {
        let sunrise = self.astronomical_service.get_sunrise(date, location)?;
        let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60)
            .expect("timezone_offset_mins is within ±24h");
        Ok(date
            .and_time(sunrise)
            .and_local_timezone(offset)
            .single()
            .expect("local sunrise time is unambiguous")
            .with_timezone(&chrono::Utc))
    }

    /// Get sunrise time for a specific Gregorian date
    ///
    /// # Errors
    /// Returns error if sunrise calculation fails.
    pub fn get_sunrise(&self, date: NaiveDate, location: Location) -> Result<chrono::NaiveTime> {
        self.astronomical_service.get_sunrise(date, &location)
    }

    /// Get sunset time for a specific Gregorian date
    ///
    /// # Errors
    /// Returns error if sunset calculation fails.
    pub fn get_sunset(&self, date: NaiveDate, location: Location) -> Result<chrono::NaiveTime> {
        self.astronomical_service.get_sunset(date, &location)
    }

    /// Generate BS recurring instances within a date range
    ///
    /// # Errors
    /// Returns error if instance generation fails.
    pub fn generate_bs_instances(
        &self,
        rule: &BsRecurrenceRule,
        start: BsDate,
        end: BsDate,
    ) -> Result<Vec<BsDate>> {
        self.instance_generator
            .generate_bs_instances(rule, start, end)
    }

    /// Generate Tithi recurring instances within a date range
    ///
    /// # Errors
    /// Returns error if instance generation fails.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_tithi_instances(
        &self,
        event_id: &str,
        title: &str,
        rule: &TithiRecurrenceRule,
        start: BsDate,
        end: BsDate,
        version: CalendarVersion,
        location: Location,
    ) -> Result<Vec<EventInstance>> {
        self.tithi_instance_generator
            .generate_instances(event_id, title, rule, start, end, version, location)
    }

    /// Format a BS date according to the specified language and pattern
    pub fn format_bs_date(&self, date: BsDate, pattern: &str, lang: Language) -> String {
        crate::utils::number_utils::DateFormatter::format(&date, pattern, lang)
    }

    /// Get Tithi name in specified language
    pub fn get_tithi_name(&self, tithi: Tithi, lang: Language) -> String {
        tithi.name_with_language(lang).to_string()
    }

    /// Get Zodiac sign name in specified language
    pub fn get_zodiac_name(&self, zodiac: ZodiacSign, lang: Language) -> String {
        zodiac.name_with_language(lang).to_string()
    }

    /// Get Nakshatra name in specified language
    pub fn get_nakshatra_name(&self, nakshatra: Nakshatra, lang: Language) -> String {
        nakshatra.name_with_language(lang).to_string()
    }

    /// Get Yoga name in specified language
    pub fn get_yoga_name(&self, yoga: Yoga, lang: Language) -> String {
        yoga.name_with_language(lang).to_string()
    }

    /// Get Karana name in specified language
    pub fn get_karana_name(&self, karana: Karana, lang: Language) -> String {
        karana.name_with_language(lang).to_string()
    }

    /// Generate AD (Gregorian) recurring instances within a date range
    ///
    /// # Errors
    /// Returns error if instance generation fails.
    pub fn generate_ad_instances(
        &self,
        rule: &crate::domain::recurrence::AdRecurrenceRule,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<NaiveDate>> {
        self.instance_generator
            .generate_ad_instances(rule, start, end)
    }

    /// Get full month calendar with astronomical data for each day
    ///
    /// Returns a vector of calendar days with BS date, Gregorian date,
    /// day of week, and astronomical information (Tithi, zodiac signs, Nakshatra).
    ///
    /// # Errors
    /// Returns error if date conversion or astronomical calculation fails.
    pub fn get_month_calendar(
        &self,
        year: u16,
        month: u8,
        location: Location,
    ) -> Result<MonthCalendarData> {
        let bs_month = BsMonth::from_u8(month)?;
        let days_in_month = self.calendar().get_month_days(year, bs_month)?;

        let mut days = Vec::new();
        let mut start_day_of_week = 0;

        for day in 1..=days_in_month {
            let bs_date = BsDate::new(year, month, day)?;
            let gregorian = self.bs_to_gregorian(bs_date)?;

            if day == 1 {
                use chrono::Datelike;
                start_day_of_week = gregorian.weekday().num_days_from_sunday() as u8;
            }

            let info = self
                .astronomical_service
                .get_daily_astro_info_for_date(gregorian, &location)?;

            days.push(CalendarDay {
                bs_year: year,
                bs_month: month,
                bs_day: day,
                gregorian_date: gregorian,
                day_of_week: {
                    use chrono::Datelike;
                    gregorian.weekday().num_days_from_sunday() as u8
                },
                tithi: info.tithi,
                sun_sign: info.sun_sign,
                moon_sign: info.moon_sign,
                nakshatra: info.nakshatra,
                yoga: info.yoga,
                karana: info.karana,
                is_overridden: info.is_overridden,
            });
        }

        Ok(MonthCalendarData {
            days,
            year,
            month,
            days_in_month,
            start_day_of_week,
        })
    }

    /// Generate event instances for a list of events within a date range
    ///
    /// # Errors
    /// Returns error if event generation fails.
    pub fn generate_event_instances(
        &self,
        events: Vec<crate::domain::event::Event>,
        start: BsDate,
        end: BsDate,
        location: Location,
    ) -> Result<Vec<EventInstance>> {
        let start_ad = self.bs_to_gregorian(start)?;
        let end_ad = self.bs_to_gregorian(end)?;

        let mut instances = Vec::new();
        let version = CalendarVersion::official("v1".to_string());

        for event in events {
            match event.recurrence {
                crate::domain::recurrence::Recurrence::Ad(rule) => {
                    let ads = self.generate_ad_instances(&rule, start_ad, end_ad)?;
                    for ad in ads {
                        let bs = self.gregorian_to_bs(ad)?;
                        instances.push(EventInstance::from_recurrence(
                            format!("{}-{}", event.id, bs.format()),
                            bs,
                            ad,
                            event.title.clone(),
                            version.clone(),
                            event.id.clone(),
                        ));
                    }
                }
                crate::domain::recurrence::Recurrence::Bs(rule) => {
                    // Use the clamp-aware variant so calendar-intrinsic day-clamping
                    // (A1: a non-existent target day forced onto the last valid day)
                    // is surfaced on the instance instead of silently absorbed.
                    let bss = self
                        .instance_generator
                        .generate_bs_instances_with_clamp(&rule, start, end)?;
                    for (bs, intended) in bss {
                        let ad = self.bs_to_gregorian(bs)?;
                        let instance = EventInstance::from_recurrence(
                            format!("{}-{}", event.id, bs.format()),
                            bs,
                            ad,
                            event.title.clone(),
                            version.clone(),
                            event.id.clone(),
                        );
                        // When the calendar clamped the rule's target day, flag the
                        // instance with the intended (un-clamped) BS date.
                        let instance = match intended {
                            Some(orig) => instance.as_exception(orig),
                            None => instance,
                        };
                        instances.push(instance);
                    }
                }
                crate::domain::recurrence::Recurrence::Tithi(rule) => {
                    let tithi_instances = self.generate_tithi_instances(
                        &event.id,
                        &event.title,
                        &rule,
                        start,
                        end,
                        version.clone(),
                        location.clone(),
                    )?;
                    instances.extend(tithi_instances);
                }
            }
        }

        Ok(instances)
    }

    /// Get the calendar provider (for advanced usage)
    pub fn calendar(&self) -> &Arc<dyn CalendarProvider> {
        self.conversion_service.calendar()
    }
}

/// Calendar day with astronomical data
#[derive(Debug, Clone)]
pub struct CalendarDay {
    pub bs_year: u16,
    pub bs_month: u8,
    pub bs_day: u8,
    pub gregorian_date: NaiveDate,
    pub day_of_week: u8,
    pub tithi: Tithi,
    pub sun_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
    pub nakshatra: Nakshatra,
    pub yoga: Yoga,
    pub karana: Karana,
    pub is_overridden: bool,
}

/// Daily astronomical info bundled with sunrise/sunset times.
///
/// Produced by [`CalendarEngine::get_daily_panchanga`], which computes
/// sunrise once for both the times and the sunrise-anchored panchanga.
#[derive(Debug, Clone)]
pub struct DailyPanchanga {
    pub info: DailyAstroInfo,
    pub sunrise: chrono::NaiveTime,
    pub sunset: chrono::NaiveTime,
}

/// Full month calendar data
#[derive(Debug, Clone)]
pub struct MonthCalendarData {
    pub days: Vec<CalendarDay>,
    pub year: u16,
    pub month: u8,
    pub days_in_month: u8,
    pub start_day_of_week: u8,
}

impl Default for CalendarEngine {
    fn default() -> Self {
        Self::new()
    }
}
