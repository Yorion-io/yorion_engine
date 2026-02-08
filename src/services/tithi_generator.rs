use crate::domain::bs_date::{BsDate, BsMonth};
use crate::domain::event::{CalendarVersion, EventInstance};
use crate::domain::recurrence::tithi_rules::TithiRecurrenceRule;
use crate::domain::tithi::Location;
use crate::error::Result;
use crate::ports::TimeProvider;
use crate::services::astronomical::AstronomicalService;
use crate::services::conversion::ConversionService;
use std::sync::Arc;

/// Generator for tithi-based event instances
pub struct TithiInstanceGenerator {
    conversion_service: Arc<ConversionService>,
    astronomical_service: Arc<AstronomicalService>,
    time_provider: Arc<dyn TimeProvider>,
}

impl TithiInstanceGenerator {
    pub fn new(
        conversion_service: Arc<ConversionService>,
        astronomical_service: Arc<AstronomicalService>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        TithiInstanceGenerator {
            conversion_service,
            astronomical_service,
            time_provider,
        }
    }

    /// Generate tithi-based instances within a date range
    pub fn generate_instances(
        &self,
        event_id: &str,
        title: &str,
        rule: &TithiRecurrenceRule,
        start_date: BsDate,
        end_date: BsDate,
        version: CalendarVersion,
        location: Location,
    ) -> Result<Vec<EventInstance>> {
        rule.validate()?;

        let mut instances = Vec::new();
        let mut count = 0;

        // We start from the anchor date or the start_date, whichever is later
        let actual_start = if start_date > rule.anchor {
            start_date
        } else {
            rule.anchor
        };
        let actual_end = if let Some(until) = rule.until {
            if until < end_date {
                until
            } else {
                end_date
            }
        } else {
            end_date
        };

        // If actual_start > actual_end, no instances in range
        if actual_start > actual_end {
            return Ok(instances);
        }

        // Create a cache for lunar month information to avoid redundant astronomical searches.
        // A lunar cycle is ~29.5 days, so we can cache this data for that period.
        let mut cached_lunar_data: Option<(f64, BsMonth, bool)> = None; // (next_amavasya_jd, name, is_adhik)

        // Iterate through all days in the BS range
        let mut current_bs = actual_start;

        while current_bs <= actual_end {
            // 1. Check if count limit reached
            if let Some(max_count) = rule.count {
                if count >= max_count {
                    break;
                }
            }

            // 2. Pre-filter by BS month (CHEAP)
            // If the solar month doesn't match and we're not filtering by lunar month or adhik, we can skip.
            // Even if we ARE filtering by lunar month, if solar month doesn't match the final condition will fail.
            let month_matches = if let Some(ref months) = rule.by_month {
                months.contains(&current_bs.month)
            } else {
                true
            };

            if !month_matches {
                if let Some(next_day) = self.next_day(current_bs) {
                    current_bs = next_day;
                    continue;
                } else {
                    break;
                }
            }

            // 3. Tithi Calculation (MODERATE)
            let gregorian = self.conversion_service.bs_to_gregorian(current_bs)?;
            let sunrise_time = self.time_provider.sunrise_time(gregorian, location)?;

            let sunrise_dt = gregorian
                .and_time(sunrise_time)
                .and_local_timezone(
                    chrono::FixedOffset::east_opt(location.timezone_offset_mins * 60).unwrap(),
                )
                .unwrap()
                .with_timezone(&chrono::Utc);

            let tithi = self
                .astronomical_service
                .calculate_tithi_with_location(sunrise_dt, &location)?;
            let jd = self.astronomical_service.get_julian_day(sunrise_dt);

            // 4. Lunar Month / Adhik Info (EXPENSIVE - Cached)
            // Refresh cache if needed (if first run or if we've passed the next Amavasya)
            if cached_lunar_data
                .as_ref()
                .map_or(true, |(ama_jd, _, _)| jd > *ama_jd)
            {
                let next_amavasya_jd = self.astronomical_service.find_next_amavasya(jd)?;
                let amavasya_dt = self.jd_to_utc(next_amavasya_jd);
                let amavasya_bs = self
                    .conversion_service
                    .gregorian_to_bs(amavasya_dt.date_naive())?;
                let lunar_month_name = amavasya_bs.month.prev();

                let is_adhik = if rule.skip_adhik || rule.by_lunar_month.is_some() {
                    self.astronomical_service.is_adhik_month(jd)?
                } else {
                    false
                };

                cached_lunar_data = Some((next_amavasya_jd, lunar_month_name, is_adhik));
            }

            let (_, lunar_month_name, is_adhik) = cached_lunar_data.as_ref().unwrap();

            let lunar_month_matches = if let Some(ref lunar_months) = rule.by_lunar_month {
                lunar_months.contains(lunar_month_name)
            } else {
                true
            };

            let adhik_matches = if rule.skip_adhik { !*is_adhik } else { true };

            if lunar_month_matches && adhik_matches && rule.matches_tithi(tithi) {
                let instance_id = format!("{}-{}", event_id, current_bs.format());
                let mut instance =
                    EventInstance::new(instance_id, current_bs, title.to_string(), version.clone());
                instance.tithi = Some(tithi);
                instance.parent_event_id = Some(event_id.to_string());
                instances.push(instance);
                count += 1;
            }

            // Increment date
            if let Some(next_day) = self.next_day(current_bs) {
                current_bs = next_day;
            } else {
                break; // End of calendar data
            }
        }

        Ok(instances)
    }

    /// Helper to convert JD to DateTime<Utc>
    fn jd_to_utc(&self, jd: f64) -> chrono::DateTime<chrono::Utc> {
        let jd = jd + 0.5;
        let z = jd.floor();
        let f = jd - z;

        let a = if z < 2299161.0 {
            z
        } else {
            let alpha = ((z - 1867216.25) / 36524.25).floor();
            z + 1.0 + alpha - (alpha / 4.0).floor()
        };

        let b = a + 1524.0;
        let c = ((b - 122.1) / 365.25).floor();
        let d = (365.25 * c).floor();
        let e = ((b - d) / 30.6001).floor();

        let day = (b - d - (30.6001 * e).floor() + f).floor();
        let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
        let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };

        let h = (f * 24.0).floor();
        let m = ((f * 24.0 - h) * 60.0).floor();
        let s = (((f * 24.0 - h) * 60.0 - m) * 60.0).floor();

        let naive_date =
            chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32).unwrap();
        let naive_time = chrono::NaiveTime::from_hms_opt(h as u32, m as u32, s as u32).unwrap();

        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            naive_date.and_time(naive_time),
            chrono::Utc,
        )
    }

    /// Helper to increment one day in BS
    fn next_day(&self, date: BsDate) -> Option<BsDate> {
        let provider = self.conversion_service.calendar();
        let month = crate::domain::bs_date::BsMonth::from_u8(date.month_u8()).ok()?;
        let days_in_month = provider.get_month_days(date.year, month).ok()?;

        if date.day < days_in_month {
            Some(BsDate::new(date.year, date.month_u8(), date.day + 1).ok()?)
        } else if date.month_u8() < 12 {
            Some(BsDate::new(date.year, date.month_u8() + 1, 1).ok()?)
        } else {
            Some(BsDate::new(date.year + 1, 1, 1).ok()?)
        }
    }
}
