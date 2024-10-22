use crate::domain::models::day::Day;
use crate::infrastructure::repositories::time_sheet_repository::{self, TimeSheetRepository};
use anyhow::Result;

pub(crate) struct TimeSheetService<'a> {
    repository: &'a mut TimeSheetRepository,
}

impl<'a> TimeSheetService<'a> {
    pub(crate) fn new(repository: &'a mut TimeSheetRepository) -> Self {
        Self { repository }
    }
}

impl TimeSheetService<'_> {
    pub(crate) async fn clear(
        &mut self,
        job: &str,
        task: &str,
        day: &Day,
    ) -> Result<(), time_sheet_repository::SetTimeError> {
        self.repository.set_time(0.0, day, job, task).await
    }
}
