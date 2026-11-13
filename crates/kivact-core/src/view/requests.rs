//! Role: Event requests and async I/O.
//!
//! Requests the view layer sends to the host and whose answers come back as
//! events: keyboard focus (`NodeRef::focus`, the focus restores after a
//! modal overlay or a popup closes and `NodeRef::restore_focus`),
//! platform effects (`kivact::platform`, the App's fonts). Request ids come
//! from one per-App counter.
//!
//! Restores are checked once at flush against engine focus eligibility.
//! Cancellation produces no host write or error report. Activations run after
//! the current handler returns, through the host activation path.

use std::panic::Location;

use kivact_engine::contract::HostRequest;
use kivact_types::{PlatformResult, RequestId};

use super::state::{
    EffectWaiter, FocusCompletion, FocusReply, FocusRequest, RequestStage, ViewState,
};
use kivact_runtime::{ErrorKind, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_style::asset::Asset;

/// Ask the host to focus `node` (written with the next revision); an unmounted
/// node fails at once.
pub(crate) fn request_focus(
    rt: &Runtime,
    node: Option<kivact_types::FocusTarget>,
    reply: FocusReply,
    visible: bool,
) {
    let Some(node) = node else {
        let err = UiError::new(ErrorKind::Platform, "focus: node is not mounted", None);
        resolve_focus_reply(rt, reply, FocusResult::Answered(Err(err)));
        return;
    };
    let mut req = ViewState::of(rt).requests.borrow_mut();
    let id = req.request_id();
    let id = RequestId(id);
    req.focus.order.push(id);
    req.focus.requests.insert(
        id,
        FocusRequest {
            target: node,
            visible,
            reply,
            stage: RequestStage::Queued,
        },
    );
    drop(req);
    rt.request_turn();
}

enum FocusResult {
    Answered(Result<(), UiError>),
    Canceled,
}

fn resolve_focus_reply(rt: &Runtime, reply: FocusReply, result: FocusResult) {
    if let FocusReply::Report(completion) | FocusReply::RestoreAndReport(completion) = reply
        && let FocusResult::Answered(Err(error)) = result
    {
        let error = UiError::new(
            error.kind().clone(),
            error.message().to_string(),
            Some(completion.at),
        );
        rt.report(Some(completion.owner), error);
    }
}

/// Send a platform effect to the host (a write of the next revision).
pub(crate) fn request_effect(rt: &Runtime, req: HostRequest, waiter: EffectWaiter) {
    let id = {
        let mut r = ViewState::of(rt).requests.borrow_mut();
        let id = r.request_id();
        r.effects.insert(id, waiter);
        id
    };
    ViewState::tree(rt).engine().effect(RequestId(id), req);
    rt.request_turn();
}

/// Register the App's fonts (`App::font`), each located at its call: one
/// `RegisterFont` effect per font, applied before mounting the root's
/// mount so the first layout measures with them. A font the host rejects
/// is reported to the default sink as `ErrorKind::Asset`.
pub(crate) fn register_app_fonts(rt: &Runtime, fonts: Vec<(Asset, &'static Location<'static>)>) {
    for (font, at) in fonts {
        let waiter = EffectWaiter::Report {
            owner: None,
            at,
            failed: ErrorKind::Asset,
        };
        request_effect(rt, HostRequest::RegisterFont(font), waiter);
    }
}

/// The host answered platform effect `id`.
pub(crate) fn resolve_effect(rt: &Runtime, id: u64, result: PlatformResult) {
    if ViewState::of(rt).resolve_foreign_size(id, result.clone()) {
        return;
    }
    let Some(w) = ViewState::of(rt).requests.borrow_mut().effects.remove(&id) else {
        return;
    };
    match w {
        EffectWaiter::Report { owner, at, failed } => {
            let err = match result {
                PlatformResult::Failed(m) => Some(UiError::new(failed, m.to_string(), Some(at))),
                other => platform_result(other)
                    .err()
                    .map(|e| UiError::new(e.kind().clone(), e.message().to_string(), Some(at))),
            };
            if let Some(e) = err {
                rt.report(owner, e);
            }
        }
        EffectWaiter::Future(slot) => slot.complete(Ok(result)),
    }
}

/// The host answered focus request `id`.
pub(crate) fn on_focus_result(rt: &Runtime, id: u64, result: Result<(), String>) {
    let request = {
        let mut requests = ViewState::of(rt).requests.borrow_mut();
        let id = RequestId(id);
        if !requests
            .focus
            .requests
            .get(&id)
            .is_some_and(|request| matches!(request.stage, RequestStage::Sent))
        {
            return;
        }
        requests.focus.requests.remove(&id)
    };
    if let Some(request) = request {
        let result = result.map_err(|reason| UiError::new(ErrorKind::Platform, reason, None));
        resolve_focus_reply(rt, request.reply, FocusResult::Answered(result));
    }
}

/// Send queued focus requests in order, canceling ineligible restores.
pub(crate) fn flush_focus(rt: &Runtime) {
    let order = std::mem::take(&mut ViewState::of(rt).requests.borrow_mut().focus.order);
    let exists = |target: kivact_types::FocusTarget| ViewState::tree(rt).contains(target.node);
    let explicit = {
        let requests = ViewState::of(rt).requests.borrow();
        order.iter().any(|id| {
            requests.focus.requests.get(id).is_some_and(|request| {
                matches!(request.reply, FocusReply::Report(_)) && exists(request.target)
            })
        })
    };
    for id in order {
        let Some(mut request) = ViewState::of(rt)
            .requests
            .borrow_mut()
            .focus
            .requests
            .remove(&id)
        else {
            continue;
        };
        if !matches!(request.stage, RequestStage::Queued) {
            continue;
        }
        let restore = !matches!(request.reply, FocusReply::Report(_));
        let owner_gone = matches!(&request.reply, FocusReply::RestoreAndReport(completion) if !completion.owner.is_alive());
        if restore && (explicit || owner_gone || !can_take_focus(rt, request.target)) {
            resolve_focus_reply(rt, request.reply, FocusResult::Canceled);
            continue;
        }
        if exists(request.target) {
            let target = request.target;
            let visible = request.visible;
            request.stage = RequestStage::Sent;
            ViewState::of(rt)
                .requests
                .borrow_mut()
                .focus
                .requests
                .insert(id, request);
            ViewState::tree(rt).engine().focus(target, id, visible);
            crate::view::devtools_hooks::focus_requested(rt, target);
        } else {
            resolve_focus_reply(
                rt,
                request.reply,
                FocusResult::Answered(Err(UiError::new(
                    ErrorKind::Platform,
                    "focus: node was removed",
                    None,
                ))),
            );
        }
    }
}

/// Whether the engine lets `target` take a restored focus now.
fn can_take_focus(rt: &Runtime, target: kivact_types::FocusTarget) -> bool {
    ViewState::tree(rt).reader().can_take_focus(target)
}

/// Ask the host to give the focus back to `node` (a popup or modal overlay
/// closed: `report` is `None`; `NodeRef::restore_focus`: the ref's owner
/// and the call): written with the next revision if `node` may take the
/// focus back then, dropped silently otherwise.
pub(crate) fn request_restore_focus(
    rt: &Runtime,
    node: kivact_types::FocusTarget,
    report: Option<(Owner, &'static Location<'static>)>,
    visible: bool,
) {
    let reply = report.map_or(FocusReply::Restore, |(owner, at)| {
        FocusReply::RestoreAndReport(FocusCompletion { owner, at })
    });
    request_focus(rt, Some(node), reply, visible);
}

/// Run the default action of `node` (`NodeRef::activate`) once the current
/// handler has returned: the client takes the queue after each event and
/// before each flush of the turn.
pub(crate) fn request_activation(rt: &Runtime, node: kivact_types::FocusTarget) {
    ViewState::of(rt)
        .requests
        .borrow_mut()
        .pending_activations
        .push(node);
    rt.request_turn();
}

/// Take the activations asked for since the last call.
pub(crate) fn take_activations(rt: &Runtime) -> Vec<kivact_types::FocusTarget> {
    std::mem::take(&mut ViewState::of(rt).requests.borrow_mut().pending_activations)
}

/// Bridge an actual engine capture release to the current part router.
pub(crate) fn request_pointer_release(
    rt: &Runtime,
    node: kivact_types::NodeId,
    pointer: kivact_types::PointerId,
) {
    ViewState::of(rt)
        .requests
        .borrow_mut()
        .pointer_releases
        .push((node, pointer));
}

/// Consume releases for this router after all its pointer defaults have run.
pub(crate) fn take_pointer_release(
    rt: &Runtime,
    node: kivact_types::NodeId,
    pointer: kivact_types::PointerId,
) -> bool {
    let mut requests = ViewState::of(rt).requests.borrow_mut();
    let before = requests.pointer_releases.len();
    requests
        .pointer_releases
        .retain(|release| *release != (node, pointer));
    before != requests.pointer_releases.len()
}

/// Plain nodes have no part router; their event facts end with dispatch.
pub(crate) fn clear_pointer_releases(rt: &Runtime) {
    ViewState::of(rt)
        .requests
        .borrow_mut()
        .pointer_releases
        .clear();
}

/// Whether activations wait to run.
pub(crate) fn has_activations(rt: &Runtime) -> bool {
    !ViewState::of(rt)
        .requests
        .borrow()
        .pending_activations
        .is_empty()
}

/// A platform result as `Ok`, or the error it reports.
pub(crate) fn platform_result(r: PlatformResult) -> Result<PlatformResult, UiError> {
    match r {
        PlatformResult::Failed(m) => Err(UiError::new(ErrorKind::Platform, m.to_string(), None)),
        PlatformResult::Unsupported => Err(UiError::new(
            ErrorKind::Unsupported,
            "the host does not support this platform request",
            None,
        )),
        PlatformResult::Cancelled => Err(UiError::new(
            ErrorKind::Cancelled,
            "a newer request replaced this deferred platform request",
            None,
        )),
        PlatformResult::TimedOut => Err(UiError::new(
            ErrorKind::Timeout,
            "the host did not answer this deferred platform request in time",
            None,
        )),
        other => Ok(other),
    }
}
