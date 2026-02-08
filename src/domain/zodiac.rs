use serde::{Deserialize, Serialize};
use std::fmt;

/// Zodiac Sign (Rashi) in Hindu Astrology
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ZodiacSign {
    Mesh,      // Aries
    Vrishabh,  // Taurus
    Mithun,    // Gemini
    Karka,     // Cancer
    Simha,     // Leo
    Kanya,     // Virgo
    Tula,      // Libra
    Vrishchik, // Scorpio
    Dhanu,     // Sagittarius
    Makar,     // Capricorn
    Kumbha,    // Aquarius
    Meen,      // Pisces
}

impl ZodiacSign {
    /// Get the English name of the zodiac sign
    pub fn english_name(&self) -> &'static str {
        match self {
            ZodiacSign::Mesh => "Aries",
            ZodiacSign::Vrishabh => "Taurus",
            ZodiacSign::Mithun => "Gemini",
            ZodiacSign::Karka => "Cancer",
            ZodiacSign::Simha => "Leo",
            ZodiacSign::Kanya => "Virgo",
            ZodiacSign::Tula => "Libra",
            ZodiacSign::Vrishchik => "Scorpio",
            ZodiacSign::Dhanu => "Sagittarius",
            ZodiacSign::Makar => "Capricorn",
            ZodiacSign::Kumbha => "Aquarius",
            ZodiacSign::Meen => "Pisces",
        }
    }

    /// Get the Nepali (transliterated or Devanagari) name
    pub fn nepali_name(&self) -> &'static str {
        match self {
            ZodiacSign::Mesh => "मेष",
            ZodiacSign::Vrishabh => "वृष",
            ZodiacSign::Mithun => "मिथुन",
            ZodiacSign::Karka => "कर्कट",
            ZodiacSign::Simha => "सिंह",
            ZodiacSign::Kanya => "कन्या",
            ZodiacSign::Tula => "तुला",
            ZodiacSign::Vrishchik => "वृश्चिक",
            ZodiacSign::Dhanu => "धनु",
            ZodiacSign::Makar => "मकर",
            ZodiacSign::Kumbha => "कुम्भ",
            ZodiacSign::Meen => "मीन",
        }
    }

    pub fn name_with_language(&self, lang: crate::domain::Language) -> &'static str {
        match lang {
            crate::domain::Language::English => self.english_name(),
            crate::domain::Language::Nepali => self.nepali_name(),
        }
    }

    /// Create from 1-based index (1=Mesh, 12=Meen)
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            1 => Some(ZodiacSign::Mesh),
            2 => Some(ZodiacSign::Vrishabh),
            3 => Some(ZodiacSign::Mithun),
            4 => Some(ZodiacSign::Karka),
            5 => Some(ZodiacSign::Simha),
            6 => Some(ZodiacSign::Kanya),
            7 => Some(ZodiacSign::Tula),
            8 => Some(ZodiacSign::Vrishchik),
            9 => Some(ZodiacSign::Dhanu),
            10 => Some(ZodiacSign::Makar),
            11 => Some(ZodiacSign::Kumbha),
            12 => Some(ZodiacSign::Meen),
            _ => None,
        }
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.english_name())
    }
}

/// Nakshatra (Lunar Mansion) - 27 divisions of the ecliptic
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Nakshatra {
    Ashwini,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Mula,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishtha,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

impl Nakshatra {
    pub fn name(&self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "Ashwini",
            Nakshatra::Bharani => "Bharani",
            Nakshatra::Krittika => "Krittika",
            Nakshatra::Rohini => "Rohini",
            Nakshatra::Mrigashira => "Mrigashira",
            Nakshatra::Ardra => "Ardra",
            Nakshatra::Punarvasu => "Punarvasu",
            Nakshatra::Pushya => "Pushya",
            Nakshatra::Ashlesha => "Ashlesha",
            Nakshatra::Magha => "Magha",
            Nakshatra::PurvaPhalguni => "Purva Phalguni",
            Nakshatra::UttaraPhalguni => "Uttara Phalguni",
            Nakshatra::Hasta => "Hasta",
            Nakshatra::Chitra => "Chitra",
            Nakshatra::Swati => "Swati",
            Nakshatra::Vishakha => "Vishakha",
            Nakshatra::Anuradha => "Anuradha",
            Nakshatra::Jyeshtha => "Jyeshtha",
            Nakshatra::Mula => "Mula",
            Nakshatra::PurvaAshadha => "Purva Ashadha",
            Nakshatra::UttaraAshadha => "Uttara Ashadha",
            Nakshatra::Shravana => "Shravana",
            Nakshatra::Dhanishtha => "Dhanishtha",
            Nakshatra::Shatabhisha => "Shatabhisha",
            Nakshatra::PurvaBhadrapada => "Purva Bhadrapada",
            Nakshatra::UttaraBhadrapada => "Uttara Bhadrapada",
            Nakshatra::Revati => "Revati",
        }
    }

    pub fn nepali_name(&self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "अश्विनी",
            Nakshatra::Bharani => "भरणी",
            Nakshatra::Krittika => "कृत्तिका",
            Nakshatra::Rohini => "रोहिणी",
            Nakshatra::Mrigashira => "मृगशिरा",
            Nakshatra::Ardra => "आर्द्रा",
            Nakshatra::Punarvasu => "पुनर्वसु",
            Nakshatra::Pushya => "पुष्य",
            Nakshatra::Ashlesha => "आश्लेषा",
            Nakshatra::Magha => "मघा",
            Nakshatra::PurvaPhalguni => "पूर्वाफाल्गुनी",
            Nakshatra::UttaraPhalguni => "उत्तराफाल्गुनी",
            Nakshatra::Hasta => "हस्त",
            Nakshatra::Chitra => "चित्रा",
            Nakshatra::Swati => "स्वाती",
            Nakshatra::Vishakha => "विशाखा",
            Nakshatra::Anuradha => "अनुराधा",
            Nakshatra::Jyeshtha => "ज्येष्ठा",
            Nakshatra::Mula => "मूल",
            Nakshatra::PurvaAshadha => "पूर्वाषाढा",
            Nakshatra::UttaraAshadha => "उत्तराषाढा",
            Nakshatra::Shravana => "श्रवण",
            Nakshatra::Dhanishtha => "धनिष्ठा",
            Nakshatra::Shatabhisha => "शतभिषा",
            Nakshatra::PurvaBhadrapada => "पूर्वाभाद्रपदा",
            Nakshatra::UttaraBhadrapada => "उत्तराभाद्रपदा",
            Nakshatra::Revati => "रेवती",
        }
    }

    pub fn name_with_language(&self, lang: crate::domain::Language) -> &'static str {
        match lang {
            crate::domain::Language::English => self.name(),
            crate::domain::Language::Nepali => self.nepali_name(),
        }
    }

    /// Create from 1-based index (1=Ashwini, 27=Revati)
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            1 => Some(Nakshatra::Ashwini),
            2 => Some(Nakshatra::Bharani),
            3 => Some(Nakshatra::Krittika),
            4 => Some(Nakshatra::Rohini),
            5 => Some(Nakshatra::Mrigashira),
            6 => Some(Nakshatra::Ardra),
            7 => Some(Nakshatra::Punarvasu),
            8 => Some(Nakshatra::Pushya),
            9 => Some(Nakshatra::Ashlesha),
            10 => Some(Nakshatra::Magha),
            11 => Some(Nakshatra::PurvaPhalguni),
            12 => Some(Nakshatra::UttaraPhalguni),
            13 => Some(Nakshatra::Hasta),
            14 => Some(Nakshatra::Chitra),
            15 => Some(Nakshatra::Swati),
            16 => Some(Nakshatra::Vishakha),
            17 => Some(Nakshatra::Anuradha),
            18 => Some(Nakshatra::Jyeshtha),
            19 => Some(Nakshatra::Mula),
            20 => Some(Nakshatra::PurvaAshadha),
            21 => Some(Nakshatra::UttaraAshadha),
            22 => Some(Nakshatra::Shravana),
            23 => Some(Nakshatra::Dhanishtha),
            24 => Some(Nakshatra::Shatabhisha),
            25 => Some(Nakshatra::PurvaBhadrapada),
            26 => Some(Nakshatra::UttaraBhadrapada),
            27 => Some(Nakshatra::Revati),
            _ => None,
        }
    }
}

impl fmt::Display for Nakshatra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Combined astronomical information for a day
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen(getter_with_clone)
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyAstroInfo {
    pub tithi: crate::domain::tithi::Tithi,
    pub sun_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
    pub nakshatra: Nakshatra,
    pub is_overridden: bool,
}
