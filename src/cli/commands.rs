use anyhow::{Context, Result};
use chrono::{Datelike, Local};
use log::info;

use crate::{
    domain::models::day::Day,
    infrastructure::repositories::time_registration_repository::TimeRegistrationRepository,
};

// TODO: allow setting date/week
pub(crate) async fn get(service: &mut TimeRegistrationRepository) -> Result<()> {
    let time_registration = service
        .get_time_registration()
        .await
        .context("failed to get time registration")?;

    println!("{time_registration}");
    Ok(())
}

pub(crate) async fn set(
    hours: f32,
    day: Option<Day>,
    repository: &mut TimeRegistrationRepository,
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

    info!("time registration successfully set: {hours} hours on {day}");

    Ok(())
}
