use reqwest::Url;
use crate::domain::models::week::WeekNumber;
use super::hours::Hours;

#[derive(Debug, serde::Serialize)]
pub(crate) struct Week {
    pub(crate) monday: Hours,
    pub(crate) tuesday: Hours,
    pub(crate) wednesday: Hours,
    pub(crate) thursday: Hours,
    pub(crate) friday: Hours,
    pub(crate) saturday: Hours,
    pub(crate) sunday: Hours,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Line {
    pub(crate) number: String,
    pub(crate) job: String,
    pub(crate) task: String,
    pub(crate) week: Week,
    pub(crate) approval_status: String,
}

impl Line {
    pub(crate) fn new(number: String, job: String, task: String, week: Week, approval_status: String) -> Self {
        Self { number, job, task, week ,approval_status}
    }

    fn has_job_and_task(&self, job: &str, task: &str) -> bool {
        self.job.to_lowercase() == job.to_lowercase()
            && self.task.to_lowercase() == task.to_lowercase()
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct TimeSheet {
    #[serde(skip_serializing)]
    pub(crate) create_action: Option<String>,

    pub(crate) lines: Vec<Line>,
    pub(crate) week_number: WeekNumber,
}

impl TimeSheet {
    pub(crate) fn new(lines: Vec<Line>, week_number: WeekNumber, url: Option<String>) -> Self {
        Self { create_action: url, lines, week_number}
    }
    pub(crate) fn find_line_nr(&self, job: &str, task: &str) -> Option<u8> {
        self.lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.has_job_and_task(job, task))
            .map(|(row, _)| row as u8)
    }
}
