use std::{borrow::Borrow, fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

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

impl From<&Day> for u8 {
    fn from(day: &Day) -> Self {
        day.clone() as u8 + 1
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
        week.get(day as usize - 1).expect("Invalid day").clone()
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
