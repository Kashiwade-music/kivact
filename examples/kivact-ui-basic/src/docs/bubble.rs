//! Role: Catalog page of Bubble: variants, alignment, reactions and pressable.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "bubble",
        name: "Bubble",
        description: "チャットの吹き出しを表示する部品です。色の種類、左右の寄せ、リアクションを持ちます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("bubble.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "variants",
                "Variants",
                "BubbleVariant の 7 種類です。column で縦に並べます。",
                variants,
            ),
            Example::new(
                "align",
                "Align",
                "align に End を渡すと、右側に寄ります。",
                align,
            ),
            Example::new(
                "reactions",
                "Reactions",
                "reactions で、絵文字と数を並べたリアクションの pill を吹き出しの縁に重ねます。reactions_side で上下を選びます。吹き出しは pill がはみ出す分の余白を取るので、次の吹き出しには重なりません。",
                reactions,
            ),
            Example::new(
                "pressable",
                "Pressable",
                "on_press を渡すと吹き出し全体がボタンになり、ホバーで色が変わります。",
                pressable,
            ),
        ],
    }
}

// example: variants
fn variants() -> impl IntoView {
    column((
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Default"),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Secondary")
            .variant(basic::BubbleVariant::Secondary),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Muted")
            .variant(basic::BubbleVariant::Muted),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Tinted")
            .variant(basic::BubbleVariant::Tinted),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Outline")
            .variant(basic::BubbleVariant::Outline),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Ghost")
            .variant(basic::BubbleVariant::Ghost),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Destructive")
            .variant(basic::BubbleVariant::Destructive),
    ))
    .style(style().min_width(0).gap(8))
    .style(style().width(360))
}
// end example

// example: align
fn align() -> impl IntoView {
    column((
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Are we still on for lunch?")
            .variant(basic::BubbleVariant::Secondary),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Yes, see you at noon.")
            .align(kivact::ui::ChatAlign::End),
    ))
    .style(style().min_width(0).gap(8))
    .style(style().width(360))
}
// end example

// example: reactions
fn reactions() -> impl IntoView {
    column((
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Shipped the release!")
            .variant(basic::BubbleVariant::Muted)
            .reactions({
                row((text("👀"), text("🚀"), text("+2")))
                    .role(Role::Image)
                    .name("Reactions: eyes, rocket, and 2 more")
            }),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Tests passed on the first try.")
            .align(kivact::ui::ChatAlign::End)
            .reactions_side(kivact::ui::ChatReactionsSide::Top)
            .reactions({
                row((text("🎉"), text("👏")))
                    .role(Role::Image)
                    .name("Reactions: party popper, clapping hands")
            }),
    ))
    .style(style().min_width(0).gap(8))
    .style(style().width(360))
}
// end example

// example: pressable
fn pressable() -> impl IntoView {
    let count = signal(0);
    kivact::ui::chat_message("")
        .variant(basic::ChatLayout::Bubble)
        .content(text(move || format!("Pressed {} times", count.get())))
        .variant(basic::BubbleVariant::Outline)
        .on_press(move || count.update(|n| *n += 1))
}
// end example
