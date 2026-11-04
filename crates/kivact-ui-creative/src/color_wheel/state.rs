//! Role: Color picker values and shared edit transaction.
use super::geometry::Channel;
use super::{color_from_hsv, color_to_hsv};
use kivact::interaction::DragApi;
use kivact::state::IntoValue;
use kivact::state::{Binding, Callback, MachineHandle, Value, machine, memo};
use kivact::style::Color;
use kivact::view::EditPhase;

type Snapshot = (Color, f64, Option<Vec<Color>>);
/// Who began the active edit of the shared session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColorOwner {
    Wheel,
    Field { hsv: bool, index: usize },
    Hex,
    Swatch,
}
#[derive(Clone, Copy)]
pub(super) struct ColorOptions {
    pub(super) color: Binding<Color>,
    pub(super) palette: Option<Binding<Vec<Color>>>,
    pub(super) blocked: Value<bool>,
}
#[derive(Clone, Copy)]
pub(super) struct Model {
    options: ColorOptions,
    pub(super) hue: Value<f64>,
    pub(super) edit: Edit,
    pub(super) controls: MachineHandle<controls_machine::Machine>,
}
impl std::ops::Deref for Model {
    type Target = ColorOptions;
    fn deref(&self) -> &ColorOptions {
        &self.options
    }
}
impl Model {
    pub(super) fn new(
        color: Binding<Color>,
        palette: Option<Binding<Vec<Color>>>,
        observer: Option<Callback<EditPhase>>,
        blocked: Value<bool>,
        drag: DragApi<Channel>,
    ) -> Self {
        let initial_hue = color_to_hsv(color.into_value().get_untracked()).0;
        let controls = machine(
            ColorControls {
                channel: Channel::Hue,
                swatch: SwatchSelection::None,
                hue: initial_hue,
            },
            || controls_machine::Machine,
            |_, ()| {},
        );
        controls.feed(color.into_value(), controls_machine::Event::ColorChanged);
        if let Some(palette) = palette {
            controls.feed(
                memo(move || palette.into_value().get().len()),
                controls_machine::Event::PaletteChanged,
            );
        }
        let hue = memo(move || controls.get().hue);
        // Rollback restores the remembered hue even when the restored color is gray.
        let edit = Edit::new(
            move |(original, h, swatches): Snapshot| {
                color.set(original);
                controls.send(controls_machine::Event::Hue(h));
                if let (Some(binding), Some(colors)) = (palette, swatches) {
                    binding.set(colors);
                }
            },
            observer,
            blocked,
            drag,
        );
        Self {
            options: ColorOptions {
                color,
                palette,
                blocked,
            },
            hue,
            edit,
            controls,
        }
    }
    /// What an edit writes back when it is canceled.
    pub(super) fn snapshot(self) -> Snapshot {
        (
            self.color.into_value().get_untracked(),
            self.hue.get_untracked(),
            self.palette.map(|p| p.into_value().get_untracked()),
        )
    }
    /// One whole edit of `owner` around `write`; nothing while the state is
    /// blocked or another source's edit runs.
    pub(super) fn once(self, owner: ColorOwner, write: impl FnOnce()) {
        if self.blocked.get_untracked() {
            return;
        }
        self.edit.once(owner, || self.snapshot(), write);
    }
    pub(super) fn hsv(self) -> (f64, f64, f64, f64) {
        let color = self.color.into_value().get_untracked();
        let (_, s, v) = color_to_hsv(color);
        (self.hue.get_untracked(), s, v, color.components()[3])
    }
    pub(super) fn channel(self, c: Channel) -> (f64, f64) {
        let (h, s, v, a) = self.hsv();
        match c {
            Channel::Hue => (h, h),
            Channel::Sv => (s, v),
            Channel::Alpha => (a, a),
        }
    }
    pub(super) fn set(self, c: Channel, x: f64, y: f64) {
        let (mut h, mut s, mut v, mut a) = self.hsv();
        match c {
            Channel::Hue => h = x.rem_euclid(360.0),
            Channel::Sv => {
                s = x.clamp(0.0, 1.0);
                v = y.clamp(0.0, 1.0)
            }
            Channel::Alpha => a = x.clamp(0.0, 1.0),
        }
        self.controls.send(controls_machine::Event::Hue(h));
        self.color.set(color_from_hsv(h, s, v, a));
    }
    /// A phase of an edit that `owner` reports.
    pub(super) fn phase(self, owner: ColorOwner, phase: EditPhase) {
        match phase {
            EditPhase::Begin => {
                if !self.blocked.get_untracked() {
                    self.edit.begin(owner, self.snapshot());
                }
            }
            EditPhase::Commit => self.edit.commit(owner),
            EditPhase::Cancel => {
                self.edit.cancel_owned(owner);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ColorControls {
    pub(super) channel: Channel,
    pub(super) swatch: SwatchSelection,
    pub(super) hue: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum SwatchSelection {
    None,
    Selected(usize),
}
impl SwatchSelection {
    pub(super) fn index(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Selected(index) => Some(index),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct State {
    model: Model,
    drag: DragApi<Channel>,
}
impl std::ops::Deref for State {
    type Target = Model;
    fn deref(&self) -> &Model {
        &self.model
    }
}
impl State {
    pub(super) fn new(model: Model, drag: DragApi<Channel>) -> Self {
        let state = Self { model, drag };
        kivact::state::on_cleanup(move || state.cancel());
        state
    }
    pub(super) fn cancel(self) {
        self.drag.cancel();
        self.edit.cancel();
    }
    pub(super) fn observe(
        self,
        focused: Value<bool>,
        canvas_blocked: Value<bool>,
        alpha: Value<bool>,
    ) {
        self.edit.handle.feed(self.blocked, machine::Event::Blocked);
        self.edit
            .handle
            .feed(canvas_blocked, machine::Event::Blocked);
        self.edit.handle.feed(focused, machine::Event::Focused);
        self.edit
            .handle
            .feed(alpha, move |shown| machine::Event::Alpha {
                shown,
                channel: self.controls.state().get_untracked().channel,
            });
        self.controls
            .feed(alpha, controls_machine::Event::AlphaChanged);
    }
}
pub(super) mod controls_machine {
    //! Role: Atomically retain valid channel, swatch and gray-preserving hue controls.
    use super::{Channel, ColorControls, SwatchSelection, color_to_hsv};
    use kivact::state::Step;
    use kivact::style::Color;
    pub(in crate::color_wheel) struct Machine;
    pub(in crate::color_wheel) enum Event {
        ColorChanged(Color),
        AlphaChanged(bool),
        PaletteChanged(usize),
        Channel(Channel),
        Swatch(SwatchSelection),
        Hue(f64),
    }
    impl kivact::state::Machine for Machine {
        type State = ColorControls;
        type Event = Event;
        type Effect = ();
        fn transition(
            &self,
            state: &ColorControls,
            event: Event,
        ) -> Step<ColorControls, Event, ()> {
            let mut next = *state;
            match event {
                Event::ColorChanged(color) => {
                    let (h, s, v) = color_to_hsv(color);
                    if s > 0.00001 && v > 0.00001 {
                        next.hue = h;
                    }
                }
                Event::AlphaChanged(false) if next.channel == Channel::Alpha => {
                    next.channel = Channel::Hue
                }
                Event::PaletteChanged(len) => {
                    if let SwatchSelection::Selected(index) = next.swatch {
                        next.swatch = if len == 0 {
                            SwatchSelection::None
                        } else {
                            SwatchSelection::Selected(index.min(len - 1))
                        };
                    }
                }
                Event::Channel(channel) => next.channel = channel,
                Event::Swatch(swatch) => next.swatch = swatch,
                Event::Hue(hue) => next.hue = hue,
                _ => {}
            }
            Step::to(next)
        }
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
        drag: DragApi<Channel>,
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
    pub(super) fn owner(self) -> Option<ColorOwner> {
        match self.handle.get() {
            machine::State::Idle => None,
            machine::State::Editing { owner, .. } => Some(owner),
        }
    }
    pub(super) fn is_active(self) -> bool {
        self.owner().is_some()
    }
    pub(super) fn begin(self, owner: ColorOwner, snapshot: Snapshot) {
        self.handle.send(machine::Event::Begin { owner, snapshot });
    }
    pub(super) fn commit(self, owner: ColorOwner) {
        self.handle.send(machine::Event::Commit(owner));
    }
    pub(super) fn cancel(self) -> bool {
        let active = self.is_active();
        self.handle.send(machine::Event::Cancel);
        active
    }
    pub(super) fn cancel_owned(self, owner: ColorOwner) -> bool {
        if self.owner() == Some(owner) {
            self.cancel()
        } else {
            false
        }
    }
    pub(super) fn once(
        self,
        owner: ColorOwner,
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
    use super::{ColorOwner, Snapshot};
    use kivact::state::Step;
    use kivact::view::EditPhase;
    #[derive(Clone, PartialEq)]
    pub(super) enum State {
        Idle,
        Editing {
            owner: ColorOwner,
            snapshot: Snapshot,
        },
    }
    pub(super) struct Machine {
        pub(super) blocked: bool,
    }
    pub(super) enum Event {
        Begin {
            owner: ColorOwner,
            snapshot: Snapshot,
        },
        Commit(ColorOwner),
        Cancel,
        Blocked(bool),
        Focused(bool),
        Alpha {
            shown: bool,
            channel: super::Channel,
        },
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
                (
                    State::Editing { owner, snapshot },
                    Event::Alpha {
                        shown: false,
                        channel,
                    },
                ) if channel == super::Channel::Alpha
                    || matches!(owner, super::ColorOwner::Field { index: 3, .. }) =>
                {
                    Step::to(State::Idle)
                        .with(Effect::StopDrag)
                        .with(Effect::Rollback(snapshot.clone()))
                        .with(Effect::Phase(EditPhase::Cancel))
                }
                _ => Step::stay(),
            }
        }
    }
}
