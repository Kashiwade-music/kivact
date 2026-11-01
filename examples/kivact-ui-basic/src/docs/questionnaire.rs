//! Role: Catalog page of Questionnaire: choices, input, optional steps and submit.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "questionnaire",
        name: "Questionnaire",
        description: "複数の質問を順に答えていく部品です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("questionnaire.rs"),
        align: PreviewAlign::End,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "steps と現在の位置を渡します。必須の質問は答えるまで Next を押せません。",
                basic,
            ),
            Example::new(
                "answers",
                "Answers and submit",
                "三つ目の引数の answers が回答を持つ binding です。on_submit で最後に受け取ります。",
                answers,
            ),
        ],
    }
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
    let answers = signal(Vec::<Option<String>>::new());
    let sent = signal(String::new());
    column((
        basic::questionnaire(steps, step, answers)
            .on_submit(move |a: Vec<Option<String>>| sent.set(format!("{a:?}")))
            .style(style().width(420)),
        basic::muted(move || format!("Submitted: {}", sent.get())),
    ))
    .style(style().gap(12))
}
// end example
