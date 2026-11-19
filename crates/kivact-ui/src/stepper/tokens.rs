//! Role: Hold the stepper tokens that the theme's `UiTokens` carries.

/// Responsive step layout metrics, independent of interaction state.
#[derive(Clone, Debug, PartialEq)]
pub struct StepperTokens {
    pub gap: f64,
    pub padding: f64,
    pub min_step_width: f64,
    pub connector_gap: f64,
}

impl Default for StepperTokens {
    fn default() -> Self {
        Self {
            gap: 8.0,
            padding: 4.0,
            min_step_width: 96.0,
            connector_gap: 8.0,
        }
    }
}
