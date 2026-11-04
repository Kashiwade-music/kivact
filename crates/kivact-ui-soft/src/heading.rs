//! Role: Provide semantic headings with the Neutral heading or display typography.
use crate::text::{
    Text, TextColor, TextJustify, TextKind, TextWeight, TextWrap, TruncateTooltip, WordBreak, text,
};
use kivact::view::Node;
use kivact::view::NodeBuilder;

use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::view::IntoView;
use kivact::view::View;

/// The heading display option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingDisplay {
    D1,
    D2,
    D3,
}

/// A semantic heading created with [`heading`].
pub struct Heading {
    text: Text,
}

/// Show a heading with a visual and semantic level from 1 to 6.
#[track_caller]
pub fn heading(level: u32, value: impl IntoText) -> Heading {
    let level = level.clamp(1, 6);
    Heading {
        text: text(value).heading_level(level).heading_appearance(level),
    }
}

impl Heading {
    /// Use display typography while keeping the heading level.
    pub fn display(mut self, display: HeadingDisplay) -> Self {
        self.text = self.text.kind(match display {
            HeadingDisplay::D1 => TextKind::Display1,
            HeadingDisplay::D2 => TextKind::Display2,
            HeadingDisplay::D3 => TextKind::Display3,
        });
        self
    }

    /// Override the level reported to assistive technology.
    pub fn accessibility_level(self, level: impl IntoValue<u32>) -> Self {
        let level = level.into_value();
        self.with_node(move |node| {
            node.with_node(move |node| node.level(move || level.get().clamp(1, 6)))
        })
    }

    /// Set the color.
    pub fn color(mut self, value: impl IntoValue<TextColor>) -> Self {
        self.text = self.text.color(value);
        self
    }

    /// Set the weight.
    pub fn weight(mut self, value: impl IntoValue<TextWeight>) -> Self {
        self.text = self.text.weight(value);
        self
    }

    /// Set the max lines.
    pub fn max_lines(mut self, value: impl IntoValue<u32>) -> Self {
        self.text = self.text.max_lines(value);
        self
    }

    /// Set the truncate tooltip.
    pub fn truncate_tooltip(mut self, value: TruncateTooltip) -> Self {
        self.text = self.text.truncate_tooltip(value);
        self
    }

    /// Set the wrap.
    pub fn wrap(mut self, value: TextWrap) -> Self {
        self.text = self.text.wrap(value);
        self
    }

    /// Set the justify.
    pub fn justify(mut self, value: impl IntoValue<TextJustify>) -> Self {
        self.text = self.text.justify(value);
        self
    }

    /// Set the word break.
    pub fn word_break(mut self, value: WordBreak) -> Self {
        self.text = self.text.word_break(value);
        self
    }

    /// Set the capsize.
    pub fn capsize(mut self, value: impl IntoValue<bool>) -> Self {
        self.text = self.text.capsize(value);
        self
    }

    /// Set the strikethrough.
    pub fn strikethrough(mut self, value: impl IntoValue<bool>) -> Self {
        self.text = self.text.strikethrough(value);
        self
    }
}

impl NodeBuilder for Heading {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.text.node_mut()
    }
}

impl IntoView for Heading {
    fn into_view(self) -> View {
        self.text.into_view()
    }
}
