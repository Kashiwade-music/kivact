//! Role: Tests for edit phase handling in interactions.
//!
//! `EditPhase`: the Begin, Commit and Cancel a `splitter` handle reports around a drag,
//! a key press and an assistive technology action, so an app can make one undo
//! step of each.

use kivact::view::{EditPhase, Node, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

use kivact::interaction::ScalarRange;
use kivact::interaction::use_splitter;
use kivact::interaction::{DraftCommit, DraftOptions, EditSession, use_draft};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions};
use kivact::view::A11yAction;
use kivact_public_api_tests::support::{mount, mount_with};

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
        (Key::Enter, 0.0),
        (Key::Enter, 100.0),
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
    // The reset option still belongs to double click, independently of Enter.
    log.borrow_mut().clear();
    split.double_click();
    assert_eq!(value.get_untracked(), 20.0);
    assert_eq!(
        *log.borrow(),
        [
            (Begin, 100.0),
            (Commit, 100.0),
            (Begin, 100.0),
            (Commit, 20.0)
        ]
    );
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
    use kivact::ui::{
        StepperItem, checkbox, select, stepper, switch, tabs, tag_input, toggle_button,
    };
    let logs = Rc::new(RefCell::new(vec![Vec::new(); 7]));
    let seen = logs.clone();
    let (screen, (checked, switched, pressed, selected, tab, tags, step)) =
        mount_with(real_window(), move || {
            let checked = signal(false);
            let switched = signal(false);
            let pressed = signal(false);
            let selected = signal(Some("a"));
            let tab = signal("a");
            let tags = signal(Vec::<String>::new());
            let step = signal(0usize);
            let observe = move |index: usize| {
                let seen = seen.clone();
                move |phase| seen.borrow_mut()[index].push(phase)
            };
            let options = vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")];
            (
                (checked, switched, pressed, selected, tab, tags, step),
                column((
                    checkbox("Check", checked).on_edit(observe(0)),
                    switch("Switch", switched).on_edit(observe(1)),
                    toggle_button("Toggle", pressed).on_edit(observe(2)),
                    select(options.clone(), selected)
                        .label("Choose")
                        .on_edit(observe(3)),
                    tabs("Pages", options, tab).on_edit(observe(4)),
                    tag_input(tags).label("Tags").on_edit(observe(5)),
                    stepper(vec![StepperItem::new("One"), StepperItem::new("Two")], step)
                        .on_edit(observe(6)),
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
    screen.get_by_role(Role::ListItem, "Two").click();
    assert!(checked.get_untracked());
    assert!(switched.get_untracked());
    assert!(pressed.get_untracked());
    assert_eq!(selected.get_untracked(), Some("b"));
    assert_eq!(tab.get_untracked(), "b");
    assert_eq!(tags.get_untracked(), ["new"]);
    assert_eq!(step.get_untracked(), 1);
    for phases in logs.borrow().iter() {
        assert_eq!(*phases, [Begin, Commit]);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn stepper_reports_no_phase_for_an_outside_write() {
    use kivact::ui::{StepperItem, stepper};
    let logs = Rc::new(RefCell::new(Vec::<EditPhase>::new()));
    let seen = logs.clone();
    let (screen, step) = mount_with(real_window(), move || {
        let step = signal(0usize);
        (
            step,
            stepper(vec![StepperItem::new("One"), StepperItem::new("Two")], step)
                .on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    step.set(1);
    screen.run_until_idle();
    assert!(logs.borrow().is_empty());
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
                kivact::ui::text_input(value)
                    .label("Standard")
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

#[test]
fn replace_initial_changes_what_the_active_edit_holds_and_cancel_restores() {
    let (screen, value) = mount_with(real_window(), || {
        let value = signal(0);
        let edit = EditSession::new(move |restored: i32| value.set(restored));
        (
            value,
            column((
                button("Begin").on_press(move || {
                    edit.begin(1);
                    edit.replace_initial(5);
                }),
                button("Cancel").on_press(move || {
                    edit.cancel();
                }),
                text(move || format!("{:?}", edit.with_initial(|v| *v))).test_id("held"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Begin").click();
    assert_eq!(screen.get_by_test_id("held").text(), "Some(5)");
    screen.get_by_role(Role::Button, "Cancel").click();
    assert_eq!(value.get_untracked(), 5);
    assert_eq!(screen.get_by_test_id("held").text(), "None");
}

#[test]
fn an_explicit_draft_writes_on_commit_follows_external_writes_and_cancels() {
    let (screen, (page, shown, editing, valid, draft)) = mount(|| {
        let page = signal(3usize);
        let draft = use_draft(
            page,
            DraftOptions::new(
                |page: &usize| page.to_string(),
                |text: &String| text.trim().parse::<usize>().ok(),
            ),
        );
        let shown = draft.draft();
        (
            (page, shown, draft.is_editing(), draft.is_valid(), draft),
            text(move || shown.get()),
        )
    });
    assert_eq!(shown.get_untracked(), "3");
    draft.edit("7".to_owned());
    assert_eq!(
        page.get_untracked(),
        3,
        "an explicit draft writes nothing while it is edited"
    );
    assert!(editing.get_untracked());
    draft.commit();
    assert_eq!(page.get_untracked(), 7);
    assert!(!editing.get_untracked());
    draft.edit("x".to_owned());
    assert!(!valid.get_untracked());
    draft.commit();
    assert_eq!(page.get_untracked(), 7, "a rejected commit writes nothing");
    assert!(editing.get_untracked(), "a rejected draft stays");
    page.set(9);
    screen.run_until_idle();
    assert_eq!(
        shown.get_untracked(),
        "9",
        "an external write replaces an unrelated draft"
    );
    draft.edit("4".to_owned());
    draft.cancel();
    assert_eq!(shown.get_untracked(), "9");
    assert_eq!(page.get_untracked(), 9);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_live_draft_writes_each_accepted_edit_and_keeps_the_text_the_writer_clamped() {
    let (screen, (stored, shown, valid, draft)) = mount(|| {
        let stored = signal(5.0f64);
        let clamped = binding(stored, move |next: f64| stored.set(next.min(10.0)));
        let draft = use_draft(
            clamped,
            DraftOptions::new(
                |value: &f64| value.to_string(),
                |text: &String| text.trim().parse::<f64>().ok(),
            )
            .commit(DraftCommit::Live),
        );
        let shown = draft.draft();
        (
            (stored, shown, draft.is_valid(), draft),
            text(move || shown.get()),
        )
    });
    draft.edit("12".to_owned());
    screen.run_until_idle();
    assert_eq!(stored.get_untracked(), 10.0);
    assert_eq!(
        shown.get_untracked(),
        "12",
        "the typed text stays over the value it wrote"
    );
    draft.commit();
    assert_eq!(shown.get_untracked(), "10");
    draft.edit("x".to_owned());
    screen.run_until_idle();
    assert!(!valid.get_untracked());
    assert_eq!(stored.get_untracked(), 10.0);
    stored.set(3.0);
    screen.run_until_idle();
    assert_eq!(
        shown.get_untracked(),
        "3",
        "an external write replaces a draft that stands for no value"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
