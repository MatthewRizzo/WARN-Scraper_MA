//! Module implementing reading of csv files
use chrono::{Duration, NaiveDate};
use office::{DataType, Excel};
use proto_generator::{
    notice_collector::NoticeCollector,
    notices::{WARNNotice, WARNNotices},
};
use std::path::PathBuf;

use crate::error::{ScraperError, ScraperResult};

/// the headings start at row 3, but the data we need starts in row 4
static START_ROW_NUMBER: u32 = 3;

pub(crate) struct YearToDateParser {
    workbook: Excel,
    is_verbose: bool,
}

impl YearToDateParser {
    pub fn new(path_to_file: &PathBuf, is_verbose: bool) -> ScraperResult<YearToDateParser> {
        if is_verbose {
            println!("Opening {}", path_to_file.display());
        }

        let workbook = match Excel::open(path_to_file) {
            Err(office_error) => panic!("Error opening workbook: {}", office_error.to_string()),
            Ok(workbook) => workbook,
        };

        Ok(YearToDateParser {
            workbook,
            is_verbose,
        })
    }

    pub fn parse_for_notices(&mut self) -> ScraperResult<WARNNotices> {
        let sheet_names: Vec<String> = match self.workbook.sheet_names() {
            Err(office_error) => panic!("Error getting sheet names: {}", office_error.to_string()),
            Ok(sheet_names) => sheet_names,
        };

        sheet_names
            .into_iter()
            .map(|sheet_name| {
                Self::parse_worksheet(sheet_name, &mut self.workbook, self.is_verbose)
            })
            .collect::<ScraperResult<Vec<WARNNotices>>>()?
            .into_iter()
            .reduce(|a: WARNNotices, b: WARNNotices| NoticeCollector::reduce_notices(a, b))
            .ok_or_else(|| {
                ScraperError::MergingNotices(
                    "Error merging notices after parsing all worksheets".to_string(),
                )
            })
    }

    /// Parses an individual worksheet for it's notices
    fn parse_worksheet(
        sheet_name: String,
        workbook: &mut Excel,
        is_verbose: bool,
    ) -> ScraperResult<WARNNotices> {
        let sheet_range = match workbook.worksheet_range(sheet_name.as_str()) {
            Err(office_error) => panic!("Error getting sheet range: {}", office_error.to_string()),
            Ok(sheet_range) => sheet_range,
        };

        let subtract_err = || {
            ScraperError::ExcelParsing(
                "Subtracting to get the number of rows to iterate over failed".to_string(),
            )
        };

        let num_rows = sheet_range.rows().count() as u32;

        let num_rows_with_data = num_rows
            .checked_sub(START_ROW_NUMBER.into())
            .ok_or_else(subtract_err)? as u32;

        let mut worksheet_notices: WARNNotices = WARNNotices::new();

        for row_idx in START_ROW_NUMBER..num_rows_with_data {
            let date_received = sheet_range.get_value(row_idx as usize, 0);
            let firm_name = sheet_range.get_value(row_idx as usize, 1);
            let firm_locations = sheet_range.get_value(row_idx as usize, 2);
            let effective_date = sheet_range.get_value(row_idx as usize, 3);
            let affected_employees = sheet_range.get_value(row_idx as usize, 4);

            // convert to a notice
            let notice = WARNNotice {
                firm_name: Self::check_for_value(firm_name, is_verbose),
                firm_locations: Self::check_for_value(firm_locations, is_verbose),
                affected_employees: Self::check_for_value(affected_employees, is_verbose),
                effective_date: Self::convert_date(effective_date),
                date_received: Self::convert_date(date_received),
                special_fields: ::std::default::Default::default(),
            };

            if !Self::are_no_fields_present(&notice) {
                worksheet_notices.notices.push(notice)
            }
        }

        Ok(worksheet_notices)
    }

    /// Checks if the data retrieved is a string (as expected) and returns it
    /// # Return
    /// * None if the cell is not a string or cannot be converted to a string
    /// * Some(item) if it is string-compatible
    fn check_for_value(data: &DataType, is_verbose: bool) -> Option<String> {
        match data {
            DataType::Bool(_) => None,
            DataType::Empty => None,
            DataType::Error(err) => {
                if is_verbose {
                    println!("Error from cell! {:?}", err);
                }
                None
            }
            DataType::Int(value) => Some(value.to_string()),
            DataType::Float(value) => Some(value.to_string()),
            DataType::String(value) => Some(value.to_owned()),
        }
    }

    /// Checks that at least one field is present
    fn are_no_fields_present(notice: &WARNNotice) -> bool {
        !notice.has_firm_name()
            && !notice.has_firm_locations()
            && !notice.has_affected_employees()
            && !notice.has_effective_date()
            && !notice.has_date_received()
    }

    /// # Brief
    /// Dates can be stored as string, int, or float depending on how it was
    /// entered by user and how our parser interprets it.
    /// We need to handle all cases
    fn convert_date(date_value: &DataType) -> Option<String> {
        if date_value == &DataType::Empty {
            return None;
        }

        match date_value {
            DataType::Int(value) => Some(Self::from_days_since_1900(value).to_string()),
            DataType::Float(value) => Some(Self::from_days_since_1900(*value as i64).to_string()),
            DataType::String(value) => Some(value.to_string()),
            _ => None,
        }
    }

    /// # Brief
    /// Excel stores dates as floats or ints in Unix Time Stamps relative to
    /// 1900. The problem is that parsers see the numbers an assume they aren't
    /// dates. We need to manually grab the value and convert it from epoch
    /// timestamp to a date we can use.
    /// # References
    /// Context for and implementation of idea below come from this rust-excel
    /// thread:
    /// https://github.com/tafia/calamine/issues/116#issuecomment-414025552
    fn from_days_since_1900<T>(raw_days_since_1900: T) -> NaiveDate
    where
        T: std::ops::Sub<i64, Output = i64>,
    {
        let d1900: NaiveDate = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        d1900 + Duration::days(raw_days_since_1900 - 2)
    }
}
