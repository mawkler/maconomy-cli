use super::maconomy_http_client::{
    self, ConcurrencyControl, ContainerInstance, MaconomyHttpClient,
};
use crate::{
    cli::arguments::WeekPart,
    domain::models::{
        day::Days,
        line_number::LineNumber,
        time_sheet::{Line, TimeSheet, Week},
        week::WeekNumber,
    },
    infrastructure::models::{
        search_response,
        taskname::{self},
        time_registration::{TableRecord, TimeRegistration},
    },
};
use anyhow::{anyhow, Context, Result};
use std::convert::TryFrom;
use chrono::{Datelike, NaiveDate};
use log::{debug, info, trace};
use std::collections::HashSet;

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

pub(crate) struct TimeSheetRepository<'a> {
    client: MaconomyHttpClient<'a>,
    container_instance: Option<ContainerInstance>,
    time_registration: Option<TimeRegistration>,
}

impl TimeSheetRepository<'_> {
    pub(crate) fn new(repository: MaconomyHttpClient) -> TimeSheetRepository {
        TimeSheetRepository {
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

        self.container_instance
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Missing container instance even though we just fetched it"))
    }

    async fn get_time_registration(&mut self) -> Result<TimeRegistration> {
        // Caching
        if let Some(time_registration) = &self.time_registration {
            return Ok(time_registration.clone());
        }

        let container_instance = self.get_container_instance().await?;
        let (time_registration, concurrency_control) = self
            .client
            .get_time_registration(&container_instance)
            .await
            .context("Failed to get time registration")?;

        self.update_concurrency_control(concurrency_control);
        let time_registration_clone = time_registration.clone();
        self.time_registration = Some(time_registration);

        Ok(time_registration_clone)
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
    pub(crate) async fn get_time_sheet(&mut self, week: &WeekNumber) -> Result<TimeSheet> {
        trace!("Incoming week number: {week}");
        // We have to get the time registration before we can set a week
        self.get_time_registration().await?;

        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;

        let date = week
            .first_day()
            .with_context(|| format!("Failed to get first day of week {week}"))?;

        let (time_registration, concurrency_control) = self
            .client
            .set_week(date, &container_instance)
            .await
            .context("Failed to set week")?;

        self.update_concurrency_control(concurrency_control);

        TimeSheet::try_from(time_registration)
    }

    async fn get_or_create_line_number(
        &mut self,
        job: &str,
        task: &str,
        time_sheet: &TimeSheet,
    ) -> Result<u8, AddLineError> {
        match time_sheet.find_line_nr(job, task) {
            Some(line_number) => Ok(line_number),
            None => {
                info!("Found no line for job '{job}', task '{task}'. Creating new line for it");
                let new_time_sheet = self.add_line(job, task).await?;

                new_time_sheet
                    .find_line_nr(job, task)
                    .ok_or_else(|| {
                        anyhow!(
                            "did not find job '{job}' and task '{task}', even after creating a new \
                            line for it"
                        )
                    })
                    .map_err(Into::into)
            }
        }
    }

    pub(crate) async fn set_time(
        &mut self,
        hours: f32,
        days: &Days,
        week: &WeekNumber,
        job: &str,
        task: &str,
    ) -> Result<(), AddLineError> {
        info!("Getting time sheet");
        let time_sheet = self
            .get_time_sheet(week)
            .await
            .context("Failed to get time sheet")?;
        
        let time_sheet = if time_sheet.create_action.is_some() {
            self.create_new_timesheet()
                .await
                .context("Failed to create new timesheet")?;
            self.get_time_sheet(week)
                .await
                .context("Failed to get time sheet after creation")?
        } else {
            debug!("No create_action, timesheet should be populated");
            time_sheet
        };
      
        let line_number = self
            .get_or_create_line_number(job, task, &time_sheet)
            .await?;
        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;

        let days: HashSet<_> = days.iter().map(|&d| d as u8).collect();

        info!("Setting time");
        let (time_registration, concurrency_control) = self
            .client
            .set_time(hours, &days, line_number, &container_instance)
            .await
            .with_context(|| format!("Failed to set {hours} hours on row {line_number}"))?;

        self.time_registration = Some(time_registration);
        self.update_concurrency_control(concurrency_control);
        Ok(())
    }

    fn update_concurrency_control(&mut self, concurrency_control: ConcurrencyControl) {
        if let Some(container_instance) = self.container_instance.as_mut() {
            container_instance.concurrency_control = concurrency_control;
        } else {
            // This should never happen in practice, but we handle it gracefully
            log::warn!("Attempted to update concurrency control with no container instance instantiated");
        }
    }

    async fn get_short_task_name_from_full_task_name(
        &self,
        task_name: &str,
        job: &str,
    ) -> Result<Option<taskname::ShortTaskName>> {
        let tasks = self.get_tasks(job).await.context("Failed to get tasks")?;
        let records = tasks.panes.filter.records;
        // `description` is the long name in this case (i.e. `tasktextvar`)
        let task_name = records
            .iter()
            .find(|row| row.data.description.eq_ignore_ascii_case(task_name))
            .map(|row| taskname::ShortTaskName(row.data.taskname.clone()));

        Ok(task_name)
    }

    async fn add_line(&mut self, job: &str, task: &str) -> Result<TimeSheet, AddLineError> {
        debug!("Getting job number for job '{job}'");
        let job_number = self
            .client
            .get_job_number_from_name(job)
            .await
            .with_context(|| format!("Could not get job number for job '{job}'"))?;

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
        let (time_registration, concurrency_control) = self
            .client
            .add_new_row(&job_number, &task_name, &container_instance)
            .await?;

        self.update_concurrency_control(concurrency_control);
        let time_sheet = TimeSheet::try_from(time_registration.clone())?;
        self.time_registration = Some(time_registration);

        Ok(time_sheet)
    }

    pub(crate) async fn delete_line(
        &mut self,
        line_number: &LineNumber,
        week: &WeekNumber,
    ) -> Result<()> {
        // We need to get the time sheet before we can modify it
        let time_sheet = self
            .get_time_sheet(week)
            .await
            .context("Failed to get time sheet")?;

        let line_number = match line_number {
            LineNumber::Number(line_number) => *line_number,
            LineNumber::Last => {
                let last_line_number = time_sheet.lines.len() as u8;
                info!("Using line number {last_line_number} as last line number");
                last_line_number
            }
        };

        let container_instance = self.get_container_instance().await?;

        let (time_registration, concurrency_control) = self
            .client
            .delete_row(line_number - 1, &container_instance)
            .await
            .with_context(|| format!("Failed to delete line number {line_number}"))?;

        self.update_concurrency_control(concurrency_control);
        self.time_registration = Some(time_registration);

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
            .with_context(|| format!("Failed to get job number for job '{job}'"))?
            .ok_or_else(|| anyhow!("No job number found for job '{job}'"))?;

        debug!("Got job number {job_number} for job {job}");

        self.client
            .get_tasks_for_job(&job_number)
            .await
            .with_context(|| format!("Failed to get tasks for job '{job}'"))
    }

    pub(crate) async fn submit(&mut self, week: &WeekNumber) -> Result<()> {
        // Set the week
        self.get_time_sheet(week)
            .await
            .context("Failed to get time sheet")?;

        let container_instance = self
            .get_container_instance()
            .await
            .context("Failed to get container instance")?;

        let concurrency_control = self
            .client
            .submit(&container_instance)
            .await
            .context("Failed to submit")?;

        self.update_concurrency_control(concurrency_control);

        Ok(())
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

        Line::new(data.jobnumber, data.jobnamevar, data.tasktextvar, week,data.approvalstatus)
    }
}

impl TryFrom<TimeRegistration> for TimeSheet {
    type Error = anyhow::Error;

    fn try_from(time_registration: TimeRegistration) -> Result<Self, Self::Error> {
        let table_records = time_registration.panes.table.records;
        let card_records = time_registration.panes.card.records;

        let lines: Vec<_> = table_records.into_iter().map(Line::from).collect();
        let data = card_records
            .first()
            .ok_or_else(|| anyhow!("time registration contains no records"))?
            .data.clone();
        let w = data.weeknumbervar;
        let part = data
            .partvar.parse::<WeekPart>()
            .unwrap_or(WeekPart::WHOLE);

        let year = NaiveDate::parse_from_str(&data.datevar, "%Y-%m-%d")
            .context("datevar should be in YYYY-MM-DD format")?
            .iso_week()
            .year();
        let week = WeekNumber::new(w, part, year)
            .context("Week number should be valid from time registration")?;
        let create_action = time_registration.panes.card.links.get("action:createtimesheet").map(|l| l.href.clone());
        Ok(Self::new(lines, week, create_action))
    }
}
