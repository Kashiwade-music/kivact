//! Role: Passive disclosure theme data.
use std::time::Duration;
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DisclosureTokens {
    pub motion: DisclosureMotion,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureMotion {
    // xtask-allow(state-fields): independent: a theme option; every combination is valid
    pub animate: bool,
    pub duration: Duration,
    // xtask-allow(state-fields): independent: a theme option; every combination is valid
    /// Optional theme transition; absent uses the duration-based ease-out tween.
    pub transition: Option<kivact_core::motion::Transition>,
}
impl Default for DisclosureMotion {
    fn default() -> Self {
        Self {
            animate: false,
            duration: Duration::from_millis(200),
            transition: None,
        }
    }
}
