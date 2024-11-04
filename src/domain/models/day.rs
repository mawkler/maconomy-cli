use anyhow::bail;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
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
        let day = s.to_lowercase();
        let day = if day.starts_with("mon") {
            Day::Monday
        } else if day.starts_with("tue") {
            Day::Tuesday
        } else if day.starts_with("wed") {
            Day::Wednesday
        } else if day.starts_with("thu") {
            Day::Thursday
        } else if day.starts_with("fri") {
            Day::Friday
        } else if day.starts_with("sat") {
            Day::Saturday
        } else if day.starts_with("sun") {
            Day::Sunday
        } else {
            bail!("Unrecognized day {s}");
        };

        Ok(day)
    }
}

impl From<&Day> for u8 {
    fn from(day: &Day) -> Self {
        day.clone() as u8 + 1
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
