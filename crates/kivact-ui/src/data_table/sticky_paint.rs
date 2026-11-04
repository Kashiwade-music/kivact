//! Role: Theme policy and paint for sticky data table cells.
use kivact_core::motion::Transition;
use kivact_core::style::{
    Color, ColorToken, Style, Theme, ThemeExtension, TransitionProp, style, use_theme,
};

/// Theme-owned paint for pinned table columns.
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableStickyPaint {
    pub surface: Color,
    pub hover: Color,
    pub selected: Color,
    pub shadow: Color,
    pub shadow_width: f64,
    pub shadow_transition: Option<Transition>,
    pub overlay_transition: Option<Transition>,
}

impl ThemeExtension for DataTableStickyPaint {
    fn fallback(_: &Theme) -> Self {
        Self {
            surface: ColorToken::Surface.into(),
            hover: ColorToken::Hover.into(),
            selected: ColorToken::SurfaceAlt.into(),
            shadow: Color::TRANSPARENT,
            shadow_width: 0.0,
            shadow_transition: None,
            overlay_transition: None,
        }
    }
}

pub(super) fn surface(header: bool, selected: bool, tint: Option<Color>) -> Style {
    let theme = use_theme().get();
    let paint = theme.ext::<DataTableStickyPaint>();
    let mut color = paint.surface;
    if let Some(tint) = tint {
        color = over(&theme, tint, color);
    }
    if selected {
        color = over(&theme, paint.selected, color);
    }
    let mut base = style().background(color);
    if !header {
        base = base.group_hover(style().background(over(&theme, paint.hover, color)));
        if let Some(transition) = paint.overlay_transition.clone() {
            base = base.transition(TransitionProp::Paint, transition);
        }
    }
    base
}

fn over(theme: &Theme, overlay: Color, surface: Color) -> Color {
    let top = overlay.resolve(theme);
    let base = surface.resolve(theme);
    let mix = |a: f64, b: f64| a * top.a + b * (1.0 - top.a);
    Color::from_rgba_f64(
        mix(top.r, base.r),
        mix(top.g, base.g),
        mix(top.b, base.b),
        1.0,
    )
}
