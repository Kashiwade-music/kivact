//! Role: Preserve collapsed sizes and settle splitter drags at normalized stops.
use super::options::Input;
use crate::nearest_snap;
use crate::policy::scalar::ScalarRange;
use kivact_core::state::{Machine as MachineTrait, Step};
#[derive(Clone, Copy, PartialEq)]
pub(super) enum PanelSize {
    Expanded(f64),
    Collapsed { restore: f64 },
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Drag {
    Idle,
    Dragging { origin: f64 },
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct State {
    pub panel: PanelSize,
    pub drag: Drag,
}
pub(super) enum Event {
    Observe(f64),
    Begin,
    Move(f64),
    End(bool),
    Toggle,
    Disabled(bool),
}
pub(super) enum Effect {
    Write(f64),
}
pub(super) struct Machine {
    pub options: Input,
    pub range: ScalarRange,
    pub value: f64,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let mut next = *state;
        match event {
            Event::Disabled(false) => Step::stay(),
            Event::Disabled(true) => {
                next.drag = Drag::Idle;
                Step::to(next)
            }
            Event::Observe(value) => {
                next.panel = self.panel_at(state, value);
                Step::to(next)
            }
            Event::Begin => {
                next.drag = Drag::Dragging { origin: self.value };
                Step::to(next)
            }
            Event::Move(travel) => {
                let Drag::Dragging { origin } = state.drag else {
                    return Step::stay();
                };
                Step::stay().with(Effect::Write(origin + travel * self.options.scale))
            }
            Event::End(commit) => {
                next.drag = Drag::Idle;
                if !commit {
                    return Step::to(next);
                }
                let span = self.range.max() - self.range.min();
                let ratio = if span > 0.0 {
                    (self.value - self.range.min()) / span
                } else {
                    0.0
                };
                if self
                    .options
                    .collapse
                    .is_some_and(|threshold| ratio < threshold)
                {
                    let restore = match state.drag {
                        Drag::Dragging { origin } => origin,
                        Drag::Idle => self.value,
                    };
                    next.panel = PanelSize::Collapsed { restore };
                    return Step::to(next).with(Effect::Write(self.range.min()));
                }
                if let Some(index) = nearest_snap(ratio, &self.options.snaps) {
                    let value = self.range.min() + span * self.options.snaps[index];
                    next.panel = self.panel_at(state, value);
                    return Step::to(next).with(Effect::Write(value));
                }
                next.panel = self.panel_at(state, self.value);
                Step::to(next)
            }
            Event::Toggle => {
                let value = if self.value == self.range.min() {
                    let restore = self.restore(state);
                    next.panel = PanelSize::Expanded(restore);
                    restore
                } else {
                    next.panel = PanelSize::Collapsed {
                        restore: self.value,
                    };
                    self.range.min()
                };
                Step::to(next).with(Effect::Write(value))
            }
        }
    }
}

impl Machine {
    fn restore(&self, state: &State) -> f64 {
        match state.drag {
            Drag::Dragging { origin } if origin > self.range.min() => origin,
            _ => match state.panel {
                PanelSize::Collapsed { restore } => restore,
                PanelSize::Expanded(value) if value > self.range.min() => value,
                PanelSize::Expanded(_) => {
                    self.range.min() + (self.range.max() - self.range.min()) / 2.0
                }
            },
        }
    }
    fn panel_at(&self, state: &State, value: f64) -> PanelSize {
        if value == self.range.min() {
            PanelSize::Collapsed {
                restore: self.restore(state),
            }
        } else {
            PanelSize::Expanded(value)
        }
    }
}
