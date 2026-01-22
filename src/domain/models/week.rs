use anyhow::anyhow;
use chrono::{Datelike, NaiveDate, Weekday, Duration};
use std::fmt::Display;
use std::convert::TryFrom;
use crate::cli::arguments::{WeekPart, WeekAndPart};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct WeekNumber {
    pub(crate) number: u8,
    pub(crate) part: WeekPart,
    pub(crate) year: i32,
}

impl WeekNumber {
    pub(crate) fn of(date: NaiveDate) -> WeekNumber {
        let iso_week = date.iso_week();
        let w: u8 = iso_week
            .week()
            .try_into()
            .expect("Week numbers are always less than 255");
        let iso_year = iso_week.year();

        let spans_new_month = week_spans_new_month(w, iso_year);

        if !spans_new_month {
            Self {number: w, part: WeekPart::WHOLE, year: iso_year,}
        } else {
            let month_a = NaiveDate::from_isoywd_opt(iso_year, w.into(), Weekday::Mon)
                .expect("Week number should be valid")
                .month();
            let month_candidate = date.month();
            if month_candidate == month_a {
                Self {number: w, part: WeekPart::A,year: iso_year,}
            } else {
                Self {number: w, part: WeekPart::B, year: iso_year,}
            }
        }
    }

    pub(crate) fn new(week: u8, part: WeekPart, year: i32) -> anyhow::Result<Self> {
        let monday = NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Mon)
            .ok_or(anyhow!("Invalid '{week}'"))?;
        let sunday = NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Sun)
            .ok_or(anyhow!("Invalid week '{week}'"))?;
        
        let spans_month = week_spans_new_month(week, year);
        
        match (spans_month, part) {
            (true, WeekPart::WHOLE) => {
                let month_monday = monday.month();
                let month_sunday = sunday.month();
                Err(anyhow!("Week part 'WHOLE' is not valid for week {week} because it spans a new month (Monday is in month {month_monday}, Sunday is in month {month_sunday})"))
            }
            (false, WeekPart::A) | (false, WeekPart::B) => {
                let month_monday = monday.month();
                Err(anyhow!("Week part '{:?}' is not valid for week {week} because it does not span a new month (both Monday and Sunday are in month {month_monday})", part))
            }
            _ => Ok(Self { number: week, part, year, })
        }
    }

    pub(crate) fn first_day(&self) -> Option<NaiveDate> {
        first_day_of_week(self.number,self.part, self.year)
    }

    pub(crate) fn previous(&self) -> Self {
        match self.part {
            WeekPart::B => {
                // If current week is B week, previous week is A week of the same number
                Self {
                    number: self.number,
                    part: WeekPart::A,
                    year: self.year,
                }
            }
            WeekPart::WHOLE | WeekPart::A => {
                // Get the first day of current week and go back 7 days to get previous week
                let current_first_day = self.first_day().expect("Week number should be valid");
                let previous_week_date = current_first_day - Duration::days(7);
                
                // Get the week number and year of the previous week
                let previous_week_number: u8 = previous_week_date
                    .iso_week()
                    .week()
                    .try_into()
                    .expect("Week numbers are always less than 255");
                let previous_year = previous_week_date.iso_week().year();
                
                // Check if the previous week spans a new month
                let prev_week_spans_month = week_spans_new_month(previous_week_number, previous_year);
                
                if prev_week_spans_month {
                    // If previous week would cross the new month, return B week
                    Self {
                        number: previous_week_number,
                        part: WeekPart::B,
                        year: previous_year,
                    }
                } else {
                    // Otherwise, return WHOLE week
                    Self {
                        number: previous_week_number,
                        part: WeekPart::WHOLE,
                        year: previous_year,
                    }
                }
            }
        }
    }
}

impl Default for WeekNumber {
    fn default() -> Self {
        WeekNumber::of(chrono::Local::now().date_naive())
    }
}

impl Display for WeekNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.part {
            WeekPart::WHOLE => write!(f, "{}W{}", self.year, self.number),
            WeekPart::A => write!(f, "{}W{}A",  self.year, self.number),
            WeekPart::B => write!(f, "{}W{}B",  self.year, self.number),
        }
    }
}

impl TryFrom<WeekAndPart> for WeekNumber {
    type Error = anyhow::Error;

    fn try_from(week_and_part: WeekAndPart) -> Result<Self, Self::Error> {
        let number = week_and_part.number.ok_or_else(|| anyhow!("Week number is required"))?;
        let part = week_and_part.part.unwrap_or(WeekPart::WHOLE);
        let year = chrono::Utc::now().year();

        WeekNumber::new(number, part, year)
    }
}

fn first_day_of_week(week: u8, part: WeekPart, year: i32) -> Option<NaiveDate> {
    match part {
        WeekPart::WHOLE => NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Mon),
        WeekPart::A => NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Mon),
        WeekPart::B => {
            let sunday = NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Sun)?;
            NaiveDate::from_ymd_opt(sunday.year(), sunday.month(), 1)
        }
    }
}

fn week_spans_new_month(week: u8, year: i32) -> bool {
    let monday = first_day_of_week(week, WeekPart::WHOLE, year)
        .expect("Week number should be valid");
    let sunday = NaiveDate::from_isoywd_opt(year, week.into(), Weekday::Sun)
        .expect("Week number should be valid");

    monday.month() != sunday.month()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_date_of_first_day_of_week() {
        let dates = [
            (2024, 46, WeekPart::WHOLE, "2024-11-11"),
            (2030, 1, WeekPart::A, "2029-12-31"),
            (2028, 23, WeekPart::WHOLE,"2028-06-05"),
        ];

        for (year, week,part, expected_date) in dates {
            let week_number = WeekNumber { number: week, part: part, year };
            let date = week_number.first_day().unwrap();

            assert_eq!(
                date,
                NaiveDate::parse_from_str(expected_date, "%Y-%m-%d").unwrap()
            )
        }
    }

    #[test]
    fn parses_date_into_week_number() {
        let dates = [
            ("2024-11-11", WeekNumber { number: 46, part: WeekPart::WHOLE, year: 2024 }),
            ("2029-12-31", WeekNumber { number: 1, part: WeekPart::A, year: 2030 }),
            ("2028-06-05", WeekNumber { number: 23, part: WeekPart::WHOLE, year: 2028 }),
        ];

        for (date, expected_week_number) in dates {
            let week_number = WeekNumber::of(NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap());
            assert_eq!(week_number, expected_week_number);
        }
    }

    #[test]
    fn test_year_end_weeks() {
        // Test year transitions: only Dec 31 and Jan 1 for each new year
        // Week over new year 2025/26 is week 1
        // Week over new year 2022/23 is week 52 (Jan 1 is week 52B)
        let test_cases = [
            ("2022-12-31", WeekNumber { number: 52, part: WeekPart::A, year: 2022 }),
            ("2023-01-01", WeekNumber { number: 52, part: WeekPart::B, year: 2022 }),

            ("2024-01-01", WeekNumber { number: 1, part: WeekPart::WHOLE, year: 2024 }),
            ("2024-12-31", WeekNumber { number: 1, part: WeekPart::A, year: 2025 }),
            ("2025-01-01", WeekNumber { number: 1, part: WeekPart::B, year: 2025 }),
            ("2025-12-31", WeekNumber { number: 1, part: WeekPart::A, year: 2026 }),
            ("2026-01-01", WeekNumber { number: 1, part: WeekPart::B, year: 2026 }),
            ("2026-12-31", WeekNumber { number: 53, part: WeekPart::A, year: 2026 }),
            ("2027-01-01", WeekNumber { number: 53, part: WeekPart::B, year: 2026 }),

            ("2023-12-31", WeekNumber { number: 52, part: WeekPart::WHOLE, year: 2023 }),
        ];

        for (date_str, expected_week_number) in test_cases {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
            let iso_week = date.iso_week();
            let w = iso_week.week();
            let iso_year = iso_week.year();
            let monday = NaiveDate::from_isoywd_opt(iso_year, w.into(), Weekday::Mon).unwrap();
            let sunday = NaiveDate::from_isoywd_opt(iso_year, w.into(), Weekday::Sun).unwrap();
            let spans = week_spans_new_month(w.try_into().unwrap(), iso_year);
            
            println!("Date: {} -> ISO week {} of year {}", date_str, w, iso_year);
            println!("  Monday: {} (month {}), Sunday: {} (month {})", monday, monday.month(), sunday, sunday.month());
            println!("  Spans month: {}", spans);
            
            let week_number = WeekNumber::of(date);
            assert_eq!(
                week_number, expected_week_number,
                "Date {} should be {:?}, but got {:?}",
                date_str, expected_week_number, week_number
            );
        }
    }

    #[test]
    fn test_previous() {
        // Test cases: (current_week, expected_previous_week)
        let test_cases = [
            // B week -> A week (same number, same year)
            (
                WeekNumber { number: 1, part: WeekPart::B, year: 2025 },
                WeekNumber { number: 1, part: WeekPart::A, year: 2025 },
            ),
            (
                WeekNumber { number: 52, part: WeekPart::B, year: 2022 },
                WeekNumber { number: 52, part: WeekPart::A, year: 2022 },
            ),
            (
                WeekNumber { number: 53, part: WeekPart::B, year: 2026 },
                WeekNumber { number: 53, part: WeekPart::A, year: 2026 },
            ),
            // A week -> previous week (could be WHOLE or B)
            (
                WeekNumber { number: 1, part: WeekPart::A, year: 2025 },
                WeekNumber { number: 52, part: WeekPart::WHOLE, year: 2024 },
            ),
            (
                WeekNumber { number: 1, part: WeekPart::A, year: 2026 },
                WeekNumber { number: 52, part: WeekPart::WHOLE, year: 2025 },
            ),
            // WHOLE week -> previous week (could be WHOLE or B)
            (
                WeekNumber { number: 46, part: WeekPart::WHOLE, year: 2024 },
                WeekNumber { number: 45, part: WeekPart::WHOLE, year: 2024 },
            ),
            (
                WeekNumber { number: 2, part: WeekPart::WHOLE, year: 2024 },
                WeekNumber { number: 1, part: WeekPart::WHOLE, year: 2024 },
            ),
            // Year transition: week 1 WHOLE -> previous year's last week
            (
                WeekNumber { number: 1, part: WeekPart::WHOLE, year: 2024 },
                WeekNumber { number: 52, part: WeekPart::WHOLE, year: 2023 },
            ),
            // Week that spans month boundary: A week -> B week of previous week
            // Need to find a week that spans a month boundary
            // Week 1 of 2025 spans Dec 2024 / Jan 2025, so week 1A -> week 52B of 2024
            (
                WeekNumber { number: 1, part: WeekPart::A, year: 2025 },
                WeekNumber { number: 52, part: WeekPart::WHOLE, year: 2024 },
            ),
            (
                WeekNumber { number: 2, part: WeekPart::WHOLE, year: 2026 },
                WeekNumber { number: 1, part: WeekPart::B, year: 2026 },
            ),

        ];

        for (current_week, expected_previous_week) in test_cases {
            let actual_previous = current_week.previous();
            assert_eq!(
                actual_previous, expected_previous_week,
                "previous() of {:?} should be {:?}, but got {:?}",
                current_week, expected_previous_week, actual_previous
            );
        }
    }
}
