//! Role: The buttons page: the button variants, sizes, groups and the standard buttons.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::state::{Machine, Step, machine};
use kivact::style::rgba;
use kivact::view::Node;
use kivact_ui_pop as ui;
use std::time::Duration;
/// The page of the buttons.
pub fn page() -> Page {
    Page::new(
        "buttons",
        "Buttons",
        "Components",
        include_str!("buttons.rs"),
    )
    .description(
        "Buttons come in variants, sizes and tones; the standard button takes the same theme.",
    )
    .example(Example::new(
        "buttons",
        "Button variants",
        "The primary action is blue; the special moment is yellow.",
        buttons_demo,
    ))
}

/// The sync button: idle, or busy until the delayed finish event arrives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SyncPhase {
    #[default]
    Idle,
    Running,
}

enum SyncEvent {
    Start,
    Finish,
}

/// What the sync machine asks the page to do after a transition.
enum SyncEffect {
    Finished,
}

struct SyncMachine;

impl Machine for SyncMachine {
    type State = SyncPhase;
    type Event = SyncEvent;
    type Effect = SyncEffect;
    fn transition(
        &self,
        state: &SyncPhase,
        event: SyncEvent,
    ) -> Step<SyncPhase, SyncEvent, SyncEffect> {
        match (state, event) {
            (SyncPhase::Idle, SyncEvent::Start) => {
                Step::to(SyncPhase::Running).after(Duration::from_millis(1600), SyncEvent::Finish)
            }
            (SyncPhase::Running, SyncEvent::Finish) => {
                Step::to(SyncPhase::Idle).with(SyncEffect::Finished)
            }
            _ => Step::stay(),
        }
    }
}

fn buttons_demo() -> impl IntoView {
    // example: buttons
    let notify = notify();
    use kivact_ui_pop::ButtonTone;
    let variants = [
        ("Primary", ButtonVariant::Primary, None),
        ("Accent", ButtonVariant::Primary, Some(ButtonTone::Accent)),
        (
            "Secondary",
            ButtonVariant::Secondary,
            Some(ButtonTone::Soft),
        ),
        ("Outline", ButtonVariant::Secondary, None),
        ("Ghost", ButtonVariant::Ghost, None),
        ("Delete", ButtonVariant::Danger, None),
    ]
    .into_iter()
    .map(|(label, v, hue)| {
        let button = kivact::ui::button(label).variant(v);
        let button = match hue {
            Some(hue) => button.variant(hue),
            None => button,
        };
        button.on_press(move || notify.call((ui::Hue::Blue, "Action".into(), label.into())))
    })
    .chain(std::iter::once(
        kivact::ui::button("Disabled").disabled(true),
    ))
    .collect::<Vec<_>>();
    section(
        "02",
        "Buttons",
        "Buttons: the primary action is blue; a special moment is yellow.",
        cards_grid((
            half(card("Variants", wrap(variants))),
            half(card(
                "Slanted (game UI style)",
                wrap((
                    kivact::ui::button("Assemble").variant(kivact::ui::Slant),
                    kivact::ui::button("Deploy")
                        .variant(kivact_ui_pop::ButtonTone::Accent)
                        .variant(kivact::ui::Slant)
                        .icon(kivact::view::icon("sword")),
                    kivact::ui::button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .variant(kivact::ui::Slant),
                    kivact::ui::button("Details")
                        .variant(kivact_ui_pop::ButtonTone::Soft)
                        .size(kivact::ui::ControlSize::Small)
                        .variant(kivact::ui::Slant),
                )),
            )),
            half(button_sizes()),
            half(button_groups(notify)),
            half(card(
                "Standard components (kivact::ui): the theme's recipes give them this kit's look",
                column((
                    wrap((
                        kivact::ui::button("Standard button").test_id("standard-button"),
                        kivact::ui::button("Standard ghost")
                            .variant(kivact::ui::ButtonVariant::Ghost),
                        kivact::ui::button("Standard disabled").disabled(true),
                    )),
                    wrap((
                        kivact::ui::checkbox("Standard checkbox", signal(true)),
                        kivact::ui::switch("Standard switch", signal(true)),
                    )),
                    kivact::ui::text_input(signal("Input value".to_string()))
                        .label("Standard input")
                        .style(style().width(240)),
                ))
                .style(style().gap(16)),
            )),
        )),
    )
    // end example
}

fn button_sizes() -> kivact::ui::Card {
    card(
        "Sizes and icons",
        column((
            wrap((
                kivact::ui::button("Small")
                    .size(kivact::ui::ControlSize::Small)
                    .icon(kivact::view::icon("plus")),
                kivact::ui::button("Medium").icon(kivact::view::icon("plus")),
                kivact::ui::button("Large")
                    .size(kivact::ui::ControlSize::Large)
                    .icon(kivact::view::icon("plus")),
            )),
            wrap((
                kivact::ui::button("Export")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .trailing(kivact::view::icon("download")),
                kivact::ui::tooltip(
                    "Add",
                    kivact::ui::button("Add")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("plus"))
                        .style(style().background(color::ACCENT).color(Color::WHITE)),
                )
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.0),
                kivact::ui::tooltip(
                    "Edit",
                    kivact::ui::button("Edit")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("pencil"))
                        .style(style().background(color::SURFACE_ALT)),
                )
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.0),
                kivact::ui::tooltip(
                    "Copy",
                    kivact::ui::button("Copy")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("copy"))
                        .style(style().border(1, color::BORDER)),
                )
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.0),
                kivact::ui::tooltip(
                    "More",
                    kivact::ui::button("More")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots")),
                )
                .placement(kivact::view::Placement::Below)
                .align(kivact::view::OverlayAlign::Center)
                .offset(6.0),
                node()
                    .role(Role::Button)
                    .name("New")
                    .focusable(true)
                    .style(
                        style()
                            .size(44, 44)
                            .rotate(45)
                            .border_radius(10)
                            .background(rgb(0xffdc3c))
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .focus(style().outline(3, color::FOCUS)),
                    )
                    .child(kivact::view::icon("sparkles").style(style().rotate(-45))),
            )),
        ))
        .style(style().gap(16)),
    )
}

fn button_groups(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::Card {
    let sync = machine(
        SyncPhase::Idle,
        || SyncMachine,
        move |_, effect| match effect {
            SyncEffect::Finished => notify.call((
                ui::Hue::Mint,
                "Sync complete".into(),
                "The latest data has been fetched.".into(),
            )),
        },
    );
    card(
        "Group and loading",
        column((
            wrap((
                view_mode_buttons(),
                kivact::ui::button(move || {
                    if sync.get() == SyncPhase::Running {
                        "Syncing…"
                    } else {
                        "Sync"
                    }
                    .to_string()
                })
                .icon(kivact::view::icon("refresh"))
                .busy(move || sync.get() == SyncPhase::Running)
                .on_press(move || sync.send(SyncEvent::Start)),
            )),
            wrap((
                row((
                    kivact::ui::button("Save and publish")
                        .style(style().border_radius(0))
                        .on_press(move || {
                            notify.call((
                                ui::Hue::Mint,
                                "Saved".into(),
                                "Your changes are live.".into(),
                            ))
                        }),
                    kivact::ui::menu(
                        "Save options",
                        vec![
                            MenuEntry::item("draft", "Save as draft"),
                            MenuEntry::item("schedule", "Schedule publish…"),
                        ],
                    )
                    .on_activate(|_: &'static str| {})
                    .attach(
                        kivact::ui::button("Save options")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("chevron-down"))
                            .style(
                                style()
                                    .border_radius(0)
                                    .background(color::ACCENT)
                                    .color(Color::WHITE)
                                    .border_left(1, rgba(0xffffff, 0.2)),
                            ),
                    ),
                ))
                .style(
                    style()
                        .border_radius(8)
                        .overflow(kivact::style::Overflow::Hidden),
                ),
                kivact::ui::button("Show more")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .size(kivact::ui::ControlSize::Small)
                    .trailing(kivact::view::icon("arrow-right")),
            )),
        ))
        .style(style().gap(16)),
    )
}

fn view_mode_buttons() -> Node {
    row((
        kivact::ui::button("Grid")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("layout-grid"))
            .style(style().border_radius_corners(8, 0, 0, 8)),
        kivact::ui::button("List")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("list"))
            .style(style().border_radius(0).margin_left(-1)),
        kivact::ui::button("Filter")
            .variant(kivact::ui::ButtonVariant::Ghost)
            .hide_label(true)
            .icon(kivact::view::icon("filter"))
            .style(
                style()
                    .border(1, color::BORDER)
                    .border_radius_corners(0, 8, 8, 0)
                    .margin_left(-1),
            ),
    ))
}
