//! Role: Tests the wiring of the standard `ui::field`.
//!
//! On real windows: the visible label is the name of the control whether or not it was
//! built with one, the hint describes the control,
//! an error appears and disappears with its message, its relation to the
//! control and the control's invalid state, a control's own invalid value is
//! kept, and the recipes of the field's parts know whether it has an error.

use kivact::prelude::*;
use kivact::ui;
use kivact::view::Node;
use kivact_public_api_tests::support::mount;

/// A control that is built from `node()` and has no name of its own.
fn unnamed_input() -> Node {
    node().role(Role::TextBox).focusable(true)
}

fn texts(elements: Vec<kivact::test::Locator<'_>>) -> Vec<String> {
    elements.iter().map(|e| e.text()).collect()
}

#[test]
fn the_field_label_names_the_control_even_when_it_has_its_own_name() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                ui::field("Nickname", unnamed_input()),
                ui::field(
                    "Email address",
                    ui::text_input(signal(String::new())).label("Email"),
                ),
            )),
        )
    });
    screen.get_by_text("Nickname");
    screen.get_by_text("Email address");
    let nickname = screen.get_by_role(Role::TextBox, "Nickname");
    assert_eq!(texts(nickname.labelled_by()), ["Nickname"]);
    let email = screen.get_by_role(Role::TextBox, "Email address");
    assert_eq!(texts(email.labelled_by()), ["Email address"]);
    assert!(
        screen.query_by_role(Role::TextBox, "Email").is_none(),
        "the name it was built with is replaced"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn inline_edit_opens_focused_commits_cancels_and_ends_with_its_row() {
    let (screen, (names, editing, committed)) = mount(|| {
        let names = signal(vec![(1u32, "alpha".to_owned()), (2, "beta".to_owned())]);
        let editing = signal(None::<u32>);
        let committed = signal(Vec::<String>::new());
        let rename = kivact::interaction::use_inline_edit(editing);
        let rows = for_each(
            names,
            |(id, _)| *id,
            move |name: Signal<(u32, String)>| {
                let id = name.with(|(id, _)| *id);
                column((
                    ui::button(format!("Rename {id}")).on_press(move || rename.edit(id)),
                    show(
                        move || rename.editing().get() == Some(id),
                        move || {
                            node().part(
                                rename
                                    .field(id, format!("Name {id}"), signal(name.get_untracked().1))
                                    .on_draft_commit(move |text| {
                                        committed.update(|all| all.push(format!("{id}:{text}")))
                                    })
                                    .control(),
                            )
                        },
                    ),
                ))
            },
        );
        (
            (names, editing, committed),
            column((
                column(rows),
                ui::button("Cancel rename").on_press(move || rename.cancel()),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Rename 1").click();
    let field = screen.get_by_role(Role::TextBox, "Name 1");
    assert!(field.is_focused());
    assert_eq!(field.text_selection(), Some((0, 5)), "the name is selected");
    field.type_text("gamma");
    field.press_key(Key::Enter);
    assert_eq!(committed.get_untracked(), ["1:gamma"]);
    assert!(screen.query_by_role(Role::TextBox, "Name 1").is_none());
    assert_eq!(editing.get_untracked(), None);
    screen.get_by_role(Role::Button, "Rename 1").click();
    screen
        .get_by_role(Role::TextBox, "Name 1")
        .press_key(Key::Escape);
    assert_eq!(
        committed.get_untracked(),
        ["1:gamma"],
        "Escape commits nothing"
    );
    assert!(screen.query_by_role(Role::TextBox, "Name 1").is_none());
    screen.get_by_role(Role::Button, "Rename 2").click();
    assert_eq!(editing.get_untracked(), Some(2));
    screen.get_by_role(Role::Button, "Cancel rename").click();
    assert_eq!(editing.get_untracked(), None, "cancel ends the rename");
    assert!(screen.query_by_role(Role::TextBox, "Name 2").is_none());
    screen.get_by_role(Role::Button, "Rename 2").click();
    assert_eq!(editing.get_untracked(), Some(2));
    names.set(vec![(1, "alpha".to_owned())]);
    screen.run_until_idle();
    assert_eq!(
        editing.get_untracked(),
        None,
        "removing the row ends the rename"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_error_appears_and_disappears_with_its_relation_and_the_invalid_state() {
    let (screen, (email, error)) = mount(|| {
        let email = signal(String::new());
        let error = memo(move || {
            let email = email.get();
            (!email.is_empty() && !email.contains('@')).then(|| "Enter an email address".to_owned())
        });
        (
            (email, error),
            ui::field("Email", ui::text_input(email).label("Email"))
                .hint("We never share it")
                .error(error),
        )
    });
    let input = || screen.get_by_role(Role::TextBox, "Email");
    assert!(error.get_untracked().is_none());
    assert!(!input().is_invalid());
    assert!(
        screen
            .query_by_role(Role::Alert, "Enter an email address")
            .is_none()
    );
    assert_eq!(texts(input().described_by()), ["We never share it"]);

    input().type_text("nobody");
    assert!(error.get_untracked().is_some());
    assert!(input().is_invalid());
    screen.get_by_role(Role::Alert, "Enter an email address");
    assert_eq!(
        texts(input().described_by()),
        ["We never share it", "Enter an email address"],
        "the hint, then the error"
    );

    email.set("nobody@example.com".to_owned());
    screen.run_until_idle();
    assert!(!input().is_invalid());
    assert!(
        screen
            .query_by_role(Role::Alert, "Enter an email address")
            .is_none()
    );
    assert_eq!(texts(input().described_by()), ["We never share it"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_text_input_without_a_name_reports_an_error_and_a_field_names_it() {
    let (screen, ()) = mount(|| {
        let a = signal(String::new());
        let b = signal(String::new());
        (
            (),
            column((
                ui::text_input(a).test_id("bare"),
                ui::field("Email", ui::text_input(b).test_id("named")),
            )),
        )
    });
    assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
    assert!(format!("{:?}", screen.errors()[0]).contains("has no accessible name"));
    // The message names the call site: the test's own file.
    assert!(format!("{:?}", screen.errors()[0]).contains("ui_field.rs"));
    let named = screen.get_by_test_id("named");
    assert_eq!(named.name(), "Email");
}

/// A custom control without a name of its own, built as a component.
#[component]
fn custom_input() -> impl Element {
    node().role(Role::TextBox).focusable(true)
}

#[test]
fn field_takes_a_standard_or_custom_control_without_annotations() {
    let (screen, ()) = mount(|| {
        let name = signal(String::new());
        (
            (),
            column((
                ui::field("Name", ui::text_input(name)),
                ui::field("Code", custom_input()),
            )),
        )
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    assert_eq!(screen.get_by_role(Role::TextBox, "Name").name(), "Name");
    assert_eq!(screen.get_by_role(Role::TextBox, "Code").name(), "Code");
}
