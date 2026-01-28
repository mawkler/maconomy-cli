use anyhow::Context;
use cli::arguments::{parse_arguments, Command, Line};
use cli::commands::CommandClient;
use config::Configuration;
use domain::time_sheet_service::TimeSheetService;
use infrastructure::repositories::maconomy_http_client::MaconomyHttpClient;
use infrastructure::repositories::time_sheet_repository::TimeSheetRepository;
use infrastructure::{auth_service::AuthService, http_service::HttpService};
use std::rc::Rc;
use tokio::sync::Mutex;

mod cli;
mod config;
mod domain;
mod infrastructure;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init(); // Enable logging

    let config = Configuration::new();
    let url = config.get_value("maconomy_url")?;
    let company_name = config.get_value("company_id")?;

    let login_url = config.get_value("authentication.sso.login_url")?;
    let cookie_path = config
        .get_optional_value("authentication.sso.cookie_path")?
        .unwrap_or("~/.local/share/maconomy-cli/maconomy_cookie".to_string());

    let auth_service = AuthService::new(login_url, cookie_path);
    let http_service = HttpService::new(&auth_service);
    let client = reqwest::Client::builder()
        .connection_verbose(true)
        .cookie_store(true)
        .build()
        .context("Failed to create HTTP client")?;

    let client = MaconomyHttpClient::new(url, company_name, client, http_service);
    let repository = Rc::new(Mutex::new(TimeSheetRepository::new(client)));
    let time_sheet_service = Rc::new(Mutex::new(TimeSheetService::new(repository.clone())));
    let mut command_client = CommandClient::new(
        repository.clone(),
        time_sheet_service.clone(),
        &auth_service,
    );

    match parse_arguments() {
        Command::Get { week, format, full } => command_client.get(week, format, full).await,
        Command::Set { hours, task, days } => command_client.set(hours, &days, &task).await,
        Command::Clear { task, days } => command_client.clear(&task, &days).await,
        Command::Submit { week } => command_client.submit(week).await,
        Command::Logout => command_client.logout().await,
        Command::Line(line) => match line {
            Line::Delete { line_number, week } => command_client.delete(&line_number, week).await,
        },
    };

    Ok(())
}
