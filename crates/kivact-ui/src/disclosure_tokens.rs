//! Role: Passive disclosure theme data.
use std::time::Duration;
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DisclosureTokens {
    pub motion: DisclosureMotion,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureMotion {
    pub animate: bool,
    pub duration: Duration,
}
impl Default for DisclosureMotion {
    fn default() -> Self {
        Self {
            animate: false,
            duration: Duration::from_millis(200),
        }
    }
}
