use super::arguments::{Format, WeekAndPart, WeekPart};
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

    pub(crate) async fn get_table(&self, week: &WeekNumber, full: bool) -> anyhow::Result<()> {
        let time_sheet = self.repository.lock().await.get_time_sheet(week).await?;

        println!("{}", time_sheet.format_table(full));
        Ok(())
    }

    async fn get_json(&self, week: &WeekNumber) -> anyhow::Result<()> {
        let time_sheet = self.repository.lock().await.get_time_sheet(week).await?;
        let json =
            serde_json::to_string(&time_sheet).context("Failed to deserialize time sheet")?;

        println!("{json}");
        Ok(())
    }

    pub(crate) async fn get(&self, week: super::arguments::Week, format: Format, full: bool) {
        let week = get_week_number(week.week, week.year, week.previous);

        match format {
            Format::Json => self.get_json(&week).await.context("JSON"),
            Format::Table => self.get_table(&week, full).await.context("table"),
        }
        .unwrap_or_else(|err| {
            exit_with_error!("Failed to get time sheet as {}", error_stack_fmt(&err));
        })
    }

    pub(crate) async fn set(
        &mut self,
        hours: f32,
        days: &super::arguments::Days,
        task: &super::arguments::Task,
    ) {
        if days.days.as_ref().is_some_and(|days| days.is_empty()) {
            exit_with_error!("`--day` is set but no day was provided");
        }

        let day = get_days(days.days.clone());
        let week = get_week_number(days.week.week.clone(),days.week.year, days.week.previous );

        self.time_sheet_service
            .lock()
            .await
            .set_time(hours, &day, &week, &task.job, &task.name)
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
        task: &super::arguments::Task,
        days: &super::arguments::Days,
    ) {
        if days.days.as_ref().is_some_and(|days| days.is_empty()) {
            exit_with_error!("`--day` is set but no day was provided");
        }

        let week = get_week_number(days.week.week.clone(), days.week.year, days.week.previous);
        self.time_sheet_service
            .lock()
            .await
            .clear(&task.job, &task.name, &get_days(days.days.clone()), &week)
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

    pub(crate) async fn delete(&mut self, line_number: &LineNumber, week: super::arguments::Week) {
        let week = get_week_number(week.week, week.year, week.previous,);

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

    pub(crate) async fn submit(&mut self, week: super::arguments::Week) {
        let week = get_week_number(week.week, week.year,week.previous);

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

fn get_week_number(
    week: Option<WeekAndPart>,
    year: Option<i32>,
    previous_week: Option<u8>,
) -> WeekNumber {
    // NOTE: `week` and `previous_week` are assumed to be mutually exclusive (handled by Clap)
    if let Some(week) = previous_week {
        nth_previous_week(week).unwrap_or_else(|err| {
            exit_with_error!("{err}");
        })
    } else {
        let y = year.unwrap_or_else(|| chrono::Utc::now().year());
        week.map(|week_and_part| {
            let number = week_and_part.number.ok_or_else(|| anyhow::anyhow!("Week number is required"))?;
            let part = week_and_part.part.unwrap_or(WeekPart::WHOLE);
            WeekNumber::new(number, part, y)
        })
        .unwrap_or_else(|| Ok(WeekNumber::default()))
        .unwrap_or_else(|err| {exit_with_error!("{err}");})
    }
}

fn get_days(days: Option<Days>) -> Days {
    days.unwrap_or_else(|| {
        // Fall back to today's weekday
        info!("Using today's as day");
        let today = chrono::Local::now().date_naive().weekday().into();
        HashSet::from([today])
    })
}

fn nth_previous_week(n: u8) -> anyhow::Result<WeekNumber> {
    let mut week = WeekNumber::of(chrono::Local::now().date_naive());
    
    for _ in 0..n {
        week = week.previous();
    }
    
    Ok(week)
}
