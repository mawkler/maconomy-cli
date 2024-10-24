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
    pub(crate) job: String,
    pub(crate) task: String,
    pub(crate) week: Week,
}

impl Line {
    pub(crate) fn new(job: String, task: String, week: Week) -> Self {
        Self { job, task, week }
    }
}

#[derive(Default, serde::Serialize)]
pub(crate) struct TimeSheet {
    pub(crate) lines: Vec<Line>,
}

impl TimeSheet {
    pub(crate) fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }
}

impl TimeSheet {
    pub(crate) fn find_line_nr(&self, job: &str, task: &str) -> Option<u8> {
        let (row, _) = self
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| line.job == job && line.task == task)?;

        Some(row as u8)
    }
}
