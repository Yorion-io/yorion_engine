use yorion_engine::prelude::*;
use std::sync::Arc;

fn main() {
    println!("=== BS Recurring Events Example ===\n");

    // Setup
    let provider = StaticCalendarProvider::new();
    let conversion = Arc::new(ConversionService::new(Arc::new(provider)));
    let generator = InstanceGenerator::new(conversion.clone());

    // 1. Daily Recurrence
    println!("--- 1. Daily Recurrence (e.g., Daily Standup) ---");
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Daily, anchor).with_count(5);

    print_instances(&generator, &rule, anchor, BsDate::new(2080, 1, 10).unwrap());

    // 2. Weekly Recurrence
    println!("\n--- 2. Weekly Recurrence (e.g., Weekly Market) ---");
    let rule = BsRecurrenceRule::new(BsFrequency::Weekly, anchor).with_count(4);
    print_instances(&generator, &rule, anchor, BsDate::new(2080, 2, 1).unwrap());

    // 3. Monthly Recurrence with Clamping
    println!("\n--- 3. Monthly Recurrence (e.g., Salary Day on 30th) ---");
    // Starting on 30th. Month 2 (Jestha) might have 32 days, Month 12 (Chaitra) usually 30/31.
    // If a month has fewer than 30 days (rare in BS but possible), it clamps.
    let anchor = BsDate::new(2080, 1, 30).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor).with_count(6);
    print_instances(&generator, &rule, anchor, BsDate::new(2080, 7, 1).unwrap());

    // 4. Yearly Recurrence
    println!("\n--- 4. Yearly Recurrence (e.g., Birthday) ---");
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Yearly, anchor).with_count(3);
    print_instances(&generator, &rule, anchor, BsDate::new(2085, 1, 1).unwrap());

    // 5. Complex: Every month on 1st and 15th
    println!("\n--- 5. Monthly on 1st and 15th ---");
    let anchor = BsDate::new(2080, 1, 1).unwrap();
    let rule = BsRecurrenceRule::new(BsFrequency::Monthly, anchor)
        .with_by_month_day(vec![1, 15])
        .with_count(6);
    print_instances(&generator, &rule, anchor, BsDate::new(2080, 4, 1).unwrap());
}

fn print_instances(
    generator: &InstanceGenerator,
    rule: &BsRecurrenceRule,
    start: BsDate,
    end: BsDate,
) {
    match generator.generate_bs_instances(rule, start, end) {
        Ok(instances) => {
            for (i, instance) in instances.iter().enumerate() {
                println!("  {}. {}", i + 1, instance);
            }
        }
        Err(e) => println!("Error generating instances: {}", e),
    }
}
