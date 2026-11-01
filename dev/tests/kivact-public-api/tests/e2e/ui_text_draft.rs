//! Role: Tests a standard text input that edits a draft, on real windows.
//!
//! Enter commits the text, Escape writes the starting text back, leaving the
//! field commits a changed and valid text and cancels an unchanged one, an
//! invalid draft is not committed, a failing commit leaves it open, and each
//! ending reports its `EditPhase` and returns the focus where asked.

use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::EditPhase;
use kivact::view::node_ref;
use std::cell::RefCell;
use std::rc::Rc;

use EditPhase::{Begin, Cancel, Commit};

type Log = Rc<RefCell<Vec<EditPhase>>>;

struct Rename {
    name: Signal<String>,
    draft: Signal<String>,
    editing: Signal<bool>,
    log: Log,
}

/// A name, a button that opens a rename field over it and another control to
/// leave the field for.
fn rename_page(invalid_with: Option<char>, fail: bool) -> (kivact::test::Screen, Rename) {
    mount(move || {
        let name = signal("Old".to_string());
        let draft = signal(String::new());
        let editing = signal(false);
        let log = Log::default();
        let opener = node_ref();
        let seen = log.clone();
        let view = column((
            button("Rename").node_ref(opener).on_press(move || {
                draft.set(name.get_untracked());
                editing.set(true);
            }),
            button("Elsewhere"),
            show(editing, move || {
                let field = node_ref();
                let seen = seen.clone();
                kivact::on_mount(move || field.focus());
                let input = text_input("Name", draft)
                    .node_ref(field)
                    .on_edit(move |phase| {
                        seen.borrow_mut().push(phase);
                        if phase != Begin {
                            editing.set(false);
                        }
                    })
                    .return_focus(opener);
                let input = if fail {
                    input.draft(|_| Err::<(), UiError>(UiError::msg("rejected")))
                } else {
                    input.draft(move |text| name.set(text))
                };
                match invalid_with {
                    Some(mark) => input.invalid(move || draft.get().contains(mark)),
                    None => input,
                }
            }),
            text(move || format!("name: {}", name.get())),
        ));
        (
            Rename {
                name,
                draft,
                editing,
                log,
            },
            view,
        )
    })
}

#[test]
fn enter_commits_the_text_ends_the_draft_and_returns_the_focus() {
    let (screen, page) = rename_page(None, false);
    screen.get_by_role(Role::Button, "Rename").click();
    let field = screen.get_by_role(Role::TextBox, "Name");
    assert!(field.is_focused());
    assert_eq!(field.value().as_deref(), Some("Old"));
    field.type_text(" Name");
    assert_eq!(
        page.name.get_untracked(),
        "Old",
        "the text is a draft until it is committed"
    );
    field.press_key(Key::Enter);
    assert_eq!(page.name.get_untracked(), "Old Name");
    assert!(!page.editing.get_untracked());
    assert_eq!(*page.log.borrow(), vec![Begin, Commit]);
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::Button, "Rename").is_focused(),
        "the focus returns to the control that opened the field"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_writes_the_starting_text_back_and_returns_the_focus() {
    let (screen, page) = rename_page(None, false);
    screen.get_by_role(Role::Button, "Rename").click();
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.type_text("zzz");
    assert_eq!(page.draft.get_untracked(), "Oldzzz");
    field.press_key(Key::Escape);
    assert_eq!(page.name.get_untracked(), "Old");
    assert_eq!(
        page.draft.get_untracked(),
        "Old",
        "a cancel writes the starting text back"
    );
    assert_eq!(*page.log.borrow(), vec![Begin, Cancel]);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "Rename").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn leaving_the_field_commits_a_changed_text_and_cancels_an_unchanged_one() {
    let (screen, page) = rename_page(None, false);
    screen.get_by_role(Role::Button, "Rename").click();
    screen.get_by_role(Role::TextBox, "Name").type_text("!");
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    screen.run_until_idle();
    assert_eq!(page.name.get_untracked(), "Old!");
    assert_eq!(*page.log.borrow(), vec![Begin, Commit]);
    assert!(
        screen.get_by_role(Role::Button, "Elsewhere").is_focused(),
        "leaving does not take the focus back"
    );

    page.log.borrow_mut().clear();
    screen.get_by_role(Role::Button, "Rename").click();
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    screen.run_until_idle();
    assert_eq!(page.name.get_untracked(), "Old!");
    assert_eq!(*page.log.borrow(), vec![Begin, Cancel]);
    assert!(!page.editing.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_invalid_draft_is_not_committed_and_stays_until_escape() {
    let (screen, page) = rename_page(Some('!'), false);
    screen.get_by_role(Role::Button, "Rename").click();
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.type_text("!");
    assert!(field.invalid());
    field.press_key(Key::Enter);
    assert!(page.editing.get_untracked(), "Enter does not end it");
    assert_eq!(page.name.get_untracked(), "Old");
    screen.get_by_role(Role::Button, "Elsewhere").focus();
    screen.run_until_idle();
    assert!(page.editing.get_untracked(), "leaving keeps it pending");
    assert_eq!(*page.log.borrow(), vec![Begin]);
    screen.get_by_role(Role::TextBox, "Name").focus();
    screen
        .get_by_role(Role::TextBox, "Name")
        .press_key(Key::Escape);
    assert!(!page.editing.get_untracked());
    assert_eq!(page.draft.get_untracked(), "Old");
    assert_eq!(*page.log.borrow(), vec![Begin, Cancel]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_failing_commit_is_reported_and_leaves_the_draft_open() {
    let (screen, page) = rename_page(None, true);
    screen.get_by_role(Role::Button, "Rename").click();
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.type_text("x");
    field.press_key(Key::Enter);
    assert!(page.editing.get_untracked());
    assert_eq!(*page.log.borrow(), vec![Begin]);
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn a_field_that_stays_begins_a_new_draft_with_the_next_edit() {
    let (screen, (name, draft, log)) = mount(|| {
        let name = signal("Old".to_string());
        let draft = signal("Old".to_string());
        let log = Log::default();
        let seen = log.clone();
        (
            (name, draft, log),
            text_input("Name", draft)
                .draft(move |text| name.set(text))
                .on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.focus();
    field.type_text("1");
    field.press_key(Key::Enter);
    assert_eq!(name.get_untracked(), "Old1");
    assert_eq!(*log.borrow(), vec![Begin, Commit]);
    field.type_text("2");
    assert_eq!(*log.borrow(), vec![Begin, Commit, Begin]);
    field.press_key(Key::Escape);
    assert_eq!(draft.get_untracked(), "Old1", "back to the committed text");
    assert_eq!(*log.borrow(), vec![Begin, Commit, Begin, Cancel]);
    assert_eq!(name.get_untracked(), "Old1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
