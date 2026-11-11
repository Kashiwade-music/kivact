//! Role: Passive theme data for progress.
use kivact_core::motion::Easing;
/// Timing of standard progress bars. Configure it through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressMotion {
    // xtask-allow(state-fields): independent: whether the first determinate fill animates from zero; the moving-fill clock that reduced_period_ms drives is unaffected
    /// Whether the first determinate fill animates from zero.
    pub animate_initial: bool,
    /// Milliseconds for changes to the determinate fill.
    pub determinate_ms: u64,
    /// Easing of determinate fill changes.
    pub determinate_easing: Easing,
    /// Milliseconds per full cycle of the moving-fill clock.
    pub period_ms: u64,
    // xtask-allow(state-fields): option: the moving-fill clock's period under reduced motion; absent leaves the indeterminate fill still
    /// Optional full clock period in reduced motion. None leaves it still.
    pub reduced_period_ms: Option<u64>,
}

impl crate::Variant for ProgressMotion {}

impl Default for ProgressMotion {
    fn default() -> Self {
        Self {
            animate_initial: true,
            determinate_ms: 400,
            determinate_easing: Easing::EaseOut,
            period_ms: 1300,
            reduced_period_ms: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ProgressTokens {
    pub motion: ProgressMotion,
}
