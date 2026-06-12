use yorion_engine::prelude::*;
use std::collections::HashMap;
use std::fs;

fn parse_almanac_csv(path: &str) -> HashMap<String, (String, String, u8)> {
    // Returns: bs_date -> (tithi_display, paksha, tithi_day)
    let mut map = HashMap::new();
    let content = fs::read_to_string(path).expect("Failed to read CSV");
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.splitn(7, ',').collect();
        if parts.len() < 6 {
            continue;
        }
        let bs_date = parts[0].to_string();
        let tithi_display = parts[2].to_string();
        let paksha = parts[3].to_string();
        let tithi_day: u8 = parts[4].parse().unwrap_or(0);
        map.insert(bs_date, (tithi_display, paksha, tithi_day));
    }
    map
}

fn tithi_to_day_and_paksha(tithi: Tithi) -> (u8, &'static str) {
    use yorion_engine::domain::tithi::Paksha;
    let paksha = tithi.paksha();
    let day = tithi.day_in_paksha();
    let paksha_str = match paksha {
        Paksha::Shukla => "Shukla",
        Paksha::Krishna => "Krishna",
    };
    (day, paksha_str)
}

fn main() {
    let almanac_csv = "../../07_tools/sources/almanac/calendar_2083.csv";
    let almanac_data = parse_almanac_csv(almanac_csv);

    let provider = std::sync::Arc::new(StaticCalendarProvider::new());
    let conversion_service = std::sync::Arc::new(ConversionService::new(provider.clone()));
    // Use NO overrides so we see raw engine mismatches vs the almanac
    let astronomical_service = std::sync::Arc::new(AstronomicalService::new());

    let year: u16 = 2083;
    let mut mismatches = vec![];
    let mut matches = 0;
    let mut total = 0;

    for month_index in 1u8..=12 {
        let month = BsMonth::from_u8(month_index).unwrap();
        let days_in_month = provider.get_month_days(year, month).unwrap();

        for day in 1..=days_in_month {
            let bs_date = BsDate::new(year, month_index, day).unwrap();
            let bs_str = bs_date.format(); // "YYYY-MM-DD"

            let ad_date = conversion_service.bs_to_gregorian(bs_date).unwrap();

            let astro_info = astronomical_service
                .get_daily_astro_info_for_date(ad_date, &Location::kathmandu())
                .expect("astro fail");

            let (engine_day, engine_paksha) = tithi_to_day_and_paksha(astro_info.tithi);
            let engine_name = astro_info.tithi.to_string();

            total += 1;

            if let Some((al_display, al_paksha, al_day)) = almanac_data.get(&bs_str) {
                let day_match = engine_day == *al_day;
                let paksha_match = engine_paksha == al_paksha.as_str()
                    || al_paksha.is_empty()
                    || (al_display == "Purnima" || al_display == "Amavasya");

                if !day_match || (!paksha_match && !al_paksha.is_empty()) {
                    mismatches.push((
                        bs_str.clone(),
                        ad_date.to_string(),
                        engine_name.clone(),
                        engine_day,
                        engine_paksha,
                        al_display.clone(),
                        *al_day,
                        al_paksha.clone(),
                        astro_info.is_overridden,
                    ));
                } else {
                    matches += 1;
                }
            } else {
                mismatches.push((
                    bs_str.clone(),
                    ad_date.to_string(),
                    engine_name.clone(),
                    engine_day,
                    engine_paksha,
                    "MISSING IN ALMANAC".to_string(),
                    0,
                    "".to_string(),
                    astro_info.is_overridden,
                ));
            }
        }
    }

    println!("=== Tithi Comparison: Engine vs Almanac 2083 ===\n");
    println!("Total days: {}", total);
    println!("Matches:    {}", matches);
    println!("Mismatches: {}\n", mismatches.len());

    if mismatches.is_empty() {
        println!("Perfect match!");
        return;
    }

    println!(
        "{:<12} {:<12} {:<25} {:<25} Override?",
        "BS Date", "AD Date", "Engine", "Almanac"
    );
    println!("{}", "-".repeat(90));

    for (bs, ad, eng_name, eng_day, eng_pak, al_disp, al_day, _al_pak, is_override) in &mismatches {
        let engine_fmt = format!("{} {} (day {})", eng_pak, eng_name, eng_day);
        let al_fmt = if al_disp == "MISSING IN ALMANAC" {
            al_disp.clone()
        } else {
            format!("{} (day {})", al_disp, al_day)
        };
        let override_marker = if *is_override { " [OVERRIDE]" } else { "" };
        println!(
            "{:<12} {:<12} {:<40} {:<30}{}",
            bs, ad, engine_fmt, al_fmt, override_marker
        );
    }
}
