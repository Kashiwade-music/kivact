//! Role: Define passive timing and metrics for chat and source-code surfaces.
/// Shared chat animation metrics.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatTokens {
    pub typing_period_ms: u64,
}
impl Default for ChatTokens {
    fn default() -> Self {
        Self {
            typing_period_ms: 1000,
        }
    }
}
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
