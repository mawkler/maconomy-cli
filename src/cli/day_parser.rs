use crate::domain::models::day::Day;
use anyhow::{anyhow, Context};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    combinator::{map, map_res},
    multi::separated_list0,
    sequence::separated_pair,
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
    separated_pair(day_prefix, tag("-"), day_prefix)(input)
}

fn parse_items(input: &str) -> IResult<&str, Vec<Item>> {
    let day = map(day_prefix, Item::from);
    let range = map(day_range, Item::from);
    separated_list0(tag(" "), alt((range, day)))(input)
}

fn days_in_range((start, end): &Range) -> Option<Vec<Day>> {
    let (start_int, end_int): (u8, u8) = (start.into(), end.into());

    if start_int < end_int {
        Some((start_int..=end_int).map(Day::from).collect())
    } else {
        None
    }
}

pub(crate) fn parse_days(input: &str) -> anyhow::Result<Vec<Day>> {
    let (_, items) = parse_items(input)
        // Because of borrow checker limitations for nom together with anyhow
        .map_err(|err| err.to_owned())
        .finish()
        .context("Failed to parse days")?;

    let items: Vec<_> = items
        .into_iter()
        .map(|item| match item {
            Item::Range(range) => days_in_range(&range).ok_or(anyhow!("Invalid range")),
            Item::Day(day) => Ok(vec![day]),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_parser_test() {
        let range = "mon-thu";
        let (_, range) = day_range(range).unwrap();

        let expected = (Day::Monday, Day::Thursday);
        assert_eq!(range, expected);
    }

    #[test]
    fn invalid_range_parser_test() {
        let range = "tue-mon";
        let (_, result) = day_range(range).unwrap();

        let expected = (Day::Tuesday, Day::Monday);
        assert_eq!(result, expected);
    }

    #[test]
    fn days_in_range_test() {
        let range = (Day::Tuesday, Day::Friday);
        let result = days_in_range(&range).unwrap();
        let expected = [Day::Tuesday, Day::Wednesday, Day::Thursday, Day::Friday];

        assert_eq!(result, expected);
    }

    #[test]
    fn days_in_invalid_range_test() {
        let range = (Day::Tuesday, Day::Monday);
        let result = days_in_range(&range);

        assert!(result.is_none());
    }
}
