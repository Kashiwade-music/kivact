//! Role: Game UI interaction tests.

use super::*;

#[test]
fn theme_change_preserves_game_choice_and_continuous_range_semantics() {
    let (screen, (theme, selected, amount)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let theme = signal(g::theme(kivact::style::ThemeMode::Light));
            let selected = signal("first");
            let amount = signal(37.5);
            (
                (theme, selected, amount),
                kivact::view::provide_theme(theme, move || {
                    column((
                        kivact::ui::tabs(
                            "Tabs",
                            choice_options(&[("first", "First"), ("second", "Second")]),
                            selected,
                        ),
                        kivact::ui::slider("Amount", amount),
                    ))
                }),
            )
        });
    screen
        .get_by_role(Role::Tab, "First")
        .press_key(Key::ArrowRight);
    let before = screen.snapshot();
    theme.set(g::theme(kivact::style::ThemeMode::Dark));
    screen.run_until_idle();
    assert_eq!(screen.snapshot(), before);
    let slider = screen.get_by_role(Role::Slider, "Amount");
    slider.a11y_with_value(A11yAction::SetValue, "12.25");
    assert_eq!(
        amount.get_untracked(),
        12.25,
        "Game retains continuous values"
    );
    slider.press_key(Key::ArrowRight);
    assert_eq!(amount.get_untracked(), 13.25);
    screen
        .get_by_role(Role::Tab, "Second")
        .press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), "first", "Game explicitly wraps");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn palette_accessibility_failure_keeps_the_popup_for_the_next_keyboard_action() {
    let (screen, (open, chosen, attempts)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let open = signal(false);
            let chosen = signal(false);
            let attempts = signal(Vec::<&'static str>::new());
            (
                (open, chosen, attempts),
                column((
                    kivact::ui::button("Open palette")
                        .variant(kivact_ui_game::Slant)
                        .on_press(move || open.set(true)),
                    kivact::ui::command_palette(
                        open,
                        vec![
                            Command::new("fail", "Fail").hint("FAIL"),
                            Command::new("success", "Success").hint("OK"),
                        ],
                        move |key: &'static str| match key {
                            "fail" => {
                                attempts.update(|items| items.push("fail"));
                                Err(kivact::UiError::msg("command failed"))
                            }
                            _ => {
                                attempts.update(|items| items.push("success"));
                                chosen.set(true);
                                Ok(())
                            }
                        },
                    ),
                )),
            )
        });
    let opener = screen.get_by_role(Role::Button, "Open palette");
    opener.click();
    screen
        .get_by_role(Role::Option, "Fail")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), vec!["fail"]);
    assert_eq!(screen.errors().len(), 1);
    assert!(open.get_untracked());
    let input = screen.get_by_role(Role::TextBox, "Search commands");
    assert!(input.is_focused());
    input.press_key(Key::ArrowDown);
    assert!(input.is_focused());
    assert!(screen.get_by_role(Role::Option, "Success").is_selected());
    screen
        .get_by_role(Role::Option, "Success")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), vec!["fail", "success"]);
    assert!(chosen.get_untracked());
    assert!(!open.get_untracked());
    assert!(opener.is_focused());
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn menu_accessibility_select_invokes_one_action_and_closes_only_after_success() {
    let (screen, attempts) = kivact::test::render_with_handle(crate::options(true), || {
        let attempts = signal(0usize);
        (
            attempts,
            kivact::ui::menu(
                "Actions",
                vec![MenuEntry::item("save", "Save")],
                move |_: &'static str| {
                    attempts.update(|count| *count += 1);
                    if attempts.get_untracked() == 1 {
                        Err(kivact::UiError::msg("save failed"))
                    } else {
                        Ok(())
                    }
                },
            )
            .offset(0.)
            .attach(
                kivact::ui::button("Actions")
                    .variant(kivact_ui_game::Slant)
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .into_node(),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.click();
    screen
        .get_by_role(Role::MenuItem, "Save")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), 1);
    assert_eq!(screen.errors().len(), 1);
    assert!(screen.get_by_role(Role::MenuItem, "Save").exists());
    screen
        .get_by_role(Role::MenuItem, "Save")
        .a11y(A11yAction::Select);
    assert_eq!(attempts.get_untracked(), 2);
    assert_eq!(screen.errors().len(), 1);
    assert!(screen.query_by_role(Role::MenuItem, "Save").is_none());
    assert!(trigger.is_focused());
}

#[test]
fn tree_is_one_tab_stop_with_arrow_keys_and_selects_only_leaves() {
    let (screen, selected) = kivact::test::render_with_handle(crate::options(true), || {
        let selected = signal("b.md".to_owned());
        (
            selected,
            g::tree(
                vec![
                    g::TreeEntry::folder(
                        "docs",
                        "docs",
                        vec![
                            g::TreeEntry::file("a.md", "a.md"),
                            g::TreeEntry::file("b.md", "b.md").marker("M"),
                        ],
                    ),
                    g::TreeEntry::folder(
                        "src",
                        "src",
                        vec![g::TreeEntry::file("lib.rs", "lib.rs")],
                    )
                    .collapsed(),
                ],
                selected,
            )
            .name("Files"),
        )
    });
    let item = |name: &str| screen.get_by_role(Role::TreeItem, name);
    assert!(screen.get_by_role(Role::Tree, "Files").exists());
    assert!(!screen.get_by_role(Role::Tree, "Files").is_multiselectable());
    assert!(item("b.md").is_selected());
    assert_eq!(item("a.md").level(), Some(2));
    assert_eq!(item("src").is_expanded(), Some(false));
    assert!(screen.query_by_role(Role::TreeItem, "lib.rs").is_none());

    screen.press_tab(false);
    assert!(
        item("b.md").is_focused(),
        "the selected row is the tab stop"
    );
    screen.press_key(Key::ArrowUp);
    assert!(item("a.md").is_focused());
    assert_eq!(selected.get_untracked(), "a.md");
    screen.press_key(Key::ArrowLeft);
    assert!(item("docs").is_focused(), "Left goes to the folder");
    assert_eq!(
        selected.get_untracked(),
        "a.md",
        "a folder is not a selected leaf"
    );
    screen.press_key(Key::ArrowLeft);
    assert_eq!(item("docs").is_expanded(), Some(false), "Left closes it");
    assert!(screen.query_by_role(Role::TreeItem, "a.md").is_none());
    screen.press_key(Key::ArrowDown);
    assert!(item("src").is_focused());
    screen.press_key(Key::ArrowRight);
    assert_eq!(item("src").is_expanded(), Some(true));
    screen.press_key(Key::ArrowRight);
    assert!(item("lib.rs").is_focused());
    assert_eq!(selected.get_untracked(), "lib.rs");
    item("src").a11y(A11yAction::Collapse);
    assert!(screen.query_by_role(Role::TreeItem, "lib.rs").is_none());
    assert!(
        item("src").is_focused(),
        "the focus follows the row to the folder that closed over it"
    );
    screen.press_key(Key::Enter);
    assert_eq!(
        item("src").is_expanded(),
        Some(true),
        "Enter opens a folder"
    );
    item("docs").double_click();
    assert_eq!(item("docs").is_expanded(), Some(true));

    let rect = item("src").rect().expect("laid out");
    screen.click_at(rect.x + 15.0, rect.y + rect.height / 2.0);
    assert_eq!(item("src").is_expanded(), Some(false), "the marker toggles");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_editing_tags_numbers_native_picker_and_clipboard() {
    let (screen, (tags, number, files)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let tags = signal(vec!["infra".to_owned()]);
            let number = signal(4.0_f64);
            let files = signal(Vec::new());
            (
                (tags, number, files),
                column((
                    kivact::ui::tag_input(
                        "Tags",
                        tags,
                        move |tag| tags.update(|items| items.push(tag)),
                        move |index| {
                            tags.update(|items| {
                                items.remove(index);
                            })
                        },
                    ),
                    kivact::ui::number_input(
                        "Workers",
                        number,
                        kivact::ui::ScalarRange::new(1.0, 8.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("invariant: the bounds and the step are finite"),
                    ),
                    g::dropzone(kivact::i18n::message("game.dropzone.label", "Upload files"))
                        .on_files(move |paths| files.set(paths))
                        .style(style().width(350)),
                ))
                .style(style().padding(20).gap(20)),
            )
        });
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
        field.a11y_with_value(A11yAction::SetValue, text);
        assert_eq!(number.get_untracked(), expected);
    }
    field.a11y(A11yAction::Increment);
    assert_eq!(number.get_untracked(), 5.0);
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
