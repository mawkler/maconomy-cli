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
use chrono::Datelike;
use log::info;
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

    pub(crate) async fn get_table(&self, week: &WeekNumber) -> anyhow::Result<()> {
        let time_sheet = self.repository.lock().await.get_time_sheet(week).await?;

        println!("{time_sheet}");
        Ok(())
    }

    async fn get_json(&self, week: &WeekNumber) -> anyhow::Result<()> {
        let time_sheet = self.repository.lock().await.get_time_sheet(week).await?;
        let json =
            serde_json::to_string(&time_sheet).context("Failed to deserialize time sheet")?;

        println!("{json}");
        Ok(())
    }

    pub(crate) async fn get(&self, week: Option<String>, year: Option<i32>, format: Format) {
        let week =
            get_week_with_fallback(week, year).unwrap_or_else(|err| exit_with_error!("{err}"));

        match format {
            Format::Json => self.get_json(&week).await.context("JSON"),
            Format::Table => self.get_table(&week).await.context("table"),
        }
        .unwrap_or_else(|err| {
            exit_with_error!("Failed to get time sheet as {}", error_stack_fmt(&err));
        })
    }

    pub(crate) async fn set(
        &mut self,
        hours: f32,
        days: Option<Days>,
        week: Option<String>,
        year: Option<i32>,
        job: &str,
        task: &str,
    ) {
        if days.as_ref().is_some_and(|days| days.is_empty()) {
            exit_with_error!("`--day` is set but no day was provided");
        }

        let day = get_days(days);
        let week =
            get_week_with_fallback(week, year).unwrap_or_else(|err| exit_with_error!("{err}"));

        self.time_sheet_service
            .lock()
            .await
            .set_time(hours, &day, &week, job, task)
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
        week: Option<String>,
        year: Option<i32>,
    ) {
        if days.as_ref().is_some_and(|days| days.is_empty()) {
            exit_with_error!("`--day` is set but no day was provided");
        }

        let week =
            get_week_with_fallback(week, year).unwrap_or_else(|err| exit_with_error!("{err}"));
        self.time_sheet_service
            .lock()
            .await
            .clear(job, task, &get_days(days), &week)
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
        week: Option<String>,
        year: Option<i32>,
    ) {
        let week =
            get_week_with_fallback(week, year).unwrap_or_else(|err| exit_with_error!("{err}"));

        self.repository
            .lock()
            .await
            .delete_line(line_number, &week)
            .await
            .unwrap_or_else(|err| {
                let source = error_stack_fmt(&err);
                exit_with_error!("Failed to delete line {line_number:?}: {source}");
            });
    }

    pub(crate) async fn submit(&mut self, week: Option<String>, year: Option<i32>) {
        let week =
            get_week_with_fallback(week, year).unwrap_or_else(|err| exit_with_error!("{err}"));

        self.repository
            .lock()
            .await
            .submit(&week)
            .await
            .unwrap_or_else(|err| {
                exit_with_error!("Failed to submit: {}", error_stack_fmt(&err));
            });
    }
}

fn get_days(days: Option<Days>) -> Days {
    days.unwrap_or_else(|| {
        // Fall back to today's weekday
        let today = chrono::Local::now().date_naive().weekday().into();
        HashSet::from([today])
    })
}

// TODO: refactor this so that it lives on WeekNumber and parse_week calls this instead of
// vice-versa
fn get_week_with_fallback(week: Option<String>, year: Option<i32>) -> anyhow::Result<WeekNumber> {
    if let Some(week) = week {
        let week = parse_week(week)?;

        let week = WeekNumber::new_with_fallback(week, year)?;
        Ok(week)
    } else {
        let week = chrono::Local::now().date_naive().iso_week().week();
        let week = week
            .try_into()
            .expect("Week numbers are always less than 255");

        WeekNumber::new_with_fallback(week, year)
    }
}

fn parse_week(week: String) -> anyhow::Result<u8> {
    let week = match week.trim().to_lowercase().as_str() {
        "previous" => {
            let today_week_last = chrono::Local::now().date_naive() - chrono::Duration::weeks(1);
            let week = today_week_last.iso_week().week();
            info!("Using previous week number: {week}");
            Ok(week)
        }
        number => number
            .parse()
            .with_context(|| format!("Invalid week number '{week}'")),
    }?
    .try_into()
    .expect("Week numbers are always less than 255");

    Ok(week)
}
