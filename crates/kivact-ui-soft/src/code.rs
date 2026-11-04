//! Role: Display an inline code fragment in the Neutral monospace treatment.
use crate::tokens;
use kivact::control::{Node, NodeBuilder, text};
use kivact::primitive::NodeExt;
use kivact::style::style;
use kivact::view::{IntoText, IntoView, View};

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
    color: CodeColor,
    size_inherit: bool,
}

/// Render an inline code fragment.
pub fn code(value: impl IntoText) -> Code {
    Code {
        root: text(value),
        color: CodeColor::default(),
        size_inherit: false,
    }
}

impl Code {
    /// Set the foreground color.
    pub fn color(mut self, color: CodeColor) -> Self {
        self.color = color;
        self
    }
    /// Use the surrounding text size and line height.
    pub fn size_inherit(mut self, inherit: bool) -> Self {
        self.size_inherit = inherit;
        self
    }
}

impl NodeBuilder for Code {
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
        root.base_style(move || {
            let p = tokens();
            let mut look = style()
                .font_family(p.typography.family_code)
                .background(p.colors.bg_muted)
                .padding_x(p.spacing.s1)
                .radius(p.radius.inner);
            if !size_inherit {
                look = look.font_size(p.typography.code.size_px);
            }
            match color {
                CodeColor::Primary => look.color(p.colors.text_primary),
                CodeColor::Secondary => look.color(p.colors.text_secondary),
                CodeColor::Inherit => look,
            }
        })
        .into_view()
    }
}
