//! Role: The catalog page of Chat Dictation Button.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "chat-dictation-button",
        "Chat Dictation Button",
        "Chat",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "chat-dictation-button",
        "Chat Dictation Button",
        "",
        collections::example_chat_dictation_button,
    ))
}
