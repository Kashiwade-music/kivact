//! Role: Reactive time-step contracts for timeline and dope sheet editors.
use super::*;
#[derive(Clone, Copy)]
struct Model {
    clips: Signal<Vec<ui::Clip>>,
    keys: Signal<Vec<ui::Keyframe>>,
    step: Signal<f64>,
    lanes: Signal<Vec<ui::Lane>>,
    selected: Signal<Vec<u64>>,
    snap: Signal<bool>,
    playhead: Signal<f64>,
    phases: Signal<Vec<kivact::ui::EditPhase>>,
    keys_editor: bool,
}
impl Model {
    fn time(self) -> f64 {
        value(self.clips, self.keys, self.keys_editor)
    }
    fn duration(self) -> f64 {
        if self.keys_editor {
            self.keys.get_untracked()[0].duration
        } else {
            self.clips.get_untracked()[0].duration
        }
    }
    fn reset(self, time: f64, duration: f64) {
        self.clips.update(|v| {
            v[0].start = time;
            v[0].duration = duration;
        });
        self.keys.update(|v| {
            v[0].time = time;
            v[0].duration = duration;
        });
    }
}
fn fixture(keys_editor: bool) -> (Screen, Model) {
    kivact::test::render_with_handle(crate::options(), move || {
        let m = Model {
            clips: signal(vec![ui::Clip::new(1, 10, 0.0, 2.0, "Item")]),
            keys: signal(vec![ui::Keyframe::new(1, 10, 0.0, "Item").duration(2.0)]),
            step: signal(0.5),
            lanes: signal(vec![ui::Lane::new(10, "Lane")]),
            selected: signal(vec![1]),
            snap: signal(true),
            playhead: signal(0.0),
            phases: signal(Vec::new()),
            keys_editor,
        };
        (
            m,
            creative_time_fixture!(keys_editor, m.clips, m.keys, ui::TimeState { selection: m.selected.into(), zoom: (signal(100.0)).into(), playhead: m.playhead.into(), ..ui::TimeState::new() },
                .lanes(m.lanes)
                .zoom_range(1.0, 100.0)
                .step(m.step)
                .snap(m.snap)
                .min_duration(0.25)
                .on_edit(time_observer(m.clips, m.keys, move |phase| m.phases.update(|v| v.push(phase))))
                .name("Step editor")
                .style(style().size(700, 280))
            ),
        )
    })
}
fn drag(screen: &Screen, name: &str, dx: f32) {
    let item = screen.get_by_role(Role::Button, name);
    let p = item.rect().expect("visible time item").center();
    item.drag(p, Point::new(p.x + dx, p.y));
}
#[test]
fn time_step_snaps_absolute_positions_edges_and_playhead_and_preserves_rollback() {
    for keys in [false, true] {
        let (s, m) = fixture(keys);
        drag(&s, "Item", 70.0);
        assert!(near(m.time(), 0.5, 1e-6));
        m.reset(0.2, 2.0);
        s.run_until_idle();
        drag(&s, "Item", 70.0);
        assert!(near(m.time(), 1.0, 1e-6));
        m.reset(0.2, 2.0);
        s.run_until_idle();
        drag(&s, "Item start trim", 70.0);
        assert!(near(m.time(), 1.0, 1e-6));
        assert!(near(m.duration(), 1.2, 1e-6));
        m.reset(0.2, 2.0);
        s.run_until_idle();
        drag(&s, "Item end trim", 70.0);
        assert!(near(m.time(), 0.2, 1e-6));
        assert!(near(m.duration(), 2.8, 1e-6));
        m.reset(0.2, 2.0);
        s.run_until_idle();
        drag(&s, "Item", 0.0);
        drag(&s, "Item start trim", 0.0);
        drag(&s, "Item end trim", 0.0);
        assert!(near(m.time(), 0.2, 1e-6));
        assert!(near(m.duration(), 2.0, 1e-6));
        drag(&s, "Item start trim", 300.0);
        assert!(near(m.duration(), 0.25, 1e-6));
        assert!(near(m.time(), 1.95, 1e-6));
        m.reset(0.2, 2.0);
        s.run_until_idle();
        drag(&s, "Item end trim", -300.0);
        assert!(near(m.time(), 0.2, 1e-6));
        assert!(near(m.duration(), 0.25, 1e-6));
        m.reset(0.2, 2.0);
        s.run_until_idle();
        let p = s.get_by_role(Role::Button, "Item").rect().unwrap().center();
        s.mouse_move(p.x, p.y);
        s.mouse_down(MouseButton::Primary);
        s.mouse_move(p.x + 70.0, p.y);
        assert!(near(m.time(), 1.0, 1e-6));
        m.step.set(0.25);
        s.mouse_move(p.x + 60.0, p.y);
        assert!(near(m.time(), 0.75, 1e-6));
        s.press_key(Key::Escape);
        s.mouse_up(MouseButton::Primary);
        assert!(near(m.time(), 0.2, 1e-6));
        m.step.set(0.5);
        s.mouse_move(270.0, 10.0);
        s.mouse_down(MouseButton::Primary);
        assert!(near(m.playhead.get_untracked(), 0.5, 1e-6));
        s.mouse_move(340.0, 10.0);
        s.mouse_up(MouseButton::Primary);
        assert!(near(m.playhead.get_untracked(), 1.5, 1e-6));
        clean(&s);
    }
}
#[test]
fn time_step_keys_accessibility_reactivity_fallback_and_unsnapped_motion() {
    for keys in [false, true] {
        let (s, m) = fixture(keys);
        let item = s.get_by_role(Role::Button, "Item");
        m.reset(0.2, 2.0);
        s.run_until_idle();
        item.press_key(Key::ArrowRight);
        assert!(near(m.time(), 0.7, 1e-6));
        item.press_key_with(
            Key::ArrowRight,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert!(near(m.time(), 5.7, 1e-6));
        item.press_key(Key::ArrowLeft);
        assert!(near(m.time(), 5.2, 1e-6));
        item.a11y_with_value(A11yAction::SetValue, "0.2");
        assert!(near(m.time(), 0.2, 1e-6));
        item.a11y(A11yAction::Increment);
        assert!(near(m.time(), 0.7, 1e-6));
        item.a11y(A11yAction::Decrement);
        assert!(near(m.time(), 0.2, 1e-6));
        item.a11y_with_value(A11yAction::SetValue, "NaN");
        assert!(near(m.time(), 0.2, 1e-6));
        let head = s.get_by_role(Role::Slider, "Playhead");
        head.a11y_with_value(A11yAction::SetValue, "0.2");
        assert!(near(m.playhead.get_untracked(), 0.2, 1e-6));
        head.a11y(A11yAction::Increment);
        assert!(near(m.playhead.get_untracked(), 0.7, 1e-6));
        head.a11y(A11yAction::Decrement);
        assert!(near(m.playhead.get_untracked(), 0.2, 1e-6));
        for step in [0.25, 0.0, -0.5, f64::NAN, f64::INFINITY] {
            m.step.set(step);
            m.reset(0.0, 2.0);
            s.run_until_idle();
            item.press_key(Key::ArrowRight);
            let unit = if step == 0.25 { 0.25 } else { 1.0 };
            assert!(near(m.time(), unit, 1e-6));
            head.a11y_with_value(A11yAction::SetValue, "0");
            head.a11y(A11yAction::Increment);
            assert!(near(m.playhead.get_untracked(), unit, 1e-6));
            m.reset(0.0, 2.0);
            s.run_until_idle();
            drag(&s, "Item", 70.0);
            assert!(near(m.time(), if step == 0.25 { 0.75 } else { 1.0 }, 1e-6));
        }
        m.step.set(0.25);
        m.reset(0.0, 2.0);
        s.run_until_idle();
        drag(&s, "Item", 70.0);
        assert!(near(m.time(), 0.75, 1e-6));
        m.step.set(f64::from_bits(1));
        m.reset(0.0, 2.0);
        s.run_until_idle();
        drag(&s, "Item", 70.0);
        assert!(m.time().is_finite());
        assert!(near(m.time(), 0.7, 1e-6));
        m.snap.set(false);
        m.reset(0.2, 2.0);
        s.run_until_idle();
        drag(&s, "Item", 70.0);
        assert!(near(m.time(), 0.9, 1e-6));
        clean(&s);
    }
}

#[test]
fn time_step_overflowing_key_and_accessibility_moves_preserve_finite_values() {
    for keys in [false, true] {
        let (s, m) = fixture(keys);
        m.step.set(f64::MAX);
        let editor = s.get_by_role(Role::Canvas, "Step editor");
        editor.press_key_with(
            Key::ArrowRight,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(m.time(), 0.0);
        m.lanes.set(Vec::new());
        m.reset(1e308, 2.0);
        s.run_until_idle();
        assert_eq!(
            m.selected.get_untracked(),
            [1],
            "hidden item remains selected"
        );
        editor.press_key(Key::ArrowRight);
        assert_eq!(m.time(), 1e308);
        m.playhead.set(1e308);
        s.run_until_idle();
        s.get_by_role(Role::Slider, "Playhead")
            .a11y(A11yAction::Increment);
        assert_eq!(m.playhead.get_untracked(), 1e308);
        assert!(
            m.phases.get_untracked().is_empty(),
            "overflow does not start an edit"
        );
        clean(&s);
    }
}

#[test]
fn a_selected_group_snaps_by_the_pressed_item_and_keeps_its_spacing() {
    for keys_editor in [false, true] {
        let (s, (clips, keys)) = kivact::test::render_with_handle(crate::options(), move || {
            let clips = signal(vec![
                ui::Clip::new(1, 10, 0.3, 0.5, "First"),
                ui::Clip::new(2, 10, 1.7, 0.5, "Second"),
            ]);
            let keys = signal(vec![
                ui::Keyframe::new(1, 10, 0.3, "First").duration(0.5),
                ui::Keyframe::new(2, 10, 1.7, "Second").duration(0.5),
            ]);
            (
                (clips, keys),
                creative_time_fixture!(keys_editor, clips, keys, ui::TimeState { selection: (signal(vec![1, 2])).into(), zoom: (signal(100.0)).into(), ..ui::TimeState::new() },
                    .lanes(vec![ui::Lane::new(10, "Lane")])
                    .zoom_range(1.0, 100.0)
                    .snap(true)
                    .step(1.0)
                    .name("Group")
                    .style(style().size(700, 280))
                ),
            )
        });
        let times = || -> Vec<f64> {
            if keys_editor {
                keys.get_untracked().iter().map(|k| k.time).collect()
            } else {
                clips.get_untracked().iter().map(|c| c.start).collect()
            }
        };
        drag(&s, "First", 60.0);
        let times = times();
        assert!(near(times[0], 1.0, 1e-6), "{times:?}");
        assert!(near(times[1], 2.4, 1e-6), "{times:?}");
        clean(&s);
    }
}
