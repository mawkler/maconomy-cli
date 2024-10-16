use super::time_registration::TimeRegistration;

/// Maconomy tasks have two different names, one called `taskname` (which seems to be shorter) and
/// one called `tasktextvar` which seems to be a longer name for the same task. Our domain uses the
/// longer name (i.e. `tasktextvar`).
///
/// This type represents a `taskname`.
pub(crate) struct ShortTaskName(pub(crate) String);

pub(crate) fn short_task_name_from_full_task_name(
    short_task_name: &str,
    time_registration: TimeRegistration,
) -> Option<ShortTaskName> {
    dbg!(&time_registration.panes.table.records);

    time_registration
        .panes
        .table
        .records
        .into_iter()
        .find(|row| row.data.tasktextvar == short_task_name)
        .map(|row| ShortTaskName(row.data.taskname))
}
