//! Role: Define passive table group projections and row-status descriptors.
use kivact_core::style::{Color, Icon};
use kivact_interaction::Tone;

/// The state a group-header recipe draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataTableGroupState {
    /// Matching rows in this group on the current page, including collapsed rows.
    pub count: usize,
    /// Whether the group is collapsed.
    pub collapsed: bool,
}

// xtask-allow(state-fields): Immutable presentation descriptor; optional icon and color are independent overrides.
/// A status announced by name and shown in the table's leading lane.
#[derive(Clone, Debug, PartialEq)]
pub struct RowStatus {
    pub(crate) label: String,
    pub(crate) tone: Tone,
    pub(crate) icon: Option<Icon>,
    pub(crate) color: Option<Color>,
}

impl RowStatus {
    /// Create a marker in the theme's color for this tone.
    pub fn new(label: impl Into<String>, tone: Tone) -> Self {
        Self {
            label: label.into(),
            tone,
            icon: None,
            color: None,
        }
    }

    /// Use an explicit color instead of the theme's tone.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Display a named or custom icon instead of the default mark.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}
