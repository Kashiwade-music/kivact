//! Role: Request settled host captures and resolve their owning futures.

use std::panic::Location;

use kivact_engine::contract::{HostRequest, SnapshotResult};
use kivact_runtime::driver::Runtime;
use kivact_runtime::{Completion, ErrorKind, Owner, UiError};
use kivact_types::{NodeId, Pixels, RequestId};

use super::state::{ViewState, Waiter};

pub(crate) fn request(
    rt: &Runtime,
    node: NodeId,
    owner: Owner,
    at: &'static Location<'static>,
) -> Completion<Result<Pixels, UiError>> {
    let reply = Completion::new();
    let mut requests = ViewState::of(rt).requests.borrow_mut();
    let id = requests.request_id();
    requests.waiters.insert(
        id,
        Waiter::Snapshot {
            owner,
            at,
            reply: reply.clone(),
        },
    );
    drop(requests);
    ViewState::tree(rt)
        .engine()
        .effect(RequestId(id), HostRequest::Snapshot(node));
    rt.request_turn();
    reply
}

pub(crate) fn resolve(rt: &Runtime, request: RequestId, result: &SnapshotResult) {
    let waiter = ViewState::of(rt)
        .requests
        .borrow_mut()
        .waiters
        .remove(&request.0);
    let Some(Waiter::Snapshot { owner, at, reply }) = waiter else {
        debug_assert!(
            waiter.is_none(),
            "invariant: a SnapshotResult answers a snapshot request"
        );
        return;
    };
    let fail = |kind, message| UiError::new(kind, message, Some(at));
    let result = if !owner.is_alive() {
        Err(fail(
            ErrorKind::Disposed {
                created_at: None,
                disposed_by: None,
            },
            "snapshot reference owner was disposed",
        ))
    } else {
        match result {
            SnapshotResult::Unmounted => {
                Err(fail(ErrorKind::Message, "snapshot node has no layout"))
            }
            SnapshotResult::Pixels(pixels) => Ok(pixels.clone()),
            SnapshotResult::Unsupported => Err(fail(
                ErrorKind::Unsupported,
                "snapshot contains an unsupported command or a GPU frame without CPU pixels",
            )),
            SnapshotResult::Failed(message) => {
                Err(UiError::new(ErrorKind::Asset, message.clone(), Some(at)))
            }
        }
    };
    reply.complete(result);
}
