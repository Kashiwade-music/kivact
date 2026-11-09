//! Role: Passive graph targets, gesture state and selective edit snapshots.

use crate::editors::graph::model::{Curve, CurveKey, GraphRequest, GraphView, KeyMove};
use crate::editors::shared::model::EditorContext;
use crate::editors::shared::ruler::Ruler;
use crate::editors::shared::targets::Scene;
use kivact::UiError;
use kivact::draw::RegionId;
use kivact::geometry::{Point, Size};
use kivact::interaction::{DragApi, EditSession, ListApi, SpacePanApi};
use kivact::state::{Binding, Callback, Signal, Value};
use kivact::style::Color;
use kivact::view::Role;
use std::rc::Rc;

pub(crate) struct Settings {
    pub(crate) ruler: Ruler,
    pub(crate) value_format: Rc<dyn Fn(f64) -> String>,
    pub(crate) step: (Value<f64>, Value<f64>),
    pub(crate) cells_width: Value<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Key(u64),
    Tangent(u64, bool),
    Playhead,
    Lane(u64),
}

#[derive(Clone, PartialEq)]
pub(crate) struct VisibleKey {
    pub(crate) curve: u64,
    pub(crate) key: CurveKey,
    pub(crate) name: String,
    pub(crate) color: Color,
    pub(crate) previous_bezier: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct Info {
    pub(crate) id: RegionId,
    pub(crate) target: Target,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) role: Role,
    pub(crate) selected: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) enum Snapshot {
    Keys(Vec<KeyMove>),
    Tangent(Vec<KeyMove>, bool),
    View(GraphView),
    Playhead(f64),
}

#[derive(Clone, PartialEq)]
pub(crate) enum DragKind {
    Keys(Vec<u64>),
    Tangent(u64, bool),
    Playhead,
    Pan,
    Marquee(Point, Point),
    Lane,
}

#[derive(Clone, PartialEq)]
pub(crate) struct Gesture {
    pub(crate) kind: DragKind,
}

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) curves: Value<Vec<Curve>>,
    pub(crate) on_edit: Option<Callback<crate::editors::shared::model::EditorEvent<GraphRequest>>>,
    pub(crate) view: Binding<GraphView>,
    pub(crate) selected: Binding<Vec<u64>>,
    pub(crate) playhead: Binding<f64>,
    pub(crate) visible: Value<Vec<VisibleKey>>,
    pub(crate) scene: Value<Result<Rc<Scene<Target>>, UiError>>,
    pub(crate) choose: ListApi<u64>,
    pub(crate) size: Value<Size>,
    pub(crate) snap: Value<bool>,
    pub(crate) step: (Value<f64>, Value<f64>),
    pub(crate) press:
        crate::editors::shared::gesture::CanvasGesture<Target, Option<Gesture>, Snapshot>,
    /// The editor's drag, without the callbacks its sources carry
    /// (`State::drag`).
    pub(crate) gesture: DragApi<Snapshot>,
    pub(crate) edit: EditSession<Snapshot>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub(crate) active: Signal<Option<Target>>,
    /// Space held on the focused editor: the next primary drag pans.
    pub(crate) space: SpacePanApi,
    pub(crate) lane_activate: Option<Callback<u64>>,
    pub(crate) item_activate: Option<Callback<u64>>,
    pub(crate) child_focus: Value<bool>,
    pub(crate) context: Signal<EditorContext>,
}

impl State {
    pub(crate) fn drag_state(self) -> Option<Gesture> {
        self.press.with_payload(Clone::clone).flatten()
    }
    pub(crate) fn set_drag(self, drag: Option<Gesture>) {
        self.press.update_payload(|payload| *payload = drag);
    }
    pub(crate) fn source_snapshot(self) -> Snapshot {
        match self.drag_state().map(|gesture| gesture.kind) {
            Some(DragKind::Keys(ids)) => self.positions(&ids),
            Some(DragKind::Tangent(id, incoming)) => self.tangent_snapshot(id, incoming),
            Some(DragKind::Playhead) => Snapshot::Playhead(self.playhead.get_untracked()),
            Some(DragKind::Pan) => Snapshot::View(self.view.get_untracked()),
            _ => Snapshot::Keys(Vec::new()),
        }
    }
}
