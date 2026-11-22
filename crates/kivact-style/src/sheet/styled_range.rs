//! Role: The style of a UTF-8 byte range within one editing line or span.

use super::Style;

/// The style of a UTF-8 byte range within one editing line: its text
/// properties (color, decoration, weight, ...) and `background`, as a
/// [`Style`]. A span of rich text takes the same vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub struct StyledRange {
    pub range: std::ops::Range<usize>,
    pub style: Style,
}
