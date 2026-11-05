//! Role: Arrangement edits transact exactly the model fields they own.
use crate::arrangement_model;
use crate::arrangement_model::arrangement as app_arrangement;
use crate::support::clean;
use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::EditPhase;
use kivact::view::{Modifiers, MouseButton};
use kivact_ui_creative as ui;

#[derive(Clone, Copy)]
struct Model {
    clips: Signal<Vec<ui::Clip>>,
    tracks: Signal<Vec<ui::TimelineTrack>>,
    selected: Signal<Vec<u64>>,
    frame: Signal<f64>,
    range: Signal<(f64, f64)>,
    cycling: Signal<bool>,
    transform: Signal<kivact::interaction::ViewTransform>,
    tool: Signal<ui::TimelineTool>,
    phases: Signal<Vec<EditPhase>>,
    observed: Signal<Vec<(EditPhase, f64)>>,
    disabled: Signal<bool>,
    visible: Signal<bool>,
    present: Signal<bool>,
}
fn fixture() -> (Screen, Model) {
    kivact::test::render_with_handle(crate::support::options(), || {
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
            transform: signal(kivact::interaction::ViewTransform {
                scale: 26.0,
                ..Default::default()
            }),
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
                            selection: m.selected.into_binding(),
                            playhead: m.frame.into_binding(),
                            loop_range: m.range.into_binding(),
                            cycling: m.cycling.into_binding(),
                            transform: m.transform.into_binding(),
                            tool: m.tool.into_binding(),
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
fn arrow_keys_move_selected_clips_by_the_step_and_shift_by_ten_steps() {
    let (s, (clips, step)) = kivact::test::render_with_handle(crate::support::options(), || {
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
                    selection: (signal(vec![1])).into_binding(),
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
