//! Role: The Lyra marker: a muted inline note with optional rules and icon.

use crate::ColorName;
use crate::parts::sized_icon;

use kivact::view::{Node, NodeBuilder, node, row};

use kivact::style::{Align, Direction, pct, style};
use kivact::view::{IntoView, View};

/// The decoration of a [`Marker`] (marker.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MarkerVariant {
    /// Plain muted text.
    #[default]
    Default,
    /// Rules fill the space on both sides of the content.
    Separator,
    /// A rule under the content.
    Border,
}

/// A muted note such as a date divider in a conversation. Created by [`marker`].
pub struct Marker {
    root: Node,
    content: View,
    variant: MarkerVariant,
    icon: Option<View>,
}

/// A marker showing `content`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn day() -> impl IntoView {
///     basic::marker("Today").variant(basic::MarkerVariant::Separator)
/// }
/// ```
#[track_caller]
pub fn marker(content: impl IntoView) -> Marker {
    Marker {
        root: node(),
        content: content.into_view(),
        variant: MarkerVariant::Default,
        icon: None,
    }
}

impl Marker {
    /// Choose the decoration.
    pub fn variant(mut self, variant: MarkerVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Show `icon` before the content at 14 px.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
}

impl NodeBuilder for Marker {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// A 1 px line that takes the free space of the marker's row.
fn rule(margin_right: f64, margin_left: f64) -> Node {
    node().style(
        style()
            .grow(1)
            .min_width(0)
            .height(1)
            .margin_right(margin_right)
            .margin_left(margin_left)
            .background(ColorName::Border),
    )
}

impl IntoView for Marker {
    fn into_view(self) -> View {
        let variant = self.variant;
        let separated = variant == MarkerVariant::Separator;
        let content = row(self.content).style(
            style()
                .min_width(0)
                .align_items(Align::Center)
                .shrink(if separated { 0 } else { 1 }),
        );
        self.root
            .base_style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let s = style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .gap(8)
                    .width(pct(100))
                    .min_height(16)
                    .font_size(12)
                    .color(p.muted_foreground);
                if variant == MarkerVariant::Border {
                    s.border_bottom(1, p.border).padding_bottom(8)
                } else {
                    s
                }
            })
            .child(separated.then(|| rule(4.0, 0.0)))
            .child(self.icon.map(|icon| sized_icon(icon, 14.0)))
            .child(content)
            .child(separated.then(|| rule(0.0, 4.0)))
            .into_view()
    }
}
