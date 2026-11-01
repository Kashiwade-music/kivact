//! Role: Catalog page of Message: received, sent and a conversation.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "message",
        name: "Message",
        description: "チャットの発言を、アバターや見出しとともに表示する部品です。内容には bubble を渡します。",
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
                "align に End を渡すと右寄りになります。中の bubble にも End を渡します。",
                sent,
            ),
            Example::new(
                "conversation",
                "Conversation",
                "message_group で発言を縦に並べます。",
                conversation,
            ),
        ],
    }
}

// example: received
fn received() -> impl IntoView {
    basic::message(
        basic::bubble("Hi! Did you see the new design?").variant(basic::BubbleVariant::Secondary),
    )
    .avatar(move || basic::avatar("AB"))
    .header(move || text("Ada"))
    .footer(move || text("10:42"))
    .style(style().width(420))
}
// end example

// example: sent
fn sent() -> impl IntoView {
    basic::message(basic::bubble("Yes, it looks great.").align(basic::BubbleAlign::End))
        .align(basic::MessageAlign::End)
        .header(move || text("You"))
        .style(style().width(420))
}
// end example

// example: conversation
fn conversation() -> impl IntoView {
    basic::message_group((
        basic::message(basic::bubble("Lunch at noon?").variant(basic::BubbleVariant::Muted))
            .avatar(move || basic::avatar("AB")),
        basic::message(basic::bubble("Sounds good.").align(basic::BubbleAlign::End))
            .align(basic::MessageAlign::End),
    ))
    .style(style().width(420))
}
// end example
