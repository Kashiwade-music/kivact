//! Role: Gradient model writes and one shared edit transaction.
use super::super::data::{GradientStop, position_value};
use super::data::{GradientOwner, Snapshot};
use super::machine;
use kivact::interaction::DragApi;
use kivact::state::IntoValue;

use kivact::state::{Binding, Callback, MachineHandle, Value};
use kivact::style::Color;
use kivact::view::EditPhase;
#[derive(Clone, Copy)]
pub(in crate::gradient) struct GradientOptions {
    pub(in crate::gradient) stops: Binding<Vec<GradientStop>>,
    pub(in crate::gradient) selection: Binding<Option<usize>>,
    pub(in crate::gradient) blocked: Value<bool>,
    pub(in crate::gradient) valid: Value<bool>,
}
#[derive(Clone, Copy)]
pub(in crate::gradient) struct Model {
    options: GradientOptions,
    pub(in crate::gradient) edit: Edit,
}
impl std::ops::Deref for Model {
    type Target = GradientOptions;
    fn deref(&self) -> &GradientOptions {
        &self.options
    }
}
impl Model {
    pub(in crate::gradient) fn new(
        stops: Binding<Vec<GradientStop>>,
        selection: Binding<Option<usize>>,
        valid: Value<bool>,
        observer: Option<Callback<EditPhase>>,
        blocked: Value<bool>,
        drag: DragApi<(usize, f64, f64)>,
    ) -> Self {
        let edit = Edit::new(
            move |(data, selected): Snapshot| {
                stops.set(data);
                selection.set(selected);
            },
            observer,
            kivact::state::memo(move || blocked.get() || !valid.get()),
            drag,
        );
        Self {
            options: GradientOptions {
                stops,
                selection,
                blocked,
                valid,
            },
            edit,
        }
    }
    pub(in crate::gradient) fn enabled(self) -> bool {
        !self.blocked.get_untracked() && self.valid.get_untracked()
    }
    pub(in crate::gradient) fn snapshot(self) -> Snapshot {
        (
            self.stops.into_value().get_untracked(),
            self.selection.into_value().get_untracked(),
        )
    }
    pub(in crate::gradient) fn once(self, write: impl FnOnce()) {
        if !self.enabled() || self.edit.is_active() {
            return;
        }
        self.edit
            .once(GradientOwner::Step, || self.snapshot(), write);
    }
    pub(in crate::gradient) fn selected(self) -> Option<GradientStop> {
        let id = self.selection.into_value().get_untracked()?;
        self.stops
            .into_value()
            .with_untracked(|stops| stops.get(id).copied())
    }
    pub(in crate::gradient) fn position(self, id: usize, position: f64) {
        if !self.edit.is_active() {
            return;
        }
        let mut stops = self.stops.into_value().get_untracked();
        if let Some(stop) = stops.get_mut(id) {
            stop.position = position_value(position);
            self.stops.set(stops);
        }
    }
    pub(in crate::gradient) fn change_position(self, id: usize, position: f64) {
        self.once(|| {
            self.selection.set(Some(id));
            self.position(id, position);
        });
    }
    pub(in crate::gradient) fn color(self, color: Color) {
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
    pub(in crate::gradient) fn remove(self, id: usize) {
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
    pub(in crate::gradient) fn add(self) {
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
    pub(in crate::gradient) fn phase(self, owner: GradientOwner, phase: EditPhase) {
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

#[derive(Clone, Copy)]
pub(in crate::gradient) struct State {
    model: Model,
    drag: DragApi<(usize, f64, f64)>,
}
impl std::ops::Deref for State {
    type Target = Model;
    fn deref(&self) -> &Model {
        &self.model
    }
}
impl State {
    pub(in crate::gradient) fn new(model: Model, drag: DragApi<(usize, f64, f64)>) -> Self {
        let state = Self { model, drag };
        kivact::state::on_cleanup(move || state.cancel());
        state
    }
    pub(in crate::gradient) fn cancel(self) {
        self.drag.cancel();
        self.edit.cancel();
    }
    pub(in crate::gradient) fn observe(self, focused: Value<bool>) {
        self.edit.handle.feed(
            kivact::state::memo(move || self.blocked.get() || !self.valid.get()),
            machine::Event::Blocked,
        );
        self.edit.handle.feed(focused, machine::Event::Focused);
    }
}

#[derive(Clone, Copy)]
pub(in crate::gradient) struct Edit {
    handle: MachineHandle<machine::Machine>,
}
impl Edit {
    fn new(
        restore: impl Fn(Snapshot) + 'static,
        observer: Option<Callback<EditPhase>>,
        blocked: Value<bool>,
        drag: DragApi<(usize, f64, f64)>,
    ) -> Self {
        let handle = kivact::state::machine(
            machine::State::Idle,
            move || machine::Machine {
                blocked: blocked.get_untracked(),
            },
            move |_, effect| match effect {
                machine::Effect::Phase(phase) => {
                    if let Some(observer) = observer {
                        observer.call(phase);
                    }
                }
                machine::Effect::Rollback(snapshot) => restore(snapshot),
                machine::Effect::StopDrag => {
                    drag.cancel();
                }
            },
        );
        let edit = Self { handle };
        kivact::state::on_cleanup(move || {
            edit.cancel();
        });
        edit
    }
    pub(in crate::gradient) fn owner(self) -> Option<GradientOwner> {
        match self.handle.get() {
            machine::State::Idle => None,
            machine::State::Editing { owner, .. } => Some(owner),
        }
    }
    pub(in crate::gradient) fn is_active(self) -> bool {
        self.owner().is_some()
    }
    pub(in crate::gradient) fn begin(self, owner: GradientOwner, snapshot: Snapshot) {
        self.handle.send(machine::Event::Begin { owner, snapshot });
    }
    pub(in crate::gradient) fn commit(self, owner: GradientOwner) {
        self.handle.send(machine::Event::Commit(owner));
    }
    pub(in crate::gradient) fn cancel(self) -> bool {
        let active = self.is_active();
        self.handle.send(machine::Event::Cancel);
        active
    }
    pub(in crate::gradient) fn cancel_owned(self, owner: GradientOwner) -> bool {
        if self.owner() == Some(owner) {
            self.cancel()
        } else {
            false
        }
    }
    pub(in crate::gradient) fn once(
        self,
        owner: GradientOwner,
        snapshot: impl FnOnce() -> Snapshot,
        write: impl FnOnce(),
    ) {
        match self.owner() {
            None => {
                self.begin(owner, snapshot());
                if self.owner() == Some(owner) {
                    write();
                    self.commit(owner);
                }
            }
            Some(current) if current == owner => write(),
            Some(_) => {}
        }
    }
}
