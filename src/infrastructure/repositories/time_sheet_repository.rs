use super::maconomy_http_client::{
    self, ConcurrencyControl, ContainerInstance, MaconomyHttpClient,
};
use crate::{
    domain::models::{
        day::Day,
        line_number::LineNumber,
        time_sheet::{Line, TimeSheet, Week},
    },
    infrastructure::models::{
        search_response,
        taskname::{self},
        time_registration::{TableRecord, TimeRegistration},
    },
};
use anyhow::{anyhow, Context, Result};
use log::{debug, info};

#[derive(thiserror::Error, Debug)]
pub(crate) enum AddLineError {
    #[error(transparent)]
    WeekUninitialized(#[from] maconomy_http_client::AddRowError),
    #[error("Job '{0}' not found")]
    JobNotFound(String),
    #[error("Task '{0}' not found")]
    TaskNotFound(String),
    #[error("Something went wrong when adding a new line to the time sheet: {0}")]
    Unknown(#[from] anyhow::Error),
}

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

    async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        let container_instance = self.get_container_instance().await?;
        let (time_registration, concurrency_control) = self
            .client
            .get_time_registration(&container_instance)
            .await
            .context("Failed to get time registration")?;

        self.update_concurrency_control(concurrency_control);
        self.time_registration = Some(time_registration.clone());

        Ok(time_registration)
    }

    pub(crate) async fn create_new_timesheet(&mut self) -> Result<()> {
        let container_instance = self.get_container_instance().await?;
        let concurrency_control = self
            .client
            .create_timesheet(&container_instance)
            .await
            .context("Failed to create timesheet")?;

        self.update_concurrency_control(concurrency_control);

        Ok(())
    }

    /// Gets and caches time sheet
    pub(crate) async fn get_time_sheet(&mut self) -> Result<TimeSheet> {
        if let Some(time_registration) = &self.time_registration {
            return Ok(time_registration.clone().into());
        }

        let time_registration = self.get_time_registration().await?;

        Ok(time_registration.into())
    }

    async fn get_or_create_line_number(
        &mut self,
        job: &str,
        task: &str,
        time_sheet: &TimeSheet,
    ) -> Result<u8, AddLineError> {
        let line_number = match time_sheet.find_line_nr(job, task) {
            Some(line_number) => line_number,
            None => {
                info!("Found no line for job '{job}', task '{task}'. Creating new line for it");
                let time_sheet = self.add_line(job, task).await?;

                time_sheet
                    .find_line_nr(job, task)
                    .with_context(|| format!(
                        "format not find job '{job}' and task '{task}', even after creating a new line for it"
                    ))?
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
    ) -> Result<(), AddLineError> {
        let time_sheet = self
            .get_time_sheet()
            .await
            .context("Failed to get time sheet")?;

        let line_number = self
            .get_or_create_line_number(job, task, &time_sheet)
            .await?;
        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;

        let concurrency_control = self
            .client
            .set_time(hours, day.clone().into(), line_number, &container_instance)
            .await
            .with_context(|| {
                format!("Failed to set {hours} hours on day {day}, row {line_number}")
            })?;

        // TODO: also update self.time_registration
        self.update_concurrency_control(concurrency_control);
        Ok(())
    }

    fn update_concurrency_control(&mut self, concurrency_control: ConcurrencyControl) {
        let container_instance = self.container_instance.as_mut().expect(
            "attempted to update concurrency control with no container instance instantiated",
        );

        container_instance.concurrency_control = concurrency_control;
    }

    async fn get_short_task_name_from_full_task_name(
        &self,
        task_name: &str,
        job: &str,
    ) -> Result<Option<taskname::ShortTaskName>> {
        let tasks = self.get_tasks(job).await.context("Failed to get tasks")?;
        let records = tasks.panes.filter.records;
        let task_name = records
            .clone()
            .into_iter()
            // `description` is the long name in this case (i.e. `tasktextvar`)
            .find(|row| row.data.description.to_lowercase() == task_name.to_lowercase())
            .map(|row| taskname::ShortTaskName(row.data.taskname));

        Ok(task_name)
    }

    async fn add_line(&mut self, job: &str, task: &str) -> Result<TimeSheet, AddLineError> {
        debug!("Getting job number for job '{job}'");
        let job_number = self
            .client
            .get_job_number_from_name(job)
            .await
            .context(format!("Could not get job number for job '{job}'"))?;

        let Some(job_number) = job_number else {
            info!("Did not find a job number for {job}");
            return Err(AddLineError::JobNotFound(job.to_string()));
        };
        debug!("Got job number '{job_number}' for job '{job}'");

        let task_name = self
            .get_short_task_name_from_full_task_name(task, job)
            .await?
            .ok_or_else(|| {
                info!("Did not find a long task name for task '{task}'");
                AddLineError::TaskNotFound(task.to_string())
            })?;

        debug!("Adding new line");
        let container_instance = self.get_container_instance().await?;
        let (time_registration, concurrecy_control) = self
            .client
            .add_new_row(&job_number, &task_name, &container_instance)
            .await?;

        self.update_concurrency_control(concurrecy_control);
        self.time_registration = Some(time_registration.clone());

        Ok(time_registration.into())
    }

    async fn get_number_of_lines(&mut self) -> Result<u8> {
        let time_sheet = self.get_time_sheet().await?;
        Ok(time_sheet.lines.len() as u8)
    }

    pub(crate) async fn delete_line(&mut self, line_number: &LineNumber) -> Result<()> {
        // We need to get the time sheet before we can modify it
        let _ = self
            .get_time_sheet()
            .await
            .context("Failed to get time sheet")?;

        let line_number = match line_number {
            LineNumber::Number(line_number) => *line_number,
            LineNumber::Last => {
                let last_line_number = self.get_number_of_lines().await?;
                info!("Using line number {last_line_number} as last line number");
                last_line_number
            }
        };

        let container_instance = self.get_container_instance().await?;

        let (time_registration, concurrecy_control) = self
            .client
            .delete_row(line_number - 1, &container_instance)
            .await
            .with_context(|| format!("Failed to delete line number {line_number}"))?;

        self.update_concurrency_control(concurrecy_control);
        self.time_registration = Some(time_registration.clone());

        Ok(())
    }

    async fn get_tasks(
        &self,
        job: &str,
    ) -> Result<search_response::SearchResponse<search_response::Tasks>> {
        let job_number = self
            .client
            .get_job_number_from_name(job)
            .await
            .context(format!("Failed to get job number for job '{job}'"))?
            .ok_or(anyhow!("No job number found for job '{job}'"))?;

        debug!("Got job number {job_number} for job {job}");

        self.client
            .get_tasks_for_job(&job_number)
            .await
            .with_context(|| format!("Failed to get tasks for job '{job}'"))
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
