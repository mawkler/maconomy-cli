use crate::infrastructure::models::time_registration;
use owo_colors::OwoColorize;
use std::fmt::Display;
use tabled::settings::{
    object::Rows, style::BorderColor, themes::Colorization, Color, Style, Theme,
};

#[derive(tabled::Tabled, Default)]
pub(crate) struct Row<'a> {
    #[tabled(rename = "Job name")]
    pub(crate) job_name: &'a str,
    #[tabled(rename = "Task name")]
    pub(crate) task_name: &'a str,
    #[tabled(rename = "Mon")]
    #[tabled(display_with = "display_hours")]
    pub(crate) monday: f32,
    #[tabled(rename = "Tue")]
    #[tabled(display_with = "display_hours")]
    pub(crate) tuesday: f32,
    #[tabled(rename = "Wed")]
    #[tabled(display_with = "display_hours")]
    pub(crate) wednesday: f32,
    #[tabled(rename = "Thu")]
    #[tabled(display_with = "display_hours")]
    pub(crate) thursday: f32,
    #[tabled(rename = "Fri")]
    #[tabled(display_with = "display_hours")]
    pub(crate) friday: f32,
    #[tabled(rename = "Sat")]
    #[tabled(display_with = "display_hours")]
    pub(crate) saturday: f32,
    #[tabled(rename = "Sun")]
    #[tabled(display_with = "display_hours")]
    pub(crate) sunday: f32,
}

fn display_hours(hours: &f32) -> impl Display {
    if let 0.0 = hours {
        return "".to_string();
    }

    let whole_hours = hours.trunc() as u32;
    let minutes = ((*hours - whole_hours as f32) * 60.0).floor() as u32;

    format!("{whole_hours}:{minutes:02}")
}

impl<'a> From<&'a time_registration::TableRecord> for Row<'a> {
    fn from(record: &'a time_registration::TableRecord) -> Self {
        Row {
            job_name: &record.data.jobnamevar,
            task_name: &record.data.tasktextvar,
            monday: record.data.numberday1,
            tuesday: record.data.numberday2,
            wednesday: record.data.numberday3,
            thursday: record.data.numberday4,
            friday: record.data.numberday5,
            saturday: record.data.numberday6,
            sunday: record.data.numberday7,
        }
    }
}

fn gray_borders() -> BorderColor {
    let gray = Color::parse(' '.fg_rgb::<85, 85, 85>().to_string()).clone();

    BorderColor::new()
        .top(gray.clone())
        .left(gray.clone())
        .bottom(gray.clone())
        .right(gray.clone())
        .corner_bottom_right(gray.clone())
        .corner_bottom_left(gray.clone())
        .corner_top_left(gray.clone())
        .corner_top_right(gray.clone())
}

impl Display for time_registration::TimeRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.panes.table.records.iter().map(Row::from);

        let mut theme = Theme::from_style(Style::modern_rounded());
        theme.remove_vertical_lines();

        let mut table = tabled::Table::new(rows);
        let table = table
            .with(theme)
            .with(BorderColor::new().top(Color::FG_GREEN))
            .with(Colorization::exact(
                [tabled::settings::Color::BOLD],
                Rows::first(),
            ))
            .with(gray_borders());

        write!(f, "{table}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_hours_test() {
        let hours = [1.5, 0.25, 12.75, 0.0, 23.999, 10.1];
        let expected = ["1:30", "0:15", "12:45", "", "23:59", "10:06"];

        for (i, &hours) in hours.iter().enumerate() {
            let result = display_hours(&hours).to_string();
            assert_eq!(result, expected[i], "Failed at index {}", i);
        }
    }
}
