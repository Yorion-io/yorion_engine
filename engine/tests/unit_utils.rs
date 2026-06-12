use yorion_engine::domain::{BsDate, Language};
use yorion_engine::utils::number_utils::{DateFormatter, NumberUtils};

mod helpers;

#[test]
fn test_to_nepali() {
    assert_eq!(NumberUtils::to_nepali(1234567890), "१२३४५६७८९०");
    assert_eq!(NumberUtils::to_nepali(2080), "२०८०");
}

#[test]
fn test_date_formatter() {
    let date = BsDate::new(2080, 1, 1).unwrap();

    // English
    assert_eq!(
        DateFormatter::format(&date, "YYYY/MM/DD", Language::English),
        "2080/01/01"
    );
    assert_eq!(
        DateFormatter::format(&date, "YYYY MMMM D", Language::English),
        "2080 Baisakh 1"
    );

    // Nepali
    assert_eq!(
        DateFormatter::format(&date, "YYYY.MM.DD", Language::Nepali),
        "२०८०.०१.०१"
    );
    assert_eq!(
        DateFormatter::format(&date, "YYYY MMMM D", Language::Nepali),
        "२०८० वैशाख १"
    );
}

#[test]
fn test_to_language() {
    assert_eq!(NumberUtils::to_language(2080, Language::English), "2080");
    assert_eq!(NumberUtils::to_language(2080, Language::Nepali), "२०८०");
}
