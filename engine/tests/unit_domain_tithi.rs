// Domain Tithi Tests
use yorion_engine::domain::tithi::{Location, Paksha, Tithi};

mod helpers;

#[test]
fn test_tithi_paksha() {
    assert_eq!(Tithi::ShuklaEkadashi.paksha(), Paksha::Shukla);
    assert_eq!(Tithi::KrishnaEkadashi.paksha(), Paksha::Krishna);
    assert_eq!(Tithi::Purnima.paksha(), Paksha::Shukla);
    assert_eq!(Tithi::Amavasya.paksha(), Paksha::Krishna);
    // All Shukla variants
    assert_eq!(Tithi::ShuklaPratipada.paksha(), Paksha::Shukla);
    assert_eq!(Tithi::ShuklaChaturdashi.paksha(), Paksha::Shukla);
    // All Krishna variants
    assert_eq!(Tithi::KrishnaPratipada.paksha(), Paksha::Krishna);
    assert_eq!(Tithi::KrishnaChaturdashi.paksha(), Paksha::Krishna);
}

#[test]
fn test_tithi_day_in_paksha() {
    assert_eq!(Tithi::ShuklaEkadashi.day_in_paksha(), 11);
    assert_eq!(Tithi::KrishnaEkadashi.day_in_paksha(), 11);
    assert_eq!(Tithi::Purnima.day_in_paksha(), 15);
    assert_eq!(Tithi::Amavasya.day_in_paksha(), 15);
    assert_eq!(Tithi::ShuklaPratipada.day_in_paksha(), 1);
    assert_eq!(Tithi::KrishnaPratipada.day_in_paksha(), 1);
    assert_eq!(Tithi::ShuklaChaturdashi.day_in_paksha(), 14);
    assert_eq!(Tithi::KrishnaChaturdashi.day_in_paksha(), 14);
}

#[test]
fn test_tithi_special_days() {
    assert!(Tithi::ShuklaEkadashi.is_ekadashi());
    assert!(Tithi::KrishnaEkadashi.is_ekadashi());
    assert!(!Tithi::Purnima.is_ekadashi());
    assert!(!Tithi::Amavasya.is_ekadashi());

    assert!(Tithi::Purnima.is_purnima());
    assert!(!Tithi::Amavasya.is_purnima());
    assert!(!Tithi::ShuklaEkadashi.is_purnima());

    assert!(Tithi::Amavasya.is_amavasya());
    assert!(!Tithi::Purnima.is_amavasya());
    assert!(!Tithi::KrishnaEkadashi.is_amavasya());
}

#[test]
fn test_tithi_index_1_to_30() {
    // Shukla range: 1–15
    assert_eq!(Tithi::ShuklaPratipada.index_1_to_30(), 1);
    assert_eq!(Tithi::ShuklaEkadashi.index_1_to_30(), 11);
    assert_eq!(Tithi::Purnima.index_1_to_30(), 15);
    // Krishna range: 16–30
    assert_eq!(Tithi::KrishnaPratipada.index_1_to_30(), 16);
    assert_eq!(Tithi::KrishnaEkadashi.index_1_to_30(), 26);
    assert_eq!(Tithi::Amavasya.index_1_to_30(), 30);
}

#[test]
fn test_tithi_from_paksha_day() {
    let tithi = Tithi::from_paksha_day(Paksha::Shukla, 11).unwrap();
    assert_eq!(tithi, Tithi::ShuklaEkadashi);

    let tithi = Tithi::from_paksha_day(Paksha::Krishna, 15).unwrap();
    assert_eq!(tithi, Tithi::Amavasya);

    let tithi = Tithi::from_paksha_day(Paksha::Shukla, 15).unwrap();
    assert_eq!(tithi, Tithi::Purnima);

    let tithi = Tithi::from_paksha_day(Paksha::Krishna, 1).unwrap();
    assert_eq!(tithi, Tithi::KrishnaPratipada);

    assert!(Tithi::from_paksha_day(Paksha::Shukla, 0).is_err());
    assert!(Tithi::from_paksha_day(Paksha::Shukla, 16).is_err());
}

#[test]
fn test_tithi_from_name_full() {
    // All 30 tithis must round-trip through from_name / name().
    let all: &[Tithi] = &[
        Tithi::ShuklaPratipada, Tithi::ShuklaDwitiya, Tithi::ShuklaTritiya,
        Tithi::ShuklaChaturthi, Tithi::ShuklaPanchami, Tithi::ShuklaShashti,
        Tithi::ShuklaSaptami, Tithi::ShuklaAshtami, Tithi::ShuklaNavami,
        Tithi::ShuklaDashami, Tithi::ShuklaEkadashi, Tithi::ShuklaDwadashi,
        Tithi::ShuklaTrayodashi, Tithi::ShuklaChaturdashi, Tithi::Purnima,
        Tithi::KrishnaPratipada, Tithi::KrishnaDwitiya, Tithi::KrishnaTritiya,
        Tithi::KrishnaChaturthi, Tithi::KrishnaPanchami, Tithi::KrishnaShashti,
        Tithi::KrishnaSaptami, Tithi::KrishnaAshtami, Tithi::KrishnaNavami,
        Tithi::KrishnaDashami, Tithi::KrishnaEkadashi, Tithi::KrishnaDwadashi,
        Tithi::KrishnaTrayodashi, Tithi::KrishnaChaturdashi, Tithi::Amavasya,
    ];
    for t in all {
        let parsed = Tithi::from_name(t.name());
        assert!(
            parsed.is_some(),
            "from_name({:?}) returned None",
            t.name()
        );
        assert_eq!(
            parsed.unwrap(),
            *t,
            "from_name round-trip failed for {:?}",
            t
        );
    }
}

#[test]
fn test_tithi_from_name_bare_forms() {
    // Bare forms (no paksha prefix) map to the Shukla variant by convention.
    assert_eq!(Tithi::from_name("EKADASHI"), Some(Tithi::ShuklaEkadashi));
    assert_eq!(Tithi::from_name("PURNIMA"), Some(Tithi::Purnima));
    assert_eq!(Tithi::from_name("AMAVASYA"), Some(Tithi::Amavasya));
    assert_eq!(Tithi::from_name("ASHTAMI"), Some(Tithi::ShuklaAshtami));
    // Unknown name returns None
    assert_eq!(Tithi::from_name("GARBAGE"), None);
}

#[test]
fn test_location() {
    let kathmandu = Location::default();
    assert_eq!(kathmandu.name, "Kathmandu");
    assert!((kathmandu.latitude - 27.7172).abs() < 0.001);
    assert!((kathmandu.longitude - 85.3240).abs() < 0.001);
}

#[test]
fn test_location_custom() {
    let loc = Location::new(40.7128, -74.0060, "New York", -300);
    assert_eq!(loc.name, "New York");
    assert!((loc.latitude - 40.7128).abs() < 0.001);
    assert_eq!(loc.timezone_offset_mins, -300);
    assert!(!loc.follow_nepal_social_calendar);
}
