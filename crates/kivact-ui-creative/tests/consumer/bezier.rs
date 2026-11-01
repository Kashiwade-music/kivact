//! Role: Public Bezier control edits, cancellation and read-only curve previews.
use super::*;
use kivact::ui::EditPhase::{Begin, Cancel, Commit};
use kivact::view::{PointerKind, callback};

#[test]
fn easing_samples_follow_time_and_keep_finite_overshoot() {
    let ease = ui::CubicBezier::default();
    for (time, expected) in [
        (0.0, 0.0),
        (0.25, 0.40851059),
        (0.5, 0.80240339),
        (1.0, 1.0),
    ] {
        assert!((ease.progress(time) - expected).abs() < 1e-6);
    }
    assert_eq!(ease.progress(f64::NAN), 0.0);
    assert_eq!(ease.progress(f64::NEG_INFINITY), 0.0);
    assert_eq!(ease.progress(f64::INFINITY), 1.0);
    let overshoot = ui::CubicBezier::new(0.0, 2.0, 1.0, 2.0);
    assert!(overshoot.progress(0.5) > 1.0);
    assert_eq!(ease.samples(0).len(), 2);
    assert_eq!(ease.samples(usize::MAX).len(), 513);
    let extreme = ui::CubicBezier::new(0.0, f64::MAX, 1.0, -f64::MAX);
    assert!(
        extreme
            .samples(513)
            .iter()
            .all(|p| p.x.is_finite() && p.y.is_finite())
    );
    let (screen, value) = kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(ease);
        let preview: ui::CurvePreview = ui::curve_preview(move || value.get().samples(64));
        (
            value,
            column((ui::bezier_curve(value), preview.test_id("sampled"))),
        )
    });
    let before = screen.screenshot().hash();
    screen
        .get_by_role(Role::Slider, "Control point 1")
        .press_key(Key::ArrowUp);
    assert_ne!(value.get_untracked(), ease);
    assert_ne!(screen.screenshot().hash(), before);
    clean(&screen);
}

#[test]
fn progress_solves_x_for_time_to_better_than_a_millionth() {
    // Controls on the diagonal make both coordinates equal to the parameter.
    let linear = ui::CubicBezier::new(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    let steep = ui::CubicBezier::new(0.0, 0.0, 1.0, 1.0);
    for i in 1..1000 {
        let time = f64::from(i) / 1000.0;
        assert!((linear.progress(time) - time).abs() < 1e-9, "{time}");
        // The mirror symmetry of an ease-in-out curve about its midpoint.
        let mirrored = steep.progress(time) + steep.progress(1.0 - time);
        assert!((mirrored - 1.0).abs() < 1e-9, "{time}");
    }
}
#[test]
fn bezier_constructor_clamps_x_and_retains_finite_overshoot_in_a_small_view() {
    let curve = ui::CubicBezier::new(f64::NEG_INFINITY, f64::INFINITY, f64::INFINITY, -2.0);
    assert_eq!(
        curve,
        ui::CubicBezier {
            x1: 0.0,
            y1: 0.1,
            x2: 1.0,
            y2: -2.0
        }
    );
    let fallback = ui::CubicBezier::new(f64::NAN, f64::NAN, f64::NAN, f64::NEG_INFINITY);
    assert_eq!(fallback, ui::CubicBezier::default());
    let (screen, value) = kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(ui::CubicBezier::new(0.25, -f64::MAX, 0.75, f64::MAX));
        (value, ui::bezier_curve(value).style(style().size(32, 24)))
    });
    screen.get_by_role(Role::Slider, "Control point 1");
    screen.get_by_role(Role::Slider, "Control point 2");
    screen.screenshot();
    assert_eq!(value.get_untracked().y2, f64::MAX);
    clean(&screen);
}

#[test]
fn bezier_keys_and_advertised_actions_make_single_edits() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, value) = kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(ui::CubicBezier::default());
        (
            value,
            ui::bezier_curve(value).on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    let first = screen.get_by_role(Role::Slider, "Control point 1");
    for action in [
        A11yAction::Select,
        A11yAction::Activate,
        A11yAction::SetValue,
        A11yAction::Increment,
        A11yAction::Decrement,
    ] {
        assert!(first.a11y_actions().contains(&action), "{action:?}");
    }
    first.a11y(A11yAction::Select);
    assert!(first.is_focused());
    first.a11y(A11yAction::Activate);
    assert!(log.borrow().is_empty());
    first.press_key(Key::ArrowRight);
    screen.press_key_with(
        Key::ArrowUp,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    screen.press_key_with(
        Key::ArrowLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!((value.get_untracked().x1 - 0.259).abs() < 1e-9);
    assert!((value.get_untracked().y1 - 0.2).abs() < 1e-9);
    first.a11y(A11yAction::Increment);
    first.a11y(A11yAction::Decrement);
    first.a11y_with_value(A11yAction::SetValue, "0.4,1.5");
    assert_eq!(
        (value.get_untracked().x1, value.get_untracked().y1),
        (0.4, 1.5)
    );
    first.a11y_with_value(A11yAction::SetValue, "0.5 -0.25");
    assert_eq!(
        (value.get_untracked().x1, value.get_untracked().y1),
        (0.5, -0.25)
    );
    assert_eq!(log.borrow().len(), 14);
    assert!(
        log.borrow()
            .as_chunks::<2>()
            .0
            .iter()
            .all(|phases| *phases == [Begin, Commit])
    );
    first.a11y_with_value(A11yAction::SetValue, "NaN,0.5");
    assert_eq!(log.borrow().len(), 14);
    assert_eq!(value.get_untracked().x2, 0.25);
    clean(&screen);
}

#[test]
fn bezier_drags_commit_or_restore_exact_external_values_and_report_disposal() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (value, shown, disabled, hidden)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let value = signal(ui::CubicBezier::default());
            let shown = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let observer =
                callback(move |phase| seen.borrow_mut().push((phase, value.get_untracked())));
            (
                (value, shown, disabled, hidden),
                show(shown, move || {
                    ui::bezier_curve(value)
                        .on_edit(move |value| observer.try_call(value))
                        .disabled(disabled)
                        .hidden(hidden)
                }),
            )
        });
    for mode in 0..6 {
        if mode == 1 {
            value.set(ui::CubicBezier {
                x1: -5.0,
                y1: f64::INFINITY,
                x2: 2.0,
                y2: f64::NEG_INFINITY,
            });
            screen.run_until_idle();
        }
        let initial = value.get_untracked();
        let first = screen.get_by_role(Role::Slider, "Control point 1");
        let point = first.rect().expect("control point geometry").center();
        let contact = if mode == 2 {
            let contact = screen.contact_down(PointerKind::Touch, point.x, point.y);
            screen.contact_move(contact, point.x + 15.0, point.y - 10.0);
            Some(contact)
        } else {
            screen.mouse_move(point.x, point.y);
            screen.mouse_down(MouseButton::Primary);
            screen.mouse_move(point.x + 15.0, point.y - 10.0);
            None
        };
        let moved = value.get_untracked();
        assert_ne!(moved, initial);
        assert_eq!(log.borrow().last().map(|item| item.0), Some(Begin));
        if mode == 0 {
            screen.press_key(Key::ArrowRight);
            first.a11y_with_value(A11yAction::SetValue, "0.9,0.9");
            assert_eq!(value.get_untracked(), moved);
        }
        match mode {
            0 => screen.mouse_up(MouseButton::Primary),
            1 => screen.press_key(Key::Escape),
            2 => screen.contact_cancel(contact.expect("the touch is down")),
            3 => {
                disabled.set(true);
                screen.run_until_idle();
            }
            4 => {
                hidden.set(true);
                screen.run_until_idle();
            }
            _ => {
                shown.set(false);
                screen.run_until_idle();
            }
        }
        let terminal = if mode == 0 { Commit } else { Cancel };
        assert_eq!(log.borrow().last().map(|item| item.0), Some(terminal));
        if mode > 0 {
            assert_eq!(value.get_untracked(), initial);
        }
        assert_eq!(
            log.borrow().last().map(|item| item.1),
            Some(value.get_untracked())
        );
        if mode != 2 {
            screen.mouse_up(MouseButton::Primary);
        }
        disabled.set(false);
        hidden.set(false);
        screen.run_until_idle();
    }
    assert_eq!(log.borrow().len(), 12);
    clean(&screen);
}

#[test]
fn curve_preview_reacts_without_writes_and_limits_samples() {
    let (screen, samples) = kivact::test::render_with_handle(crate::options(), || {
        let samples = signal(
            (0..1000)
                .map(|i| Point::new(i as f32 / 512.0, i as f32 / 512.0))
                .collect::<Vec<_>>(),
        );
        (
            samples,
            column((
                ui::curve_preview(samples),
                ui::curve_preview(vec![Point::new(0.0, -2.0), Point::new(1.0, 3.0)])
                    .style(style().size(24, 16))
                    .name("Tiny preview"),
            )),
        )
    });
    screen.get_by_role(Role::Canvas, "Curve preview");
    let original = screen.screenshot().hash();
    samples.update(|points| points.truncate(513));
    screen.run_until_idle();
    assert_eq!(
        screen.screenshot().hash(),
        original,
        "samples beyond the limit do not affect the plot"
    );
    let changed = vec![
        Point::new(0.0, 0.0),
        Point::new(0.25, -2.0),
        Point::new(0.75, 3.0),
        Point::new(1.0, 1.0),
    ];
    samples.set(changed.clone());
    screen.run_until_idle();
    assert_ne!(screen.screenshot().hash(), original);
    assert_eq!(samples.get_untracked(), changed);
    samples.set(vec![
        Point::new(f32::NAN, 0.0),
        Point::new(1.0, f32::INFINITY),
        Point::new(0.5, 2e6),
    ]);
    screen.run_until_idle();
    screen.screenshot();
    assert!(
        screen
            .get_by_role(Role::Canvas, "Tiny preview")
            .a11y_actions()
            .is_empty()
    );
    clean(&screen);
}

#[test]
fn bezier_release_applies_its_position_when_intermediate_moves_are_consumed() {
    use kivact::view::{Handled, PointerEvent, PointerPhase};
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, value) = kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(ui::CubicBezier::default());
        (
            value,
            ui::bezier_curve(value)
                .on_edit(move |phase| seen.borrow_mut().push((phase, value.get_untracked())))
                .on_pointer(|event: PointerEvent| {
                    if event.phase == PointerPhase::Move {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
        )
    });
    let initial = value.get_untracked();
    let point = screen
        .get_by_role(Role::Slider, "Control point 1")
        .rect()
        .expect("point")
        .center();
    screen.mouse_move(point.x, point.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(point.x + 20.0, point.y - 10.0);
    assert_eq!(
        value.get_untracked(),
        initial,
        "the source received no Move"
    );
    screen.mouse_up(MouseButton::Primary);
    let final_value = value.get_untracked();
    assert_ne!(final_value, initial);
    assert_eq!(*log.borrow(), [(Begin, initial), (Commit, final_value)]);
    clean(&screen);
}
