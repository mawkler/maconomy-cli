use super::arguments::Format;
use crate::domain::models::day::Days;
use crate::domain::models::line_number::LineNumber;
use crate::domain::models::week::WeekNumber;
use crate::{
    domain::time_sheet_service::{SetTimeError, TimeSheetService},
    infrastructure::{
        auth_service::AuthService, repositories::time_sheet_repository::TimeSheetRepository,
    },
    utils::errors::error_stack_fmt,
};
use anyhow::Context;
use chrono::{Datelike, Local};
use std::collections::HashSet;
use std::rc::Rc;
use tokio::sync::Mutex;

macro_rules! exit_with_error {
    ($($arg:tt)*) => {{
        log::warn!("Exiting with error");
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

pub struct CommandClient<'a> {
    pub repository: Rc<Mutex<TimeSheetRepository<'a>>>,
    pub time_sheet_service: Rc<Mutex<TimeSheetService<'a>>>,
    pub auth_service: &'a AuthService,
}

impl<'a> CommandClient<'a> {
    pub fn new(
        repository: Rc<Mutex<TimeSheetRepository<'a>>>,
        time_sheet_service: Rc<Mutex<TimeSheetService<'a>>>,
        auth_service: &'a AuthService,
    ) -> CommandClient<'a> {
        CommandClient {
            repository,
            time_sheet_service,
            auth_service,
        }
    }

    pub(crate) async fn get_table(&self, week: Option<&WeekNumber>) -> anyhow::Result<()> {
        let time_sheet = self.repository.lock().await.get_time_sheet(week).await?;

        println!("{time_sheet}");
        Ok(())
    }

    async fn get_json(&self, week: Option<&WeekNumber>) -> anyhow::Result<()> {
        let time_sheet = self.repository.lock().await.get_time_sheet(week).await?;
        let json =
            serde_json::to_string(&time_sheet).context("Failed to deserialize time sheet")?;

        println!("{json}");
        Ok(())
    }

    pub(crate) async fn get(&self, week: Option<u8>, year: Option<i32>, format: Format) {
        let week = create_week_with_fallback(week, year);

        match format {
            Format::Json => self.get_json(week.as_ref()).await.context("JSON"),
            Format::Table => self.get_table(week.as_ref()).await.context("table"),
        }
        .unwrap_or_else(|err| {
            exit_with_error!("Failed to get time sheet as {}", error_stack_fmt(&err));
        })
    }

    pub(crate) async fn set(
        &mut self,
        hours: f32,
        days: Option<Days>,
        week: Option<u8>,
        year: Option<i32>,
        job: &str,
        task: &str,
    ) {
        if days.as_ref().is_some_and(|days| days.is_empty()) {
            exit_with_error!("`--day` is set but no day was provided");
        }

        let day = get_days(days);
        let week = create_week_with_fallback(week, year);

        self.time_sheet_service
            .lock()
            .await
            .set_time(hours, &day, week.as_ref(), job, task)
            .await
            .unwrap_or_else(|err| {
                if let SetTimeError::Unknown(err) = err {
                    exit_with_error!("{}", error_stack_fmt(&err));
                } else {
                    exit_with_error!("{err}");
                }
            });
    }

    pub(crate) async fn clear(
        &mut self,
        job: &str,
        task: &str,
        days: Option<Days>,
        week: Option<u8>,
        year: Option<i32>,
    ) {
        if days.as_ref().is_some_and(|days| days.is_empty()) {
            exit_with_error!("`--day` is set but no day was provided");
        }

        let week = create_week_with_fallback(week, year);
        self.time_sheet_service
            .lock()
            .await
            .clear(job, task, &get_days(days), week.as_ref())
            .await
            .unwrap_or_else(|err| {
                if let SetTimeError::Unknown(err) = err {
                    exit_with_error!("{}", error_stack_fmt(&err));
                } else {
                    exit_with_error!("{err}");
                }
            });
    }

    pub(crate) async fn logout(&self) {
        self.auth_service.logout().await.unwrap_or_else(|err| {
            exit_with_error!("Logout failed: {}", error_stack_fmt(&err));
        });
    }

    pub(crate) async fn delete(
        &mut self,
        line_number: &LineNumber,
        week: Option<u8>,
        year: Option<i32>,
    ) {
        let week = create_week_with_fallback(week, year);

        self.repository
            .lock()
            .await
            .delete_line(line_number, week.as_ref())
            .await
            .unwrap_or_else(|err| {
                let source = error_stack_fmt(&err);
                exit_with_error!("Failed to delete line {line_number:?}: {source}");
            });
    }

    pub(crate) async fn submit(&mut self) {
        self.repository
            .lock()
            .await
            .submit()
            .await
            .unwrap_or_else(|err| {
                exit_with_error!("Failed to submit: {}", error_stack_fmt(&err));
            });
    }
}

fn get_days(days: Option<Days>) -> Days {
    days.unwrap_or_else(|| {
        // Fall back to today's weekday
        let today = Local::now().date_naive().weekday().into();
        HashSet::from([today])
    })
}

fn create_week_with_fallback(week: Option<u8>, year: Option<i32>) -> Option<WeekNumber> {
    week.map(|week| {
        // Fall back to today's year
        let year = year.unwrap_or_else(|| year.unwrap_or_else(|| chrono::Utc::now().year()));
        WeekNumber::new(week, year).unwrap_or_else(|err| exit_with_error!("{err}"))
    })
}
