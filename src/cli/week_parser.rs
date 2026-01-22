use crate::cli::arguments::{WeekAndPart, WeekPart};
use anyhow::{anyhow, Context};
pub(crate) fn parse_week_and_part(input: &str) -> anyhow::Result<WeekAndPart> {
    if input.chars().next_back().is_some_and(|c| c.is_ascii_digit()) {
        // String ends with a digit, parse entire string as week number
        let number = input
            .parse::<u8>()
            .with_context(|| format!("Failed to parse week number from '{}'", input))?;
        Ok(WeekAndPart {
            number: Some(number),
            part: Some(WeekPart::WHOLE),
        })
    } else {
        // String doesn't end with a digit, last character should be A or B
        if input.is_empty() {
            return Err(anyhow!("Week string cannot be empty"));
        }
        let (week_str, part_str) = input.split_at(input.len() - 1);
        let part = part_str
            .parse::<WeekPart>()
            .map_err(|e| anyhow!("Failed to parse week part from '{}', expected 'A' or 'B': {}", part_str, e))?;
        let number = week_str
            .parse::<u8>()
            .with_context(|| format!("Failed to parse week number from '{}'", week_str))?;
        Ok(WeekAndPart {
            number: Some(number),
            part: Some(part),
        })
    }
}
