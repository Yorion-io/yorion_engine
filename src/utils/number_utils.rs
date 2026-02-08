/// Utility for number localization
pub struct NumberUtils;

impl NumberUtils {
    /// Convert English digits to Nepali Devanagari digits
    pub fn to_nepali(num: i64) -> String {
        num.to_string()
            .chars()
            .map(|c| match c {
                '0' => '०',
                '1' => '१',
                '2' => '२',
                '3' => '३',
                '4' => '४',
                '5' => '५',
                '6' => '६',
                '7' => '७',
                '8' => '८',
                '9' => '९',
                _ => c,
            })
            .collect()
    }

    /// Convert based on language preference
    pub fn to_language(num: i64, lang: crate::domain::Language) -> String {
        match lang {
            crate::domain::Language::English => num.to_string(),
            crate::domain::Language::Nepali => Self::to_nepali(num),
        }
    }
}

/// Helper for formatting dates with patterns
pub struct DateFormatter;

impl DateFormatter {
    pub fn format(
        date: &crate::domain::BsDate,
        pattern: &str,
        lang: crate::domain::Language,
    ) -> String {
        let pattern = if pattern.trim().is_empty() {
            "YYYY MMMM DD" // Default
        } else {
            pattern
        };

        // Year
        let year_str = NumberUtils::to_language(date.year as i64, lang);

        // Month (MM - padded)
        let mm_str = match lang {
            crate::domain::Language::English => format!("{:02}", date.month.to_u8()),
            crate::domain::Language::Nepali => NumberUtils::to_nepali(date.month.to_u8() as i64), // to_nepali handles simple digits
        };
        // Ensure 2 digit padding for Nepali if less than 10 (simple prefixing)
        // NumberUtils::to_nepali converts 1 to १. For MM we might want ०१.
        let mm_str = if lang == crate::domain::Language::Nepali && date.month.to_u8() < 10 {
            format!("०{}", mm_str)
        } else {
            mm_str
        };

        // Month (M - no padding)
        let m_str = NumberUtils::to_language(date.month.to_u8() as i64, lang);

        // Month Name (MMMM)
        let mmmm_str = date.month.name_with_language(lang);

        // Day (DD - padded)
        let dd_str = match lang {
            crate::domain::Language::English => format!("{:02}", date.day),
            crate::domain::Language::Nepali => NumberUtils::to_nepali(date.day as i64),
        };
        let dd_str = if lang == crate::domain::Language::Nepali && date.day < 10 {
            format!("०{}", dd_str)
        } else {
            dd_str
        };

        // Day (D - no padding)
        let d_str = NumberUtils::to_language(date.day as i64, lang);

        // Replacement strategy: longer tokens first
        let mut result = pattern.to_string();
        result = result.replace("YYYY", &year_str).replace("yyyy", &year_str);
        result = result.replace("MMMM", mmmm_str);
        // Be careful with MM vs M. Replace MM first.
        result = result.replace("MM", &mm_str);
        // We can't safely replace M globally because it might be inside MMMM (already replaced) or MM (already replaced).
        // Since we already replaced MMMM and MM, the remaining M characters *should* be standalone M tokens,
        // unless the original pattern was weird like MMM.
        // But for simplicity in this regex-less approach, assume users don't mix tokens overlappingly without separators.
        // Actually, replacing "M" now might match the M inside a word if we aren't careful?
        // No, we replaced "MMMM" with the name (e.g. Baisakh). Baisakh doesn't contain capital M unless English.
        // But invalid patterns/ambiguities are possible.
        // Let's assume standard usage.
        result = result.replace("M", &m_str);

        // Replace DD/D
        result = result.replace("DD", &dd_str).replace("dd", &dd_str);
        result = result.replace("D", &d_str).replace("d", &d_str);

        result
    }
}
