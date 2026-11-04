//! Role: Color picker values and shared edit transaction.
use super::super::geometry::Channel;
use super::super::{color_from_hsv, color_to_hsv};
use super::data::{ColorControls, ColorOwner, Snapshot, SwatchSelection};
use super::{controls_machine, machine};
use kivact::interaction::DragApi;
use kivact::state::IntoValue;
use kivact::state::{Binding, Callback, MachineHandle, Value, machine, memo};
use kivact::style::Color;
use kivact::view::EditPhase;

#[derive(Clone, Copy)]
pub(in crate::color_wheel) struct ColorOptions {
    pub(in crate::color_wheel) color: Binding<Color>,
    pub(in crate::color_wheel) palette: Option<Binding<Vec<Color>>>,
    pub(in crate::color_wheel) blocked: Value<bool>,
}
#[derive(Clone, Copy)]
pub(in crate::color_wheel) struct Model {
    options: ColorOptions,
    pub(in crate::color_wheel) hue: Value<f64>,
    pub(in crate::color_wheel) edit: Edit,
    pub(in crate::color_wheel) controls: MachineHandle<controls_machine::Machine>,
}
impl std::ops::Deref for Model {
    type Target = ColorOptions;
    fn deref(&self) -> &ColorOptions {
        &self.options
    }
}
impl Model {
    pub(in crate::color_wheel) fn new(
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
    pub(in crate::color_wheel) fn snapshot(self) -> Snapshot {
        (
            self.color.into_value().get_untracked(),
            self.hue.get_untracked(),
            self.palette.map(|p| p.into_value().get_untracked()),
        )
    }
    /// One whole edit of `owner` around `write`; nothing while the state is
    /// blocked or another source's edit runs.
    pub(in crate::color_wheel) fn once(self, owner: ColorOwner, write: impl FnOnce()) {
        if self.blocked.get_untracked() {
            return;
        }
        self.edit.once(owner, || self.snapshot(), write);
    }
    pub(in crate::color_wheel) fn hsv(self) -> (f64, f64, f64, f64) {
        let color = self.color.into_value().get_untracked();
        let (_, s, v) = color_to_hsv(color);
        (self.hue.get_untracked(), s, v, color.components()[3])
    }
    pub(in crate::color_wheel) fn channel(self, c: Channel) -> (f64, f64) {
        let (h, s, v, a) = self.hsv();
        match c {
            Channel::Hue => (h, h),
            Channel::Sv => (s, v),
            Channel::Alpha => (a, a),
        }
    }
    pub(in crate::color_wheel) fn set(self, c: Channel, x: f64, y: f64) {
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
    pub(in crate::color_wheel) fn phase(self, owner: ColorOwner, phase: EditPhase) {
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

#[derive(Clone, Copy)]
pub(in crate::color_wheel) struct State {
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
    pub(in crate::color_wheel) fn new(model: Model, drag: DragApi<Channel>) -> Self {
        let state = Self { model, drag };
        kivact::state::on_cleanup(move || state.cancel());
        state
    }
    pub(in crate::color_wheel) fn cancel(self) {
        self.drag.cancel();
        self.edit.cancel();
    }
    pub(in crate::color_wheel) fn observe(
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

#[derive(Clone, Copy)]
pub(in crate::color_wheel) struct Edit {
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
    pub(in crate::color_wheel) fn owner(self) -> Option<ColorOwner> {
        match self.handle.get() {
            machine::State::Idle => None,
            machine::State::Editing { owner, .. } => Some(owner),
        }
    }
    pub(in crate::color_wheel) fn is_active(self) -> bool {
        self.owner().is_some()
    }
    pub(in crate::color_wheel) fn begin(self, owner: ColorOwner, snapshot: Snapshot) {
        self.handle.send(machine::Event::Begin { owner, snapshot });
    }
    pub(in crate::color_wheel) fn commit(self, owner: ColorOwner) {
        self.handle.send(machine::Event::Commit(owner));
    }
    pub(in crate::color_wheel) fn cancel(self) -> bool {
        let active = self.is_active();
        self.handle.send(machine::Event::Cancel);
        active
    }
    pub(in crate::color_wheel) fn cancel_owned(self, owner: ColorOwner) -> bool {
        if self.owner() == Some(owner) {
            self.cancel()
        } else {
            false
        }
    }
    pub(in crate::color_wheel) fn once(
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
