use crate::domain::models::{day::Day, line_number::LineNumber};
use clap::{Parser, Subcommand};
use color_print::cformat;
use std::str::FromStr;

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
        #[arg(short, long)]
        week: Option<u8>,
        /// Output format (defaults to "table")
        #[arg(short, long)]
        format: Option<Format>,
    },

    /// Set number of hours on some day for a given job and task
    Set {
        // TODO: change to string that allows "4:30" hours, etc.
        /// Number of hours to set
        hours: f32,

        /// Name of the job
        #[arg(short, long)]
        job: String,

        /// Name of the task
        #[arg(long)]
        task: String,

        /// Day of the week, for example "tuesday"
        ///
        /// Will default to today if omitted
        #[arg(short, long, value_parser = |s: &str| Day::from_str(s))]
        day: Option<Day>,
    },

    /// Remove hours hours on some day for a given job and task
    Clear {
        /// Name of the job
        #[arg(short, long)]
        job: String,

        /// Name of the task
        #[arg(long)]
        task: String,

        /// Day of the week, for example "tuesday"
        #[arg(short, long)]
        day: Option<Day>,
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
    \n  maconomy set --job '<<job name>>' --task '<<task name>>' --day tuesday 8 \
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
