//! Role: `Numeric`'s edit phases, text parser, mixed value and reactive property state.

use crate::support::clean;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::EditPhase::{Begin, Commit};
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

fn center(screen: &Screen, role: Role, name: &str) -> (f64, f64) {
    let rect = screen.get_by_role(role, name).rect().expect("laid out");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn gain(log: &Log) -> (Screen, Signal<f64>) {
    let seen = log.clone();
    kivact::test::render_with_handle(crate::support::options(), move || {
        let value = signal(10.0);
        (
            value,
            ui::number("Gain", value)
                .range(
                    kivact::interaction::ScalarRange::new(0.0, 100.0)
                        .expect("invariant: finite ordered control range"),
                )
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

#[test]
fn numeric_arithmetic_submit_cancel_and_blur_commit() {
    let (screen, value) = kivact::test::render_with_handle(crate::support::options(), || {
        let value = signal(0.25);
        (
            value,
            column((
                ui::number("Gain", value)
                    .range(
                        kivact::interaction::ScalarRange::new(-10.0, 10.0)
                            .expect("invariant: finite ordered control range"),
                    )
                    .precision(3),
                kivact::ui::button("Outside").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    let editor = screen.get_by_role(Role::TextBox, "Gain");
    editor.type_text("(2 + 3) / 4");
    screen.press_key(Key::ArrowLeft);
    assert_eq!(editor.text_selection(), Some((10, 10)));
    screen.press_key(Key::Home);
    assert_eq!(editor.text_selection(), Some((0, 0)));
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::End);
    assert_eq!(editor.text_selection(), Some((11, 11)));
    assert_eq!(value.get_untracked(), 0.25);
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 1.25);
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("1 / 0");
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 1.25);
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("3 * 2");
    screen.get_by_role(Role::Button, "Outside").click();
    assert_eq!(value.get_untracked(), 6.0);
    clean(&screen);
}
