//! Role: The catalog page of Tokenizer.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "tokenizer",
        "Tokenizer",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "tokenizer",
        "Tokenizer",
        "",
        forms::example_tokenizer,
    ))
}
