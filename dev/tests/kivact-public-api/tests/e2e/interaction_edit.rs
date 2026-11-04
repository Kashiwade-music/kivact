//! Role: Tests for edit phase handling in interactions.
//!
//! `EditPhase`: the Begin, Commit and Cancel a `splitter` handle reports around a drag,
//! a key press and an assistive technology action, so an app can make one undo
//! step of each.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::ScalarRange;
use kivact::interaction::use_splitter;
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions};
use kivact::view::A11yAction;
use kivact::view::EditPhase;
use kivact::view::MouseButton;

/// The phases an observer saw, each with the value in the binding at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

fn real_window() -> RenderOptions {
    RenderOptions::from_env().mirror(MirrorMode::Required)
}

fn range() -> ScalarRange {
    ScalarRange::new(0.0, 100.0).expect("invariant: finite range")
}

fn observer(value: Signal<f64>, log: &Log) -> impl Fn(EditPhase) + 'static {
    let log = log.clone();
    move |phase| log.borrow_mut().push((phase, value.get_untracked()))
}

fn phases(log: &Log) -> Vec<EditPhase> {
    log.borrow().iter().map(|(phase, _)| *phase).collect()
}

fn handle() -> Node {
    node().name("Split").style(style().size(8, 100))
}

use EditPhase::{Begin, Commit};

#[test]
fn a_drag_is_one_edit_from_press_to_release() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(40.0);
        let divider = handle().part(
            use_splitter(value, range(), Orientation::Horizontal)
                .on_edit(observer(value, &seen))
                .handle(),
        );
        (value, divider)
    });
    let rect = screen.get_by_role(Role::Splitter, "Split").rect().unwrap();
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 40.0)], "before the first write");
    screen.mouse_move(x + 10.0, y);
    screen.mouse_move(x + 25.0, y);
    assert_eq!(value.get_untracked(), 65.0);
    assert_eq!(
        phases(&log),
        [Begin],
        "the moves are not edits of their own"
    );
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *log.borrow(),
        [(Begin, 40.0), (Commit, 65.0)],
        "after the last write"
    );

    // A press that never moves is still one edit, with nothing written.
    log.borrow_mut().clear();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 65.0), (Commit, 65.0)]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn each_key_and_assistive_technology_write_is_its_own_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(50.0);
        let divider = handle().part(
            use_splitter(value, range(), Orientation::Horizontal)
                .key_step(5.0)
                .unwrap()
                .reset_to(Some(20.0))
                .on_edit(observer(value, &seen))
                .handle(),
        );
        (value, divider)
    });
    let split = screen.get_by_role(Role::Splitter, "Split");
    split.focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(*log.borrow(), [(Begin, 50.0), (Commit, 55.0)]);
    for (key, expected) in [
        (Key::ArrowLeft, 50.0),
        (Key::Home, 0.0),
        (Key::End, 100.0),
        (Key::Enter, 20.0),
    ] {
        let before = value.get_untracked();
        log.borrow_mut().clear();
        screen.press_key(key);
        assert_eq!(value.get_untracked(), expected, "{key:?}");
        assert_eq!(
            *log.borrow(),
            [(Begin, before), (Commit, expected)],
            "{key:?}"
        );
    }
    log.borrow_mut().clear();

    split.a11y(A11yAction::Increment);
    split.a11y(A11yAction::Decrement);
    split.a11y_set_value("73");
    assert_eq!(
        *log.borrow(),
        [
            (Begin, 20.0),
            (Commit, 25.0),
            (Begin, 25.0),
            (Commit, 20.0),
            (Begin, 20.0),
            (Commit, 73.0)
        ]
    );
    log.borrow_mut().clear();

    // Input that writes nothing is no edit.
    split.a11y_set_value("not a number");
    split.a11y_set_value("NaN");
    screen.press_key(Key::ArrowUp);
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn every_value_writing_standard_component_reports_on_edit() {
    use kivact::interaction::ListItem;
    use kivact::ui::{checkbox, select, switch, tabs, tag_input, toggle_button};
    let logs = Rc::new(RefCell::new(vec![Vec::new(); 6]));
    let seen = logs.clone();
    let (screen, (checked, switched, pressed, selected, tab, tags)) =
        mount_with(real_window(), move || {
            let checked = signal(false);
            let switched = signal(false);
            let pressed = signal(false);
            let selected = signal(Some("a"));
            let tab = signal("a");
            let tags = signal(Vec::<String>::new());
            let observe = move |index: usize| {
                let seen = seen.clone();
                move |phase| seen.borrow_mut()[index].push(phase)
            };
            let options = vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")];
            (
                (checked, switched, pressed, selected, tab, tags),
                column((
                    checkbox("Check", checked).on_edit(observe(0)),
                    switch("Switch", switched).on_edit(observe(1)),
                    toggle_button("Toggle", pressed).on_edit(observe(2)),
                    select("Choose", options.clone(), selected).on_edit(observe(3)),
                    tabs("Pages", options, tab).on_edit(observe(4)),
                    tag_input("Tags", tags)
                        .on_add(move |tag| tags.update(|tags| tags.push(tag)))
                        .on_edit(observe(5)),
                )),
            )
        });
    screen.get_by_role(Role::CheckBox, "Check").click();
    screen.get_by_role(Role::Switch, "Switch").click();
    screen.get_by_role(Role::Button, "Toggle").click();
    screen.get_by_role(Role::ComboBox, "Choose").focus();
    screen.press_key(Key::ArrowDown);
    screen.get_by_role(Role::Tab, "Beta").click();
    screen.get_by_role(Role::TextBox, "Tags").type_text("new");
    screen.press_key(Key::Enter);
    assert!(checked.get_untracked());
    assert!(switched.get_untracked());
    assert!(pressed.get_untracked());
    assert_eq!(selected.get_untracked(), Some("b"));
    assert_eq!(tab.get_untracked(), "b");
    assert_eq!(tags.get_untracked(), ["new"]);
    for phases in logs.borrow().iter() {
        assert_eq!(*phases, [Begin, Commit]);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn text_input_reports_edit_phases_from_the_native_editor() {
    use kivact::custom::TextEditing;
    use kivact::interaction::text_edit_policy;
    let logs = Rc::new(RefCell::new(vec![Vec::new(); 2]));
    let seen = logs.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(String::new());
        let ui_seen = seen.clone();
        let native = node()
            .role(Role::TextBox)
            .name("Native")
            .focusable(true)
            .text_editing(
                TextEditing::new("")
                    .behavior(text_edit_policy)
                    .on_edit(move |phase| seen.borrow_mut()[0].push(phase)),
            );
        (
            value,
            column((
                native,
                kivact::ui::text_input("Standard", value)
                    .on_edit(move |phase| ui_seen.borrow_mut()[1].push(phase)),
            )),
        )
    });
    screen.get_by_role(Role::TextBox, "Native").type_text("x");
    screen.get_by_role(Role::TextBox, "Standard").type_text("y");
    assert_eq!(value.get_untracked(), "y");
    for phases in logs.borrow().iter() {
        assert_eq!(*phases, [Begin, Commit]);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
