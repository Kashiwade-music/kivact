//! Role: Part input preserves original targets and caller customization order.
//!
//! A node with parts or canvas regions replaces its key, key release, pointer
//! and wheel lists by one router entry each. The router keeps the node's own
//! entries (callers and `on_*_default`), picks the target of the event (the
//! node, or the canvas region the event carries) and hands the lists of
//! node, target and region fallback to `handler_layers::deliver`.

use std::cell::Cell;
use std::rc::Rc;

use kivact_style::draw::RegionId;

use crate::view::common::{Constraints, Registry, Target};
use crate::view::handler_layers::{EventHandlers, Layer, Source, deliver, open};
use crate::view::tree::{Handlers, PartActions};
use crate::view::{A11yAction, Handled, KeyEvent, WheelEvent};

/// The layers the node's own target takes from the node's list.
const TARGET_LAYERS: [Layer; 3] = [Layer::Observer, Layer::PartDefault, Layer::RegionDefault];

type KeyList = fn(&Handlers) -> &EventHandlers<KeyEvent>;

fn key_list(handlers: &Handlers) -> &EventHandlers<KeyEvent> {
    &handlers.key
}

fn key_up_list(handlers: &Handlers) -> &EventHandlers<KeyEvent> {
    &handlers.key_up
}

fn wheel_list(handlers: &Handlers) -> &EventHandlers<WheelEvent> {
    &handlers.wheel
}

/// Where one event goes: the target whose observers see it and the target
/// whose defaults run.
struct Selected {
    observed: Option<Rc<Target>>,
    target: Option<Rc<Target>>,
    /// The event came from this node's own canvas.
    own_canvas: bool,
}

/// Region identifiers belong to their original canvas. Ancestors receive
/// bubbling through node defaults without retargeting.
fn select(
    registry: &Registry,
    root: &Rc<Target>,
    own_canvas: bool,
    region: Option<RegionId>,
) -> Selected {
    let found = region
        .filter(|_| own_canvas)
        .and_then(|id| registry.lookup(id));
    let observed = match &found {
        Some(target) => Some(target.clone()),
        None if region.is_none() && own_canvas => Some(root.clone()),
        None => None,
    };
    let target = if !own_canvas || region.is_none() {
        Some(root.clone())
    } else {
        found
    };
    Selected {
        observed,
        target,
        own_canvas,
    }
}

/// Deliver one key or wheel event: the observers of the observed target,
/// the node's callers and defaults, the part defaults of the target when it
/// is enabled, then the node's fallback for region input when the target is
/// a region and both it and the node are enabled.
fn deliver_selected<E: Clone + 'static>(
    list: fn(&Handlers) -> &EventHandlers<E>,
    own: &EventHandlers<E>,
    root: &Target,
    selected: &Selected,
    event: &E,
) -> crate::Result<Handled> {
    let target_enabled = Cell::new(None);
    let observing = || {
        selected
            .observed
            .as_ref()
            .is_some_and(|target| target.live())
    };
    let target_ready = || {
        let enabled = selected
            .target
            .as_ref()
            .is_some_and(|target| target.enabled());
        target_enabled.set(Some(enabled));
        enabled
    };
    let fallback_ready = || target_enabled.get() == Some(true) && root.enabled();
    let mut sources = vec![Source::new(own, &Layer::NODE, &open)];
    if let Some(target) = &selected.observed {
        sources.push(Source::new(
            list(&target.handlers),
            &[Layer::Observer],
            &observing,
        ));
    }
    if let Some(target) = &selected.target {
        sources.push(Source::new(
            list(&target.handlers),
            &[Layer::PartDefault],
            &target_ready,
        ));
        if selected.own_canvas && target.region.is_some() {
            sources.push(Source::new(
                list(&root.handlers),
                &[Layer::RegionDefault],
                &fallback_ready,
            ));
        }
    }
    deliver(&sources, event)
}

/// The routed lists of a node's own target, and the node's own lists.
fn split<E: 'static>(own: &EventHandlers<E>) -> (EventHandlers<E>, EventHandlers<E>) {
    (own.only(&Layer::NODE), own.only(&TARGET_LAYERS))
}

/// Wrapping happens after every caller modifier, before refs bind at mount end.
pub(super) fn route(
    node: kivact_wire::NodeId,
    mut caller: Handlers,
    part_actions: PartActions,
    registry: Rc<Registry>,
    constraints: Constraints,
) -> Handlers {
    let (own_key, root_key) = split(&caller.key);
    let (own_key_up, root_key_up) = split(&caller.key_up);
    let (own_pointer, root_pointer) = split(&caller.pointer);
    let (own_wheel, root_wheel) = split(&caller.wheel);
    let root = Rc::new(Target {
        generation: 0,
        region: None,
        owner: registry.owner,
        runtime: registry.runtime.clone(),
        handlers: Handlers {
            key: root_key,
            key_up: root_key_up,
            pointer: root_pointer,
            wheel: root_wheel,
            ..Default::default()
        },
        actions: part_actions,
        bindings: Vec::new(),
        constraints,
    });
    for up in [false, true] {
        let (own, list) = if up {
            (own_key_up.clone(), key_up_list as KeyList)
        } else {
            (own_key.clone(), key_list as KeyList)
        };
        let root = root.clone();
        let registry = registry.clone();
        let router = EventHandlers::router(move |event: KeyEvent| {
            let own_canvas = event.origin.is_some_and(|origin| origin.node == node);
            let selected = select(&registry, &root, own_canvas, event.region);
            deliver_selected(list, &own, &root, &selected, &event)
        });
        if up {
            caller.key_up = router;
        } else {
            caller.key = router;
        }
    }
    caller.pointer = super::part_pointer::route(registry.clone(), root.clone(), own_pointer);
    let wheel_root = root.clone();
    let wheel_registry = registry.clone();
    caller.wheel = EventHandlers::router(move |event: WheelEvent| {
        let selected = select(
            &wheel_registry,
            &wheel_root,
            event.origin.node == node,
            event.region(),
        );
        deliver_selected(wheel_list, &own_wheel, &wheel_root, &selected, &event)
    });
    let custom = caller.text.take();
    let text_root = root.clone();
    if custom.is_some() || text_root.actions.text.is_some() {
        caller.text = Some(Rc::new(move |event| {
            if let Some(custom) = &custom {
                custom(event.clone())?;
            }
            if text_root.enabled()
                && let Some(handler) = &text_root.actions.text
            {
                handler(event)?;
            }
            Ok(())
        }));
    }
    let custom = caller.activate.take();
    let action_root = root.clone();
    let action_registry = registry.clone();
    // Dynamic region actions may appear after mounting; each action dispatches
    // through the current registry, while root advertised actions stay separate.
    caller.activate = Some(Rc::new(move |region| {
        let target = region.and_then(|id| action_registry.lookup(id));
        if let Some(target) = target {
            if target.enabled()
                && let Some(handler) = &target.actions.activate
            {
                handler(region)?;
            }
        } else if region.is_none() && action_root.enabled() {
            if let Some(custom) = &custom {
                custom(None)?;
            }
            if let Some(handler) = &action_root.actions.activate {
                handler(None)?;
            }
        }
        Ok(())
    }));
    let custom = Rc::new(std::mem::take(&mut caller.a11y));
    for action in [
        A11yAction::Focus,
        A11yAction::Increment,
        A11yAction::Decrement,
        A11yAction::Expand,
        A11yAction::Collapse,
        A11yAction::Toggle,
        A11yAction::Select,
        A11yAction::ScrollIntoView,
        A11yAction::SetValue,
        A11yAction::ShowMenu,
        A11yAction::Dismiss,
    ] {
        let custom = custom.clone();
        let root = root.clone();
        let registry = registry.clone();
        caller.a11y.push((
            action,
            Rc::new(move |event| {
                if let Some(id) = event.region {
                    if let Some(target) = registry.lookup(id)
                        && target.enabled()
                        && let Some((_, handler)) = target
                            .actions
                            .a11y
                            .iter()
                            .find(|(candidate, _)| *candidate == action)
                    {
                        handler(event)?;
                    }
                } else if root.enabled() {
                    if let Some((_, handler)) =
                        custom.iter().find(|(candidate, _)| *candidate == action)
                    {
                        handler(event.clone())?;
                    }
                    if let Some((_, handler)) = root
                        .actions
                        .a11y
                        .iter()
                        .find(|(candidate, _)| *candidate == action)
                    {
                        handler(event)?;
                    }
                }
                Ok(())
            }),
        ));
    }
    caller
}
