//! Role: Stable gradient stop editing, transforms and exact cancellation.
use crate::support::clean;
use kivact::prelude::*;
use kivact::view::EditPhase::{Begin, Commit};
use kivact::view::{A11yAction, Modifiers};
use kivact_ui_creative as ui;
use std::cell::RefCell;
use std::rc::Rc;
fn stops() -> Vec<ui::GradientStop> {
    vec![
        ui::GradientStop::new(0.25, rgb(0x804020)),
        ui::GradientStop::new(0.75, rgb(0x204080)),
    ]
}

#[test]
fn gradient_keys_actions_selection_and_position_reorder_keep_storage_order() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (data, selected)) =
        kivact::test::render_with_handle(crate::support::options(), move || {
            let data = signal(stops());
            let selected = signal(None);
            (
                (data, selected),
                ui::gradient_editor(data, selected)
                    .on_edit(move |phase| seen.borrow_mut().push(phase)),
            )
        });
    let stop = screen.get_by_role(Role::Slider, "Gradient stop 0");
    for action in [
        A11yAction::Select,
        A11yAction::Activate,
        A11yAction::SetValue,
        A11yAction::Increment,
        A11yAction::Decrement,
    ] {
        assert!(stop.a11y_actions().contains(&action));
    }
    stop.a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), Some(0));
    assert!(stop.is_focused());
    stop.a11y(A11yAction::Activate);
    assert!(log.borrow().is_empty());
    stop.press_key(Key::ArrowRight);
    screen.press_key_with(
        Key::ArrowUp,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert!((data.get_untracked()[0].position - 0.36).abs() < 1e-9);
    stop.a11y_set_value("0.9");
    assert_eq!(selected.get_untracked(), Some(0));
    stop.a11y(A11yAction::Increment);
    stop.a11y(A11yAction::Decrement);
    stop.press_key(Key::Home);
    assert_eq!(data.get_untracked()[0].position, 0.0);
    stop.press_key(Key::End);
    assert_eq!(data.get_untracked()[0].position, 1.0);
    screen
        .get_by_role(Role::SpinButton, "Stop position")
        .a11y_set_value("0.6");
    assert_eq!(data.get_untracked()[0].position, 0.6);
    let count = log.borrow().len();
    stop.a11y_set_value("NaN");
    assert_eq!(log.borrow().len(), count);
    stop.press_key(Key::Delete);
    assert_eq!(selected.get_untracked(), Some(0));
    assert_eq!(data.get_untracked().len(), 1);
    assert!(
        log.borrow()
            .as_chunks::<2>()
            .0
            .iter()
            .all(|p| *p == [Begin, Commit])
    );
    clean(&screen);
}
