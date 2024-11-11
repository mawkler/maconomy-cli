use crate::domain::models::{day::Days, line_number::LineNumber};
use clap::{Parser, Subcommand};
use color_print::cformat;
use std::str::FromStr;

use super::day_parser::parse_days_of_week;

#[derive(Debug, Clone)]
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
    Delete { line_number: LineNumber },
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get the time sheet for the current week
    Get {
        /// Week number (NOT YET SUPPORTED)
        #[arg(long, short)]
        week: Option<u8>,
        /// Output format (defaults to "table")
        #[arg(long, short)]
        format: Option<Format>,
    },

    /// Set number of hours on some day for a given job and task
    Set {
        // TODO: change to string that allows "4:30" hours, etc.
        /// Number of hours to set
        hours: f32,

        /// Name of the job
        #[arg(long, short)]
        job: String,

        /// Name of the task
        #[arg(long, short)]
        task: String,

        /// Day(s) of the week, for example "tuesday"
        ///
        /// Defaults to today if omitted
        ///
        /// You can also specify multiple days, and/or a range of days, for example
        /// "monday-tuesday, friday"
        ///
        /// Also accepts short day names like "mon", "tue", etc.
        #[arg(long, short, value_parser = parse_days_of_week)]
        day: Option<Days>,
    },

    /// Remove hours hours on some day for a given job and task
    Clear {
        /// Name of the job
        #[arg(long, short)]
        job: String,

        /// Name of the task
        #[arg(long, short)]
        task: String,

        /// Day(s) of the week, for example "tuesday"
        ///
        /// Defaults to today if omitted
        ///
        /// You can also specify multiple days, and/or a range of days, for example
        /// "monday-tuesday, friday"
        ///
        /// Also accepts short day names like "mon", "tue", etc.
        #[arg(long, short, value_parser = parse_days_of_week)]
        day: Option<Days>,
    },

    /// Submit time sheet for week
    Submit,

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
    \n  maconomy set --job '<<job name>>' --task '<<task name>>' --day tuesday 8 \
    \n  maconomy set --job '<<job name>>' --task '<<task name>>' --day 'mon-wed, fri' 8 \
    \n  maconomy clear --job '<<job name>>' --task '<<task name>>' \
    \n\
    \n<bold,underline>NOTE:</bold,underline> currently you can only interact with the current week. In the future you'll be able to specify any week.
    ")
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

pub fn parse_arguments() -> Command {
    Args::parse().command
}
