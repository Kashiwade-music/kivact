//! Role: Build semantic headings with theme-owned typography.
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::state::IntoText;
use kivact_core::view::{IntoView, Node, NodeBuilder, Role, View, text};

use crate::component_values::HeadingState;
/// A semantic heading whose typography comes from the theme.
pub struct Heading {
    root: Node,
    level: u8,
    variants: Variants,
}
/// Show `text` as a heading, clamping `level` to 1 through 6.
#[track_caller]
pub fn heading(value: impl IntoText, level: u8) -> Heading {
    let value = value.into_text_value();
    let level = level.clamp(1, 6);
    Heading {
        root: text(value)
            .role(Role::Heading)
            .level(u32::from(level))
            .name(value),
        level,
        variants: Variants::default(),
    }
}
impl Heading {
    /// Add an appearance read by the heading recipe.
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for Heading {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Heading {
    fn into_view(self) -> View {
        with_base_recipe_style(
            self.root,
            |r| r.heading_recipe(),
            &self.variants,
            move || HeadingState { level: self.level },
        )
        .into_view()
    }
}
