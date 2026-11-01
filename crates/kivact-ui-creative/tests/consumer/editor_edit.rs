//! Role: Time-editor edit boundaries, rollback ownership and surface forwarding.
use super::*;
use kivact::interaction::ViewTransform;
use kivact::ui::EditPhase;
#[derive(Clone, Copy)]
struct Model {
    clips: Signal<Vec<ui::Clip>>,
    keys: Signal<Vec<ui::Keyframe>>,
    selected: Signal<Vec<u64>>,
    pan: Signal<Point>,
    zoom: Signal<f64>,
    frame: Signal<f64>,
    phases: Signal<Vec<EditPhase>>,
    disabled: Signal<bool>,
    visible: Signal<bool>,
    present: Signal<bool>,
}
fn fixture(keys: bool) -> (Screen, Model) {
    kivact::test::render_with_handle(crate::options(), move || {
        let m = Model {
            clips: signal(vec![
                ui::Clip::new(1, 10, 40.0, 70.0, "Item"),
                ui::Clip::new(2, 10, 140.0, 40.0, "Other"),
            ]),
            keys: signal(vec![
                ui::Keyframe::new(1, 10, 40.0, "Item"),
                ui::Keyframe::new(2, 10, 140.0, "Other"),
            ]),
            selected: signal(vec![1]),
            pan: signal(Point::new(0.0, 0.0)),
            zoom: signal(1.0),
            frame: signal(0.0),
            phases: signal(Vec::new()),
            disabled: signal(false),
            visible: signal(true),
            present: signal(true),
        };
        (
            m,
            column((
                show(m.present, move || {
                    let observer = move |p| m.phases.update(|v| v.push(p));
                    creative_time_fixture!(keys, m.clips, m.keys, ui::TimeState { selection: m.selected.into(), pan: m.pan.into(), zoom: m.zoom.into(), playhead: m.frame.into()},
                        .on_edit(time_observer(m.clips, m.keys, observer))
                        .disabled(m.disabled)
                        .hidden(move || !m.visible.get())
                        .name("Editor")
                        .style(style().size(640, 280))
                    )
                }),
                kivact::ui::button("Outside"),
            )),
        )
    })
}
fn start(screen: &Screen) -> Point {
    let p = screen
        .get_by_role(Role::Button, "Item")
        .rect()
        .expect("visible item geometry")
        .center();
    screen.mouse_move(p.x, p.y);
    screen.mouse_down(MouseButton::Primary);
    p
}
#[test]
fn time_drag_is_one_transaction_and_selection_alone_has_no_edit() {
    for keys in [false, true] {
        let (s, m) = fixture(keys);
        s.get_by_role(Role::Button, "Item").click();
        assert!(m.phases.get_untracked().is_empty());
        let p = start(&s);
        s.mouse_move(p.x + 12.0, p.y);
        s.mouse_move(p.x + 25.0, p.y);
        assert_eq!(value(m.clips, m.keys, keys), 65.0);
        assert_eq!(m.phases.get_untracked(), [EditPhase::Begin]);
        s.mouse_up(MouseButton::Secondary);
        assert_eq!(m.phases.get_untracked(), [EditPhase::Begin]);
        s.mouse_up(MouseButton::Primary);
        assert_eq!(
            m.phases.get_untracked(),
            [EditPhase::Begin, EditPhase::Commit]
        );
        clean(&s);
    }
}
#[test]
fn time_cancel_restores_fields_and_preserves_external_changes_and_deletions() {
    for keys in [false, true] {
        let (s, m) = fixture(keys);
        let p = start(&s);
        s.mouse_move(p.x + 20.0, p.y);
        m.clips.update(|v| {
            v[0].label = "External".into();
            v[0].muted = true;
            v.remove(1);
        });
        m.keys.update(|v| {
            v[0].label = "External".into();
            v[0].muted = true;
            v.remove(1);
        });
        s.press_key(Key::Escape);
        assert_eq!(value(m.clips, m.keys, keys), 40.0);
        assert_eq!(
            m.phases.get_untracked(),
            [EditPhase::Begin, EditPhase::Cancel]
        );
        assert_eq!(m.clips.get_untracked().len(), 1);
        assert_eq!(m.keys.get_untracked().len(), 1);
        assert_eq!(m.clips.get_untracked()[0].label, "External");
        assert!(m.clips.get_untracked()[0].muted);
        assert_eq!(m.keys.get_untracked()[0].label, "External");
        assert!(m.keys.get_untracked()[0].muted);
        clean(&s);
    }
}
#[test]
fn time_cancel_lifecycle_paths_restore_before_cancel() {
    for keys in [false, true] {
        for mode in 0..6 {
            let (s, m) = fixture(keys);
            let p = s
                .get_by_role(Role::Button, "Item")
                .rect()
                .expect("visible item geometry")
                .center();
            let contact = if mode == 5 {
                let contact = s.contact_down(kivact::view::PointerKind::Touch, p.x, p.y);
                s.contact_move(contact, p.x + 20.0, p.y);
                Some(contact)
            } else {
                s.mouse_move(p.x, p.y);
                s.mouse_down(MouseButton::Primary);
                s.mouse_move(p.x + 20.0, p.y);
                None
            };
            match mode {
                0 => m.disabled.set(true),
                1 => m.visible.set(false),
                2 => m.present.set(false),
                3 => s.get_by_role(Role::Button, "Outside").focus(),
                4 => s.set_window_focused(false),
                _ => s.contact_cancel(contact.expect("the touch is down")),
            };
            s.run_until_idle();
            assert_eq!(
                value(m.clips, m.keys, keys),
                40.0,
                "keys={keys} mode={mode}"
            );
            assert_eq!(
                m.phases.get_untracked(),
                [EditPhase::Begin, EditPhase::Cancel],
                "keys={keys} mode={mode}"
            );
            clean(&s);
        }
    }
}
#[test]
fn time_keys_playhead_at_and_view_each_emit_one_pair() {
    for keys in [false, true] {
        let (s, m) = fixture(keys);
        s.get_by_role(Role::Canvas, "Editor")
            .press_key(Key::ArrowRight);
        assert_eq!(value(m.clips, m.keys, keys), 41.0);
        let slider = s.get_by_role(Role::Slider, "Playhead");
        slider.press_key(Key::ArrowRight);
        assert_eq!(m.frame.get_untracked(), 1.0);
        assert_eq!(value(m.clips, m.keys, keys), 41.0);
        slider.a11y_with_value(A11yAction::SetValue, "12.5");
        assert_eq!(m.frame.get_untracked(), 12.5);
        slider.a11y(A11yAction::Increment);
        slider.a11y(A11yAction::Decrement);
        assert_eq!(m.frame.get_untracked(), 12.5);
        s.get_by_role(Role::Canvas, "Editor").scroll(0.0, 10.0);
        assert_eq!(
            m.phases.get_untracked(),
            [EditPhase::Begin, EditPhase::Commit].repeat(6)
        );
        clean(&s);
    }
}
#[test]
fn surface_forwards_only_shared_transform_edits() {
    for bound in [false, true] {
        let (s, (transform, phases)) =
            kivact::test::render_with_handle(crate::options(), move || {
                let transform = signal(ViewTransform::default());
                let phases = signal(Vec::<EditPhase>::new());
                let surface = ui::viewport_surface(None)
                    .on_edit(move |p| phases.update(|v| v.push(p)))
                    .name("Surface")
                    .style(style().size(400, 200));
                let surface = if bound {
                    surface.transform(transform)
                } else {
                    surface
                };
                ((transform, phases), surface)
            });
        let canvas = s.get_by_role(Role::Canvas, "Surface");
        let p = canvas.rect().expect("visible item geometry").center();
        s.mouse_move(p.x, p.y);
        s.mouse_down(MouseButton::Middle);
        s.mouse_move(p.x + 20.0, p.y + 10.0);
        s.mouse_up(MouseButton::Middle);
        if bound {
            assert_eq!(transform.get_untracked().offset, Point::new(20.0, 10.0));
            assert_eq!(
                phases.get_untracked(),
                [EditPhase::Begin, EditPhase::Commit]
            );
        } else {
            assert_eq!(transform.get_untracked(), ViewTransform::default());
            assert!(phases.get_untracked().is_empty());
        }
        clean(&s);
    }
}

#[test]
fn time_trim_pan_snap_and_competing_edits_use_the_original_snapshot() {
    let (s, m) = fixture(false);
    let trim = s.get_by_role(Role::Button, "Item end trim");
    let p = trim.rect().expect("trim").center();
    s.mouse_move(p.x, p.y);
    s.mouse_down(MouseButton::Primary);
    s.mouse_move(p.x + 12.0, p.y);
    s.mouse_move(p.x + 30.0, p.y);
    assert_eq!(m.clips.get_untracked()[0].duration, 100.0);
    s.get_by_role(Role::Slider, "Playhead")
        .a11y_with_value(A11yAction::SetValue, "99");
    s.press_key(Key::ArrowRight);
    s.get_by_role(Role::Canvas, "Editor").scroll(0.0, 20.0);
    assert_eq!(m.frame.get_untracked(), 0.0);
    assert_eq!(m.pan.get_untracked(), Point::new(0.0, 0.0));
    s.press_key(Key::Escape);
    assert_eq!(m.clips.get_untracked()[0].duration, 70.0);
    s.mouse_up(MouseButton::Primary);
    m.phases.set(Vec::new());
    let p = s
        .get_by_role(Role::Canvas, "Editor")
        .rect()
        .expect("canvas")
        .center();
    s.mouse_move(p.x, p.y);
    s.mouse_down(MouseButton::Middle);
    s.mouse_move(p.x + 20.0, p.y + 10.0);
    s.press_key(Key::Escape);
    assert_eq!(m.pan.get_untracked(), Point::new(0.0, 0.0));
    assert_eq!(
        m.phases.get_untracked(),
        [EditPhase::Begin, EditPhase::Cancel]
    );
    clean(&s);
}
