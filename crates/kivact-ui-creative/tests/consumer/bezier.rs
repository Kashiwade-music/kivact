//! Role: Public Bezier control edits, cancellation and read-only curve previews.
use super::clean;
use kivact::prelude::*;
use kivact::ui::EditPhase::{Begin, Commit};
use kivact::view::{A11yAction, Modifiers};
use kivact_ui_creative as ui;
use std::cell::RefCell;
use std::rc::Rc;

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
