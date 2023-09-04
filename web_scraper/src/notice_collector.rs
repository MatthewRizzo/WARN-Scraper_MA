use proto_generator::notices::WARNNotices;

pub(crate) struct NoticeCollector {}

impl NoticeCollector {
    /// Reduce 2 notices into 1. Note this creates more resources, but does not
    /// consume any!
    pub(crate) fn reduce_notices(current: WARNNotices, other: WARNNotices) -> WARNNotices {
        let mut combined = WARNNotices::new();
        combined.notices.append(&mut current.clone().notices);
        combined.notices.append(&mut other.clone().notices);
        combined
    }
}
