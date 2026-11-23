//! Role: The catalog page of Rich Text Editor.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "rich-text-editor",
        "Rich Text Editor",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "rich-text-editor",
        "Rich Text Editor",
        "",
        forms::example_rich_text_editor,
    ))
}
