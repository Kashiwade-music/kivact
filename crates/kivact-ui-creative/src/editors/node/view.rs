//! Role: The node editor component and the node canvas view it builds.

use super::a11y::{semantic, targets};
use crate::editors::node::gesture::{ConnectGuard, State};
use crate::editors::node::gesture::{ControlKind, Target};
use crate::editors::node::input::{key, pointer};
use crate::editors::node::model::{NodeData, NodeLink, NodeRequest};
use crate::editors::shared::model::EditorEvent;
use crate::editors::shared::selection;
use crate::editors::shared::targets::{self};
use kivact::component;
use kivact::custom::NodeRefExt;
use kivact::custom::part;
use kivact::interaction::ListItem;
use kivact::interaction::{DragCoordinates, EditSession, use_drag, use_zoom_pan};
use kivact::state::{Callback, Value, memo, signal};
use kivact::state::{IntoValue, callback};
use kivact::style::{Overflow, style};
use kivact::view::EditPhase;
use kivact::view::{A11yAction, A11yRequest, Handled, Role, node_ref};
use kivact::view::{Node, node};
use std::rc::Rc;

/// An editor for typed nodes, sockets, controls and links.
///
/// Display application-owned nodes and links without mutating either collection.
/// Apply proposals from `on_edit` to make edits visible. `state` holds the
/// selected nodes, the selected link and the canvas's pan and zoom.
/// `on_edit` reports absolute proposals and gesture phases; the application
/// applies requests and restores its own items on Cancel, and pan/zoom forwards
/// shared phases. `can_connect` guards connection highlighting and Connect
/// reports; it defaults to accepting all links. `snap` rounds the distance a
/// drag moves the selected nodes to multiples of 10 px on both axes, off by default.
#[component]
pub fn node_editor(
    nodes: Value<Vec<NodeData>>,
    links: Value<Vec<NodeLink>>,
    state: crate::NodeState,
    #[prop(default = false)] snap: Value<bool>,
    can_connect: Option<Rc<dyn Fn(&NodeLink) -> bool>>,
    on_edit: Option<Callback<EditorEvent<NodeRequest>>>,
) -> impl kivact::view::Element {
    let can_connect: ConnectGuard = match can_connect {
        Some(guard) => signal(guard).into_value(),
        None => signal(Rc::new(|_: &NodeLink| true) as Rc<dyn Fn(&NodeLink) -> bool>).into_value(),
    };
    NodeCanvas {
        nodes,
        links,
        state,
        snap,
        can_connect,
        edit: on_edit,
    }
    .into_node()
}

/// The settings a component collects, which build the node canvas.
struct NodeCanvas {
    nodes: Value<Vec<NodeData>>,
    links: Value<Vec<NodeLink>>,
    state: crate::NodeState,
    snap: Value<bool>,
    can_connect: ConnectGuard,
    edit: Option<Callback<EditorEvent<NodeRequest>>>,
}

/// The focusable canvas that the nodes, the pointer and the keys attach to.
fn canvas_root() -> Node {
    node().role(Role::Canvas).focusable(true).style(
        style()
            .height(280)
            .overflow(Overflow::Hidden)
            .background(crate::color::ISLAND),
    )
}

impl NodeCanvas {
    fn into_node(self) -> Node {
        let root = canvas_root();
        let (state, root_part) = self.prepare();
        let data = state.nodes;
        let edges = state.links;
        let selected = state.selected;
        let selected_link = state.selected_link;
        let transform = state.transform;
        let scene = state.scene;
        let reference = state.reference;
        let zoom = state.zoom;
        let gesture = crate::editors::node::input::drag(state);
        let infos = kivact::state::memo(move || {
            semantic(
                data.get(),
                edges.get(),
                selected.get(),
                selected_link.get(),
                scene.get().ok().as_deref(),
                state.texts,
            )
        });
        let focus = state.reference.focus_within();
        let ok = kivact::state::memo(move || focus.get() && !state.blocked.get());
        state.press.watch_inputs(ok, move || state.cancel());
        let root = root
            .value_text(move || {
                format!(
                    "{} items; {} selected; zoom {:.0}%",
                    data.with(Vec::len),
                    selected.get().len(),
                    transform.get().scale * 100.0
                )
            })
            .node_ref(reference)
            .part(root_part)
            .regions(
                infos,
                |info| info.id,
                move |_, info| region_control(state, gesture, info),
            );
        let root = root
            .draw(move |draw, size| crate::editors::node::paint::editor(draw, size, state))
            .on_pointer_default(move |event| pointer(state, event))
            .on_wheel_default(move |_| {
                if state.drag_state().is_some() || state.edit.is_active() {
                    Handled::Yes
                } else {
                    Handled::No
                }
            });
        root.part(gesture.source_with(move || state.source_snapshot()))
            // After the drag's sources, which take Escape during a drag.
            .part({
                let input_part = part(|input| input);
                kivact::custom::Part::new(input_part.state(), move |node| {
                    node.part(input_part)
                        .region_events()
                        .on_key(move |event| key(state, event))
                })
            })
            .part(zoom.viewport())
            .child(crate::editors::node::parameters::popup(state))
    }
}

impl NodeCanvas {
    fn prepare(self) -> (State, kivact::custom::Part<kivact::custom::PartState>) {
        let Self {
            nodes,
            links,
            state:
                crate::NodeState {
                    selection: selected,
                    link: selected_link,
                    transform,
                },
            snap,
            can_connect,
            edit: observer,
        } = self;
        let data = nodes;
        let edges = links;
        let order = kivact::state::memo(move || {
            data.get()
                .into_iter()
                .map(|n| ListItem::new(n.id, n.label))
                .collect::<Vec<_>>()
        });
        let choose = selection::controller(order, selected);
        let targets =
            targets::targets::<crate::editors::node::gesture::Target, (u64, usize)>(move || {
                targets(&data.get(), &edges.get())
            });
        let scene = targets.scene();
        let active = targets.active();
        let reference = node_ref();
        let phase = phase_observer(observer);
        let edit =
            crate::editors::shared::model::observed(EditSession::new(move |_| {}), Some(phase));
        let bounds = memo(move || crate::editors::node::layout::bounds(&data.get()));
        let root_part = {
            let input_part = part(|input| input);
            kivact::custom::Part::new(input_part.state(), move |node| {
                node.part(input_part).region_events()
            })
        };
        let input = root_part.state();
        let blocked = memo(move || {
            let input = input.get();
            input.is_disabled() || input.is_hidden() || !reference.is_visible()
        });
        // The canvas pans and zooms; the pan forwards its phases as the
        // editor's own.
        let zoom = use_zoom_pan(transform)
            .content(bounds)
            .on_edit(move |event| phase.try_call(event));
        let (armed, gesturing) = (zoom.is_space_armed(), zoom.is_gesturing());
        // A gesture begins its edit at its first change. The view pans
        // instead while Space is held or a pan runs.
        let gesture =
            use_drag::<crate::editors::node::gesture::DragSnapshot>(DragCoordinates::Local)
                .slop(0.0)
                .disabled(move || armed.get() || gesturing.get())
                .edit_on_change(edit, || ());
        let press = crate::editors::shared::gesture::CanvasGesture::new(gesture);
        let state = State {
            on_edit: observer,
            can_connect,
            nodes,
            links,
            selected,
            selected_link,
            choose,
            transform,
            scene,
            snap,
            active,
            mode: crate::editors::node::gesture::NodeController::new(),
            reference,
            texts: crate::editors::shared::texts::Texts::new(),
            press,
            gesture,
            edit,
            blocked: blocked.into_value(),
            zoom,
        };
        (state, root_part)
    }
}
fn phase_observer(
    observer: Option<Callback<crate::editors::shared::model::EditorEvent<NodeRequest>>>,
) -> Callback<EditPhase> {
    callback(move |phase| {
        if let Some(observer) = observer {
            observer.call(match phase {
                EditPhase::Begin => crate::editors::shared::model::EditorEvent::Begin,
                EditPhase::Commit => crate::editors::shared::model::EditorEvent::Commit,
                EditPhase::Cancel => crate::editors::shared::model::EditorEvent::Cancel,
            });
        }
    })
}
fn region_control(
    state: State,
    gesture: kivact::interaction::DragApi<crate::editors::node::gesture::DragSnapshot>,
    info: Value<crate::editors::node::gesture::Info>,
) -> kivact::custom::Part<kivact::custom::PartState> {
    let initial = info.get_untracked();
    let target = initial.target;
    // A press on a region drags from there.
    let mut control = {
        let input_part = part(|input| input);
        let drag_source = gesture.source_with(move || state.source_snapshot());
        kivact::custom::Part::new(input_part.state(), move |node| {
            node.part(input_part)
                .focusable(true)
                .role(initial.role)
                .name(move || info.get().name)
                .value_text(move || info.get().value)
                .part(drag_source)
        })
    };
    if initial.range().is_some() {
        control = kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .value_min(move || info.get().range().map_or(0.0, |v| v.0))
                .value_max(move || info.get().range().map_or(0.0, |v| v.1))
                .value_now(move || info.get().range().map_or(0.0, |v| v.2))
        });
    }
    if initial.checked().is_some() {
        control = kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .checked(move || info.get().checked().unwrap_or(false))
        });
    }
    if matches!(target, Target::Link(..)) {
        control = kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .selected(move || info.get().is_selected())
        });
    }
    if matches!(
        target,
        Target::Socket(..) | Target::Control(_, _, ControlKind::Choice)
    ) {
        return kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .a11y_action(A11yAction::Activate, move |_| state.invoke(target, false))
        });
    }
    region_actions(control, initial, target, state)
}
fn region_actions(
    control: kivact::custom::Part<kivact::custom::PartState>,
    initial: crate::editors::node::gesture::Info,
    target: Target,
    state: State,
) -> kivact::custom::Part<kivact::custom::PartState> {
    match (initial.role, initial.checked().is_some()) {
        (Role::SpinButton, _) => kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .a11y_action(A11yAction::SetValue, move |event| {
                    if let Target::Control(id, index, _) = target
                        && let A11yRequest::SetValue(text) = event.request
                        && let Some(value) = text.parse().ok()
                    {
                        state.change(id, index, Some(value), 0.0);
                    }
                })
                .a11y_action(A11yAction::Increment, move |_| {
                    if let Target::Control(id, index, _) = target {
                        state.change(id, index, None, 1.0);
                    }
                })
                .a11y_action(A11yAction::Decrement, move |_| {
                    if let Target::Control(id, index, _) = target {
                        state.change(id, index, None, -1.0);
                    }
                })
        }),
        (_, true) => kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .a11y_action(A11yAction::Toggle, move |_| state.invoke(target, false))
                .a11y_action(A11yAction::Activate, move |_| state.invoke(target, false))
        }),
        _ => kivact::custom::Part::new(control.state(), move |node| {
            node.part(control)
                .a11y_action(A11yAction::Activate, move |_| state.invoke(target, false))
                .a11y_action(A11yAction::Select, move |_| state.invoke(target, true))
        }),
    }
}
