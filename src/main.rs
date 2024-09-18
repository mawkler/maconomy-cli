use crate::{
    cli::arguments::parse_arguments, cli::arguments::Command::*, config::Configuration,
    infrastructure::time_registration_repository::TimeRegistrationRepository,
};
use anyhow::{Context, Ok, Result};
use login::open_login_webpage;
use tao::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

mod cli;
mod config;
mod infrastructure;
mod login;

#[tokio::main]
async fn main() -> Result<()> {
    // let config = Configuration::new();
    // let url = config.get_value("maconomy_url")?;
    // let company_name = config.get_value("company")?;
    // let auth_url = config.get_value("authentication.sso.url")?;
    // let client_id = config.get_value("authentication.sso.client_id")?;
    // let tenant_id = config.get_value("authentication.sso.tenant_id")?;

    // let mut repository = TimeRegistrationRepository::new(url, company_name).unwrap();

    // let arguments = parse_arguments();

    open_login_webpage()?;

    // let _ = repository.login_sso(auth_url, client_id, tenant_id).await;

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
