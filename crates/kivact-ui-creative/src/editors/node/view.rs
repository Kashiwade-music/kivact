//! Role: The NodeEditor builder and the node canvas view it builds.

use super::a11y::{semantic, targets};
use crate::editors::node::gesture::{ConnectGuard, DragMode, NodeMode, State};
use crate::editors::node::gesture::{ControlKind, Target};
use crate::editors::node::input::{key, pointer};
use crate::editors::node::model::{NodeData, NodeLink, NodeRequest};
use crate::editors::shared::selection;
use crate::editors::shared::targets::{self};
use kivact::control::{Node, NodeBuilder, node};
use kivact::interaction::{DragCoordinates, EditSession, use_drag, use_zoom_pan};
use kivact::primitive::{NodeExt, NodeRefExt, part};
use kivact::style::{ColorToken, Overflow, style};
use kivact::ui::{EditPhase, ListItem};
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yRequest, Handled, IntoValue, IntoView, Role, View, callback, node_ref,
};
use kivact::{Callback, Value, effect, memo, signal};

/// An editor for typed nodes, sockets, controls and links.
pub struct NodeEditor {
    root: Node,
    nodes: Value<Vec<NodeData>>,
    links: Value<Vec<NodeLink>>,
    state: crate::NodeState,
    snap: Value<bool>,
    can_connect: ConnectGuard,
    edit: Option<Callback<crate::editors::shared::model::EditorEvent<NodeRequest>>>,
}

/// Display application-owned nodes and links without mutating either collection.
/// Apply proposals from [`NodeEditor::on_edit`] to make edits visible.
/// `state` holds the selected nodes, the selected link and the canvas's pan
/// and zoom.
#[track_caller]
pub fn node_editor(
    nodes: impl IntoValue<Vec<NodeData>>,
    links: impl IntoValue<Vec<NodeLink>>,
    state: crate::NodeState,
) -> NodeEditor {
    NodeEditor {
        root: node().role(Role::Canvas).focusable(true).style(
            style()
                .height(280)
                .overflow(Overflow::Hidden)
                .background(ColorToken::Surface),
        ),
        nodes: nodes.into_value(),
        links: links.into_value(),
        state,
        snap: false.into_value(),
        edit: None,
        can_connect: signal(
            std::rc::Rc::new(|_: &NodeLink| true) as std::rc::Rc<dyn Fn(&NodeLink) -> bool>
        )
        .into_value(),
    }
}

impl NodeEditor {
    /// Report absolute proposals and gesture phases. The application applies requests
    /// and restores its own items on Cancel. Pan/zoom forwards shared phases.
    pub fn on_edit<R: HandlerResult>(
        mut self,
        edit: impl Fn(crate::editors::shared::model::EditorEvent<NodeRequest>) -> R + 'static,
    ) -> Self {
        self.edit = Some(callback(edit));
        self
    }

    /// Guard connection highlighting and Connect reports. Defaults to accepting all links.
    pub fn can_connect(mut self, can_connect: impl Fn(&NodeLink) -> bool + 'static) -> Self {
        self.can_connect =
            signal(std::rc::Rc::new(can_connect) as std::rc::Rc<dyn Fn(&NodeLink) -> bool>)
                .into_value();
        self
    }
    /// Round the distance a drag moves the selected nodes to multiples of
    /// 10 px on both axes. Off by default.
    pub fn snap(mut self, snap: impl IntoValue<bool>) -> Self {
        self.snap = snap.into_value();
        self
    }
}

impl NodeBuilder for NodeEditor {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for NodeEditor {
    fn into_view(self) -> View {
        let Self {
            root,
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
        let data = nodes.into_value();
        let edges = links.into_value();
        let order = kivact::memo(move || {
            data.get()
                .into_iter()
                .map(|n| ListItem::new(n.id, n.label))
                .collect::<Vec<_>>()
        });
        let choose = selection::controller(order, selected);
        let scene = targets::targets(move || targets(&data.get(), &edges.get()));
        let reference = node_ref();
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |_| {}),
            Some(callback(move |phase| {
                if let Some(observer) = observer {
                    observer.call(match phase {
                        EditPhase::Begin => crate::editors::shared::model::EditorEvent::Begin,
                        EditPhase::Commit => crate::editors::shared::model::EditorEvent::Commit,
                        EditPhase::Cancel => crate::editors::shared::model::EditorEvent::Cancel,
                    });
                }
            })),
        );
        let press = crate::editors::shared::gesture::CanvasGesture::<
            crate::editors::node::gesture::Target,
            DragMode,
        >::new();
        let bounds = memo(move || crate::editors::node::layout::bounds(&data.get()));
        let root_part = part(|input| input).region_events();
        let input = root_part.state();
        let blocked = memo(move || {
            let input = input.get();
            input.disabled || input.hidden || !reference.is_visible()
        });
        // The canvas pans and zooms; the pan forwards its phases as the
        // editor's own.
        let pan_observer = callback(move |phase| {
            if let Some(observer) = observer {
                observer.call(match phase {
                    EditPhase::Begin => crate::editors::shared::model::EditorEvent::Begin,
                    EditPhase::Commit => crate::editors::shared::model::EditorEvent::Commit,
                    EditPhase::Cancel => crate::editors::shared::model::EditorEvent::Cancel,
                });
            }
        });
        let zoom = use_zoom_pan(transform)
            .content(bounds)
            .on_edit(move |phase| pan_observer.try_call(phase));
        let (armed, gesturing) = (zoom.space_armed(), zoom.is_gesturing());
        // A gesture begins its edit at its first change. The view pans
        // instead while Space is held or a pan runs.
        let gesture = use_drag::<()>(DragCoordinates::Local)
            .disabled(move || armed.get() || gesturing.get())
            .edit_on_change(edit, || ());
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
            active: signal(None),
            mode: signal(NodeMode::Idle),
            reference,
            texts: crate::editors::shared::texts::Texts::new(),
            press,
            gesture,
            edit,
            blocked: blocked.into_value(),
            zoom,
        };
        let gesture = crate::editors::node::input::drag(state);
        let infos = kivact::memo(move || {
            semantic(
                data.get(),
                edges.get(),
                selected.get(),
                selected_link.get(),
                scene.get().ok().as_deref(),
                state.texts,
            )
        });
        let focus = reference.focus_within();
        effect(move || {
            if blocked.get() {
                state.cancel();
            } else if !focus.get() {
                if state.drag_state().is_some() || state.edit.is_active() {
                    state.cancel();
                } else {
                    // A keyboard connection ends when the focus leaves; an
                    // open popup follows its own focus rules.
                    state.mode.update(|mode| {
                        if matches!(mode, NodeMode::Connecting { .. }) {
                            *mode = NodeMode::Idle;
                        }
                    });
                }
            }
        });
        effect(move || {
            let nodes = data.get();
            if let Some((id, index, original)) = state.held_slot()
                && !nodes
                    .iter()
                    .find(|n| n.id == id)
                    .and_then(|n| n.controls.get(index))
                    .is_some_and(|c| crate::editors::node::edit::same_slot(&original, c))
            {
                state.cancel();
            }
        });
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
                move |_, info| {
                    let initial = info.get_untracked();
                    let target = initial.target;
                    // A press on a region drags from there.
                    let mut control = part(|input| input)
                        .focusable(true)
                        .role(initial.role)
                        .name(move || info.get().name)
                        .value_text(move || info.get().value)
                        .with(gesture.source(()));
                    if initial.range.is_some() {
                        control = control
                            .value_min(move || info.get().range.map_or(0.0, |v| v.0))
                            .value_max(move || info.get().range.map_or(0.0, |v| v.1))
                            .value_now(move || info.get().range.map_or(0.0, |v| v.2));
                    }
                    if initial.checked.is_some() {
                        control = control.checked(move || info.get().checked.unwrap_or(false));
                    }
                    if matches!(target, Target::Link(..)) {
                        control = control.selected(move || info.get().selected);
                    }
                    if matches!(
                        target,
                        Target::Socket(..) | Target::Control(_, _, ControlKind::Choice)
                    ) {
                        return control.on_a11y_event(A11yAction::Activate, move |_| {
                            state.invoke(target, false)
                        });
                    }
                    match (initial.role, initial.checked.is_some()) {
                        (Role::SpinButton, _) => control
                            .on_a11y_event(A11yAction::SetValue, move |event| {
                                if let Target::Control(id, index, _) = target
                                    && let A11yRequest::SetValue(text) = event.request
                                    && let Some(value) = text.parse().ok()
                                {
                                    state.change(id, index, Some(value), 0.0);
                                }
                            })
                            .on_a11y_event(A11yAction::Increment, move |_| {
                                if let Target::Control(id, index, _) = target {
                                    state.change(id, index, None, 1.0);
                                }
                            })
                            .on_a11y_event(A11yAction::Decrement, move |_| {
                                if let Target::Control(id, index, _) = target {
                                    state.change(id, index, None, -1.0);
                                }
                            }),
                        (_, true) => control
                            .on_a11y_event(A11yAction::Toggle, move |_| state.invoke(target, false))
                            .on_a11y_event(A11yAction::Activate, move |_| {
                                state.invoke(target, false)
                            }),
                        _ => control
                            .on_a11y_event(A11yAction::Activate, move |_| {
                                state.invoke(target, false)
                            })
                            .on_a11y_event(A11yAction::Select, move |_| state.invoke(target, true)),
                    }
                },
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
        root.part(gesture.source(()))
            // After the drag's sources, which take Escape during a drag.
            .part(
                part(|input| input)
                    .region_events()
                    .on_key(move |event| key(state, event)),
            )
            .part(zoom.viewport())
            .child(crate::editors::node::parameters::popup(state))
            .into_view()
    }
}
