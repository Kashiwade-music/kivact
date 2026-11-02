//! Role: Pointer routing retains adopted part identity while native capture runs.
//!
//! Each pointer has its own capture of the part it pressed (the engine
//! captures per pointer too), so two fingers on one node each keep their
//! own target. The part hover (`Enter`/`Leave` of a region) follows the
//! primary pointers only, like the engine's hover.

use std::rc::Rc;

use crate::view::common::{Capture, Gestures, PrimaryHover, Registry, Target, TargetId};
use crate::view::handler_layers::{EventHandlers, Layer, Source, deliver};
use crate::view::state::ViewState;
use crate::view::{Handled, MouseButton, PointerButtons, PointerEvent, PointerId, PointerPhase};

impl Gestures {
    fn capture(&mut self, pointer: PointerId) -> Option<&mut Capture> {
        self.captures.iter_mut().find(|c| c.pointer == pointer)
    }
}

fn event_for(mut event: PointerEvent, target: &Target, phase: PointerPhase) -> PointerEvent {
    event.phase = phase;
    event.origin.region = target.region();
    if matches!(
        phase,
        PointerPhase::Enter | PointerPhase::Leave | PointerPhase::Cancel
    ) {
        event.button = None;
    }
    if phase == PointerPhase::Cancel {
        event.buttons = PointerButtons::NONE;
    }
    event
}

/// The state observations of `target`, while it is alive. They cannot fail
/// or consume.
fn observe(target: &Target, event: PointerEvent) {
    let live = || target.live();
    let observed = Source::new(&target.handlers.pointer, &[Layer::Observer], &live);
    let _ = deliver(&[observed], &event);
}

/// The defaults of the parts attached to `target`. A disabled target still
/// gets the events that end a hover or a gesture.
fn default(target: &Target, event: PointerEvent) -> crate::Result<Handled> {
    let admit = || {
        target.live()
            && (target.enabled()
                || matches!(event.phase, PointerPhase::Cancel | PointerPhase::Leave))
    };
    let defaults = Source::new(&target.handlers.pointer, &[Layer::PartDefault], &admit);
    deliver(&[defaults], &event)
}

/// The node's fallback for region input.
fn fallback(root: &Target, event: PointerEvent) -> crate::Result<Handled> {
    let admit = || root.live() && (root.enabled() || event.phase == PointerPhase::Cancel);
    let fallback = Source::new(&root.handlers.pointer, &[Layer::RegionDefault], &admit);
    deliver(&[fallback], &event)
}

/// End the gestures that target `id`: its hover, and the captures it holds.
pub(super) fn cancel(registry: &Registry, id: TargetId) {
    let root = &*registry.root;
    let of_id =
        |target: &Option<Rc<Target>>| target.as_ref().is_some_and(|target| target.id == id);
    let (captured, hovered, last) = {
        let mut gestures = registry.gestures.borrow_mut();
        let mut captured = Vec::new();
        for capture in &mut gestures.captures {
            if of_id(&capture.target) {
                let target = capture
                    .target
                    .take()
                    .expect("invariant: a target of the id was checked");
                let ends_fallback = capture.fallback.take().is_some();
                captured.push((target, capture.last, capture.pointer, ends_fallback));
            }
        }
        let hovered = match &mut gestures.hover {
            PrimaryHover::Inside(target) if of_id(target) => target.take(),
            _ => None,
        };
        (captured, hovered, gestures.last)
    };
    let report = |target: &Target, result: crate::Result<Handled>| {
        if let Err(error) = result
            && let Some(runtime) = target.runtime.upgrade()
        {
            runtime.report(Some(target.owner), error);
        }
    };
    if let (Some(target), Some(event)) = (hovered, last) {
        let event = event_for(event, &target, PointerPhase::Leave);
        observe(&target, event);
        report(&target, default(&target, event));
    }
    for (target, event, pointer, cancel_fallback) in captured {
        let event = event_for(event, &target, PointerPhase::Cancel);
        observe(&target, event);
        report(&target, default(&target, event));
        if cancel_fallback {
            report(root, fallback(root, event));
            // The gesture ended without a release: its click activates
            // whatever the pointer presses next.
            forget_adoption(registry, pointer);
        }
    }
}

/// The pointer's gesture no longer holds its adoption of the click.
fn forget_adoption(registry: &Registry, pointer: PointerId) {
    if let Some(runtime) = registry.root.runtime.upgrade() {
        ViewState::tree(&runtime)
            .engine()
            .forget_adopted_pointer(pointer);
    }
}

/// Caller callbacks retain native hit identity; part defaults get gesture identity.
/// `own` holds the node's callers and `on_pointer_default` handlers, which see
/// every pointer event that reaches the node. Each logical event (a hover
/// transition, then the raw event) is delivered to the observers and then to
/// the defaults of its own target; only the raw event's answer decides whether
/// the node's fallback for region input runs.
pub(super) fn route(
    registry: Rc<Registry>,
    own: EventHandlers<PointerEvent>,
) -> EventHandlers<PointerEvent> {
    EventHandlers::router(move |event: PointerEvent| {
        let root = &registry.root;
        let mut transitions = Vec::new();
        let (target, deliver_raw, finish_fallback) = {
            let mut gestures = registry.gestures.borrow_mut();
            if event.origin.node != registry.node {
                (None, false, false)
            } else {
                let finish_fallback = gestures.capture(event.id).is_some_and(|capture| {
                    capture.fallback.is_some_and(|button| {
                        event.phase == PointerPhase::Cancel
                            || (event.phase == PointerPhase::Up && event.button == Some(button))
                    })
                });
                if finish_fallback && let Some(capture) = gestures.capture(event.id) {
                    capture.fallback = None;
                }
                let captured = gestures.capture(event.id).is_some();
                if let Some(capture) = gestures.capture(event.id) {
                    capture.last = event;
                }
                let under = |inside: bool| -> Option<Rc<Target>> {
                    if !inside {
                        return None;
                    }
                    match event.region() {
                        Some(id) => registry.lookup(id).filter(|target| target.enabled()),
                        None => root.enabled().then(|| root.clone()),
                    }
                };
                // A pointer that is not primary hovers no part.
                let hovered = if event.is_primary {
                    gestures.last = Some(event);
                    let was_inside = matches!(gestures.hover, PrimaryHover::Inside(_));
                    let inside = match event.phase {
                        PointerPhase::Enter => true,
                        PointerPhase::Leave | PointerPhase::Cancel => false,
                        PointerPhase::Move | PointerPhase::Down if !captured => true,
                        _ => was_inside,
                    };
                    let hovered = under(inside);
                    let previous = match std::mem::take(&mut gestures.hover) {
                        PrimaryHover::Inside(target) => target,
                        PrimaryHover::Outside => None,
                    };
                    if previous.as_ref().map(|target| target.id)
                        != hovered.as_ref().map(|target| target.id)
                    {
                        if let Some(previous) = previous {
                            transitions.push((previous, PointerPhase::Leave));
                        }
                        if let Some(next) = &hovered {
                            transitions.push((next.clone(), PointerPhase::Enter));
                        }
                    }
                    gestures.hover = if inside {
                        PrimaryHover::Inside(hovered.clone())
                    } else {
                        PrimaryHover::Outside
                    };
                    hovered
                } else {
                    under(matches!(
                        event.phase,
                        PointerPhase::Down | PointerPhase::Move
                    ))
                };
                if event.phase == PointerPhase::Down
                    && let Some(button) = event.button
                {
                    if let Some(capture) = gestures.capture(event.id) {
                        if !capture.buttons.contains(&button) {
                            capture.buttons.push(button);
                        }
                    } else {
                        gestures.captures.push(Capture {
                            pointer: event.id,
                            target: hovered.clone(),
                            buttons: vec![button],
                            last: event,
                            fallback: None,
                        });
                    }
                }
                let target = match gestures.capture(event.id) {
                    Some(capture) => capture.target.clone(),
                    None => hovered.clone(),
                };
                if event.phase == PointerPhase::Up
                    && let Some(capture) = gestures.capture(event.id)
                {
                    if let Some(button) = event.button {
                        capture.buttons.retain(|held| *held != button);
                    }
                    if capture.buttons.is_empty() {
                        gestures.captures.retain(|c| c.pointer != event.id);
                    }
                } else if event.phase == PointerPhase::Cancel {
                    gestures.captures.retain(|c| c.pointer != event.id);
                }
                (
                    target,
                    !matches!(event.phase, PointerPhase::Enter | PointerPhase::Leave),
                    finish_fallback,
                )
            }
        };
        let mut delivered_finish = false;
        let result = (|| {
            // State always sees both logical hover transitions and gesture updates,
            // even when a caller consumes the original native event or returns an error.
            for (target, phase) in &transitions {
                observe(target, event_for(event, target, *phase));
            }
            if deliver_raw && let Some(target) = &target {
                observe(target, event_for(event, target, event.phase));
            }
            let mut handled = own.deliver(event)?;
            if handled == Handled::Yes {
                return Ok(Handled::Yes);
            }
            for (target, phase) in transitions {
                handled = handled.and(default(&target, event_for(event, &target, phase))?);
            }
            let raw_handled = if deliver_raw && let Some(target) = &target {
                default(target, event_for(event, target, event.phase))?
            } else {
                Handled::No
            };
            handled = handled.and(raw_handled);
            // Only explicitly opted-in defaults receive unhandled region input
            // (a prevented default still continues to them).
            if raw_handled != Handled::Yes
                && deliver_raw
                && event.origin.node == registry.node
                && target
                    .as_ref()
                    .is_some_and(|target| target.region().is_some() && target.enabled())
            {
                let target = target
                    .as_ref()
                    .expect("invariant: a live region target was checked");
                let event = event_for(event, target, event.phase);
                delivered_finish = finish_fallback;
                let result = fallback(root, event)?;
                handled = handled.and(result);
                if result == Handled::Yes {
                    if event.phase == PointerPhase::Down
                        && let Some(button) = event.button
                    {
                        registry
                            .gestures
                            .borrow_mut()
                            .capture(event.id)
                            .expect("invariant: a press creates its pointer's capture")
                            .fallback = Some(button);
                        // Only a primary pointer's press can activate, so
                        // only its gesture declines the activation. The
                        // engine remembers it until the click is answered.
                        if button == MouseButton::Primary
                            && event.is_primary
                            && let Some(region) = target.region()
                            && let Some(runtime) = root.runtime.upgrade()
                        {
                            ViewState::tree(&runtime).engine().adopt_pointer(
                                registry.node,
                                region,
                                event.id,
                            );
                        }
                    }
                    return Ok(Handled::Yes);
                }
            }
            Ok(handled)
        })();
        // Consuming a terminal event cannot leave an adopted fallback gesture open.
        if finish_fallback && (!delivered_finish || result.is_err()) {
            let mut cancelled = event;
            cancelled.phase = PointerPhase::Cancel;
            cancelled.button = None;
            cancelled.buttons = PointerButtons::NONE;
            if let Some(target) = &target {
                cancelled = event_for(cancelled, target, PointerPhase::Cancel);
            }
            if let Err(error) = fallback(root, cancelled)
                && let Some(runtime) = root.runtime.upgrade()
            {
                runtime.report(Some(root.owner), error);
            }
        }
        result
    })
}
