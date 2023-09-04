use std::{path::PathBuf, str::FromStr};

use crate::{
    download_manager::DownloaderWrapper,
    error::{ScraperError, ScraperResult},
    notice_collector::NoticeCollector,
    notice_paragraph_parser::{NoticeParagraphParser, INDIVIDUAL_NOTICE_PREFIX},
    scraper_adapter,
    scraper_adapter::ScraperSiblingElement,
    year_to_date_xslx_parser::YearToDateParser,
};
use scraper::{Element, ElementRef, Html, Selector};

use proto_generator::notices::WARNNotices;

const WARN_HEADING: &str = "Companies that submitted WARN notices this past week";
const CURRENT_YEAR_REPORT_TEXT: &str = "WARN Report for the week ending";
const DOWNLOADED_FILE_PATH_DIRECTORY: &str = "/tmp/WARN_Reports/";
const DOWNLOADED_FILE_PATH_AFTER_DIRECTORY: &str = "WARN_report_20";
const DOWNLOADED_FILE_PATH_SUFFIX: &str = ".xlsx";

pub struct Parser {}

impl Parser {}

/// Struct to adapt the scraper crate to our use cases
pub struct ScraperAdapter {
    document: Html,
    base_url: String,
}

/// TODO - split this into its pure scraper to retrieve information, and
/// consumer of that information. I.e. the download and parsing of the file
/// should not be happening here.
/// Make refactor it out to have a parsing controller that uses this and the
/// xlsx parser. External parties will ONLY have access to that parsing
/// controller.
impl ScraperAdapter {
    /// # Params
    /// * base_url - The base url (i.e. http://foo/)
    /// * relative_page_to_request_url - the page relative to the base url to request
    pub fn new(
        base_url: String,
        relative_page_to_request_url: &str,
    ) -> ScraperResult<ScraperAdapter> {
        let full_page_to_request_url =
            Self::construct_full_url(&base_url, &relative_page_to_request_url)?;

        let main_page_html = reqwest::blocking::get(full_page_to_request_url)?.text()?;
        let document = Html::parse_document(&main_page_html);

        Ok(ScraperAdapter { document, base_url })
    }

    pub fn get_notices(&self) -> ScraperResult<WARNNotices> {
        let notice_section: ElementRef = self.get_submit_notice_reference(&self.document)?;
        let notice_section_first_sibling = Self::get_notices_first_child(notice_section.clone())?;

        let ytd_notices = match self.get_current_ytd_warn_notices(notice_section_first_sibling) {
            Ok(notices) => notices,
            Err(err) => panic!(
                "Error when getting year-to-date notices: {}",
                err.to_string()
            ),
        };

        let current_week_notices: WARNNotices =
            self.get_notices_from_section(notice_section_first_sibling)?;
        let overall_notices: WARNNotices =
            NoticeCollector::reduce_notices(current_week_notices, ytd_notices);
        Ok(overall_notices)
    }

    /// Returns an element reference to the first element under the submission
    /// header
    fn get_submit_notice_reference<'a>(
        &self,
        main_page: &'a Html,
    ) -> ScraperResult<ElementRef<'a>> {
        let selector: Selector = Selector::parse(r#"body section > h2"#).unwrap();

        let warning_headers = main_page
            .select(&selector)
            .filter_map(|child: ElementRef<'a>| {
                ElementRef::wrap(*child).and_then(|el| {
                    match scraper_adapter::element_text_to_string(&el).contains(WARN_HEADING) {
                        true => Some(el),
                        false => None,
                    }
                })
            })
            .collect::<Vec<ElementRef<'a>>>();

        if warning_headers.len() > 1 {
            return Err(ScraperError::Parsing(
                format!("Found multiple headings that match {}", WARN_HEADING).to_string(),
            ));
        }

        let warning_header_el: ElementRef<'a> = warning_headers[0];

        let notices_section: ElementRef<'a> =
            warning_header_el.next_sibling_element().ok_or_else(|| {
                ScraperError::Parsing("No sibling element to warning notice heading".to_string())
            })?;

        Ok(notices_section)
    }

    /// # Return
    /// A representation of the siblings within the notice section
    fn get_notices_first_child(
        notice_section_parent: ElementRef,
    ) -> ScraperResult<ScraperSiblingElement> {
        let inner_notices_first_child = Self::unwrap_inner_notices(notice_section_parent)?;
        let sibling_manager: ScraperSiblingElement =
            ScraperSiblingElement::new(inner_notices_first_child);
        Ok(sibling_manager)
    }

    fn get_notices_from_section<'a>(
        &self,
        notice_section_first_sibling: ScraperSiblingElement<'a>,
    ) -> ScraperResult<WARNNotices> {
        let notices_paragraphs = notice_section_first_sibling
            .filter(|sibling| {
                scraper_adapter::element_text_to_string(sibling).contains(INDIVIDUAL_NOTICE_PREFIX)
            })
            .collect::<Vec<ElementRef<'a>>>();

        let parsed_notices = notices_paragraphs
            .into_iter()
            .filter_map(|notice_paragraph_el: ElementRef<'_>| {
                notice_paragraph_el.first_element_child()
            })
            .map(ScraperSiblingElement::new)
            .map(|individual_notice| NoticeParagraphParser::new(individual_notice).parse_notice())
            .collect::<Vec<_>>();

        let mut notices = WARNNotices::new();

        for notice in parsed_notices {
            notices.notices.push(notice?);
        }

        Ok(notices)
    }

    ///
    /// # Return
    /// A reference to the html element containing the href to the csv of the
    /// year to date warn notices.
    ///
    /// # Params
    /// * The reference to the first element of the notice paragraphs
    fn get_current_ytd_warn_notices<'a>(
        &self,
        notice_section_first_sibling: ScraperSiblingElement<'a>,
    ) -> ScraperResult<WARNNotices> {
        let notice_children =
            notice_section_first_sibling.map(|notice_element| notice_element.first_element_child());
        let ytd_notice_element: ElementRef<'a> = Self::flatten_yearly_report(notice_children)?;

        let href = ytd_notice_element.value().attr("href").ok_or_else(|| {
            ScraperError::Parsing(format!(
                "Yearly report element {} does not have an href!",
                scraper_adapter::element_text_to_string(&ytd_notice_element)
            ))
        })?;

        let full_download_url = Self::construct_full_url(&self.base_url, &href)?;
        let year = Self::get_year_from_yearly_report_element(ytd_notice_element);
        let downloaded_file_path = Self::create_download_filename(&year)?;
        let download_directory = PathBuf::from_str(DOWNLOADED_FILE_PATH_DIRECTORY)?;
        let downloader = DownloaderWrapper::new(
            full_download_url.to_string(),
            &downloaded_file_path,
            &download_directory,
        );
        downloader.download_file()?;

        let mut xslx_parser = YearToDateParser::new(&downloaded_file_path)?;
        let xslx_notices = xslx_parser.parse_for_notices()?;

        Ok(xslx_notices)
    }

    /// There is an empty section between the the parent section of notices
    /// (the sibling to the heading), and the start of the notices. Strip that away.
    /// # Return
    /// The 1st child element of the notices section!
    fn unwrap_inner_notices<'a>(
        notice_section_parent: ElementRef<'a>,
    ) -> ScraperResult<ElementRef<'a>> {
        if !notice_section_parent.has_children() {
            return Err(ScraperError::Parsing(
                "notice_section_parent has no kids".to_string(),
            ));
        }

        let inner_notice_section = notice_section_parent
            .first_element_child()
            .expect("Error stripping away empty wrapper section to notices")
            .first_element_child()
            .expect("Error stripping away 2nd empty layer");

        Ok(inner_notice_section)
    }

    /// # Brief
    /// Determines if the child of a notice element has the text wrapper
    /// expected for the yearly report url
    ///
    /// # Return
    /// * The nested element with the href if it exists
    /// * None if no element fits the criteria
    fn is_yearly_report_element<'a>(
        notice_element_child: Option<ElementRef<'a>>,
    ) -> Option<ElementRef<'a>> {
        match notice_element_child {
            None => None,
            Some(notice_child) => {
                let child_text = scraper_adapter::element_text_to_string(&notice_child);
                if child_text.contains(CURRENT_YEAR_REPORT_TEXT) {
                    Some(notice_child)
                } else {
                    None
                }
            }
        }
    }

    /// # Brief
    /// Taking an iterator to (potential) children of notice elements, find
    /// the one most recently element containing an href to a spreadsheet of
    /// WARN reports.
    /// # Returns
    /// * Error - if the href element can not be found
    /// * The element if it exists
    fn flatten_yearly_report<'a, I>(notice_children_iter: I) -> ScraperResult<ElementRef<'a>>
    where
        I: Iterator<Item = Option<ElementRef<'a>>>,
    {
        let yearly_report_element = notice_children_iter
            .filter_map(Self::is_yearly_report_element)
            .collect::<Vec<_>>()
            .first()
            .ok_or_else(|| {
                ScraperError::Parsing(format!(
                    "No elements contain the prefix {} expected for yearly report",
                    CURRENT_YEAR_REPORT_TEXT
                ))
            })?
            .to_owned();

        Ok(yearly_report_element)
    }

    fn create_download_filename(year: &str) -> ScraperResult<PathBuf> {
        let full_path_str = format!(
            "{}{}{}{}",
            DOWNLOADED_FILE_PATH_DIRECTORY,
            DOWNLOADED_FILE_PATH_AFTER_DIRECTORY,
            year,
            DOWNLOADED_FILE_PATH_SUFFIX
        );

        Ok(PathBuf::from_str(&full_path_str)?)
    }

    /// Retrieve the year from the full user-available string. The user is given
    /// the year....We just want to extract it from there
    fn get_year_from_yearly_report_element<'a>(yearly_report_element: ElementRef<'a>) -> String {
        let full_text = scraper_adapter::element_text_to_string(&yearly_report_element);
        let year_digits_reversed = full_text.chars().rev().take(2).collect::<String>();
        year_digits_reversed
            .chars()
            .rev()
            .take(2)
            .collect::<String>()
    }

    fn construct_full_url(base_url: &str, relative_url: &str) -> ScraperResult<String> {
        let base_url: String = match base_url.ends_with("/") {
            true => base_url.chars().rev().take(1).collect::<String>(),
            false => base_url.to_string(),
        };

        let relative_url_start_no_slash = match relative_url.starts_with("/") {
            true => relative_url
                .chars()
                .next()
                .map(|char| &relative_url[char.len_utf8()..])
                .ok_or_else(|| {
                    ScraperError::Downloading(
                        "Error Constructing a full url for download".to_string(),
                    )
                })?,
            false => relative_url,
        };

        let full_path = format!("{}/{}", base_url, relative_url_start_no_slash);
        Ok(full_path)
    }
}
