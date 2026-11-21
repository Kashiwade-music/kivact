//! Role: The skeleton recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{Style, Theme, pct, style};
use kivact_macros::Prop;
pub(crate) fn skeleton(t: &Theme, s: &Look<SkeletonState>) -> Style {
    let tokens = t.ext::<crate::tokens::UiTokens>();
    let base = style()
        .min_height(tokens.indicators.line_height)
        .width(pct(100))
        .background(kivact_core::style::color::BORDER)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36).border_radius(999),
        SkeletonShape::Rounded => base.border_radius(t.get(&kivact_core::style::radius::MD_VAR)),
        SkeletonShape::Rect => base.border_radius(0),
        SkeletonShape::Text => base.border_radius(t.get(&kivact_core::style::radius::SM_VAR)),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonState {
    pub shape: SkeletonShape,
    pub pulse: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum SkeletonShape {
    #[default]
    Rect,
    Circle,
    Text,
    Rounded,
}

recipe_table! {
    /// The recipes of a skeleton placeholder.
    SkeletonTheme {
        root(root_recipe): SkeletonState = skeleton,
    }
}
