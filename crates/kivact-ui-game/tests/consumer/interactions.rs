//! Role: Game UI interaction tests.

use kivact::prelude::*;
use kivact::view::A11yAction;
use kivact_ui_game as g;

#[test]
fn tree_is_one_tab_stop_with_arrow_keys_and_selects_only_leaves() {
    let (screen, selected) =
        kivact::test::render_with_handle(crate::support::options(true), || {
            let selected = signal(Some("b.md".to_owned()));
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
    assert_eq!(selected.get_untracked(), Some("a.md".to_owned()));
    screen.press_key(Key::ArrowLeft);
    assert!(item("docs").is_focused(), "Left goes to the folder");
    assert_eq!(
        selected.get_untracked(),
        Some("a.md".to_owned()),
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
    assert_eq!(selected.get_untracked(), Some("lib.rs".to_owned()));
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
fn dropzone_failure_clears_on_the_next_drop() {
    let (screen, ()) = kivact::test::render_with_handle(crate::support::options(true), || {
        (
            (),
            g::dropzone("Upload files")
                .on_files(|paths| {
                    if paths.iter().any(|path| path.ends_with("bad.csv")) {
                        Err(kivact::UiError::msg("bad.csv is refused"))
                    } else {
                        Ok(())
                    }
                })
                .style(style().width(350)),
        )
    });
    let rect = screen
        .get_by_role(Role::Button, "Upload files")
        .rect()
        .expect("laid out");
    let center = kivact::draw::Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    for (file, refused) in [("bad.csv", true), ("good.csv", false)] {
        screen.file_drag(kivact::view::FilePhase::Enter, vec![file.into()], center);
        screen.file_drag(kivact::view::FilePhase::Drop, vec![file.into()], center);
        assert_eq!(
            screen
                .query_by_role(Role::Alert, "bad.csv is refused")
                .is_some(),
            refused
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pagination_moves_focus_only_when_the_page_changes() {
    let (screen, page) = kivact::test::render_with_handle(crate::support::options(true), || {
        let page = signal(0usize);
        (
            page,
            column((
                g::pagination(
                    binding(page, move |next: usize| {
                        if next != 2 {
                            page.set(next);
                        }
                    }),
                    12,
                ),
                kivact::ui::button("Elsewhere"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "2").click();
    assert_eq!(page.get_untracked(), 1);
    assert!(
        screen.get_by_role(Role::Button, "2").is_focused(),
        "a page change keeps the focus on the current page"
    );
    screen.get_by_role(Role::Button, "3").click();
    assert_eq!(page.get_untracked(), 1, "the binding refused page 3");
    let elsewhere = screen.get_by_role(Role::Button, "Elsewhere");
    elsewhere.focus();
    page.set(4);
    screen.run_until_idle();
    assert!(
        elsewhere.is_focused(),
        "a refused press does not arm a later focus move"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consumer_editing_tags_numbers_native_picker_and_clipboard() {
    let (screen, (tags, number, files)) =
        kivact::test::render_with_handle(crate::support::options(true), || {
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
        field.a11y_set_value(text);
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
