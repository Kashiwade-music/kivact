//! Role: Passive interaction timing policy.
use std::time::Duration;
/// Theme-wide tooltip timing, configured through [`crate::InteractionPolicy`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipTiming {
    /// How long the pointer or the keyboard focus has to stay before a
    /// tooltip shows. Zero shows it at once.
    pub delay: Duration,
    /// After a tooltip hides, a tooltip of another trigger that becomes
    /// engaged within this time shows without the delay, so moving along a
    /// toolbar does not wait at every button.
    pub skip_delay: Duration,
}

impl Default for TooltipTiming {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(500),
            skip_delay: Duration::from_millis(300),
        }
    }
}
