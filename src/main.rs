use crate::infrastructure::time_registration_repository::TimeRegistrationRepository;
use anyhow::{Ok, Result};
use std::env;

mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "url".to_string();
    let company_name = "company name".to_string();
    let username = "username";
    let password = env::var("PASSWORD").expect("Environment variable PASSWORD not found");
    let mut repository = TimeRegistrationRepository::new(url, company_name).unwrap();

    repository.login(username, &password).await?;
    repository.get_container_instance_id().await?;
    let response = repository.get_time_registration().await?;

    println!("response = {:#?}", response);
    Ok(())
}
