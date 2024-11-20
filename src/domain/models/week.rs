use anyhow::anyhow;
use chrono::{Datelike, NaiveDate, Weekday};
use std::fmt::Display;

#[derive(Debug, Clone, Default, serde::Serialize)]
pub(crate) struct WeekNumber {
    number: u8,
    year: i32,
}

impl WeekNumber {
    pub(crate) fn new(week: u8, year: i32) -> anyhow::Result<Self> {
        first_day_of_week(week, year).ok_or(anyhow!("Invalid week '{week}'"))?;

        Ok(Self { number: week, year })
    }

    fn new_with_year_fallback(week: u8, year: Option<i32>) -> anyhow::Result<Self> {
        // Fall back to today's year
        let year = year.unwrap_or_else(|| chrono::Utc::now().year());
        WeekNumber::new(week, year)
    }

    pub(crate) fn new_with_fallback(
        week: Option<u8>,
        year: Option<i32>,
    ) -> anyhow::Result<WeekNumber> {
        let week = week.unwrap_or_else(|| {
            // Fall back to today's week
            let this_week = chrono::Local::now().date_naive().iso_week().week();
            this_week
                .try_into()
                .expect("Week numbers are always less than 255")
        });

        WeekNumber::new_with_year_fallback(week, year)
    }

    pub(crate) fn first_day(&self) -> Option<NaiveDate> {
        first_day_of_week(self.number, self.year)
    }
}

impl Display for WeekNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Week {}, year {}", self.number, self.year)
    }
}

fn first_day_of_week(week: u8, year: i32) -> Option<NaiveDate> {
    NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Mon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_date_of_first_day_of_week() {
        let dates = [
            (2024, 46, "2024-11-11"),
            (2030, 1, "2029-12-31"),
            (2028, 23, "2028-06-05"),
        ];

        for (year, week, expected_date) in dates {
            let week_number = WeekNumber { number: week, year };
            let date = week_number.first_day().unwrap();

            assert_eq!(
                date,
                NaiveDate::parse_from_str(expected_date, "%Y-%m-%d").unwrap()
            )
        }
    }
}
