//! Role: Gradient model writes and one shared edit transaction.
use super::data::{GradientStop, position_value};
use kivact::interaction::DragApi;
use kivact::state::IntoValue;

use kivact::state::{Binding, Callback, MachineHandle, Value};
use kivact::style::Color;
use kivact::view::EditPhase;
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
pub(super) struct GradientOptions {
    pub(super) stops: Binding<Vec<GradientStop>>,
    pub(super) selection: Binding<Option<usize>>,
    pub(super) blocked: Value<bool>,
    pub(super) valid: Value<bool>,
}
#[derive(Clone, Copy)]
pub(super) struct Model {
    options: GradientOptions,
    pub(super) edit: Edit,
}
impl std::ops::Deref for Model {
    type Target = GradientOptions;
    fn deref(&self) -> &GradientOptions {
        &self.options
    }
}
impl Model {
    pub(super) fn new(
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
    pub(super) fn enabled(self) -> bool {
        !self.blocked.get_untracked() && self.valid.get_untracked()
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

#[derive(Clone, Copy)]
pub(super) struct State {
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
    pub(super) fn new(model: Model, drag: DragApi<(usize, f64, f64)>) -> Self {
        let state = Self { model, drag };
        kivact::state::on_cleanup(move || state.cancel());
        state
    }
    pub(super) fn cancel(self) {
        self.drag.cancel();
        self.edit.cancel();
    }
    pub(super) fn observe(self, focused: Value<bool>) {
        self.edit.handle.feed(
            kivact::state::memo(move || self.blocked.get() || !self.valid.get()),
            machine::Event::Blocked,
        );
        self.edit.handle.feed(focused, machine::Event::Focused);
    }
}

#[derive(Clone, Copy)]
pub(super) struct Edit {
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
                machine::Effect::StopDrag => drag.cancel(),
            },
        );
        let edit = Self { handle };
        kivact::state::on_cleanup(move || {
            edit.cancel();
        });
        edit
    }
    pub(super) fn owner(self) -> Option<GradientOwner> {
        match self.handle.get() {
            machine::State::Idle => None,
            machine::State::Editing { owner, .. } => Some(owner),
        }
    }
    pub(super) fn is_active(self) -> bool {
        self.owner().is_some()
    }
    pub(super) fn begin(self, owner: GradientOwner, snapshot: Snapshot) {
        self.handle.send(machine::Event::Begin { owner, snapshot });
    }
    pub(super) fn commit(self, owner: GradientOwner) {
        self.handle.send(machine::Event::Commit(owner));
    }
    pub(super) fn cancel(self) -> bool {
        let active = self.is_active();
        self.handle.send(machine::Event::Cancel);
        active
    }
    pub(super) fn cancel_owned(self, owner: GradientOwner) -> bool {
        if self.owner() == Some(owner) {
            self.cancel()
        } else {
            false
        }
    }
    pub(super) fn once(
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
mod machine {
    //! Role: Own the edit snapshot and decide rollback and drag cancellation in one transition.
    use super::{GradientOwner, Snapshot};
    use kivact::state::Step;
    use kivact::view::EditPhase;
    #[derive(Clone, PartialEq)]
    pub(super) enum State {
        Idle,
        Editing {
            owner: GradientOwner,
            snapshot: Snapshot,
        },
    }
    pub(super) struct Machine {
        pub(super) blocked: bool,
    }
    pub(super) enum Event {
        Begin {
            owner: GradientOwner,
            snapshot: Snapshot,
        },
        Commit(GradientOwner),
        Cancel,
        Blocked(bool),
        Focused(bool),
    }
    pub(super) enum Effect {
        Phase(EditPhase),
        Rollback(Snapshot),
        StopDrag,
    }
    impl kivact::state::Machine for Machine {
        type State = State;
        type Event = Event;
        type Effect = Effect;
        fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
            match (state, event) {
                (State::Idle, Event::Begin { owner, snapshot }) if !self.blocked => {
                    Step::to(State::Editing { owner, snapshot })
                        .with(Effect::Phase(EditPhase::Begin))
                }
                (State::Editing { owner, .. }, Event::Commit(by)) if *owner == by => {
                    Step::to(State::Idle).with(Effect::Phase(EditPhase::Commit))
                }
                (
                    State::Editing { snapshot, .. },
                    Event::Cancel | Event::Blocked(true) | Event::Focused(false),
                ) => Step::to(State::Idle)
                    .with(Effect::StopDrag)
                    .with(Effect::Rollback(snapshot.clone()))
                    .with(Effect::Phase(EditPhase::Cancel)),
                _ => Step::stay(),
            }
        }
    }
}
