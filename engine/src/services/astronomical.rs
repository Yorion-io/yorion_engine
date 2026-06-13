use crate::domain::tithi::{Location, Paksha, Tithi};
use crate::domain::zodiac::{DailyAstroInfo, Karana, Nakshatra, Yoga, ZodiacSign};
use crate::error::{BsCalendarError, Result};
use astro::time::{julian_day, CalType, Date};
use astro::{lunar, sun};
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Timelike, Utc};
use suncalc::{get_times, Timestamp};

use crate::ports::TithiOverrideProvider;

/// Julian Day of the J2000.0 epoch (2000-01-01 12:00 TT).
const J2000_JD: f64 = 2_451_545.0;

/// Lahiri (Chitrapaksha) ayanamsa at the J2000.0 epoch, in degrees
/// (23°51′11.5″). Source: Indian Astronomical Ephemeris.
const LAHIRI_AYANAMSA_J2000: f64 = 23.85319;

/// Mean accumulation rate of the ayanamsa (general precession),
/// ~50.29 arcseconds per Julian year, in degrees per day.
const AYANAMSA_RATE_DEG_PER_DAY: f64 = 50.2888 / 3600.0 / 365.25;

/// Mean moon-sun relative angular velocity in degrees per day, used to seed
/// iterative searches (new-moon finder, tithi transition finder).
const MEAN_ELONGATION_RATE: f64 = 12.19;

/// Normalize an angle in degrees to the range [0, 360).
fn normalize_degrees(deg: f64) -> f64 {
    deg.rem_euclid(360.0)
}

/// Convert a Julian Day to a UTC datetime (exact, via the Unix epoch).
pub fn jd_to_datetime(jd: f64) -> Result<DateTime<Utc>> {
    // Unix epoch 1970-01-01T00:00:00Z is JD 2440587.5.
    let unix_secs = (jd - 2_440_587.5) * 86_400.0;
    DateTime::<Utc>::from_timestamp_millis((unix_secs * 1000.0).round() as i64).ok_or_else(|| {
        BsCalendarError::AstronomicalError(format!("Julian Day {jd} is out of representable range"))
    })
}

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
    pub fn get_sunrise(&self, date: NaiveDate, location: &Location) -> Result<NaiveTime> {
        let dt = date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time");
        let timestamp_ms = dt.and_local_timezone(Utc).single().expect("UTC has no ambiguous times").timestamp_millis();

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
    pub fn get_sunset(&self, date: NaiveDate, location: &Location) -> Result<NaiveTime> {
        let dt = date.and_hms_opt(12, 0, 0).expect("12:00:00 is always a valid time");
        let timestamp_ms = dt.and_local_timezone(Utc).single().expect("UTC has no ambiguous times").timestamp_millis();

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
            let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60)
                .expect("timezone_offset_mins is within ±24h");
            let local_dt = date_time.with_timezone(&offset);
            if let Some(overridden) = provider.get_override(local_dt.date_naive(), location) {
                return Ok((overridden, true));
            }
        }

        let jd = self.get_julian_day(date_time);
        let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
        let diff = normalize_degrees(moon_long - sun_long);

        let tithi_index = (diff / 12.0).floor() as u8 + 1;

        if tithi_index <= 15 {
            Tithi::from_paksha_day(Paksha::Shukla, tithi_index).map(|t| (t, false))
        } else {
            Tithi::from_paksha_day(Paksha::Krishna, tithi_index - 15).map(|t| (t, false))
        }
    }

    /// Calculate the primary Tithi for a given calendar date at a specific location.
    /// This is the Tithi active at the moment of local sunrise.
    pub fn calculate_tithi_for_date(&self, date: NaiveDate, location: &Location) -> Result<Tithi> {
        let sunrise = self.get_sunrise(date, location)?;
        let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60)
            .expect("timezone_offset_mins is within ±24h");

        let sunrise_dt = date
            .and_time(sunrise)
            .and_local_timezone(offset)
            .single()
            .expect("local sunrise time is unambiguous")
            .with_timezone(&Utc);

        self.calculate_tithi_with_location(sunrise_dt, location)
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
        let elongation = normalize_degrees(moon_long - sun_long);

        Ok(DailyAstroInfo {
            tithi,
            sun_sign: self.get_sun_zodiac_sign_from_long(sun_long, jd),
            moon_sign: self.get_moon_zodiac_sign_from_long(moon_long, jd),
            nakshatra: self.get_nakshatra_from_long(moon_long, jd),
            yoga: self.get_yoga_from_longs(sun_long, moon_long, jd),
            karana: self.get_karana_from_elongation(elongation),
            is_overridden,
        })
    }

    /// Calculate the primary astronomical info for a given date at sunrise.
    pub fn get_daily_astro_info_for_date(
        &self,
        date: NaiveDate,
        location: &Location,
    ) -> Result<DailyAstroInfo> {
        let sunrise = self.get_sunrise(date, location)?;
        let offset = chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60)
            .expect("timezone_offset_mins is within ±24h");

        let sunrise_dt = date
            .and_time(sunrise)
            .and_local_timezone(offset)
            .single()
            .expect("local sunrise time is unambiguous")
            .with_timezone(&Utc);

        self.get_daily_astro_info(sunrise_dt, location)
    }

    /// Find the JD of the next Amavasya (new moon) starting from jd
    pub fn find_next_amavasya(&self, mut jd: f64) -> Result<f64> {
        let mut iterations = 0;
        loop {
            let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
            let diff = normalize_degrees(moon_long - sun_long);

            if !(0.001..=359.999).contains(&diff) {
                return Ok(jd);
            }

            let days_to_go = (360.0 - diff) / MEAN_ELONGATION_RATE;
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
            let diff = normalize_degrees(moon_long - sun_long);

            if !(0.001..=359.999).contains(&diff) {
                return Ok(jd);
            }

            // Go backwards (moon moves ~12.19 deg/day relative to sun)
            let days_back = diff / MEAN_ELONGATION_RATE;
            jd -= days_back.clamp(0.0001, 1.0);

            iterations += 1;
            if iterations > 2000 {
                return Err(BsCalendarError::AstronomicalError(
                    "Amavasya (prev) search timed out".to_string(),
                ));
            }
        }
    }

    /// Find the UTC instant at which the tithi active at `from` ends — i.e.
    /// when the moon-sun elongation next crosses a multiple of 12°.
    ///
    /// Published panchangas list tithi start/end times; this exposes the end
    /// boundary of the current tithi. The start of the current tithi is the
    /// end of the previous one.
    pub fn find_tithi_end(&self, from: DateTime<Utc>) -> Result<DateTime<Utc>> {
        let mut jd = self.get_julian_day(from);
        let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
        let start_diff = normalize_degrees(moon_long - sun_long);
        // Next 12° boundary strictly ahead of the current elongation.
        let boundary = ((start_diff / 12.0).floor() + 1.0) * 12.0;

        let mut iterations = 0;
        loop {
            let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
            let diff = normalize_degrees(moon_long - sun_long);
            // Degrees still to travel to reach the boundary (mod 360 handles
            // the boundary at 360° → 0° wrap).
            let remaining = normalize_degrees(boundary - diff);

            // Converged on the boundary, or just crossed it (remaining wraps
            // toward 360 after a crossing). The crossing case bounds the
            // error by the last step, which shrinks as the boundary nears.
            if remaining < 0.001 || remaining > 180.0 {
                return jd_to_datetime(jd);
            }

            // Step with the *fastest* lunar elongation rate (~13.0°/day at
            // perigee, padded to 14) so a step can never overshoot the
            // boundary by more than the convergence tolerance.
            let days_to_go = remaining / 14.0;
            jd += days_to_go.clamp(0.0001, 0.5);

            iterations += 1;
            if iterations > 2000 {
                return Err(BsCalendarError::AstronomicalError(
                    "Tithi transition search timed out".to_string(),
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
        self.get_sun_zodiac_sign_from_long(sun_long, jd)
    }

    /// Helper: Get Sun Sign from a tropical longitude already computed for `jd`
    /// (avoids recalculating the ephemeris; `jd` is still needed for the ayanamsa).
    pub fn get_sun_zodiac_sign_from_long(&self, sun_long: f64, jd: f64) -> ZodiacSign {
        let sidereal_long = self.get_sidereal_longitude(sun_long, jd);
        let index = ((sidereal_long / 30.0).floor() as u8 % 12) + 1;
        ZodiacSign::from_index(index)
            .unwrap_or_else(|| unreachable!("index is always 1..=12 by modular arithmetic"))
    }

    /// Get the zodiac sign the moon is currently in (Nirayana/Sidereal)
    pub fn get_moon_zodiac_sign(&self, jd: f64) -> ZodiacSign {
        let (_, moon_long) = self.calculate_longitudes_from_jd(jd);
        self.get_moon_zodiac_sign_from_long(moon_long, jd)
    }

    /// Helper: Get Moon Sign from a tropical longitude already computed for `jd`.
    pub fn get_moon_zodiac_sign_from_long(&self, moon_long: f64, jd: f64) -> ZodiacSign {
        let sidereal_long = self.get_sidereal_longitude(moon_long, jd);
        let index = ((sidereal_long / 30.0).floor() as u8 % 12) + 1;
        ZodiacSign::from_index(index)
            .unwrap_or_else(|| unreachable!("index is always 1..=12 by modular arithmetic"))
    }

    /// Get the Nakshatra for a given JD based on Moon's longitude
    pub fn get_nakshatra(&self, jd: f64) -> Nakshatra {
        let (_, moon_long) = self.calculate_longitudes_from_jd(jd);
        self.get_nakshatra_from_long(moon_long, jd)
    }

    /// Helper: Get Nakshatra from a tropical longitude already computed for `jd`.
    pub fn get_nakshatra_from_long(&self, moon_long: f64, jd: f64) -> Nakshatra {
        let sidereal_long = self.get_sidereal_longitude(moon_long, jd);
        // 27 Nakshatras, each 13°20' (360/27 degrees)
        let index = ((sidereal_long / (360.0 / 27.0)).floor() as u8 % 27) + 1;
        Nakshatra::from_index(index)
            .unwrap_or_else(|| unreachable!("index is always 1..=27 by modular arithmetic"))
    }

    /// Get the Yoga for a given JD.
    pub fn get_yoga(&self, jd: f64) -> Yoga {
        let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
        self.get_yoga_from_longs(sun_long, moon_long, jd)
    }

    /// Helper: Get Yoga from tropical longitudes already computed for `jd`.
    ///
    /// Yoga is the sum of the sidereal longitudes of sun and moon, divided
    /// into 27 segments of 13°20′ each.
    pub fn get_yoga_from_longs(&self, sun_long: f64, moon_long: f64, jd: f64) -> Yoga {
        let sum = normalize_degrees(
            self.get_sidereal_longitude(sun_long, jd) + self.get_sidereal_longitude(moon_long, jd),
        );
        let index = ((sum / (360.0 / 27.0)).floor() as u8 % 27) + 1;
        Yoga::from_index(index)
            .unwrap_or_else(|| unreachable!("index is always 1..=27 by modular arithmetic"))
    }

    /// Get the Karana for a given JD.
    pub fn get_karana(&self, jd: f64) -> Karana {
        let (sun_long, moon_long) = self.calculate_longitudes_from_jd(jd);
        self.get_karana_from_elongation(normalize_degrees(moon_long - sun_long))
    }

    /// Helper: Get Karana from a moon-sun elongation already normalized to [0, 360).
    ///
    /// A karana is half a tithi: half-tithi index = floor(elongation / 6°),
    /// mapped onto the 11 karanas (see [`Karana::from_half_tithi_index`]).
    pub fn get_karana_from_elongation(&self, elongation: f64) -> Karana {
        let k = (elongation / 6.0).floor() as u8 % 60;
        Karana::from_half_tithi_index(k)
            .unwrap_or_else(|| unreachable!("index is always 0..=59 by modular arithmetic"))
    }

    /// Lahiri (Chitrapaksha) ayanamsa in degrees for a given Julian Day.
    ///
    /// Linear model: 23°51′11.5″ at J2000.0 accumulating ~50.29″ per Julian
    /// year. Accurate to well under 0.01° across the supported BS 1975–2100
    /// range, replacing the previous fixed 24.0° approximation (which was
    /// ~0.25° off near the range edges — enough to misassign signs and
    /// nakshatras near boundaries).
    pub fn ayanamsa(&self, jd: f64) -> f64 {
        LAHIRI_AYANAMSA_J2000 + (jd - J2000_JD) * AYANAMSA_RATE_DEG_PER_DAY
    }

    /// Helper to convert tropical longitude to sidereal using the Lahiri ayanamsa.
    fn get_sidereal_longitude(&self, tropical_long: f64, jd: f64) -> f64 {
        normalize_degrees(tropical_long - self.ayanamsa(jd))
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
