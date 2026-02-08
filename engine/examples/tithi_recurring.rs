use bs_calendar_core::adapters::StaticTithiOverrideProvider;
use bs_calendar_core::prelude::*;
use std::sync::Arc;

fn main() {
    println!("=== Tithi Recurring Events Example (Optimized) ===\n");

    // Setup
    let provider = StaticCalendarProvider::new();
    let conversion = Arc::new(ConversionService::new(Arc::new(provider)));

    // Initialize AstronomicalService with overrides enabled
    let astro = Arc::new(AstronomicalService::with_overrides(Box::new(
        StaticTithiOverrideProvider::new(),
    )));

    // Initialize InstanceGenerator
    let generator = InstanceGenerator::new(conversion.clone());

    let start_date = BsDate::new(2080, 1, 1).unwrap();
    let end_date = BsDate::new(2080, 6, 1).unwrap(); // First 6 months

    println!(
        "Searching for matches between {} and {}...\n",
        start_date, end_date
    );

    // 1. Ekadashi (Twice a month)
    println!("--- 1. Ekadashi (11th Tithi) - Both Pakshas ---");
    let ekadashi_rule = TithiRecurrenceRule::ekadashi(start_date);
    print_instances(
        &generator,
        &ekadashi_rule,
        start_date,
        end_date,
        &astro,
        &conversion,
    );

    // 2. Purnima (Full Moon - Once a month)
    println!("\n--- 2. Purnima (Full Moon) ---");
    let purnima_rule = TithiRecurrenceRule::purnima(start_date);
    print_instances(
        &generator,
        &purnima_rule,
        start_date,
        end_date,
        &astro,
        &conversion,
    );

    // 3. Krishna Janmashtami (Krishna Asthami in Bhadra? - Just showing Krishna Asthami Generic here)
    println!("\n--- 3. Krishna Paksha Ashtami (Monthly) ---");
    let ashtami_rule =
        TithiRecurrenceRule::with_paksha(vec![Tithi::KrishnaAshtami], Paksha::Krishna, start_date);
    print_instances(
        &generator,
        &ashtami_rule,
        start_date,
        end_date,
        &astro,
        &conversion,
    );
}

fn print_instances(
    generator: &InstanceGenerator,
    rule: &TithiRecurrenceRule,
    start: BsDate,
    end: BsDate,
    astro: &AstronomicalService,
    conversion: &ConversionService, // for converting matched BS dates to AD/Astro info for printing
) {
    let instances = generator
        .generate_tithi_instances(rule, start, end, astro)
        .unwrap();

    for (i, bs_date) in instances.iter().enumerate() {
        // Get details for valid output verification
        let ad_date = conversion.bs_to_gregorian(*bs_date).unwrap();
        let tithi = astro
            .calculate_tithi_for_date(ad_date, Location::KATHMANDU)
            .unwrap();
        println!("  {}. {} - {:?}", i + 1, bs_date, tithi);
    }
}
