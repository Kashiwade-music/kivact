//! Role: Style canonical loading placeholders with Neutral corner and pulse appearance.
use crate::Tokens;
use kivact::style::{Style, Theme, pct, style};
use kivact::ui::{Look, SkeletonShape, SkeletonState};
/// Corner profile supplied to the standard skeleton's variant hook.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonRadius {
    /// Use square corners.
    None,
    /// Use the zero-radius profile.
    R0,
    /// Use the theme inner radius.
    R1,
    /// Use the theme element radius.
    R2,
    #[default]
    /// Use the theme container radius.
    R3,
    /// Use the theme container radius for the larger profile.
    R4,
    /// Use the theme fully rounded radius.
    Rounded,
}
impl kivact::ui::Variant for SkeletonRadius {}
pub(crate) fn skeleton(theme: &Theme, s: &Look<SkeletonState>) -> Style {
    let t = theme.ext::<Tokens>();
    let radius = match s.variant::<SkeletonRadius>() {
        Some(SkeletonRadius::None | SkeletonRadius::R0) => 0.0,
        Some(SkeletonRadius::R1) => t.radius.inner,
        Some(SkeletonRadius::R2) => t.radius.element,
        Some(SkeletonRadius::Rounded) => t.radius.full,
        Some(SkeletonRadius::R3 | SkeletonRadius::R4) => t.radius.container,
        None => match s.shape {
            SkeletonShape::Circle => t.radius.full,
            SkeletonShape::Rounded | SkeletonShape::Rect => t.radius.container,
            SkeletonShape::Text => t.radius.inner,
        },
    };
    let base = style()
        .size(pct(100), pct(100))
        .radius(radius)
        .background(t.colors.skeleton)
        .opacity((1.75 - 1.5 * s.pulse).min(0.925));
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36),
        SkeletonShape::Text => base.height(t.typography.body.line_height_px),
        _ => base,
    }
}
