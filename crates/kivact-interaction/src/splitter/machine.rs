//! Role: Preserve collapsed sizes and settle splitter drags at normalized stops.
use super::{ScalarRange, SplitterOptions};
use crate::nearest_snap;
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
    pub options: SplitterOptions,
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
                if !matches!(state.panel, PanelSize::Collapsed { .. }) || value != self.range.min()
                {
                    next.panel = PanelSize::Expanded(value);
                }
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
                let scale = self
                    .options
                    .scale
                    .map_or(1.0, |scale| scale.get_untracked());
                Step::stay().with(Effect::Write(origin + travel * scale))
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
                if let Some(stops) = self.options.snaps {
                    let stops: Vec<f64> = stops
                        .get_untracked()
                        .into_iter()
                        .filter(|stop| stop.is_finite() && (0.0..=1.0).contains(stop))
                        .collect();
                    if let Some(index) = nearest_snap(ratio, &stops) {
                        let value = self.range.min() + span * stops[index];
                        next.panel = PanelSize::Expanded(value);
                        return Step::to(next).with(Effect::Write(value));
                    }
                }
                next.panel = PanelSize::Expanded(self.value);
                Step::to(next)
            }
            Event::Toggle => {
                let value = match state.panel {
                    PanelSize::Collapsed { restore } if self.value == self.range.min() => {
                        next.panel = PanelSize::Expanded(restore);
                        restore
                    }
                    _ => {
                        next.panel = PanelSize::Collapsed {
                            restore: self.value,
                        };
                        self.range.min()
                    }
                };
                Step::to(next).with(Effect::Write(value))
            }
        }
    }
}
