//! Role: Shared arrangement gesture state and logical beat conversion.
use crate::Clip;
use crate::editors::arrangement::a11y::targets::Regions;
use crate::editors::arrangement::a11y::texts::ArrangementTexts;
use crate::editors::arrangement::model::{
    AutomationPoint, TimelineMarker, TimelineTool, TimelineTrack,
};
use crate::editors::shared::metrics::ARRANGEMENT_SIDEBAR;
use kivact::geometry::{Point, Size};
use kivact::interaction::EditSession;
use kivact::interaction::{ListApi, ListGesture};
use kivact::state::{Binding, Callback, Signal, Value};
#[derive(Clone, PartialEq)]
pub(crate) enum Snapshot {
    Clips(Vec<(u64, f64, f64)>),
    Automation(u64, u64, Option<AutomationPoint>),
    Playhead(f64),
    Loop((f64, f64)),
    Cycle(bool),
    View(f64, Point),
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Drag {
    Loop(LoopPart),
    Playhead,
    Clips,
    Trim(u64, bool),
    Auto(u64, u64),
    AddPoint(u64),
    Marquee(Point),
    Pan,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LoopPart {
    Start,
    End,
    Body,
    Create,
}
#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) size: Value<Size>,
    pub(crate) data: Value<Vec<Clip>>,
    pub(crate) length: Value<f64>,
    pub(crate) bounds: Value<kivact::interaction::ScalarRange>,
    pub(crate) min_duration: Value<f64>,
    pub(crate) loop_min_duration: Value<f64>,
    // xtask-allow(state-fields): Bindings, reactive options and active keyboard target are independent of the gesture lifecycle owned by DecisionMachine.
    pub(crate) on_edit: Option<
        Callback<
            crate::editors::shared::model::EditorEvent<
                crate::editors::arrangement::model::ArrangementRequest,
            >,
        >,
    >,
    pub(crate) track_data: Value<Vec<TimelineTrack>>,
    pub(crate) selected: Binding<Vec<u64>>,
    pub(crate) selection: Value<Vec<u64>>,
    pub(crate) playhead: Binding<f64>,
    pub(crate) frame: Value<f64>,
    pub(crate) loop_range: Binding<(f64, f64)>,
    pub(crate) range: Value<(f64, f64)>,
    // xtask-allow(state-fields): Bindings, reactive options and active keyboard target are independent of the gesture lifecycle owned by DecisionMachine.
    pub(crate) cycle: Value<bool>,
    // xtask-allow(state-fields): Bindings, reactive options and active keyboard target are independent of the gesture lifecycle owned by DecisionMachine.
    pub(crate) cycling: Binding<bool>,
    pub(crate) transform: Binding<kivact::interaction::ViewTransform>,
    pub(crate) pixels: Value<f64>,
    pub(crate) offset: Value<Point>,
    pub(crate) tool_binding: Binding<TimelineTool>,
    pub(crate) tool: Value<TimelineTool>,
    pub(crate) markers: Value<Vec<TimelineMarker>>,
    // xtask-allow(state-fields): Bindings, reactive options and active keyboard target are independent of the gesture lifecycle owned by DecisionMachine.
    pub(crate) snap: Value<bool>,
    pub(crate) step: Value<f64>,
    // xtask-allow(state-fields): Bindings, reactive options and active keyboard target are independent of the gesture lifecycle owned by DecisionMachine.
    pub(crate) active_auto: Signal<Option<(u64, u64)>>,
    pub(crate) press: crate::editors::shared::gesture::CanvasGesture<
        crate::editors::arrangement::a11y::targets::Target,
        Option<Drag>,
        Snapshot,
    >,
    /// The arrangement's drag, without the callbacks its sources carry
    /// (`input::drag`).
    pub(crate) gesture: kivact::interaction::DragApi<Snapshot>,
    pub(crate) edit: EditSession<Snapshot>,
    pub(crate) regions: Value<Regions>,
    pub(crate) texts: ArrangementTexts,
    pub(crate) choose: ListApi<u64>,
}
impl State {
    /// Logical pixels per beat within the zoom bounds.
    pub(crate) fn scale(self) -> f64 {
        self.pixels
            .get_untracked()
            .clamp(self.bounds.get().min(), self.bounds.get().max())
    }
    pub(crate) fn beat(self, x: f64) -> f64 {
        ((x - ARRANGEMENT_SIDEBAR - self.offset.get().x) / self.scale())
            .clamp(0.0, self.length.get())
    }
    /// The step in beats; an invalid one falls back to 1.
    pub(crate) fn step(self) -> f64 {
        let step = self.step.get_untracked();
        if step.is_finite() && step > 0.0 {
            step
        } else {
            1.0
        }
    }
    pub(crate) fn snapping(self, value: f64) -> f64 {
        if self.snap.get() {
            let unit = self.step();
            (value / unit).round() * unit
        } else {
            (value * 16.0).round() / 16.0
        }
    }
    pub(crate) fn select(self, id: u64, gesture: ListGesture) {
        self.choose.select(id, gesture);
    }
}

impl State {
    pub(crate) fn drag_state(self) -> Option<Drag> {
        self.press.with_payload(|payload| *payload).flatten()
    }
    pub(crate) fn initial(self) -> Option<Snapshot> {
        self.press.initial()
    }
}

impl State {
    pub(crate) fn marquee(self) -> Option<(Point, Point)> {
        match self.drag_state() {
            Some(Drag::Marquee(from)) => Some((from, self.press.current())),
            _ => None,
        }
    }
}
