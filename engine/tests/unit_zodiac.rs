use yorion_engine::prelude::*;
use chrono::NaiveDate;

fn astro() -> AstronomicalService {
    AstronomicalService::with_overrides(Box::new(StaticTithiOverrideProvider::new()))
}

#[test]
fn sun_zodiac_mesh_sankranti() {
    let service = astro();
    let jd = service.get_julian_day(
        NaiveDate::from_ymd_opt(2024, 4, 14).unwrap().and_hms_opt(12, 0, 0).unwrap().and_utc(),
    );
    assert_eq!(service.get_sun_zodiac_sign(jd).english_name(), "Aries");
}

#[test]
fn sun_zodiac_capricorn_jan_2026() {
    let service = astro();
    let jd = service.get_julian_day(
        NaiveDate::from_ymd_opt(2026, 1, 30).unwrap().and_hms_opt(12, 0, 0).unwrap().and_utc(),
    );
    assert_eq!(service.get_sun_zodiac_sign(jd).english_name(), "Capricorn");
}
