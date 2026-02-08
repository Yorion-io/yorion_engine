use bs_calendar_core::prelude::*;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::Arc;

fn main() {
    println!("=== Generate Detailed BS Calendar (2080-2082) ===\n");

    // 1. Setup services
    let provider = Arc::new(StaticCalendarProvider::new());
    let conversion_service = Arc::new(ConversionService::new(provider.clone()));
    let astronomical_service = Arc::new(AstronomicalService::new());

    // Create output directory
    create_dir_all("data/gen").expect("Failed to create data/gen directory");

    // 2. Generate for each year
    for year in 2080..=2082 {
        let file_path = format!("data/gen/calendar_{}.csv", year);
        let mut file = File::create(&file_path).expect("Failed to create CSV file");

        // Write header
        writeln!(
            file,
            "BS Date,AD Date,Day of Week,Tithi,Paksha,Sun Sign,Moon Sign,Nakshatra"
        )
        .expect("Failed to write header");

        println!("Generating for year {}...", year);

        for month_index in 1..=12 {
            let month = BsMonth::from_u8(month_index as u8).unwrap();
            let days_in_month = provider
                .get_month_days(year, month)
                .expect("Failed to get days in month");

            for day in 1..=days_in_month {
                let bs_date = BsDate::new(year, month_index as u8, day).unwrap();

                // Convert to AD
                let ad_date = conversion_service.bs_to_gregorian(bs_date).unwrap();

                // Get Astronomical Info (at Sunrise in Kathmandu)
                let astro_info = astronomical_service
                    .get_daily_astro_info_for_date(ad_date, Location::KATHMANDU)
                    .expect("Failed to calculate astro info");

                writeln!(
                    file,
                    "{},{},{},{},{:?},{},{},{}",
                    bs_date.format(),
                    ad_date.format("%Y-%m-%d"),
                    ad_date.format("%A"),
                    astro_info.tithi,
                    astro_info.tithi.paksha(),
                    astro_info.sun_sign,
                    astro_info.moon_sign,
                    astro_info.nakshatra
                )
                .expect("Failed to write row");
            }
        }
        println!("Saved to {}", file_path);
    }

    println!("\nDone! Check data/gen/ directory for output files.");
}
