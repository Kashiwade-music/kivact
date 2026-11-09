//! Role: Present a quotation and optional attribution with a logical start rule.
use crate::{TextColor, TextKind, styled_text};
use kivact::state::Value;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::node;

use kivact::state::IntoText;
use kivact::style::Direction;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::ReadingDirection;
use kivact::view::View;
use kivact::view::use_direction;

/// A quotation with optional attribution.
pub struct Blockquote {
    root: Node,
    content: View,
    cite: Option<Value<String>>,
}

/// Render a quotation with caller-supplied content.
pub fn blockquote(content: impl IntoView) -> Blockquote {
    Blockquote {
        root: node(),
        content: content.into_view(),
        cite: None,
    }
}

/// Render a plain-text quotation.
pub fn blockquote_text(value: impl IntoText) -> Blockquote {
    let value = value.into_text_value();
    blockquote(styled_text(value).kind(TextKind::Inherit))
}

impl Blockquote {
    /// Show an attribution beneath the quotation.
    pub fn cite(mut self, value: impl IntoText) -> Self {
        self.cite = Some(value.into_text_value());
        self
    }
}

impl Element for Blockquote {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Blockquote {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            cite,
        } = self;
        root.with_node(move |node| {
            node.base_style(|| {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let look = style()
                    .direction(Direction::Column)
                    .min_width(0)
                    .color(p.colors.text_secondary);
                if use_direction().get() == ReadingDirection::Rtl {
                    look.border_right(p.spacing.s0_5, p.colors.border_emphasized)
                        .padding_right(p.spacing.s4)
                } else {
                    look.border_left(p.spacing.s0_5, p.colors.border_emphasized)
                        .padding_left(p.spacing.s4)
                }
            })
        })
        .child(content)
        .child(cite.map(|value| {
            styled_text(value)
                .kind(TextKind::Supporting)
                .color(TextColor::Inherit)
                .style(|| {
                    style()
                        .margin_top(kivact::style::use_tokens::<crate::Tokens>().spacing.s2)
                        .italic(false)
                })
        }))
        .into_view()
    }
}
