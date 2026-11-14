//! Role: Adapt raw touch pointer events and the display clock to scalar transitions.
pub use super::touch_data::TouchSliderGesture;
use super::touch_data::{Contact, Travel};
use super::touch_machine::{Effect, Event, Machine, State};
use super::track::{Grip, Pointer};
use crate::DragVelocity;
use crate::edit::EditSession;
use kivact_core::custom::clock_now;
use kivact_core::motion::{FrameRate, frame_loop};
use kivact_core::state::{MachineHandle, Value, effect_on, machine, memo};
use kivact_core::view::{Handled, PointerEvent, PointerKind, PointerPhase};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Copy)]
pub(crate) struct Touch {
    pointer: Pointer,
    session: EditSession<[f64; 2]>,
    machine: MachineHandle<Machine>,
}
impl Touch {
    pub(crate) fn new(
        config: TouchSliderGesture,
        pointer: Pointer,
        session: EditSession<[f64; 2]>,
        snapshot: Value<[f64; 2]>,
    ) -> Self {
        let machine = machine(
            State::Idle,
            move || Machine {
                config,
                geometry: pointer.geometry(),
                thumbs: pointer.thumbs.get(),
                snapshot: snapshot.get_untracked(),
            },
            move |cx, effect| match effect {
                Effect::Begin(values) => session.begin(values),
                Effect::Write(target, position) => target.set_position(position),
                Effect::Commit => session.commit(),
                Effect::Cancel => {
                    session.cancel();
                }
                Effect::Tap(index, along, values) => {
                    session.once(values, || pointer.write(index, along))
                }
                Effect::Release {
                    index,
                    now,
                    units,
                    sign,
                    speed_px,
                } => cx.send(Event::Release {
                    index,
                    now,
                    units,
                    sign,
                    speed_px,
                }),
            },
        );
        let clock = frame_loop(move |frame| {
            machine.send(Event::Frame(
                frame.now.as_nanos().min(u128::from(u64::MAX)) as u64
            ));
        });
        effect_on(
            move || matches!(machine.get(), State::Coasting { .. }),
            move |running| {
                if *running {
                    clock.start(FrameRate::Display);
                } else {
                    clock.stop();
                }
            },
        );
        Self {
            pointer,
            session,
            machine,
        }
    }
    pub(crate) fn lifted(self, index: usize) -> Value<bool> {
        memo(move || match self.machine.get() {
            State::Pressed { contact } | State::Tracking { contact, .. } => {
                contact.grip == Grip::Thumb(index)
            }
            _ => false,
        })
    }
    pub(crate) fn finish(self, restore: bool) -> bool {
        let active = !matches!(self.machine.get(), State::Idle);
        self.machine.send(Event::Finish(restore));
        active
    }
    pub(crate) fn event(self, grip: Grip, event: PointerEvent) -> Handled {
        if event.kind() != PointerKind::Touch {
            if matches!(event.phase(), PointerPhase::Down(_)) {
                self.finish(false);
            }
            return Handled::No;
        }
        if self.pointer.geometry().is_none() {
            return Handled::No;
        }
        let position = event.window_pos();
        if !position.x.is_finite() || !position.y.is_finite() {
            self.finish(true);
            return Handled::Yes;
        }
        let now = clock_now();
        if matches!(event.phase(), PointerPhase::Down(_)) {
            if !event.is_primary() {
                self.finish(false);
                return Handled::Yes;
            }
            self.finish(false);
            if self.session.is_active() {
                return Handled::Yes;
            }
            let initial = match grip {
                Grip::Thumb(index) => {
                    let Some(target) = self.pointer.thumbs.get().get(index).copied() else {
                        return Handled::Yes;
                    };
                    target.position(target.binding.get_untracked())
                }
                _ => 0.0,
            };
            let mut velocity = DragVelocity::new(Duration::from_millis(100));
            velocity.reset(now, position);
            self.machine.send(Event::Press(Contact {
                id: event.id(),
                grip,
                origin: position,
                initial,
                travel: Travel::Still,
                velocity: Rc::new(velocity),
            }));
        } else {
            self.machine.send(Event::Pointer {
                id: event.id(),
                phase: event.phase(),
                position,
                now,
            });
        }
        Handled::Yes
    }
}
