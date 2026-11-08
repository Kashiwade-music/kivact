//! Role: The drawables alive on this thread, by id, for engine events.
//!
//! Each live drawable keeps its export link and its notifier here, so an
//! engine event for its id reaches it. An event for an id that is not here
//! is stale and dropped.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use kivact_runtime::Notifier;
use kivact_types::DrawableId;

use crate::sync::Inbox;
use crate::view::drawable_share::ExportLink;

thread_local! {
    /// The exports of the drawables alive on this thread, by id.
    static LIVE: RefCell<HashMap<DrawableId, Registered>> = RefCell::new(HashMap::new());
}

/// What the UI thread keeps of a live drawable to deliver engine events to it.
#[derive(Clone)]
pub(super) struct Registered {
    pub(super) inbox: Arc<Inbox<ExportLink>>,
    pub(super) changed: Notifier,
}

/// Register the live drawable `id`.
pub(super) fn insert(id: DrawableId, registered: Registered) {
    LIVE.with(|live| {
        live.borrow_mut().insert(id, registered);
    });
}

/// Forget the drawable `id`, which was disposed.
pub(super) fn remove(id: DrawableId) {
    LIVE.with(|live| {
        live.borrow_mut().remove(&id);
    });
}

/// Run `f` on the link of the drawable `id` when it is alive on this thread.
pub(super) fn with_link(id: DrawableId, f: impl FnOnce(&Inbox<ExportLink>, &Notifier)) {
    let registered = LIVE.with(|live| live.borrow().get(&id).cloned());
    if let Some(registered) = registered {
        f(&registered.inbox, &registered.changed);
    }
}

/// Every drawable alive on this thread.
pub(super) fn all() -> Vec<Registered> {
    LIVE.with(|live| live.borrow().values().cloned().collect())
}
