//! Role: Sends a drawable's export request and geometry to the engine.
//!
//! The requests are sent only when they differ from the ones sent last, so
//! a turn that changes nothing sends nothing.

use std::cell::RefCell;
use std::sync::Arc;

use kivact_engine::contract::{ExportRequestMode, HostRequest};
use kivact_runtime::{AppId, Notifier, Runtime};
use kivact_types::DrawableId;

use crate::sync::Inbox;
use crate::view::drawable_share::ExportLink;

/// The task of a drawable's share: each change the app made marks the
/// drawable's notifier, so its requests are sent in the next turn.
pub(super) async fn wake(inbox: Arc<Inbox<ExportLink>>, changed: Notifier) {
    loop {
        inbox.next(ExportLink::take_changed).await;
        changed.notify();
    }
}

/// What the engine was last asked to export: mode, generation and scale
/// (`None` follows the window's scale, which the engine reads).
type Sent = kivact_types::cell::Sent<(ExportRequestMode, u64, Option<f64>)>;

/// Send the export request of the drawable `id` when it differs from the
/// one sent last, and its geometry when the app set one.
pub(super) fn send_requests(
    app: AppId,
    id: DrawableId,
    inbox: &Inbox<ExportLink>,
    sent: &RefCell<Sent>,
) {
    let Some(rt) = Runtime::get(app) else {
        return;
    };
    let writer = crate::view::state::ViewState::tree(&rt).engine().clone();
    let (request, geometry) = inbox.update(|link| match link {
        ExportLink::Open(open) => (
            Some((open.request_mode(), open.current(), open.scale)),
            open.geometry.take().map(|g| (open.current(), g)),
        ),
        ExportLink::Closed => (None, None),
    });
    if let Some(now) = request
        && sent.borrow_mut().update(now.clone()).is_some()
    {
        let (mode, generation, scale) = now;
        writer.drawable_request(HostRequest::DrawableExport {
            id,
            generation,
            mode,
            scale,
        });
    }
    if let Some((generation, geometry)) = geometry {
        writer.drawable_request(HostRequest::SetDrawableGeometry {
            id,
            generation,
            geometry,
        });
    }
}
