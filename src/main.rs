use anyhow::Context;
use cli::arguments::{parse_arguments, Command};
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

    let auth_service = Rc::from(AuthService::new(login_url, cookie_path));
    let http_service = HttpService::new(auth_service.clone());
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .context("Failed to create HTTP client")?;

    let client = MaconomyHttpClient::new(url, company_name, client, http_service);
    let repository = Rc::new(Mutex::new(TimeSheetRepository::new(client)));
    let time_sheet_service = Rc::new(Mutex::new(TimeSheetService::new(repository.clone())));
    let mut command_client = CommandClient::new(
        repository.clone(),
        time_sheet_service.clone(),
        auth_service.clone(),
    );

    match parse_arguments() {
        Command::Get { week, format } => command_client.get(week, format).await,
        Command::Set {
            hours,
            day,
            job,
            task,
        } => command_client.set(hours, day, &job, &task).await,
        Command::Clear { job, task, day } => command_client.clear(&job, &task, day).await,
        // TODO: haven't actually tested this yet (can only be tested once a week)
        Command::Submit => command_client.submit().await,
        Command::Logout => command_client.logout().await,
        Command::Line(line) => match line {
            cli::arguments::Line::Delete { line_number } => {
                command_client.delete(&line_number).await
            }
        },
    };

    Ok(())
}
