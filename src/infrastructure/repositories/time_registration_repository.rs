use super::maconomy_http_client::{ConcurrencyControl, ContainerInstance, MaconomyHttpClient};
use crate::infrastructure::models::time_registration::TimeRegistration;
use anyhow::{anyhow, Context, Result};
use log::info;

// TODO: perhaps rename to TimeRegistrationRepository, and rename TimeRegistrationRepository to TimeRegistrationHttpRepository
pub(crate) struct TimeRegistrationRepository {
    client: MaconomyHttpClient,
    container_instance: Option<ContainerInstance>,
    time_registration: Option<TimeRegistration>,
}

impl TimeRegistrationRepository {
    pub(crate) fn new(repository: MaconomyHttpClient) -> Self {
        Self {
            client: repository,
            container_instance: None,
            time_registration: None,
        }
    }

    /// Gets and caches container instance
    async fn get_container_instance(&mut self) -> Result<ContainerInstance> {
        if self.container_instance.is_none() {
            info!("Fetching container instance");
            let container_instance = self
                .client
                .get_container_instance()
                .await
                .context("Failed to get container instance")?;

            self.container_instance = Some(container_instance);
        } else {
            info!("Using cached container instance")
        }

        let container_instance = self.container_instance.as_ref().ok_or(anyhow!(
            "Missing container instance even though we just fetched it"
        ))?;
        Ok(container_instance.clone())
    }

    // TODO: perhaps rename to `get_week` or something like that?
    pub(crate) async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;
        let (time_registration, concurrency_control) = self
            .client
            .get_time_registration(container_instance)
            .await
            .context("Failed to get time registration")?;

        self.update_concurrency_control(concurrency_control);
        self.time_registration = Some(time_registration.clone());

        Ok(time_registration)
    }

    pub(crate) async fn set_time(&mut self, hours: f32, day: u8, row: u8) -> Result<()> {
        // We need to get the time registration before we can set any data
        let _ = self
            .get_time_registration()
            .await
            .context("Failed to get time registration")?;

        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;

        let concurrency_control = self
            .client
            .set_time(hours, day, row, container_instance)
            .await
            .with_context(|| format!("Failed to set {hours} hours on day {day}, row {row}"))?;

        self.update_concurrency_control(concurrency_control);
        Ok(())
    }

    fn update_concurrency_control(&mut self, concurrency_control: ConcurrencyControl) {
        let container_instance = self.container_instance.as_mut().expect(
            "attempted to update concurrency control with no container instance instantiated",
        );

        container_instance.concurrency_control = concurrency_control;
    }
}
