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

pub struct CommandClient {
    pub repository: Rc<Mutex<TimeSheetRepository>>,
    pub time_sheet_service: Rc<Mutex<TimeSheetService>>,
    pub auth_service: Rc<AuthService>,
}

impl CommandClient {
    pub fn new(
        repository: Rc<Mutex<TimeSheetRepository>>,
        time_sheet_service: Rc<Mutex<TimeSheetService>>,
        auth_service: Rc<AuthService>,
    ) -> CommandClient {
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
            Format::Json => self
                .get_json(week)
                .await
                .unwrap_or_else(|err| eprintln!("Failed to get time sheet as JSON: {err}")),
            Format::Table => self
                .get_table(week)
                .await
                .unwrap_or_else(|err| eprintln!("Failed to get time sheet as table: {err}")),
        }
    }

    pub(crate) async fn set(&mut self, hours: f32, day: Option<Day>, job: &str, task: &str) {
        let day = get_day(day);
        self.time_sheet_service
            .lock()
            .await
            .set_time(hours, &day, job, task)
            .await
            .unwrap_or_else(|err| {
                if let SetTimeError::Unknown(err) = err {
                    println!("{}", error_stack_fmt(&err));
                } else {
                    eprintln!("{err}");
                }
            });
    }

    pub(crate) async fn clear(&mut self, job: &str, task: &str, day: Option<Day>) {
        self.time_sheet_service
            .lock()
            .await
            .clear(job, task, &get_day(day))
            .await
            .unwrap_or_else(|err| {
                if let SetTimeError::Unknown(err) = err {
                    println!("{}", error_stack_fmt(&err));
                } else {
                    eprintln!("{err}");
                }
            });
    }

    pub(crate) async fn logout(&self) {
        self.auth_service
            .logout()
            .await
            .context("Logout failed")
            .unwrap_or_else(|err| {
                eprintln!("Logout failed: {err}");
            });
    }

    pub(crate) async fn delete(&mut self, line_number: &LineNumber) {
        self.repository
            .lock()
            .await
            .delete_line(line_number)
            .await
            .unwrap_or_else(|err| {
                eprintln!("Failed to delete line {line_number:?}: {err}");
            });
    }

    pub(crate) async fn submit(&mut self) {
        self.repository
            .lock()
            .await
            .submit()
            .await
            .unwrap_or_else(|err| {
                eprintln!("Failed to submit: {err}");
            });
    }
}

fn get_day(day: Option<Day>) -> Day {
    if let Some(day) = day {
        day
    } else {
        // Fall back to today's weekday
        let today = Local::now().date_naive().weekday().to_string();
        let today = today.parse().expect("Failed to parse today's weekday");
        info!("no day passed to 'set', using today's weekday '{today}'");

        today
    }
}
