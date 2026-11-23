//! Role: The small enums of the style sheet: classes, groups, scrollbar width.

/// Whether a declaration can be a host-evaluated paint delta of a state
/// (`Delta`) or must be compiled into the node's style (`Rest`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PropClass {
    Delta,
    Rest,
}

/// The group a declaration belongs to (diagnostics order, text detection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PropGroup {
    Size,
    Spacing,
    Flow,
    Paint,
    Transform,
    Text,
    States,
    Motion,
}

/// Overlay scrollbar width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScrollbarWidth {
    /// The default (6 logical pixels).
    #[default]
    Auto,
    /// Hidden; scrolling still works.
    None,
    /// Explicit logical pixel width.
    Px(f64),
}

impl From<f64> for ScrollbarWidth {
    fn from(px: f64) -> Self {
        Self::Px(px)
    }
}

impl From<i32> for ScrollbarWidth {
    fn from(px: i32) -> Self {
        Self::Px(px as f64)
    }
}
