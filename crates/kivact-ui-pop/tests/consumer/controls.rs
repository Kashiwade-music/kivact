//! Role: Buttons, checks, switches and the standard inputs through the public API.

use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact_ui_pop as ui;

#[test]
fn button_states_and_user_modifiers_use_public_contract() {
    let (screen, count) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let count = signal(0);
            (
                count,
                column((
                    kivact::ui::button("Save")
                        .on_press(move || count.update(|v| *v += 1))
                        .style(style().height(40))
                        .test_id("save"),
                    kivact::ui::button("Disabled")
                        .disabled(true)
                        .on_press(move || count.update(|v| *v += 100)),
                )),
            )
        },
    );
    let button = screen.get_by_role(Role::Button, "Save");
    button.click();
    button.press_key(Key::Enter);
    assert_eq!(count.get_untracked(), 2);
    assert_eq!(button.rect().map(|r| r.height), Some(40.0));
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn checkbox_switch_and_mixed_states_are_accessible() {
    let (screen, (checked, on)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let checked = signal(false);
            let on = signal(true);
            (
                (checked, on),
                column((
                    kivact::ui::checkbox("Reports", checked),
                    kivact::ui::checkbox("Partial", kivact::state::signal(false))
                        .indeterminate(true),
                    kivact::ui::switch("Sync", on),
                )),
            )
        },
    );
    let check = screen.get_by_role(Role::CheckBox, "Reports");
    check.press_key(Key::Space);
    assert!(checked.get_untracked());
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "Partial").checked(),
        Some(CheckedState::Indeterminate)
    );
    screen.get_by_role(Role::Switch, "Sync").click();
    assert!(!on.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn inputs_and_tag_editor_commit_native_edits() {
    let (screen, (draft, tags, memo)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_pop::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let draft = signal(String::new());
            let tags = signal(vec!["one".into()]);
            let memo = signal(String::new());
            ((draft, tags, memo), input_controls(draft, tags, memo))
        },
    );
    screen
        .get_by_role(Role::TextBox, "Name")
        .type_text("Minato");
    assert_eq!(draft.get_untracked(), "Minato");
    let note = screen.get_by_role(Role::TextBox, "Resizable note");
    note.type_text("Keep this draft");
    let initial = note.rect().expect("note laid out");
    assert_eq!(initial.height, 80.0);
    let resize = |delta: f64| {
        let grip = screen.get_by_role(Role::Splitter, "Resize text area");
        grip.hover();
        let rect = grip.rect().expect("resize grip laid out");
        let x = rect.x + rect.width / 2.0;
        let y = rect.y + rect.height / 2.0;
        screen.mouse_move(x, y);
        screen.mouse_down(kivact::view::MouseButton::Primary);
        screen.mouse_move(x, y + delta / 2.0);
        screen.mouse_move(x, y + delta);
        screen.mouse_up(kivact::view::MouseButton::Primary);
    };
    resize(48.0);
    assert_eq!(note.rect().unwrap().height, 128.0);
    assert_eq!(note.rect().unwrap().width, initial.width);
    assert_eq!(memo.get_untracked(), "Keep this draft");
    assert!(note.is_focused());
    screen.type_text(" after resizing");
    assert_eq!(memo.get_untracked(), "Keep this draft after resizing");
    resize(-200.0);
    assert_eq!(note.rect().unwrap().height, 72.0);
    assert_eq!(note.rect().unwrap().width, initial.width);
    assert!(note.is_focused());
    screen.type_text(".");
    assert_eq!(memo.get_untracked(), "Keep this draft after resizing.");
    let grip = screen.get_by_role(Role::Splitter, "Resize text area");
    grip.press_key(Key::ArrowDown);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.press_key(Key::ArrowDown);
    assert_eq!(note.rect().unwrap().height, 88.0);
    grip.press_key(Key::ArrowUp);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.press_key(Key::Home);
    assert_eq!(note.rect().unwrap().height, 72.0);
    grip.a11y(A11yAction::Increment);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.a11y_set_value("-100");
    assert_eq!(note.rect().unwrap().height, 72.0);
    grip.a11y_set_value("NaN");
    assert_eq!(note.rect().unwrap().height, 72.0);
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Fixed note")
            .rect()
            .unwrap()
            .height,
        80.0
    );
    let tag = screen.get_by_role(Role::TextBox, "Add tag");
    tag.type_text("two");
    tag.press_key(Key::Enter);
    assert_eq!(tags.get_untracked(), vec!["one", "two"]);
    tag.press_key(Key::Backspace);
    assert_eq!(tags.get_untracked(), vec!["one"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn input_controls(
    draft: Signal<String>,
    tags: Signal<Vec<String>>,
    memo: Signal<String>,
) -> impl IntoView {
    column((
        kivact::ui::text_input(draft)
            .label("Name")
            .placeholder("Enter name"),
        ui::resizable_text_area(
            kivact::ui::text_area(memo).label("Resizable note"),
            80.0,
            72.0,
        ),
        kivact::ui::text_area(kivact::state::signal(String::new())).label("Fixed note"),
        kivact::ui::tag_input(tags).label("Add tag"),
    ))
    .style(style().width(360).gap(8))
}
