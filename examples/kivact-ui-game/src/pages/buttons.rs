//! Role: The Buttons page of the game catalog: primary and secondary actions.
//! Their sizes, states, groups and toolbars.

use crate::support::common::{cards, choice_options, line, list, segmented, specimen};
use crate::support::dialogs::overlay_layer;
use crate::support::state::{Demo, Overlay};
use catalog_shell::{Example, Page};
use kivact::interaction::{Tone, use_toaster};
use kivact::platform;
use kivact::prelude::*;
use kivact::state::{Machine, MachineHandle, Step, machine};
use kivact::view::Node;
use kivact_ui_game as g;
use std::time::Duration;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new("buttons", "Buttons", "Console", include_str!("buttons.rs"))
        .description("Primary and secondary actions, their sizes, states, groups and toolbars")
        .example(Example::new(
            "specimens",
            "Specimens",
            "The console buttons in their looks, sizes, states, groups and toolbars.",
            demo,
        ))
}

// example: specimens
fn demo() -> impl IntoView {
    let scene = Demo::new();
    column((buttons(scene), overlay_layer(scene)))
}
// end example

/// Whether a button's one-shot task runs.
#[derive(Clone, Copy, PartialEq)]
enum Task {
    Idle,
    Running,
}

enum TaskEvent {
    Start,
    Expire,
}

struct TaskInput;

impl Machine for TaskInput {
    type State = Task;
    type Event = TaskEvent;
    type Effect = ();
    fn transition(&self, state: &Task, event: TaskEvent) -> Step<Task, TaskEvent, ()> {
        match (event, state) {
            (TaskEvent::Start, Task::Idle) => {
                Step::to(Task::Running).after(Duration::from_millis(1400), TaskEvent::Expire)
            }
            (TaskEvent::Expire, Task::Running) => Step::to(Task::Idle),
            _ => Step::stay(),
        }
    }
}

/// A task's machine and the busy value its button shows.
fn task() -> (MachineHandle<TaskInput>, Value<bool>) {
    let task = machine(Task::Idle, || TaskInput, |_, ()| {});
    let busy = memo(move || task.get() == Task::Running);
    (task, busy)
}

// xtask-allow(state): independent: grid_on and filter_on are the two toggle buttons of the button toolbar, each flipped only by its own on_press
pub(crate) fn buttons(scene: Demo) -> Node {
    let (sync, sync_busy) = task();
    let (verify, verify_busy) = task();
    let period = signal(Some("7d".to_owned()));
    let grid_on = signal(true);
    let filter_on = signal(false);
    cards(
        (
            button_variants(scene),
            button_sizes(scene),
            button_states(sync, sync_busy, verify, verify_busy),
            button_group(scene, period),
            button_toolbar(scene, grid_on, filter_on),
            chevron_action(text(
                "The one most important action on a screen. A normal form uses the primary button.",
            )),
        ),
        330.,
    )
}

pub(crate) fn separator() -> Node {
    node().style(style().size(1, 18).margin_x(4).background(color::BORDER))
}

fn button_variants(scene: Demo) -> impl kivact::view::Element {
    specimen(
        "BTN-01",
        "Variants",
        line((
            kivact::ui::button("Approve")
                .variant(kivact::ui::Slant)
                .test_id("game-primary")
                .on_press(move || {
                    use_toaster().push(
                        Toast::new("Approve")
                            .body("Operation approved.")
                            .tone(Tone::Success)
                            .duration(Some(Duration::from_secs(3))),
                    );
                }),
            kivact::ui::button("Save draft")
                .variant(kivact::ui::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary),
            kivact::ui::button("Cancel")
                .variant(kivact::ui::Slant)
                .variant(kivact::ui::ButtonVariant::Ghost),
            kivact::ui::button("Delete")
                .variant(kivact::ui::Slant)
                .icon(kivact::view::icon("trash"))
                .variant(kivact::ui::ButtonVariant::Danger)
                .on_press(move || scene.show(Overlay::Delete)),
            kivact::ui::button("Console")
                .variant(kivact::ui::Slant)
                .icon(kivact::view::icon("terminal-2"))
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || scene.show(Overlay::Palette)),
        )),
    )
}

fn button_sizes(scene: Demo) -> impl kivact::view::Element {
    specimen(
        "BTN-02",
        "Sizes / icons",
        line((
            kivact::ui::button("Small")
                .variant(kivact::ui::Slant)
                .size(kivact::ui::ControlSize::Small),
            kivact::ui::button("Medium").variant(kivact::ui::Slant),
            kivact::ui::button("New")
                .variant(kivact::ui::Slant)
                .icon(kivact::view::icon("plus"))
                .size(kivact::ui::ControlSize::Large)
                .on_press(move || scene.show(Overlay::Create)),
            kivact::ui::tooltip(
                "Download",
                kivact::ui::button("Download")
                    .variant(kivact::ui::Slant)
                    .icon(kivact::view::icon("download"))
                    .hide_label(true)
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .style(style().width(34).padding_x(0)),
            )
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.),
            kivact::ui::tooltip(
                "Add",
                kivact::ui::button("Add")
                    .variant(kivact::ui::Slant)
                    .icon(kivact::view::icon("plus"))
                    .hide_label(true)
                    .style(style().width(34).padding_x(0)),
            )
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.),
        )),
    )
}

fn button_states(
    sync: MachineHandle<TaskInput>,
    sync_busy: Value<bool>,
    verify: MachineHandle<TaskInput>,
    verify_busy: Value<bool>,
) -> impl kivact::view::Element {
    specimen(
        "BTN-03",
        "Status",
        line((
            kivact::ui::button("Sync")
                .variant(kivact::ui::Slant)
                .test_id("game-busy")
                .icon(kivact::view::icon("refresh"))
                .busy(sync_busy)
                .on_press(move || sync.send(TaskEvent::Start)),
            kivact::ui::button("Run verification")
                .variant(kivact::ui::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .busy(verify_busy)
                .on_press(move || verify.send(TaskEvent::Start)),
            kivact::ui::button("Disabled")
                .variant(kivact::ui::Slant)
                .test_id("game-disabled")
                .disabled(true),
            kivact::ui::button("Disabled")
                .variant(kivact::ui::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .disabled(true),
        )),
    )
}

fn button_group(
    scene: Demo,
    period: kivact::state::Signal<Option<String>>,
) -> impl kivact::view::Element {
    specimen(
        "BTN-04",
        "Button group",
        list((
            segmented(
                "Period",
                choice_options(&[("24h", "24H"), ("7d", "7D"), ("30d", "30D"), ("90d", "90D")]),
                period,
            ),
            line((
                kivact::ui::button("Save and publish")
                    .variant(kivact::ui::Slant)
                    .icon(kivact::view::icon("device-floppy")),
                kivact::ui::menu(
                    "▾",
                    vec![
                        MenuEntry::item("schedule", "Schedule…"),
                        MenuEntry::item("limited", "Save as restricted"),
                        MenuEntry::item("draft", "Save as draft"),
                    ],
                )
                .on_activate(move |_: &'static str| scene.show(Overlay::Create))
                .offset(0.)
                .attach(
                    kivact::ui::button("▾")
                        .variant(kivact::ui::Slant)
                        .variant(kivact::ui::ButtonVariant::Secondary),
                ),
            )),
        )),
    )
}

fn button_toolbar(
    scene: Demo,
    grid_on: kivact::state::Signal<bool>,
    filter_on: kivact::state::Signal<bool>,
) -> impl kivact::view::Element {
    specimen(
        "BTN-05",
        "Toolbar",
        line((
            kivact::ui::toolbar("Console tools").start((
                kivact::ui::tooltip("Undo  Ctrl+Z", g::tool_button("Undo"))
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.),
                g::tool_button("Copy").on_press(|| {
                    spawn(platform::copy_text("OPS-2041"));
                }),
                separator(),
                g::tool_button("Grid view")
                    .with_node(|node| node.selected(grid_on))
                    .on_press(move || grid_on.update(|v| *v = !*v)),
                g::tool_button("Filter")
                    .test_id("game-toolbar-filter")
                    .with_node(|node| node.selected(filter_on))
                    .on_press(move || filter_on.update(|v| *v = !*v)),
                separator(),
                g::tool_button("Edit"),
                g::tool_button("Delete").on_press(move || scene.show(Overlay::Delete)),
            )),
            kivact::ui::button("Show details ↗")
                .variant(kivact::ui::Slant)
                .variant(kivact::ui::ButtonVariant::Ghost)
                .on_press(move || scene.show(Overlay::Drawer)),
        )),
    )
}

fn chevron_action(note: impl IntoView) -> impl kivact::view::Element {
    crate::support::specimens::specimen_frame(
        "BTN-06".to_owned(),
        "Chevron CTA".to_owned(),
        node()
            .child(g::chevron("▷ DEPLOY", "READY").on_press(move || {
                use_toaster().push(
                    Toast::new("Deploy complete")
                        .body("All workers started.")
                        .tone(Tone::Success)
                        .duration(Some(Duration::from_secs(4))),
                );
            }))
            .into_view(),
    )
    .caption("hero action".to_owned())
    .note(note)
    .test_id("BTN-06")
}
