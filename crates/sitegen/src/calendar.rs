//! UTC footer year and Gregorian calendar arithmetic.
use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn current_utc_year() -> Result<i32> {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock predates the Unix epoch")?
        .as_secs()
        / 86_400;
    Ok(year_from_days_since_epoch(days))
}

fn year_from_days_since_epoch(mut days: u64) -> i32 {
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            return year;
        }
        days -= days_in_year;
        year += 1;
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::{is_leap_year, year_from_days_since_epoch};

    #[test]
    fn computes_year_from_unix_days() {
        assert_eq!(year_from_days_since_epoch(0), 1970);
        assert_eq!(year_from_days_since_epoch(364), 1970);
        assert_eq!(year_from_days_since_epoch(365), 1971);
        assert_eq!(year_from_days_since_epoch(10_957), 2000);
        assert_eq!(year_from_days_since_epoch(11_323), 2001);
    }

    #[test]
    fn applies_gregorian_leap_year_rules() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2025));
    }
}
