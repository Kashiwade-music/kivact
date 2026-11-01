//! Role: Gradient model writes and one shared edit transaction.
use super::data::{GradientStop, position_value};
use kivact::interaction::{Drag, EditSession};
use kivact::style::Color;
use kivact::ui::EditPhase;
use kivact::view::IntoValue;
use kivact::{Binding, Callback, Signal, Value, Var, signal, var};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Domain {
    Stop,
    Position,
    Color,
}
type Snapshot = (Vec<GradientStop>, Option<usize>);
#[derive(Clone, Copy)]
pub(super) struct State {
    pub(super) stops: Binding<Vec<GradientStop>>,
    pub(super) selection: Binding<Option<usize>>,
    pub(super) edit: EditSession<Snapshot>,
    /// The ramp's drag of a stop, once the ramp made it.
    pub(super) drag: Var<Option<Drag<usize>>>,
    pub(super) blocked: Signal<bool>,
    pub(super) valid: Value<bool>,
    pub(super) domain: Var<Option<Domain>>,
}
impl State {
    pub(super) fn new(
        stops: Binding<Vec<GradientStop>>,
        selection: Binding<Option<usize>>,
        valid: Value<bool>,
        observer: Option<Callback<EditPhase>>,
    ) -> Self {
        // Every edit, a drag's or a field's, ends its domain when it ends.
        let domain = var(None);
        let edit = EditSession::new(move |(data, selected): Snapshot| {
            stops.set(data);
            selection.set(selected);
        })
        .on_edit(move |phase| {
            if phase != EditPhase::Begin {
                domain.set(None);
            }
            if let Some(observer) = observer {
                observer.call(phase);
            }
        });
        Self {
            stops,
            selection,
            edit,
            drag: var(None),
            blocked: signal(false),
            valid,
            domain,
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
    pub(super) fn begin(self) {
        self.edit.begin(self.snapshot());
    }
    pub(super) fn once(self, write: impl FnOnce()) {
        if !self.enabled() || self.edit.is_active() {
            return;
        }
        self.edit.once(self.snapshot(), write);
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
        if !self.enabled() || self.domain.get() != Some(Domain::Color) || !self.edit.is_active() {
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
    pub(super) fn phase(self, domain: Domain, phase: EditPhase) {
        if !self.enabled() {
            return;
        }
        match phase {
            EditPhase::Begin if !self.edit.is_active() => {
                self.domain.set(Some(domain));
                self.begin();
            }
            EditPhase::Commit if self.domain.get() == Some(domain) => self.edit.commit(),
            EditPhase::Cancel if self.domain.get() == Some(domain) => {
                self.edit.cancel();
            }
            _ => {}
        }
    }
}
