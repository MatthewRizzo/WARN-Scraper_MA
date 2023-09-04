use std::default;

use crate::notices::{WARNNotice, WARNNotices};

pub struct NoticeCollector {}

impl NoticeCollector {
    /// Reduce 2 notices into 1. Note this creates more resources, but does not
    /// consume any!
    pub fn reduce_notices(current: WARNNotices, other: WARNNotices) -> WARNNotices {
        let mut combined = WARNNotices::new();
        combined.notices.append(&mut current.clone().notices);
        combined.notices.append(&mut other.clone().notices);
        combined
    }

    /// Converts a notice into a grouping of notices
    pub fn to_notices(current: WARNNotice) -> WARNNotices {
        let mut notices = WARNNotices::new();
        notices.notices.push(current);
        notices
    }

    pub fn to_notices_from_vec(notice_vec: Vec<WARNNotice>) -> WARNNotices {
        WARNNotices {
            notices: notice_vec,
            special_fields: default::Default::default(),
        }
    }

    /// Retrieves all notices that reference the company in question.
    /// Ignores cases
    pub fn search_notices_for_company(notices: WARNNotices, company_name_key: &str) -> WARNNotices {
        let matches: Vec<WARNNotice> = notices
            .notices
            .into_iter()
            .filter(|notice| {
                notice
                    .firm_name()
                    .to_lowercase()
                    .contains(&company_name_key.to_lowercase())
            })
            .collect();

        Self::to_notices_from_vec(matches)
    }
}
