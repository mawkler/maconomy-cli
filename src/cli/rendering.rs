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

#[derive(Default)]
pub(crate) struct SumWithApproval {
    pub(crate) sum: f32,
    pub(crate) is_approved: bool,
}

fn is_approved(approval_status: &str) -> bool {
    // Check if approval_status indicates approval (non-empty and not "pending" or similar)
    // Common approval statuses might be "approved", "Approved", etc.
    !approval_status.is_empty() && 
    !approval_status.eq_ignore_ascii_case("pending") &&
    !approval_status.eq_ignore_ascii_case("draft")
}

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
    #[tabled(display_with = "display_sum_with_approval")]
    pub(crate) sum: SumWithApproval,
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

fn display_sum_with_approval(sum_with_approval: &SumWithApproval) -> impl Display {
    let hours_str = if (sum_with_approval.sum - 0.0).abs() < f32::EPSILON {
        String::new()
    } else {
        let whole_hours = sum_with_approval.sum.trunc() as u32;
        let minutes = ((sum_with_approval.sum - whole_hours as f32) * 60.0).floor() as u32;
        format!("{whole_hours}:{minutes:02}")
    };

    if sum_with_approval.is_approved && !hours_str.is_empty() {
        // Use green checkmark (✓) with ANSI color codes
        // Format: right-aligned hours + space + checkmark
        // This keeps numbers right-aligned while adding checkmark on the right
        format!("{hours_str} \x1b[32m✓\x1b[0m")
    } else {
        hours_str
    }
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
        let sum_value = monday + tuesday + wednesday + thursday + friday + saturday + sunday;
        let sum = SumWithApproval {
            sum: sum_value,
            is_approved: is_approved(&line.approval_status),
        };
        
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
    dates
        .iter()
        .map(|&date| if should_be_red(date) { Color::FG_RED } else { Color::FG_BLUE })
        .collect::<Vec<Color>>()
        .try_into()
        .expect("Iterator should produce exactly 7 colors")
}

/// Applies a Color (via OR) to each element in the array
fn apply_color_to_array(colors: [Color; 7], color_to_apply: Color) -> [Color; 7] {
    colors
        .iter()
        .map(|c| c.clone() | color_to_apply.clone())
        .collect::<Vec<Color>>()
        .try_into()
        .expect("Iterator should produce exactly 7 colors")
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
        // Default to showing all rows when using Display trait
        write!(f, "{}", self.format_table(true))
    }
}

impl TimeSheet {
    pub(crate) fn format_table(&self, full: bool) -> String {
        let line_rowz = self.time_rows(full);

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
            .with(Colorization::exact([Color::BOLD|Color::FG_BRIGHT_WHITE], (Rows::last()-1).intersect(Columns::new(0..11))))
        ;
        table.to_string()
    }
}

impl TimeSheet {
    fn time_rows<'a>(&'a self, full: bool) -> Chain<Map<IntoIter<LineRow<'a>>, fn(LineRow<'a>) -> Row<'a>>, Once<Row<'a>>> {
        // Convert all Lines to LineRows for sum calculation
        let line_rows: Vec<LineRow<'a>> = self.lines.iter().map(LineRow::from).collect();
        
        // Filter out rows with no hours if full is false
        let filtered_line_rows: Vec<LineRow<'a>> = if full {
            line_rows
        } else {
            line_rows.into_iter()
                .filter(|row| (row.sum.sum - 0.0).abs() >= f32::EPSILON)
                .collect()
        };
        
        // Sum all weekday values across filtered rows
        let monday_sum: f32 = filtered_line_rows.iter().map(|r| r.monday).sum();
        let tuesday_sum: f32 = filtered_line_rows.iter().map(|r| r.tuesday).sum();
        let wednesday_sum: f32 = filtered_line_rows.iter().map(|r| r.wednesday).sum();
        let thursday_sum: f32 = filtered_line_rows.iter().map(|r| r.thursday).sum();
        let friday_sum: f32 = filtered_line_rows.iter().map(|r| r.friday).sum();
        let saturday_sum: f32 = filtered_line_rows.iter().map(|r| r.saturday).sum();
        let sunday_sum: f32 = filtered_line_rows.iter().map(|r| r.sunday).sum();
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
            sum: SumWithApproval {
                sum: total_sum,
                is_approved: false, // Sum row is never approved
            },
        };
        // Convert to Vec<Row> and chain the rows iterator with the sum row and date row
        let line_rowz = filtered_line_rows.into_iter()
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
            create_action: None,
            lines: vec![
                Line {
                    number: "one".to_string(),
                    job: "Job number one".to_string(),
                    task: "Task number one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "two".to_string(),
                    job: "job number two".to_string(),
                    task: "task number two".to_string(),
                    week: create_week([0, 0, 8, 8, 1, 1, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "three".to_string(),
                    job: "job number three".to_string(),
                    task: "task number three".to_string(),
                    week: create_week([0, 0, 0, 0, 7, 7, 8]),
                    approval_status: "".to_string(),
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
            create_action: None,
            lines: vec![
                Line {
                    number: "one".to_string(),
                    job: "Job number one".to_string(),
                    task: "Task number one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                    approval_status: "approved".to_string(),
                },
                Line {
                    number: "two".to_string(),
                    job: "job number two".to_string(),
                    task: "task number two".to_string(),
                    week: create_week([0, 0, 8, 8, 1, 1, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "three".to_string(),
                    job: "job number three".to_string(),
                    task: "task number three".to_string(),
                    week: create_week([0, 0, 0, 0, 7, 7, 8]),
                    approval_status: "Approved".to_string(),
                },
            ],
            week_number: WeekNumber::new(47, WeekPart::WHOLE, 2024).unwrap(),
        })
        .to_string();
        insta::assert_snapshot!(time_sheet.to_string());
    }

    #[test]
    fn format_table_hides_rows_with_no_hours_when_full_is_false() {
        let time_sheet = TimeSheet {
            create_action: None,
            lines: vec![
                Line {
                    number: "one".to_string(),
                    job: "Job with hours".to_string(),
                    task: "Task with hours".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "two".to_string(),
                    job: "Job with no hours".to_string(),
                    task: "Task with no hours".to_string(),
                    week: create_week([0, 0, 0, 0, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "three".to_string(),
                    job: "Another job with hours".to_string(),
                    task: "Another task with hours".to_string(),
                    week: create_week([0, 0, 8, 8, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
            ],
            week_number: WeekNumber::new(47, WeekPart::WHOLE, 2024).unwrap(),
        };

        let output = time_sheet.format_table(false);
        let ansi_stripped = anstream::adapter::strip_str(&output).to_string();
        
        // Should not contain the row with no hours
        assert!(!ansi_stripped.contains("Job with no hours"));
        assert!(!ansi_stripped.contains("Task with no hours"));
        
        // Should contain rows with hours
        assert!(ansi_stripped.contains("Job with hours"));
        assert!(ansi_stripped.contains("Another job with hours"));
        
        // Should contain the sum row
        assert!(ansi_stripped.contains("Sum"));
        
        insta::assert_snapshot!(ansi_stripped);
    }

    #[test]
    fn format_table_shows_all_rows_when_full_is_true() {
        let time_sheet = TimeSheet {
            create_action: None,
            lines: vec![
                Line {
                    number: "one".to_string(),
                    job: "Job with hours".to_string(),
                    task: "Task with hours".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "two".to_string(),
                    job: "Job with no hours".to_string(),
                    task: "Task with no hours".to_string(),
                    week: create_week([0, 0, 0, 0, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "three".to_string(),
                    job: "Another job with hours".to_string(),
                    task: "Another task with hours".to_string(),
                    week: create_week([0, 0, 8, 8, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
            ],
            week_number: WeekNumber::new(47, WeekPart::WHOLE, 2024).unwrap(),
        };

        let output = time_sheet.format_table(true);
        let ansi_stripped = anstream::adapter::strip_str(&output).to_string();
        
        // Should contain all rows including the one with no hours
        assert!(ansi_stripped.contains("Job with hours"));
        assert!(ansi_stripped.contains("Job with no hours"));
        assert!(ansi_stripped.contains("Task with no hours"));
        assert!(ansi_stripped.contains("Another job with hours"));
        
        // Should contain the sum row
        assert!(ansi_stripped.contains("Sum"));
        
        insta::assert_snapshot!(ansi_stripped);
    }

    #[test]
    fn format_table_sum_row_reflects_filtered_rows() {
        let time_sheet = TimeSheet {
            create_action: None,
            lines: vec![
                Line {
                    number: "one".to_string(),
                    job: "Job one".to_string(),
                    task: "Task one".to_string(),
                    week: create_week([8, 8, 0, 0, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
                Line {
                    number: "two".to_string(),
                    job: "Job two".to_string(),
                    task: "Task two".to_string(),
                    week: create_week([0, 0, 0, 0, 0, 0, 0]), // No hours
                    approval_status: "".to_string(),
                },
                Line {
                    number: "three".to_string(),
                    job: "Job three".to_string(),
                    task: "Task three".to_string(),
                    week: create_week([0, 0, 4, 4, 0, 0, 0]),
                    approval_status: "".to_string(),
                },
            ],
            week_number: WeekNumber::new(47, WeekPart::WHOLE, 2024).unwrap(),
        };

        // When full=false, sum should only include rows with hours (16 + 8 = 24)
        let output_filtered = time_sheet.format_table(false);
        let ansi_stripped_filtered = anstream::adapter::strip_str(&output_filtered).to_string();
        assert!(ansi_stripped_filtered.contains("24:00")); // 16 + 8 hours
        
        // When full=true, sum should include all rows (16 + 0 + 8 = 24, same in this case)
        let output_full = time_sheet.format_table(true);
        let ansi_stripped_full = anstream::adapter::strip_str(&output_full).to_string();
        assert!(ansi_stripped_full.contains("24:00"));
    }
}
