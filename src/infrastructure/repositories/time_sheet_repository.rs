use super::maconomy_http_client::{ConcurrencyControl, ContainerInstance, MaconomyHttpClient};
use crate::{
    domain::models::{
        day::Day,
        time_sheet::{Line, TimeSheet, Week},
    },
    infrastructure::models::time_registration::{TableRecord, TimeRegistration},
};
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};

pub(crate) struct TimeSheetRepository {
    client: MaconomyHttpClient,
    container_instance: Option<ContainerInstance>,
    time_registration: Option<TimeRegistration>,
}

impl TimeSheetRepository {
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

    /// Gets and caches time sheet
    pub(crate) async fn get_time_sheet(&mut self) -> Result<TimeSheet> {
        if let Some(time_registration) = &self.time_registration {
            return Ok(time_registration.clone().into());
        }

        let container_instance = self.get_container_instance().await?;
        let (time_registration, concurrency_control) = self
            .client
            .get_time_registration(container_instance)
            .await
            .context("Failed to get time sheet")?;

        self.update_concurrency_control(concurrency_control);
        self.time_registration = Some(time_registration.clone());

        Ok(time_registration.into())
    }

    async fn get_or_create_line_number(&mut self, job: &str, task: &str) -> Result<u8> {
        let time_sheet = self
            .get_time_sheet()
            .await
            .context("Failed to get time sheet")?;
        let line_number = match time_sheet.find_line_nr(job, task) {
            Some(line_number) => line_number,
            None => {
                info!("Found no line for job {job}, task {task}. Creating new line for it");
                let time_sheet = self.add_new_line(job, task).await.with_context(|| {
                    format!("Failed to add new line to time sheet for job {job} and task {task}")
                })?;

                time_sheet
                    .find_line_nr(job, task)
                    .unwrap_or_else(|| panic!("Could not find job {job}, task {task} even after creating a new line for it"))
            }
        };

        Ok(line_number)
    }

    pub(crate) async fn set_time(
        &mut self,
        hours: f32,
        day: &Day,
        job: &str,
        task: &str,
    ) -> Result<()> {
        let line_number = self.get_or_create_line_number(job, task).await?;
        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;

        let concurrency_control = self
            .client
            .set_time(hours, day.clone().into(), line_number, container_instance)
            .await
            .with_context(|| {
                format!("Failed to set {hours} hours on day {day}, row {line_number}")
            })?;

        self.update_concurrency_control(concurrency_control);
        Ok(())
    }

    fn update_concurrency_control(&mut self, concurrency_control: ConcurrencyControl) {
        let container_instance = self.container_instance.as_mut().expect(
            "attempted to update concurrency control with no container instance instantiated",
        );

        container_instance.concurrency_control = concurrency_control;
    }

    pub async fn add_new_line(&mut self, job: &str, task: &str) -> Result<TimeSheet> {
        // We need to get the time sheet before we can create a new line for some reason
        let _ = self
            .get_time_sheet()
            .await
            .context("Failed to get time sheet")?;

        debug!("Getting job number for job '{job}'");
        let job_number = self
            .client
            .get_job_number_from_name(job)
            .await
            .context(format!("Could not get job number for job '{job}'"))?;

        let Some(job_number) = job_number else {
            bail!("Job '{job}' not found");
        };
        debug!("Got job number '{job_number}' for job '{job}'");

        let container_instance = self.get_container_instance().await?;

        debug!("Adding new row");
        let (time_registration, concurrecy_control) = self
            .client
            .add_new_row(&job_number, task, container_instance)
            .await
            .context("Failed to add new row")?;

        self.update_concurrency_control(concurrecy_control);
        dbg!(&time_registration);
        self.time_registration = Some(time_registration.clone());

        Ok(time_registration.into())
    }
}

impl From<TableRecord> for Line {
    fn from(table_record: TableRecord) -> Self {
        let data = table_record.data;
        let week = Week {
            monday: data.numberday1.into(),
            tuesday: data.numberday2.into(),
            wednesday: data.numberday3.into(),
            thursday: data.numberday4.into(),
            friday: data.numberday5.into(),
            saturday: data.numberday6.into(),
            sunday: data.numberday7.into(),
        };

        Line::new(data.jobnamevar, data.tasktextvar, week)
    }
}

impl From<TimeRegistration> for TimeSheet {
    fn from(time_registration: TimeRegistration) -> Self {
        let lines: Vec<_> = time_registration
            .panes
            .table
            .records
            .into_iter()
            .map(Line::from)
            .collect();
        Self::new(lines)
    }
}
