//! Role: Tests the wiring of the standard `ui::field`.
//!
//! On real windows: the visible label is the name of the control whether or not it was
//! built with one, the hint describes the control,
//! an error appears and disappears with its message, its relation to the
//! control and the control's invalid state, a control's own invalid value is
//! kept, and the recipes of the field's parts know whether it has an error.

use crate::support::{mount, mount_with};
use kivact::prelude::*;
use kivact::style::{Align, Style, Theme};
use kivact::test::RenderOptions;
use kivact::ui::{self, FieldState, Look, UiRecipes};

/// A control that is built from `node()` and has no name of its own.
fn unnamed_input() -> Node {
    node().role(Role::TextBox).focusable(true)
}

fn texts(elements: Vec<kivact::test::Element<'_>>) -> Vec<String> {
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
                    ui::text_input("Email", signal(String::new())),
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
fn the_hint_describes_the_control() {
    let (screen, ()) = mount(|| {
        (
            (),
            ui::field(
                "Password",
                ui::text_input("Password", signal(String::new())),
            )
            .hint("At least 8 characters"),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Password");
    assert_eq!(texts(input.described_by()), ["At least 8 characters"]);
    assert!(!input.invalid());
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
            ui::field("Email", ui::text_input("Email", email))
                .hint("We never share it")
                .error(error),
        )
    });
    let input = || screen.get_by_role(Role::TextBox, "Email");
    assert!(error.get_untracked().is_none());
    assert!(!input().invalid());
    assert!(
        screen
            .query_by_role(Role::Alert, "Enter an email address")
            .is_none()
    );
    assert_eq!(texts(input().described_by()), ["We never share it"]);

    input().type_text("nobody");
    assert!(error.get_untracked().is_some());
    assert!(input().invalid());
    screen.get_by_role(Role::Alert, "Enter an email address");
    assert_eq!(
        texts(input().described_by()),
        ["We never share it", "Enter an email address"],
        "the hint, then the error"
    );

    email.set("nobody@example.com".to_owned());
    screen.run_until_idle();
    assert!(!input().invalid());
    assert!(
        screen
            .query_by_role(Role::Alert, "Enter an email address")
            .is_none()
    );
    assert_eq!(texts(input().described_by()), ["We never share it"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_error_works_on_any_control_and_keeps_the_controls_own_invalid_value() {
    let (screen, (error, own)) = mount(|| {
        let error = signal(None::<String>);
        let own = signal(false);
        (
            (error, own),
            column((
                ui::field("Terms", ui::checkbox("I agree", signal(false))).error(error),
                ui::field(
                    "Code",
                    ui::text_input("Code", signal(String::new())).invalid(own),
                )
                .error(None::<String>),
            )),
        )
    });
    let terms = || screen.get_by_role(Role::CheckBox, "Terms");
    let code = || screen.get_by_role(Role::TextBox, "Code");
    assert!(!terms().invalid());
    error.set(Some("Required".to_owned()));
    screen.run_until_idle();
    assert!(terms().invalid());
    assert_eq!(texts(terms().described_by()), ["Required"]);
    error.set(None);
    screen.run_until_idle();
    assert!(!terms().invalid());
    assert!(terms().described_by().is_empty());

    assert!(!code().invalid());
    own.set(true);
    screen.run_until_idle();
    assert!(code().invalid(), "the control's own value counts");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_recipes_know_whether_the_field_has_an_error_and_the_control_draws_it() {
    fn label(_: &Theme, state: &Look<FieldState>) -> Style {
        style().size(if state.invalid { 120 } else { 80 }, 20)
    }
    fn input(_: &Theme, state: &Look<kivact::ui::TextInputState>) -> Style {
        style().size(if state.invalid { 250 } else { 150 }, 30)
    }
    let (screen, error) = mount_with(
        RenderOptions {
            theme: Theme::light().with(
                UiRecipes::default()
                    .text_input(|_| input)
                    .field(|field| field.label(|_| label)),
            ),
            ..RenderOptions::from_env()
        },
        || {
            let error = signal(None::<String>);
            let view = ui::field("Name", ui::text_input("Name", signal(String::new())))
                .error(error)
                .style(style().align_items(Align::Start));
            (error, view)
        },
    );
    let width = |e: kivact::test::Element<'_>| e.rect().expect("laid out").width;
    assert_eq!(width(screen.get_by_text("Name")), 80.0);
    assert_eq!(width(screen.get_by_role(Role::TextBox, "Name")), 150.0);
    error.set(Some("Too short".to_owned()));
    screen.run_until_idle();
    assert_eq!(width(screen.get_by_text("Name")), 120.0);
    assert_eq!(width(screen.get_by_role(Role::TextBox, "Name")), 250.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
