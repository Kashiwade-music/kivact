//! Role: The catalog page of Chat Tool Calls.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "chat-tool-calls",
        "Chat Tool Calls",
        "Chat",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "chat-tool-calls",
        "Chat Tool Calls",
        "",
        collections::example_chat_tool_calls,
    ))
}
