//! Role: Assemble the catalog's component demonstrations.
use crate::PageInfo;
use kivact::control::{Node, node};

pub(crate) fn examples(page: &PageInfo) -> Node {
    super::controls::examples(page.slug)
        .or_else(|| super::forms::examples(page.slug))
        .or_else(|| super::navigation::examples(page.slug))
        .or_else(|| super::overlays::examples(page.slug))
        .or_else(|| super::collections::examples(page.slug))
        .or_else(|| super::content::examples(page.slug))
        .or_else(|| super::feedback::examples(page.slug))
        .or_else(|| super::dates::examples(page.slug))
        .or_else(|| super::utilities::examples(page.slug))
        .unwrap_or_else(node)
}
