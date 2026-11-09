//! Role: Passive node editor targets, gesture, semantic and edit session state.

use crate::editors::node::model::{NodeControl, NodeControlValue, NodeData, NodeLink};
use crate::editors::shared::targets::Scene;
use kivact::UiError;
use kivact::draw::RegionId;
use kivact::geometry::Point;
use kivact::interaction::{DragApi as Gesture, EditSession, ListApi, ViewTransform, ZoomPanApi};
use kivact::state::{Binding, Callback, Machine, MachineHandle, Step, Value, machine};
use kivact::view::{NodeRef, Role};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ControlKind {
    Number,
    Choice,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Item(u64),
    Socket(u64, bool, usize),
    Control(u64, usize, ControlKind),
    Link(u64, usize, u64, usize),
}

impl Target {
    pub(crate) fn control(node: &NodeData, index: usize) -> Self {
        let kind = match node.controls[index].value {
            NodeControlValue::Number { .. } => ControlKind::Number,
            NodeControlValue::Choice { .. } => ControlKind::Choice,
            NodeControlValue::Toggle(_) => ControlKind::Toggle,
        };
        Self::Control(node.id, index, kind)
    }
    pub(crate) fn link(link: NodeLink) -> Self {
        Self::Link(link.from, link.output, link.to, link.input)
    }
    pub(crate) fn as_link(self) -> Option<NodeLink> {
        if let Self::Link(a, b, c, d) = self {
            Some(NodeLink::new(a, b, c, d))
        } else {
            None
        }
    }
}

pub(crate) type ConnectGuard = Value<Rc<dyn Fn(&NodeLink) -> bool>>;

#[derive(Clone, PartialEq)]
pub(crate) struct Info {
    pub(crate) id: RegionId,
    pub(crate) target: Target,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) role: Role,
    // xtask-allow(state-fields): option: a target reports a checked state only when its role has one
    pub(crate) checked: Option<bool>,
    pub(crate) selected: bool,
    // xtask-allow(state-fields): option: a target reports a numeric range only when its role has one
    pub(crate) range: Option<(f64, f64, f64)>,
}

/// What the editor does besides a pointer drag. Its data lives in the variant.
#[derive(Clone, PartialEq)]
pub(crate) enum NodeMode {
    Idle,
    /// An output socket was activated without the pointer; activating an
    /// input socket connects them.
    Connecting {
        from: u64,
        output: usize,
    },
    /// The parameter popup of a control is open at `at` (window
    /// coordinates), with the control as it was when it opened or when its
    /// edit began.
    Popup {
        node: u64,
        index: usize,
        at: Point,
        slot: NodeControl,
    },
}

enum ModeEvent {
    BeginConnect {
        from: u64,
        output: usize,
    },
    OpenPopup {
        node: u64,
        index: usize,
        at: Point,
        slot: NodeControl,
    },
    ClosePopup,
    UpdatePopupSlot {
        slot: NodeControl,
    },
    Cancel,
    Inputs {
        focused: bool,
        blocked: bool,
        data_valid: bool,
        editing: bool,
        dragging: bool,
        rollback: Rc<dyn Fn()>,
    },
}
struct ModeMachine;
impl Machine for ModeMachine {
    type State = NodeMode;
    type Event = ModeEvent;
    type Effect = Rc<dyn Fn()>;
    fn transition(
        &self,
        state: &NodeMode,
        event: ModeEvent,
    ) -> Step<NodeMode, ModeEvent, Self::Effect> {
        match event {
            ModeEvent::BeginConnect { from, output } => {
                Step::to(NodeMode::Connecting { from, output })
            }
            ModeEvent::OpenPopup {
                node,
                index,
                at,
                slot,
            } => Step::to(NodeMode::Popup {
                node,
                index,
                at,
                slot,
            }),
            ModeEvent::ClosePopup => {
                if matches!(state, NodeMode::Popup { .. }) {
                    Step::to(NodeMode::Idle)
                } else {
                    Step::stay()
                }
            }
            ModeEvent::UpdatePopupSlot { slot } => match state {
                NodeMode::Popup {
                    node, index, at, ..
                } => Step::to(NodeMode::Popup {
                    node: *node,
                    index: *index,
                    at: *at,
                    slot,
                }),
                _ => Step::stay(),
            },
            ModeEvent::Cancel => Step::to(NodeMode::Idle),
            ModeEvent::Inputs {
                focused,
                blocked,
                data_valid,
                editing,
                dragging,
                rollback,
            } => {
                if blocked || !data_valid || (!focused && (editing || dragging)) {
                    Step::to(NodeMode::Idle).with(rollback)
                } else if !focused && matches!(state, NodeMode::Connecting { .. }) {
                    Step::to(NodeMode::Idle)
                } else {
                    Step::stay()
                }
            }
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) struct NodeController {
    machine: MachineHandle<ModeMachine>,
}
impl NodeController {
    pub(crate) fn new() -> Self {
        Self {
            machine: machine(NodeMode::Idle, || ModeMachine, |_, rollback| rollback()),
        }
    }
    pub(crate) fn begin_connect(self, from: u64, output: usize) {
        self.machine.send(ModeEvent::BeginConnect { from, output });
    }
    pub(crate) fn open_popup(self, node: u64, index: usize, at: Point, slot: NodeControl) {
        self.machine.send(ModeEvent::OpenPopup {
            node,
            index,
            at,
            slot,
        });
    }
    pub(crate) fn close_popup(self) {
        self.machine.send(ModeEvent::ClosePopup);
    }
    pub(crate) fn update_popup_slot(self, slot: NodeControl) {
        self.machine.send(ModeEvent::UpdatePopupSlot { slot });
    }
    pub(crate) fn cancel(self) {
        self.machine.send(ModeEvent::Cancel);
    }
    pub(crate) fn with<R>(self, read: impl FnOnce(&NodeMode) -> R) -> R {
        self.machine.state().with(read)
    }
    pub(crate) fn with_untracked<R>(self, read: impl FnOnce(&NodeMode) -> R) -> R {
        self.machine.state().with_untracked(read)
    }
    pub(crate) fn get_untracked(self) -> NodeMode {
        self.machine.state().get_untracked()
    }
    pub(crate) fn inputs(
        self,
        focused: bool,
        blocked: bool,
        data_valid: bool,
        editing: bool,
        dragging: bool,
        rollback: impl Fn() + 'static,
    ) {
        self.machine.send(ModeEvent::Inputs {
            focused,
            blocked,
            data_valid,
            editing,
            dragging,
            rollback: Rc::new(rollback),
        });
    }
}

/// The primary drag the press chose; the payload of `State::press`.
#[derive(Clone, PartialEq)]
pub(crate) enum DragMode {
    /// The selected nodes, from their positions at the press.
    Items {
        ids: Vec<u64>,
    },
    /// A number control, from the control at the press.
    Control {
        node: u64,
        index: usize,
    },
    /// A link from an output socket to `to`; `released` is the input socket
    /// under the release.
    Link {
        to: Point,
        source: LinkSource,
        // xtask-allow(state-fields): independent: the socket under the pointer at release, for either source
        released: Option<(u64, usize)>,
    },
    Marquee {
        from: Point,
        to: Point,
    },
}

/// Where a link drag starts: at an output socket, or at the input socket of
/// an existing link, which it re-routes.
#[derive(Clone, PartialEq)]
pub(crate) enum LinkSource {
    /// A new link from this output socket.
    Output { from: u64, output: usize },
    /// The link drawn again from its own output socket.
    Rewire(NodeLink),
}

impl LinkSource {
    /// The output socket the dragged link starts from.
    pub(crate) fn output(&self) -> (u64, usize) {
        match self {
            Self::Output { from, output } => (*from, *output),
            Self::Rewire(link) => (link.from, link.output),
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) enum DragSnapshot {
    Items(Vec<(u64, Point)>),
    Control {
        node: u64,
        index: usize,
        slot: NodeControl,
    },
    None,
}

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) on_edit: Option<
        Callback<
            crate::editors::shared::model::EditorEvent<crate::editors::node::model::NodeRequest>,
        >,
    >,
    pub(crate) can_connect: ConnectGuard,
    pub(crate) nodes: Value<Vec<NodeData>>,
    pub(crate) links: Value<Vec<NodeLink>>,
    pub(crate) selected: Binding<Vec<u64>>,
    pub(crate) selected_link: Binding<Option<NodeLink>>,
    pub(crate) choose: ListApi<u64>,
    pub(crate) transform: Binding<ViewTransform>,
    pub(crate) scene: Value<Result<Rc<Scene<Target>>, UiError>>,
    pub(crate) snap: Value<bool>,
    pub(crate) active: kivact::state::Binding<Option<(u64, usize)>>,
    pub(crate) mode: NodeController,
    pub(crate) reference: NodeRef,
    pub(crate) texts: crate::editors::shared::texts::Texts,
    pub(crate) press:
        crate::editors::shared::gesture::CanvasGesture<Target, DragMode, DragSnapshot>,
    /// The editor's drag, without the callbacks its sources carry
    /// (`node_input::drag`).
    pub(crate) gesture: Gesture<DragSnapshot>,
    /// The editor's edit. The application restores its own items on
    /// `Cancel`, so the session restores nothing.
    pub(crate) edit: EditSession<()>,
    /// The editor is disabled, hidden or not visible.
    pub(crate) blocked: Value<bool>,
    /// The canvas's pan and zoom, which also tracks Space.
    pub(crate) zoom: ZoomPanApi,
}

impl State {
    pub(crate) fn source_snapshot(self) -> DragSnapshot {
        match self.drag_state() {
            Some(DragMode::Items { ids }) => {
                DragSnapshot::Items(self.nodes.with_untracked(|nodes| {
                    nodes
                        .iter()
                        .filter(|node| ids.contains(&node.id))
                        .map(|node| (node.id, node.position))
                        .collect()
                }))
            }
            Some(DragMode::Control { node, index }) => {
                self.control(node, index)
                    .map_or(DragSnapshot::None, |slot| DragSnapshot::Control {
                        node,
                        index,
                        slot,
                    })
            }
            _ => DragSnapshot::None,
        }
    }
    /// What the pressed drag does; `None` while no press drags.
    pub(crate) fn drag_state(self) -> Option<DragMode> {
        self.press.with_payload(Clone::clone)
    }
    /// The link a drag re-routes from its input socket.
    pub(crate) fn rewired(self) -> Option<NodeLink> {
        self.press
            .with_payload(|payload| match payload {
                DragMode::Link {
                    source: LinkSource::Rewire(link),
                    ..
                } => Some(*link),
                _ => None,
            })
            .flatten()
    }
}
