use crate::domain::models::day::Day;
use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Get time
    Get { week: Option<u32> },

    Set {
        // TODO: change to string that allows "4:30" hours, etc.
        /// Number of hours to set
        hours: f32,

        /// Job
        #[arg(short, long)]
        job: String,

        /// Task
        #[arg(long)]
        task: String,

        /// Day of the week
        #[arg(short, long, value_parser = |s: &str| Day::from_str(s))]
        day: Option<Day>,
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

        /// Day of the week
        #[arg(short, long)]
        day: Option<String>,
    },

    Clear {
        /// Job
        #[arg(short, long)]
        job: String,

        /// Task
        #[arg(long)]
        task: String,

        /// Day of the week
        #[arg(short, long)]
        day: Option<Day>,
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
