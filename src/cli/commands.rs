use crate::infrastructure::time_registration_repository::TimeRegistrationRepository;
use anyhow::{Context, Result};

// TODO: allow setting date/week
pub(crate) async fn get(repository: &mut TimeRegistrationRepository) -> Result<()> {
    let time_registration = repository
        .get_time_registration()
        .await
        .context("failed to get time registration")?;

    println!("{time_registration}");
    Ok(())
}
