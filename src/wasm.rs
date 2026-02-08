use crate::core_api::CalendarEngine;
use crate::domain::tithi::Location;
use crate::domain::Language;
use crate::prelude::*;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Clone)]
pub struct WasmCalendarDay {
    pub bs_year: u16,
    pub bs_month: u8,
    pub bs_day: u8,
    pub gregorian_date: String,
    pub day_of_week: u8, // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
    pub tithi: Tithi,
    pub sun_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
    pub nakshatra: Nakshatra,
    pub is_overridden: bool,
}

#[wasm_bindgen]
pub struct MonthCalendar {
    days: Vec<WasmCalendarDay>,
    pub year: u16,
    pub month: u8,
    pub days_in_month: u8,
    pub start_day_of_week: u8,
}

#[wasm_bindgen]
impl MonthCalendar {
    #[wasm_bindgen(getter)]
    pub fn days(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        for day in &self.days {
            // We can't directly store WasmCalendarDay in Array if it's not a JsValue.
            // But it IS a #[wasm_bindgen] struct, so we can convert it.
            arr.push(&JsValue::from(day.clone()));
        }
        arr
    }
}

struct WasmTimeProvider;
impl crate::ports::TimeProvider for WasmTimeProvider {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn sunrise_time(
        &self,
        _date: NaiveDate,
        _location: Location,
    ) -> crate::error::Result<chrono::NaiveTime> {
        Ok(chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap())
    }
    fn sunset_time(
        &self,
        _date: NaiveDate,
        _location: Location,
    ) -> crate::error::Result<chrono::NaiveTime> {
        Ok(chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap())
    }
}

static ENGINE: OnceLock<CalendarEngine> = OnceLock::new();

fn get_engine() -> &'static CalendarEngine {
    ENGINE.get_or_init(|| {
        let time_provider = std::sync::Arc::new(WasmTimeProvider);
        CalendarEngine::with_time_provider(time_provider)
    })
}

#[wasm_bindgen]
pub fn get_month_calendar_with_location(
    year: u16,
    month: u8,
    location: &Location,
) -> std::result::Result<MonthCalendar, JsValue> {
    let engine = get_engine();

    let calendar_data = engine
        .get_month_calendar(year, month, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Convert CalendarDay to WasmCalendarDay
    let days: Vec<WasmCalendarDay> = calendar_data
        .days
        .into_iter()
        .map(|day| WasmCalendarDay {
            bs_year: day.bs_year,
            bs_month: day.bs_month,
            bs_day: day.bs_day,
            gregorian_date: day.gregorian_date.format("%Y-%m-%d").to_string(),
            day_of_week: day.day_of_week,
            tithi: day.tithi,
            sun_sign: day.sun_sign,
            moon_sign: day.moon_sign,
            nakshatra: day.nakshatra,
            is_overridden: day.is_overridden,
        })
        .collect();

    Ok(MonthCalendar {
        days,
        year: calendar_data.year,
        month: calendar_data.month,
        days_in_month: calendar_data.days_in_month,
        start_day_of_week: calendar_data.start_day_of_week,
    })
}

#[wasm_bindgen]
pub fn bs_to_gregorian(year: u16, month: u8, day: u8) -> std::result::Result<String, JsValue> {
    let engine = get_engine();
    let bs_date = BsDate::new(year, month, day).map_err(|e| JsValue::from_str(&e.to_string()))?;
    engine
        .bs_to_gregorian(bs_date)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn gregorian_to_bs(year: i32, month: u32, day: u32) -> std::result::Result<BsDate, JsValue> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| JsValue::from_str("Invalid Gregorian Date"))?;
    engine
        .gregorian_to_bs(gregorian)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn get_tithi(year: i32, month: u32, day: u32) -> std::result::Result<Tithi, JsValue> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| JsValue::from_str("Invalid Gregorian Date"))?;
    engine
        .get_tithi(gregorian)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn get_tithi_name(tithi: Tithi, lang: Language) -> String {
    let engine = get_engine();
    engine.get_tithi_name(tithi, lang)
}

#[wasm_bindgen]
pub fn get_zodiac_name(zodiac: ZodiacSign, lang: Language) -> String {
    let engine = get_engine();
    engine.get_zodiac_name(zodiac, lang)
}

#[wasm_bindgen]
pub fn get_nakshatra_name(nakshatra: Nakshatra, lang: Language) -> String {
    let engine = get_engine();
    engine.get_nakshatra_name(nakshatra, lang)
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WasmDailyAstroInfo {
    pub tithi: Tithi,
    pub sun_sign: ZodiacSign,
    pub moon_sign: ZodiacSign,
    pub nakshatra: Nakshatra,
    pub sunrise: String,
    pub sunset: String,
    pub is_overridden: bool,
}

#[wasm_bindgen]
pub fn get_sunrise_with_location(
    year: i32,
    month: u32,
    day: u32,
    location: &Location,
) -> std::result::Result<String, JsValue> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| JsValue::from_str("Invalid Gregorian Date"))?;
    let sunrise = engine
        .get_sunrise(gregorian, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(sunrise.format("%H:%M:%S").to_string())
}

#[wasm_bindgen]
pub fn get_sunset_with_location(
    year: i32,
    month: u32,
    day: u32,
    location: &Location,
) -> std::result::Result<String, JsValue> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| JsValue::from_str("Invalid Gregorian Date"))?;
    let sunset = engine
        .get_sunset(gregorian, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(sunset.format("%H:%M:%S").to_string())
}

#[wasm_bindgen]
pub fn get_daily_astro_info_with_location(
    year: i32,
    month: u32,
    day: u32,
    location: &Location,
) -> std::result::Result<WasmDailyAstroInfo, JsValue> {
    let engine = get_engine();
    let gregorian = NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| JsValue::from_str("Invalid Gregorian Date"))?;

    let info = engine
        .get_daily_astro_info(gregorian, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let sunrise = engine
        .get_sunrise(gregorian, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let sunset = engine
        .get_sunset(gregorian, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(WasmDailyAstroInfo {
        tithi: info.tithi,
        sun_sign: info.sun_sign,
        moon_sign: info.moon_sign,
        nakshatra: info.nakshatra,
        sunrise: sunrise.format("%H:%M:%S").to_string(),
        sunset: sunset.format("%H:%M:%S").to_string(),
        is_overridden: info.is_overridden,
    })
}

#[wasm_bindgen]
pub fn get_month_events(
    year: u16,
    month: u8,
    events_json: String,
    location: &Location,
) -> std::result::Result<js_sys::Array, JsValue> {
    let engine = get_engine();
    let events: Vec<crate::domain::event::Event> = serde_json::from_str(&events_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse events: {}", e)))?;

    let bs_month =
        crate::domain::BsMonth::from_u8(month).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let start_date = crate::domain::BsDate::new(year, month, 1)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let days_in_month = engine
        .calendar()
        .get_month_days(year, bs_month)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let end_date = crate::domain::BsDate::new(year, month, days_in_month)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let instances = engine
        .generate_event_instances(events, start_date, end_date, *location)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let arr = js_sys::Array::new();
    for instance in instances {
        arr.push(
            &serde_wasm_bindgen::to_value(&instance)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        );
    }
    Ok(arr)
}
