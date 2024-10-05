use super::repositories::time_sheet_repository::TimeSheetRepository;

struct TimeSheetService {
    repository: TimeSheetRepository,
}

impl TimeSheetService {
    fn new(repository: TimeSheetRepository) -> Self {
        Self { repository }
    }
}

impl TimeSheetService {}
