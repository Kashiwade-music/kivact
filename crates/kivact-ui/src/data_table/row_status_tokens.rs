//! Role: Passive table paint data.
use kivact_core::style::{Color, ColorToken};
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
