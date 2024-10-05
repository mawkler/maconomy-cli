use anyhow::{Context, Result};
use chrono::{Datelike, Local};
use log::info;

use crate::{
    domain::models::day::Day,
    infrastructure::repositories::time_sheet_repository::TimeSheetRepository,
};

// TODO: allow setting date/week
pub(crate) async fn get(service: &mut TimeSheetRepository) -> Result<()> {
    let time_sheet = service
        .get_time_sheet()
        .await
        .context("failed to get time sheet")?;

    println!("{time_sheet}");
    Ok(())
}

pub(crate) async fn set(
    hours: f32,
    day: Option<Day>,
    job: &str,
    task: &str,
    repository: &mut TimeSheetRepository,
) -> Result<()> {
    let day: Day = if let Some(day) = day {
        day
    } else {
        // Fall back to today's weekday
        let today = Local::now().date_naive().weekday().to_string().parse()?;
        info!("no day passed to 'set', using today's weekday '{today}'");
        today
    };

    repository
        .set_time(hours, day.clone(), job, task)
        .await
        .with_context(|| format!("Failed to set {hours} hours on {day}, job {job}, task {task}"))?;

    info!("time sheet successfully set: {hours} hours on {day}");

    Ok(())
}
