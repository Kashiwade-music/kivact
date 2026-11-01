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
                "BubbleVariant の 7 種類です。bubble_group で縦に並べます。",
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
    basic::bubble_group((
        basic::bubble("Default"),
        basic::bubble("Secondary").variant(basic::BubbleVariant::Secondary),
        basic::bubble("Muted").variant(basic::BubbleVariant::Muted),
        basic::bubble("Tinted").variant(basic::BubbleVariant::Tinted),
        basic::bubble("Outline").variant(basic::BubbleVariant::Outline),
        basic::bubble("Ghost").variant(basic::BubbleVariant::Ghost),
        basic::bubble("Destructive").variant(basic::BubbleVariant::Destructive),
    ))
    .style(style().width(360))
}
// end example

// example: align
fn align() -> impl IntoView {
    basic::bubble_group((
        basic::bubble("Are we still on for lunch?").variant(basic::BubbleVariant::Secondary),
        basic::bubble("Yes, see you at noon.").align(basic::BubbleAlign::End),
    ))
    .style(style().width(360))
}
// end example

// example: reactions
fn reactions() -> impl IntoView {
    basic::bubble_group((
        basic::bubble("Shipped the release!")
            .variant(basic::BubbleVariant::Muted)
            .reactions(move || {
                row((text("👀"), text("🚀"), text("+2")))
                    .role(Role::Image)
                    .name("Reactions: eyes, rocket, and 2 more")
            }),
        basic::bubble("Tests passed on the first try.")
            .align(basic::BubbleAlign::End)
            .reactions_side(basic::BubbleReactionsSide::Top)
            .reactions(move || {
                row((text("🎉"), text("👏")))
                    .role(Role::Image)
                    .name("Reactions: party popper, clapping hands")
            }),
    ))
    .style(style().width(360))
}
// end example

// example: pressable
fn pressable() -> impl IntoView {
    let count = signal(0);
    basic::bubble(text(move || format!("Pressed {} times", count.get())))
        .variant(basic::BubbleVariant::Outline)
        .on_press(move || count.update(|n| *n += 1))
}
// end example
