use std::str::FromStr;

/// 1-indexed number for line in time sheet
#[derive(Debug, Clone)]
pub(crate) enum LineNumber {
    Number(u8),
    Last,
}

impl FromStr for LineNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line_number = match s {
            "last" => Self::Last,
            _ => {
                let number = s
                    .parse()
                    .map_err(|err| format!("Failed to parse number: {err}"))?;
                Self::Number(number)
            }
        };

        match line_number {
            Self::Number(0) => {
                Err("Invalid line number. Note that line numbers are 1-based".to_string())
            }
            n => Ok(n),
        }
    }
}
