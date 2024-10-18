/// Maconomy tasks have two different names, one called `taskname` (which seems to be shorter) and
/// one called `tasktextvar` which seems to be a longer name for the same task. Our domain uses the
/// longer name (i.e. `tasktextvar`).
///
/// This type represents a `taskname`.
pub(crate) struct ShortTaskName(pub(crate) String);
