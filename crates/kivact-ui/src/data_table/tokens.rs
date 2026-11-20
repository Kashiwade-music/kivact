//! Role: Hold the data table tokens that the theme's `UiTokens` carries.
use kivact_core::motion::Transition;
use kivact_core::style::Color;

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
            neutral: kivact_core::style::color::TEXT_MUTED,
            info: kivact_core::style::color::ACCENT,
            success: kivact_core::style::color::SUCCESS,
            warning: kivact_core::style::color::WARNING,
            danger: kivact_core::style::color::DANGER,
            accent: kivact_core::style::color::ACCENT,
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
            surface: kivact_core::style::color::SURFACE,
            hover: kivact_core::style::color::HOVER,
            selected: kivact_core::style::color::SURFACE_ALT,
            shadow: Color::TRANSPARENT,
            shadow_width: 0.0,
            shadow_transition: None,
            overlay_transition: None,
        }
    }
}
