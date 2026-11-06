//! Role: Passive disclosure theme data.
use std::time::Duration;
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DisclosureTokens {
    pub motion: DisclosureMotion,
}
// xtask-allow(state-fields): Independent immutable theme motion settings; an optional transition overrides duration without representing a mounted phase.
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureMotion {
    pub animate: bool,
    pub duration: Duration,
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
