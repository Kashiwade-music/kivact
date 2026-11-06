//! Role: Verify follow-to-latest through the public chat layout.

use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::Screen;
use kivact::test::render_with;
use kivact::ui::button;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use kivact::view::column;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::text;
use kivact_ui_soft as soft;

fn offset(screen: &Screen) -> f64 {
    screen
        .get_by_test_id("offset")
        .text()
        .parse()
        .expect("published scroll offset")
}

fn chat_screen() -> Screen {
    render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        || {
            let items = signal((0..20).collect::<Vec<i32>>());
            let history = signal(0_u64);
            let viewport = node_ref();
            let offset = viewport.scroll_offset();
            let messages = kivact::view::for_each(
                items,
                |id| *id,
                |id| {
                    text(move || format!("Message {}", id.get()))
                        .style(style().height(24).shrink(0))
                },
            );
            let chat = kivact::ui::chat_thread(messages)
                .gap(0.0)
                .viewport_ref(viewport)
                .history_revision(history)
                .style(style().size(340, 144).shrink(0))
                .test_id("chat-layout");
            let append = button("Append").on_press(move || {
                items.update(|items| items.push(items.last().copied().unwrap_or(0) + 1));
            });
            let prepend = button("Load history").on_press(move || {
                items.update(|items| {
                    let first = items.first().copied().unwrap_or(0);
                    items.splice(..0, first - 2..first);
                });
                history.update(|revision| *revision += 1);
            });
            column((
                chat,
                row((append, prepend)),
                text(move || offset.get().y.to_string()).test_id("offset"),
            ))
        },
    )
}

#[test]
fn append_follows_until_manual_scroll_and_jump_reattaches() {
    let screen = chat_screen();
    let initial = offset(&screen);
    assert!(initial > 100.0);
    let append = screen.get_by_role(Role::Button, "Append");
    append.click();
    assert!(offset(&screen) > initial);

    screen.get_by_test_id("chat-layout").scroll(0.0, -180.0);
    let detached = offset(&screen);
    append.click();
    assert!((offset(&screen) - detached).abs() < 1.0);

    screen.get_by_role(Role::Button, "Jump to latest").click();
    let latest = offset(&screen);
    assert!(latest > detached);
    assert!(
        screen
            .query_by_role(Role::Button, "Jump to latest")
            .is_none()
    );
    append.click();
    assert!(offset(&screen) > latest);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn prepending_history_keeps_the_same_messages_in_view_while_detached() {
    let screen = chat_screen();
    screen.get_by_test_id("chat-layout").scroll(0.0, -180.0);
    let detached = offset(&screen);
    let message = screen.get_by_text("Message 9");
    let before = message.rect().expect("mounted message").y;
    screen.get_by_role(Role::Button, "Load history").click();
    let anchored = offset(&screen);
    assert!(anchored > detached);
    assert!((message.rect().expect("mounted message").y - before).abs() < 1.0);
    screen.get_by_role(Role::Button, "Append").click();
    assert!((offset(&screen) - anchored).abs() < 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
