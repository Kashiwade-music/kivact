//! Role: Passive node editor targets, gesture, semantic and edit session state.

use crate::editors::node::model::{NodeControl, NodeControlValue, NodeData, NodeLink};
use crate::editors::shared::targets::Scene;
use kivact::draw::{Point, RegionId};
use kivact::interaction::{Drag as Gesture, EditSession, List, ViewTransform};
use kivact::view::{NodeRef, Role};
use kivact::{Binding, Callback, Signal, UiError, Value, Var};
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Domain {
    Positions,
    Control(u64, usize),
}

#[derive(Clone)]
pub(crate) enum Snapshot {
    Positions(Vec<(u64, Point)>),
    Control(u64, usize, NodeControlValue),
    Empty,
}

#[derive(Clone, PartialEq)]
pub(crate) struct Info {
    pub(crate) id: RegionId,
    pub(crate) target: Target,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) role: Role,
    pub(crate) checked: Option<bool>,
    pub(crate) selected: bool,
    pub(crate) range: Option<(f64, f64, f64)>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Drag {
    Items,
    Control(u64, usize),
    Link(u64, usize, Point),
    Marquee(Point, Point),
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
    pub(crate) choose: List<u64>,
    pub(crate) transform: Binding<ViewTransform>,
    pub(crate) scene: Value<Result<Rc<Scene<Target>>, UiError>>,
    pub(crate) snap: Value<bool>,
    pub(crate) active: Signal<Option<(u64, usize)>>,
    pub(crate) port: Signal<Option<(u64, usize)>>,
    pub(crate) popup: Signal<Option<(u64, usize, Point)>>,
    pub(crate) reference: NodeRef,
    pub(crate) texts: crate::editors::shared::texts::Texts,
    pub(crate) press:
        crate::editors::shared::gesture::CanvasGesture<Target, (Option<Drag>, Option<Snapshot>)>,
    /// The editor's drag, without the callbacks its sources carry
    /// (`node_input::drag`).
    pub(crate) gesture: Gesture<()>,
    /// The target under the release of a drag (a link's input socket).
    pub(crate) released: Var<Option<Target>>,
    pub(crate) edit: EditSession<Snapshot>,
    pub(crate) blocked: Signal<bool>,
    pub(crate) pan_active: Signal<bool>,
    pub(crate) fitting: Var<bool>,
    pub(crate) slot: Signal<Option<(u64, usize, NodeControl)>>,
    pub(crate) rewire: Signal<Option<NodeLink>>,
    pub(crate) domain: Var<Option<Domain>>,
    pub(crate) space: Signal<bool>,
}

impl State {
    pub(crate) fn drag_state(self) -> Option<Drag> {
        self.press.with_payload(|payload| payload.0).flatten()
    }
    pub(crate) fn set_drag(self, drag: Option<Drag>) {
        self.press.update_payload(|payload| payload.0 = drag);
    }
    pub(crate) fn initial(self) -> Option<Snapshot> {
        self.press
            .with_payload(|payload| payload.1.clone())
            .flatten()
    }
}
