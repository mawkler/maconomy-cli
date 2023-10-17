use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Log in
    Login {
        /// Username
        #[arg(short, long)]
        username: String,

        /// Password
        #[arg(short, long)]
        password: String,
    },

    /// Get time
    Get { date: String },

    /// Add time
    Add {
        /// Amount of time
        #[arg(short, long)]
        time: u8,

        /// Job
        #[arg(short, long)]
        job: String,

        /// Task
        #[arg(short = 'T', long)]
        task: String,

        /// Date
        #[arg(short, long)]
        date: String,
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
