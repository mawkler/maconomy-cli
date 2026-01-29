use super::day_parser::parse_days_of_week;
use crate::cli::week_parser::parse_week_and_part;
use crate::domain::models::{day, line_number::LineNumber};
use clap::{Parser, Subcommand, ValueEnum};
use color_print::cformat;
use serde::Serialize;
use std::str::FromStr;

#[derive(Parser, ValueEnum, Default, Debug, Clone, Serialize, Copy, PartialEq, Eq)]
pub(crate) enum WeekPart {
    #[default]
    #[serde(rename = "")]
    WHOLE,
    A,
    B,
}
impl FromStr for WeekPart {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "a" => Ok(WeekPart::A),
            "b" => Ok(WeekPart::B),
            part => Err(format!("Invalid week part '{part}'")),
        }
    }
}
#[derive(Parser, Debug)]
pub(crate) struct Week {
    /// Week number (defaults to current week if omitted)
    #[arg(long = "week", short = 'w',value_parser = parse_week_and_part )]
    pub(crate) week: Option<WeekAndPart>,
    /// N:th previous week counted from current week (defaults to 1 if N is omitted)
    #[arg(
        long = "previous-week",
        short,
        value_name = "N",
        conflicts_with = "week",
        default_missing_value = Some("1"),
        num_args(0..=1),
    )]
    pub(crate) previous: Option<u8>,

    /// Year (defaults to current year if omitted)
    #[arg(long, short, requires = "week")]
    pub(crate) year: Option<i32>,
}

#[derive(Debug, Clone)]
pub(crate) struct WeekAndPart {
    pub(crate) number: Option<u8>,
    pub(crate) part: Option<WeekPart>,
}

#[derive(Parser, Debug)]
pub(crate) struct Task {
    /// Name of the job
    #[arg(long, short)]
    pub(crate) job: String,

    /// Name of the task
    #[arg(long = "task", short = 't')]
    pub(crate) name: String,
}

#[derive(Parser, Debug)]
pub(crate) struct Days {
    /// Day(s) of the week, for example "tuesday"
    ///
    /// Defaults to today if omitted
    ///
    /// You can also specify multiple days, and/or a range of days, for example
    /// "monday-tuesday, friday"
    ///
    /// Also accepts short day names like "mon", "tue", etc.
    #[arg(long = "day", short, value_parser = parse_days_of_week)]
    pub(crate) days: Option<day::Days>,

    #[command(flatten)]
    pub(crate) week: Week,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub(crate) enum Format {
    Json,
    Table,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Format::Json),
            "table" => Ok(Format::Table),
            format => Err(format!("Invalid format '{format}'")),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Line {
    /// Delete line based on line number (1-indexed)
    Delete {
        line_number: LineNumber,

        #[command(flatten)]
        week: Week,
    },
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get the time sheet for the current week
    Get {
        /// Output format
        #[arg(long, short, default_value = "table")]
        format: Format,

        /// Show all rows, including those with no hours reported
        #[arg(long)]
        full: bool,

        #[command(flatten)]
        week: Week,
    },

    /// Set number of hours on day(s) for a given job and task
    Set {
        // TODO: change to string that allows "4:30" hours, etc.
        /// Number of hours to set
        hours: f32,

        #[command(flatten)]
        task: Task,

        #[command(flatten)]
        days: Days,
    },

    /// Remove hours on day(s) for a given job and task
    Clear {
        #[command(flatten)]
        task: Task,

        #[command(flatten)]
        days: Days,
    },

    /// Submit time sheet for week
    Submit {
        #[command(flatten)]
        week: Week,
    },

    /// Log out
    Logout,

    /// Operate on entire lines in the time sheet
    #[command(subcommand)]
    Line(Line),
}

#[derive(Parser, Debug)]
#[command(
    author = "Melker Ulander",
    about,
    version,
    arg_required_else_help = true,
    after_help = cformat!("<bold,underline>Examples:</bold,underline>\
    \n  maconomy get \
    \n  maconomy get --full\
    \n  maconomy set 8 --job '<<job name>>' --task '<<task name>>' \
    \n  maconomy set 8 --job '<<job name>>' --task '<<task name>>' --day 'mon-wed, fri' --week 46 \
    \n  maconomy set 8 --job '<<job name>>' --task '<<task name>>' --day mo --previous-week 2 \
    \n  maconomy clear --job '<<job name>>' --task '<<task name>>' --day tuesday \
    \n  maconomy line delete 2 \
    ")
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

pub fn parse_arguments() -> Command {
    Args::parse().command
}
