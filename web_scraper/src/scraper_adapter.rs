//! The scraper library used has some flaws in its downcasing to Nodes for
//! children. The structs in this module adapt the scraper API's into iterators
//! over children.
use scraper::{Element, ElementRef};

/// The library downcasts children to nodes when creating an iterator over them.
/// We need to be able to iterate over Elements to access their struct-specific
/// information.
/// This struct implements an iterator over HTML siblings starting from the
/// first sibling provided.
#[derive(Clone)]
pub(crate) struct ScraperSiblingElement<'a> {
    current_sibling: Option<ElementRef<'a>>,
}

impl<'a> ScraperSiblingElement<'a> {
    /// Given a sibling, provides an Element iterator over siblings at the same
    /// level of HTML tree
    pub(crate) fn new(initial_sibling: ElementRef<'a>) -> ScraperSiblingElement {
        ScraperSiblingElement {
            current_sibling: Some(initial_sibling),
        }
    }
}

impl<'a> Iterator for ScraperSiblingElement<'a> {
    type Item = ElementRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current: Option<ElementRef<'a>> = match self.current_sibling {
            None => None,
            Some(current_sibling) => {
                // Set the next value
                self.current_sibling = current_sibling.next_sibling_element();
                Some(current_sibling)
            }
        };

        // Return the current value
        current
    }
}

/// Utility function to get the text from an element
pub(crate) fn element_text_to_string(el: &ElementRef) -> String {
    el.text().collect::<String>()
}
