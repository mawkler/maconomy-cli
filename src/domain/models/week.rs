use std::fmt::Display;

use chrono::{NaiveDate, Weekday};

// TODO: switch from `u8` to `IsoWeek`
#[derive(Debug, Clone)]
pub(crate) struct WeekNumber(u8);

impl WeekNumber {
    pub(crate) fn first_day(&self, year: i32) -> Option<NaiveDate> {
        NaiveDate::from_isoywd_opt(year, self.0.into(), Weekday::Mon)
    }
}

impl From<u8> for WeekNumber {
    fn from(week: u8) -> Self {
        Self(week)
    }
}

impl Display for WeekNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
            let week_number = WeekNumber(week);
            let date = week_number.first_day(year).unwrap();

            assert_eq!(
                date,
                NaiveDate::parse_from_str(expected_date, "%Y-%m-%d").unwrap()
            )
        }
    }
}
