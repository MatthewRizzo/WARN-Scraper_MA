use crate::{
    error::{ScraperError, ScraperResult},
    notice_paragraph_parser::{NoticeParagraphParser, INDIVIDUAL_NOTICE_PREFIX},
    scraper_adapter,
    scraper_adapter::ScraperSiblingElement,
};
use scraper::{Element, ElementRef, Html, Selector};

use proto_generator::notices::WARNNotices;

static WARN_HEADING: &str = "Companies that submitted WARN notices this past week";
static CURRENT_YEAR_REPORT_TEXT: &str = "WARN Report for the week ending";

/// Struct to adapt the scraper crate to our use cases
pub struct ScraperAdapter {
    document: Html,
}

impl ScraperAdapter {
    pub fn new(page_to_request_url: &str) -> ScraperResult<ScraperAdapter> {
        let main_page_html = reqwest::blocking::get(page_to_request_url)?.text()?;
        let document = Html::parse_document(&main_page_html);

        Ok(ScraperAdapter { document })
    }

    pub fn get_notices(&self) -> ScraperResult<WARNNotices> {
        let notice_section: ElementRef = self.get_submit_notice_reference(&self.document)?;
        let notice_section_first_sibling = Self::get_notices_first_child(notice_section.clone())?;

        let notices = self.get_notices_from_section(notice_section_first_sibling)?;
        let ytd_notices = self.get_current_ytd_warn_notices(notice_section_first_sibling)?;
        Ok(notices)
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
    /// * The reference to the
    fn get_current_ytd_warn_notices<'a>(
        &self,
        notice_section_first_sibling: ScraperSiblingElement<'a>,
    ) -> ScraperResult<()> {
        let notice_children =
            notice_section_first_sibling.map(|notice_element| notice_element.first_element_child());
        let ytd_notice_element = Self::flatten_yearly_report(notice_children)?;

        if let Some(href) = ytd_notice_element.value().attr("href") {
            println!("Found href = {}", href)
        }

        // TODO - use the href to download the file
        Ok(())
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
}
