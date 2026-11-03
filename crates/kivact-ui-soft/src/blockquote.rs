//! Role: Present a quotation and optional attribution with a logical start rule.
use crate::{TextColor, TextKind, text, tokens};
use kivact::Value;
use kivact::control::{Node, NodeBuilder, node};
use kivact::primitive::NodeExt;
use kivact::style::{Direction, style};
use kivact::view::{IntoText, IntoView, ReadingDirection, View, current_direction};

/// A quotation with optional attribution.
pub struct Blockquote {
    root: Node,
    content: Box<dyn FnOnce() -> View>,
    cite: Option<Value<String>>,
}

/// Render a quotation with caller-supplied content.
pub fn blockquote<V: IntoView>(content: impl FnOnce() -> V + 'static) -> Blockquote {
    Blockquote {
        root: node(),
        content: Box::new(move || content().into_view()),
        cite: None,
    }
}

/// Render a plain-text quotation.
pub fn blockquote_text(value: impl IntoText) -> Blockquote {
    let value = value.into_text_value();
    blockquote(move || text(value).kind(TextKind::Inherit))
}

impl Blockquote {
    /// Show an attribution beneath the quotation.
    pub fn cite(mut self, value: impl IntoText) -> Self {
        self.cite = Some(value.into_text_value());
        self
    }
}

impl NodeBuilder for Blockquote {
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
        root.base_style(|| {
            let p = tokens();
            let look = style()
                .direction(Direction::Column)
                .min_width(0)
                .color(p.colors.text_secondary);
            if current_direction() == ReadingDirection::Rtl {
                look.border_right(p.spacing.s0_5, p.colors.border_emphasized)
                    .padding_right(p.spacing.s4)
            } else {
                look.border_left(p.spacing.s0_5, p.colors.border_emphasized)
                    .padding_left(p.spacing.s4)
            }
        })
        .child(content())
        .child(cite.map(|value| {
            text(value)
                .kind(TextKind::Supporting)
                .color(TextColor::Inherit)
                .style(|| style().margin_top(tokens().spacing.s2).italic(false))
        }))
        .into_view()
    }
}
