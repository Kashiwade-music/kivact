//! Role: Render domain-specific conversation-preview rows with caller-owned state.

use crate::variants::{AvatarSize, Hue};
use kivact::component;
use kivact::state::Value;
use kivact::style::color as core;
use kivact::style::{Align, Shadow, TextOverflow, style};
use kivact::view::{Element, Role, View, column, row, show, text};

/// A conversation row with a portrait, a title, a preview and an unread count.
///
/// `portrait` replaces the initials avatar with application-provided avatar
/// content. `selected` marks the open conversation.
#[component]
pub fn thread(
    initials: Value<String>,
    title: Value<String>,
    #[prop(default = "")] preview: Value<String>,
    #[prop(default = 0usize)] unread: Value<usize>,
    #[prop(default = false)] selected: Value<bool>,
    portrait: Option<View>,
) -> impl Element {
    let portrait = match portrait {
        Some(portrait) => portrait,
        None => kivact::ui::avatar(initials)
            .fallback_content(move || text(initials))
            .variant(AvatarSize(32.0))
            .into_view(),
    };
    row(())
        .with_node(|root| root.role(Role::Button))
        .focusable(true)
        .name(title)
        .selected(selected)
        .style(
            style()
                .font_size(14)
                .line_height(1.5)
                .padding(8)
                .gap(10)
                .align_items(Align::Center)
                .border_radius(8)
                .hover(style().background(core::SURFACE_ALT))
                .selected(style().background(core::SURFACE).shadow(Shadow::new(
                    0,
                    1,
                    3,
                    core::SHADOW,
                ))),
        )
        .child((
            portrait,
            column((
                text(title).style(style().font_size(14).font_weight(500).line_height(1.5)),
                text(preview).style(
                    style()
                        .font_size(12)
                        .line_height(1.5)
                        .color(core::TEXT_MUTED)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis),
                ),
            ))
            .style(style().flex_grow(1).min_width(0)),
            show(
                move || unread.get() > 0,
                move || {
                    kivact::ui::badge(move || unread.get().to_string())
                        .variant(Hue::Pink)
                        .variant(crate::BadgeVariant::Count)
                        .into_view()
                },
            ),
        ))
}
