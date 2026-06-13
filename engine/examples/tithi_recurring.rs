use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::CalendarVersion;
use yorion_engine::prelude::*;

fn main() {
    println!("=== Tithi Recurring Events Example ===\n");

    let engine = CalendarEngine::new();

    let start_date = BsDate::new(2080, 1, 1).unwrap();
    let end_date = BsDate::new(2080, 6, 1).unwrap(); // First 6 months

    println!(
        "Searching for matches between {} and {}...\n",
        start_date, end_date
    );

    // 1. Ekadashi (Twice a month)
    println!("--- 1. Ekadashi (11th Tithi) - Both Pakshas ---");
    let ekadashi_rule = TithiRecurrenceRule::ekadashi(start_date);
    print_instances(&engine, "ekadashi", &ekadashi_rule, start_date, end_date);

    // 2. Purnima (Full Moon - Once a month)
    println!("\n--- 2. Purnima (Full Moon) ---");
    let purnima_rule = TithiRecurrenceRule::purnima(start_date);
    print_instances(&engine, "purnima", &purnima_rule, start_date, end_date);

    // 3. Krishna Janmashtami (Krishna Asthami in Bhadra? - Just showing Krishna Asthami Generic here)
    println!("\n--- 3. Krishna Paksha Ashtami (Monthly) ---");
    let ashtami_rule =
        TithiRecurrenceRule::with_paksha(vec![Tithi::KrishnaAshtami], Paksha::Krishna, start_date);
    print_instances(&engine, "ashtami", &ashtami_rule, start_date, end_date);
}

fn print_instances(
    engine: &CalendarEngine,
    event_id: &str,
    rule: &TithiRecurrenceRule,
    start: BsDate,
    end: BsDate,
) {
    let instances = engine
        .generate_tithi_instances(
            event_id,
            event_id,
            rule,
            start,
            end,
            CalendarVersion::official("example".to_string()),
            Location::kathmandu(),
        )
        .unwrap();

    for (i, instance) in instances.iter().enumerate() {
        println!(
            "  {}. {} - {:?}",
            i + 1,
            instance.bs_date,
            instance.tithi.expect("tithi instances always carry a tithi")
        );
    }
}
