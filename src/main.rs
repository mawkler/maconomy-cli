use crate::{
    cli::arguments::parse_arguments, cli::arguments::Command::*, config::Configuration,
    infrastructure::time_registration_repository::TimeRegistrationRepository,
};
use anyhow::{Ok, Result};

mod cli;
mod config;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Configuration::new();
    let url = config.get_value("maconomy_url")?;
    let company_name = config.get_value("company")?;
    let auth_url = config.get_value("authentication.sso.url")?;
    let client_id = config.get_value("authentication.sso.client_id")?;
    let tenant_id = config.get_value("authentication.sso.tenant_id")?;

    let login_url = config.get_value("authentication.sso.login_url")?;
    let auth_service = AuthService::new(login_url, config);
    let http_service = HttpService::new(auth_service);

    let mut repository = TimeRegistrationRepository::new(url, company_name, http_service).unwrap();
    repository.get_container_instance_id().await.unwrap();

    // let arguments = parse_arguments();

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
