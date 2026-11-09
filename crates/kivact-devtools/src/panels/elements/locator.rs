//! Role: The test locator expression of the selected node.

use kivact_devtools_protocol as p;

use crate::state::Session;

/// The `kivact::test` query of `detail`, as the review's Markdown writes it.
pub(crate) fn locator_of(session: Session, detail: &p::NodeDetail) -> Option<String> {
    let node = &detail.node;
    let window_title = session
        .windows
        .with(|windows| {
            windows
                .iter()
                .find(|window| window.id == node.window)
                .map(|window| window.title.clone())
        })
        .unwrap_or_default();
    let item = p::ReviewItem {
        id: String::new(),
        window_title,
        window: node.window,
        target: p::ReviewTarget::Node {
            node: node.id,
            role: node.role,
            name: node.name.clone(),
            test_id: node.test_id.clone(),
            source: node.source.clone(),
            call_chain: detail.call_chain.clone(),
            bounds: detail.bounds.border.clone(),
            states: detail.states.clone(),
        },
        style_changes: Vec::new(),
        image: None,
        comments: Vec::new(),
    };
    p::locator(&item)
}
