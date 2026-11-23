//! Catalog page of Message: received, sent and a conversation.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("message", "Message", "Components", include_str!("message.rs"))
        .description("A chat message shown with an avatar and a heading. Give the author and the content, and choose the color with the Basic BubbleVariant.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "received",
            "Received",
            "A received message with an avatar, a header and a footer.",
            received,
        ))
        .example(Example::new(
            "sent",
            "Sent",
            "Pass End as align to put the message on the right. It applies to the whole message.",
            sent,
        ))
        .example(Example::new(
            "conversation",
            "Conversation",
            "column stacks the messages vertically.",
            conversation,
        ))
}

// example: received
fn received() -> impl IntoView {
    kivact::ui::chat_message("Ada")
        .content("Hi! Did you see the new design?")
        .variant(basic::ChatBubbleVariant::Secondary)
        .avatar(kivact::ui::avatar("AB").fallback_content(|| text("AB")))
        .footer(text("10:42"))
        .style(style().width(420))
}
// end example

// example: sent
fn sent() -> impl IntoView {
    kivact::ui::chat_message("You")
        .content("Yes, it looks great.")
        .align(kivact::ui::ChatAlign::End)
        .style(style().width(420))
}
// end example

// example: conversation
fn conversation() -> impl IntoView {
    column((
        kivact::ui::chat_message("")
            .content("Lunch at noon?")
            .variant(basic::ChatBubbleVariant::Muted)
            .avatar(kivact::ui::avatar("AB").fallback_content(|| text("AB"))),
        kivact::ui::chat_message("")
            .content("Sounds good.")
            .align(kivact::ui::ChatAlign::End),
    ))
    .style(style().min_width(0).gap(6).width(420))
}
// end example
