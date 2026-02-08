use crate::error::{BsCalendarError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Paksha (lunar fortnight) - waxing or waning moon phase
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Paksha {
    /// Shukla Paksha - waxing moon (bright fortnight)
    Shukla,
    /// Krishna Paksha - waning moon (dark fortnight)
    Krishna,
}

impl Paksha {
    pub fn name(&self) -> &'static str {
        match self {
            Paksha::Shukla => "Shukla",
            Paksha::Krishna => "Krishna",
        }
    }

    pub fn nepali_name(&self) -> &'static str {
        match self {
            Paksha::Shukla => "शुक्ल",
            Paksha::Krishna => "कृष्ण",
        }
    }

    pub fn name_with_language(&self, lang: crate::domain::Language) -> &'static str {
        match lang {
            crate::domain::Language::English => self.name(),
            crate::domain::Language::Nepali => self.nepali_name(),
        }
    }
}

impl fmt::Display for Paksha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Tithi - lunar day in Hindu calendar
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tithi {
    // Shukla Paksha (Waxing Moon) - Days 1-15
    ShuklaPratipada,   // 1st day
    ShuklaDwitiya,     // 2nd day
    ShuklaTritiya,     // 3rd day
    ShuklaChaturthi,   // 4th day
    ShuklaPanchami,    // 5th day
    ShuklaShashti,     // 6th day
    ShuklaSaptami,     // 7th day
    ShuklaAshtami,     // 8th day
    ShuklaNavami,      // 9th day
    ShuklaDashami,     // 10th day
    ShuklaEkadashi,    // 11th day
    ShuklaDwadashi,    // 12th day
    ShuklaTrayodashi,  // 13th day
    ShuklaChaturdashi, // 14th day
    Purnima,           // Full moon (15th day)

    // Krishna Paksha (Waning Moon) - Days 1-15
    KrishnaPratipada,   // 1st day
    KrishnaDwitiya,     // 2nd day
    KrishnaTritiya,     // 3rd day
    KrishnaChaturthi,   // 4th day
    KrishnaPanchami,    // 5th day
    KrishnaShashti,     // 6th day
    KrishnaSaptami,     // 7th day
    KrishnaAshtami,     // 8th day
    KrishnaNavami,      // 9th day
    KrishnaDashami,     // 10th day
    KrishnaEkadashi,    // 11th day
    KrishnaDwadashi,    // 12th day
    KrishnaTrayodashi,  // 13th day
    KrishnaChaturdashi, // 14th day
    Amavasya,           // New moon (15th day)
}

impl Tithi {
    /// Get the paksha (fortnight) this tithi belongs to
    pub fn paksha(&self) -> Paksha {
        match self {
            Tithi::ShuklaPratipada
            | Tithi::ShuklaDwitiya
            | Tithi::ShuklaTritiya
            | Tithi::ShuklaChaturthi
            | Tithi::ShuklaPanchami
            | Tithi::ShuklaShashti
            | Tithi::ShuklaSaptami
            | Tithi::ShuklaAshtami
            | Tithi::ShuklaNavami
            | Tithi::ShuklaDashami
            | Tithi::ShuklaEkadashi
            | Tithi::ShuklaDwadashi
            | Tithi::ShuklaTrayodashi
            | Tithi::ShuklaChaturdashi
            | Tithi::Purnima => Paksha::Shukla,

            Tithi::KrishnaPratipada
            | Tithi::KrishnaDwitiya
            | Tithi::KrishnaTritiya
            | Tithi::KrishnaChaturthi
            | Tithi::KrishnaPanchami
            | Tithi::KrishnaShashti
            | Tithi::KrishnaSaptami
            | Tithi::KrishnaAshtami
            | Tithi::KrishnaNavami
            | Tithi::KrishnaDashami
            | Tithi::KrishnaEkadashi
            | Tithi::KrishnaDwadashi
            | Tithi::KrishnaTrayodashi
            | Tithi::KrishnaChaturdashi
            | Tithi::Amavasya => Paksha::Krishna,
        }
    }

    /// Get the day number within the paksha (1-15)
    pub fn day_in_paksha(&self) -> u8 {
        match self {
            Tithi::ShuklaPratipada | Tithi::KrishnaPratipada => 1,
            Tithi::ShuklaDwitiya | Tithi::KrishnaDwitiya => 2,
            Tithi::ShuklaTritiya | Tithi::KrishnaTritiya => 3,
            Tithi::ShuklaChaturthi | Tithi::KrishnaChaturthi => 4,
            Tithi::ShuklaPanchami | Tithi::KrishnaPanchami => 5,
            Tithi::ShuklaShashti | Tithi::KrishnaShashti => 6,
            Tithi::ShuklaSaptami | Tithi::KrishnaSaptami => 7,
            Tithi::ShuklaAshtami | Tithi::KrishnaAshtami => 8,
            Tithi::ShuklaNavami | Tithi::KrishnaNavami => 9,
            Tithi::ShuklaDashami | Tithi::KrishnaDashami => 10,
            Tithi::ShuklaEkadashi | Tithi::KrishnaEkadashi => 11,
            Tithi::ShuklaDwadashi | Tithi::KrishnaDwadashi => 12,
            Tithi::ShuklaTrayodashi | Tithi::KrishnaTrayodashi => 13,
            Tithi::ShuklaChaturdashi | Tithi::KrishnaChaturdashi => 14,
            Tithi::Purnima | Tithi::Amavasya => 15,
        }
    }

    /// Get the name of the tithi
    pub fn name(&self) -> &'static str {
        match self {
            Tithi::ShuklaPratipada => "Shukla Pratipada",
            Tithi::ShuklaDwitiya => "Shukla Dwitiya",
            Tithi::ShuklaTritiya => "Shukla Tritiya",
            Tithi::ShuklaChaturthi => "Shukla Chaturthi",
            Tithi::ShuklaPanchami => "Shukla Panchami",
            Tithi::ShuklaShashti => "Shukla Shashti",
            Tithi::ShuklaSaptami => "Shukla Saptami",
            Tithi::ShuklaAshtami => "Shukla Ashtami",
            Tithi::ShuklaNavami => "Shukla Navami",
            Tithi::ShuklaDashami => "Shukla Dashami",
            Tithi::ShuklaEkadashi => "Shukla Ekadashi",
            Tithi::ShuklaDwadashi => "Shukla Dwadashi",
            Tithi::ShuklaTrayodashi => "Shukla Trayodashi",
            Tithi::ShuklaChaturdashi => "Shukla Chaturdashi",
            Tithi::Purnima => "Purnima",
            Tithi::KrishnaPratipada => "Krishna Pratipada",
            Tithi::KrishnaDwitiya => "Krishna Dwitiya",
            Tithi::KrishnaTritiya => "Krishna Tritiya",
            Tithi::KrishnaChaturthi => "Krishna Chaturthi",
            Tithi::KrishnaPanchami => "Krishna Panchami",
            Tithi::KrishnaShashti => "Krishna Shashti",
            Tithi::KrishnaSaptami => "Krishna Saptami",
            Tithi::KrishnaAshtami => "Krishna Ashtami",
            Tithi::KrishnaNavami => "Krishna Navami",
            Tithi::KrishnaDashami => "Krishna Dashami",
            Tithi::KrishnaEkadashi => "Krishna Ekadashi",
            Tithi::KrishnaDwadashi => "Krishna Dwadashi",
            Tithi::KrishnaTrayodashi => "Krishna Trayodashi",
            Tithi::KrishnaChaturdashi => "Krishna Chaturdashi",
            Tithi::Amavasya => "Amavasya",
        }
    }

    pub fn nepali_name(&self) -> &'static str {
        match self {
            Tithi::ShuklaPratipada => "शुक्ल प्रतिपदा",
            Tithi::ShuklaDwitiya => "शुक्ल द्वितीया",
            Tithi::ShuklaTritiya => "शुक्ल तृतीया",
            Tithi::ShuklaChaturthi => "शुक्ल चतुर्थी",
            Tithi::ShuklaPanchami => "शुक्ल पञ्चमी",
            Tithi::ShuklaShashti => "शुक्ल षष्ठी",
            Tithi::ShuklaSaptami => "शुक्ल सप्तमी",
            Tithi::ShuklaAshtami => "शुक्ल अष्टमी",
            Tithi::ShuklaNavami => "शुक्ल नवमी",
            Tithi::ShuklaDashami => "शुक्ल दशमी",
            Tithi::ShuklaEkadashi => "शुक्ल एकादशी",
            Tithi::ShuklaDwadashi => "शुक्ल द्वादशी",
            Tithi::ShuklaTrayodashi => "शुक्ल त्रयोदशी",
            Tithi::ShuklaChaturdashi => "शुक्ल चतुर्दशी",
            Tithi::Purnima => "पुर्णिमा",
            Tithi::KrishnaPratipada => "कृष्ण प्रतिपदा",
            Tithi::KrishnaDwitiya => "कृष्ण द्वितीया",
            Tithi::KrishnaTritiya => "कृष्ण तृतीया",
            Tithi::KrishnaChaturthi => "कृष्ण चतुर्थी",
            Tithi::KrishnaPanchami => "कृष्ण पञ्चमी",
            Tithi::KrishnaShashti => "कृष्ण षष्ठी",
            Tithi::KrishnaSaptami => "कृष्ण सप्तमी",
            Tithi::KrishnaAshtami => "कृष्ण अष्टमी",
            Tithi::KrishnaNavami => "कृष्ण नवमी",
            Tithi::KrishnaDashami => "कृष्ण दशमी",
            Tithi::KrishnaEkadashi => "कृष्ण एकादशी",
            Tithi::KrishnaDwadashi => "कृष्ण द्वादशी",
            Tithi::KrishnaTrayodashi => "कृष्ण त्रयोदशी",
            Tithi::KrishnaChaturdashi => "कृष्ण चतुर्दशी",
            Tithi::Amavasya => "औंसी",
        }
    }

    pub fn name_with_language(&self, lang: crate::domain::Language) -> &'static str {
        match lang {
            crate::domain::Language::English => self.name(),
            crate::domain::Language::Nepali => self.nepali_name(),
        }
    }

    /// Check if this is an Ekadashi (11th day)
    pub fn is_ekadashi(&self) -> bool {
        matches!(self, Tithi::ShuklaEkadashi | Tithi::KrishnaEkadashi)
    }

    /// Check if this is Purnima (full moon)
    pub fn is_purnima(&self) -> bool {
        matches!(self, Tithi::Purnima)
    }

    /// Check if this is Amavasya (new moon)
    pub fn is_amavasya(&self) -> bool {
        matches!(self, Tithi::Amavasya)
    }

    /// Get the tithi index from 1-30
    /// Shukla 1-15 -> 1-15
    /// Krishna 1-15 -> 16-30
    pub fn index_1_to_30(&self) -> u8 {
        match self.paksha() {
            Paksha::Shukla => self.day_in_paksha(),
            Paksha::Krishna => 15 + self.day_in_paksha(),
        }
    }

    /// Create tithi from paksha and day number
    pub fn from_paksha_day(paksha: Paksha, day: u8) -> Result<Self> {
        if day < 1 || day > 15 {
            return Err(BsCalendarError::AstronomicalError(format!(
                "Invalid tithi day: {} (must be 1-15)",
                day
            )));
        }

        Ok(match (paksha, day) {
            (Paksha::Shukla, 1) => Tithi::ShuklaPratipada,
            (Paksha::Shukla, 2) => Tithi::ShuklaDwitiya,
            (Paksha::Shukla, 3) => Tithi::ShuklaTritiya,
            (Paksha::Shukla, 4) => Tithi::ShuklaChaturthi,
            (Paksha::Shukla, 5) => Tithi::ShuklaPanchami,
            (Paksha::Shukla, 6) => Tithi::ShuklaShashti,
            (Paksha::Shukla, 7) => Tithi::ShuklaSaptami,
            (Paksha::Shukla, 8) => Tithi::ShuklaAshtami,
            (Paksha::Shukla, 9) => Tithi::ShuklaNavami,
            (Paksha::Shukla, 10) => Tithi::ShuklaDashami,
            (Paksha::Shukla, 11) => Tithi::ShuklaEkadashi,
            (Paksha::Shukla, 12) => Tithi::ShuklaDwadashi,
            (Paksha::Shukla, 13) => Tithi::ShuklaTrayodashi,
            (Paksha::Shukla, 14) => Tithi::ShuklaChaturdashi,
            (Paksha::Shukla, 15) => Tithi::Purnima,
            (Paksha::Krishna, 1) => Tithi::KrishnaPratipada,
            (Paksha::Krishna, 2) => Tithi::KrishnaDwitiya,
            (Paksha::Krishna, 3) => Tithi::KrishnaTritiya,
            (Paksha::Krishna, 4) => Tithi::KrishnaChaturthi,
            (Paksha::Krishna, 5) => Tithi::KrishnaPanchami,
            (Paksha::Krishna, 6) => Tithi::KrishnaShashti,
            (Paksha::Krishna, 7) => Tithi::KrishnaSaptami,
            (Paksha::Krishna, 8) => Tithi::KrishnaAshtami,
            (Paksha::Krishna, 9) => Tithi::KrishnaNavami,
            (Paksha::Krishna, 10) => Tithi::KrishnaDashami,
            (Paksha::Krishna, 11) => Tithi::KrishnaEkadashi,
            (Paksha::Krishna, 12) => Tithi::KrishnaDwadashi,
            (Paksha::Krishna, 13) => Tithi::KrishnaTrayodashi,
            (Paksha::Krishna, 14) => Tithi::KrishnaChaturdashi,
            (Paksha::Krishna, 15) => Tithi::Amavasya,
            _ => unreachable!(),
        })
    }

    /// Parse tithi from name string (case-insensitive)
    /// Supports both full names ("Shukla Ekadashi") and short forms ("EKADASHI", "PURNIMA")
    pub fn from_name(name: &str) -> Option<Self> {
        let name_upper = name.to_uppercase();
        let name_trimmed = name_upper.trim();

        // Special cases for unique tithis
        if name_trimmed.contains("PURNIMA") || name_trimmed == "PURNIMA" {
            return Some(Tithi::Purnima);
        }
        if name_trimmed.contains("AMAVASYA") || name_trimmed == "AMAVASYA" {
            return Some(Tithi::Amavasya);
        }

        // Handle short forms (e.g., "EKADASHI" matches both Shukla and Krishna)
        // Return Shukla variant by default for ambiguous cases
        if name_trimmed == "EKADASHI" {
            return Some(Tithi::ShuklaEkadashi);
        }

        // Full name matching
        match name_trimmed {
            "SHUKLA PRATIPADA" | "SHUKLAPRATIPADA" => Some(Tithi::ShuklaPratipada),
            "SHUKLA DWITIYA" | "SHUKLADWITIYA" => Some(Tithi::ShuklaDwitiya),
            "SHUKLA TRITIYA" | "SHUKLATRITIYA" => Some(Tithi::ShuklaTritiya),
            "SHUKLA CHATURTHI" | "SHUKLACHATURTHI" => Some(Tithi::ShuklaChaturthi),
            "SHUKLA PANCHAMI" | "SHUKLAPANCHAMI" => Some(Tithi::ShuklaPanchami),
            "SHUKLA SHASHTI" | "SHUKLASHASHTI" => Some(Tithi::ShuklaShashti),
            "SHUKLA SAPTAMI" | "SHUKLASAPTAMI" => Some(Tithi::ShuklaSaptami),
            "SHUKLA ASHTAMI" | "SHUKLAASHTAMI" => Some(Tithi::ShuklaAshtami),
            "SHUKLA NAVAMI" | "SHUKLANAVAMI" => Some(Tithi::ShuklaNavami),
            "SHUKLA DASHAMI" | "SHUKLADASHAMI" => Some(Tithi::ShuklaDashami),
            "SHUKLA EKADASHI" | "SHUKLAEKADASHI" => Some(Tithi::ShuklaEkadashi),
            "SHUKLA DWADASHI" | "SHUKLADWADASHI" => Some(Tithi::ShuklaDwadashi),
            "SHUKLA TRAYODASHI" | "SHUKLATRAYODASHI" => Some(Tithi::ShuklaTrayodashi),
            "SHUKLA CHATURDASHI" | "SHUKLACHATURDASHI" => Some(Tithi::ShuklaChaturdashi),

            "KRISHNA PRATIPADA" | "KRISHNAPRATIPADA" => Some(Tithi::KrishnaPratipada),
            "KRISHNA DWITIYA" | "KRISHNADWITIYA" => Some(Tithi::KrishnaDwitiya),
            "KRISHNA TRITIYA" | "KRISHNATRITIYA" => Some(Tithi::KrishnaTritiya),
            "KRISHNA CHATURTHI" | "KRISHNACHATURTHI" => Some(Tithi::KrishnaChaturthi),
            "KRISHNA PANCHAMI" | "KRISHNAPANCHAMI" => Some(Tithi::KrishnaPanchami),
            "KRISHNA SHASHTI" | "KRISHNASHASHTI" => Some(Tithi::KrishnaShashti),
            "KRISHNA SAPTAMI" | "KRISHNASAPTAMI" => Some(Tithi::KrishnaSaptami),
            "KRISHNA ASHTAMI" | "KRISHNAASHTAMI" => Some(Tithi::KrishnaAshtami),
            "KRISHNA NAVAMI" | "KRISHNANAVAMI" => Some(Tithi::KrishnaNavami),
            "KRISHNA DASHAMI" | "KRISHNADASHAMI" => Some(Tithi::KrishnaDashami),
            "KRISHNA EKADASHI" | "KRISHNAEKADASHI" => Some(Tithi::KrishnaEkadashi),
            "KRISHNA DWADASHI" | "KRISHNADWADASHI" => Some(Tithi::KrishnaDwadashi),
            "KRISHNA TRAYODASHI" | "KRISHNATRAYODASHI" => Some(Tithi::KrishnaTrayodashi),
            "KRISHNA CHATURDASHI" | "KRISHNACHATURDASHI" => Some(Tithi::KrishnaChaturdashi),

            _ => None,
        }
    }
}

impl fmt::Display for Tithi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(getter_with_clone)
)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    #[cfg_attr(feature = "wasm", wasm_bindgen(skip))]
    pub name: &'static str,
    pub timezone_offset_mins: i32,
    pub follow_nepal_social_calendar: bool,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl Location {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(
        latitude: f64,
        longitude: f64,
        name: String,
        timezone_offset_mins: i32,
    ) -> Self {
        Location {
            latitude,
            longitude,
            name: Box::leak(name.into_boxed_str()), // Safe enough for this app
            timezone_offset_mins,
            follow_nepal_social_calendar: false,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

impl Location {
    /// Kathmandu, Nepal (default location)
    pub const KATHMANDU: Location = Location {
        latitude: 27.7172,
        longitude: 85.3240,
        name: "Kathmandu",
        timezone_offset_mins: 345, // +5:45
        follow_nepal_social_calendar: true,
    };

    /// New York, USA
    pub const NEW_YORK: Location = Location {
        latitude: 40.7128,
        longitude: -74.0060,
        name: "New York",
        timezone_offset_mins: -300, // -5:00 (EST)
        follow_nepal_social_calendar: false,
    };

    /// London, UK
    pub const LONDON: Location = Location {
        latitude: 51.5074,
        longitude: -0.1278,
        name: "London",
        timezone_offset_mins: 0, // GMT
        follow_nepal_social_calendar: false,
    };

    /// Create a new location
    pub fn new(
        latitude: f64,
        longitude: f64,
        name: &'static str,
        timezone_offset_mins: i32,
    ) -> Self {
        Location {
            latitude,
            longitude,
            name,
            timezone_offset_mins,
            follow_nepal_social_calendar: false, // Default to false for custom locations
        }
    }

    /// Create a new location with explicit social calendar preference
    pub fn with_social_calendar(
        latitude: f64,
        longitude: f64,
        name: &'static str,
        timezone_offset_mins: i32,
        follow_nepal: bool,
    ) -> Self {
        Location {
            latitude,
            longitude,
            name,
            timezone_offset_mins,
            follow_nepal_social_calendar: follow_nepal,
        }
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::KATHMANDU
    }
}
