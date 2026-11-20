//! Role: Event requests and async I/O.
//!
//! Requests the view layer sends to the host and whose answers come back as
//! events: keyboard focus (`NodeRef::focus`, the focus restores after a
//! modal overlay or a popup closes and `NodeRef::restore_focus`),
//! platform effects (`kivact::platform`, the theme's fonts). Request ids come
//! from one per-App counter.
//!
//! Restores are checked once at flush against engine focus eligibility.
//! Cancellation produces no host write or error report. Activations run after
//! the current handler returns, through the host activation path.

use std::panic::Location;

use kivact_engine::contract::HostRequest;
use kivact_types::{PlatformResult, RequestId};

use super::state::{FocusCompletion, FocusReply, FocusRequest, ViewState, Waiter};
use kivact_runtime::Owner;
use kivact_runtime::driver::Runtime;
use kivact_runtime::{ErrorKind, UiError};
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
    req.focus.queued.push((
        id,
        FocusRequest {
            target: node,
            reply,
            visible,
        },
    ));
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
pub(crate) fn request_effect(rt: &Runtime, req: HostRequest, waiter: Waiter) {
    let id = {
        let mut r = ViewState::of(rt).requests.borrow_mut();
        let id = r.request_id();
        r.waiters.insert(id, waiter);
        id
    };
    ViewState::tree(rt).engine().effect(RequestId(id), req);
    rt.request_turn();
}

/// Register the fonts of `theme` the App has not registered yet (by asset
/// identity: the path, the static address and length, or the shared
/// allocation), located at `at`. The first layout after this call sees
/// them (the engine registers a font when it is written). A file that is
/// not a usable font is reported as `ErrorKind::Asset` at `at`.
pub(crate) fn register_theme_fonts(
    rt: &Runtime,
    theme: &kivact_style::Theme,
    at: &'static Location<'static>,
) {
    let fonts: Vec<Asset> = {
        let mut seen = ViewState::of(rt).registered_fonts.borrow_mut();
        theme
            .fonts()
            .iter()
            .filter(|asset| seen.insert((*asset).clone()))
            .cloned()
            .collect()
    };
    for font in fonts {
        let waiter = Waiter::Report {
            owner: None,
            at,
            failed: ErrorKind::Asset,
        };
        request_effect(rt, HostRequest::RegisterFont(font), waiter);
    }
}

/// The host answered platform effect `id`.
pub(crate) fn resolve_effect(rt: &Runtime, id: u64, result: PlatformResult) {
    let Some(waiter) = ViewState::of(rt).requests.borrow_mut().waiters.remove(&id) else {
        return;
    };
    match waiter {
        Waiter::Report { owner, at, failed } => {
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
        Waiter::Future(slot) => slot.complete(Ok(result)),
        Waiter::ForeignSize { at, slot, .. } => slot.complete(foreign_size_outcome(result, at)),
        Waiter::Snapshot { .. } => {
            unreachable!("invariant: a snapshot is answered by SnapshotResult, not EffectResult")
        }
    }
}

/// The outcome of a foreign window's size request from the host's answer.
fn foreign_size_outcome(
    result: PlatformResult,
    at: &'static Location<'static>,
) -> Result<(), UiError> {
    match result {
        PlatformResult::Failed(message) => Err(UiError::new(
            ErrorKind::Platform,
            message.to_string(),
            Some(at),
        )),
        PlatformResult::Unsupported => Err(UiError::new(
            ErrorKind::Unsupported,
            "the host does not support this platform request",
            Some(at),
        )),
        PlatformResult::Cancelled => Err(UiError::new(
            ErrorKind::Cancelled,
            "a newer request replaced this deferred platform request",
            Some(at),
        )),
        PlatformResult::TimedOut => Err(UiError::new(
            ErrorKind::Timeout,
            "the host did not answer this deferred platform request in time",
            Some(at),
        )),
        _ => Ok(()),
    }
}

/// The host answered focus request `id`.
pub(crate) fn on_focus_result(rt: &Runtime, id: u64, result: Result<(), String>) {
    let reply = ViewState::of(rt)
        .requests
        .borrow_mut()
        .focus
        .sent
        .remove(&RequestId(id));
    if let Some(reply) = reply {
        let result = result.map_err(|reason| UiError::new(ErrorKind::Platform, reason, None));
        resolve_focus_reply(rt, reply, FocusResult::Answered(result));
    }
}

/// Send queued focus requests in order, canceling ineligible restores.
pub(crate) fn flush_focus(rt: &Runtime) {
    let queued = std::mem::take(&mut ViewState::of(rt).requests.borrow_mut().focus.queued);
    let exists = |target: kivact_types::FocusTarget| ViewState::tree(rt).contains(target.node);
    let explicit = queued.iter().any(|(_, request)| {
        matches!(request.reply, FocusReply::Report(_)) && exists(request.target)
    });
    for (id, request) in queued {
        let restore = !matches!(request.reply, FocusReply::Report(_));
        let owner_gone = matches!(&request.reply, FocusReply::RestoreAndReport(completion) if !completion.owner.is_alive());
        if restore && (explicit || owner_gone || !can_take_focus(rt, request.target)) {
            resolve_focus_reply(rt, request.reply, FocusResult::Canceled);
            continue;
        }
        if exists(request.target) {
            let FocusRequest {
                target,
                reply,
                visible,
            } = request;
            ViewState::of(rt)
                .requests
                .borrow_mut()
                .focus
                .sent
                .insert(id, reply);
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
