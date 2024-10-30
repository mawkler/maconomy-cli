use std::{fmt::Display, iter};

/// Format anyhow error and include all its sources recursively
pub(crate) fn error_stack_fmt(err: &anyhow::Error) -> impl Display {
    let error_stack = iter::successors(err.source(), |&next_err| next_err.source())
        .map(|source| format!("  - {source}"))
        .collect::<Vec<_>>()
        .join("\n");

    if error_stack.is_empty() {
        format!("{err}")
    } else {
        format!("{err}\n\nError stack:\n{error_stack}")
    }
}
