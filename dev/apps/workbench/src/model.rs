//! Role: Synthetic incident records.

use std::rc::Rc;

pub type RecordKey = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub key: RecordKey,
    pub service: &'static str,
    pub summary: &'static str,
    pub details: String,
}

impl Record {
    /// Case-insensitive match on service, summary and details.
    pub fn matches(&self, needle: &str) -> bool {
        needle.is_empty()
            || self.service.contains(needle)
            || self.summary.contains(needle)
            || self.details.to_lowercase().contains(needle)
    }
}

pub fn generate_records(count: usize) -> Rc<Vec<Record>> {
    Rc::new(
        (0..count)
            .map(|index| Record {
                key: index as u64,
                service: match index % 4 {
                    0 => "gateway",
                    1 => "billing",
                    2 => "search",
                    _ => "storage",
                },
                summary: if index % 8 == 0 {
                    "timeout investigation"
                } else {
                    "routine event"
                },
                details: format!("Record {index}: diagnostic detail"),
            })
            .collect(),
    )
}
