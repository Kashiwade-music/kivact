//! Role: Define passive node observation values independently of retained nodes.

/// Position within the scrollable extent across the node's scrolling axes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollState {
    /// No axis has a scrollable extent, including an unbound reference.
    #[default]
    NotScrollable,
    /// Every scrollable axis is at its start.
    AtStart,
    /// At least one scrollable axis is between the combined endpoints.
    Middle,
    /// Every scrollable axis is at its end.
    AtEnd,
}
