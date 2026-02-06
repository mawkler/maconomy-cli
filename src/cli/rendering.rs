use crate::domain::models::time_sheet::{Line, TimeSheet};
use owo_colors::OwoColorize;
use std::fmt::Display;
use tabled::settings::{
    object::Rows, style::BorderColor, themes::Colorization, Color, Panel, Style, Theme,
};

#[derive(tabled::Tabled, Default)]
pub(crate) struct Row<'a> {
    #[tabled(rename = "Job name")]
    pub(crate) job_name: &'a str,
    #[tabled(rename = "Task name")]
    pub(crate) task_name: &'a str,
    #[tabled(rename = "Mon")]
    #[tabled(display = "display_hours")]
    pub(crate) monday: f32,
    #[tabled(rename = "Tue")]
    #[tabled(display = "display_hours")]
    pub(crate) tuesday: f32,
    #[tabled(rename = "Wed")]
    #[tabled(display = "display_hours")]
    pub(crate) wednesday: f32,
    #[tabled(rename = "Thu")]
    #[tabled(display = "display_hours")]
    pub(crate) thursday: f32,
    #[tabled(rename = "Fri")]
    #[tabled(display = "display_hours")]
    pub(crate) friday: f32,
    #[tabled(rename = "Sat")]
    #[tabled(display = "display_hours")]
    pub(crate) saturday: f32,
    #[tabled(rename = "Sun")]
    #[tabled(display = "display_hours")]
    pub(crate) sunday: f32,
}

fn display_hours(hours: &f32) -> String {
    if let 0.0 = hours {
        return "".to_string();
    }

    let whole_hours = hours.trunc() as u32;
    let minutes = ((*hours - whole_hours as f32) * 60.0).floor() as u32;

    format!("{whole_hours}:{minutes:02}")
}

impl<'a> From<&'a Line> for Row<'a> {
    fn from(line: &'a Line) -> Self {
        Row {
            job_name: &line.job,
            task_name: &line.task,
            monday: line.week.monday.0,
            tuesday: line.week.tuesday.0,
            wednesday: line.week.wednesday.0,
            thursday: line.week.thursday.0,
            friday: line.week.friday.0,
            saturday: line.week.saturday.0,
            sunday: line.week.sunday.0,
        }
    }
}

fn gray() -> Color {
    Color::parse(' '.fg_rgb::<85, 85, 85>().to_string()).clone()
}

fn gray_borders() -> BorderColor {
    BorderColor::new()
        .top(gray().clone())
        .left(gray().clone())
        .bottom(gray().clone())
        .right(gray().clone())
        .corner_bottom_right(gray().clone())
        .corner_bottom_left(gray().clone())
        .corner_top_left(gray().clone())
        .corner_top_right(gray().clone())
}

impl Display for TimeSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.lines.iter().map(Row::from);

        let mut theme = Theme::from_style(Style::modern_rounded());
        theme.remove_vertical_lines();

        let mut table = tabled::Table::new(rows);
        let table = table
            .with(theme)
            .with(Colorization::exact(
                [tabled::settings::Color::BOLD],
                Rows::first(),
            ))
            .with(Panel::footer(format!("Week {}", self.week_number)))
            .with(Colorization::exact([gray()], Rows::last()))
            .with(gray_borders());

        write!(f, "{table}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::models::{hours::Hours, time_sheet::Week};

    #[test]
    fn displays_hours() {
        let hours = [1.5, 0.25, 12.75, 0.0, 23.999, 10.1];
        let expected = ["1:30", "0:15", "12:45", "", "23:59", "10:06"];

        for (i, &hours) in hours.iter().enumerate() {
            let result = display_hours(&hours).to_string();
            assert_eq!(result, expected[i], "Failed at index {}", i);
        }
    }

    fn create_week(days: [u8; 7]) -> Week {
        let days: Vec<f32> = days.into_iter().map(Into::into).collect();
        Week {
            monday: Hours(days[0]),
            tuesday: Hours(days[1]),
            wednesday: Hours(days[2]),
            thursday: Hours(days[3]),
            friday: Hours(days[4]),
            saturday: Hours(days[5]),
            sunday: Hours(days[6]),
        }
    }

    #[test]
    fn display_ansi_stripped_timesheet() {
        let time_sheet = (TimeSheet {
            lines: vec![
                Line {
                    job: "Job number one".to_string(),
                    task: "Task number one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                },
                Line {
                    job: "job number two".to_string(),
                    task: "task number two".to_string(),
                    week: create_week([0, 0, 8, 8, 1, 1, 0]),
                },
                Line {
                    job: "job number three".to_string(),
                    task: "task number three".to_string(),
                    week: create_week([0, 0, 0, 0, 7, 7, 8]),
                },
            ],
            week_number: 47,
        })
        .to_string();

        // Strip away ANSI colorsa for easier debugging
        let ansi_stripped_time_sheet = anstream::adapter::strip_str(&time_sheet);
        insta::assert_snapshot!(ansi_stripped_time_sheet.to_string());
    }

    #[test]
    fn display_timesheet() {
        let time_sheet = (TimeSheet {
            lines: vec![
                Line {
                    job: "Job number one".to_string(),
                    task: "Task number one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                },
                Line {
                    job: "job number two".to_string(),
                    task: "task number two".to_string(),
                    week: create_week([0, 0, 8, 8, 1, 1, 0]),
                },
                Line {
                    job: "job number three".to_string(),
                    task: "task number three".to_string(),
                    week: create_week([0, 0, 0, 0, 7, 7, 8]),
                },
            ],
            week_number: 47,
        })
        .to_string();
        insta::assert_snapshot!(time_sheet.to_string());
    }
}
