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
        // Public, FFI-stable API: returns the bare occurrence dates. The clamp
        // signal (A1) is dropped here; callers that need it use
        // `generate_bs_instances_with_clamp` (e.g. the event orchestrator).
        Ok(self
            .generate_bs_instances_with_clamp(rule, start, end)?
            .into_iter()
            .map(|(date, _intended)| date)
            .collect())
    }

    /// Generate BS instances, reporting calendar-intrinsic day-clamping (A1).
    ///
    /// Each element is `(occurrence_date, intended_unclamped_date)`. The second
    /// field is `Some(intended)` when the calendar forced the occurrence to a
    /// different real day than the rule asked for (a non-existent target day,
    /// e.g. day 30/31 in a 29-day BS month), and `None` otherwise. This lets the
    /// orchestrator flag the resulting `EventInstance` as an exception instead of
    /// silently absorbing the clamp.
    pub fn generate_bs_instances_with_clamp(
        &self,
        rule: &BsRecurrenceRule,
        start: BsDate,
        end: BsDate,
    ) -> Result<Vec<(BsDate, Option<BsDate>)>> {
        rule.validate()?;

        let mut instances: Vec<(BsDate, Option<BsDate>)> = Vec::new();
        let mut current_frame_start = rule.anchor;
        let mut count = 0u32;

        // Safety guard against unbounded rules (no COUNT, no UNTIL, huge window).
        const MAX_INSTANCES: usize = 10_000;

        // Determine maximum occurrences
        let max_count = rule.count.unwrap_or(u32::MAX);

        loop {
            // Frame-level UNTIL bound: candidates expanded from a frame never precede
            // the frame start, so once the frame start is past UNTIL no further frame
            // can produce an in-bounds candidate. The per-candidate check below remains
            // authoritative for the boundary frame.
            if let Some(until) = rule.until {
                if current_frame_start > until {
                    break;
                }
            }
            if current_frame_start > end {
                // Heuristic: if frame start is past end, usually safe to stop
                // unless we have weird negative offsets (not supported yet)
                break;
            }

            // Expand candidates from the current frame
            let candidates = self.expand_candidates_with_clamp(current_frame_start, rule)?;

            for (candidate, intended) in candidates {
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
                    instances.push((candidate, intended));
                    count += 1;
                }
            }

            // Advance to next frame
            match self.advance_date(current_frame_start, rule) {
                Ok(next) => current_frame_start = next,
                Err(_) => break, // Can't advance further
            }

            // Refuse to silently truncate: callers must bound the rule. A
            // silent cap returned a clipped list indistinguishable from a
            // complete one.
            if instances.len() > MAX_INSTANCES {
                return Err(BsCalendarError::InstanceLimitExceeded(MAX_INSTANCES));
            }
        }

        Ok(instances)
    }

    /// Expand the current date into a list of candidate dates based on BYxxx rules.
    ///
    /// Each candidate is `(date, intended_unclamped_date)`; `intended` is
    /// `Some(..)` only when BYMONTHDAY named a day that does not exist in that BS
    /// month and the calendar clamped it (A1). All other candidates carry `None`.
    fn expand_candidates_with_clamp(
        &self,
        date: BsDate,
        rule: &BsRecurrenceRule,
    ) -> Result<Vec<(BsDate, Option<BsDate>)>> {
        let mut candidates: Vec<(BsDate, Option<BsDate>)> = Vec::new();

        // If no expansion rules, just return the date itself (implied instance)
        if rule.by_month.is_none() && rule.by_month_day.is_none() && rule.by_day.is_none() {
            candidates.push((date, None));
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
                                candidates.push((new_date, None));
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

                // 1. Initial Set: [(date, intended_clamp)]
                let mut set: Vec<(BsDate, Option<BsDate>)> = vec![(date, None)];

                // 2. BYMONTH (Limit) - if present and date.month not in it, clear set
                if let Some(ref months) = rule.by_month {
                    set.retain(|(d, _)| months.contains(&d.month));
                }

                // 3. BYMONTHDAY (Expand) - if present, replace days in set with new days
                set = self.expand_by_month_day_with_clamp(set, rule)?;

                // 4. BYDAY (Expand/Limit)
                // For Monthly string: "FREQ=MONTHLY;BYDAY=MO,WE" -> Every Mon/Wed in month. (Expand)
                set = self.expand_or_filter_by_day(set, rule)?;

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

                // Weekly never names a non-existent day → no clamp signal.
                candidates.extend(set.into_iter().map(|d| (d, None)));
            }
            BsFrequency::Daily => {
                // Simple filtering
                let mut set = vec![date];
                if let Some(ref months) = rule.by_month {
                    set.retain(|d| months.contains(&d.month));
                }
                if let Some(ref days) = rule.by_month_day {
                    // Clamp target days to the month length (consistent with Monthly/
                    // Yearly), so the last-day sentinel BYMONTHDAY=32 resolves to the
                    // actual last day instead of never matching.
                    let mut retained = Vec::new();
                    for d in set {
                        let month_days = self.conversion.calendar().get_month_days(d.year, d.month)?;
                        if days.iter().any(|&t| t.min(month_days) == d.day) {
                            retained.push(d);
                        }
                    }
                    set = retained;
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
                // Daily BYMONTHDAY is a filter, not an expand → no clamp signal.
                candidates.extend(set.into_iter().map(|d| (d, None)));
            }
        }

        // Candidates must be ascending within a frame: generate_bs_instances relies
        // on ascending order for its count/until/end early-returns. BYMONTHDAY and
        // BYMONTH expansions push in rule-list order, which may be unsorted. Sort by
        // the real occurrence date; on ties, order clamped (Some) before unclamped
        // (None) so the A1 signal survives the dedup below.
        candidates.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.is_some().cmp(&a.1.is_some())));
        // A2 (collision after clamp): two BY targets can clamp onto the same real
        // day (e.g. BYMONTHDAY=30,31 both -> 29 in a 29-day month). Dedup by the
        // real day so it is not emitted as two instances within one frame.
        candidates.dedup_by_key(|(date, _)| *date);

        Ok(candidates)
    }

    /// Helper for Yearly sub-expansion (same logic as Monthly roughly).
    ///
    /// Returns `(date, intended_unclamped_date)` candidates; `intended` is
    /// `Some(..)` only where BYMONTHDAY named a day that did not exist in the month
    /// and was clamped (A1).
    fn expand_sub_candidates(
        &self,
        date: BsDate,
        rule: &BsRecurrenceRule,
    ) -> Result<Vec<(BsDate, Option<BsDate>)>> {
        // Treat `date` as defining the Month we are in.
        // Apply BYMONTHDAY and BYDAY as if it was Monthly freq for that specific month.

        let set: Vec<(BsDate, Option<BsDate>)> = vec![(date, None)];

        // BYMONTHDAY (Expand), then BYDAY (Expand/Limit) — shared with the Monthly branch.
        let set = self.expand_by_month_day_with_clamp(set, rule)?;
        let set = self.expand_or_filter_by_day(set, rule)?;

        Ok(set)
    }

    /// BYMONTHDAY expansion with A1 clamp tracking. For each `(date, _)` in `set`,
    /// emit one candidate per `rule.by_month_day` target, clamping the target to the
    /// month length. When a target named a day that did not exist in the month, the
    /// returned `intended` carries the un-clamped date (the A1 signal); otherwise it
    /// is `None`. When `rule.by_month_day` is absent the set is returned unchanged.
    fn expand_by_month_day_with_clamp(
        &self,
        set: Vec<(BsDate, Option<BsDate>)>,
        rule: &BsRecurrenceRule,
    ) -> Result<Vec<(BsDate, Option<BsDate>)>> {
        let Some(ref days) = rule.by_month_day else {
            return Ok(set);
        };
        let mut new_set = Vec::new();
        for (d, _) in set {
            let month_days_count = self.conversion.calendar().get_month_days(d.year, d.month)?;
            for &target_day in days {
                let clamped = target_day.min(month_days_count);
                if let Ok(new_d) = BsDate::from_parts(d.year, d.month, clamped) {
                    let intended = if clamped != target_day {
                        BsDate::from_parts(d.year, d.month, target_day).ok()
                    } else {
                        None
                    };
                    new_set.push((new_d, intended));
                }
            }
        }
        Ok(new_set)
    }

    /// BYDAY for Monthly/Yearly frames. When BYMONTHDAY was also present the set has
    /// already been expanded to concrete days, so BYDAY acts as a weekday *filter*.
    /// Otherwise each entry represents a month and BYDAY *expands* to every matching
    /// weekday in that month. When `rule.by_day` is absent the set is unchanged.
    fn expand_or_filter_by_day(
        &self,
        mut set: Vec<(BsDate, Option<BsDate>)>,
        rule: &BsRecurrenceRule,
    ) -> Result<Vec<(BsDate, Option<BsDate>)>> {
        let Some(ref weekdays) = rule.by_day else {
            return Ok(set);
        };
        if rule.by_month_day.is_some() {
            // Limit/Filter
            set.retain(|(d, _)| {
                if let Ok(greg) = self.conversion.bs_to_gregorian(*d) {
                    let wd = greg.weekday().num_days_from_sunday() as u8;
                    weekdays.contains(&wd)
                } else {
                    false
                }
            });
            Ok(set)
        } else {
            // Expand to all matching weekdays in each month represented by the set.
            let mut new_set = Vec::new();
            for (d, _) in set {
                let month_days_count =
                    self.conversion.calendar().get_month_days(d.year, d.month)?;
                for day_num in 1..=month_days_count {
                    if let Ok(scan_date) = BsDate::from_parts(d.year, d.month, day_num) {
                        let greg = self.conversion.bs_to_gregorian(scan_date)?;
                        let wd = greg.weekday().num_days_from_sunday() as u8;
                        if weekdays.contains(&wd) {
                            new_set.push((scan_date, None));
                        }
                    }
                }
            }
            Ok(new_set)
        }
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
        let mut year = i32::from(date.year);
        let mut month = date.month.to_u8() as i32;

        month = month.checked_add(months).ok_or_else(|| {
            BsCalendarError::ConversionError(format!(
                "Month arithmetic overflow advancing {date} by {months} months"
            ))
        })?;

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

        let year = u16::try_from(year).map_err(|_| BsCalendarError::InvalidYear(0))?;

        // Clamp day to valid range for new month
        self.conversion
            .clamp_bs_date(year, new_month, date.day)
    }

    /// Advance a BS date by a number of years
    fn advance_years(&self, date: BsDate, years: i32) -> Result<BsDate> {
        // Checked arithmetic: silent `as u16` wraparound at the range edges
        // previously produced a nonsense year instead of an error.
        let new_year = i32::from(date.year)
            .checked_add(years)
            .and_then(|y| u16::try_from(y).ok())
            .ok_or(BsCalendarError::InvalidYear(date.year))?;

        // Clamp day to valid range for new year/month
        self.conversion
            .clamp_bs_date(new_year, date.month, date.day)
    }
}
