use anyhow::Context;
use anyhow::{Ok, Result};
use cli::arguments::{parse_arguments, Command};
use cli::commands;
use config::Configuration;
use infrastructure::repositories::maconomy_http_client::MaconomyHttpClient;
use infrastructure::repositories::time_sheet_repository::TimeSheetRepository;
use infrastructure::{auth_service::AuthService, http_service::HttpService};

mod cli;
mod config;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init(); // Enable logging

    let config = Configuration::new();
    let url = config.get_value("maconomy_url")?;
    let company_name = config.get_value("company")?;

    let login_url = config.get_value("authentication.sso.login_url")?;
    let auth_service = AuthService::new(login_url);
    let http_service = HttpService::new(auth_service);
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .context("Failed to create HTTP client")?;

    let client = MaconomyHttpClient::new(url, company_name, client, http_service);
    let mut repository = TimeSheetRepository::new(client);

    match parse_arguments() {
        Command::Get { date: _ } => commands::get(&mut repository).await?,
        Command::Set {
            hours,
            day,
            job: _,
            task: _,
        } => commands::set(hours, day, &mut repository).await?,
        Command::Add {
            hours: _,
            job: _,
            task: _,
            date: _,
        } => todo!(),
    };

    Ok(())
}
