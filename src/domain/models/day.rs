use std::{collections::HashSet, fmt::Display, str::FromStr};

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
        match s.to_lowercase().as_str() {
            "monday" => Ok(Day::Monday),
            "tuesday" => Ok(Day::Tuesday),
            "wednesday" => Ok(Day::Wednesday),
            "thursday" => Ok(Day::Thursday),
            "friday" => Ok(Day::Friday),
            "saturday" => Ok(Day::Saturday),
            "sunday" => Ok(Day::Sunday),
            _ => anyhow::bail!("Unrecognized day {s}"),
        }
    }
}

impl TryFrom<u8> for Day {
    type Error = &'static str;

    fn try_from(day: u8) -> Result<Self, Self::Error> {
        match day {
            1 => Ok(Day::Monday),
            2 => Ok(Day::Tuesday),
            3 => Ok(Day::Wednesday),
            4 => Ok(Day::Thursday),
            5 => Ok(Day::Friday),
            6 => Ok(Day::Saturday),
            7 => Ok(Day::Sunday),
            _ => Err("Day must be between 1 and 7"),
        }
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
