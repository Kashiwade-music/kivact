//! Role: Read-only arrangement proposals and application-owned decisions.
use super::*;

#[test]
fn rejected_arrangement_requests_remain_absolute_and_do_not_move_the_view() {
    let (s, (clips, events)) = kivact::test::render_with_handle(crate::options(), || {
        let clips = signal(vec![ui::Clip::new(1, 10, 2.0, 8.0, "Clip")]);
        let events = signal(Vec::new());
        (
            (clips, events),
            ui::arrangement(
                clips,
                vec![ui::TimelineTrack::new(
                    10,
                    "Track",
                    ui::TrackKind::Audio,
                    rgb(0x34d399),
                )],
                ui::ArrangementState {
                    selection: (signal(vec![1])).into(),
                    ..ui::ArrangementState::new()
                },
            )
            .length(24.0)
            .on_edit(move |event| events.update(|v| v.push(event)))
            .style(style().size(800, 280)),
        )
    });
    let clip = s.get_by_role(Role::Button, "Clip");
    let original_x = clip.rect().expect("clip").x;
    let p = clip.rect().expect("clip").center();
    s.mouse_move(p.x, p.y);
    s.mouse_down(MouseButton::Primary);
    s.mouse_move(p.x + 26.0, p.y);
    clips.update(|clips| clips[0].duration = 9.0);
    s.run_until_idle();
    s.mouse_move(p.x + 52.0, p.y);
    assert_eq!(clips.get_untracked()[0].start, 2.0);
    assert_eq!(clip.rect().expect("rejected clip").x, original_x);
    assert!(events.get_untracked().contains(&ui::EditorEvent::Request(
        ui::ArrangementRequest::Retime(vec![ui::ItemTime {
            id: 1,
            time: 4.0,
            duration: 8.0
        }])
    )));
    s.press_key(Key::Escape);
    assert_eq!(
        clips.get_untracked()[0].duration,
        9.0,
        "kit never restores domain data"
    );
    assert_eq!(
        events.get_untracked().last(),
        Some(&ui::EditorEvent::Cancel)
    );
    let mute = s.get_by_role(Role::Button, "Track Mute");
    events.set(Vec::new());
    mute.a11y(A11yAction::Activate);
    assert_eq!(
        mute.is_checked(),
        Some(false),
        "application rejected the track flag"
    );
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::ArrangementRequest::TrackFlag {
                track: 10,
                flag: ui::TrackFlag::Mute,
                value: true
            }),
            ui::EditorEvent::Commit,
        ]
    );
    clean(&s);
}

#[test]
fn arrangement_short_length_clamps_loop_and_reports_invalid_reactive_length() {
    let (s, (length, range, frame)) = kivact::test::render_with_handle(crate::options(), || {
        let length = signal(0.001);
        let range = signal((0.0, 16.0));
        let frame = signal(0.0);
        (
            (length, range, frame),
            ui::arrangement(
                Vec::new(),
                Vec::new(),
                ui::ArrangementState {
                    loop_range: range.into(),
                    playhead: frame.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .length(length)
            .zoom_range(8.0, 80.0)
            .min_duration(1.0)
            .style(style().size(800, 280)),
        )
    });
    s.get_by_role(Role::Slider, "Loop start")
        .a11y_with_value(A11yAction::SetValue, "8");
    s.get_by_role(Role::Slider, "Loop end")
        .a11y_with_value(A11yAction::SetValue, "8");
    s.get_by_role(Role::Slider, "Playhead")
        .a11y_with_value(A11yAction::SetValue, "8");
    assert_eq!(range.get_untracked(), (0.0, 0.001));
    assert_eq!(frame.get_untracked(), 0.001);
    clean(&s);
    length.set(f64::NAN);
    s.run_until_idle();
    assert!(!s.errors().is_empty());
}

#[test]
fn application_reuses_one_created_point_until_commit_or_cancel() {
    let (s, tracks) = kivact::test::render_with_handle(crate::options(), || {
        let clips = signal(Vec::new());
        let tracks = signal(vec![
            ui::TimelineTrack::new(10, "Track", ui::TrackKind::Audio, rgb(0x34d399))
                .automation(vec![ui::AutomationPoint::new(1, 2.0, 0.25)]),
        ]);
        (
            tracks,
            app_arrangement(clips, tracks, ui::ArrangementState::new())
                .name("Arrange")
                .style(style().size(800, 280)),
        )
    });
    s.mouse_move(400.0, 117.0);
    s.mouse_down(MouseButton::Primary);
    let id = tracks.get_untracked()[0]
        .automation
        .iter()
        .find(|p| p.id != 1)
        .expect("created")
        .id;
    s.mouse_move(426.0, 110.0);
    s.mouse_move(452.0, 108.0);
    assert_eq!(tracks.get_untracked()[0].automation.len(), 2);
    assert!(
        tracks.get_untracked()[0]
            .automation
            .iter()
            .any(|p| p.id == id && p.beat == 9.0)
    );
    s.press_key(Key::Escape);
    assert_eq!(tracks.get_untracked()[0].automation.len(), 1);
    clean(&s);
}

#[test]
fn single_arrangement_delete_captures_selection_before_begin_callback() {
    let (screen, (clips, events)) = kivact::test::render_with_handle(crate::options(), || {
        let clips = signal(vec![
            ui::Clip::new(1, 10, 2.0, 4.0, "First"),
            ui::Clip::new(2, 10, 8.0, 4.0, "Second"),
        ]);
        let selected = signal(vec![1]);
        let events = signal(Vec::new());
        (
            (clips, events),
            ui::arrangement(
                clips,
                vec![ui::TimelineTrack::new(
                    10,
                    "Track",
                    ui::TrackKind::Audio,
                    rgb(0x34d399),
                )],
                ui::ArrangementState {
                    selection: selected.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .length(24.0)
            .on_edit(move |event| {
                if event == ui::EditorEvent::Begin {
                    selected.set(vec![2]);
                    clips.update(|clips| clips[0].label = "Application update".into());
                }
                events.update(|events| events.push(event));
            })
            .name("Captured delete")
            .style(style().size(800, 280)),
        )
    });
    screen
        .get_by_role(Role::Canvas, "Captured delete")
        .press_key(Key::Delete);
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::ArrangementRequest::Delete(vec![1])),
            ui::EditorEvent::Commit,
        ]
    );
    assert_eq!(
        clips.get_untracked(),
        vec![
            ui::Clip::new(1, 10, 2.0, 4.0, "Application update"),
            ui::Clip::new(2, 10, 8.0, 4.0, "Second"),
        ]
    );
    clean(&screen);
}
