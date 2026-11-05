//! Role: Catalog page of Message: received, sent and a conversation.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "message",
        name: "Message",
        description: "チャットの発言を、アバターや見出しとともに表示する部品です。author と content を指定し、Basic の BubbleVariant で色を選びます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("message.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "received",
                "Received",
                "avatar、header、footer を添えた、受け取った発言です。",
                received,
            ),
            Example::new(
                "sent",
                "Sent",
                "align に End を渡すと右寄りになります。発言全体の配置に反映されます。",
                sent,
            ),
            Example::new(
                "conversation",
                "Conversation",
                "column で発言を縦に並べます。",
                conversation,
            ),
        ],
    }
}

// example: received
fn received() -> impl IntoView {
    kivact::ui::chat_message("Ada")
        .content("Hi! Did you see the new design?")
        .variant(basic::BubbleVariant::Secondary)
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
            .variant(basic::BubbleVariant::Muted)
            .avatar(kivact::ui::avatar("AB").fallback_content(|| text("AB"))),
        kivact::ui::chat_message("")
            .content("Sounds good.")
            .align(kivact::ui::ChatAlign::End),
    ))
    .style(style().min_width(0).gap(6).width(420))
}
// end example
