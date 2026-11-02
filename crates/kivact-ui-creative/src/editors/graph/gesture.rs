//! Role: Passive graph targets, gesture state and selective edit snapshots.

use crate::editors::graph::model::{Curve, CurveKey, GraphRequest, GraphView, KeyMove};
use crate::editors::shared::model::EditorContext;
use crate::editors::shared::ruler::Ruler;
use crate::editors::shared::targets::Scene;
use kivact::draw::{Point, RegionId, Size};
use kivact::interaction::{Drag, EditSession, List, SpacePan};
use kivact::style::Color;
use kivact::view::Role;
use kivact::{Binding, Callback, Signal, UiError, Value};
use std::rc::Rc;

pub(crate) struct Settings {
    pub(crate) ruler: Ruler,
    pub(crate) value_format: Rc<dyn Fn(f64) -> String>,
    pub(crate) step: (Value<f64>, Value<f64>),
    pub(crate) cells_width: f32,
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
    pub(crate) choose: List<u64>,
    pub(crate) size: Value<Size>,
    pub(crate) snap: Value<bool>,
    pub(crate) step: (Value<f64>, Value<f64>),
    pub(crate) press:
        crate::editors::shared::gesture::CanvasGesture<Target, (Option<Gesture>, Option<Snapshot>)>,
    /// The editor's drag, without the callbacks its sources carry
    /// (`State::drag`).
    pub(crate) gesture: Drag<()>,
    pub(crate) edit: EditSession<Snapshot>,
    pub(crate) active: Signal<Option<Target>>,
    /// Space held on the focused editor: the next primary drag pans.
    pub(crate) space: SpacePan,
    pub(crate) lane_activate: Option<Callback<u64>>,
    pub(crate) item_activate: Option<Callback<u64>>,
    pub(crate) child_focus: Value<bool>,
    pub(crate) context: Signal<EditorContext>,
}

impl State {
    pub(crate) fn drag_state(self) -> Option<Gesture> {
        self.press
            .with_payload(|payload| payload.0.clone())
            .flatten()
    }
    pub(crate) fn set_drag(self, drag: Option<Gesture>) {
        self.press.update_payload(|payload| payload.0 = drag);
    }
    pub(crate) fn initial(self) -> Option<Snapshot> {
        self.press
            .with_payload(|payload| payload.1.clone())
            .flatten()
    }
}
