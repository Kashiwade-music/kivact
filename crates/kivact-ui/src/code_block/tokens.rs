//! Role: Hold the code block tokens that the theme's `UiTokens` carries.

/// Code metrics keep the gutter and selectable editor on the same line grid.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeBlockTokens {
    pub font_size: f64,
    pub line_height: f64,
    pub copy_feedback_ms: u64,
}

impl Default for CodeBlockTokens {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_height: 22.0,
            copy_feedback_ms: 2000,
        }
    }
}
