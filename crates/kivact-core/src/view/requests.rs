//! Role: Event requests and async I/O.
//!
//! Requests the view layer sends to the host and whose answers come back as
//! events: keyboard focus (`NodeRef::focus`, the focus restores after a
//! modal overlay or a popup closes and `NodeRef::restore_focus`),
//! platform effects (`kivact::platform`, the App's fonts), and layout observation
//! requested before a `NodeRef` is bound. Request ids come from one
//! per-App counter.
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

use kivact_wire::{PlatformRequest, PlatformResult, RequestId};

use super::state::{EffectWaiter, FocusWaiter, NodeRefState, ViewState};
use kivact_reactive::{ErrorKind, UiError};
use kivact_reactive::{Owned, Owner, Runtime};
use kivact_style::asset::Asset;

/// Ask the host to focus `node` (written with the next revision); an unmounted
/// node fails at once.
pub(crate) fn request_focus(
    rt: &Runtime,
    node: Option<kivact_wire::FocusTarget>,
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
    req.focus_waiters.insert(id, waiter);
    req.pending_focus.push((node, id, visible));
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
                PlatformResult::Failed(m) => Some(UiError::new(failed, m, Some(at))),
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
pub(crate) fn on_focus_result(rt: &Runtime, id: u64, ok: bool, reason: Option<String>) {
    let w = ViewState::of(rt)
        .requests
        .borrow_mut()
        .focus_waiters
        .remove(&id);
    if let Some(w) = w {
        let r = if ok {
            Ok(())
        } else {
            Err(UiError::new(
                ErrorKind::Platform,
                reason.unwrap_or_else(|| "focus request failed".into()),
                None,
            ))
        };
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
    let pending = std::mem::take(&mut ViewState::of(rt).requests.borrow_mut().pending_focus);
    let is_live =
        |node: kivact_wire::FocusTarget| ViewState::tree(rt).get(node.node).is_some_and(|n| n.live);
    let is_restore = |w: Option<&FocusWaiter>| matches!(w, Some(FocusWaiter::Restore { .. }));
    let explicit = {
        let r = ViewState::of(rt).requests.borrow();
        pending
            .iter()
            .any(|(node, id, _)| !is_restore(r.focus_waiters.get(id)) && is_live(*node))
    };
    for (node, id, visible) in pending {
        let cancelled = match ViewState::of(rt).requests.borrow().focus_waiters.get(&id) {
            Some(FocusWaiter::Restore { report }) => {
                explicit
                    || report.is_some_and(|(owner, _)| !owner.is_alive())
                    || !can_take_focus(rt, node)
            }
            _ => false,
        };
        if cancelled {
            ViewState::of(rt)
                .requests
                .borrow_mut()
                .focus_waiters
                .remove(&id);
            continue;
        }
        let live = is_live(node);
        if live {
            ViewState::tree(rt)
                .engine()
                .focus(node, RequestId(id), visible);
        } else {
            let w = ViewState::of(rt)
                .requests
                .borrow_mut()
                .focus_waiters
                .remove(&id);
            if let Some(w) = w {
                resolve_focus_waiter(
                    rt,
                    w,
                    Err(UiError::new(
                        ErrorKind::Platform,
                        "focus: node was removed",
                        None,
                    )),
                );
            }
        }
    }
}

/// Whether the engine lets `target` take a restored focus now. Before the
/// App started there is no engine and nothing to restore to.
fn can_take_focus(rt: &Runtime, target: kivact_wire::FocusTarget) -> bool {
    ViewState::tree(rt)
        .reader()
        .is_some_and(|reader| reader.can_take_focus(target))
}

/// Ask the host to give the focus back to `node` (a popup or modal overlay
/// closed: `report` is `None`; `NodeRef::restore_focus`: the ref's owner
/// and the call): written with the next revision if `node` may take the
/// focus back then, dropped silently otherwise.
pub(crate) fn request_restore_focus(
    rt: &Runtime,
    node: kivact_wire::FocusTarget,
    report: Option<(Owner, &'static Location<'static>)>,
    visible: bool,
) {
    request_focus(rt, Some(node), FocusWaiter::Restore { report }, visible);
}

/// Run the default action of `node` (`NodeRef::activate`) once the current
/// handler has returned: the client takes the queue after each event and
/// before each flush of the turn.
pub(crate) fn request_activation(rt: &Runtime, node: kivact_wire::FocusTarget) {
    ViewState::of(rt)
        .requests
        .borrow_mut()
        .pending_activations
        .push(node);
    rt.request_turn();
}

/// Take the activations asked for since the last call.
pub(crate) fn take_activations(rt: &Runtime) -> Vec<kivact_wire::FocusTarget> {
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

/// Observe the node of NodeRef `key` as soon as it is bound.
pub(crate) fn observe_on_bind(rt: &Runtime, key: Owned<NodeRefState>) {
    ViewState::of(rt)
        .requests
        .borrow_mut()
        .observe_on_bind
        .push(key);
}

/// Take the pending observe request of NodeRef `key`, if any.
pub(crate) fn take_observe_on_bind(rt: &Runtime, key: Owned<NodeRefState>) -> bool {
    let mut r = ViewState::of(rt).requests.borrow_mut();
    match r.observe_on_bind.iter().position(|k| *k == key) {
        Some(p) => {
            r.observe_on_bind.swap_remove(p);
            true
        }
        None => false,
    }
}

/// Drop the pending observe request of a disposed NodeRef `key`.
pub(crate) fn forget_observe_on_bind(rt: &Runtime, key: Owned<NodeRefState>) {
    ViewState::of(rt)
        .requests
        .borrow_mut()
        .observe_on_bind
        .retain(|x| *x != key);
}

/// A platform result as `Ok`, or the error it reports.
pub(crate) fn platform_result(r: PlatformResult) -> Result<PlatformResult, UiError> {
    match r {
        PlatformResult::Failed(m) => Err(UiError::new(ErrorKind::Platform, m, None)),
        PlatformResult::Unsupported => Err(UiError::new(
            ErrorKind::Unsupported,
            "the host does not support this platform request",
            None,
        )),
        other => Ok(other),
    }
}
