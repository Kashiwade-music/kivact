//! Role: Passive theme data for button.
/// Timing for the standard button's busy indicator, configured through
/// [`crate::UiTokens`]; the default turns every 700 ms and reveals immediately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonBusyMotion {
    /// Milliseconds per revolution. A zero period is treated as 1 ms.
    pub spin_ms: u64,
    /// Milliseconds per revolution when reduced motion is active.
    pub reduced_spin_ms: u64,
    /// Milliseconds to wait before revealing. During this time the content
    /// recipe still sees the idle look. Skipped under reduced motion.
    pub reveal_delay_ms: u64,
    /// Milliseconds for the opacity fade. Zero reveals at once after the delay.
    pub reveal_fade_ms: u64,
}

impl Default for ButtonBusyMotion {
    fn default() -> Self {
        Self {
            spin_ms: 700,
            reduced_spin_ms: 700,
            reveal_delay_ms: 0,
            reveal_fade_ms: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ButtonTokens {
    pub busy_motion: ButtonBusyMotion,
}
