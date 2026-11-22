//! Role: Form tests: the questionnaire, the one-time code input and the native select.

use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::BACKGROUND)
            .padding(16)
            .gap(12)
            .flex_grow(1)
            .align_items(Align::Start),
    )
}

#[test]
fn questionnaire_submits_once_and_records_skips() {
    let (screen, (answers, submitted)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
            let step = signal(0usize);
            let answers = signal(Vec::<basic::QuestionnaireAnswer>::new());
            let submitted = signal(0usize);
            let steps = vec![
                basic::QuestionnaireStep::input("Name?").optional(true),
                basic::QuestionnaireStep::choices(
                    "Color?",
                    vec![
                        basic::QuestionnaireChoice::new("red", "Red"),
                        basic::QuestionnaireChoice::new("blue", "Blue"),
                    ],
                ),
            ];
            (
                (answers, submitted),
                page(basic::questionnaire(steps, step, answers).on_submit(
                    move |_: Vec<basic::QuestionnaireAnswer>| submitted.update(|count| *count += 1),
                )),
            )
        },
    );
    screen.get_by_role(Role::Button, "Skip").click();
    assert_eq!(
        answers.get_untracked(),
        vec![basic::QuestionnaireAnswer::Skipped]
    );
    screen.get_by_text("Red").click();
    assert_eq!(
        answers.get_untracked()[1],
        basic::QuestionnaireAnswer::Answered("red".to_owned())
    );
    let submit = screen.get_by_role(Role::Button, "Submit");
    submit.click();
    assert_eq!(submitted.get_untracked(), 1);
    assert!(submit.is_disabled(), "a submitted sheet is not sent again");
    screen.get_by_text("Blue").click();
    assert!(!submit.is_disabled(), "a changed answer asks again");
    submit.click();
    assert_eq!(submitted.get_untracked(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

#[test]
fn input_otp_fills_slots_completes_and_backspaces() {
    let (screen, (code, done)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
            let code = signal(String::new());
            let done = signal(None::<String>);
            (
                (code, done),
                page(
                    basic::input_otp(code, 6usize)
                        .name("Code")
                        .groups([3, 3])
                        .on_complete(move |c: String| done.set(Some(c))),
                ),
            )
        },
    );
    screen.get_by_role(Role::TextBox, "Code").click();
    screen.type_text("12a3");
    screen.run_until_idle();
    assert_eq!(code.get_untracked(), "123", "letters are not digits");
    screen.get_by_text("1");
    screen.get_by_text("3");
    assert!(done.get_untracked().is_none());
    screen.type_text("456");
    screen.run_until_idle();
    assert_eq!(code.get_untracked(), "123456");
    assert_eq!(done.get_untracked().as_deref(), Some("123456"));
    screen.press_key(Key::Backspace);
    screen.run_until_idle();
    assert_eq!(code.get_untracked(), "12345");
    screen.assert_layout();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}

fn fruits() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("pear", "Pear").disabled(true),
        ListItem::new("fig", "Fig"),
    ]
}

#[test]
fn native_select_chooses_by_keyboard_and_list() {
    let (screen, value) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .reduced_motion(),
        || {
            let value = signal(None::<&'static str>);
            (
                value,
                page(
                    basic::native_select(fruits(), value)
                        .placeholder("Select fruit")
                        .name("Fruit"),
                ),
            )
        },
    );
    screen.get_by_text("Select fruit");
    let trigger = screen.get_by_role(Role::ComboBox, "Fruit");
    trigger.focus();
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("apple"));
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.run_until_idle();
    assert_eq!(
        value.get_untracked(),
        Some("fig"),
        "the disabled option is skipped"
    );
    trigger.click();
    screen.run_until_idle();
    assert!(trigger.is_expanded() == Some(true));
    screen.get_by_role(Role::Option, "Banana").click();
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), Some("banana"));
    screen.assert_layout();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
