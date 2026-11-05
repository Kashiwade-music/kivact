//! Role: Typed time-editor gestures and transaction snapshots.
use crate::editors::shared::model::{EditorContext, Lane};
use crate::editors::shared::targets;
use crate::editors::time::model::TimeItem;
use kivact::geometry::Point;
use kivact::interaction::{DragApi as Gesture, EditSession, ListApi};
use kivact::state::{Binding, Callback, Signal, Value};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Item(u64),
    Trim(u64, bool),
    Playhead,
    Lane(u64),
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Drag {
    Items,
    Trim(u64, bool),
    Playhead,
    Pan,
    Marquee(Point, Point),
}
/// Item snapshots are proposal sources; only bound values are restored by the kit.
#[derive(Clone, PartialEq)]
pub(crate) enum Snapshot {
    Items(Vec<(u64, f64, f64, u64)>),
    Playhead(f64),
    View(kivact::interaction::ViewTransform),
}
pub(crate) struct State<T: TimeItem> {
    pub data: Value<Vec<T>>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub on_edit: Option<
        Callback<
            crate::editors::shared::model::EditorEvent<crate::editors::time::model::TimeRequest>,
        >,
    >,
    pub selected: Binding<Vec<u64>>,
    pub selection: Value<Vec<u64>>,
    pub(crate) transform: Binding<kivact::interaction::ViewTransform>,
    pub scale: Value<f64>,
    pub offset: Value<Point>,
    pub playhead: Binding<f64>,
    pub visible: Value<Vec<Lane>>,
    pub choose: ListApi<u64>,
    pub scene: Value<Result<std::rc::Rc<targets::Scene<Target>>, kivact::UiError>>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub press: crate::editors::shared::gesture::CanvasGesture<Target, Option<Drag>, Snapshot>,
    /// The editor's drag, without the callbacks its sources carry
    /// (`input::drag`).
    pub gesture: Gesture<Snapshot>,
    pub edit: EditSession<Snapshot>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub snap: Value<bool>,
    pub step: Value<f64>,
    pub bounds: Value<kivact::interaction::ScalarRange>,
    pub min_duration: Value<f64>,
    pub lane_height: f64,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub active: Signal<Option<Target>>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub child_focus: Value<bool>,
    pub context: Signal<EditorContext>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub lane_activate: Option<Callback<u64>>,
    // xtask-allow(state-fields): Bindings, reactive options and keyboard targets are independent of the gesture lifecycle owned by DecisionMachine.
    pub item_activate: Option<Callback<u64>>,
}
impl<T: TimeItem> Copy for State<T> {}
impl<T: TimeItem> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TimeItem> State<T> {
    pub(crate) fn step(self) -> f64 {
        let step = self.step.get_untracked();
        if step.is_finite() && step > 0.0 {
            step
        } else {
            1.0
        }
    }
}

impl<T: TimeItem> State<T> {
    pub(crate) fn drag_state(self) -> Option<Drag> {
        self.press.with_payload(|payload| *payload).flatten()
    }
    pub(crate) fn set_drag(self, drag: Option<Drag>) {
        self.press.update_payload(|payload| *payload = drag);
    }
    pub(crate) fn source_snapshot(self) -> Snapshot {
        match self.drag_state() {
            Some(Drag::Items) => self.snapshot(&self.selection.get_untracked()),
            Some(Drag::Trim(id, _)) => self.snapshot(&[id]),
            Some(Drag::Playhead) => Snapshot::Playhead(self.playhead.get_untracked()),
            Some(Drag::Pan) => self.view_snapshot(),
            _ => self.snapshot(&[]),
        }
    }
    pub(crate) fn initial(self) -> Option<Snapshot> {
        self.press.initial()
    }
}
