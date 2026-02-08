// Tithi Recurrence Tests
// Verifying Tithi recurrence behavior covering standard festivals and complex rules
use bs_calendar_core::domain::bs_date::{BsDate, BsMonth};
use bs_calendar_core::domain::recurrence::TithiRecurrenceRule;
use bs_calendar_core::domain::tithi::{Paksha, Tithi};
use bs_calendar_core::services::{AstronomicalService, ConversionService, InstanceGenerator};
use std::sync::Arc;

mod helpers;
use helpers::TestCalendarProvider;

fn create_services() -> (InstanceGenerator, AstronomicalService) {
    let provider = Arc::new(TestCalendarProvider::new());
    let conversion = ConversionService::new(provider.clone());
    let generator = InstanceGenerator::new(Arc::new(conversion));
    let astro = AstronomicalService::new();
    (generator, astro)
}

#[test]
fn test_recurrence_ekadashi() {
    let (generator, astro) = create_services();

    // Every Ekadashi (11th tithi of BOTH pakshas)
    // 2080 Baisakh 1 is Krishna Navami (9) (approx)
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor).with_count(4);

    // Search wide enough to find 4 occurrences (approx 2 months)
    let end = BsDate::new(2080, 3, 1).unwrap();

    let instances = generator
        .generate_tithi_instances(&rule, anchor, end, &astro)
        .unwrap();

    assert_eq!(instances.len(), 4);

    // Verify each instance is indeed an Ekadashi
    for date in instances {
        // Easier way: just calculate tithi for that date
        let provider = TestCalendarProvider::new();
        let conv = ConversionService::new(Arc::new(provider));
        let greg = conv.bs_to_gregorian(date).unwrap();
        let tithi = astro
            .calculate_tithi_for_date(greg, bs_calendar_core::domain::Location::KATHMANDU)
            .unwrap();

        assert!(
            tithi.is_ekadashi(),
            "Date {} should be Ekadashi but is {:?}",
            date,
            tithi
        );
    }
}

#[test]
fn test_recurrence_purnima() {
    let (generator, astro) = create_services();

    // Every Purnima (Full Moon)
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::purnima(anchor).with_count(2);

    let end = BsDate::new(2080, 3, 1).unwrap(); // 2 months

    let instances = generator
        .generate_tithi_instances(&rule, anchor, end, &astro)
        .unwrap();

    assert_eq!(instances.len(), 2);

    // Verify
    let provider = TestCalendarProvider::new();
    let conv = ConversionService::new(Arc::new(provider));

    for date in instances {
        let greg = conv.bs_to_gregorian(date).unwrap();
        let tithi = astro
            .calculate_tithi_for_date(greg, bs_calendar_core::domain::Location::KATHMANDU)
            .unwrap();
        assert!(
            tithi.is_purnima(),
            "Date {} should be Purnima but is {:?}",
            date,
            tithi
        );
    }
}

#[test]
fn test_recurrence_krishna_ashtami() {
    let (generator, astro) = create_services();

    // Every Krishna Paksha Ashtami (Krishna 8) - e.g. Janai Purnima is different but Krishna Janmashtami is Krishna 8
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule =
        TithiRecurrenceRule::with_paksha(vec![Tithi::KrishnaAshtami], Paksha::Krishna, anchor)
            .with_count(2);

    let end = BsDate::new(2080, 4, 1).unwrap(); // 3 months

    let instances = generator
        .generate_tithi_instances(&rule, anchor, end, &astro)
        .unwrap();

    // Should occur once per lunar month (~29.5 days)
    assert_eq!(instances.len(), 2);

    let provider = TestCalendarProvider::new();
    let conv = ConversionService::new(Arc::new(provider));

    for date in instances {
        let greg = conv.bs_to_gregorian(date).unwrap();
        let tithi = astro
            .calculate_tithi_for_date(greg, bs_calendar_core::domain::Location::KATHMANDU)
            .unwrap();
        assert_eq!(
            tithi,
            Tithi::KrishnaAshtami,
            "Date {} should be Krishna Ashtami",
            date
        );
    }
}

#[test]
fn test_recurrence_multiple_tithis() {
    let (generator, astro) = create_services();

    // Panchami (5) and Dashami (10) of Shukla Paksha
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaPanchami, Tithi::ShuklaDashami],
        Paksha::Shukla,
        anchor,
    )
    .with_count(4);

    let end = BsDate::new(2080, 3, 1).unwrap();

    let instances = generator
        .generate_tithi_instances(&rule, anchor, end, &astro)
        .unwrap();

    // Should find: M1-P5, M1-P10, M2-P5, M2-P10
    assert_eq!(instances.len(), 4);

    let provider = TestCalendarProvider::new();
    let conv = ConversionService::new(Arc::new(provider));

    for date in instances {
        let greg = conv.bs_to_gregorian(date).unwrap();
        let tithi = astro
            .calculate_tithi_for_date(greg, bs_calendar_core::domain::Location::KATHMANDU)
            .unwrap();
        assert!(
            tithi == Tithi::ShuklaPanchami || tithi == Tithi::ShuklaDashami,
            "Date {} should be Shukla Panchami or Dashami but is {:?}",
            date,
            tithi
        );
    }
}

#[test]
fn test_tithi_in_specific_bs_month() {
    let (generator, astro) = create_services();

    // Ekadashi only in Baisakh
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = TithiRecurrenceRule::ekadashi(anchor)
        .with_by_month(vec![BsMonth::Baisakh])
        .with_count(2); // Should find 2 in Baisakh (Krishna and Shukla)

    let end = BsDate::new(2080, 12, 30).unwrap(); // Search whole year

    let instances = generator
        .generate_tithi_instances(&rule, anchor, end, &astro)
        .unwrap();

    // Only 2 expected (or maybe 3 if Baisakh spans enough?)
    // Baisakh is ~31 days. Tithis form ~29.5 days.
    // Likely 2 Ekadashis in one solar month.

    assert!(instances.len() >= 2);
    for date in instances {
        assert_eq!(date.month, BsMonth::Baisakh, "Instance MUST be in Baisakh");
    }
}
