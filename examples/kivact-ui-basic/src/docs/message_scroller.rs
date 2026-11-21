//! Catalog page of Message Scroller: a long log and a conversation that grows.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Node;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("message-scroller", "Message Scroller", "Components", include_str!("message_scroller.rs"))
        .description("A scrolling area that keeps showing the latest position as messages are added. Scrolling up stops the following, and while the end is out of view a 'Jump to latest' button appears. The state of following can be read and written through a binding.")
        .usage("use kivact::prelude::*;\nuse kivact::style::use_theme;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the content to chat_thread. Give the parent a height. It starts at the end, and scrolling up shows the button.",
            default,
        ))
        .example(Example::new(
            "follow",
            "Conversation",
            "Following the end is the scroller's own state. Adding a message with 'Send' keeps the end in view while it follows; if you scrolled away it stays put, and 'Jump to latest' brings it back to the end and follows again.",
            follow,
        ))
}

// example: default
fn default() -> impl IntoView {
    /// A bordered box that gives the scroller its height.
    fn frame(content: impl IntoView) -> Node {
        node()
            .style(
                style()
                    .width(360)
                    .height(200)
                    .padding(12)
                    .border(1, basic::BORDER),
            )
            .child(content)
    }
    let lines: Vec<Node> = (1..=30)
        .map(|n| text(format!("Message number {n}")))
        .collect();
    frame(kivact::ui::chat_thread(column(lines)).style(style().height(pct(100))))
}
// end example

// example: follow
fn follow() -> impl IntoView {
    /// A bordered box that gives the scroller its height.
    fn frame(content: impl IntoView) -> Node {
        node()
            .style(
                style()
                    .width(360)
                    .height(200)
                    .padding(12)
                    .border(1, basic::BORDER),
            )
            .child(content)
    }
    /// One received message.
    fn reply(line: String) -> impl IntoView {
        kivact::ui::chat_message("")
            .content(line)
            .variant(basic::ChatBubbleVariant::Secondary)
    }
    let log = signal(vec!["Hello!".to_string(), "How can I help?".to_string()]);
    let send = move || log.update(|log| log.push(format!("Reply number {}", log.len() + 1)));
    column((
        frame(
            kivact::ui::chat_thread(column(for_each(
                log,
                |line| line.clone(),
                |line| reply(line.get()),
            )))
            .style(style().height(pct(100))),
        ),
        row(button("Send")
            .variant(ButtonVariant::Outline)
            .on_press(send))
        .style(style().gap(12).align_items(Align::Center)),
    ))
    .style(style().gap(12))
}
// end example
