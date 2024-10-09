use crate::domain::models::day::Day;
use clap::{Parser, Subcommand};
use color_print::cformat;
use std::str::FromStr;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get the time sheet for the current week
    Get {
        /// Week number (NOT YET SUPPORTED)
        #[arg(short, long)]
        week: Option<u8>,
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
