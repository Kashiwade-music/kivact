//! Role: Arrangement edits transact exactly the model fields they own.
use super::*;
use kivact::ui::EditPhase;

#[test]
fn long_arrangement_reaches_last_track_and_clamps_both_pan_inputs() {
    let (s, pan) = kivact::test::render_with_handle(crate::options(), || {
        let pan = signal(Point::new(0.0, 0.0));
        (
            pan,
            app_arrangement(
                signal(Vec::new()),
                signal(
                    (0..15)
                        .map(|id| {
                            ui::TimelineTrack::new(
                                id,
                                format!("Track {id}"),
                                ui::TrackKind::Audio,
                                rgb(0x34d399),
                            )
                        })
                        .collect(),
                ),
                ui::ArrangementState {
                    pan: pan.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .style(style().size(800, 280)),
        )
    });
    s.mouse_move(600.0, 180.0);
    s.scroll(0.0, 2000.0);
    let wheel = pan.get_untracked().y;
    s.scroll(0.0, 100.0);
    let bottom = pan.get_untracked().y;
    pan.set(Point::new(0.0, 0.0));
    s.run_until_idle();
    s.mouse_down(MouseButton::Middle);
    s.mouse_move(600.0, -1000.0);
    s.mouse_up(MouseButton::Middle);
    assert_eq!(
        (wheel, bottom, pan.get_untracked().y),
        (-660.0, -660.0, -660.0)
    );
    assert!(s.get_by_role(Role::Button, "Track 14 Mute").is_visible());
    clean(&s);
}

#[test]
fn a_stored_vertical_pan_past_the_content_is_clamped_where_it_is_read() {
    let (s, (tracks, pan)) = kivact::test::render_with_handle(crate::options(), || {
        let pan = signal(Point::new(0.0, 0.0));
        let tracks = signal(
            (0..15)
                .map(|id| {
                    ui::TimelineTrack::new(
                        id,
                        format!("Track {id}"),
                        ui::TrackKind::Audio,
                        rgb(0x34d399),
                    )
                })
                .collect::<Vec<_>>(),
        );
        (
            (tracks, pan),
            app_arrangement(
                signal(Vec::new()),
                tracks,
                ui::ArrangementState {
                    pan: pan.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .name("Long")
            .style(style().size(800, 280)),
        )
    });
    s.mouse_move(600.0, 180.0);
    s.scroll(0.0, 2000.0);
    assert_eq!(pan.get_untracked().y, -660.0);
    tracks.update(|tracks| tracks.truncate(5));
    s.run_until_idle();
    let canvas = s.get_by_role(Role::Canvas, "Long").rect().expect("canvas");
    let first = s
        .get_by_role(Role::Button, "Track 0 Mute")
        .rect()
        .expect("first track");
    assert!(
        first.y >= canvas.y && first.y < canvas.y + 120.0,
        "{first:?}"
    );
    assert_eq!(pan.get_untracked().y, -660.0, "no effect writes the pan");
    // The next wheel starts from the clamped offset, not from the stored one.
    s.scroll(0.0, -5.0);
    assert_eq!(pan.get_untracked().y, -55.0);
    clean(&s);
}

#[test]
fn loop_input_matches_the_drawn_endpoint_boundary() {
    let (s, range) = kivact::test::render_with_handle(crate::options(), || {
        let range = signal((4.0, 12.0));
        (
            range,
            app_arrangement(
                signal(Vec::new()),
                signal(Vec::new()),
                ui::ArrangementState {
                    loop_range: range.into(),
                    zoom: (signal(26.0)).into(),
                    ..ui::ArrangementState::new()
                },
            )
            .snap(false)
            .style(style().size(800, 280)),
        )
    });
    // 5.5px before the start is outside the drawn +/-5px endpoint.
    let x = 220.0 + 4.0 * 26.0 - 5.5;
    s.mouse_move(x, 5.0);
    s.mouse_down(MouseButton::Primary);
    s.mouse_move(x - 26.0, 5.0);
    s.mouse_up(MouseButton::Primary);
    assert!(
        range.get_untracked().1 < 4.0,
        "outside endpoint creates a new loop"
    );
    range.set((0.0, 12.0));
    s.run_until_idle();
    s.mouse_move(218.0, 5.0);
    s.mouse_down(MouseButton::Primary);
    s.mouse_move(244.0, 5.0);
    s.mouse_up(MouseButton::Primary);
    assert_eq!(
        range.get_untracked(),
        (0.0, 12.0),
        "sidebar does not start a ruler edit"
    );
    clean(&s);
}

#[test]
fn invalid_clips_are_absent_but_zero_duration_keys_remain() {
    let (s, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                app_timeline(
                    signal(vec![
                        ui::Clip::new(1, 0, 1.0, 0.0, "Zero clip"),
                        ui::Clip::new(2, 0, 2.0, -1.0, "Negative clip"),
                        ui::Clip::new(3, 0, f64::NAN, 1.0, "Invalid clip"),
                    ]),
                    ui::TimeState::new(),
                )
                .lanes(vec![ui::Lane::new(0, "Lane")]),
                app_dope_sheet(
                    signal(vec![ui::Keyframe::new(4, 0, 2.0, "Zero key")]),
                    ui::TimeState::new(),
                )
                .lanes(vec![ui::Lane::new(0, "Keys")]),
            )),
        )
    });
    for label in ["Zero clip", "Negative clip", "Invalid clip"] {
        assert!(s.query_by_role(Role::Button, label).is_none(), "{label}");
    }
    s.get_by_role(Role::Button, "Zero key");
    clean(&s);
    let (s, (data, selected, phases)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(vec![
            ui::Clip::new(1, 0, 1.0, 0.0, "Zero clip"),
            ui::Clip::new(2, 0, 2.0, 1.0, "Valid clip"),
        ]);
        let selected = signal(vec![1]);
        let phases = signal(Vec::new());
        (
            (data, selected, phases),
            app_timeline(
                data,
                ui::TimeState {
                    selection: selected.into(),
                    ..ui::TimeState::new()
                },
            )
            .on_edit(time_edits(data, apply_clips, move |phase| {
                phases.update(|phases| phases.push(phase))
            }))
            .lanes(vec![ui::Lane::new(0, "Lane")])
            .name("Filtered time"),
        )
    });
    let before = data.get_untracked();
    let canvas = s.get_by_role(Role::Canvas, "Filtered time");
    canvas.press_key(Key::ArrowRight);
    canvas.press_key(Key::Delete);
    assert_eq!(data.get_untracked(), before);
    assert_eq!(
        selected.get_untracked(),
        [1],
        "external selection is not pruned"
    );
    assert!(phases.get_untracked().is_empty());
    clean(&s);
    let (s, (data, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(vec![
            ui::Clip::new(1, 10, 1.0, 0.0, "Zero clip"),
            ui::Clip::new(2, 10, 2.0, 1.0, "Valid clip"),
        ]);
        let selected = signal(Vec::new());
        (
            (data, selected),
            app_arrangement(
                data,
                signal(vec![ui::TimelineTrack::new(
                    10,
                    "Track",
                    ui::TrackKind::Audio,
                    rgb(0x34d399),
                )]),
                ui::ArrangementState {
                    selection: selected.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .name("Valid arrangement"),
        )
    });
    let canvas = s.get_by_role(Role::Canvas, "Valid arrangement");
    canvas.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(selected.get_untracked(), [2]);
    assert_eq!(data.get_untracked().len(), 2, "caller data is retained");
    assert!(s.query_by_role(Role::Button, "Zero clip").is_none());
    clean(&s);
}

#[test]
fn arrangement_switches_and_invalid_scene_keep_their_semantic_contract() {
    let (s, (tracks, clips)) = kivact::test::render_with_handle(crate::options(), || {
        let tracks = signal(vec![ui::TimelineTrack::new(
            10,
            "Track",
            ui::TrackKind::Audio,
            rgb(0x34d399),
        )]);
        let clips = signal(vec![ui::Clip::new(7, 10, 1.0, 1.0, "Clip")]);
        (
            (tracks, clips),
            app_arrangement(clips, tracks, ui::ArrangementState::new())
                .style(style().size(800, 280)),
        )
    });
    for label in [
        "Track Mute",
        "Track Solo",
        "Track Record",
        "Track Read Automation",
    ] {
        s.get_by_role(Role::Button, label)
            .a11y(A11yAction::Activate);
    }
    let track = &tracks.get_untracked()[0];
    assert!(track.muted && track.solo && track.armed && track.automation_read);
    assert!(
        s.get_by_role(Role::Group, "Loop ruler")
            .a11y_actions()
            .is_empty()
    );
    clean(&s);
    clips.update(|clips| clips.push(clips[0].clone()));
    s.run_until_idle();
    assert!(s.query_by_role(Role::Button, "Clip").is_none());
    assert!(s.query_by_role(Role::Button, "Track Mute").is_none());
    assert!(!s.errors().is_empty(), "duplicate scene reports its error");
}
#[derive(Clone, Copy)]
struct Model {
    clips: Signal<Vec<ui::Clip>>,
    tracks: Signal<Vec<ui::TimelineTrack>>,
    selected: Signal<Vec<u64>>,
    frame: Signal<f64>,
    range: Signal<(f64, f64)>,
    cycling: Signal<bool>,
    zoom: Signal<f64>,
    pan: Signal<Point>,
    tool: Signal<ui::TimelineTool>,
    phases: Signal<Vec<EditPhase>>,
    observed: Signal<Vec<(EditPhase, f64)>>,
    disabled: Signal<bool>,
    visible: Signal<bool>,
    present: Signal<bool>,
}
fn fixture() -> (Screen, Model) {
    kivact::test::render_with_handle(crate::options(), || {
        let m = Model {
            clips: signal(vec![
                ui::Clip::new(1, 10, 2.0, 8.0, "Clip"),
                ui::Clip::new(2, 10, 14.0, 4.0, "Other"),
            ]),
            tracks: signal(vec![
                ui::TimelineTrack::new(10, "Vox", ui::TrackKind::Audio, rgb(0x34d399)).automation(
                    vec![
                        ui::AutomationPoint::new(100, 3.0, 0.25),
                        ui::AutomationPoint::new(101, 15.0, 0.75),
                    ],
                ),
            ]),
            selected: signal(vec![1]),
            frame: signal(0.0),
            range: signal((0.0, 16.0)),
            cycling: signal(true),
            zoom: signal(26.0),
            pan: signal(Point::new(0.0, 0.0)),
            tool: signal(ui::TimelineTool::Select),
            phases: signal(Vec::new()),
            observed: signal(Vec::new()),
            disabled: signal(false),
            visible: signal(true),
            present: signal(true),
        };
        (
            m,
            column((
                show(m.present, move || {
                    app_arrangement(
                        m.clips,
                        m.tracks,
                        ui::ArrangementState {
                            selection: m.selected.into(),
                            playhead: m.frame.into(),
                            loop_range: m.range.into(),
                            cycling: m.cycling.into(),
                            zoom: m.zoom.into(),
                            pan: m.pan.into(),
                            tool: m.tool.into(),
                        },
                    )
                    .on_edit(arrangement_model::edits(m.clips, m.tracks, move |phase| {
                        m.phases.update(|v| v.push(phase));
                        m.observed.update(|v| {
                            v.push((
                                phase,
                                m.clips.get_untracked().first().map_or(-1.0, |c| c.start),
                            ))
                        });
                    }))
                    .name("Arrange")
                    .disabled(m.disabled)
                    .hidden(move || !m.visible.get())
                    .style(style().size(800, 280))
                }),
                kivact::ui::button("Outside"),
            )),
        )
    })
}
fn start(s: &Screen, role: Role, name: &str) -> Point {
    let p = s
        .get_by_role(role, name)
        .rect()
        .expect("edit region")
        .center();
    s.mouse_move(p.x, p.y);
    s.mouse_down(MouseButton::Primary);
    p
}
fn phases(m: Model, expected: &[EditPhase]) {
    assert_eq!(m.phases.get_untracked(), expected);
}
#[test]
fn clip_drag_uses_one_transaction_and_snap_from_the_original_position() {
    let (s, m) = fixture();
    s.get_by_role(Role::Button, "Clip").click();
    phases(m, &[]);
    let p = start(&s, Role::Button, "Clip");
    s.mouse_move(p.x + 15.0, p.y);
    s.mouse_move(p.x + 39.0, p.y);
    assert_eq!(
        m.clips.get_untracked().first().map_or(-1.0, |c| c.start),
        4.0
    );
    s.mouse_up(MouseButton::Secondary);
    phases(m, &[EditPhase::Begin]);
    s.mouse_up(MouseButton::Primary);
    phases(m, &[EditPhase::Begin, EditPhase::Commit]);
    assert_eq!(
        m.observed.get_untracked(),
        [(EditPhase::Begin, 2.0), (EditPhase::Commit, 4.0)]
    );
    clean(&s);
}
#[test]
fn application_snapshot_restores_items_on_escape_and_lifecycle_cancel() {
    for mode in 0..7 {
        let (s, m) = fixture();
        let p = s
            .get_by_role(Role::Button, "Clip")
            .rect()
            .expect("clip")
            .center();
        let contact = if mode == 6 {
            let contact = s.contact_down(kivact::view::PointerKind::Touch, p.x, p.y);
            s.contact_move(contact, p.x + 52.0, p.y);
            Some(contact)
        } else {
            s.mouse_move(p.x, p.y);
            s.mouse_down(MouseButton::Primary);
            s.mouse_move(p.x + 52.0, p.y);
            None
        };
        m.clips.update(|v| {
            v[0].label = "External".into();
            v[0].muted = true;
            v[0].duration = 9.0;
            v.remove(1);
        });
        match mode {
            0 => s.press_key(Key::Escape),
            1 => m.disabled.set(true),
            2 => m.visible.set(false),
            3 => m.present.set(false),
            4 => s.get_by_role(Role::Button, "Outside").focus(),
            5 => s.set_window_focused(false),
            _ => s.contact_cancel(contact.expect("the touch is down")),
        };
        s.run_until_idle();
        let clips = m.clips.get_untracked();
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].start, 2.0, "mode {mode}");
        assert_eq!(clips[0].duration, 8.0);
        assert_eq!(clips[0].label, "Clip");
        assert!(!clips[0].muted);
        phases(m, &[EditPhase::Begin, EditPhase::Cancel]);
        assert_eq!(
            m.observed.get_untracked().last(),
            Some(&(EditPhase::Cancel, 2.0))
        );
        clean(&s);
    }
}
#[test]
fn trims_use_application_snapshot_rollback() {
    let (s, m) = fixture();
    let p = start(&s, Role::Button, "Clip end trim");
    s.mouse_move(p.x + 13.0, p.y);
    s.mouse_move(p.x + 39.0, p.y);
    assert_eq!(m.clips.get_untracked()[0].duration, 9.5);
    m.clips.update(|v| v[0].start = 3.0);
    s.press_key(Key::Escape);
    assert_eq!(
        (
            m.clips.get_untracked().first().map_or(-1.0, |c| c.start),
            m.clips.get_untracked()[0].duration
        ),
        (2.0, 8.0)
    );
    phases(m, &[EditPhase::Begin, EditPhase::Cancel]);
    s.mouse_up(MouseButton::Primary);
    m.phases.set(Vec::new());
    let p = start(&s, Role::Button, "Clip start trim");
    s.mouse_move(p.x + 26.0, p.y);
    s.mouse_up(MouseButton::Primary);
    assert_eq!(
        (
            m.clips.get_untracked().first().map_or(-1.0, |c| c.start),
            m.clips.get_untracked()[0].duration
        ),
        (3.0, 7.0)
    );
    phases(m, &[EditPhase::Begin, EditPhase::Commit]);
    clean(&s);
}
#[test]
fn loop_playhead_and_pan_drag_commit_or_restore_their_initial_values() {
    let (s, m) = fixture();
    let p = start(&s, Role::Slider, "Loop end");
    s.mouse_move(p.x + 26.0, p.y);
    s.mouse_move(p.x + 52.0, p.y);
    assert_eq!(m.range.get_untracked(), (0.0, 18.0));
    s.press_key(Key::Escape);
    assert_eq!(m.range.get_untracked(), (0.0, 16.0));
    s.mouse_up(MouseButton::Primary);
    m.phases.set(Vec::new());
    let p = start(&s, Role::Slider, "Playhead");
    s.mouse_move(p.x + 26.0, p.y);
    s.mouse_up(MouseButton::Primary);
    phases(m, &[EditPhase::Begin, EditPhase::Commit]);
    assert!(m.frame.get_untracked() > 0.0);
    m.phases.set(Vec::new());
    let p = s
        .get_by_role(Role::Canvas, "Arrange")
        .rect()
        .expect("canvas")
        .center();
    s.mouse_move(p.x, p.y);
    s.mouse_down(MouseButton::Middle);
    s.mouse_move(p.x + 23.0, p.y + 11.0);
    s.press_key(Key::Escape);
    assert_eq!(m.pan.get_untracked(), Point::new(0.0, 0.0));
    phases(m, &[EditPhase::Begin, EditPhase::Cancel]);
    clean(&s);
}
#[test]
fn application_automation_snapshot_restores_the_track() {
    let (s, m) = fixture();
    let p = start(&s, Role::Slider, "Vox automation point 100");
    s.mouse_move(p.x + 26.0, p.y - 8.5);
    m.tracks.update(|v| {
        v[0].name = "External".into();
        v[0].muted = true;
        v[0].automation[1].value = 0.9;
    });
    s.press_key(Key::Escape);
    let track = &m.tracks.get_untracked()[0];
    assert_eq!(track.name, "Vox");
    assert!(!track.muted);
    assert_eq!(
        (track.automation[0].beat, track.automation[0].value),
        (3.0, 0.25)
    );
    assert_eq!(track.automation[1].value, 0.75);
    phases(m, &[EditPhase::Begin, EditPhase::Cancel]);
    clean(&s);
}
#[test]
fn each_discrete_write_emits_a_pair_and_drag_rejects_competing_edits() {
    let (s, m) = fixture();
    let root = s.get_by_role(Role::Canvas, "Arrange");
    root.press_key(Key::ArrowRight);
    assert_eq!(
        m.clips.get_untracked().first().map_or(-1.0, |c| c.start),
        3.0
    );
    s.get_by_role(Role::Slider, "Playhead")
        .a11y_with_value(A11yAction::SetValue, "5");
    s.get_by_role(Role::Slider, "Loop end")
        .a11y_with_value(A11yAction::SetValue, "12");
    s.get_by_role(Role::Slider, "Vox automation point 100")
        .a11y_with_value(A11yAction::SetValue, "0.5");
    s.get_by_role(Role::Button, "Vox Mute")
        .a11y(A11yAction::Activate);
    // The tracks fit the viewport, so a plain wheel has nothing to pan; the
    // zoom wheel writes.
    s.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    root.scroll(0.0, 10.0);
    s.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    assert_eq!(
        m.phases.get_untracked(),
        [EditPhase::Begin, EditPhase::Commit].repeat(6)
    );
    m.phases.set(Vec::new());
    let p = start(&s, Role::Button, "Clip");
    s.mouse_move(p.x + 26.0, p.y);
    let frame = m.frame.get_untracked();
    let zoom = m.zoom.get_untracked();
    let muted = m.tracks.get_untracked()[0].muted;
    root.press_key(Key::ArrowRight);
    root.scroll(0.0, 10.0);
    s.get_by_role(Role::Slider, "Playhead")
        .a11y_with_value(A11yAction::SetValue, "99");
    s.get_by_role(Role::Button, "Vox Mute")
        .a11y(A11yAction::Activate);
    assert_eq!(m.frame.get_untracked(), frame);
    assert_eq!(m.zoom.get_untracked(), zoom);
    assert_eq!(m.tracks.get_untracked()[0].muted, muted);
    phases(m, &[EditPhase::Begin]);
    s.mouse_up(MouseButton::Primary);
    phases(m, &[EditPhase::Begin, EditPhase::Commit]);
    clean(&s);
}

#[test]
fn application_cancel_removes_new_automation_and_restores_deleted_items() {
    let (s, m) = fixture();
    let r = s
        .get_by_role(Role::Canvas, "Arrange")
        .rect()
        .expect("arrangement");
    s.mouse_move(r.x + 402.0, r.y + 117.0);
    s.mouse_down(MouseButton::Primary);
    assert_eq!(m.tracks.get_untracked()[0].automation.len(), 3);
    m.tracks.update(|v| v[0].automation[0].value = 0.8);
    s.press_key(Key::Escape);
    assert_eq!(m.tracks.get_untracked()[0].automation.len(), 2);
    assert_eq!(m.tracks.get_untracked()[0].automation[0].value, 0.25);
    phases(m, &[EditPhase::Begin, EditPhase::Cancel]);
    s.mouse_up(MouseButton::Primary);
    m.phases.set(Vec::new());
    m.selected.set(vec![1, 2]);
    s.run_until_idle();
    let p = start(&s, Role::Button, "Clip");
    s.mouse_move(p.x + 26.0, p.y);
    m.clips.update(|v| v.retain(|c| c.id != 1));
    s.run_until_idle();
    let clips = m.clips.get_untracked();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[0].id, 1);
    assert_eq!(clips[0].start, 2.0);
    phases(m, &[EditPhase::Begin, EditPhase::Cancel]);
    clean(&s);
}

#[test]
fn wheel_zoom_at_the_bound_or_with_an_invalid_delta_opens_no_edit() {
    let (s, m) = fixture();
    let root = s.get_by_role(Role::Canvas, "Arrange");
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    s.key_down(Key::Unidentified, KeyCode::ControlLeft, ctrl);
    root.scroll(0.0, -40.0);
    phases(m, &[EditPhase::Begin, EditPhase::Commit]);
    m.phases.set(Vec::new());
    m.zoom.set(1e9);
    s.run_until_idle();
    root.scroll(0.0, -40.0);
    root.scroll(0.0, f32::NAN);
    root.scroll(f32::INFINITY, -40.0);
    phases(m, &[]);
    assert_eq!(m.zoom.get_untracked(), 1e9, "nothing is written");
    s.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    clean(&s);
}

#[test]
fn arrow_keys_move_selected_clips_by_the_step_and_shift_by_ten_steps() {
    let (s, (clips, step)) = kivact::test::render_with_handle(crate::options(), || {
        let clips = signal(vec![ui::Clip::new(1, 10, 2.0, 1.0, "Clip")]);
        let step = signal(0.5);
        (
            (clips, step),
            app_arrangement(
                clips,
                signal(vec![ui::TimelineTrack::new(
                    10,
                    "Track",
                    ui::TrackKind::Audio,
                    rgb(0x34d399),
                )]),
                ui::ArrangementState {
                    selection: (signal(vec![1])).into(),
                    ..ui::ArrangementState::new()
                },
            )
            .step(step)
            .name("Arrange")
            .style(style().size(800, 280)),
        )
    });
    let root = s.get_by_role(Role::Canvas, "Arrange");
    let start = || clips.get_untracked()[0].start;
    root.press_key(Key::ArrowRight);
    assert_eq!(start(), 2.5);
    root.press_key_with(
        Key::ArrowRight,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(start(), 7.5);
    step.set(2.0);
    s.run_until_idle();
    root.press_key(Key::ArrowLeft);
    assert_eq!(start(), 5.5);
    step.set(f64::NAN);
    s.run_until_idle();
    root.press_key(Key::ArrowLeft);
    assert_eq!(start(), 4.5, "an invalid step falls back to one beat");
    clean(&s);
}

#[test]
fn a_selected_group_snaps_by_the_pressed_clip_and_keeps_its_spacing() {
    let (s, clips) = kivact::test::render_with_handle(crate::options(), || {
        let clips = signal(vec![
            ui::Clip::new(1, 10, 0.3, 0.5, "First"),
            ui::Clip::new(2, 10, 1.7, 0.5, "Second"),
        ]);
        (
            clips,
            app_arrangement(
                clips,
                signal(vec![ui::TimelineTrack::new(
                    10,
                    "Track",
                    ui::TrackKind::Audio,
                    rgb(0x34d399),
                )]),
                ui::ArrangementState {
                    selection: (signal(vec![1, 2])).into(),
                    zoom: (signal(40.0)).into(),
                    ..ui::ArrangementState::new()
                },
            )
            .snap(true)
            .step(1.0)
            .style(style().size(800, 280)),
        )
    });
    let first = s.get_by_role(Role::Button, "First");
    let p = first.rect().expect("first clip").center();
    first.drag(p, Point::new(p.x + 24.0, p.y));
    let starts = clips
        .get_untracked()
        .iter()
        .map(|c| c.start)
        .collect::<Vec<_>>();
    assert!(near(starts[0], 1.0, 1e-6), "{starts:?}");
    assert!(near(starts[1], 2.4, 1e-6), "{starts:?}");
    clean(&s);
}
