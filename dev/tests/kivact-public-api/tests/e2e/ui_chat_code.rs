//! Role: Verify shared chat scrolling, native composition and selectable code contracts.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{
    ChatMessageStatus, ChatSender, chat_composer, chat_message, chat_thread, code_block,
};
use std::time::Duration;
#[test]
fn chat_thread_is_a_log_and_follows_the_end() {
    let (screen, (items, _revision, offset)) = mount(|| {
        let items = signal((0..20).collect::<Vec<i32>>());
        let revision = signal(1_u64);
        let viewport = node_ref();
        let offset = viewport.scroll_offset();
        let messages = for_each(
            items,
            |id| *id,
            |id| text(move || format!("Message {}", id.get())).style(style().height(24).shrink(0)),
        );
        (
            (items, revision, offset),
            column((
                chat_thread(messages)
                    .name("Conversation")
                    .viewport_ref(viewport)
                    .history_revision(revision)
                    .gap(0.0)
                    .style(style().size(340, 144).shrink(0)),
                kivact::ui::button("Load history").on_press(move || {
                    items.update(|v| {
                        v.splice(..0, [-2, -1]);
                    });
                    revision.set(2);
                }),
            )),
        )
    });
    screen.get_by_role(Role::Log, "Conversation");
    let initial = offset.get_untracked().y;
    assert!(initial > 100.0);
    items.update(|v| v.push(20));
    screen.advance(Duration::ZERO);
    assert!(offset.get_untracked().y > initial);
    screen
        .get_by_role(Role::Log, "Conversation")
        .scroll(0.0, -180.0);
    let detached = offset.get_untracked().y;
    items.update(|v| v.push(21));
    screen.advance(Duration::ZERO);
    assert!((offset.get_untracked().y - detached).abs() < 1.0);
    let anchor = screen.get_by_text("Message 9");
    let before = anchor.rect().expect("message laid out").y;
    screen.get_by_role(Role::Button, "Load history").click();
    assert!((anchor.rect().expect("message laid out").y - before).abs() < 1.0);
    screen.get_by_role(Role::Button, "Jump to latest").click();
    let latest = offset.get_untracked().y;
    assert!(latest > detached);
    items.update(|v| v.push(22));
    screen.advance(Duration::ZERO);
    assert!(offset.get_untracked().y > latest);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn code_block_copies_and_numbers_lines() {
    let (screen, (source, numbered, expanded)) = mount(|| {
        let source = signal("fn main() {\n    println!(\"hello\");\n}".to_owned());
        let numbered = signal(true);
        let expanded = signal(true);
        (
            (source, numbered, expanded),
            code_block(source)
                .title("main.rs")
                .language("Rust")
                .line_numbers(numbered)
                .highlight((), |_, line, state: &bool| {
                    (
                        vec![kivact::style::StyledRange {
                            range: 0..line.len(),
                            color: Some(Color::token(ColorToken::Accent)),
                            background: None,
                            decoration: None,
                        }],
                        !*state,
                    )
                })
                .highlight_lines(vec![2])
                .diff(true)
                .wrap(true)
                .collapsible(true)
                .collapsible_threshold(2usize)
                .expanded(expanded)
                .max_height(160)
                .footer(text("Example"))
                .style(style().width(360)),
        )
    });
    screen.get_by_text("1\n2\n3");
    let editor = screen.get_by_role(Role::TextBox, "main.rs");
    assert_eq!(editor.value(), Some(source.get_untracked()));
    editor.type_text("changed");
    assert_eq!(editor.value(), Some(source.get_untracked()));
    screen.get_by_role(Role::Button, "Copy").click();
    assert_eq!(screen.clipboard(), Some(source.get_untracked()));
    screen.get_by_role(Role::Button, "Copied");
    screen.advance(Duration::from_millis(2100));
    screen.get_by_role(Role::Button, "Copy");
    source.set("alpha\nbeta".into());
    screen.advance(Duration::ZERO);
    screen.get_by_text("1\n2");
    screen.get_by_role(Role::Button, "Copy").click();
    assert_eq!(screen.clipboard().as_deref(), Some("alpha\nbeta"));
    numbered.set(false);
    screen.advance(Duration::ZERO);
    assert!(screen.query_by_text("1\n2").is_none());
    expanded.set(false);
    screen.advance(Duration::ZERO);
    assert!(screen.query_by_role(Role::TextBox, "main.rs").is_none());
    screen
        .get_by_role(Role::Button, "main.rs")
        .press_key(Key::Space);
    assert!(expanded.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
#[test]
fn chat_composer_preserves_newlines_history_stop_and_ime() {
    let (screen, (draft, sent, stopped, stops)) = mount(|| {
        let draft = signal(String::new());
        let sent = signal(Vec::<String>::new());
        let stopped = signal(false);
        let stops = signal(0);
        (
            (draft, sent, stopped, stops),
            column((
                chat_message("Ada")
                    .sender(ChatSender::Assistant)
                    .content(text("Hello"))
                    .time("10:42")
                    .status(ChatMessageStatus::Read),
                chat_composer(draft)
                    .on_submit(move |s| sent.update(|v| v.push(s)))
                    .stop_shown(stopped)
                    .on_stop(move || stops.update(|n| *n += 1))
                    .header_context(text("Context"))
                    .drawer(text("Attachments")),
            )),
        )
    });
    screen.get_by_text("Ada");
    screen.get_by_text("Read");
    let input = screen.get_by_role(Role::TextBox, "Message input");
    input.type_text("First");
    input.press_key_with(
        Key::Enter,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    input.type_text("Second");
    assert!(sent.get_untracked().is_empty());
    input.press_key(Key::Enter);
    assert_eq!(sent.get_untracked(), vec!["First\nSecond"]);
    assert!(draft.get_untracked().is_empty());
    input.press_key(Key::ArrowUp);
    assert_eq!(input.value().as_deref(), Some("First\nSecond"));
    input.press_key(Key::ArrowDown);
    assert_eq!(input.value().as_deref(), Some(""));
    input.compose("draft");
    input.press_key(Key::Enter);
    assert_eq!(sent.get_untracked().len(), 1);
    input.commit("draft");
    input.press_key(Key::Enter);
    assert_eq!(sent.get_untracked().len(), 2);
    stopped.set(true);
    screen.advance(Duration::ZERO);
    input.type_text("waiting");
    input.press_key(Key::Enter);
    assert_eq!(sent.get_untracked().len(), 2);
    screen.get_by_role(Role::Button, "Stop").click();
    assert_eq!(stops.get_untracked(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn code_block_numbers_follow_native_wrapped_rows_and_recipe_line_height() {
    use kivact::style::LineHeight;
    use kivact::test::{Element, RenderOptions};
    fn gutter(root: Element<'_>) -> Element<'_> {
        let mut pending = vec![root];
        while let Some(element) = pending.pop() {
            if element.text().starts_with("1\n") {
                return element;
            }
            pending.extend(element.children());
        }
        panic!("numbered gutter is mounted");
    }
    let (screen, (source, width)) = crate::support::mount_with(
        RenderOptions::from_env().theme(Theme::light().with(
            kivact::ui::UiRecipes::default().code_block(|r| {
                r.code(|previous| {
                    move |theme, state| {
                        previous(theme, state)
                            .font_size(18)
                            .line_height(LineHeight::px(31))
                    }
                })
            }),
        )),
        || {
            let source = signal(format!("{}\ntail", "abcdefghij ".repeat(8)));
            let width = signal(220.0);
            (
                (source, width),
                code_block(source)
                    .title("Wrapped")
                    .line_numbers(true)
                    .wrap(true)
                    .test_id("wrapped-code")
                    .style(move || style().width(width.get())),
            )
        },
    );
    let numbers = gutter(screen.get_by_test_id("wrapped-code"));
    let text = numbers.text();
    let second = text
        .split('\n')
        .position(|line| line == "2")
        .expect("second source line is numbered");
    assert!(
        second > 1,
        "wrapped source leaves blank numbered rows: {text:?}"
    );
    let editor = screen.get_by_role(Role::TextBox, "Wrapped");
    editor.press_key_with(
        Key::End,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    let caret = editor.text_caret().expect("native caret");
    let rect = numbers.rect().expect("gutter geometry");
    let rows = text.split('\n').count();
    let height = rect.height / rows as f64;
    assert!(
        (caret.y - rect.y - second as f64 * height).abs() < 2.0,
        "caret {caret:?}, gutter {rect:?}, rows {text:?}"
    );
    source.set(format!("tail\n{}", "abcdefghij ".repeat(8)));
    screen.advance(Duration::ZERO);
    assert_eq!(
        gutter(screen.get_by_test_id("wrapped-code"))
            .text()
            .split('\n')
            .position(|line| line == "2"),
        Some(1)
    );
    width.set(760.0);
    screen.advance(Duration::ZERO);
    let wide = gutter(screen.get_by_test_id("wrapped-code")).text();
    assert!(wide.split('\n').count() < rows);
    assert_eq!(editor.value(), Some(source.get_untracked()));
    screen.get_by_role(Role::Button, "Copy").click();
    assert_eq!(screen.clipboard(), Some(source.get_untracked()));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
