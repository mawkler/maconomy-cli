use std::str::FromStr;

use clap::{Parser, Subcommand};

use crate::domain::models::day::{self, Day};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get time
    Get { date: Option<String> },

    Set {
        // TODO: change to string that allows "4:30" hours, etc.
        /// Number of hours to set
        hours: f32,

        /// Day of the week
        #[arg(short, long, value_parser = |s: &str| Day::from_str(s))]
        day: Option<day::Day>,

        /// Job
        #[arg(short, long)]
        job: String,

        /// Task
        #[arg(long)]
        task: String,
    },

    /// Add time
    Add {
        /// Amount of time
        #[arg(short, long)]
        hours: u8,

        /// Job
        #[arg(short, long)]
        job: String,

        /// Task
        #[arg(long)]
        task: String,

        /// Date
        #[arg(short, long)]
        date: Option<String>,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

pub fn parse_arguments() -> Command {
    Args::parse().command
}
