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
    let url = config.get_value("url")?;
    let company_name = config.get_value("company")?;
    let mut repository = TimeRegistrationRepository::new(url, company_name).unwrap();

    let arguments = parse_arguments();
    println!("arguments = {:#?}", arguments);

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

    let username = config.get_value("username")?;
    let password = config.get_value("password")?;

    repository.login(username, password).await?;
    repository.get_container_instance_id().await?;
    let response = repository.get_time_registration().await?;

    println!("response = {:#?}", response);
    Ok(())
}
