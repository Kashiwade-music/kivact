//! Role: The skeleton recipe state, tokens, recipe table and default looks.

use kivact_core::style::{ColorToken, Style, Theme, pct, style};
pub(crate) fn skeleton(t: &Theme, s: &SkeletonState) -> Style {
    let tokens = t.ext::<crate::tokens::UiTokens>();
    let base = style()
        .min_height(tokens.indicators.line_height)
        .width(pct(100))
        .background(ColorToken::Border)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36).radius(999),
        SkeletonShape::Rounded => base.radius(t.radius().md),
        SkeletonShape::Rect => base.radius(0),
        SkeletonShape::Text => base.radius(t.radius().sm),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonState {
    pub shape: SkeletonShape,
    pub pulse: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    #[default]
    Rect,
    Circle,
    Text,
    Rounded,
}

kivact_core::state::constant_value!(SkeletonShape);
