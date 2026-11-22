//! Catalog page of Bubble: variants, alignment, reactions and pressable.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("bubble", "Bubble", "Components", include_str!("bubble.rs"))
        .description("A chat bubble. It has a color variant, a side (left or right) and reactions.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "variants",
            "Variants",
            "The seven BubbleVariant values. column stacks them vertically.",
            variants,
        ))
        .example(Example::new(
            "align",
            "Align",
            "Pass End as align to place the bubble on the right.",
            align,
        ))
        .example(Example::new(
            "reactions",
            "Reactions",
            "reactions overlays a pill of emoji and counts on the edge of the bubble. reactions_side chooses the top or the bottom. The bubble keeps room for the part of the pill that sticks out, so the next bubble does not overlap it.",
            reactions,
        ))
        .example(Example::new(
            "pressable",
            "Pressable",
            "With on_press the whole bubble is a button, and its color changes on hover.",
            pressable,
        ))
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
            .variant(basic::ChatBubbleVariant::Secondary),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Muted")
            .variant(basic::ChatBubbleVariant::Muted),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Tinted")
            .variant(basic::ChatBubbleVariant::Tinted),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Outline")
            .variant(basic::ChatBubbleVariant::Outline),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Ghost")
            .variant(basic::ChatBubbleVariant::Ghost),
        kivact::ui::chat_message("")
            .variant(basic::ChatLayout::Bubble)
            .content("Destructive")
            .variant(basic::ChatBubbleVariant::Destructive),
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
            .variant(basic::ChatBubbleVariant::Secondary),
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
            .variant(basic::ChatBubbleVariant::Muted)
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
        .variant(basic::ChatBubbleVariant::Outline)
        .on_press(move || count.update(|n| *n += 1))
}
// end example
