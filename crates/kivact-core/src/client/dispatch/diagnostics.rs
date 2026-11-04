//! Role: Diagnostic reports of the engine: layout warnings, dropped draw
//! commands, cut turns. None of them is an error of a node's owner chain.

use kivact_reactive::{ErrorKind, Runtime, UiError};
use kivact_types::{LayoutRule, NodeId};

use crate::view::state::ViewState;

pub(super) fn layout_warning(rt: &Runtime, node: NodeId, rule: LayoutRule, amount: f64) {
    // A lint of debug builds: where the node was created and what
    // it shows, in the `kivact::layout` tracing target.
    let tree = ViewState::tree(rt);
    let at = tree.get(node).and_then(|n| n.at);
    let content = tree.content(node).unwrap_or_default();
    drop(tree);
    let at = at.map(|at| format!(" at {at}")).unwrap_or_default();
    crate::client::notice::warn_layout(
        rt,
        format!("layout: {rule:?} by {amount:.1}px, text {content:?}{at}"),
    );
}

pub(super) fn draw_dropped(rt: &Runtime, node: NodeId, count: u32) {
    // The engine sends this once per drawing. Like an asset failure
    // it is a diagnostic of the default sink: the rest of the
    // drawing stays, so an `error_boundary` must not swap in its
    // fallback.
    let at = ViewState::tree(rt).get(node).and_then(|n| n.at);
    rt.report(
        None,
        UiError::new(
            ErrorKind::Unsupported,
            format!(
                "{count} draw command(s) have values that cannot be drawn (a non-finite or negative coordinate, size, radius or arc, a gradient stop that is not finite, in 0..=1 and in order, ..); they were dropped"
            ),
            at,
        ),
    );
}

pub(super) fn turns_cut(rt: &Runtime, turns: u32) {
    rt.report(
        None,
        UiError::new(
            ErrorKind::Cycle { path: Vec::new() },
            format!(
                "the host stopped after {turns} turns in a row that each left new events \
                 (a handler or an effect answering every layout or focus report with a write \
                 that reports again); the rest runs at its next wake"
            ),
            None,
        ),
    );
}
