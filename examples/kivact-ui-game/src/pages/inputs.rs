//! Role: The Inputs page of the game catalog: text, number and validated inputs.
//! File inputs, with tags and the entry of a value.

use crate::support::common::{cards, line, list, specimen, strings, suppress_search_shortcut};
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_game as g;
use std::time::Duration;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new("inputs", "Inputs", "Console", include_str!("inputs.rs"))
        .description("Text, number, validated and file inputs, with tags and the entry of a value")
        .example(Example::new(
            "fields",
            "Fields",
            "Text, number, validated and file inputs, and a tag entry.",
            demo,
        ))
}

// example: fields
fn demo() -> impl IntoView {
    let search = node_ref();
    inputs(search)
}
// end example

pub(crate) fn inputs(search: NodeRef) -> Node {
    let tags = signal(strings(&["infra", "Periodic inspection"]));
    let code = signal("OPS_2041!".to_owned());
    let invalid = move || {
        let value = code.get();
        value.is_empty()
            || !value
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    };
    let memo = signal(String::new());
    let files = signal(vec![("inventory_2026Q3.csv".to_owned(), 64_f64)]);
    cards(
        (
            text_fields(code, invalid),
            validated_fields(search),
            text_and_numbers(memo),
            file_upload(files),
            tag_entry(tags),
        ),
        330.,
    )
}

fn text_fields(
    code: kivact::state::Signal<String>,
    invalid: impl Fn() -> bool + Copy + 'static,
) -> impl kivact::view::Element {
    specimen(
        "INP-01",
        "Text fields",
        list((
            kivact::ui::field(
                "Operation name *",
                kivact::ui::text_input(signal("Section 3 periodic inspection".to_owned()))
                    .label("Operation name")
                    .test_id("game-title-input"),
            )
            .hint("REQUIRED · used as the title in the list and in email notices."),
            kivact::ui::field(
                "Access code",
                kivact::ui::text_input(code)
                    .label("Access code")
                    .invalid(invalid)
                    .style(move || {
                        let _t = &use_theme().get();
                        style()
                            .border_left(
                                3,
                                if invalid() {
                                    color::DANGER
                                } else {
                                    color::SUCCESS
                                },
                            )
                            .border_color(if invalid() {
                                color::DANGER
                            } else {
                                color::SUCCESS
                            })
                    }),
            )
            .hint("The code is available")
            .error(move || {
                invalid()
                    .then(|| "Only uppercase letters, digits and hyphens are allowed".to_owned())
            }),
            kivact::ui::field(
                "Endpoint",
                kivact::ui::text_input(signal("api.ops.example.internal".to_owned()))
                    .label("Endpoint")
                    .style(move || {
                        let _t = &use_theme().get();
                        style()
                            .border_left(3, color::SUCCESS)
                            .border_color(color::SUCCESS)
                    }),
            )
            .hint("Reachable (38 ms)"),
        )),
    )
}

fn validated_fields(search: NodeRef) -> impl kivact::view::Element {
    specimen(
        "INP-02",
        "Decorated inputs",
        list((
            kivact::ui::field(
                "Search",
                kivact::ui::input_group(
                    "",
                    (kivact::ui::text_input(kivact::state::signal(String::new()))
                        .label("Search")
                        .with_node(|node| node.node_ref(search))
                        .test_id("game-search-input")
                        .placeholder("ID, title or owner")
                        .style(style().width(pct(100)).padding_left(34).padding_right(52)),),
                )
                .variant(g::InputGroupVariant::Adorned)
                .start(kivact::view::icon("search"))
                .end(g::mono("/")),
            ),
            kivact::ui::field(
                "Webhook URL",
                kivact::ui::input_group(
                    "",
                    (kivact::ui::text_input(kivact::state::signal(String::new()))
                        .label("Webhook URL")
                        .placeholder("hooks.example.com/ops"),),
                )
                .variant(g::InputGroupVariant::Inline)
                .start(g::mono("https://"))
                .end(g::mono("POST")),
            ),
            kivact::ui::field(
                "Monthly budget",
                kivact::ui::input_group(
                    "",
                    (kivact::ui::text_input(signal("1,200,000".to_owned()))
                        .label("Monthly budget")
                        .style(style().width(pct(100)).padding_left(34).padding_right(52)),),
                )
                .variant(g::InputGroupVariant::Adorned)
                .start(kivact::view::icon("database"))
                .end(g::mono("JPY")),
            ),
            kivact::ui::field(
                "API key ID",
                kivact::ui::text_input(signal("key_7f3a…c91e".to_owned()))
                    .label("API key ID")
                    .read_only(true)
                    .on_key(suppress_search_shortcut)
                    .style(style().background(color::SURFACE_ALT)),
            ),
        )),
    )
}

fn text_and_numbers(memo: kivact::state::Signal<String>) -> impl kivact::view::Element {
    specimen(
        "INP-03",
        "Text area / number",
        list((
            kivact::ui::field(
                "Work notes",
                kivact::ui::text_area(memo)
                    .label("Work notes")
                    .max_length(200)
                    .placeholder("Enter handover notes…"),
            )
            .hint(move || format!("{} / 200", memo.get().chars().count())),
            line((
                kivact::ui::field(
                    "Concurrency",
                    kivact::ui::number_input(
                        signal(4.0_f64),
                        kivact::interaction::ScalarRange::new(1.0, 32.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("invariant: the bounds and the step are finite"),
                    )
                    .label("Concurrency"),
                )
                .style(style().width(140)),
                kivact::ui::field(
                    "Region",
                    kivact::ui::text_input(signal("ap-northeast-1".to_owned()))
                        .label("Region")
                        .disabled(true),
                )
                .style(style().width(150)),
            )),
        )),
    )
}

fn file_upload(files: kivact::state::Signal<Vec<(String, f64)>>) -> impl kivact::view::Element {
    specimen(
        "INP-04",
        "File upload",
        list((
            kivact::ui::dropzone(kivact::i18n::message("game.dropzone.label", "Upload files"))
                .multiple(true)
                .pick_on_press(true)
                .icon(kivact::view::icon("cloud-upload"))
                .on_files(move |paths| {
                    for file in paths {
                        let path = file.path;
                        let name = path
                            .file_name()
                            .map(|v| v.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.display().to_string());
                        let index = files.with_untracked(Vec::len);
                        files.update(|v| v.push((name, 0.)));
                        spawn(async move {
                            for _ in 0..10 {
                                kivact::task::sleep(Duration::from_millis(220)).await;
                                files.update(|v| {
                                    if let Some(file) = v.get_mut(index) {
                                        file.1 = (file.1 + 10.).min(100.);
                                    }
                                });
                            }
                        });
                    }
                })
                .hint(kivact::i18n::message("game.dropzone.hint", "CSV / JSON"))
                .filters(vec![(
                    "CSV / JSON".to_owned(),
                    vec!["csv".to_owned(), "json".to_owned()],
                )]),
            for_index(files, |index, item: Value<(String, f64)>| {
                row((
                    dynamic(
                        move || item.get().1 >= 100.,
                        |done| {
                            kivact::view::icon(if done { "circle-check" } else { "file-text" })
                                .style(style().color(if done {
                                    color::SUCCESS
                                } else {
                                    color::ACCENT
                                }))
                        },
                    ),
                    column((
                        text(move || item.get().0).style(style().font_weight(700).font_size(12)),
                        kivact::ui::progress(move || Some(item.get().1 / 100.0))
                            .name("File processing")
                            .variant(kivact::interaction::Tone::Info)
                            .variant(kivact::ui::Striped)
                            .moving(true)
                            .size(kivact::ui::ControlSize::Small),
                    ))
                    .style(style().flex_grow(1).gap(4)),
                    g::mono(move || format!("{:.0}%", item.get().1))
                        .test_id(format!("game-file-progress-{index}")),
                ))
                .style(
                    style()
                        .padding_x(10)
                        .padding_y(8)
                        .align_items(Align::Center)
                        .gap(10)
                        .border(1, color::BORDER)
                        .background(color::SURFACE_ALT),
                )
            }),
        )),
    )
}

fn tag_entry(tags: kivact::state::Signal<Vec<String>>) -> impl kivact::view::Element {
    specimen(
        "INP-05",
        "Tag input",
        kivact::ui::field("Label", kivact::ui::tag_input(tags).label("Add a tag")),
    )
}
