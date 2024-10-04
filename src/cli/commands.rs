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
    repository: &mut TimeSheetRepository,
) -> Result<()> {
    let row = 0; // TODO: allow specifying row (i.e. job + task)

    let day: Day = if let Some(day) = day {
        day
    } else {
        // Fall back to today's weekday
        let today = Local::now().date_naive().weekday().to_string().parse()?;
        info!("no day passed to 'set', using today's weekday '{today}'");
        today
    };

    repository
        .set_time(hours, day.clone().into(), row)
        .await
        .context(format!("Failed to set {hours} hours on {day}, row {row}"))?;

    info!("time sheet successfully set: {hours} hours on {day}");

    Ok(())
}
