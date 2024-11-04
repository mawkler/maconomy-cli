use super::arguments::Format;
use crate::{
    domain::{
        models::{day::Day, line_number::LineNumber},
        time_sheet_service::{SetTimeError, TimeSheetService},
    },
    infrastructure::{
        auth_service::AuthService, repositories::time_sheet_repository::TimeSheetRepository,
    },
    utils::errors::error_stack_fmt,
};
use anyhow::Context;
use chrono::{Datelike, Local};
use log::info;
use std::rc::Rc;
use tokio::sync::Mutex;

macro_rules! exit_with_error {
    ($($arg:tt)*) => {{
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

    // TODO: allow setting week
    pub(crate) async fn get_table(&self, week: Option<u8>) -> anyhow::Result<()> {
        if week.is_some() {
            panic!("--week flag is not yet supported")
        }

        let time_sheet = self.repository.lock().await.get_time_sheet().await?;

        println!("{time_sheet}");
        Ok(())
    }

    async fn get_json(&self, week: Option<u8>) -> anyhow::Result<()> {
        if week.is_some() {
            panic!("--week flag is not yet supported")
        }

        let time_sheet = self.repository.lock().await.get_time_sheet().await?;
        let json =
            serde_json::to_string(&time_sheet).context("Failed to deserialize time sheet")?;

        println!("{json}");
        Ok(())
    }

    pub(crate) async fn get(&self, week: Option<u8>, format: Option<Format>) {
        let format = format.unwrap_or(Format::Table);
        match format {
            Format::Json => self.get_json(week).await.context("JSON"),
            Format::Table => self.get_table(week).await.context("table"),
        }
        .unwrap_or_else(|err| {
            exit_with_error!("Failed to get time sheet as {}", error_stack_fmt(&err));
        })
    }

    pub(crate) async fn set(&mut self, hours: f32, days: Option<&[Day]>, job: &str, task: &str) {
        let day = get_days(days);
        self.time_sheet_service
            .lock()
            .await
            .set_time(hours, &day, job, task)
            .await
            .unwrap_or_else(|err| {
                if let SetTimeError::Unknown(err) = err {
                    exit_with_error!("{}", error_stack_fmt(&err));
                } else {
                    exit_with_error!("{err}");
                }
            });
    }

    pub(crate) async fn clear(&mut self, job: &str, task: &str, days: Option<&[Day]>) {
        self.time_sheet_service
            .lock()
            .await
            .clear(job, task, &get_days(days))
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

    pub(crate) async fn delete(&mut self, line_number: &LineNumber) {
        self.repository
            .lock()
            .await
            .delete_line(line_number)
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

fn get_days(days: Option<&[Day]>) -> Vec<Day> {
    if let Some(day) = days {
        day.to_vec()
    } else {
        // Fall back to today's weekday
        let today = Local::now().date_naive().weekday().to_string();
        let today = today.parse().expect("Failed to parse today's weekday");
        info!("no day passed to 'set', using today's weekday '{today}'");

        vec![today]
    }
}
