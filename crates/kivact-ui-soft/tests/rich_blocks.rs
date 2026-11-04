//! Role: Verify block editing preserves source and native caret operations.
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{MirrorMode, RenderOptions, render_with_handle};
use kivact::view::{Key, Role};
use kivact_ui_soft::rich_text_editor;

fn options() -> RenderOptions {
    RenderOptions::default()
        .theme(kivact_ui_soft::theme(ThemeMode::Light))
        .mirror(MirrorMode::Off)
}

#[test]
fn block_editor_edits_heading_and_task_without_rewriting_neighboring_markdown() {
    let original =
        "## Title  \r\n\r\n> Quoted text\r\n- [X] Task\r\n\r\n| A | B |\r\n|---|---|\r\n";
    let (screen, source) = render_with_handle(options(), move || {
        let source = signal(original.to_owned());
        (
            source,
            rich_text_editor("Article", source).block_editing(true),
        )
    });
    assert_eq!(source.get_untracked(), original);
    let title = screen.get_by_role(Role::TextBox, "Article, block 1");
    assert_eq!(title.value().as_deref(), Some("Title"));
    title.focus();
    screen.press_key(Key::End);
    screen.type_text(" edited");
    screen.get_by_role(Role::CheckBox, "Task completed").click();
    let edited = original
        .replace("Title", "Title edited")
        .replace("[X]", "[ ]");
    assert_eq!(source.get_untracked(), edited);
    screen.get_by_role(Role::Button, "Preview").click();
    assert!(
        screen
            .query_by_role(Role::Heading, "Title edited")
            .is_some()
    );
    screen.get_by_role(Role::Button, "Write").click();
    assert!(screen.query_by_role(Role::TextBox, "Article").is_some());
    screen.get_by_role(Role::Button, "Blocks").click();
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Article, block 1")
            .value()
            .as_deref(),
        Some("Title edited")
    );
    assert_eq!(source.get_untracked(), edited);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn enter_splits_and_backspace_at_block_start_rejoins_without_losing_caret() {
    let (screen, source) = render_with_handle(options(), || {
        let source = signal("Alpha Beta\r\n\r\n# untouched\r\n".to_owned());
        (
            source,
            rich_text_editor("Article", source).block_editing(true),
        )
    });
    screen
        .get_by_role(Role::TextBox, "Article, block 1")
        .focus();
    screen.press_key(Key::Home);
    for _ in 0..5 {
        screen.press_key(Key::ArrowRight);
    }
    screen.press_key(Key::Enter);
    assert_eq!(
        source.get_untracked(),
        "Alpha\r\n\r\n Beta\r\n\r\n# untouched\r\n"
    );
    assert!(
        screen
            .get_by_role(Role::TextBox, "Article, block 2")
            .is_focused()
    );
    screen.press_key(Key::Backspace);
    assert_eq!(source.get_untracked(), "Alpha Beta\r\n\r\n# untouched\r\n");
    assert!(
        screen
            .get_by_role(Role::TextBox, "Article, block 1")
            .is_focused()
    );
    screen.type_text("!");
    assert_eq!(source.get_untracked(), "Alpha! Beta\r\n\r\n# untouched\r\n");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
