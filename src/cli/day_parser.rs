use crate::domain::models::day::{Day, Days};
use std::convert::TryFrom;
use anyhow::{anyhow, Context};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{space0, space1},
    combinator::{map, map_res},
    multi::separated_list0,
    sequence::{delimited, separated_pair},
    Finish, IResult,
};

type Range = (Day, Day);

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

pub(crate) fn parse_days_of_week(input: &str) -> anyhow::Result<Days> {
    let (_, items) = parse_items(input)
        // Because of borrow checker limitations for nom together with anyhow
        .map_err(|err| err.to_owned())
        .finish()
        .context("Failed to parse days")?;

    let days = items
        .into_iter()
        .map(|item| match item {
            Item::Range(range) => days_in_range(range).ok_or(anyhow!("Invalid range")),
            Item::Day(day) => Ok(vec![day]),
        })
        .collect::<Result<Vec<Vec<_>>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(days)
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
            .map(|&d| d.parse().expect("Day validity has already been checked"))
            .ok_or("No matching day")
    })(input)
}

fn day_range(input: &str) -> IResult<&str, Range> {
    let hyphen = delimited(space0, tag("-"), space0);
    separated_pair(day_prefix, hyphen, day_prefix)(input)
}

fn parse_items(input: &str) -> IResult<&str, Vec<Item>> {
    let day = map(day_prefix, Item::from);
    let range = map(day_range, Item::from);
    let comma = delimited(space0, tag(","), space0);
    let separator = alt((comma, space1));

    separated_list0(separator, alt((range, day)))(input)
}

fn days_in_range((start, end): Range) -> Option<Vec<Day>> {
    let (start, end) = (start as u8, end as u8);

    if start < end {
        (start..=end)
            .map(|d| Day::try_from(d).ok())
            .collect()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_parser() {
        let range = "mon-thu";
        let (_, range) = day_range(range).unwrap();

        let expected = (Day::Monday, Day::Thursday);
        assert_eq!(range, expected);
    }

    #[test]
    fn invalid_range_parser() {
        let range = "tue-mon";
        let (_, result) = day_range(range).unwrap();

        let expected = (Day::Tuesday, Day::Monday);
        assert_eq!(result, expected);
    }

    #[test]
    fn individual_days() {
        let days = parse_days_of_week("mon, thu").unwrap();

        let expected = [Day::Monday, Day::Thursday];
        assert_eq!(days, Days::from(expected));
    }

    #[test]
    fn days_and_range() {
        let days = parse_days_of_week("mon, wed, fri-sun").unwrap();

        let expected = [
            Day::Monday,
            Day::Wednesday,
            Day::Friday,
            Day::Saturday,
            Day::Sunday,
        ];
        assert_eq!(days, Days::from(expected));
    }

    #[test]
    fn overlapping_day_ranges() {
        let days = parse_days_of_week("mon, mon-wed").unwrap();

        let expected = [Day::Monday, Day::Tuesday, Day::Wednesday];
        assert_eq!(days, Days::from(expected));
    }

    #[test]
    fn empty_range() {
        let days = parse_days_of_week("").unwrap();

        assert_eq!(days, Days::from([]));
    }

    #[test]
    fn input_with_no_days() {
        let days = parse_days_of_week("foo:bar").unwrap();

        assert_eq!(days, Days::from([]));
    }

    #[test]
    fn invalid_range() {
        let err: String = parse_days_of_week("tue-mon").unwrap_err().to_string();

        assert_eq!(err, "Invalid range");
    }

    #[test]
    fn partial_day_names() {
        // Ensures that "mo", "mon", "mond", etc. all map to `Day::Monday`
        let days = [
            ("monday", Day::Monday),
            ("tuesday", Day::Tuesday),
            ("wednesday", Day::Wednesday),
            ("thursday", Day::Thursday),
            ("friday", Day::Friday),
            ("saturday", Day::Saturday),
            ("sunday", Day::Sunday),
        ];

        for n in 2..=9 {
            days.into_iter()
                .map(move |(input_day, expected_day)| {
                    let input_day: String = input_day.chars().take(n).collect();
                    (input_day, expected_day)
                })
                .for_each(|(input_day, expected_day)| {
                    let days = parse_days_of_week(&input_day).unwrap();

                    assert_eq!(days, Days::from([expected_day]));
                })
        }
    }

    #[test]
    fn gets_days_in_range() {
        let range = (Day::Tuesday, Day::Friday);
        let result = days_in_range(range).unwrap();
        let expected = [Day::Tuesday, Day::Wednesday, Day::Thursday, Day::Friday];

        assert_eq!(result, expected);
    }

    #[test]
    fn days_in_invalid_range() {
        let range = (Day::Tuesday, Day::Monday);
        let result = days_in_range(range);

        assert!(result.is_none());
    }
}
