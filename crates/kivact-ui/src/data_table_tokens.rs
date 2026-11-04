//! Role: Passive data table theme paint and token definitions.
use kivact_core::motion::Transition;
use kivact_core::style::{Color, ColorToken};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DataTableTokens {
    pub sticky: DataTableStickyPaint,
    pub row_status: DataTableRowStatusPaint,
}

/// The theme's row-status colors, in the same order as the named tones.
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableRowStatusPaint {
    pub neutral: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub accent: Color,
    pub header_key: &'static str,
}

impl Default for DataTableRowStatusPaint {
    fn default() -> Self {
        Self {
            neutral: ColorToken::TextMuted.into(),
            info: ColorToken::Accent.into(),
            success: ColorToken::Success.into(),
            warning: ColorToken::Warning.into(),
            danger: ColorToken::Danger.into(),
            accent: ColorToken::Accent.into(),
            header_key: "data_table.row_status.column_header",
        }
    }
}

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
