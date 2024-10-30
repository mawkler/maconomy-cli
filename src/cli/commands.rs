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

// TODO: allow setting week
pub(crate) async fn get_table(
    week: Option<u8>,
    repository: &mut TimeSheetRepository,
) -> anyhow::Result<()> {
    if week.is_some() {
        panic!("--week flag is not yet supported")
    }

    let time_sheet = repository.get_time_sheet().await?;

    println!("{time_sheet}");
    Ok(())
}

async fn get_json(week: Option<u8>, repository: &mut TimeSheetRepository) -> anyhow::Result<()> {
    if week.is_some() {
        panic!("--week flag is not yet supported")
    }

    let time_sheet = repository.get_time_sheet().await?;
    let json = serde_json::to_string(&time_sheet).context("Failed to deserialize time sheet")?;

    println!("{json}");
    Ok(())
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

pub(crate) async fn get(
    week: Option<u8>,
    format: Option<Format>,
    repository: &mut TimeSheetRepository,
) {
    let format = format.unwrap_or(Format::Table);
    match format {
        Format::Json => get_json(week, repository)
            .await
            .unwrap_or_else(|err| eprintln!("Failed to get time sheet as JSON: {err}")),
        Format::Table => get_table(week, repository)
            .await
            .unwrap_or_else(|err| eprintln!("Failed to get time sheet as table: {err}")),
    }
}

pub(crate) async fn set(
    hours: f32,
    day: Option<Day>,
    job: &str,
    task: &str,
    service: &mut TimeSheetService<'_>,
) {
    let day = get_day(day);
    service
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

pub(crate) async fn clear(
    job: &str,
    task: &str,
    day: Option<Day>,
    service: &mut TimeSheetService<'_>,
) {
    service
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
