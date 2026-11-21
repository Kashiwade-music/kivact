//! Role: Buttons, mixed checks and the editing controls through the public API.

use kivact::prelude::*;
use kivact::view::A11yAction;

#[test]
fn consumer_buttons_checks_mixed_states_and_modifiers() {
    let (screen, (count, checked, mixed)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let count = signal(0);
            let checked = signal(false);
            let mixed = signal(true);
            (
                (count, checked, mixed),
                column((
                    kivact::ui::button("Approve")
                        .variant(kivact::ui::Slant)
                        .test_id("approve")
                        .style(style().height(46))
                        .on_press(move || count.update(|v| *v += 1)),
                    kivact::ui::button("Disabled")
                        .variant(kivact::ui::Slant)
                        .disabled(true),
                    kivact::ui::checkbox("Permissions", checked).indeterminate(mixed),
                    kivact::ui::switch("Network", checked),
                ))
                .style(style().gap(20).padding(20)),
            )
        },
    );
    let button = screen.get_by_test_id("approve");
    assert_eq!(button.accessibility_bounds().map(|r| r.height), Some(46.));
    button.click();
    button.press_key(Key::Space);
    assert_eq!(count.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    let check = screen.get_by_role(Role::CheckBox, "Permissions");
    assert_eq!(check.checked(), Some(CheckedState::Indeterminate));
    check.a11y(A11yAction::Toggle);
    assert!(checked.get_untracked());
    mixed.set(false);
    screen.run_until_idle();
    assert_ne!(check.checked(), Some(CheckedState::Indeterminate));
    screen
        .get_by_role(Role::Switch, "Network")
        .press_key(Key::Space);
    assert!(!checked.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_editing_tags_numbers_native_picker_and_clipboard() {
    let (screen, (tags, number, files)) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let tags = signal(vec!["infra".to_owned()]);
            let number = signal(4.0_f64);
            let files = signal(Vec::new());
            ((tags, number, files), editing_controls(tags, number, files))
        },
    );
    let edit = screen.get_by_role(Role::TextBox, "Tags");
    edit.type_text("ops");
    edit.press_key(Key::Enter);
    assert_eq!(tags.get_untracked(), vec!["infra", "ops"]);
    edit.press_key(Key::Backspace);
    assert_eq!(tags.get_untracked(), vec!["infra"]);
    edit.type_text("infra");
    edit.press_key(Key::Enter);
    assert_eq!(tags.get_untracked(), vec!["infra", "infra"]);
    screen.all_by_role(Role::Button, "Remove infra")[0].click();
    assert_eq!(tags.get_untracked(), vec!["infra"]);
    screen.get_by_role(Role::Button, "Remove infra").click();
    assert!(tags.get_untracked().is_empty());
    let field = screen.get_by_role(Role::SpinButton, "Workers");
    field.focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(field.text_selection(), Some((0, 0)));
    assert_eq!(number.get_untracked(), 4.0);
    screen.press_key(Key::ArrowRight);
    assert_eq!(field.text_selection(), Some((1, 1)));
    for (key, expected) in [
        (Key::Home, 1.0),
        (Key::End, 8.0),
        (Key::PageDown, 1.0),
        (Key::PageUp, 8.0),
        (Key::ArrowDown, 7.0),
        (Key::ArrowUp, 8.0),
    ] {
        screen.press_key(key);
        assert_eq!(number.get_untracked(), expected);
        assert!(field.is_focused());
    }
    for key in [Key::Home, Key::Home, Key::End, Key::End] {
        screen.press_key_with(
            key,
            kivact::view::Modifiers {
                shift: true,
                ..kivact::view::Modifiers::default()
            },
        );
        assert_eq!(number.get_untracked(), 8.0);
    }
    for (text, expected) in [
        ("5.0", 5.0),
        ("5.5", 6.0),
        ("NaN", 6.0),
        ("inf", 6.0),
        ("9999999999", 8.0),
        ("-9999999999", 1.0),
        ("4", 4.0),
    ] {
        field.a11y_set_value(text);
        assert_eq!(number.get_untracked(), expected);
    }
    field.a11y(A11yAction::Increment);
    assert_eq!(number.get_untracked(), 5.0);
    assert_native_file_selection(&screen, files);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn editing_controls(
    tags: Signal<Vec<String>>,
    number: Signal<f64>,
    files: Signal<Vec<std::path::PathBuf>>,
) -> impl IntoView {
    column((
        kivact::ui::tag_input(tags).label("Tags"),
        kivact::ui::number_input(
            number,
            kivact::interaction::ScalarRange::new(1.0, 8.0)
                .and_then(|range| range.with_step(1.0))
                .expect("invariant: the bounds and the step are finite"),
        )
        .label("Workers"),
        kivact::ui::dropzone(kivact::i18n::message("game.dropzone.label", "Upload files"))
            .multiple(true)
            .pick_on_press(true)
            .icon(kivact::view::icon("cloud-upload"))
            .on_files(move |paths| files.set(paths.into_iter().map(|file| file.path).collect()))
            .style(style().width(350)),
    ))
    .style(style().padding(20).gap(20))
}

fn assert_native_file_selection(
    screen: &kivact::test::Screen,
    files: Signal<Vec<std::path::PathBuf>>,
) {
    screen.respond_files_dialog(Some(vec![
        std::path::PathBuf::from("inventory.csv"),
        std::path::PathBuf::from("records.json"),
    ]));
    screen.get_by_role(Role::Button, "Upload files").click();
    screen.run_until_idle();
    assert_eq!(
        files.get_untracked(),
        vec![
            std::path::PathBuf::from("inventory.csv"),
            std::path::PathBuf::from("records.json")
        ]
    );
}
