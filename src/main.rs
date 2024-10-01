use crate::{
    config::Configuration, infrastructure::time_registration_repository::TimeRegistrationRepository,
};
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

    let mut repository = TimeRegistrationRepository::new(url, company_name, http_service).unwrap();

    match parse_arguments() {
        Command::Get { date: _ } => {
            cli::commands::get(&mut repository).await;
        }
        Command::Add {
            time: _,
            job: _,
            task: _,
            date: _,
        } => todo!(),
    };

    // match arguments {
    //     Login {
    //         username: _,
    //         password: _,
    //     } => todo!(),
    //     Get { date: _ } => todo!(),
    //     Add {
    //         time: _,
    //         job: _,
    //         task: _,
    //         date: _,
    //     } => todo!(),
    // };

    // let username = config.get_value("username")?;
    // let password = config.get_value("password")?;

    // repository.login(username, password).await?;
    // repository.get_container_instance_id().await?;
    // let response = repository.get_time_registration().await?;

    // println!("response = {:#?}", response);
    Ok(())
}
