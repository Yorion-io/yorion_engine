use crate::error::{BsCalendarError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Bikram Sambat month enumeration
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BsMonth {
    Baisakh = 1,
    Jestha = 2,
    Ashadh = 3,
    Shrawan = 4,
    Bhadra = 5,
    Ashwin = 6,
    Kartik = 7,
    Mangsir = 8,
    Poush = 9,
    Magh = 10,
    Falgun = 11,
    Chaitra = 12,
}

impl BsMonth {
    /// Create BsMonth from u8 (1-12)
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(BsMonth::Baisakh),
            2 => Ok(BsMonth::Jestha),
            3 => Ok(BsMonth::Ashadh),
            4 => Ok(BsMonth::Shrawan),
            5 => Ok(BsMonth::Bhadra),
            6 => Ok(BsMonth::Ashwin),
            7 => Ok(BsMonth::Kartik),
            8 => Ok(BsMonth::Mangsir),
            9 => Ok(BsMonth::Poush),
            10 => Ok(BsMonth::Magh),
            11 => Ok(BsMonth::Falgun),
            12 => Ok(BsMonth::Chaitra),
            _ => Err(BsCalendarError::InvalidMonth(value)),
        }
    }

    /// Convert to u8 (1-12)
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Get month name in English
    pub fn name(&self) -> &'static str {
        match self {
            BsMonth::Baisakh => "Baisakh",
            BsMonth::Jestha => "Jestha",
            BsMonth::Ashadh => "Ashadh",
            BsMonth::Shrawan => "Shrawan",
            BsMonth::Bhadra => "Bhadra",
            BsMonth::Ashwin => "Ashwin",
            BsMonth::Kartik => "Kartik",
            BsMonth::Mangsir => "Mangsir",
            BsMonth::Poush => "Poush",
            BsMonth::Magh => "Magh",
            BsMonth::Falgun => "Falgun",
            BsMonth::Chaitra => "Chaitra",
        }
    }

    /// Get month name in Nepali (Devanagari)
    pub fn nepali_name(&self) -> &'static str {
        match self {
            BsMonth::Baisakh => "वैशाख",
            BsMonth::Jestha => "जेठ",
            BsMonth::Ashadh => "असार",
            BsMonth::Shrawan => "साउन",
            BsMonth::Bhadra => "भदौ",
            BsMonth::Ashwin => "असोज",
            BsMonth::Kartik => "कात्तिक",
            BsMonth::Mangsir => "मंसिर",
            BsMonth::Poush => "पुस",
            BsMonth::Magh => "माघ",
            BsMonth::Falgun => "फागुन",
            BsMonth::Chaitra => "चैत",
        }
    }

    pub fn name_with_language(&self, lang: crate::domain::Language) -> &'static str {
        match lang {
            crate::domain::Language::English => self.name(),
            crate::domain::Language::Nepali => self.nepali_name(),
        }
    }

    /// Get next month (wraps to Baisakh after Chaitra)
    pub fn next(self) -> Self {
        match self {
            BsMonth::Chaitra => BsMonth::Baisakh,
            _ => BsMonth::from_u8(self.to_u8() + 1).unwrap(),
        }
    }

    /// Get previous month (wraps to Chaitra before Baisakh)
    pub fn prev(self) -> Self {
        match self {
            BsMonth::Baisakh => BsMonth::Chaitra,
            _ => BsMonth::from_u8(self.to_u8() - 1).unwrap(),
        }
    }
}

impl fmt::Display for BsMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TryFrom<u8> for BsMonth {
    type Error = BsCalendarError;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_u8(value)
    }
}

/// Bikram Sambat date
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(getter_with_clone)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BsDate {
    pub year: u16,
    pub month: BsMonth,
    pub day: u8,
}

impl BsDate {
    /// Create a new BS date with validation
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self> {
        let month = BsMonth::from_u8(month)?;

        // Basic day validation (1-32)
        if !(1..=32).contains(&day) {
            return Err(BsCalendarError::InvalidDay(day, month.to_u8()));
        }

        Ok(BsDate { year, month, day })
    }

    /// Create from BsMonth enum
    pub fn from_parts(year: u16, month: BsMonth, day: u8) -> Result<Self> {
        if !(1..=32).contains(&day) {
            return Err(BsCalendarError::InvalidDay(day, month.to_u8()));
        }

        Ok(BsDate { year, month, day })
    }

    /// Get the month as u8 (1-12)
    pub fn month_u8(&self) -> u8 {
        self.month.to_u8()
    }

    /// Format as YYYY-MM-DD
    pub fn format(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month.to_u8(), self.day)
    }
}

impl fmt::Display for BsDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04} {} {:02}", self.year, self.month.name(), self.day)
    }
}
