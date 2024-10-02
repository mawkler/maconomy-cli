use crate::{
    config::Configuration, infrastructure::time_registration_repository::TimeRegistrationRepository,
};
use anyhow::Context;
use anyhow::{Ok, Result};
use cli::arguments::{parse_arguments, Command};
use infrastructure::{auth_service::AuthService, http_service::HttpService};

mod cli;
mod config;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init(); // Enable logging

    let config = Configuration::new();
    let url = config.get_value("maconomy_url")?;
    let company_name = config.get_value("company")?;

    let login_url = config.get_value("authentication.sso.login_url")?;
    let auth_service = AuthService::new(login_url, config);
    let http_service = HttpService::new(auth_service);
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .context("Failed to create HTTP client")?;

    let mut repository = TimeRegistrationRepository::new(url, company_name, client, http_service);

    match parse_arguments() {
        Command::Get { date: _ } => {
            cli::commands::get(&mut repository).await?;
        }
        Command::Add {
            time: _,
            job: _,
            task: _,
            date: _,
        } => todo!(),
    };

    Ok(())
}
