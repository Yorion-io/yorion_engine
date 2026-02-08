// Domain Tithi Tests
use bs_calendar_core::domain::tithi::{Location, Paksha, Tithi};

mod helpers;

#[test]
fn test_tithi_paksha() {
    assert_eq!(Tithi::ShuklaEkadashi.paksha(), Paksha::Shukla);
    assert_eq!(Tithi::KrishnaEkadashi.paksha(), Paksha::Krishna);
    assert_eq!(Tithi::Purnima.paksha(), Paksha::Shukla);
    assert_eq!(Tithi::Amavasya.paksha(), Paksha::Krishna);
}

#[test]
fn test_tithi_day_in_paksha() {
    assert_eq!(Tithi::ShuklaEkadashi.day_in_paksha(), 11);
    assert_eq!(Tithi::KrishnaEkadashi.day_in_paksha(), 11);
    assert_eq!(Tithi::Purnima.day_in_paksha(), 15);
    assert_eq!(Tithi::Amavasya.day_in_paksha(), 15);
}

#[test]
fn test_tithi_special_days() {
    assert!(Tithi::ShuklaEkadashi.is_ekadashi());
    assert!(Tithi::KrishnaEkadashi.is_ekadashi());
    assert!(!Tithi::Purnima.is_ekadashi());

    assert!(Tithi::Purnima.is_purnima());
    assert!(!Tithi::Amavasya.is_purnima());

    assert!(Tithi::Amavasya.is_amavasya());
    assert!(!Tithi::Purnima.is_amavasya());
}

#[test]
fn test_tithi_from_paksha_day() {
    let tithi = Tithi::from_paksha_day(Paksha::Shukla, 11).unwrap();
    assert_eq!(tithi, Tithi::ShuklaEkadashi);

    let tithi = Tithi::from_paksha_day(Paksha::Krishna, 15).unwrap();
    assert_eq!(tithi, Tithi::Amavasya);

    assert!(Tithi::from_paksha_day(Paksha::Shukla, 0).is_err());
    assert!(Tithi::from_paksha_day(Paksha::Shukla, 16).is_err());
}

#[test]
fn test_location() {
    let kathmandu = Location::default();
    assert_eq!(kathmandu.name, "Kathmandu");
    assert!((kathmandu.latitude - 27.7172).abs() < 0.001);
    assert!((kathmandu.longitude - 85.3240).abs() < 0.001);
}
