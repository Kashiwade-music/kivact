//! Role: `Numeric`'s edit phases, text parser, mixed value and reactive property state.

use super::clean;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::ui::EditPhase::{self, Begin, Commit};
use kivact::view::MouseButton;
use kivact_ui_creative as ui;
use std::cell::RefCell;
use std::rc::Rc;

/// The phases an observer saw, each with the value in the binding at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

fn observer(value: Signal<f64>, log: &Log) -> impl Fn(EditPhase) + 'static {
    let log = log.clone();
    move |phase| log.borrow_mut().push((phase, value.get_untracked()))
}

fn center(screen: &Screen, role: Role, name: &str) -> (f32, f32) {
    let rect = screen.get_by_role(role, name).rect().expect("laid out");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn gain(log: &Log) -> (Screen, Signal<f64>) {
    let seen = log.clone();
    kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(10.0);
        (
            value,
            ui::number("Gain", value)
                .range(0.0, 100.0)
                .step(1.0)
                .precision(1)
                .on_edit(observer(value, &seen)),
        )
    })
}

#[test]
fn a_scrub_is_one_edit_from_press_to_release() {
    let log = Log::default();
    let (screen, value) = gain(&log);
    let (x, y) = center(&screen, Role::SpinButton, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0)], "before the first write");
    screen.mouse_move(x + 10.0, y);
    screen.mouse_move(x + 20.0, y);
    assert_eq!(value.get_untracked(), 30.0);
    assert_eq!(log.borrow().len(), 1, "the moves are not edits");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 30.0)]);
    assert!(
        screen.query_by_role(Role::TextBox, "Gain").is_none(),
        "a scrub does not open the text field"
    );
    clean(&screen);
}
