//! Role: Accessibility semantics carried by nodes and scene regions.

pub use kivact_types::{
    A11yAction, A11yRequest, CheckedState, DismissReason, LiveRegion, Orientation, Role,
    SortDirection,
};
/// Optional semantics used by advanced controls.
///
/// A node keeps them in one box (`NodeSemantics::accessibility`) that only
/// the nodes using them allocate: numeric values, live regions, sorting,
/// orientation, the active descendant, the relations, list and tree
/// positions. The engine hands
/// them to Arvane's snapshot and native validation as they are, except
/// what it must not publish (see the fields).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticsProperties {
    /// Numeric lower bound.
    pub value_min: Option<f64>,
    /// Numeric upper bound.
    pub value_max: Option<f64>,
    /// Current numeric value.
    pub value_now: Option<f64>,
    /// Live announcement policy.
    pub live_region: Option<LiveRegion>,
    /// Sort direction.
    pub sort: SortDirection,
    /// Direction of the control.
    pub orientation: Orientation,
    /// Active descendant identity. Published only while it is shown, has a
    /// role and lies in this node's subtree or in that of a node it
    /// `controls` (this node included).
    pub active_descendant: Option<super::FocusTarget>,
    /// Nodes that describe this one (`aria-describedby`), in order.
    /// Like the other relations, a target is published only while it
    /// exists, is attached and has a role (a hidden one stays).
    pub described_by: Vec<super::FocusTarget>,
    /// Nodes this one controls (`aria-controls`), in order.
    pub controls: Vec<super::FocusTarget>,
    /// Nodes that label this one (`aria-labelledby`), in order.
    pub labelled_by: Vec<super::FocusTarget>,
    /// Depth in a hierarchy, counted from 1 (`aria-level`); `0` is not
    /// published.
    pub level: Option<u32>,
    /// Number of items in the set this node belongs to or holds
    /// (`aria-setsize`); `0` is published, an empty set.
    pub set_size: Option<u32>,
    /// Position in its set, counted from 1 (`aria-posinset`); `0`, or a
    /// position past `set_size`, is not published.
    pub pos_in_set: Option<u32>,
    /// Whether the node allows more than one selected item
    /// (`aria-multiselectable`). `None` leaves it unstated.
    pub multiselectable: Option<bool>,
    /// The text of a node that draws and edits its own (not a native
    /// editor), as runs for assistive technology, in the node's own
    /// coordinates. The engine moves the rectangles into the window's and
    /// gives each run a platform identity of its own (a run's `id` is its
    /// key in this node). Dropped when [`check_text`] finds it invalid.
    pub text: Option<kivact_types::SemanticsText>,
}

/// Why `text` cannot be published as a node's text runs, if it cannot: at
/// least one run, finite non-negative rectangles, carets inside their runs
/// and a selection and caret inside the text.
pub fn check_text(text: &kivact_types::SemanticsText) -> Result<(), &'static str> {
    let finite = |r: kivact_types::Rect| {
        [r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height]
            .into_iter()
            .all(f32::is_finite)
            && r.width >= 0.0
            && r.height >= 0.0
    };
    if text.runs.is_empty() {
        return Err("it has no run (empty text is one empty run)");
    }
    if !finite(text.caret) {
        return Err("its caret rectangle is not finite or has a negative size");
    }
    let mut length = 0u32;
    for run in text.runs.iter() {
        let count = u32::try_from(run.value.chars().count()).map_err(|_| "a run is too long")?;
        if !finite(run.bounds) || run.carets.iter().any(|c| !finite(c.rect)) {
            return Err("a run's rectangle is not finite or has a negative size");
        }
        if run.carets.iter().any(|c| c.offset > count) {
            return Err("a caret lies past the end of its run");
        }
        length = length.checked_add(count).ok_or("the text is too long")?;
    }
    let end = text
        .start
        .checked_add(length)
        .ok_or("the text is too long")?;
    let inside = |at: u32| (text.start..=end).contains(&at);
    if !inside(text.selection.0) || !inside(text.selection.1) {
        return Err("the selection lies outside the runs");
    }
    let mut ids = std::collections::HashSet::new();
    if !text.runs.iter().all(|run| ids.insert(run.id)) {
        return Err("two runs have the same id");
    }
    Ok(())
}
