//! Role: Public time-editor lane extension and selective trim contracts.
use super::*;
use kivact::ui::EditPhase;
#[test]
fn right_trim_proposes_initial_time_and_application_restores_timing_on_cancel() {
    let (screen, (clips, phases)) = kivact::test::render_with_handle(crate::options(), || {
        let clips = signal(vec![ui::Clip::new(1, 10, 40.0, 60.0, "Clip")]);
        let phases = signal(Vec::new());
        (
            (clips, phases),
            app_timeline(
                clips,
                ui::TimeState {
                    selection: (signal(vec![1])).into(),
                    zoom: (signal(2.0)).into(),
                    ..ui::TimeState::new()
                },
            )
            .lanes(vec![ui::Lane::new(10, "Lane")])
            .on_edit(time_edits(clips, apply_clips, move |phase| {
                phases.update(|p| p.push(phase))
            }))
            .name("Time")
            .style(style().size(700, 280)),
        )
    });
    let p = screen
        .get_by_role(Role::Button, "Clip end trim")
        .rect()
        .unwrap()
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 20.0, p.y);
    clips.update(|clips| {
        clips[0].start = 45.0;
        clips[0].label = "External".into();
        clips[0].color = kivact::style::rgb(0x123456);
    });
    screen.run_until_idle();
    screen.mouse_move(p.x + 30.0, p.y);
    assert_eq!(clips.get_untracked()[0].start, 40.0);
    assert_eq!(clips.get_untracked()[0].duration, 75.0);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    let clip = &clips.get_untracked()[0];
    assert_eq!(clip.start, 40.0);
    assert_eq!(clip.duration, 60.0);
    assert_eq!(clip.label, "External");
    assert_eq!(clip.color, kivact::style::rgb(0x123456));
    assert_eq!(
        phases.get_untracked(),
        [EditPhase::Begin, EditPhase::Cancel]
    );
    clean(&screen);
}

use super::editor_extensions::{TIME_EDITORS, fixture};
#[test]
fn time_cells_keep_fixed_widths_react_to_lanes_and_isolate_edits() {
    for editor in TIME_EDITORS {
        super::editor_extensions::cells(editor);
    }
}
#[test]
fn time_activation_is_once_for_enter_or_unmoved_double_click() {
    for editor in TIME_EDITORS {
        super::editor_extensions::activation(editor);
    }
}
#[test]
fn time_double_click_activates_nothing_when_the_lane_is_removed_before_release() {
    for editor in TIME_EDITORS {
        super::editor_extensions::activation_needs_a_live_target(editor);
    }
}
#[test]
fn time_context_menu_uses_request_snapshot_and_restores_region_focus() {
    for editor in TIME_EDITORS {
        super::editor_extensions::context(editor);
    }
}
#[test]
fn time_constraints_keep_caller_child_disabled_and_hide_subtree() {
    for editor in TIME_EDITORS {
        super::editor_extensions::constraints(editor);
    }
}
#[test]
fn failed_time_context_commands_keep_menu_open_and_report_once() {
    for editor in TIME_EDITORS {
        super::editor_extensions::failure(editor);
    }
}
#[test]
fn assistive_time_menu_cancels_drag_before_taking_the_context_snapshot() {
    for editor in TIME_EDITORS {
        let (screen, m) = fixture(editor);
        let p = screen
            .get_by_role(Role::Button, "Item")
            .rect()
            .expect("item geometry")
            .center();
        screen.mouse_move(p.x, p.y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(p.x + 12.0, p.y);
        screen
            .get_by_role(Role::Canvas, "Time")
            .a11y(A11yAction::ShowMenu);
        screen.get_by_role(Role::MenuItem, "Inspect Some(1) Some(10) 40");
        assert_eq!(m.value(), 40.0);
        assert_eq!(
            m.phases.get_untracked(),
            [EditPhase::Begin, EditPhase::Cancel]
        );
        screen.press_key(Key::Escape);
        screen.mouse_up(MouseButton::Primary);
        clean(&screen);
    }
}

#[test]
fn wheel_zoom_at_the_bound_opens_no_edit() {
    for editor in TIME_EDITORS {
        let (screen, m) = fixture(editor);
        let canvas = screen.get_by_role(Role::Canvas, "Time");
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        screen.key_down(Key::Unidentified, KeyCode::ControlLeft, ctrl);
        canvas.scroll(0.0, -40.0);
        assert_eq!(
            m.phases.get_untracked(),
            [EditPhase::Begin, EditPhase::Commit],
            "inside the bounds the wheel is one edit"
        );
        m.phases.set(Vec::new());
        m.zoom.set(1e9);
        screen.run_until_idle();
        canvas.scroll(0.0, -40.0);
        assert!(m.phases.get_untracked().is_empty());
        assert_eq!(m.zoom.get_untracked(), 1e9, "nothing is written");
        screen.key_up(
            Key::Unidentified,
            KeyCode::ControlLeft,
            Modifiers::default(),
        );
        clean(&screen);
    }
}

#[test]
fn ignored_time_requests_keep_items_fixed_and_report_absolute_proposals() {
    let (screen, (items, selected, events)) =
        kivact::test::render_with_handle(crate::options(), || {
            let items = signal(vec![ui::Clip::new(1, 10, 40.0, 60.0, "Clip")]);
            let selected = signal(vec![1]);
            let events = signal(Vec::new());
            (
                (items, selected, events),
                ui::timeline(
                    items,
                    ui::TimeState {
                        selection: selected.into(),
                        zoom: (signal(2.0)).into(),
                        ..ui::TimeState::new()
                    },
                )
                .step(0.5)
                .on_edit(move |event| events.update(|events| events.push(event)))
                .name("Read only")
                .style(style().size(700, 280)),
            )
        });
    let before = items.get_untracked();
    let item = screen.get_by_role(Role::Button, "Clip");
    let rect = item.rect().unwrap();
    let p = rect.center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(p.x + 20.0, p.y);
    screen.mouse_move(p.x + 40.0, p.y);
    assert_eq!(items.get_untracked(), before);
    assert_eq!(item.rect().unwrap(), rect);
    let recorded = events.get_untracked();
    assert_eq!(
        recorded
            .iter()
            .filter(|e| matches!(e, ui::EditorEvent::Begin))
            .count(),
        1
    );
    assert!(
        recorded.contains(&ui::EditorEvent::Request(ui::TimeRequest::Retime(vec![
            ui::ItemTime {
                id: 1,
                time: 50.0,
                duration: 60.0
            }
        ])))
    );
    assert_eq!(
        recorded.last(),
        Some(&ui::EditorEvent::Request(ui::TimeRequest::Retime(vec![
            ui::ItemTime {
                id: 1,
                time: 60.0,
                duration: 60.0
            }
        ])))
    );
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        events.get_untracked().last(),
        Some(&ui::EditorEvent::Cancel)
    );
    events.set(Vec::new());
    for (name, time, duration) in [
        ("Clip end trim", 40.0, 70.0),
        ("Clip start trim", 50.0, 50.0),
    ] {
        let trim = screen.get_by_role(Role::Button, name);
        let p = trim.rect().unwrap().center();
        trim.drag(p, Point::new(p.x + 20.0, p.y));
        assert!(events.get_untracked().contains(&ui::EditorEvent::Request(
            ui::TimeRequest::Retime(vec![ui::ItemTime {
                id: 1,
                time,
                duration
            }])
        )));
        assert_eq!(items.get_untracked(), before);
    }
    events.set(Vec::new());
    item.a11y_with_value(A11yAction::SetValue, "12.5");
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::TimeRequest::Retime(vec![ui::ItemTime {
                id: 1,
                time: 12.5,
                duration: 60.0
            }])),
            ui::EditorEvent::Commit
        ]
    );
    events.set(Vec::new());
    item.a11y_with_value(A11yAction::SetValue, "NaN");
    assert!(events.get_untracked().is_empty());
    item.a11y(A11yAction::Increment);
    item.a11y(A11yAction::Decrement);
    item.press_key(Key::ArrowRight);
    item.press_key(Key::Delete);
    assert_eq!(items.get_untracked(), before);
    assert!(selected.get_untracked().is_empty());
    assert!(
        events
            .get_untracked()
            .contains(&ui::EditorEvent::Request(ui::TimeRequest::Delete(vec![1])))
    );
    clean(&screen);
}
