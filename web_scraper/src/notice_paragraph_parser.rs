//! File with structs used to parse a notice paragraph
use crate::{
    error::{ScraperError, ScraperResult},
    scraper_adapter::{element_text_to_string, ScraperSiblingElement},
};
use proto_generator::notices::WARNNotice;
pub static INDIVIDUAL_NOTICE_PREFIX: &str = "Company:";
static COMPANY_LOCATION_LINE_PREFIX: &str = "Company location(s):";
static AFFECTED_EMPLOYEES_LINE_PREFIX: &str = "Affected employees:";
static EFFECTIVE_DATE_LINE_PREFIX: &str = "Effective Date:";

/// Struct to parse the paragraph of a notice into a WarnNotice
pub(crate) struct NoticeParagraphParser<'a> {
    notice_paragraph_first_sibling: ScraperSiblingElement<'a>,
}

impl<'a> NoticeParagraphParser<'a> {
    pub(crate) fn new(
        notice_paragraph_first_sibling: ScraperSiblingElement<'a>,
    ) -> NoticeParagraphParser {
        NoticeParagraphParser {
            notice_paragraph_first_sibling,
        }
    }

    /// Parses an individual notice from HTML and produces the protobuf
    pub(crate) fn parse_notice(&self) -> ScraperResult<WARNNotice> {
        let mut notice = WARNNotice::new();

        for individual_notice in self.notice_paragraph_first_sibling.clone() {
            match element_text_to_string(&individual_notice).as_str() {
                company_line if company_line.contains(INDIVIDUAL_NOTICE_PREFIX) => {
                    notice.set_firm_name(self.parse_company_line(company_line)?)
                }
                location_line if location_line.contains(COMPANY_LOCATION_LINE_PREFIX) => {
                    notice.set_firm_locations(self.parse_location(location_line)?)
                }
                affected_employees_line
                    if affected_employees_line.contains(AFFECTED_EMPLOYEES_LINE_PREFIX) =>
                {
                    notice.set_affected_employees(self.parse_affected(affected_employees_line)?)
                }
                effective_date_line if effective_date_line.contains(EFFECTIVE_DATE_LINE_PREFIX) => {
                    notice.set_effective_date(self.parse_effective_date(effective_date_line)?)
                }
                _ => {}
            }
        }

        Ok(notice)
    }

    /// Parses the company line and produces the firm_name string for the proto
    fn parse_company_line(&self, raw_company_line: &str) -> ScraperResult<String> {
        Self::general_parse(
            raw_company_line,
            INDIVIDUAL_NOTICE_PREFIX,
            "Error getting company from splitting company line".to_string(),
        )
    }

    fn parse_location(&self, raw_locations_line: &str) -> ScraperResult<String> {
        Self::general_parse(
            raw_locations_line,
            COMPANY_LOCATION_LINE_PREFIX,
            "Error getting location from splitting location line".to_string(),
        )
    }

    fn parse_affected(&self, raw_affected_line: &str) -> ScraperResult<String> {
        Self::general_parse(
            raw_affected_line,
            AFFECTED_EMPLOYEES_LINE_PREFIX,
            "Error getting location from splitting affected employees line".to_string(),
        )
    }

    fn parse_effective_date(&self, raw_date_line: &str) -> ScraperResult<String> {
        Self::general_parse(
            raw_date_line,
            EFFECTIVE_DATE_LINE_PREFIX,
            "Error getting location from splitting effective date line".to_string(),
        )
    }

    /// # Params
    /// * raw_line raw - line to parse
    /// * line_prefix - expected prefix for the line to parse
    /// * err_msg - The error message to use if the parsing fails
    fn general_parse(raw_line: &str, line_prefix: &str, err_msg: String) -> ScraperResult<String> {
        let parsed_line = raw_line
            .trim()
            .split(line_prefix)
            .nth(1)
            .ok_or_else(|| ScraperError::Parsing(err_msg))?
            .trim()
            .to_string();

        Ok(parsed_line)
    }
}
