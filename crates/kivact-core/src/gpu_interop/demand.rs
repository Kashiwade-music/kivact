//! Role: Asking the host to start a window's GPU device when it is needed.

use crate::view::state::ViewState;
use kivact_engine::contract::HostRequest;
use kivact_runtime::Runtime;
use kivact_types::{RequestId, WindowId};

/// Ask `window` to start its GPU device: the App needs it (it imports a
/// shared image, shares one, displays a GPU frame or exports a drawable as
/// a GPU texture). Sent once per object that needs it; starting the device
/// again changes nothing. The host answers no effect for it.
pub(crate) fn demand_gpu(rt: &Runtime, window: WindowId) {
    let id = ViewState::of(rt).requests.borrow_mut().request_id();
    ViewState::tree(rt)
        .engine()
        .effect(RequestId(id), HostRequest::DemandGpu { window });
    rt.request_turn();
}
