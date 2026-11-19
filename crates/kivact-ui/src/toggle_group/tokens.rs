//! Role: Hold the toggle group tokens that the theme's `UiTokens` carries.

/// Passive spacing for toggle groups.
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroupTokens {
    pub gap: f64,
}

impl Default for ToggleGroupTokens {
    fn default() -> Self {
        Self { gap: 4.0 }
    }
}
