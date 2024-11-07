use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    combinator::map,
    error::Error as NomError,
    sequence::separated_pair,
    IResult,
};
use std::{fmt::Display, str::FromStr};

#[derive(Debug)]
struct Range(Day, Day);

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseRangeError<'a> {
    #[error("Invalid day '{0}'")]
    Day(&'a str),
    #[error("Invalid range")]
    Range(Day, Day),
    #[error("Invalid input")]
    Input,
}

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

impl Day {
    // TODO: move to `impl Range` instead
    fn parse(input: &str) -> IResult<&str, Day> {
        alt((
            map(tag_no_case("mon"), |_| Day::Monday),
            map(tag_no_case("tue"), |_| Day::Tuesday),
            map(tag_no_case("wed"), |_| Day::Wednesday),
            map(tag_no_case("thu"), |_| Day::Thursday),
            map(tag_no_case("fri"), |_| Day::Friday),
            map(tag_no_case("sat"), |_| Day::Saturday),
            map(tag_no_case("sun"), |_| Day::Sunday),
        ))(input)
    }

    fn range(start: &Day, end: &Day) -> Option<Vec<Day>> {
        let (start_int, end_int): (u8, u8) = (start.into(), end.into());

        if start_int < end_int {
            Some((start_int..=end_int).map(Day::from).collect())
        } else {
            None
        }
    }
}

fn parse_day_range(input: &str) -> Result<Vec<Day>, ParseRangeError> {
    let mut range_parser = separated_pair(Day::parse, tag("-"), Day::parse);
    match range_parser(input) {
        Ok((_, (start, end))) => Day::range(&start, &end).ok_or(ParseRangeError::Range(start, end)),
        Err(_) => Err(ParseRangeError::Input),
    }
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
            anyhow::bail!("Unrecognized day {s}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_range_test() {
        let result = Day::range(&Day::Tuesday, &Day::Friday).unwrap();
        let expected = [Day::Tuesday, Day::Wednesday, Day::Thursday, Day::Friday];

        assert_eq!(result, expected);
    }

    #[test]
    fn range_test() {
        let range = "mon-thu";
        let result = parse_day_range(range).unwrap();

        let expected = [Day::Monday, Day::Tuesday, Day::Wednesday, Day::Thursday];
        assert_eq!(result, expected);
    }

    #[test]
    fn invalid_range_test() {
        let range = "tue-mon";
        let result = parse_day_range(range);

        assert!(matches!(
            ParseRangeError::Range(Day::Tuesday, Day::Monday),
            result
        ));
    }
}
