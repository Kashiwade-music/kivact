//! Role: Answers and failures from the platform: effect results, asset
//! failures, host failures, memory warnings.

use kivact_runtime::{ErrorKind, Runtime, UiError};
use kivact_types::{NodeId, PlatformResult, RequestId, Revision};

use super::common::{handler, node_owner, run_handler};
use crate::client::state::HostState;
use crate::view::requests;
use crate::view::state::ViewState;

pub(super) fn effect_result(rt: &Runtime, request: RequestId, result: &PlatformResult) {
    requests::resolve_effect(rt, request.0, result.clone());
}

pub(super) fn asset_failed(rt: &Runtime, node: NodeId, message: &str) {
    if let Some(handle) = handler(rt, node, |handlers| handlers.image_error.clone()) {
        run_handler(rt, node_owner(rt, node), || handle(message));
        return;
    }
    let first = HostState::of(rt)
        .asset_failures
        .borrow_mut()
        .insert((node, message.to_owned()));
    if !first {
        return;
    }
    // Located where the node was created (`image(..)`, `text(..)`).
    // A diagnostic of the default sink, not an error of the node's
    // owner chain: the node draws nothing (or uses the system UI
    // font) and the rest of its subtree stays, so an
    // `error_boundary` around it must not swap in its fallback.
    let at = ViewState::tree(rt).get(node).and_then(|n| n.at);
    rt.report(None, UiError::new(ErrorKind::Asset, message.to_owned(), at));
}

pub(super) fn host_failure(rt: &Runtime, revision: Revision, reason: &str) {
    rt.note(format!("host failure at revision {}: {reason}", revision.0));
}

// Core retains live state, not a second text/image cache. Release only
// excess container capacity before application lifecycle callbacks.
pub(super) fn memory_warning(rt: &Runtime) {
    ViewState::of(rt).release_capacity();
    HostState::of(rt)
        .asset_failures
        .borrow_mut()
        .shrink_to_fit();
    rt.warn(
        "the platform is short of memory (MemoryWarning); the host released text and unused asset caches".into(),
    );
}
