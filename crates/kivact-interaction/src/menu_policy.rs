//! Role: Passive interaction timing policy.
use std::time::Duration;
/// Whether disabled menu entries participate in roving keyboard focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisabledMenuFocus {
    Skip,
    #[default]
    Retain,
}

/// Pointer delays for opening a submenu or leaving it for a sibling item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubmenuTiming {
    pub open: Duration,
    pub close: Duration,
}

impl Default for SubmenuTiming {
    fn default() -> Self {
        Self {
            open: Duration::from_millis(140),
            close: Duration::ZERO,
        }
    }
}
/// A theme's opt-in to native menu press, drag and release selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuPress(pub bool);
