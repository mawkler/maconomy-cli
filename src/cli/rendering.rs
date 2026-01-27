use crate::domain::models::time_sheet::{Line, TimeSheet};
use crate::domain::models::swedish_holidays::{is_holiday, HolidayType};
use std::fmt::Display;
use std::iter;
use std::ops::Add;
use chrono::{Datelike, Days, NaiveDate, Weekday};
use tabled::grid::config::Borders;
use tabled::settings::{object::Rows, style::BorderColor, themes::Colorization, Color, Highlight, Modify, Panel, Style, Theme};
use tabled::settings::object::{Columns, Object};
use tabled::settings::panel::HorizontalPanel;
use tabled::settings::style::VerticalLine;
use crate::domain::models::week::{WeekNumber};

#[derive(tabled::Tabled, Default)]
pub(crate) struct LineRow<'a> {
    #[tabled(rename = "Job number")]
    pub(crate) job_number: &'a str,
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
    #[tabled(rename = "Sum")]
    #[tabled(display_with = "display_hours")]
    pub(crate) sum: f32,
}
#[derive(tabled::Tabled)]
pub(crate) struct DateRow {
    #[tabled(rename = "Job number")]
    pub(crate) job_number: String,
    #[tabled(rename = "Job name")]
    pub(crate) job_name: String,
    #[tabled(rename = "Task name")]
    pub(crate) task_name: String,
    #[tabled(rename = "Mon")]
    #[tabled(display_with = "day")]
    pub(crate) monday: NaiveDate,
    #[tabled(rename = "Tue")]
    #[tabled(display_with = "day")]
    pub(crate) tuesday: NaiveDate,
    #[tabled(rename = "Wed")]
    #[tabled(display_with = "day")]
    pub(crate) wednesday: NaiveDate,
    #[tabled(rename = "Thu")]
    #[tabled(display_with = "day")]
    pub(crate) thursday: NaiveDate,
    #[tabled(rename = "Fri")]
    #[tabled(display_with = "day")]
    pub(crate) friday: NaiveDate,
    #[tabled(rename = "Sat")]
    #[tabled(display_with = "day")]
    pub(crate) saturday: NaiveDate,
    #[tabled(rename = "Sun")]
    #[tabled(display_with = "day")]
    pub(crate) sunday: NaiveDate,
    #[tabled(rename = "Sum")]
    #[tabled(display_with = "day_or_empty")]
    pub(crate) sum: NaiveDate,
}

use std::borrow::Cow;
use std::iter::{Chain, Map, Once};
use std::vec::IntoIter;
use tabled::Tabled;

pub(crate) enum Row<'a> {
    LineRow(LineRow<'a>),
    DateRow(DateRow),
}

impl<'a> Tabled for Row<'a> {
    const LENGTH: usize = 11;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        match self {
            Row::LineRow(line) => line.fields(),
            Row::DateRow(date) => date.fields(),
        }
    }

    fn headers() -> Vec<Cow<'static, str>> {
        LineRow::headers()
    }
}

fn day(day: &NaiveDate) -> impl Display {
    day.format("%-d").to_string()
}

fn day_or_empty(day: &NaiveDate) -> impl Display {
    if *day == NaiveDate::default() {
        String::new()
    } else {
        day.format("%-d").to_string()
    }
}
fn display_hours(hours: &f32) -> impl Display {
    if (*hours - 0.0).abs() < f32::EPSILON {
        return String::new();
    }

    let whole_hours = hours.trunc() as u32;
    let minutes = ((hours - whole_hours as f32) * 60.0).floor() as u32;

    format!("{whole_hours}:{minutes:02}")
}

impl<'a> From<&'a Line> for LineRow<'a> {
    fn from(line: &'a Line) -> Self {
        let monday = line.week.monday.0;
        let tuesday = line.week.tuesday.0;
        let wednesday = line.week.wednesday.0;
        let thursday = line.week.thursday.0;
        let friday = line.week.friday.0;
        let saturday = line.week.saturday.0;
        let sunday = line.week.sunday.0;
        let sum = monday + tuesday + wednesday + thursday + friday + saturday + sunday;
        
        LineRow {
            job_number: &line.number,
            job_name: &line.job,
            task_name: &line.task,
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
            sum,
        }
    }
}

impl<'a> From<LineRow<'a>> for Row<'a> {
    fn from(line_row: LineRow<'a>) -> Self {
        Row::LineRow(line_row)
    }
}

fn gray() -> Color {
    Color::parse("\x1b[38;2;085;085;085m \x1b[39m")
  //  Color::parse(' '.fg_rgb::<85, 85, 85>().to_string())
}

fn gray_borders() -> BorderColor {
    let gray_color = gray();
    BorderColor::new()
        .top(gray_color.clone())
        .left(gray_color.clone())
        .bottom(gray_color.clone())
        .right(gray_color.clone())
        .corner_bottom_right(gray_color.clone())
        .corner_bottom_left(gray_color.clone())
        .corner_top_left(gray_color.clone())
        .corner_top_right(gray_color)
}

/// Checks if a date should be marked in red (weekend, public holiday, or de facto full holiday)
fn should_be_red(date: NaiveDate) -> bool {
    // Check if it's a weekend
    if date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun {
        return true;
    }
    
    // Check if it's a public holiday or de facto full holiday
    if let Some(holiday) = is_holiday(date) {
        return matches!(holiday.holiday_type, HolidayType::Public | HolidayType::DeFacto);
    }
    
    false
}

/// Returns an array of Colors (FG_RED or FG_BLUE) based on whether each date should be red
fn get_column_colors(dates: [NaiveDate; 7]) -> [Color; 7] {
    [
        if should_be_red(dates[0]) { Color::FG_RED } else { Color::FG_BLUE },
        if should_be_red(dates[1]) { Color::FG_RED } else { Color::FG_BLUE },
        if should_be_red(dates[2]) { Color::FG_RED } else { Color::FG_BLUE },
        if should_be_red(dates[3]) { Color::FG_RED } else { Color::FG_BLUE },
        if should_be_red(dates[4]) { Color::FG_RED } else { Color::FG_BLUE },
        if should_be_red(dates[5]) { Color::FG_RED } else { Color::FG_BLUE },
        if should_be_red(dates[6]) { Color::FG_RED } else { Color::FG_BLUE },
    ]
}

/// Applies a Color (via OR) to each element in the array
fn apply_color_to_array(colors: [Color; 7], color_to_apply: Color) -> [Color; 7] {
    [
        colors[0].clone() | color_to_apply.clone(),
        colors[1].clone() | color_to_apply.clone(),
        colors[2].clone() | color_to_apply.clone(),
        colors[3].clone() | color_to_apply.clone(),
        colors[4].clone() | color_to_apply.clone(),
        colors[5].clone() | color_to_apply.clone(),
        colors[6].clone() | color_to_apply.clone(),
    ]
}

/// Builds a color array by concatenating prefix colors with column colors
fn build_color_array(prefix: [Color; 3], column_colors: &[Color; 7]) -> [Color; 10] {
    prefix
        .iter()
        .cloned()
        .chain(column_colors.iter().cloned())
        .collect::<Vec<Color>>()
        .try_into()
        .expect("Iterator should produce exactly 10 colors")
}

impl Display for TimeSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line_rowz = self.time_rows();

        let date_row = self.date_row();
        
        // Determine which columns should be red based on dates (extract before moving date_row)
        let dates = [
            date_row.monday,
            date_row.tuesday,
            date_row.wednesday,
            date_row.thursday,
            date_row.friday,
            date_row.saturday,
            date_row.sunday,
        ];
        
        let column_colors_normal = get_column_colors(dates);
        let column_colors = apply_color_to_array(column_colors_normal.clone(), Color::BOLD);

        let rows: Vec<Row> = line_rowz
            .chain(iter::once(Row::DateRow(date_row)))
            .collect();

        let mut theme = Theme::from_style(Style::modern_rounded());
        theme.remove_vertical_lines();
        theme.insert_vertical_line(10, VerticalLine::inherit(Style::modern_rounded()));

        let mut table = tabled::Table::new(rows);
        let table = table
           .with(theme)
            .with(Colorization::exact(
                build_color_array(
                    [
                        Color::FG_WHITE,
                        Color::BOLD | Color::FG_WHITE,
                        Color::BOLD | Color::FG_WHITE,
                    ],
                    &column_colors,
                ),
                Rows::first().intersect(Columns::new(0..10))
            )).with(Colorization::exact(
            build_color_array(
                [
                    Color::FG_WHITE,
                    Color::FG_WHITE,
                    Color::FG_WHITE,
                ],
                &column_colors_normal,
            ),
            Rows::first().inverse().intersect(Columns::new(0..10))
        ))
            .with(Colorization::exact([Color::BOLD], Rows::last()))
            //.with(Colorization::exact([Color::BOLD|Color::FG_BRIGHT_BLACK|Color::BG_BRIGHT_WHITE], (Rows::last()-1).intersect(Columns::new(0..11))))
            .with(Colorization::exact([Color::BOLD|Color::FG_BRIGHT_WHITE], (Rows::last()-1).intersect(Columns::new(0..11))))
         //   .with(Highlight::color( Rows::last()-1,BorderColor::filled(Color::BG_BRIGHT_WHITE)))
         /*   .with(
                Modify::new(Columns::new(1))
                    .with(Borders::new().vertical(Border::filled()))
            );*/

        // .with(gray_borders());
;
        write!(f, "{table}")
    }
}

impl TimeSheet {
    fn time_rows<'a>(&'a self) -> Chain<Map<IntoIter<LineRow<'a>>, fn(LineRow<'a>) -> Row<'a>>, Once<Row<'a>>> {
        // Convert all Lines to LineRows for sum calculation
        let line_rows: Vec<LineRow<'a>> = self.lines.iter().map(LineRow::from).collect();
        // Sum all weekday values across all rows
        let monday_sum: f32 = line_rows.iter().map(|r| r.monday).sum();
        let tuesday_sum: f32 = line_rows.iter().map(|r| r.tuesday).sum();
        let wednesday_sum: f32 = line_rows.iter().map(|r| r.wednesday).sum();
        let thursday_sum: f32 = line_rows.iter().map(|r| r.thursday).sum();
        let friday_sum: f32 = line_rows.iter().map(|r| r.friday).sum();
        let saturday_sum: f32 = line_rows.iter().map(|r| r.saturday).sum();
        let sunday_sum: f32 = line_rows.iter().map(|r| r.sunday).sum();
        let total_sum = monday_sum + tuesday_sum + wednesday_sum + thursday_sum + friday_sum + saturday_sum + sunday_sum;

        let sum_row = LineRow {
            job_number: "--",
            job_name: "Sum",
            task_name: "",
            monday: monday_sum,
            tuesday: tuesday_sum,
            wednesday: wednesday_sum,
            thursday: thursday_sum,
            friday: friday_sum,
            saturday: saturday_sum,
            sunday: sunday_sum,
            sum: total_sum,
        };
        // Convert to Vec<Row> and chain the rows iterator with the sum row and date row
        let line_rowz = line_rows.into_iter()
            .map(Row::LineRow as fn(LineRow<'a>) -> Row<'a>)
            .chain(iter::once(Row::LineRow(sum_row)));
        let chain = line_rowz;
        chain
    }

    fn date_row(&self) -> DateRow {
       // let monday = self.week_number.first_day().unwrap();
        let week_str = format!("Week {}", self.week_number);
        let w = self.week_number.number;
        let monday = NaiveDate::from_isoywd_opt(self.week_number.year, w.into(), Weekday::Mon).unwrap();
        let sunday = NaiveDate::from_isoywd_opt(self.week_number.year, w.into(), Weekday::Sun).unwrap();
        let start_month_str = monday.format("%B").to_string();
        let end_month_str = sunday.format("%B").to_string();
        let date_row = DateRow {
            job_number: week_str,
            job_name: start_month_str,
            task_name: end_month_str,
            monday: monday,
            tuesday: monday.add(Days::new(1)),
            wednesday: monday.add(Days::new(2)),
            thursday: monday.add(Days::new(3)),
            friday: monday.add(Days::new(4)),
            saturday: monday.add(Days::new(5)),
            sunday: monday.add(Days::new(6)),
            sum: Default::default(),
        };
        date_row
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::arguments::WeekPart;
    use crate::domain::models::{hours::Hours, time_sheet::Week, week::WeekNumber};

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
                    number: "one".to_string(),
                    job: "Job number one".to_string(),
                    task: "Task number one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                },
                Line {
                    number: "two".to_string(),
                    job: "job number two".to_string(),
                    task: "task number two".to_string(),
                    week: create_week([0, 0, 8, 8, 1, 1, 0]),
                },
                Line {
                    number: "three".to_string(),
                    job: "job number three".to_string(),
                    task: "task number three".to_string(),
                    week: create_week([0, 0, 0, 0, 7, 7, 8]),
                },
            ],
            week_number: WeekNumber::new(47, WeekPart::WHOLE, 2024).unwrap(),
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
                    number: "one".to_string(),
                    job: "Job number one".to_string(),
                    task: "Task number one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                },
                Line {
                    number: "two".to_string(),
                    job: "job number two".to_string(),
                    task: "task number two".to_string(),
                    week: create_week([0, 0, 8, 8, 1, 1, 0]),
                },
                Line {
                    number: "three".to_string(),
                    job: "job number three".to_string(),
                    task: "task number three".to_string(),
                    week: create_week([0, 0, 0, 0, 7, 7, 8]),
                },
            ],
            week_number: WeekNumber::new(47, WeekPart::WHOLE, 2024).unwrap(),
        })
        .to_string();
        insta::assert_snapshot!(time_sheet.to_string());
    }
}
