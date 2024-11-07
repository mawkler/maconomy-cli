use anyhow::{anyhow, Context};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while_m_n},
    combinator::{map, map_res},
    multi::separated_list0,
    sequence::separated_pair,
    Finish, IResult,
};
use std::{fmt::Display, str::FromStr};

type Range = (Day, Day);

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
    // Or perhaps move it to its own module in `cli`
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

fn day_prefix(input: &str) -> IResult<&str, Day> {
    map_res(take_while_m_n(2, 9, char::is_alphabetic), |prefix: &str| {
        let week = [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ];

        week.iter()
            .find(|&day| day.starts_with(prefix))
            .map(|&d| {
                let (_, day) = Day::parse(d).expect("Day validity has already been checked");
                day
            })
            .ok_or("No matching day")
    })(input)
}

enum Item {
    Day(Day),
    Range(Range),
}

impl From<Day> for Item {
    fn from(day: Day) -> Self {
        Item::Day(day)
    }
}

impl From<Range> for Item {
    fn from(range: Range) -> Self {
        Item::Range(range)
    }
}

fn day_range(input: &str) -> IResult<&str, Range> {
    separated_pair(day_prefix, tag("-"), day_prefix)(input)
}

fn parse_items(input: &str) -> IResult<&str, Vec<Item>> {
    let day = map(day_prefix, Item::from);
    let range = map(day_range, Item::from);
    separated_list0(tag(" "), alt((range, day)))(input)
}

fn parse_days(input: &str) -> anyhow::Result<Vec<Day>> {
    let (_, items) = parse_items(input)
        // because of borrow checker limitations for nom together with anyhow
        .map_err(|err| err.to_owned())
        .finish()
        .context("Failed to parse days")?;

    let items: Vec<_> = items
        .into_iter()
        .map(|item| match item {
            Item::Range((start, end)) => Day::range(&start, &end).ok_or(anyhow!("Invalid range")),
            Item::Day(day) => Ok(vec![day]),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(items)
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
        let (_, range) = day_range(range).unwrap();

        let expected = (Day::Monday, Day::Thursday);
        assert_eq!(range, expected);
    }

    #[test]
    fn invalid_range_test() {
        let range = "tue-mon";
        let (_, result) = day_range(range).unwrap();

        let expected = (Day::Tuesday, Day::Monday);
        assert_eq!(result, expected);
    }
}
