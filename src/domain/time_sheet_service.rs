use crate::domain::models::day::Day;
use crate::infrastructure::repositories::maconomy_http_client::AddRowError;
use crate::infrastructure::repositories::time_sheet_repository::{
    AddLineError, TimeSheetRepository,
};
use anyhow::Result;
use log::{info, warn};

#[derive(thiserror::Error, Debug)]
pub(crate) enum SetTimeError {
    #[error("Job '{0}' not found")]
    JobNotFound(String),
    #[error("Task '{0}' not found")]
    TaskNotFound(String),
    #[error("Something went wrong when setting hours: {0}")]
    Unknown(#[from] anyhow::Error),
    // TODO: handle authentication error
}

pub(crate) struct TimeSheetService<'a> {
    repository: &'a mut TimeSheetRepository,
}

impl<'a> TimeSheetService<'a> {
    pub(crate) fn new(repository: &'a mut TimeSheetRepository) -> Self {
        Self { repository }
    }
}

impl TimeSheetService<'_> {
    pub(crate) async fn clear(
        &mut self,
        job: &str,
        task: &str,
        day: &Day,
    ) -> Result<(), SetTimeError> {
        self.set_time(0.0, day, job, task).await
    }

    /// Sets time (initializes the week if it is uninitialized)
    pub(crate) async fn set_time(
        &mut self,
        hours: f32,
        day: &Day,
        job: &str,
        task: &str,
    ) -> Result<(), SetTimeError> {
        if let Err(err) = self.repository.set_time(hours, day, job, task).await {
            return match err {
                AddLineError::WeekUninitialized(AddRowError::Unknown(err)) => todo!("{}", err),
                AddLineError::WeekUninitialized(AddRowError::WeekUninitialized) => {
                    info!("Creating new timesheet");

                    self.repository.create_new_timesheet().await?;

                    self.repository
                        .set_time(hours, day, job, task)
                        .await
                        .map_err(|err| {
                            let msg = format!(
                                "Failed to set time, even after creating a new timesheet: {err}"
                            );
                            warn!("{msg}");
                            anyhow::anyhow!(msg)
                        })?;

                    Ok(())
                }
                AddLineError::JobNotFound(err) => Err(SetTimeError::JobNotFound(err)),
                AddLineError::TaskNotFound(err) => Err(SetTimeError::TaskNotFound(err)),
                err => {
                    warn!("{err}");
                    Err(anyhow::anyhow!(err).into())
                }
            };
        };

        Ok(())
    }
}
