//! Role: Display an inline code fragment in the Neutral monospace treatment.

use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::View;

/// The ink used for an inline code fragment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeColor {
    #[default]
    Primary,
    Secondary,
    Inherit,
}

/// A monospace fragment with a muted background.
pub struct Code {
    root: Node,
    color: Value<CodeColor>,
    size_inherit: Value<bool>,
}

/// Render an inline code fragment.
pub fn code(value: impl IntoText) -> Code {
    Code {
        root: text(value),
        color: (CodeColor::default()).into_value(),
        size_inherit: (false).into_value(),
    }
}

impl Code {
    /// Set the foreground color.
    pub fn color(mut self, color: impl IntoValue<CodeColor>) -> Self {
        self.color = color.into_value();
        self
    }
    /// Use the surrounding text size and line height.
    pub fn size_inherit(mut self, inherit: impl IntoValue<bool>) -> Self {
        self.size_inherit = inherit.into_value();
        self
    }
}

impl NodeBuilder for Code {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Code {
    fn into_view(self) -> View {
        let Self {
            root,
            color,
            size_inherit,
        } = self;
        root.with_node(move |node| {
            node.base_style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let mut look = style()
                    .font_family(p.typography.family_code)
                    .background(p.colors.bg_muted)
                    .padding_x(p.spacing.s1)
                    .radius(p.radius.inner);
                if !size_inherit.get() {
                    look = look.font_size(p.typography.code.size_px);
                }
                match color.get() {
                    CodeColor::Primary => look.color(p.colors.text_primary),
                    CodeColor::Secondary => look.color(p.colors.text_secondary),
                    CodeColor::Inherit => look,
                }
            })
        })
        .into_view()
    }
}

kivact::state::constant_value!(CodeColor);
