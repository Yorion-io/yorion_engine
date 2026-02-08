use crate::domain::bs_date::{BsDate, BsMonth};
use crate::domain::recurrence::{AdRecurrenceRule, BsFrequency, BsRecurrenceRule};
use crate::error::{BsCalendarError, Result};
use crate::services::ConversionService;
use chrono::{Datelike, NaiveDate};
use rrule::RRuleSet;
use std::str::FromStr;
use std::sync::Arc;

/// Instance generator for expanding recurrence rules into occurrences
///
/// Generates event instances from BS recurrence rules within a given date range.
/// Uses lazy evaluation and respects BS calendar semantics (e.g., month-day clamping).
pub struct InstanceGenerator {
    conversion: Arc<ConversionService>,
}

impl InstanceGenerator {
    /// Create a new instance generator
    pub fn new(conversion: Arc<ConversionService>) -> Self {
        InstanceGenerator { conversion }
    }

    /// Generate instances from a BS recurrence rule within a date range
    ///
    /// Returns a vector of BS dates representing occurrences.
    /// The range is inclusive on both ends.
    /// Generate instances from a BS recurrence rule within a date range
    ///
    /// Returns a vector of BS dates representing occurrences.
    /// The range is inclusive on both ends.
    pub fn generate_bs_instances(
        &self,
        rule: &BsRecurrenceRule,
        start: BsDate,
        end: BsDate,
    ) -> Result<Vec<BsDate>> {
        rule.validate()?;

        let mut instances = Vec::new();
        let mut current_frame_start = rule.anchor;
        let mut count = 0u32;

        // Determine maximum occurrences
        let max_count = rule.count.unwrap_or(u32::MAX);

        loop {
            // Check if we've passed the until date (early check on frame)
            // But strict check is on candidates
            if let Some(until) = rule.until {
                if current_frame_start > until {
                    // If the frame start is way past until, we might stop.
                    // But strictly we should check candidates.
                    // For yearly/monthly, frame start might be before until, but candidates after.
                }
            }
            if current_frame_start > end {
                // Heuristic: if frame start is past end, usually safe to stop
                // unless we have weird negative offsets (not supported yet)
                break;
            }

            // Expand candidates from the current frame
            let candidates = self.expand_candidates(current_frame_start, rule)?;

            for candidate in candidates {
                // Stop if we hit count limit
                if count >= max_count {
                    return Ok(instances);
                }

                // Check until limit
                if let Some(until) = rule.until {
                    if candidate > until {
                        return Ok(instances);
                    }
                }

                // Check range window
                if candidate > end {
                    return Ok(instances);
                }

                if candidate >= start {
                    // One final check: does this candidate actually match the rule?
                    // The expansion generates POTENTIAL candidates based on BY rules.
                    // We might need to filter them further if there are other constraints.
                    // For now, assume expansion is correct.
                    instances.push(candidate);
                    count += 1;
                }
            }

            // Advance to next frame
            match self.advance_date(current_frame_start, rule) {
                Ok(next) => current_frame_start = next,
                Err(_) => break, // Can't advance further
            }

            // Safety check
            if instances.len() > 10000 {
                break;
            }
        }

        Ok(instances)
    }

    /// Expand the current date into a list of candidate dates based on BYxxx rules
    fn expand_candidates(&self, date: BsDate, rule: &BsRecurrenceRule) -> Result<Vec<BsDate>> {
        let mut candidates = Vec::new();

        // If no expansion rules, just return the date itself (implied instance)
        if rule.by_month.is_none() && rule.by_month_day.is_none() && rule.by_day.is_none() {
            candidates.push(date);
            return Ok(candidates);
        }

        // Handle expansions based on Frequency
        match rule.frequency {
            BsFrequency::Yearly => {
                // Expand BYMONTH
                if let Some(ref months) = rule.by_month {
                    for &month in months {
                        // For each month, keep the day (clamped)
                        if let Ok(new_date) =
                            self.conversion.clamp_bs_date(date.year, month, date.day)
                        {
                            // Recursive expansion for days?
                            // If BYMONTHDAY also exists, we should apply it here?
                            // RFC says BY rules are applied in specific order: MONTH, WEEKNO, YEARDAY, MONTHDAY, DAY...
                            // Implementing full hierarchy is hard.
                            // Simplifying: Assume max 1 expansion rule usually.
                            // But if BYMONTH and BYMONTHDAY both exist: Year -> Month -> Days

                            if rule.by_month_day.is_some() || rule.by_day.is_some() {
                                // Expand sub-rules
                                let sub_candidates = self.expand_sub_candidates(new_date, rule)?;
                                candidates.extend(sub_candidates);
                            } else {
                                candidates.push(new_date);
                            }
                        }
                    }
                } else {
                    // No BYMONTH, just year frame. Check sub-rules.
                    let sub_candidates = self.expand_sub_candidates(date, rule)?;
                    candidates.extend(sub_candidates);
                }
            }
            BsFrequency::Monthly => {
                // No BYMONTH expansion (limit instead? RFC: BYMONTH limits monthly freq).
                // Actually if FREQ=MONTHLY, BYMONTH limits.
                // My expansion logic above for Yearly was "Replace month".
                // Logic:
                // 1. Start with [date]
                // 2. Apply BYMONTH (Limit)
                // 3. Apply BYMONTHDAY (Expand)
                // 4. Apply BYDAY (Expand/Limit depending on context)

                // Let's adopt a "Filter + Expand" pipeline.

                // 1. Initial Set: [date]
                let mut set = vec![date];

                // 2. BYMONTH (Limit) - if present and date.month not in it, clear set
                if let Some(ref months) = rule.by_month {
                    set.retain(|d| months.contains(&d.month));
                }

                // 3. BYMONTHDAY (Expand) - if present, replace days in set with new days
                if let Some(ref days) = rule.by_month_day {
                    let mut new_set = Vec::new();
                    for d in set {
                        let month_days_count =
                            self.conversion.calendar().get_month_days(d.year, d.month)?;
                        for &target_day in days {
                            let clamped = target_day.min(month_days_count);
                            if let Ok(new_d) = BsDate::from_parts(d.year, d.month, clamped) {
                                new_set.push(new_d);
                            }
                        }
                    }
                    set = new_set;
                }

                // 4. BYDAY (Expand/Limit)
                // For Monthly string: "FREQ=MONTHLY;BYDAY=MO,WE" -> Every Mon/Wed in month. (Expand)
                if let Some(ref weekdays) = rule.by_day {
                    let mut new_set = Vec::new();
                    // If we already expanded by day (e.g. BYMONTHDAY), then BYDAY is a Filter (Limit).
                    // If we didn't expand by day yet (so set contains just anchor/month-start), expand to all matching weekdays in month.

                    if rule.by_month_day.is_some() {
                        // Limit/Filter
                        set.retain(|d| {
                            if let Ok(greg) = self.conversion.bs_to_gregorian(*d) {
                                let wd = greg.weekday().num_days_from_sunday() as u8;
                                weekdays.contains(&wd)
                            } else {
                                false
                            }
                        });
                    } else {
                        // Expand to all occurrences in the month
                        // For each date in set (which represents a month), find all matching weekdays
                        for d in set {
                            let month_days_count =
                                self.conversion.calendar().get_month_days(d.year, d.month)?;
                            for day_num in 1..=month_days_count {
                                if let Ok(scan_date) = BsDate::from_parts(d.year, d.month, day_num)
                                {
                                    let greg = self.conversion.bs_to_gregorian(scan_date)?;
                                    let wd = greg.weekday().num_days_from_sunday() as u8;
                                    if weekdays.contains(&wd) {
                                        new_set.push(scan_date);
                                    }
                                }
                            }
                        }
                        set = new_set;
                    }
                }

                candidates.extend(set);
            }
            BsFrequency::Weekly => {
                // 1. Initial Set: [date]
                let mut set = vec![date];

                // 2. BYMONTH (Limit)
                if let Some(ref months) = rule.by_month {
                    set.retain(|d| months.contains(&d.month));
                }

                // 3. BYDAY (Expand)
                // FREQ=WEEKLY; BYDAY=MO,TU -> Expand to Mo/Tu in that week.
                // We need to determine "that week".
                // BS Calendar doesn't strictly define "Week 1", but we can assume standard 7-day windows.
                // Or better: The week containing the current date.
                // We find the Sunday of this week, then scan 7 days.
                if let Some(ref weekdays) = rule.by_day {
                    let mut new_set = Vec::new();
                    for d in set {
                        let greg = self.conversion.bs_to_gregorian(d)?;
                        let wd_idx = greg.weekday().num_days_from_sunday() as i64; // Sun=0
                                                                                   // Start of week (Sunday)
                        let sow_greg = greg - chrono::Duration::days(wd_idx);

                        for &target_wd in weekdays {
                            let offset = target_wd as i64; // 0-6
                            let target_greg = sow_greg + chrono::Duration::days(offset);
                            let target_bs = self.conversion.gregorian_to_bs(target_greg)?;
                            new_set.push(target_bs);
                        }
                    }
                    // Sort because BYDAY order in list might not match date order (e.g. MO,SU -> SU comes first)
                    // But we generated based on sorted offset 0..6? No, strictly we used target_wd which might be unordered.
                    // We should sort.
                    new_set.sort();
                    set = new_set;
                }

                candidates.extend(set);
            }
            BsFrequency::Daily => {
                // Simple filtering
                let mut set = vec![date];
                if let Some(ref months) = rule.by_month {
                    set.retain(|d| months.contains(&d.month));
                }
                if let Some(ref days) = rule.by_month_day {
                    set.retain(|d| days.contains(&d.day)); // No clamping here? strict equality
                }
                if let Some(ref weekdays) = rule.by_day {
                    set.retain(|d| {
                        if let Ok(greg) = self.conversion.bs_to_gregorian(*d) {
                            let wd = greg.weekday().num_days_from_sunday() as u8;
                            weekdays.contains(&wd)
                        } else {
                            false
                        }
                    });
                }
                candidates.extend(set);
            }
        }

        Ok(candidates)
    }

    /// Helper for Yearly sub-expansion (same logic as Monthly roughly)
    fn expand_sub_candidates(&self, date: BsDate, rule: &BsRecurrenceRule) -> Result<Vec<BsDate>> {
        // Treat `date` as defining the Month we are in.
        // Apply BYMONTHDAY and BYDAY as if it was Monthly freq for that specific month.

        let mut set = vec![date];

        // BYMONTHDAY (Expand)
        if let Some(ref days) = rule.by_month_day {
            let mut new_set = Vec::new();
            for d in set {
                let month_days_count =
                    self.conversion.calendar().get_month_days(d.year, d.month)?;
                for &target_day in days {
                    let clamped = target_day.min(month_days_count);
                    if let Ok(new_d) = BsDate::from_parts(d.year, d.month, clamped) {
                        new_set.push(new_d);
                    }
                }
            }
            set = new_set;
        }

        // BYDAY
        if let Some(ref weekdays) = rule.by_day {
            let mut new_set = Vec::new();
            if rule.by_month_day.is_some() {
                // Limit
                set.retain(|d| {
                    if let Ok(greg) = self.conversion.bs_to_gregorian(*d) {
                        let wd = greg.weekday().num_days_from_sunday() as u8;
                        weekdays.contains(&wd)
                    } else {
                        false
                    }
                });
            } else {
                // Expand to all occurrences in the month
                for d in set {
                    let month_days_count =
                        self.conversion.calendar().get_month_days(d.year, d.month)?;
                    for day_num in 1..=month_days_count {
                        if let Ok(scan_date) = BsDate::from_parts(d.year, d.month, day_num) {
                            let greg = self.conversion.bs_to_gregorian(scan_date)?;
                            let wd = greg.weekday().num_days_from_sunday() as u8;
                            if weekdays.contains(&wd) {
                                new_set.push(scan_date);
                            }
                        }
                    }
                }
                set = new_set;
            }
        }

        Ok(set)
    }

    /// Generate instances from an AD recurrence rule within a date range
    pub fn generate_ad_instances(
        &self,
        rule: &AdRecurrenceRule,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<NaiveDate>> {
        // Use rrule crate for AD instances
        // We use UTC for all conversions to ensure date stability
        let rrule_set = RRuleSet::from_str(&rule.rrule).map_err(|e| {
            BsCalendarError::InvalidRecurrenceRule(format!("Failed to parse RRULE: {}", e))
        })?;

        // Convert range to DateTime<Utc>
        // Start of day for start date
        let start_dt = start
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                BsCalendarError::InvalidRecurrenceRule("Invalid start date".to_string())
            })?
            .and_utc();

        // End of day for end date
        let end_dt = end
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| BsCalendarError::InvalidRecurrenceRule("Invalid end date".to_string()))?
            .and_utc();

        // Generate instances
        // We iterate through occurrences properly
        // Note: rrule iterator yields DateTime<Tz>
        // We convert back to NaiveDate

        let instances: Vec<NaiveDate> = rrule_set
            .into_iter()
            .skip_while(|dt| dt < &start_dt)
            .take_while(|dt| dt <= &end_dt)
            .map(|dt| dt.date_naive())
            .collect();

        Ok(instances)
    }

    /// Advance a date according to the recurrence rule
    fn advance_date(&self, date: BsDate, rule: &BsRecurrenceRule) -> Result<BsDate> {
        match rule.frequency {
            BsFrequency::Daily => self.advance_days(date, rule.interval as i32),
            BsFrequency::Weekly => self.advance_days(date, (rule.interval as i32) * 7),
            BsFrequency::Monthly => self.advance_months(date, rule.interval as i32),
            BsFrequency::Yearly => self.advance_years(date, rule.interval as i32),
        }
    }

    /// Advance a BS date by a number of days
    fn advance_days(&self, date: BsDate, days: i32) -> Result<BsDate> {
        let gregorian = self.conversion.bs_to_gregorian(date)?;
        let advanced = gregorian + chrono::Duration::days(days as i64);
        self.conversion.gregorian_to_bs(advanced)
    }

    /// Advance a BS date by a number of months
    fn advance_months(&self, date: BsDate, months: i32) -> Result<BsDate> {
        let mut year = date.year as i32;
        let mut month = date.month.to_u8() as i32;

        month += months;

        // Handle month overflow/underflow
        while month > 12 {
            month -= 12;
            year += 1;
        }
        while month < 1 {
            month += 12;
            year -= 1;
        }

        let new_month = BsMonth::from_u8(month as u8)?;

        // Clamp day to valid range for new month
        self.conversion
            .clamp_bs_date(year as u16, new_month, date.day)
    }

    /// Advance a BS date by a number of years
    fn advance_years(&self, date: BsDate, years: i32) -> Result<BsDate> {
        let new_year = (date.year as i32 + years) as u16;

        // Clamp day to valid range for new year/month
        self.conversion
            .clamp_bs_date(new_year, date.month, date.day)
    }
    /// Generate instances from a Tithi recurrence rule via heuristic search
    ///
    /// Optimized to skip non-matching days by estimating the time to the next target tithi.
    pub fn generate_tithi_instances(
        &self,
        rule: &crate::domain::recurrence::TithiRecurrenceRule,
        start: BsDate,
        end: BsDate,
        astro: &crate::services::AstronomicalService,
    ) -> Result<Vec<BsDate>> {
        rule.validate()?;

        let mut instances = Vec::new();
        let mut current = {
            if rule.anchor > start {
                rule.anchor
            } else {
                start
            }
        };

        let mut count = 0u32;
        let max_count = rule.count.unwrap_or(u32::MAX);

        // Loop until we exceed end date
        while current <= end {
            // Check limits
            if count >= max_count {
                break;
            }
            if let Some(until) = rule.until {
                if current > until {
                    break;
                }
            }

            // 1. Calculate Tithi for the current day
            let gregorian = self.conversion.bs_to_gregorian(current)?;
            let location = crate::domain::tithi::Location::KATHMANDU; // Standard specific location for Tithi

            // Using calculate_tithi_for_date (sunrise)
            let current_tithi = astro.calculate_tithi_for_date(gregorian, location)?;

            // 2. Check if it matches
            if rule.matches_tithi(current_tithi) {
                // Apply date filters if any
                if self.matches_tithi_date_filters(&current, rule)? {
                    instances.push(current);
                    count += 1;
                }

                // Jump at least 1 day, or heuristic for next occurrence
                // If the rule is for specific tithi (e.g. Ekadashi), recurrence is roughly cyclic
                // But we just matched it, so next match is likely far away (unless consecutive days match same tithi, which is possible)
                // Safest small jump is 1 day to catch consecutive matches (Kshay/Vriddhi tithi effects)
                // But if we want speed, we can check if the *next* day also matches.
                // For now, strict correctness: jump 1 day.
                // Optimization: If we matched, we can maybe jump ~14 days if it's a specific Paksha/Tithi rule,
                // but "Every Ekadashi" is ~15 days.
                // Let's keep simple safe jump=1 for match case to be correct with Kshaya/Vriddhi.
                current = self.advance_days(current, 1)?;
                continue;
            }

            // 3. Heuristic Jumping (Optimization)
            // Calculate distance in "tithi indices" to the target
            // Tithi cycle is 1-30 (Shukla 1-15, Krishna 1-15)
            // 1 solar day approx 0.98 tithi days.

            let mut nearest_dist = 30u8;
            let current_idx = current_tithi.index_1_to_30();

            for &target in &rule.by_tithi {
                let target_idx = target.index_1_to_30();

                let dist = if target_idx >= current_idx {
                    target_idx - current_idx
                } else {
                    (30 - current_idx) + target_idx
                };

                nearest_dist = nearest_dist.min(dist);

                // Special handling for dual-paksha rules (like Ekadashi):
                // If rule ignores paksha, we have another target at (target_idx + 15) % 30
                let is_any_paksha =
                    rule.paksha_filter.is_none() && !target.is_purnima() && !target.is_amavasya();
                if is_any_paksha {
                    let alt_target_idx = (target_idx + 15 - 1) % 30 + 1;
                    let alt_dist = if alt_target_idx >= current_idx {
                        alt_target_idx - current_idx
                    } else {
                        (30 - current_idx) + alt_target_idx
                    };
                    nearest_dist = nearest_dist.min(alt_dist);
                }
            }

            let jump_days = if nearest_dist > 2 {
                // Heuristic: jump slightly less than distance to be safe
                // (moon can speed up, tithis can be skipped)
                (nearest_dist as f32 * 0.9) as i32
            } else {
                1
            };

            current = self.advance_days(current, jump_days.max(1))?;
        }

        Ok(instances)
    }

    fn matches_tithi_date_filters(
        &self,
        date: &BsDate,
        rule: &crate::domain::recurrence::TithiRecurrenceRule,
    ) -> Result<bool> {
        // Check BYMONTH filter
        if let Some(ref months) = rule.by_month {
            if !months.contains(&date.month) {
                return Ok(false);
            }
        }

        // Skip Adhik Month if requested
        // This requires Astro check which is expensive.
        // For now, ignoring as basic implementation.

        Ok(true)
    }
}
