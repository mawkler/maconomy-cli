use crate::infrastructure::time_registration;
use std::fmt::Display;
use tabled::settings::{object::Rows, themes::Colorization, Color, Format, Style, Theme};

#[derive(tabled::Tabled, Default)]
pub(crate) struct Row<'a> {
    #[tabled(rename = "Job name")]
    pub(crate) job_name: &'a str,
    #[tabled(rename = "Task name")]
    pub(crate) task_name: &'a str,
    #[tabled(rename = "Mon")]
    pub(crate) monday: f32,
    #[tabled(rename = "Tue")]
    pub(crate) tuesday: f32,
    #[tabled(rename = "Wed")]
    pub(crate) wednesday: f32,
    #[tabled(rename = "Thu")]
    pub(crate) thursday: f32,
    #[tabled(rename = "Fri")]
    pub(crate) friday: f32,
    #[tabled(rename = "Sat")]
    pub(crate) saturday: f32,
    #[tabled(rename = "Sun")]
    pub(crate) sunday: f32,
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

impl Display for time_registration::TimeRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.panes.table.records.iter().map(Row::from);
        let mut table = tabled::Table::new(rows);

        let table = table.with(Colorization::exact(
            [tabled::settings::Color::BOLD],
            Rows::first(),
        ));

        let mut theme = Theme::from_style(Style::modern_rounded());
        theme.remove_vertical_lines();

        let table = table.with(theme).to_string();

        write!(f, "{}", table)
    }
}
