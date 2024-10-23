use crate::{
    domain::{
        models::{day::Day, line_number::LineNumber},
        time_sheet_service::TimeSheetService,
    },
    infrastructure::{
        auth_service::AuthService,
        repositories::time_sheet_repository::{self, TimeSheetRepository},
    },
};
use anyhow::Context;
use chrono::{Datelike, Local};
use log::{error, info};

// TODO: allow setting week
pub(crate) async fn get(week: Option<u8>, repository: &mut TimeSheetRepository) {
    if week.is_some() {
        panic!("--week flag is not yet supported")
    }

    let time_sheet = match repository.get_time_sheet().await {
        Ok(time_sheet) => time_sheet,
        Err(err) => {
            eprintln!("Failed to get time sheet: {err}");
            return;
        }
    };

    println!("{time_sheet}");
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

#[derive(thiserror::Error, Debug)]
pub(crate) enum SetTimeError {
    #[error(transparent)]
    Known(#[from] time_sheet_repository::SetTimeError),
    #[error("Something went wrong when setting hours")]
    Unknown(#[from] anyhow::Error),
}

pub(crate) async fn set(
    hours: f32,
    day: Option<Day>,
    job: &str,
    task: &str,
    repository: &mut TimeSheetRepository,
) {
    let day = get_day(day);
    repository
        .set_time(hours, &day, job, task)
        .await
        .unwrap_or_else(|err| {
            eprintln!("{err}");
        });
}

pub(crate) async fn clear(
    job: &str,
    task: &str,
    day: Option<Day>,
    service: &mut TimeSheetService<'_>,
) {
    // TODO
    service
        .clear(job, task, &get_day(day))
        .await
        .unwrap_or_else(|err| {
            eprintln!("{err}");
        });
}

pub(crate) async fn logout(auth_service: &AuthService) {
    auth_service
        .logout()
        .await
        .context("Logout failed")
        .unwrap_or_else(|err| {
            eprintln!("Logout failed: {err}");
        });
}

pub(crate) async fn delete(line_number: &LineNumber, repository: &mut TimeSheetRepository) {
    repository
        .delete_line(line_number)
        .await
        .unwrap_or_else(|err| {
            eprintln!("Failed to delete line {line_number:?}: {err}");
        });
}
