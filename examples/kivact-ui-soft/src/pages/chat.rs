//! Role: The catalog page of Chat.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("chat", "Chat", "Chat", include_str!("collections.rs")).example(Example::new(
        "chat",
        "Chat",
        "",
        collections::example_chat,
    ))
}
