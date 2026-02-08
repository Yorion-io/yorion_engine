// Domain BS Date Tests
use bs_calendar_core::domain::bs_date::{BsDate, BsMonth};

mod helpers;

#[test]
fn test_bs_month_creation() {
    assert_eq!(BsMonth::from_u8(1).unwrap(), BsMonth::Baisakh);
    assert_eq!(BsMonth::from_u8(12).unwrap(), BsMonth::Chaitra);
    assert!(BsMonth::from_u8(0).is_err());
    assert!(BsMonth::from_u8(13).is_err());
}

#[test]
fn test_bs_month_navigation() {
    assert_eq!(BsMonth::Baisakh.next(), BsMonth::Jestha);
    assert_eq!(BsMonth::Chaitra.next(), BsMonth::Baisakh);
    assert_eq!(BsMonth::Jestha.prev(), BsMonth::Baisakh);
    assert_eq!(BsMonth::Baisakh.prev(), BsMonth::Chaitra);
}

#[test]
fn test_bs_date_creation() {
    let date = BsDate::new(2080, 1, 15).unwrap();
    assert_eq!(date.year, 2080);
    assert_eq!(date.month, BsMonth::Baisakh);
    assert_eq!(date.day, 15);
}

#[test]
fn test_bs_date_validation() {
    assert!(BsDate::new(2080, 1, 0).is_err());
    assert!(BsDate::new(2080, 1, 33).is_err());
    assert!(BsDate::new(2080, 13, 1).is_err());
}

#[test]
fn test_bs_date_formatting() {
    let date = BsDate::new(2080, 1, 15).unwrap();
    assert_eq!(date.format(), "2080-01-15");
    assert_eq!(date.to_string(), "2080 Baisakh 15");
}
