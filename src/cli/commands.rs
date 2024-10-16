use crate::{
    domain::{
        models::{day::Day, line_number::LineNumber},
        time_sheet_service::TimeSheetService,
    },
    infrastructure::{
        auth_service::AuthService, repositories::time_sheet_repository::TimeSheetRepository,
    },
};
use anyhow::{Context, Result};
use chrono::{Datelike, Local};
use log::info;

// TODO: allow setting week
pub(crate) async fn get(week: Option<u8>, repository: &mut TimeSheetRepository) -> Result<()> {
    if week.is_some() {
        panic!("--week flag is not yet supported")
    }

    let time_sheet = repository
        .get_time_sheet()
        .await
        .context("failed to get time sheet")?;

    println!("{time_sheet}");
    Ok(())
}

fn get_day(day: Option<Day>) -> Result<Day> {
    if let Some(day) = day {
        Ok(day)
    } else {
        // Fall back to today's weekday
        let today = Local::now().date_naive().weekday().to_string().parse()?;
        info!("no day passed to 'set', using today's weekday '{today}'");
        Ok(today)
    }
}

pub(crate) async fn set(
    hours: f32,
    day: Option<Day>,
    job: &str,
    task: &str,
    repository: &mut TimeSheetRepository,
) -> Result<()> {
    let day = get_day(day)?;
    repository
        .set_time(hours, &day.clone(), job, task)
        .await
        .with_context(|| format!("Failed to set {hours} hours on {day}, job {job}, task {task}"))?;

    info!("time sheet successfully set: {hours} hours on {day}");

    Ok(())
}

pub(crate) async fn clear(
    job: &str,
    task: &str,
    day: Option<Day>,
    service: &mut TimeSheetService<'_>,
) -> Result<()> {
    let day = get_day(day)?;
    service.clear(job, task, &day).await
}

pub(crate) async fn logout(auth_service: &AuthService) -> Result<()> {
    auth_service.logout().await.context("Logout failed")
}

pub(crate) async fn delete(
    line_number: &LineNumber,
    repository: &mut TimeSheetRepository,
) -> Result<()> {
    repository
        .delete_line(line_number)
        .await
        .with_context(|| format!("Failed to delete line {line_number:?}"))
}
