//! Role: Gradient model writes and one shared edit transaction.
use super::data::{GradientStop, position_value};
use crate::editors::shared::owned_edit::OwnedEdit;
use kivact::interaction::Drag;
use kivact::style::Color;
use kivact::ui::EditPhase;
use kivact::view::IntoValue;
use kivact::{Binding, Callback, Signal, Value, Var, signal, var};
/// Who began the active edit of the shared session.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GradientOwner {
    /// A drag of a stop on the ramp.
    Stop,
    /// The position field.
    Position,
    /// The color picker.
    Color,
    /// A whole step: adding, removing or moving a stop by key.
    Step,
}
type Snapshot = (Vec<GradientStop>, Option<usize>);
#[derive(Clone, Copy)]
pub(super) struct State {
    pub(super) stops: Binding<Vec<GradientStop>>,
    pub(super) selection: Binding<Option<usize>>,
    pub(super) edit: OwnedEdit<GradientOwner, Snapshot>,
    /// The ramp's drag of a stop, once the ramp made it.
    pub(super) drag: Var<Option<Drag<usize>>>,
    pub(super) blocked: Signal<bool>,
    pub(super) valid: Value<bool>,
}
impl State {
    pub(super) fn new(
        stops: Binding<Vec<GradientStop>>,
        selection: Binding<Option<usize>>,
        valid: Value<bool>,
        observer: Option<Callback<EditPhase>>,
    ) -> Self {
        let edit = OwnedEdit::new(
            move |(data, selected): Snapshot| {
                stops.set(data);
                selection.set(selected);
            },
            observer,
        );
        Self {
            stops,
            selection,
            edit,
            drag: var(None),
            blocked: signal(false),
            valid,
        }
    }
    pub(super) fn enabled(self) -> bool {
        !self.blocked.get_untracked() && self.valid.get_untracked()
    }
    /// End the active edit, a drag's or a field's, writing its start back.
    pub(super) fn cancel(self) {
        if let Some(drag) = self.drag.get() {
            drag.cancel();
        }
        self.edit.cancel();
    }
    pub(super) fn snapshot(self) -> Snapshot {
        (
            self.stops.into_value().get_untracked(),
            self.selection.into_value().get_untracked(),
        )
    }
    pub(super) fn once(self, write: impl FnOnce()) {
        if !self.enabled() || self.edit.is_active() {
            return;
        }
        self.edit
            .once(GradientOwner::Step, || self.snapshot(), write);
    }
    pub(super) fn selected(self) -> Option<GradientStop> {
        let id = self.selection.into_value().get_untracked()?;
        self.stops
            .into_value()
            .with_untracked(|stops| stops.get(id).copied())
    }
    pub(super) fn position(self, id: usize, position: f64) {
        if !self.edit.is_active() {
            return;
        }
        let mut stops = self.stops.into_value().get_untracked();
        if let Some(stop) = stops.get_mut(id) {
            stop.position = position_value(position);
            self.stops.set(stops);
        }
    }
    pub(super) fn change_position(self, id: usize, position: f64) {
        self.once(|| {
            self.selection.set(Some(id));
            self.position(id, position);
        });
    }
    pub(super) fn color(self, color: Color) {
        if !self.enabled() || self.edit.owner() != Some(GradientOwner::Color) {
            return;
        }
        let Some(id) = self.selection.into_value().get_untracked() else {
            return;
        };
        let mut stops = self.stops.into_value().get_untracked();
        if let Some(stop) = stops.get_mut(id) {
            stop.color = color;
            self.stops.set(stops);
        }
    }
    pub(super) fn remove(self, id: usize) {
        self.once(|| {
            let mut stops = self.stops.into_value().get_untracked();
            if id < stops.len() {
                stops.remove(id);
                let selected = self
                    .selection
                    .into_value()
                    .get_untracked()
                    .and_then(|selected| {
                        if stops.is_empty() {
                            None
                        } else if selected == id {
                            Some(id.min(stops.len() - 1))
                        } else {
                            Some(if selected > id {
                                selected - 1
                            } else {
                                selected
                            })
                        }
                    });
                self.stops.set(stops);
                self.selection.set(selected);
            }
        });
    }
    pub(super) fn add(self) {
        self.once(|| {
            let mut stops = self.stops.into_value().get();
            let id = stops.len();
            let color = self.selected().map_or(Color::WHITE, |s| s.color);
            stops.push(GradientStop::new(0.5, color));
            self.stops.set(stops);
            self.selection.set(Some(id));
        });
    }
    /// A phase of an edit that `owner` reports.
    pub(super) fn phase(self, owner: GradientOwner, phase: EditPhase) {
        if !self.enabled() {
            return;
        }
        match phase {
            EditPhase::Begin => self.edit.begin(owner, self.snapshot()),
            EditPhase::Commit => self.edit.commit(owner),
            EditPhase::Cancel => {
                self.edit.cancel_owned(owner);
            }
        }
    }
}
