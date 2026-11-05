//! Role: Tests for the advanced Creative controls through the public API.

use crate::arrangement_model::arrangement as app_arrangement;
use crate::table_rows::{line, lines};
use kivact::geometry::Point;
use kivact::prelude::*;

use kivact::view::{A11yAction, Handled, KeyCode, Modifiers, MouseButton};
use kivact_ui_creative as ui;
use std::time::Duration;

#[test]
fn arrangement_loop_automation_track_flags_and_clip_tools_update_public_models() {
    let (screen, (clips, tracks, range, tool)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let clips = signal(vec![
                ui::Clip::new(1, 0, 0.0, 16.0, "Clip").color(rgb(0x34d399)),
            ]);
            let tracks = signal(vec![
                ui::TimelineTrack::new(0, "Vox", ui::TrackKind::Audio, rgb(0x34d399)).automation(
                    vec![
                        ui::AutomationPoint::new(1, 2.0, 0.25),
                        ui::AutomationPoint::new(2, 16.0, 0.75),
                    ],
                ),
            ]);
            let range = signal((0.0, 16.0));
            let tool = signal(ui::TimelineTool::Select);
            (
                (clips, tracks, range, tool),
                app_arrangement(
                    clips,
                    tracks,
                    ui::ArrangementState {
                        loop_range: range.into_binding(),
                        tool: tool.into_binding(),
                        ..ui::ArrangementState::new()
                    },
                )
                .test_id("arrangement"),
            )
        });
    let end = screen.get_by_role(Role::Slider, "Loop end");
    end.a11y_set_value("12");
    screen
        .get_by_role(Role::Slider, "Loop start")
        .a11y_set_value("4");
    assert_eq!(range.get_untracked(), (4.0, 12.0));
    end.a11y_set_value("NaN");
    assert_eq!(range.get_untracked(), (4.0, 12.0));
    let r = end.rect().expect("loop end");
    end.drag(r.center(), Point::new(r.center().x + 26.0, r.center().y));
    assert_eq!(range.get_untracked(), (4.0, 13.0));
    let auto = screen.get_by_role(Role::Slider, "Vox automation point 1");
    auto.a11y_set_value("0.5");
    assert_eq!(tracks.get_untracked()[0].automation[0].value, 0.5);
    let r = auto.rect().expect("automation");
    auto.drag(
        r.center(),
        Point::new(r.center().x + 26.0, r.center().y - 8.5),
    );
    assert_eq!(tracks.get_untracked()[0].automation[0].beat, 3.0);
    assert!((tracks.get_untracked()[0].automation[0].value - 0.75).abs() < 0.01);
    let mute = screen.get_by_role(Role::Button, "Vox Mute");
    mute.click();
    assert!(tracks.get_untracked()[0].muted);
    assert_eq!(mute.checked(), Some(CheckedState::Checked));
    screen
        .get_by_test_id("arrangement")
        .press_key(Key::Char('3'));
    assert_eq!(tool.get_untracked(), ui::TimelineTool::Split);
    screen.get_by_role(Role::Button, "Clip").click();
    assert_eq!(clips.get_untracked().len(), 2);
    tool.set(ui::TimelineTool::Mute);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Clip (2)").click();
    assert!(clips.get_untracked()[1].muted);
    tool.set(ui::TimelineTool::Erase);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Clip (2)").click();
    assert_eq!(clips.get_untracked().len(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn floating_window_drags_collapses_and_restores_opener_focus() {
    let (screen, ()) = kivact::test::render_with_handle(crate::support::options(), || {
        (
            (),
            column((
                kivact::ui::button("Another opener").variant(kivact::ui::ButtonVariant::Secondary),
                ui::window_button("Render Settings", ui::WindowOptions::default(), |_| {
                    column((
                        text("Retained settings"),
                        kivact::ui::text_input("Window text", signal(String::new()))
                            .with_node(move |node| node.value_text("Settings")),
                    ))
                    .into_view()
                }),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Render Settings");
    opener.focus();
    opener.click();
    let title = screen.get_by_role(Role::Button, "Render Settings title bar");
    let retained = screen.get_by_text("Retained settings");
    let original = title.rect().expect("title");
    title.drag(Point::new(20.0, 15.0), Point::new(50.0, 35.0));
    let moved = title.rect().expect("moved title");
    assert!((moved.x - original.x - 30.0).abs() < 0.5);
    assert!((moved.y - original.y - 20.0).abs() < 0.5);
    title.drag(Point::new(20.0, 15.0), Point::new(-5000.0, -5000.0));
    let clamped = screen
        .get_by_role(Role::Group, "Render Settings window")
        .rect()
        .expect("clamped window");
    assert!((clamped.y - 0.0).abs() < 0.5);
    assert!((clamped.x + clamped.width - 80.0).abs() < 0.5);
    // Recover through the visible title-bar segment, rather than bypassing hit testing.
    screen.mouse_move(5.0, 8.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(505.0, 108.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(title.rect().expect("recovered title").x > -100.0);
    title.a11y(A11yAction::Toggle);
    assert_eq!(title.is_expanded(), Some(false));
    assert!(!retained.is_visible());
    title.a11y(A11yAction::Toggle);
    assert!(retained.is_visible());
    screen.get_by_role(Role::TextBox, "Window text").focus();
    screen.press_key(Key::Escape);
    assert!(opener.is_focused());
    opener.click();
    screen.get_by_role(Role::TextBox, "Window text").focus();
    screen.get_by_role(Role::Button, "Close window").click();
    assert!(opener.is_focused());
    opener.click();
    screen.get_by_role(Role::TextBox, "Window text").focus();
    let other = screen.get_by_role(Role::Button, "Another opener");
    other.click();
    assert!(other.is_focused());
    assert!(
        screen
            .query_by_role(Role::Dialog, "Render Settings")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_rows_are_one_tab_stop_and_follow_the_list_keys() {
    let (screen, (selection, rows)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let selection = signal(vec![1_u64]);
            let rows = signal(vec![
                line(1, ["Apple"]),
                line(2, ["Banana"]),
                line(3, ["Cherry"]),
                line(4, ["Date"]),
            ]);
            (
                (selection, rows),
                column((
                    kivact::ui::button("Before").variant(kivact::ui::ButtonVariant::Secondary),
                    lines(&["Name"], rows, selection),
                )),
            )
        });
    assert!(
        screen
            .get_by_role(Role::Grid, "Data table")
            .is_multiselectable()
    );
    let row = |name: &str| screen.get_by_role(Role::Row, name);
    // The header, then one tab stop for the rows: the selected row.
    screen.get_by_role(Role::Button, "Before").click();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::ColumnHeader, "Name").is_focused());
    screen.press_tab(false);
    assert!(row("Apple").is_focused());
    screen.press_tab(false);
    assert!(
        !screen.get_by_role(Role::Row, "Banana").is_focused(),
        "the other rows are not tab stops"
    );
    screen.press_tab(true);
    assert!(row("Apple").is_focused());
    screen.press_key(Key::ArrowDown);
    assert_eq!(selection.get_untracked(), vec![2]);
    assert!(row("Banana").is_focused());
    screen.press_key(Key::End);
    assert_eq!(selection.get_untracked(), vec![4]);
    assert!(row("Date").is_focused());
    screen.press_key(Key::Home);
    assert_eq!(selection.get_untracked(), vec![1]);
    assert!(row("Apple").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(row("Apple").is_focused(), "stops at the first row");
    // Ctrl moves the focus alone and Space toggles; Shift extends.
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    screen.press_key_with(Key::ArrowDown, ctrl);
    screen.press_key_with(Key::ArrowDown, ctrl);
    assert!(row("Cherry").is_focused());
    assert_eq!(selection.get_untracked(), vec![1]);
    screen.press_key_with(Key::Space, ctrl);
    assert_eq!(selection.get_untracked(), vec![1, 3]);
    screen.press_key_with(Key::ArrowDown, shift);
    assert_eq!(selection.get_untracked(), vec![3, 4]);
    // Ctrl+A and Cmd+A select every row.
    selection.set(Vec::new());
    screen.press_key_with(Key::Char('a'), ctrl);
    assert_eq!(selection.get_untracked(), vec![1, 2, 3, 4]);
    selection.set(Vec::new());
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            meta: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(selection.get_untracked(), vec![1, 2, 3, 4]);
    // Typing a row's name focuses it; the filter and removal move the stop.
    screen.press_key(Key::Char('b'));
    assert!(row("Banana").is_focused());
    assert_eq!(selection.get_untracked(), vec![2]);
    rows.update(|rows| rows.retain(|row| row.id != 2));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Before").click();
    screen.press_tab(false);
    screen.press_tab(false);
    assert!(row("Cherry").is_focused(), "the row that took its place");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn color_popup_escape_rolls_back_while_outside_and_accessibility_dismissal_commit() {
    let (screen, color) = kivact::test::render_with_handle(crate::support::options(), || {
        let color = signal(rgb(0x8b7cff));
        (
            color,
            column((
                ui::color_popup("Edit color", color),
                kivact::ui::button("Other").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let original = color.get_untracked();
    let opener = screen.get_by_role(Role::Button, "Edit color");
    opener.click();
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_set_value("0.25");
    assert_ne!(color.get_untracked(), original);
    screen.press_key(Key::Escape);
    assert_eq!(color.get_untracked(), original);
    assert!(opener.is_focused());
    opener.click();
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_set_value("0.25");
    let committed = color.get_untracked();
    screen.get_by_role(Role::Button, "Other").click();
    assert_eq!(color.get_untracked(), committed);
    opener.click();
    screen
        .get_by_role(Role::SpinButton, "Color G")
        .a11y_set_value("0.5");
    let committed = color.get_untracked();
    screen
        .get_by_role(Role::Dialog, "Edit color")
        .a11y(A11yAction::Dismiss);
    assert_eq!(color.get_untracked(), committed);
    let next = rgb(0x446688);
    color.set(next);
    screen.run_until_idle();
    opener.a11y(A11yAction::Expand);
    screen
        .get_by_role(Role::SpinButton, "Color R")
        .a11y_set_value("0.75");
    screen.press_key(Key::Escape);
    assert_eq!(
        color.get_untracked(),
        next,
        "Expand snapshots the current color transaction"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pie_hold_direction_release_and_keyboard_commands_share_public_state() {
    let (screen, (state, chosen)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let state = signal(ui::PieState::default());
            let chosen = signal(None::<usize>);
            let entries = vec![
                ui::PieEntry::new("Wireframe", 180.0, "box"),
                ui::PieEntry::new("Solid", 0.0, "eye"),
            ];
            (
                (state, chosen),
                column((
                    ui::pie_menu(entries, state).on_activate(move |index| chosen.set(Some(index))),
                ))
                .role(Role::Group)
                .name("Pie host")
                .focusable(true)
                .style(style().size(640, 480))
                .on_key(move |event| {
                    if event.key() == Key::Char('z') {
                        state.set(ui::PieState::open_at(Point::new(300.0, 200.0), true));
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .on_key_up(move |event| {
                    if event.key() == Key::Char('z')
                        && let ui::PieState::Open {
                            mode: ui::PieMode::Held,
                            hot,
                            ..
                        } = state.get_untracked()
                    {
                        chosen.set(hot);
                        state.set(ui::PieState::default());
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
            )
        });
    screen.get_by_role(Role::Group, "Pie host").focus();
    screen.key_down(Key::Char('z'), KeyCode::KeyZ, Modifiers::default());
    assert!(matches!(state.get_untracked(), ui::PieState::Open { .. }));
    screen.mouse_move(196.0, 200.0);
    assert!(matches!(
        state.get_untracked(),
        ui::PieState::Open { hot: Some(0), .. }
    ));
    screen.key_up(Key::Char('z'), KeyCode::KeyZ, Modifiers::default());
    assert_eq!(chosen.get_untracked(), Some(0));
    assert_eq!(state.get_untracked(), ui::PieState::Closed);
    state.set(ui::PieState::open_at(Point::new(300.0, 200.0), false));
    screen.advance(Duration::ZERO);
    screen
        .get_by_role(Role::Menu, "Shading pie")
        .press_key(Key::Char('2'));
    assert_eq!(chosen.get_untracked(), Some(1));
    assert_eq!(state.get_untracked(), ui::PieState::Closed);
    state.set(ui::PieState::open_at(Point::new(300.0, 200.0), true));
    screen.run_until_idle();
    screen
        .get_by_role(Role::Menu, "Shading pie")
        .press_key(Key::Escape);
    assert_eq!(state.get_untracked(), ui::PieState::Closed);
    assert!(
        !matches!(
            state.get_untracked(),
            ui::PieState::Open {
                mode: ui::PieMode::Held,
                ..
            }
        ),
        "shared dismissal cancels the held transaction"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn property_group_exclusive_expansion_drag_and_accessibility_reorder_keep_content() {
    let (screen, (panels, value)) =
        kivact::test::render_with_handle(crate::support::options(), || {
            let panels = signal(vec![
                ui::InspectorPanel::new(1, "Transform"),
                ui::InspectorPanel::new(2, "Relations"),
            ]);
            let value = signal(4.0);
            (
                (panels, value),
                ui::panel_group(panels, move |_| {
                    ui::number("Retained", value).range(
                        kivact::interaction::ScalarRange::new(0.0, 10.0)
                            .expect("invariant: finite ordered control range"),
                    )
                }),
            )
        });
    let transform = screen.get_by_role(Role::Button, "Transform");
    screen
        .get_by_role(Role::Group, "Transform panel")
        .get_by_role(Role::SpinButton, "Retained")
        .a11y_set_value("7");
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    screen.get_by_role(Role::Button, "Relations").click();
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    assert!(!panels.get_untracked()[0].expanded);
    assert!(panels.get_untracked()[1].expanded);
    assert_eq!(value.get_untracked(), 7.0);
    let grip = screen.get_by_role(Role::Button, "Reorder Relations");
    let a = grip.rect().expect("grip");
    let destination = transform.rect().expect("transform");
    grip.drag(
        Point::new(a.width / 2.0, a.height / 2.0),
        Point::new(destination.center().x - a.x, destination.y - a.y),
    );
    assert_eq!(panels.get_untracked()[0].id, 2);
    assert!(
        panels.get_untracked()[0].expanded,
        "dragging the grip does not toggle its panel"
    );
    screen
        .get_by_role(Role::Button, "Reorder Relations")
        .click();
    assert!(
        panels.get_untracked()[0].expanded,
        "clicking the grip does not toggle its panel"
    );
    screen
        .get_by_role(Role::Button, "Reorder Relations")
        .a11y(A11yAction::Increment);
    assert_eq!(panels.get_untracked()[0].id, 1);
    transform.a11y(A11yAction::Toggle);
    assert_eq!(value.get_untracked(), 7.0);
    assert!(
        screen
            .get_by_role(Role::Group, "Transform panel")
            .get_by_role(Role::SpinButton, "Retained")
            .is_visible()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn panel_group_ctrl_opens_only_the_chosen_panel_and_a_plain_expand_closes_nothing() {
    let (screen, panels) = kivact::test::render_with_handle(crate::support::options(), || {
        let panels = signal(vec![
            ui::InspectorPanel::new(1, "Transform"),
            ui::InspectorPanel::new(2, "Relations"),
        ]);
        (panels, ui::panel_group(panels, |_| text("Body")))
    });
    let transform = screen.get_by_role(Role::Button, "Transform");
    let relations = screen.get_by_role(Role::Button, "Relations");
    // Ctrl and a press that never becomes a click change nothing...
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let header = relations.rect().expect("relations header");
    screen.mouse_move(header.center().x, header.center().y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(header.center().x, header.center().y + 400.0);
    screen.mouse_up(MouseButton::Primary);
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    // ...and leave nothing armed for a later assistive expand.
    transform.a11y(A11yAction::Collapse);
    transform.a11y(A11yAction::Expand);
    assert!(panels.get_untracked().iter().all(|panel| panel.expanded));
    relations.a11y(A11yAction::Collapse);
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    relations.a11y(A11yAction::Expand);
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    let open: Vec<bool> = panels.get_untracked().iter().map(|p| p.expanded).collect();
    assert_eq!(open, vec![false, true], "Ctrl keeps only the chosen panel");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
