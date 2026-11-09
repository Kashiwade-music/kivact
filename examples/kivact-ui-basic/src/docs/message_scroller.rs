//! Role: Catalog page of Message Scroller: a long log and a conversation that grows.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "message-scroller",
        name: "Message Scroller",
        description: "メッセージが増えても最後の位置を表示し続ける、スクロールする領域です。上へスクロールすると追従をやめ、末尾が見えない間は「Jump to latest」のボタンが出ます。追従の状態は束縛で読み書きできます。",
        usage: "use kivact::prelude::*;\nuse kivact::style::use_theme;\nuse kivact_ui_basic as basic;",
        source: include_str!("message_scroller.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "chat_thread に内容を渡します。親に高さを持たせてください。最初から末尾が表示され、上へスクロールするとボタンが出ます。",
                default,
            ),
            Example::new(
                "follow",
                "Conversation",
                "末尾への追従はスクローラー自身の状態です。「Send」でメッセージを足すと、追従中は末尾が見え続け、上へ離れていると動かず、「Jump to latest」で末尾へ戻ると再び追従します。",
                follow,
            ),
        ],
    }
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
                    .border(1, basic::ColorName::Border),
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
                    .border(1, basic::ColorName::Border),
            )
            .child(content)
    }
    /// One received message.
    fn reply(line: String) -> impl IntoView {
        kivact::ui::chat_message("")
            .content(line)
            .variant(basic::BubbleVariant::Secondary)
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
