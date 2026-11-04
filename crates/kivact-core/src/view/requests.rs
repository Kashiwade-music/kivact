//! Role: Event requests and async I/O.
//!
//! Requests the view layer sends to the host and whose answers come back as
//! events: keyboard focus (`NodeRef::focus`, the focus restores after a
//! modal overlay or a popup closes and `NodeRef::restore_focus`),
//! platform effects (`kivact::platform`, the App's fonts). Request ids come
//! from one per-App counter.
//!
//! Invariants:
//! - Every focus restore (a modal overlay or a popup closing,
//!   `NodeRef::restore_focus`, the focus step of `NodeRef::activate`) is one
//!   `FocusWaiter::Restore { report }` queued here and checked once, at the
//!   flush, by the engine's `Reader::can_take_focus` alone: mounted and live,
//!   neither it nor an ancestor hidden (`Hidden` or a hidden layout) or
//!   leaving, focusable (a native text editor too), not disabled, inside
//!   the topmost modal that stays open. A restore that fails it, one that an explicit live focus
//!   request of the same flush beats, and one whose reporting owner is gone
//!   are cancelled without a write or an answer.
//! - `report` is `None` for the core's own restores and the ref's owner
//!   and call site for `NodeRef::restore_focus`; only a request the host
//!   still refuses after it was sent reaches the error sink, located there.
//!   An unbound or disposed ref restores nothing.
//! - Components use this path and never check the liveness, visibility or
//!   disabled state of their anchor themselves.
//! - An activation (`NodeRef::activate`) waits in a queue the client takes
//!   after each event and before each flush of the turn, and runs through
//!   the same activation as a host `Activate` event, after the handler
//!   that asked for it returned.

use std::panic::Location;

use kivact_types::{PlatformRequest, PlatformResult, RequestId};

use super::state::{EffectWaiter, FocusWaiter, QueuedFocus, ViewState};
use kivact_runtime::{ErrorKind, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_style::asset::Asset;

/// Ask the host to focus `node` (written with the next revision); an unmounted
/// node fails at once.
pub(crate) fn request_focus(
    rt: &Runtime,
    node: Option<kivact_types::FocusTarget>,
    waiter: FocusWaiter,
    visible: bool,
) {
    let Some(node) = node else {
        let err = UiError::new(ErrorKind::Platform, "focus: node is not mounted", None);
        resolve_focus_waiter(rt, waiter, Err(err));
        return;
    };
    let mut req = ViewState::of(rt).requests.borrow_mut();
    let id = req.request_id();
    req.queued_focus.push(QueuedFocus {
        id,
        target: node,
        visible,
        waiter,
    });
    drop(req);
    rt.request_turn();
}

fn resolve_focus_waiter(rt: &Runtime, waiter: FocusWaiter, result: Result<(), UiError>) {
    match waiter {
        FocusWaiter::Report(owner, at)
        | FocusWaiter::Restore {
            report: Some((owner, at)),
        } => {
            if let Err(e) = result {
                let e = UiError::new(e.kind().clone(), e.message().to_string(), Some(at));
                rt.report(Some(owner), e);
            }
        }
        FocusWaiter::Restore { report: None } => {}
    }
}

/// Send a platform effect to the host (a write of the next revision).
pub(crate) fn request_effect(rt: &Runtime, req: PlatformRequest, waiter: EffectWaiter) {
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
        request_effect(rt, PlatformRequest::RegisterFont(font), waiter);
    }
}

/// The host answered platform effect `id`.
pub(crate) fn resolve_effect(rt: &Runtime, id: u64, result: PlatformResult) {
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
    let w = ViewState::of(rt)
        .requests
        .borrow_mut()
        .sent_focus
        .remove(&id);
    if let Some(w) = w {
        let r = result.map_err(|reason| UiError::new(ErrorKind::Platform, reason, None));
        resolve_focus_waiter(rt, w, r);
    }
}

/// Record Focus writes for requests whose node exists; the others fail.
///
/// An explicit request (`NodeRef::focus`) wins over a restore queued in the
/// same flush (a popup or modal overlay closed in the same turn, or
/// `NodeRef::restore_focus`), and a restore whose target may not take the
/// focus back now (`Reader::can_take_focus`) or whose reporting owner is
/// gone is cancelled: such restores are dropped without a write or an
/// answer, so the host does not move the focus away from the node the App
/// asked for, nor to a node that left.
pub(crate) fn flush_focus(rt: &Runtime) {
    let queued = std::mem::take(&mut ViewState::of(rt).requests.borrow_mut().queued_focus);
    let exists = |target: kivact_types::FocusTarget| ViewState::tree(rt).contains(target.node);
    let explicit = queued
        .iter()
        .any(|q| matches!(q.waiter, FocusWaiter::Report(..)) && exists(q.target));
    for QueuedFocus {
        id,
        target,
        visible,
        waiter,
    } in queued
    {
        if let FocusWaiter::Restore { report } = &waiter
            && (explicit
                || report.is_some_and(|(owner, _)| !owner.is_alive())
                || !can_take_focus(rt, target))
        {
            continue; // cancelled: dropped without a write or an answer
        }
        if exists(target) {
            ViewState::of(rt)
                .requests
                .borrow_mut()
                .sent_focus
                .insert(id, waiter);
            ViewState::tree(rt)
                .engine()
                .focus(target, RequestId(id), visible);
        } else {
            resolve_focus_waiter(
                rt,
                waiter,
                Err(UiError::new(
                    ErrorKind::Platform,
                    "focus: node was removed",
                    None,
                )),
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
    request_focus(rt, Some(node), FocusWaiter::Restore { report }, visible);
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
        other => Ok(other),
    }
}
