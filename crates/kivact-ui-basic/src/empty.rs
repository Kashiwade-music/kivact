//! Role: The Lyra empty state: media, title, description and actions, centered.

use crate::ColorName;

use kivact::state::IntoText;
use kivact::state::Value;
use kivact::style::{Align, Direction, Justify, TextAlign, pct, style};
use kivact::view::{IntoView, View};
use kivact::view::{Node, NodeBuilder, column, node, text};

/// The frame of an [`EmptyState`] media slot (empty.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EmptyMediaVariant {
    /// Bare media.
    #[default]
    Default,
    /// A 32 px muted square around an icon.
    Icon,
}

/// An empty state (empty.tsx). Created by [`empty_state`].
pub struct EmptyState {
    root: Node,
    title: Value<String>,
    media: Option<kivact::view::View>,
    media_variant: EmptyMediaVariant,
    description: Option<Value<String>>,
    content: Option<kivact::view::View>,
}

/// An empty state titled `title`.
///
/// ```no_run
/// use kivact::view::{icon};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn no_projects() -> impl IntoView {
///     basic::empty_state("No Projects Yet")
///         .media(icon("folder"))
///         .media_variant(basic::EmptyMediaVariant::Icon)
///         .description("Get started by creating your first project.")
///         .content(button("Create Project"))
/// }
/// ```
#[track_caller]
pub fn empty_state(title: impl IntoText) -> EmptyState {
    EmptyState {
        root: node().style(
            style()
                .direction(Direction::Column)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(16)
                .width(pct(100))
                .min_width(0)
                .padding(24)
                .radius(0)
                .color(ColorName::Foreground)
                .font_size(12)
                .text_align(TextAlign::Center),
        ),
        title: title.into_text_value(),
        media: None,
        media_variant: EmptyMediaVariant::Default,
        description: None,
        content: None,
    }
}

impl EmptyState {
    /// Show `media` (an icon or an illustration) above the title.
    pub fn media(mut self, media: impl IntoView) -> Self {
        self.media = Some(media.into_view());
        self
    }

    /// Choose the frame of the media.
    pub fn media_variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.media_variant = variant;
        self
    }

    /// Show `text` under the title.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `content` (usually buttons) under the description.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(content.into_view());
        self
    }
}

impl NodeBuilder for EmptyState {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for EmptyState {
    fn into_view(self) -> View {
        let variant = self.media_variant;
        let media = self.media.map(|media| {
            node()
                .style(move || {
                    let s = style()
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .shrink(0)
                        .margin_bottom(8);
                    match variant {
                        EmptyMediaVariant::Default => s,
                        EmptyMediaVariant::Icon => {
                            let p = kivact::style::use_tokens::<crate::Tokens>();
                            s.size(32, 32).background(p.muted).color(p.foreground)
                        }
                    }
                })
                .child(media)
        });
        let header = column((
            media,
            text(self.title).style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
            self.description.map(|description| {
                text(description)
                    .style(style().line_height(1.625).color(ColorName::MutedForeground))
            }),
        ))
        .style(
            style()
                .align_items(Align::Center)
                .gap(8)
                .max_width(384)
                .text_align(TextAlign::Center),
        );
        let content = self.content.map(|content| {
            column(content).style(
                style()
                    .align_items(Align::Center)
                    .gap(10)
                    .width(pct(100))
                    .max_width(384)
                    .min_width(0)
                    .font_size(12),
            )
        });
        self.root.child(header).child(content).into_view()
    }
}
