//! Role: The Navigation page of the game catalog: tabs, wizard steps, pagination.
//! Vertical navigation.

use crate::support::common::{cards, choice_options, list, specimen, strings, wide};
use catalog_shell::{Example, Page};
use kivact::interaction::{Tone, use_toaster};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_game as g;
use std::time::Duration;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new(
        "navigation",
        "Navigation",
        "Console",
        include_str!("navigation.rs"),
    )
    .description("Tabs, wizard steps, pagination and vertical navigation")
    .example(Example::new(
        "navigation",
        "Navigation",
        "Tabs, wizard steps, pagination and vertical navigation.",
        demo,
    ))
}

// example: navigation
fn demo() -> impl IntoView {
    navigation()
}
// end example

// xtask-allow(state): independent: step is the wizard step moved by the back and next buttons and the stepper; page is the separate pagination specimen's binding; no control writes both
/// The navigation specimens.
pub(crate) fn navigation() -> Node {
    let underline = signal("summary".to_owned());
    let boxed_tabs = signal("metrics".to_owned());
    let step = signal(1_usize);
    let page = signal(2_usize);
    cards(
        (
            underline_tabs(underline),
            boxed_tab_examples(boxed_tabs),
            wide(wizard_steps(step)),
            pagination_controls(page),
            vertical_navigation(),
        ),
        460.,
    )
}

fn underline_tabs(underline: kivact::state::Signal<String>) -> impl kivact::view::Element {
    specimen(
        "NAV-01",
        "Tabs (underline)",
        kivact::ui::tabs(
            "Operation view",
            choice_options(&[
                ("summary", "Overview"),
                ("tasks", "Task"),
                ("files", "File"),
                ("audit", "Audit log"),
            ]),
            underline,
        )
        .content(|key| {
            text(match key.as_str() {
                "summary" => "Shows the basic information, owner and deadline of the operation.",
                "tasks" => "8 of 12 tasks are complete.",
                "files" => "4 attachments (18.2 MB in total).",
                "audit" => "Shows the operation history of the last 30 days.",
                _ => "",
            })
            .style(style().font_size(12.5).color(color::TEXT_MUTED))
        }),
    )
}

fn boxed_tab_examples(boxed_tabs: kivact::state::Signal<String>) -> impl kivact::view::Element {
    specimen(
        "NAV-02",
        "Tabs (boxed)",
        kivact::ui::tabs(
            "Record view",
            vec![
                ListItem::new("metrics".to_owned(), "Metrics").with_icon("activity"),
                ListItem::new("logs".to_owned(), "Log").with_icon("terminal-2"),
                ListItem::new("settings".to_owned(), "Settings").with_icon("settings"),
            ],
            boxed_tabs,
        )
        .variant(g::TabsVariant::Boxed)
        .content(|key| {
            text(match key.as_str() {
                "metrics" => "p95 latency 142 ms / error rate 0.04%",
                "logs" => "[09:40:12] INFO worker-3 job=OPS-2041 completed",
                "settings" => "The settings of this environment are read-only.",
                _ => "",
            })
            .style(style().font_size(12.5).color(color::TEXT_MUTED))
        }),
    )
}

fn wizard_steps(step: kivact::state::Signal<usize>) -> impl kivact::view::Element {
    specimen(
        "NAV-03",
        "Stepper (wizard)",
        list((
            kivact::ui::stepper(
                strings(&[
                    "Basic information",
                    "Permission settings",
                    "Integration",
                    "Confirm",
                    "Done",
                ])
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    kivact::ui::StepperItem::new(label).indicator(
                        kivact::ui::StepIndicator::custom(move || {
                            text(move || {
                                if index < step.get() {
                                    "✓".to_owned()
                                } else {
                                    format!("{:02}", index + 1)
                                }
                            })
                        }),
                    )
                })
                .collect::<Vec<_>>(),
                step,
            ),
            row((
                g::mono(move || format!("STEP {} / 5", step.get() + 1)).style(style().flex_grow(1)),
                kivact::ui::button("Back")
                    .variant(kivact::ui::Slant)
                    .size(kivact::ui::ControlSize::Small)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .disabled(move || step.get() == 0)
                    .on_press(move || step.update(|v| *v = v.saturating_sub(1))),
                kivact::ui::button(move || {
                    (if step.get() == 4 { "Done" } else { "Next ›" }).to_owned()
                })
                .variant(kivact::ui::Slant)
                .size(kivact::ui::ControlSize::Small)
                .on_press(move || {
                    if step.get() == 4 {
                        step.set(0);
                        use_toaster().push(
                            Toast::new("Setup complete")
                                .body("The workspace setup is complete.")
                                .tone(Tone::Success)
                                .duration(Some(Duration::from_secs(4))),
                        );
                    } else {
                        step.update(|v| *v += 1);
                    }
                }),
            ))
            .style(style().gap(12).align_items(Align::Center)),
        )),
    )
}

fn pagination_controls(page: kivact::state::Signal<usize>) -> impl kivact::view::Element {
    specimen(
        "NAV-04",
        "Breadcrumb / pagination",
        list((
            kivact::ui::breadcrumb(
                strings(&[
                    "Organization",
                    "Project",
                    "Environment: production",
                    "OPS-2041",
                ])
                .into_iter()
                .enumerate()
                .map(|(index, label)| ListItem::new(index, label))
                .collect::<Vec<_>>(),
            ),
            row((
                kivact::ui::pagination(12, page).item(|item| {
                    text(match item {
                        kivact::ui::PageItem::First => "«".to_owned(),
                        kivact::ui::PageItem::Previous => "‹".to_owned(),
                        kivact::ui::PageItem::Next => "›".to_owned(),
                        kivact::ui::PageItem::Last => "»".to_owned(),
                        kivact::ui::PageItem::Ellipsis => "…".to_owned(),
                        kivact::ui::PageItem::Page(page) => (page + 1).to_string(),
                    })
                }),
                spacer(),
                g::mono(move || {
                    format!(
                        "{}–{} / 289",
                        (page.get() - 1) * 25 + 1,
                        (page.get() * 25).min(289)
                    )
                })
                .style(style().color(color::TEXT_MUTED)),
            ))
            .style(style().align_items(Align::Center)),
        )),
    )
}

fn vertical_navigation() -> impl kivact::view::Element {
    specimen(
        "NAV-05",
        "Vertical navigation",
        kivact::ui::tabs(
            "Main menu",
            vec![
                ListItem::new("dashboard".to_owned(), "Dashboard").with_icon("layout-grid"),
                ListItem::new("operations".to_owned(), "Operation").with_icon("target"),
                ListItem::new("materials".to_owned(), "Materials").with_icon("box"),
                ListItem::new("security".to_owned(), "Security").with_icon("shield-check"),
            ],
            signal("dashboard".to_owned()),
        )
        .orientation(kivact::view::Orientation::Vertical)
        .variant(g::TabsVariant::Navigation)
        .item(|item, state| {
            let count = match item.with_untracked(|item| item.key().clone()).as_str() {
                "operations" => Some(kivact::ui::badge("3").tone(kivact::interaction::Tone::Info)),
                "security" => Some(kivact::ui::badge("!").tone(kivact::interaction::Tone::Danger)),
                _ => None,
            };
            (g::navigation_label(item, state), count)
        }),
    )
}
