use bs_calendar_core::prelude::*;
use chrono::{NaiveDate, NaiveTime};

fn main() {
    println!("=== Zodiac Sign & Horoscope Details Example ===\n");

    // Setup
    let astro = AstronomicalService::new();

    // Case 1: Specific Date of Birth and Time (Kathmandu)
    // Example: 2058 Baisakh 15, 10:30 AM
    // We need to convert BS to AD first usually, but let's assume we have AD DOB for this Example or use Conversion.
    // Let's use a known AD date for simplicity/accuracy of example demonstration.
    // 2001-04-28 (which is approx 2058 Baisakh 15)

    let dob_date = NaiveDate::from_ymd_opt(2001, 4, 28).unwrap();
    let dob_time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
    let location = Location::KATHMANDU; // can be customized

    println!("Date of Birth: {} {}", dob_date, dob_time);
    println!(
        "Location: {}, (Lat: {}, Lon: {})\n",
        location.name, location.latitude, location.longitude
    );

    // Construct DateTime<Utc>
    // 1. Combine Date and Time
    // 2. Set Timezone (Nepal is UTC+5:45)
    let offset = chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap();
    let local_dt = dob_date
        .and_time(dob_time)
        .and_local_timezone(offset)
        .unwrap();
    let utc_dt = local_dt.with_timezone(&chrono::Utc);

    // Get Astro Info
    match astro.get_daily_astro_info(utc_dt, &location) {
        Ok(info) => {
            println!("--- Zodiac Details ---");
            println!("Sun Sign (Surya Rashi):  {:?}", info.sun_sign);
            println!("Moon Sign (Chandra Rashi): {:?}", info.moon_sign); // This is usually the "Rashi" in horoscope
            println!("\n--- Other Details ---");
            println!("Nakshatra: {:?}", info.nakshatra);
            println!("Tithi: {:?}", info.tithi);
            println!("Paksha: {:?}", info.tithi.paksha());
        }
        Err(e) => println!("Error calculating astro info: {}", e),
    }
}
