//! Catalog page of Questionnaire: choices, input, optional steps and submit.

use catalog_shell::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("questionnaire", "Questionnaire", "Components", include_str!("questionnaire.rs"))
        .description("A component for answering several questions in turn.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .align(PreviewAlign::End)
        .example(Example::new(
            "basic",
            "Basic",
            "Pass the steps and the current position. A required question keeps Next disabled until it is answered.",
            basic,
        ))
        .example(Example::new(
            "answers",
            "Answers and submit",
            "The third argument, answers, is the binding that holds the answers. It is received at the end through on_submit.",
            answers,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let steps = vec![
        basic::QuestionnaireStep::choices(
            "How did you hear about us?",
            vec![
                basic::QuestionnaireChoice::new("search", "Search").description("Google or Bing"),
                basic::QuestionnaireChoice::new("friend", "A friend"),
                basic::QuestionnaireChoice::new("other", "Somewhere else"),
            ],
        )
        .description("Pick one."),
        basic::QuestionnaireStep::input("What is your name?")
            .placeholder("Ada")
            .optional(true),
    ];
    let step = signal(0usize);
    basic::questionnaire(steps, step, signal(Vec::new())).style(style().width(420))
}
// end example

// example: answers
fn answers() -> impl IntoView {
    let steps = vec![
        basic::QuestionnaireStep::choices(
            "How did you hear about us?",
            vec![
                basic::QuestionnaireChoice::new("search", "Search").description("Google or Bing"),
                basic::QuestionnaireChoice::new("friend", "A friend"),
                basic::QuestionnaireChoice::new("other", "Somewhere else"),
            ],
        )
        .description("Pick one."),
        basic::QuestionnaireStep::input("What is your name?")
            .placeholder("Ada")
            .optional(true),
    ];
    let step = signal(0usize);
    let answers = signal(Vec::<basic::QuestionnaireAnswer>::new());
    let sent = signal(String::new());
    column((
        basic::questionnaire(steps, step, answers)
            .on_submit(move |a: Vec<basic::QuestionnaireAnswer>| sent.set(format!("{a:?}")))
            .style(style().width(420)),
        basic::muted(move || format!("Submitted: {}", sent.get())),
    ))
    .style(style().gap(12))
}
// end example
