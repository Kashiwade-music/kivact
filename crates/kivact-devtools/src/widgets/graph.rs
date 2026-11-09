//! Role: The cause graph of one recorded turn.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

/// A graph of causes whose cards select a node or open a source location.
pub(crate) fn cause_graph(
    graph: Value<Option<p::TurnGraph>>,
    on_select: impl Fn(p::CauseNode) + 'static,
    on_open: impl Fn(p::SourceLocation) + 'static,
) -> impl IntoView {
    let _ = (graph, on_select, on_open);
    node()
}
