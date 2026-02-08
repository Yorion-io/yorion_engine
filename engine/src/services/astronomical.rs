use crate::domain::tithi::{Location, Paksha, Tithi};
use crate::domain::zodiac::{DailyAstroInfo, Nakshatra, ZodiacSign};
use crate::error::{BsCalendarError, Result};
use astro::time::{julian_day, CalType, Date};
use astro::{lunar, sun};
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Timelike, Utc};
use suncalc::{get_times, Timestamp};

use crate::ports::TithiOverrideProvider;

/// Astronomical service for sun and moon calculations
pub struct AstronomicalService {
    override_provider: Option<Box<dyn TithiOverrideProvider>>,
}

impl AstronomicalService {
    pub fn new() -> Self {
        AstronomicalService {
            override_provider: None,
        }
    }
}

impl Default for AstronomicalService {
    fn default() -> Self {
        Self::new()
    }
}

impl AstronomicalService {
    pub fn with_overrides(provider: Box<dyn TithiOverrideProvider>) -> Self {
        AstronomicalService {
            override_provider: Some(provider),
        }
    }

    /// Get accurate sunrise time for a given date and location
    pub fn get_sunrise(&self, date: NaiveDate, location: Location) -> Result<NaiveTime> {
        let dt = date.and_hms_opt(12, 0, 0).unwrap();
        let timestamp_ms = dt.and_local_timezone(Utc).unwrap().timestamp_millis();

        let times = get_times(
            Timestamp(timestamp_ms),
            location.latitude,
            location.longitude,
            None,
        );

        // Convert timestamp (ms) back to DateTime<Utc>
        let sunrise_secs = times.sunrise.0 / 1000;
        let sunrise_nanos = ((times.sunrise.0 % 1000) * 1_000_000) as u32;

        let sunrise_dt =
            DateTime::<Utc>::from_timestamp(sunrise_secs, sunrise_nanos).ok_or_else(|| {
                BsCalendarError::AstronomicalError("Failed to calculate sunrise".to_string())
            })?;

        // Adjust to local time using the location's offset
        let sunrise_local =
            sunrise_dt + chrono::Duration::minutes(location.timezone_offset_mins as i64);

        Ok(sunrise_local.time())
    }

    /// Get accurate sunset time for a given date and location
    pub fn get_sunset(&self, date: NaiveDate, location: Location) -> Result<NaiveTime> {
        let dt = date.and_hms_opt(12, 0, 0).unwrap();
        let timestamp_ms = dt.and_local_timezone(Utc).unwrap().timestamp_millis();

        let times = get_times(
            Timestamp(timestamp_ms),
            location.latitude,
            location.longitude,
            None,
        );

        // Convert timestamp (ms) back to DateTime<Utc>
        let sunset_secs = times.sunset.0 / 1000;
        let sunset_nanos = ((times.sunset.0 % 1000) * 1_000_000) as u32;

        let sunset_dt =
            DateTime::<Utc>::from_timestamp(sunset_secs, sunset_nanos).ok_or_else(|| {
                BsCalendarError::AstronomicalError("Failed to calculate sunset".to_string())
            })?;

        // Adjust to local time using the location's offset
        let sunset_local =
            sunset_dt + chrono::Duration::minutes(location.timezone_offset_mins as i64);

        Ok(sunset_local.time())
    }

    /// Calculate the Tithi for a given date and time.
    /// Uses the provided timezone offset for override lookup.
    pub fn calculate_tithi(&self, date_time: DateTime<Utc>) -> Result<Tithi> {
        // By default, if no location is provided, we use Nepal time and assume
        // Nepal social calendar for backward compatibility.
        self.calculate_tithi_with_location(date_time, &Location::KATHMANDU)
    }

    /// Calculate the Tithi at a specific moment, using a custom offset for override lookup.
    pub fn calculate_tithi_with_offset(
        &self,
        date_time: DateTime<Utc>,
        offset_mins: i32,
    ) -> Result<Tithi> {
        // Create a temporary location for the override lookup
        let temp_loc = Location::new(0.0, 0.0, "Temp", offset_mins);
        self.calculate_tithi_with_location(date_time, &temp_loc)
    }

    /// Calculate the Tithi at a specific moment for a given location.
    pub fn calculate_tithi_with_location(
        &self,
        date_time: DateTime<Utc>,
        location: &Location,
    ) -> Result<Tithi> {
        self.calculate_tithi_full_with_location(date_time, location)
            .map(|(t, _)| t)
    }

    pub fn calculate_tithi_full_with_location(
        &self,
        date_time: DateTime<Utc>,
        location: &Location,
    ) -> Result<(Tithi, bool)> {
        // Check for overrides first
        if let Some(ref provider) = self.override_provider {
            let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60).unwrap();
            let local_dt = date_time.with_timezone(&offset);
            if let Some(overridden) = provider.get_override(local_dt.date_naive(), location) {
                return Ok((overridden, true));
            }
        }

        let (sun_long, moon_long) = self.calculate_longitudes(date_time);

        let mut diff = moon_long - sun_long;
        while diff < 0.0 {
            diff += 360.0;
        }
        while diff >= 360.0 {
            diff -= 360.0;
        }

        let tithi_index = (diff / 12.0).floor() as u8 + 1;

        if tithi_index <= 15 {
            Tithi::from_paksha_day(Paksha::Shukla, tithi_index).map(|t| (t, false))
        } else {
            Tithi::from_paksha_day(Paksha::Krishna, tithi_index - 15).map(|t| (t, false))
        }
    }

    /// Calculate the primary Tithi for a given calendar date at a specific location.
    /// This is the Tithi active at the moment of local sunrise.
    pub fn calculate_tithi_for_date(&self, date: NaiveDate, location: Location) -> Result<Tithi> {
        let sunrise = self.get_sunrise(date, location)?;
        let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60).unwrap();

        let sunrise_dt = date
            .and_time(sunrise)
            .and_local_timezone(offset)
            .unwrap()
            .with_timezone(&Utc);

        self.calculate_tithi_with_location(sunrise_dt, &location)
    }

    /// Combined calculation of all daily astronomical info
    /// Optimized to reuse intermediate calculations (JD, longitudes)
    pub fn get_daily_astro_info(
        &self,
        date_time: DateTime<Utc>,
        location: &Location,
    ) -> Result<DailyAstroInfo> {
        let (tithi, is_overridden) =
            self.calculate_tithi_full_with_location(date_time, location)?;

        let jd = self.get_julian_day(date_time);
        let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);

        Ok(DailyAstroInfo {
            tithi,
            sun_sign: self.get_sun_zodiac_sign_from_long(sun_long),
            moon_sign: self.get_moon_zodiac_sign_from_long(moon_long),
            nakshatra: self.get_nakshatra_from_long(moon_long),
            is_overridden,
        })
    }

    /// Calculate the primary astronomical info for a given date at sunrise.
    pub fn get_daily_astro_info_for_date(
        &self,
        date: NaiveDate,
        location: Location,
    ) -> Result<DailyAstroInfo> {
        let sunrise = self.get_sunrise(date, location)?;
        let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60).unwrap();

        let sunrise_dt = date
            .and_time(sunrise)
            .and_local_timezone(offset)
            .unwrap()
            .with_timezone(&Utc);

        self.get_daily_astro_info(sunrise_dt, &location)
    }

    /// High-precision calculation of solar and lunar ecliptic longitudes
    /// Uses VSOP87 for Sun and ELP-2000/82 for Moon via the 'astro' crate.
    fn calculate_longitudes(&self, date_time: DateTime<Utc>) -> (f64, f64) {
        let jd = self.get_julian_day(date_time);

        // High-precision Sun position (VSOP87)
        let (sun_pos, _) = sun::geocent_ecl_pos(jd);
        let sun_long = sun_pos.long.to_degrees() % 360.0;

        // High-precision Moon position (ELP-2000/82)
        let (moon_pos, _) = lunar::geocent_ecl_pos(jd);
        let moon_long = moon_pos.long.to_degrees() % 360.0;

        (sun_long, moon_long)
    }

    /// Find the JD of the next Amavasya (new moon) starting from jd
    pub fn find_next_amavasya(&self, mut jd: f64) -> Result<f64> {
        let mut iterations = 0;
        loop {
            let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
            let mut diff = moon_long - sun_long;
            while diff < 0.0 {
                diff += 360.0;
            }
            while diff >= 360.0 {
                diff -= 360.0;
            }

            if !(0.001..=359.999).contains(&diff) {
                return Ok(jd);
            }

            let days_to_go = (360.0 - diff) / 12.19;
            jd += days_to_go.clamp(0.0001, 1.0);

            iterations += 1;
            if iterations > 2000 {
                return Err(BsCalendarError::AstronomicalError(
                    "Amavasya (next) search timed out".to_string(),
                ));
            }
        }
    }

    /// Find the JD of the previous Amavasya (new moon) starting from jd
    pub fn find_prev_amavasya(&self, mut jd: f64) -> Result<f64> {
        let mut iterations = 0;
        loop {
            let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
            let mut diff = moon_long - sun_long;
            while diff < 0.0 {
                diff += 360.0;
            }
            while diff >= 360.0 {
                diff -= 360.0;
            }

            if !(0.001..=359.999).contains(&diff) {
                return Ok(jd);
            }

            // Go backwards (moon moves ~12.19 deg/day relative to sun)
            let days_back = diff / 12.19;
            jd -= days_back.clamp(0.0001, 1.0);

            iterations += 1;
            if iterations > 2000 {
                return Err(BsCalendarError::AstronomicalError(
                    "Amavasya (prev) search timed out".to_string(),
                ));
            }
        }
    }

    /// Check if the lunar month containing the given JD is an Adhik (Extra) month.
    /// A lunar month is Adhik if no solar Sankranti occurs within it.
    pub fn is_adhik_month(&self, jd: f64) -> Result<bool> {
        let prev_amavasya = self.find_prev_amavasya(jd)?;
        let next_amavasya = self.find_next_amavasya(jd + 0.1)?;

        // Use a small offset to check the sign the sun was in
        // leading up to each Amavasya.
        let sign_at_start = self.get_sun_zodiac_sign(prev_amavasya - 0.01);
        let sign_at_end = self.get_sun_zodiac_sign(next_amavasya - 0.01);

        // A month is Adhik if the sun is in the same sign at both
        // the start and end Amavasyas (i.e., no Sankranti occurred).
        Ok(sign_at_start == sign_at_end)
    }

    /// Get the zodiac sign (1-12) the sun is currently in (Nirayana/Sidereal)
    /// 1 = Mesh (Aries), 12 = Meen (Pisces)
    pub fn get_sun_zodiac_sign(&self, jd: f64) -> ZodiacSign {
        let (sun_long, _) = self.calculate_longitudes_from_jd(jd);
        self.get_sun_zodiac_sign_from_long(sun_long)
    }

    /// Helper: Get Sun Sign from longitude (avoids recalculation)
    pub fn get_sun_zodiac_sign_from_long(&self, sun_long: f64) -> ZodiacSign {
        let sidereal_long = self.get_sidereal_longitude(sun_long);
        let index = ((sidereal_long / 30.0).floor() as u8 % 12) + 1;
        ZodiacSign::from_index(index).unwrap()
    }

    /// Get the zodiac sign the moon is currently in (Nirayana/Sidereal)
    pub fn get_moon_zodiac_sign(&self, jd: f64) -> ZodiacSign {
        let (_, moon_long) = self.calculate_longitudes_from_jd(jd);
        self.get_moon_zodiac_sign_from_long(moon_long)
    }

    /// Helper: Get Moon Sign from longitude (avoids recalculation)
    pub fn get_moon_zodiac_sign_from_long(&self, moon_long: f64) -> ZodiacSign {
        let sidereal_long = self.get_sidereal_longitude(moon_long);
        let index = ((sidereal_long / 30.0).floor() as u8 % 12) + 1;
        ZodiacSign::from_index(index).unwrap()
    }

    /// Get the Nakshatra for a given JD based on Moon's longitude
    pub fn get_nakshatra(&self, jd: f64) -> Nakshatra {
        let (_, moon_long) = self.calculate_longitudes_from_jd(jd);
        self.get_nakshatra_from_long(moon_long)
    }

    /// Helper: Get Nakshatra from longitude (avoids recalculation)
    pub fn get_nakshatra_from_long(&self, moon_long: f64) -> Nakshatra {
        let sidereal_long = self.get_sidereal_longitude(moon_long);
        // There are 27 Nakshatras, each 13° 20' (13.333... degrees)
        let index = ((sidereal_long / (360.0 / 27.0)).floor() as u8 % 27) + 1;
        Nakshatra::from_index(index).unwrap()
    }

    /// Helper to convert tropical longitude to sidereal using Ayanamsa
    fn get_sidereal_longitude(&self, tropical_long: f64) -> f64 {
        // Simplified Ayanamsa. In many Hindu calendars, Lahiri Ayanamsa is used.
        // For approx calculations, 24.0 is commonly used for recent years.
        let ayanamsa = 24.0;
        (tropical_long - ayanamsa + 360.0) % 360.0
    }

    /// Internal helper for longitude calculation from JD
    pub fn calculate_longitudes_from_jd(&self, jd: f64) -> (f64, f64) {
        // High-precision Sun position (VSOP87)
        let (sun_pos, _) = sun::geocent_ecl_pos(jd);
        let sun_long = sun_pos.long.to_degrees() % 360.0;

        // High-precision Moon position (ELP-2000/82)
        let (moon_pos, _) = lunar::geocent_ecl_pos(jd);
        let moon_long = moon_pos.long.to_degrees() % 360.0;

        (sun_long, moon_long)
    }

    pub fn get_julian_day(&self, dt: DateTime<Utc>) -> f64 {
        let decimal_day = dt.day() as f64
            + (dt.hour() as f64 / 24.0)
            + (dt.minute() as f64 / 1440.0)
            + (dt.second() as f64 / 86400.0);

        let date = Date {
            year: dt.year() as i16,
            month: dt.month() as u8,
            decimal_day,
            cal_type: CalType::Gregorian,
        };

        julian_day(&date)
    }
}
