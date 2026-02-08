use bs_calendar_core::prelude::*;
use chrono::NaiveDate;
use std::sync::Arc;

fn get_astro_service() -> AstronomicalService {
    let override_provider = Box::new(StaticTithiOverrideProvider::new());
    AstronomicalService::with_overrides(override_provider)
}

#[test]
fn test_sun_zodiac_mesh_sankranti() {
    let service = get_astro_service();
    let gregorian = NaiveDate::from_ymd_opt(2024, 4, 14).unwrap();
    let jd = service.get_julian_day(gregorian.and_hms_opt(12, 0, 0).unwrap().and_utc());
    let sign = service.get_sun_zodiac_sign(jd);
    assert_eq!(sign.english_name(), "Aries");
}

#[test]
fn test_today_zodiac_nakshatra() {
    let service = get_astro_service();
    let gregorian = NaiveDate::from_ymd_opt(2026, 1, 30).unwrap();
    let jd = service.get_julian_day(gregorian.and_hms_opt(12, 0, 0).unwrap().and_utc());
    
    let sun = service.get_sun_zodiac_sign(jd);
    let moon = service.get_moon_zodiac_sign(jd);
    let nakshatra = service.get_nakshatra(jd);
    
    println!("Sun: {}, Moon: {}, Nakshatra: {}", sun, moon, nakshatra);
    
    assert_eq!(sun.english_name(), "Capricorn");
}

#[test]
fn test_user_birth_details() {
    let service = get_astro_service();
    // 1998-11-11 14:00 (2 PM) Nepal Time
    // Temal, Kavre is roughly 27.57 N, 85.74 E
    let location = Location::new(27.57, 85.74, "Temal", 345);
    
    let gregorian = NaiveDate::from_ymd_opt(1998, 11, 11).unwrap();
    let time = chrono::NaiveTime::from_hms_opt(14, 0, 0).unwrap();
    let dt = gregorian.and_time(time).and_local_timezone(chrono::FixedOffset::east_opt(345 * 60).unwrap()).unwrap().with_timezone(&chrono::Utc);
    
    let info = service.get_daily_astro_info(dt, &location).unwrap();
    
    println!("--- User Birth Details ---");
    println!("Date: 1998-11-11 (BS: 2055-07-25), Time: 2:00 PM");
    println!("Location: Temal, Kavre");
    println!("Tithi: {}", info.tithi);
    println!("Sun Sign: {} ({})", info.sun_sign.english_name(), info.sun_sign.nepali_name());
    println!("Moon Sign: {} ({})", info.moon_sign.english_name(), info.moon_sign.nepali_name());
    println!("Nakshatra: {} ({})", info.nakshatra.name(), info.nakshatra.nepali_name());
    println!("--------------------------");
}

#[test]
fn test_tithi_transition_investigation() {
    let service = get_astro_service();
    let location = Location::new(27.57, 85.74, "Temal", 345);
    let gregorian = NaiveDate::from_ymd_opt(1998, 11, 11).unwrap();
    
    println!("--- Tithi Transition Check for 1998-11-11 ---");
    for hour in [0, 6, 12, 14, 18, 23] {
        let time = chrono::NaiveTime::from_hms_opt(hour, 0, 0).unwrap();
        let dt = gregorian.and_time(time).and_local_timezone(chrono::FixedOffset::east_opt(345 * 60).unwrap()).unwrap().with_timezone(&chrono::Utc);
        let tithi = service.calculate_tithi_with_location(dt, &location).unwrap();
        let (sun, moon) = service.calculate_longitudes_from_jd(service.get_julian_day(dt));
        println!("{}:00 - {} (Sun: {:.2}, Moon: {:.2}, Diff: {:.2})", hour, tithi, sun, moon, (moon-sun+360.0)%360.0);
    }
}
