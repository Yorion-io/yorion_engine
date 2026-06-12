use crate::domain::bs_date::BsDate;
use crate::domain::recurrence::Recurrence;
use crate::domain::tithi::Tithi;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Unified event definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub recurrence: Recurrence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Calendar version identifier
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(getter_with_clone)
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarVersion {
    pub version: String,
    pub is_official: bool,
}

impl CalendarVersion {
    pub fn new(version: String, is_official: bool) -> Self {
        CalendarVersion {
            version,
            is_official,
        }
    }

    pub fn projected(version: String) -> Self {
        CalendarVersion {
            version,
            is_official: false,
        }
    }

    pub fn official(version: String) -> Self {
        CalendarVersion {
            version,
            is_official: true,
        }
    }
}

/// Event instance - a single occurrence of an event
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(getter_with_clone)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInstance {
    /// Unique identifier for this instance
    pub id: String,

    /// BS date of occurrence
    pub bs_date: BsDate,

    /// Gregorian (AD) date of occurrence — the same real day as `bs_date`,
    /// expressed in the Gregorian calendar. Always populated.
    #[cfg_attr(feature = "wasm", wasm_bindgen(skip))]
    pub ad_date: NaiveDate,

    /// Optional tithi if this is a tithi-based event
    pub tithi: Option<Tithi>,

    /// Event title/description
    pub title: String,

    /// Calendar version used to generate this instance
    pub calendar_version: CalendarVersion,

    /// Whether this instance departs from a naive reading of the rule. Currently
    /// set only for calendar-intrinsic day-clamping (a non-existent target day
    /// forced onto the last valid day of the month).
    pub is_exception: bool,

    /// Parent event ID if this is a recurring instance
    pub parent_event_id: Option<String>,

    /// The intended (un-clamped) BS date when `is_exception` is set because the
    /// calendar clamped the rule's target day; `None` otherwise.
    pub original_date: Option<BsDate>,
}

impl EventInstance {
    /// Create a new event instance
    pub fn new(
        id: String,
        bs_date: BsDate,
        ad_date: NaiveDate,
        title: String,
        calendar_version: CalendarVersion,
    ) -> Self {
        EventInstance {
            id,
            bs_date,
            ad_date,
            tithi: None,
            title,
            calendar_version,
            is_exception: false,
            parent_event_id: None,
            original_date: None,
        }
    }

    /// Create an instance from a recurrence
    pub fn from_recurrence(
        id: String,
        bs_date: BsDate,
        ad_date: NaiveDate,
        title: String,
        calendar_version: CalendarVersion,
        parent_event_id: String,
    ) -> Self {
        let mut instance = Self::new(id, bs_date, ad_date, title, calendar_version);
        instance.parent_event_id = Some(parent_event_id);
        instance
    }

    /// Create a tithi-based instance
    pub fn with_tithi(mut self, tithi: Tithi) -> Self {
        self.tithi = Some(tithi);
        self
    }

    /// Mark as exception
    pub fn as_exception(mut self, original_date: BsDate) -> Self {
        self.is_exception = true;
        self.original_date = Some(original_date);
        self
    }

    /// Check if this instance needs reconciliation
    /// (i.e., was generated with projected data and official data is now available)
    pub fn needs_reconciliation(&self, current_version: &CalendarVersion) -> bool {
        !self.calendar_version.is_official && current_version.is_official
    }
}
