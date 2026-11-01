//! Role: The Lyra avatar, its status badge, group and overflow count.

use crate::Color;
use crate::slot::{Slot, slot};
use crate::theme_access::tokens;
use kivact::Asset;
use kivact::control::{ImageFit, Node, NodeBuilder, image, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Justify, Position, style};
use kivact::ui::IntoChildren;
use kivact::view::{IntoText, IntoView, Role, View};

/// The diameter of an [`Avatar`] (avatar.tsx `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AvatarSize {
    /// 32 px.
    #[default]
    Default,
    /// 24 px.
    Sm,
    /// 40 px.
    Lg,
}

impl AvatarSize {
    fn side(self) -> f32 {
        match self {
            AvatarSize::Default => 32.0,
            AvatarSize::Sm => 24.0,
            AvatarSize::Lg => 40.0,
        }
    }

    fn badge(self) -> f32 {
        match self {
            AvatarSize::Default => 10.0,
            AvatarSize::Sm => 8.0,
            AvatarSize::Lg => 12.0,
        }
    }
}

/// A round user picture with a text fallback (avatar.tsx). Created by [`avatar`].
pub struct Avatar {
    root: Node,
    fallback: kivact::Value<String>,
    image: Option<Asset>,
    size: AvatarSize,
    badge: Option<Slot>,
}

/// An avatar whose fallback text is `fallback` (usually initials).
///
/// The fallback shows until, or unless, an [`Avatar::image`] is available.
/// With an image the avatar is a `Role::Image` named by the fallback text.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn me() -> impl IntoView {
///     basic::avatar("CN").size(basic::AvatarSize::Lg)
/// }
/// ```
#[track_caller]
pub fn avatar(fallback: impl IntoText) -> Avatar {
    Avatar {
        root: node(),
        fallback: fallback.into_text_value(),
        image: None,
        size: AvatarSize::Default,
        badge: None,
    }
}

impl Avatar {
    /// Show `image` over the fallback.
    pub fn image(mut self, image: impl Into<Asset>) -> Self {
        self.image = Some(image.into());
        self.root = self.root.role(Role::Image).name(self.fallback);
        self
    }

    /// Choose the diameter.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Show `badge` at the bottom right (for example a status dot or a small icon).
    pub fn badge<V: IntoView>(mut self, badge: impl FnOnce() -> V + 'static) -> Self {
        self.badge = Some(slot(badge));
        self
    }
}

impl NodeBuilder for Avatar {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Avatar {
    fn into_view(self) -> View {
        let size = self.size;
        let side = size.side();
        let fallback = node()
            .style(move || {
                let p = tokens();
                style()
                    .position(Position::Absolute)
                    .left(0)
                    .top(0)
                    .size(side, side)
                    .direction(kivact::style::Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .radius(999)
                    .background(p.muted)
                    .color(p.muted_foreground)
                    .font_size(if size == AvatarSize::Sm { 12.0 } else { 14.0 })
                    .text_wrap(false)
            })
            .child(text(self.fallback));
        // The picture sits in a round clip: an image does not round its own corners.
        let picture = self.image.map(|asset| {
            node()
                .style(
                    style()
                        .position(Position::Absolute)
                        .left(0)
                        .top(0)
                        .size(side, side)
                        .radius(999)
                        .clip(true),
                )
                .child(
                    image(asset)
                        .fit(ImageFit::Cover)
                        .style(style().size(side, side)),
                )
        });
        let outline = node().style(
            style()
                .position(Position::Absolute)
                .left(0)
                .top(0)
                .size(side, side)
                .radius(999)
                .border(1, Color::Border),
        );
        let badge = self.badge.map(|content| {
            let dot = size.badge();
            node()
                .style(
                    style()
                        .position(Position::Absolute)
                        .right(0)
                        .bottom(0)
                        .size(dot, dot)
                        .direction(kivact::style::Direction::Row)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .radius(999)
                        .background(Color::Primary)
                        .color(Color::PrimaryForeground)
                        .ring(2, Color::Background)
                        .z_index(10),
                )
                .child(content)
        });
        self.root
            .base_style(
                style()
                    .position(Position::Relative)
                    .size(side, side)
                    .shrink(0),
            )
            .child(fallback)
            .child(picture)
            .child(outline)
            .child(badge)
            .into_view()
    }
}

/// A row of avatars that overlap by 8 px, each ringed in the background color.
///
/// Unlike the other containers, which take one `impl IntoView`, this takes a
/// tuple of views or a `Vec` of them: every child is wrapped to draw its ring
/// and overlap, so the avatars (and an [`avatar_group_count`]) must be
/// separate views.
#[track_caller]
pub fn avatar_group(children: impl IntoChildren) -> Node {
    row(children
        .into_views()
        .into_iter()
        .enumerate()
        .map(|(index, child)| {
            node()
                .style(move || {
                    let s = style().radius(999).ring(2, tokens().background).shrink(0);
                    if index == 0 { s } else { s.margin_left(-8) }
                })
                .child(child)
        })
        .collect::<Vec<_>>())
    .style(style().align_items(Align::Center))
}

/// The "+3" disc that ends an [`avatar_group`].
#[track_caller]
pub fn avatar_group_count(count_text: impl IntoText) -> Node {
    node()
        .style(
            style()
                .size(32, 32)
                .shrink(0)
                .direction(kivact::style::Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .radius(999)
                .background(Color::Muted)
                .color(Color::MutedForeground)
                .font_size(12),
        )
        .child(text(count_text))
}
