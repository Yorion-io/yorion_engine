use thiserror::Error;

/// Normative BS-RRULE validation rule identifiers (spec §11).
///
/// A conformant parser MUST reject a BS-RRULE that violates one of these rules.
/// The discriminant text matches the spec's `Vn` code so it round-trips into the
/// conformance vectors' `expected.rejectReason`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RRuleRejectReason {
    /// V1 — any parameter lacks `=`.
    V1,
    /// V2 — family is BS or AD and `FREQ` is missing.
    V2,
    /// V3 — `FREQ` value is not one of the four defined frequencies.
    V3,
    /// V4 — family is BS or PANCHANGA and `DTSTART` is missing.
    V4,
    /// V5 — `DTSTART`/`UNTIL` is not exactly 8 digits, or its month is outside 1–12.
    V5,
    /// V6 — `INTERVAL` or `COUNT` is present but not a positive integer.
    V6,
    /// V7 — `BYMONTH` / `X-BYLUNARMONTH` contains a value outside 1–12.
    V7,
    /// V8 — `BYDAY` contains a token that is not a defined weekday code.
    V8,
    /// V9 — family is PANCHANGA and `X-TITHI` is missing, empty, or unrecognized.
    V9,
    /// V10 — `X-PAKSHA` is present with a value other than `SHUKLA`/`KRISHNA`.
    V10,
    /// V11 — `X-TAKE` is present with a value other than `FIRST`.
    V11,
}

impl RRuleRejectReason {
    /// The spec code (`"V1"`..`"V11"`).
    pub fn code(self) -> &'static str {
        match self {
            RRuleRejectReason::V1 => "V1",
            RRuleRejectReason::V2 => "V2",
            RRuleRejectReason::V3 => "V3",
            RRuleRejectReason::V4 => "V4",
            RRuleRejectReason::V5 => "V5",
            RRuleRejectReason::V6 => "V6",
            RRuleRejectReason::V7 => "V7",
            RRuleRejectReason::V8 => "V8",
            RRuleRejectReason::V9 => "V9",
            RRuleRejectReason::V10 => "V10",
            RRuleRejectReason::V11 => "V11",
        }
    }
}

impl std::fmt::Display for RRuleRejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

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

    #[error("RRULE rejected [{reason}]: {detail}")]
    RRuleRejected {
        reason: RRuleRejectReason,
        detail: String,
    },

    #[error("Unsupported RRULE feature: {0}")]
    UnsupportedRRuleFeature(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Astronomical calculation failed: {0}")]
    AstronomicalError(String),
}

impl BsCalendarError {
    /// Construct a spec-coded RRULE rejection (§11).
    pub fn rrule_rejected(reason: RRuleRejectReason, detail: impl Into<String>) -> Self {
        BsCalendarError::RRuleRejected {
            reason,
            detail: detail.into(),
        }
    }

    /// The spec reject-reason code if this error is a coded RRULE rejection.
    pub fn reject_reason(&self) -> Option<RRuleRejectReason> {
        match self {
            BsCalendarError::RRuleRejected { reason, .. } => Some(*reason),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, BsCalendarError>;
