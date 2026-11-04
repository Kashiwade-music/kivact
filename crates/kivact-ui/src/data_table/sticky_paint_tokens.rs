//! Role: Passive table paint data.
use kivact_core::motion::Transition;
use kivact_core::style::{Color, ColorToken};
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

impl Default for DataTableStickyPaint {
    fn default() -> Self {
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
