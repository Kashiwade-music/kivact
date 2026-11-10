//! Role: Part input preserves original targets and caller customization order.
//!
//! A node with parts or canvas regions replaces its key, key release, pointer
//! and wheel lists by one router entry each. The router keeps the node's own
//! entries (callers and `on_*_default`), picks the target of the event (the
//! node, or the canvas region the event carries) and hands the lists of
//! node, target and region fallback to `handler_layers::deliver`.

use std::rc::Rc;

use kivact_style::draw::RegionId;

use crate::view::descriptor::Mcx;
use crate::view::element_data::{Constraints, Registry, Target, TargetId};
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
    // xtask-allow(state-fields): independent: a per-event routing value; the event sets each from its hit
    observed: Option<Rc<Target>>,
    // xtask-allow(state-fields): independent: a per-event routing value; the event sets each from its hit
    target: Option<Rc<Target>>,
    // xtask-allow(state-fields): independent: a per-event routing value; any combination with the targets is valid
    /// The event came from this node's own canvas.
    own_canvas: bool,
}

/// Region identifiers belong to their original canvas. Ancestors receive
/// bubbling through node defaults without retargeting.
fn select(registry: &Registry, own_canvas: bool, region: Option<RegionId>) -> Selected {
    let found = region
        .filter(|_| own_canvas)
        .and_then(|id| registry.lookup(id));
    let observed = match &found {
        Some(target) => Some(target.clone()),
        None if region.is_none() && own_canvas => Some(registry.root.clone()),
        None => None,
    };
    let target = if !own_canvas || region.is_none() {
        Some(registry.root.clone())
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
    let observing = || {
        selected
            .observed
            .as_ref()
            .is_some_and(|target| target.live())
    };
    let target_ready = || {
        selected
            .target
            .as_ref()
            .is_some_and(|target| target.enabled())
    };
    let fallback_ready = || target_ready() && root.enabled();
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
        if selected.own_canvas && target.region().is_some() {
            sources.push(Source::new(
                list(&root.handlers),
                &[Layer::RegionDefault],
                &fallback_ready,
            ));
        }
    }
    deliver(&sources, event)
}

/// The node's own target: its share of the node's lists (observers, part defaults and the region
/// fallback) and the parts' action defaults.
pub(super) fn root_target(
    handlers: &Handlers,
    actions: PartActions,
    constraints: Constraints,
    cx: &Mcx<'_>,
) -> Rc<Target> {
    Rc::new(Target {
        id: TargetId::Root,
        owner: cx.owner,
        runtime: cx.weak(),
        handlers: Handlers {
            key: handlers.key.only(&TARGET_LAYERS),
            key_up: handlers.key_up.only(&TARGET_LAYERS),
            pointer: handlers.pointer.only(&TARGET_LAYERS),
            wheel: handlers.wheel.only(&TARGET_LAYERS),
            ..Default::default()
        },
        actions,
        bindings: Vec::new(),
        constraints,
    })
}

/// Wrapping happens after every caller modifier, before refs bind at mount end.
pub(super) fn route(
    node: kivact_types::NodeId,
    mut caller: Handlers,
    registry: Rc<Registry>,
) -> Handlers {
    let own_key = caller.key.only(&Layer::NODE);
    let own_key_up = caller.key_up.only(&Layer::NODE);
    let own_pointer = caller.pointer.only(&Layer::NODE);
    let own_wheel = caller.wheel.only(&Layer::NODE);
    for up in [false, true] {
        let (own, list) = if up {
            (own_key_up.clone(), key_up_list as KeyList)
        } else {
            (own_key.clone(), key_list as KeyList)
        };
        let registry = registry.clone();
        let router = EventHandlers::router(move |event: KeyEvent| {
            let own_canvas = event.origin().is_some_and(|origin| origin.node == node);
            let selected = select(&registry, own_canvas, event.region());
            deliver_selected(list, &own, &registry.root, &selected, &event)
        });
        if up {
            caller.key_up = router;
        } else {
            caller.key = router;
        }
    }
    caller.pointer = super::part_pointer::route(registry.clone(), own_pointer);
    let wheel_registry = registry.clone();
    caller.wheel = EventHandlers::router(move |event: WheelEvent| {
        let selected = select(&wheel_registry, event.origin().node == node, event.region());
        deliver_selected(
            wheel_list,
            &own_wheel,
            &wheel_registry.root,
            &selected,
            &event,
        )
    });
    route_actions(&mut caller, &registry);
    route_accessibility(&mut caller, &registry);
    caller
}

fn route_actions(caller: &mut Handlers, registry: &Rc<Registry>) {
    let custom = caller.text.take();
    let text_root = registry.root.clone();
    if custom.is_some() || text_root.actions.text.is_some() {
        caller.text = Some(kivact_runtime::Handler::from_fn(
            move |event: crate::view::editing_data::TextEditEvent| {
                if let Some(custom) = &custom {
                    custom.call(event.clone())?;
                }
                if text_root.enabled()
                    && let Some(handler) = &text_root.actions.text
                {
                    handler.call(event)?;
                }
                Ok(())
            },
        ));
    }
    let custom = caller.activate.take();
    let action_root = registry.root.clone();
    let action_registry = registry.clone();
    // Dynamic region actions may appear after mounting; each action dispatches
    // through the current registry, while root advertised actions stay separate.
    caller.activate = Some(kivact_runtime::Handler::from_fn(
        move |region: Option<kivact_types::RegionId>| {
            let target = region.and_then(|id| action_registry.lookup(id));
            if let Some(target) = target {
                if target.enabled()
                    && let Some(handler) = &target.actions.activate
                {
                    handler.call(region)?;
                }
            } else if region.is_none() && action_root.enabled() {
                if let Some(custom) = &custom {
                    custom.call(None)?;
                }
                if let Some(handler) = &action_root.actions.activate {
                    handler.call(None)?;
                }
            }
            Ok(())
        },
    ));
}

fn route_accessibility(caller: &mut Handlers, registry: &Rc<Registry>) {
    let custom = Rc::new(std::mem::take(&mut caller.a11y));
    let declares = registry.declares_regions();
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
        // Without regions only the node's own handlers can run: route no other action.
        let offered = custom.iter().any(|(candidate, _)| *candidate == action)
            || registry
                .root
                .actions
                .a11y
                .iter()
                .any(|(candidate, _)| *candidate == action);
        if !declares && !offered {
            continue;
        }
        let custom = custom.clone();
        let root = registry.root.clone();
        let registry = registry.clone();
        caller.a11y.push((
            action,
            kivact_runtime::Handler::from_fn(move |event: crate::view::A11yEvent| {
                if let Some(id) = event.region {
                    if let Some(target) = registry.lookup(id)
                        && target.enabled()
                        && let Some((_, handler)) = target
                            .actions
                            .a11y
                            .iter()
                            .find(|(candidate, _)| *candidate == action)
                    {
                        handler.call(event)?;
                    }
                } else if root.enabled() {
                    if let Some((_, handler)) =
                        custom.iter().find(|(candidate, _)| *candidate == action)
                    {
                        handler.call(event.clone())?;
                    }
                    if let Some((_, handler)) = root
                        .actions
                        .a11y
                        .iter()
                        .find(|(candidate, _)| *candidate == action)
                    {
                        handler.call(event)?;
                    }
                }
                Ok(())
            }),
        ));
    }
}
