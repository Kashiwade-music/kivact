//! Role: Range endpoint identities.
use kivact::draw::RegionId;
/// What the keys and actions on the root of a range edit.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RangePart {
    Start,
    End,
    Body,
}

impl RangePart {
    /// The region its handle draws.
    pub(crate) fn region(self) -> RegionId {
        RegionId(match self {
            RangePart::Start => 1,
            RangePart::End => 2,
            RangePart::Body => 3,
        })
    }
}
