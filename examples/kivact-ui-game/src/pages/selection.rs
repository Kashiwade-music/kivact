//! Role: The Selection page of the game catalog: checkboxes, radio groups, switches.
//! Selects, sliders and selectable cards.

use crate::support::common::{
    cards, choice_options, line, list, meta, segmented, specimen, suppress_search_shortcut,
};
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::state::IntoValue;
use kivact::style::auto;
use kivact::view::Node;
use kivact_ui_game as g;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new(
        "selection",
        "Selection",
        "Console",
        include_str!("selection.rs"),
    )
    .description("Checkboxes, radio groups, switches, selects, sliders and selectable cards")
    .example(Example::new(
        "choices",
        "Choices",
        "Checkboxes, radio groups, switches, selects, sliders and selectable cards.",
        demo,
    ))
}

// example: choices
fn demo() -> impl IntoView {
    selection()
}
// end example

pub(crate) fn selection() -> Node {
    let permissions = signal((true, true, false));
    let all = binding(
        (move || {
            let v = permissions.get();
            v.0 && v.1 && v.2
        })
        .into_value(),
        move |v| permissions.set((v, v, v)),
    );
    let mixed = move || {
        let v = permissions.get();
        (v.0 || v.1 || v.2) && !(v.0 && v.1 && v.2)
    };
    let cpu = signal(72_f64);
    cards(
        (
            checkbox_choices(permissions, all, mixed).on_key(suppress_search_shortcut),
            radio_choices().on_key(suppress_search_shortcut),
            switch_choices().on_key(suppress_search_shortcut),
            select_choices(),
            slider_choices(cpu),
            selectable_cards().on_key(suppress_search_shortcut),
        ),
        330.,
    )
}

pub(crate) fn setting(title: &str, description: &str, control: impl IntoView) -> Node {
    row((
        column((
            text(title.to_owned()).style(style().font_weight(700)),
            text(description.to_owned()).style(style().font_size(11.5).color(color::TEXT_MUTED)),
        ))
        .style(style().flex_grow(1).gap(3)),
        control,
    ))
    .style(
        style()
            .padding_x(14)
            .padding_y(12)
            .gap(16)
            .align_items(Align::Center)
            .border_bottom(1, color::BORDER),
    )
}

fn checkbox_choices(
    permissions: kivact::state::Signal<(bool, bool, bool)>,
    all: kivact::state::Signal<bool>,
    mixed: impl Fn() -> bool + Copy + 'static,
) -> impl kivact::view::Element {
    specimen(
        "SEL-01",
        "Checkboxes",
        list((
            kivact::ui::checkbox("All permissions", all).indeterminate(mixed),
            column((
                kivact::ui::checkbox("Viewer", permissions.project(|v| &v.0, |v| &mut v.0)),
                kivact::ui::checkbox("Edit", permissions.project(|v| &v.1, |v| &mut v.1)),
                kivact::ui::checkbox("Approve", permissions.project(|v| &v.2, |v| &mut v.2)),
                kivact::ui::checkbox("Delete audit log (disabled)", kivact::state::signal(false))
                    .disabled(true),
            ))
            .style(style().padding_left(24).gap(10)),
        )),
    )
}

fn radio_choices() -> impl kivact::view::Element {
    specimen(
        "SEL-02",
        "Radio (diamond)",
        list((
            kivact::ui::radio_group(
                "Sync mode",
                choice_options(&[
                    ("live", "Realtime sync"),
                    ("scheduled", "Periodic sync"),
                    ("manual", "Manual"),
                ]),
                signal(Some("live".to_owned())),
            ),
            kivact::ui::radio_group(
                "External trigger",
                vec![
                    ListItem::new("external".to_owned(), "External trigger")
                        .with_description("Enterprise plan only")
                        .disabled(true),
                ],
                signal(None),
            ),
        )),
    )
}

fn switch_choices() -> impl kivact::view::Element {
    specimen(
        "SEL-03",
        "Switch",
        list((
            setting(
                "Require two-factor authentication",
                "Applies to all member sign-ins",
                kivact::ui::switch("Require two-factor authentication", signal(true))
                    .hide_label(true),
            ),
            setting(
                "Night maintenance notice",
                "02:00–04:00 JST",
                kivact::ui::switch("Night maintenance notice", signal(false)).hide_label(true),
            ),
            setting(
                "Enforce SSO",
                "Locked by an administrator",
                kivact::ui::switch("Enforce SSO", signal(true))
                    .hide_label(true)
                    .disabled(true),
            ),
        )),
    )
}

fn select_choices() -> impl kivact::view::Element {
    specimen(
        "SEL-04",
        "Select",
        list((
            kivact::ui::field(
                "Department",
                kivact::ui::select(
                    choice_options(&[
                        ("maintenance", "Maintenance"),
                        ("information", "Information management"),
                        ("finance", "Accounting"),
                        ("security", "Security office"),
                    ]),
                    signal(Some("maintenance".to_owned())),
                )
                .placeholder("")
                .name("Department"),
            )
            .on_key(suppress_search_shortcut),
            kivact::ui::field(
                "Time zone",
                kivact::ui::select(
                    choice_options(&[
                        ("jst", "JST (UTC+09:00)"),
                        ("utc", "UTC"),
                        ("pst", "PST (UTC-08:00)"),
                    ]),
                    signal(Some("jst".to_owned())),
                )
                .placeholder("")
                .name("Time zone"),
            )
            .hint(|| "Choose the time zone for displayed times."),
        )),
    )
}

fn slider_choices(cpu: kivact::state::Signal<f64>) -> impl kivact::view::Element {
    specimen(
        "SEL-05",
        "Slider / segments",
        list((
            kivact::ui::field(
                "CPU limit",
                line((
                    kivact::ui::slider(cpu)
                        .label("CPU limit")
                        .test_id("game-cpu-slider")
                        .style(style().flex_grow(1).width(auto()))
                        .test_id("game-cpu-slider")
                        .style(style().flex_grow(1).width(auto())),
                    meta(move || format!("{:.0}%", cpu.get())),
                )),
            )
            .hint(
                || "0                 25                 50                 75                 100",
            )
            .on_key(suppress_search_shortcut),
            kivact::ui::field(
                "Log level",
                segmented(
                    "Log level",
                    choice_options(&[
                        ("debug", "DEBUG"),
                        ("info", "INFO"),
                        ("warn", "WARN"),
                        ("error", "ERROR"),
                    ]),
                    signal(Some("info".to_owned())),
                ),
            ),
        )),
    )
}

fn selectable_cards() -> impl kivact::view::Element {
    specimen(
        "SEL-06",
        "Selectable cards",
        kivact::ui::radio_group(
            "Plan",
            vec![
                ListItem::new("standard".to_owned(), "STANDARD").with_description("10 seats"),
                ListItem::new("tactical".to_owned(), "TACTICAL").with_description("50 seats · SSO"),
                ListItem::new("command".to_owned(), "COMMAND")
                    .with_description("Unlimited · audited"),
            ],
            signal(Some("tactical".to_owned())),
        )
        .variant(g::ChoiceCard)
        .item(|item, state| {
            let price = match item.with_untracked(|item| item.key().clone()).as_str() {
                "standard" => "¥9,800/mo",
                "tactical" => "¥39,800/mo",
                _ => "To discuss",
            };
            (
                g::card_label(item, state),
                g::mono(price).style(style().color(color::ACCENT)),
            )
        }),
    )
}
