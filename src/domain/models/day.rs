use std::{borrow::Borrow, collections::HashSet, fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Day {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

pub(crate) type Days = HashSet<Day>;

impl FromStr for Day {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let day = match s.to_lowercase().borrow() {
            "monday" => Day::Monday,
            "tuesday" => Day::Tuesday,
            "wednesday" => Day::Wednesday,
            "thursday" => Day::Thursday,
            "friday" => Day::Friday,
            "saturday" => Day::Saturday,
            "sunday" => Day::Sunday,
            _ => anyhow::bail!("Unrecognized day {s}"),
        };
        Ok(day)
    }
}

impl From<u8> for Day {
    fn from(day: u8) -> Self {
        let week = [
            Day::Monday,
            Day::Tuesday,
            Day::Wednesday,
            Day::Thursday,
            Day::Friday,
            Day::Saturday,
            Day::Sunday,
        ];
        *week.get(day as usize - 1).expect("Invalid day")
    }
}

impl From<chrono::Weekday> for Day {
    fn from(day: chrono::Weekday) -> Self {
        match day {
            chrono::Weekday::Mon => Self::Monday,
            chrono::Weekday::Tue => Self::Tuesday,
            chrono::Weekday::Wed => Self::Wednesday,
            chrono::Weekday::Thu => Self::Thursday,
            chrono::Weekday::Fri => Self::Friday,
            chrono::Weekday::Sat => Self::Saturday,
            chrono::Weekday::Sun => Self::Sunday,
        }
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let day = match self {
            Day::Monday => "Monday",
            Day::Tuesday => "Tuesday",
            Day::Wednesday => "Wednesday",
            Day::Thursday => "Thursday",
            Day::Friday => "Friday",
            Day::Saturday => "Saturday",
            Day::Sunday => "Sunday",
        };
        write!(f, "{day}")
    }
}
