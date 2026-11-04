//! Role: Attach the toast swipe machine and its visual dismissal to the card.
use super::machine::{self, Event, State, Visual, edge_sign};
use kivact_core::custom::clock_now;
use kivact_core::geometry::{Edge, Rect};
use kivact_core::motion::{FrameRate, Transition, frame_loop};
use kivact_core::state::{MachineHandle, Signal, Value, effect, machine};
use kivact_core::style::{Style, TransitionProp, pct, style, use_theme};
use kivact_core::view::{Handled, PointerEvent, PointerPhase};
use kivact_interaction::{InteractionPolicy, ToastId, Toaster};
#[derive(Clone, Copy)]
pub(crate) struct Swipe {
    edge: Edge,
    session: MachineHandle<machine::Machine>,
}
impl Swipe {
    pub(crate) fn new(
        edge: Edge,
        rect: Value<Option<Rect>>,
        held: Signal<bool>,
        toaster: Toaster,
        id: ToastId,
    ) -> Self {
        let session = machine(
            State {
                swipe: machine::Swipe::Rest,
                exit: Visual::Rest,
            },
            move || machine::Machine {
                edge,
                rect: rect.get(),
                now: clock_now(),
                slop: use_theme()
                    .get()
                    .ext::<InteractionPolicy>()
                    .toast
                    .swipe_slop,
            },
            move |_, effect| match effect {
                machine::Effect::Hold(value) => held.set(value),
            },
        );
        // Preserve the committed exit target until I-7 replaces visual dismissal.
        let clock = frame_loop(move |_| {
            if matches!(session.get().exit, Visual::DismissOn { .. }) {
                toaster.dismiss(id);
            }
        });
        effect(move || {
            if matches!(session.get().exit, Visual::DismissOn { .. }) {
                clock.start(FrameRate::Display);
            } else {
                clock.stop();
            }
        });
        Self { edge, session }
    }
    pub(crate) fn protect_action(self, event: PointerEvent) {
        if matches!(event.phase(), PointerPhase::Down(_)) {
            self.session.send(Event::Protect(event.id()));
        }
    }
    pub(crate) fn event(self, event: PointerEvent) -> Handled {
        let before = self.session.get();
        self.session.send(Event::Pointer(event));
        if !matches!(
            event.kind(),
            kivact_core::view::PointerKind::Touch | kivact_core::view::PointerKind::Pen
        ) || !event.window_pos().x.is_finite()
            || !event.window_pos().y.is_finite()
            || matches!(before.exit, Visual::DismissOn { .. })
        {
            return Handled::No;
        }
        let handled = match event.phase() {
            PointerPhase::Move => {
                matches!(self.session.get().swipe, machine::Swipe::Dragging { contact, .. } if contact.pointer == event.id())
            }
            PointerPhase::Up(_) => {
                matches!(before.swipe, machine::Swipe::Dragging { contact, .. } if contact.pointer == event.id())
            }
            _ => false,
        };
        if handled { Handled::Yes } else { Handled::No }
    }
    pub(crate) fn style(self, transition: &Transition) -> Style {
        let visual = self.session.get().visual();
        let timing = if matches!(visual, Visual::Drag { .. }) {
            Transition::tween(0.0, kivact_core::motion::Easing::Linear)
        } else {
            transition.clone()
        };
        let base = style().transition(TransitionProp::Paint, timing);
        match visual {
            Visual::Rest => base,
            Visual::Drag { travel, height } | Visual::DismissOn { travel, height } => {
                let progress = (travel / height).clamp(0.0, 1.0);
                let card = base
                    .translate_y(travel * edge_sign(self.edge))
                    .opacity(1.0 - 0.4 * progress)
                    .scale(1.0 - 0.02 * progress);
                if matches!(visual, Visual::DismissOn { .. }) {
                    card.exit(
                        style()
                            .opacity(0.0)
                            .translate_y(pct(120.0 * edge_sign(self.edge)))
                            .scale(1.0),
                    )
                } else {
                    card
                }
            }
        }
    }
}
