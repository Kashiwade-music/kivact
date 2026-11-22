//! Role: The dialogs, drawer and command palette that a page demo can open.

use super::common::list;
use super::state::{Demo, Overlay};
use kivact::interaction::use_toaster;
use kivact::state::{Machine, Step, machine};
use kivact::view::{rich_text, span};

use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_game as g;
use std::time::Duration;

/// What a dialog's field holds: no text while closed, the text while open.
#[derive(Clone, PartialEq)]
enum Editing {
    Closed,
    Open { title: String },
}

enum EditingEvent {
    Open(bool),
    Title(String),
}

struct EditingInput;

impl Machine for EditingInput {
    type State = Editing;
    type Event = EditingEvent;
    type Effect = ();
    fn transition(&self, state: &Editing, event: EditingEvent) -> Step<Editing, EditingEvent, ()> {
        match event {
            EditingEvent::Open(true) if matches!(state, Editing::Closed) => {
                Step::to(Editing::Open {
                    title: String::new(),
                })
            }
            EditingEvent::Open(true) => Step::stay(),
            EditingEvent::Open(false) => Step::to(Editing::Closed),
            EditingEvent::Title(title) => match state {
                Editing::Open { .. } => Step::to(Editing::Open { title }),
                Editing::Closed => Step::stay(),
            },
        }
    }
}

/// The field of the dialog that `which` opens: empty when it opens, gone when
/// it closes (the dialog's open binding and the text are one state).
fn editing(overlay: Signal<Overlay>, which: Overlay) -> Signal<String> {
    let initial = if overlay.get_untracked() == which {
        Editing::Open {
            title: String::new(),
        }
    } else {
        Editing::Closed
    };
    let dialog = machine(initial, || EditingInput, |_, ()| {});
    dialog.feed(overlay, move |open| EditingEvent::Open(open == which));
    let state = dialog.state();
    binding(
        move || match state.get() {
            Editing::Open { title } => title,
            Editing::Closed => String::new(),
        },
        move |title| dialog.send(EditingEvent::Title(title)),
    )
}

pub(crate) fn create_dialog(scene: Demo) -> impl IntoView {
    let title = editing(scene.overlay, Overlay::Create);
    kivact::ui::dialog("New operation", scene.open(Overlay::Create))
        .content(move || {
            list((
                kivact::ui::field(
                    "Title *",
                    kivact::ui::text_input(title)
                        .label("New operation name")
                        .placeholder("e.g. Section 4 night patrol"),
                ),
                row((
                    kivact::ui::field(
                        "Owner",
                        kivact::ui::select(
                            crate::support::common::choice_options(&[
                                ("maintenance", "Maintenance"),
                                ("information", "Information management"),
                                ("security", "Security office"),
                            ]),
                            signal(Some("maintenance".to_owned())),
                        )
                        .placeholder("")
                        .name("New operation department"),
                    )
                    .style(style().flex_grow(1).flex_basis(0)),
                    kivact::ui::field(
                        "Deadline",
                        kivact::ui::text_input(signal("2026-10-10".to_owned()))
                            .label("New operation deadline"),
                    )
                    .style(style().flex_grow(1).flex_basis(0)),
                ))
                .style(style().gap(12)),
                kivact::ui::checkbox("Notify the owner after creating", signal(true)),
            ))
        })
        .footer(move || {
            (
                kivact::ui::button("Cancel")
                    .variant(kivact::ui::Slant)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || scene.hide(Overlay::Create)),
                kivact::ui::button("Create")
                    .variant(kivact::ui::Slant)
                    .disabled(move || title.get().trim().is_empty())
                    .on_press(move || {
                        scene.hide(Overlay::Create);
                        use_toaster().push(
                            Toast::new("Operation created")
                                .body(title.get())
                                .tone(Tone::Success)
                                .duration(Some(Duration::from_secs(4))),
                        );
                    }),
            )
        })
}

pub(crate) fn delete_dialog(scene: Demo) -> impl IntoView {
    let confirmation = editing(scene.overlay, Overlay::Delete);
    // An irreversible question: an outside press does not answer it.
    kivact::ui::dialog("Delete operation", scene.open(Overlay::Delete))
        .mode(kivact::ui::DialogKind::AlertModal)
        .content(move || {
            list((
                row((
                    node().child(kivact::view::icon("alert-triangle").style(style().size(24, 24))).style(style().size(44, 44).align_items(Align::Center).justify_content(Justify::Center).background(rgb(0xfde4e8)).color(color::DANGER)),
                    column((
                        text("Delete the selected operations.").style(style().font_weight(700)),
                        text("Related tasks and attachments are deleted too, and this cannot be undone. Type DELETE to confirm.").style(style().color(color::TEXT_MUTED).font_size(13)),
                    )).style(style().flex_grow(1).gap(4)),
                )).style(style().gap(14)),
                kivact::ui::text_input(confirmation).label("DELETE confirmation")
                    .placeholder("DELETE"),
            ))
        }).footer(move || {
            (
                kivact::ui::button("Cancel").variant(kivact::ui::Slant)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || scene.hide(Overlay::Delete)),
                kivact::ui::button("Delete").variant(kivact::ui::Slant)
                    .icon(kivact::view::icon("trash"))
                    .variant(kivact::ui::ButtonVariant::Danger)
                    .disabled(move || confirmation.get() != "DELETE")
                    .on_press(move || {
                        let ids = scene.selected.get();
                        scene
                            .rows
                            .update(|rows| rows.retain(|r| !ids.contains(&r.id)));
                        scene.selected.set(Vec::new());
                        scene.hide(Overlay::Delete);
                        use_toaster().push(Toast::new("Deleted").body(format!("Deleted {} operations.", ids.len())).tone(Tone::Danger).duration(Some(Duration::from_secs(4))));
                    }),
            )
        })
}

pub(crate) fn drawer_body() -> Node {
    list((
        kivact::ui::description_list(
            (vec![
                (
                    "Status".into(),
                    kivact::ui::badge("RUNNING")
                        .tone(kivact::interaction::Tone::Info)
                        .into_view(),
                ),
                ("Owner".into(), text("Maintenance / Mashiba").into_view()),
                ("Due".into(), g::mono("2026-10-10").into_view()),
            ])
            .into_iter()
            .map(|(term, detail): (String, _)| kivact::ui::description(term, detail))
            .collect::<Vec<_>>(),
        ),
        row((
            text("Progress").style(style().width(46).font_size(12)),
            kivact::ui::progress(Some(0.68))
                .name("Operation progress")
                .variant(kivact::interaction::Tone::Info)
                .variant(kivact::ui::Striped)
                .moving(true)
                .style(style().flex_grow(1)),
            g::mono("68%"),
        ))
        .style(style().gap(10).align_items(Align::Center)),
        g::activity(vec![
            g::ActivityEntry::new("09:40", "Start work", kivact::interaction::Tone::Info),
            g::ActivityEntry::new(
                "08:30",
                rich_text(vec![span("Mashiba").font_weight(700), span(" approved")]),
                kivact::interaction::Tone::Success,
            ),
        ]),
    ))
}

/// The commands of the catalog's command palette.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PaletteCommand {
    Create,
    Notify,
    Drawer,
}

/// The dialogs, the drawer and the command palette of a page. Each one is
/// closed until the demo of the page opens it.
pub(crate) fn overlay_layer(scene: Demo) -> Node {
    column((
        create_dialog(scene),
        delete_dialog(scene),
        drawer_dialog(scene),
        kivact::ui::command_palette(palette_commands(), scene.open(Overlay::Palette))
            .on_activate(move |command| run_command(scene, command))
            .footer(g::mono("Up/Down move     Enter run     Esc close")),
    ))
}

fn palette_commands() -> Vec<kivact::interaction::Command<PaletteCommand>> {
    vec![
        kivact::interaction::Command::new(PaletteCommand::Create, "New operation")
            .with_hint("DIALOG"),
        kivact::interaction::Command::new(PaletteCommand::Notify, "Send a test notification")
            .with_hint("TOAST"),
        kivact::interaction::Command::new(PaletteCommand::Drawer, "Open the details drawer")
            .with_hint("DRAWER"),
    ]
}

fn run_command(scene: Demo, command: PaletteCommand) {
    match command {
        PaletteCommand::Create => scene.show(Overlay::Create),
        PaletteCommand::Notify => {
            kivact::interaction::use_toaster().push(
                Toast::new("Notice")
                    .body("The system is running normally.")
                    .tone(Tone::Info)
                    .duration(Some(Duration::from_secs(4))),
            );
        }
        PaletteCommand::Drawer => scene.show(Overlay::Drawer),
    }
}

fn drawer_dialog(scene: Demo) -> impl IntoView {
    kivact::ui::dialog("Operation details", scene.open(Overlay::Drawer))
        .mode(kivact::ui::DialogKind::Drawer(
            kivact::geometry::Edge::Right,
        ))
        .content(drawer_body)
        .footer(move || {
            (
                kivact::ui::button("Close")
                    .variant(kivact::ui::Slant)
                    .test_id("game-drawer-footer-close")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || scene.hide(Overlay::Drawer)),
                kivact::ui::button("Edit")
                    .variant(kivact::ui::Slant)
                    .test_id("game-drawer-footer-edit")
                    .on_press(move || scene.hide(Overlay::Drawer)),
            )
        })
}
