// Engine performance benchmarks
//
// Groups:
//   conversion   — bs_to_gregorian, gregorian_to_bs (cheap table lookups)
//   astronomical — get_tithi, get_sunrise, get_daily_astro_info (VSOP87/ELP + suncalc)
//   month_cal    — get_month_calendar (29-32 astronomical calls)
//   tithi_gen    — generate_tithi_instances across various window widths
//   bs_gen       — generate_bs_instances (pure date arithmetic)
//   unbounded    — tithi_gen without UNTIL/COUNT to show how many days are walked

use yorion_engine::core_api::CalendarEngine;
use yorion_engine::domain::bs_date::{BsDate, BsMonth};
use yorion_engine::domain::event::CalendarVersion;
use yorion_engine::domain::recurrence::{BsFrequency, BsRecurrenceRule, TithiRecurrenceRule};
use yorion_engine::domain::tithi::{Location, Paksha, Tithi};
use chrono::NaiveDate;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

fn engine() -> CalendarEngine {
    CalendarEngine::new()
}

fn version() -> CalendarVersion {
    CalendarVersion::official("bench".to_string())
}

fn ktm() -> Location {
    Location::kathmandu()
}

fn bs(y: u16, m: u8, d: u8) -> BsDate {
    BsDate::new(y, m, d).unwrap()
}

fn ad(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

// ─── Conversion ─────────────────────────────────────────────────────────────

fn bench_conversion(c: &mut Criterion) {
    let eng = engine();
    let mut g = c.benchmark_group("conversion");

    g.bench_function("bs_to_gregorian", |b| {
        b.iter(|| eng.bs_to_gregorian(black_box(bs(2080, 6, 15))).unwrap())
    });

    g.bench_function("gregorian_to_bs", |b| {
        b.iter(|| eng.gregorian_to_bs(black_box(ad(2023, 10, 2))).unwrap())
    });

    g.finish();
}

// ─── Astronomical (per-day) ──────────────────────────────────────────────────

fn bench_astronomical(c: &mut Criterion) {
    let eng = engine();
    let loc = ktm();
    let date = ad(2023, 10, 5); // Bijaya Dashami 2080

    let mut g = c.benchmark_group("astronomical");

    g.bench_function("get_tithi", |b| {
        b.iter(|| eng.get_tithi(black_box(date)).unwrap())
    });

    g.bench_function("get_sunrise", |b| {
        b.iter(|| eng.get_sunrise(black_box(date), black_box(loc.clone())).unwrap())
    });

    g.bench_function("get_daily_astro_info", |b| {
        b.iter(|| {
            eng.get_daily_astro_info(black_box(date), black_box(loc.clone()))
                .unwrap()
        })
    });

    // Override hit vs miss — compare a date with a known override vs one without
    let override_date = ad(2022, 8, 2); // Nag Panchami 2079: has override entry
    let clean_date = ad(2023, 10, 5);   // Bijaya Dashami 2080: no override entry

    g.bench_function("tithi_override_hit", |b| {
        b.iter(|| eng.get_tithi(black_box(override_date)).unwrap())
    });

    g.bench_function("tithi_override_miss", |b| {
        b.iter(|| eng.get_tithi(black_box(clean_date)).unwrap())
    });

    g.finish();
}

// ─── Month calendar ──────────────────────────────────────────────────────────

fn bench_month_calendar(c: &mut Criterion) {
    let eng = engine();
    let loc = ktm();
    let mut g = c.benchmark_group("month_calendar");

    // Short month (29 days) vs long month (32 days)
    for (year, month, label) in [(2080u16, 4u8, "shrawan_32"), (2080u16, 9u8, "poush_29")] {
        g.bench_with_input(
            BenchmarkId::from_parameter(label),
            &(year, month),
            |b, &(y, m)| {
                b.iter(|| {
                    eng.get_month_calendar(black_box(y), black_box(m), black_box(loc.clone()))
                        .unwrap()
                })
            },
        );
    }

    g.finish();
}

// ─── Tithi instance generation ───────────────────────────────────────────────

fn tithi_rule_bijaya_dashami(anchor_year: u16) -> TithiRecurrenceRule {
    TithiRecurrenceRule::with_paksha(
        vec![Tithi::ShuklaDashami],
        Paksha::Shukla,
        bs(anchor_year, 1, 1),
    )
    .with_by_month(vec![BsMonth::Ashwin, BsMonth::Kartik])
    .with_take_first(true)
}

fn tithi_rule_shivaratri(anchor_year: u16) -> TithiRecurrenceRule {
    TithiRecurrenceRule::with_paksha(
        vec![Tithi::KrishnaChaturdashi],
        Paksha::Krishna,
        bs(anchor_year, 1, 1),
    )
    .with_by_month(vec![BsMonth::Falgun])
    .with_take_first(true)
}

fn tithi_rule_purnima_unfiltered(anchor_year: u16) -> TithiRecurrenceRule {
    // No BYMONTH — fires every lunar month (~12–13 times/year)
    TithiRecurrenceRule::new(vec![Tithi::Purnima], bs(anchor_year, 1, 1))
}

fn bench_tithi_gen(c: &mut Criterion) {
    let eng = engine();
    let ver = version();
    let loc = ktm();
    let mut g = c.benchmark_group("tithi_gen");

    // Window widths: 1 year, 5 years, 10 years
    for years in [1u16, 5, 10] {
        let start = bs(2079, 1, 1);
        let end = bs(2079 + years - 1, 12, 30);
        let label = format!("{years}yr");

        // Bijaya Dashami (BYMONTH filtered + take_first)
        g.bench_with_input(
            BenchmarkId::new("bijaya_dashami", &label),
            &years,
            |b, _| {
                let rule = tithi_rule_bijaya_dashami(2079);
                b.iter(|| {
                    eng.generate_tithi_instances(
                        "test",
                        "test",
                        black_box(&rule),
                        black_box(start),
                        black_box(end),
                        black_box(ver.clone()),
                        black_box(loc.clone()),
                    )
                    .unwrap()
                })
            },
        );

        // Shivaratri (single month filter)
        g.bench_with_input(
            BenchmarkId::new("shivaratri", &label),
            &years,
            |b, _| {
                let rule = tithi_rule_shivaratri(2079);
                b.iter(|| {
                    eng.generate_tithi_instances(
                        "test",
                        "test",
                        black_box(&rule),
                        black_box(start),
                        black_box(end),
                        black_box(ver.clone()),
                        black_box(loc.clone()),
                    )
                    .unwrap()
                })
            },
        );

        // Purnima every month — no month filter, hits every lunar cycle
        g.bench_with_input(
            BenchmarkId::new("purnima_all_months", &label),
            &years,
            |b, _| {
                let rule = tithi_rule_purnima_unfiltered(2079);
                b.iter(|| {
                    eng.generate_tithi_instances(
                        "test",
                        "test",
                        black_box(&rule),
                        black_box(start),
                        black_box(end),
                        black_box(ver.clone()),
                        black_box(loc.clone()),
                    )
                    .unwrap()
                })
            },
        );
    }

    g.finish();
}

// ─── BS (solar) instance generation ─────────────────────────────────────────

fn bench_bs_gen(c: &mut Criterion) {
    let eng = engine();
    let mut g = c.benchmark_group("bs_gen");

    // Annual event (Nepali New Year = Baisakh 1)
    let ny_rule = BsRecurrenceRule {
        frequency: BsFrequency::Yearly,
        interval: 1,
        by_month: Some(vec![BsMonth::Baisakh]),
        by_month_day: Some(vec![1]),
        by_day: None,
        count: None,
        until: None,
        anchor: bs(2073, 1, 1),
    };

    for years in [1u16, 5, 10] {
        let start = bs(2079, 1, 1);
        let end = bs(2079 + years - 1, 12, 30);
        let label = format!("{years}yr");

        g.bench_with_input(
            BenchmarkId::new("new_year_annual", &label),
            &years,
            |b, _| {
                b.iter(|| {
                    eng.generate_bs_instances(
                        black_box(&ny_rule),
                        black_box(start),
                        black_box(end),
                    )
                    .unwrap()
                })
            },
        );
    }

    // Weekly recurring event — lots of instances
    let weekly_rule = BsRecurrenceRule {
        frequency: BsFrequency::Weekly,
        interval: 1,
        by_month: None,
        by_month_day: None,
        by_day: None,
        count: None,
        until: None,
        anchor: bs(2079, 1, 1),
    };

    g.bench_function("weekly_1yr", |b| {
        b.iter(|| {
            eng.generate_bs_instances(
                black_box(&weekly_rule),
                black_box(bs(2079, 1, 1)),
                black_box(bs(2079, 12, 30)),
            )
            .unwrap()
        })
    });

    g.finish();
}

// ─── Unbounded window — "how far does it walk without UNTIL/COUNT?" ──────────

fn bench_unbounded(c: &mut Criterion) {
    let eng = engine();
    let ver = version();
    let loc = ktm();
    let mut g = c.benchmark_group("unbounded_window");

    // The engine always requires explicit start+end from the caller.
    // "Unbounded" here means we pass the full engine data range (BS 1975–2100)
    // so we can see the true cost of walking every day with no month filter.

    let full_start = bs(2079, 1, 1);
    let full_end = bs(2083, 12, 30); // 5-year span — representative real-world calendar load

    // Worst case: no BYMONTH, walks every day to find Purnima
    let purnima_unfiltered = tithi_rule_purnima_unfiltered(2079);
    g.bench_function("purnima_no_bymonth_5yr", |b| {
        b.iter(|| {
            let instances = eng
                .generate_tithi_instances(
                    "test",
                    "test",
                    black_box(&purnima_unfiltered),
                    black_box(full_start),
                    black_box(full_end),
                    black_box(ver.clone()),
                    black_box(loc.clone()),
                )
                .unwrap();
            black_box(instances.len())
        })
    });

    // With BYMONTH=Shrawan — skips 11/12 months per year
    let purnima_shrawan = TithiRecurrenceRule::new(vec![Tithi::Purnima], bs(2079, 1, 1))
        .with_by_month(vec![BsMonth::Shrawan]);
    g.bench_function("purnima_bymonth_shrawan_5yr", |b| {
        b.iter(|| {
            let instances = eng
                .generate_tithi_instances(
                    "test",
                    "test",
                    black_box(&purnima_shrawan),
                    black_box(full_start),
                    black_box(full_end),
                    black_box(ver.clone()),
                    black_box(loc.clone()),
                )
                .unwrap();
            black_box(instances.len())
        })
    });

    // COUNT=1 — stops at first hit regardless of window
    let purnima_count1 = TithiRecurrenceRule::new(vec![Tithi::Purnima], bs(2079, 1, 1))
        .with_count(1);
    g.bench_function("purnima_count1", |b| {
        b.iter(|| {
            eng.generate_tithi_instances(
                "test",
                "test",
                black_box(&purnima_count1),
                black_box(full_start),
                black_box(full_end),
                black_box(ver.clone()),
                black_box(loc.clone()),
            )
            .unwrap()
        })
    });

    // COUNT=12 — one full year of Purnimas then stop
    let purnima_count12 = TithiRecurrenceRule::new(vec![Tithi::Purnima], bs(2079, 1, 1))
        .with_count(12);
    g.bench_function("purnima_count12", |b| {
        b.iter(|| {
            eng.generate_tithi_instances(
                "test",
                "test",
                black_box(&purnima_count12),
                black_box(full_start),
                black_box(full_end),
                black_box(ver.clone()),
                black_box(loc.clone()),
            )
            .unwrap()
        })
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_conversion,
    bench_astronomical,
    bench_month_calendar,
    bench_tithi_gen,
    bench_bs_gen,
    bench_unbounded,
);
criterion_main!(benches);
