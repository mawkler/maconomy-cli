use anyhow::Context;
use cli::arguments::{parse_arguments, Command};
use cli::commands;
use config::Configuration;
use domain::time_sheet_service::TimeSheetService;
use infrastructure::repositories::maconomy_http_client::MaconomyHttpClient;
use infrastructure::repositories::time_sheet_repository::TimeSheetRepository;
use infrastructure::{auth_service::AuthService, http_service::HttpService};
use std::rc::Rc;

mod cli;
mod config;
mod domain;
mod infrastructure;

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
    let mut repository = TimeSheetRepository::new(client);
    let mut time_sheet_service = TimeSheetService::new(&mut repository);

    match parse_arguments() {
        Command::Get { week, format } => commands::get(week, format, &mut repository).await,
        Command::Set {
            hours,
            day,
            job,
            task,
        } => commands::set(hours, day, &job, &task, &mut repository).await,
        Command::Clear { job, task, day } => {
            commands::clear(&job, &task, day, &mut time_sheet_service).await
        }
        Command::Logout => commands::logout(&auth_service.clone()).await,
        Command::Line(line) => match line {
            cli::arguments::Line::Delete { line_number } => {
                commands::delete(&line_number, &mut repository).await
            }
        },
    };

    Ok(())
}
