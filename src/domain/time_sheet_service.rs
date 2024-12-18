use crate::infrastructure::repositories::maconomy_http_client::AddRowError;
use crate::infrastructure::repositories::time_sheet_repository::{
    AddLineError, TimeSheetRepository,
};
use anyhow::Result;
use log::warn;
use std::rc::Rc;
use tokio::sync::Mutex;

use super::models::day::Days;
use super::models::week::WeekNumber;

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
    repository: Rc<Mutex<TimeSheetRepository<'a>>>,
}

impl TimeSheetService<'_> {
    pub(crate) fn new(repository: Rc<Mutex<TimeSheetRepository>>) -> TimeSheetService {
        TimeSheetService { repository }
    }
}

impl TimeSheetService<'_> {
    pub(crate) async fn clear(
        &mut self,
        job: &str,
        task: &str,
        days: &Days,
        week: &WeekNumber,
    ) -> Result<(), SetTimeError> {
        self.set_time(0.0, days, week, job, task).await
    }

    /// Sets time (initializes the week if it is uninitialized)
    pub(crate) async fn set_time(
        &mut self,
        hours: f32,
        days: &Days,
        week: &WeekNumber,
        job: &str,
        task: &str,
    ) -> Result<(), SetTimeError> {
        let mut repository = self.repository.lock().await;
        if let Err(err) = repository.set_time(hours, days, week, job, task).await {
            return match err {
                AddLineError::WeekUninitialized(AddRowError::Unknown(err)) => todo!("{}", err),
                AddLineError::WeekUninitialized(AddRowError::WeekUninitialized) => {
                    eprintln!("Creating new timesheet...");

                    repository.create_new_timesheet().await?;

                    repository
                        .set_time(hours, days, week, job, task)
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
