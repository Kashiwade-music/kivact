//! Role: Attach the toast swipe machine and its visual dismissal to the card.
use super::machine::{self, Event, edge_sign};
use kivact_core::custom::clock_now;
use kivact_core::geometry::{Edge, Rect};
use kivact_core::motion::Transition;
use kivact_core::state::{MachineHandle, Value, machine, memo};
use kivact_core::style::{Style, TransitionProp, pct, style, use_theme};
use kivact_core::task::spawn;
use kivact_core::view::{Handled, PointerEvent, PointerPhase};
use kivact_interaction::{InteractionPolicy, ToastId, ToasterApi};
#[derive(Clone, Copy)]
pub(crate) struct Swipe {
    edge: Edge,
    session: MachineHandle<machine::Machine>,
}
impl Swipe {
    pub(crate) fn new(
        edge: Edge,
        rect: Value<Option<Rect>>,
        toaster: ToasterApi,
        id: ToastId,
    ) -> Self {
        let session = machine(
            machine::Swipe::Rest,
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
                // A task runs in the same turn once the effects have settled:
                // the card's style has committed its released look (the swipe
                // exit and the theme's transition) before the card is removed,
                // so the exit starts from the drag offset instead of jumping
                // through the drag's zero-length transition.
                machine::Effect::Dismiss => {
                    spawn(async move { toaster.dismiss(id) });
                }
            },
        );
        Self { edge, session }
    }
    /// Whether a touch on the card holds its timer: from the press, through
    /// the drag and a qualifying swipe's exit, until the gesture resets.
    pub(crate) fn held(self) -> Value<bool> {
        let session = self.session;
        memo(move || {
            matches!(
                session.get(),
                machine::Swipe::Pressed(_)
                    | machine::Swipe::Dragging { .. }
                    | machine::Swipe::Released { .. }
            )
        })
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
            || matches!(before, machine::Swipe::Released { .. })
        {
            return Handled::No;
        }
        let handled = match event.phase() {
            PointerPhase::Move => {
                matches!(self.session.get(), machine::Swipe::Dragging { contact, .. } if contact.pointer == event.id())
            }
            PointerPhase::Up(_) => {
                matches!(before, machine::Swipe::Dragging { contact, .. } if contact.pointer == event.id())
            }
            _ => false,
        };
        if handled { Handled::Yes } else { Handled::No }
    }
    /// The style of the card's row. After a qualifying swipe the row closes
    /// the gap only once the card has slid off: its size transition waits
    /// for the swipe's exit (`exit`), so the row's collapse does not carry
    /// the card past the region's edge before it is seen leaving. A spring
    /// exit has no set length, and the row then collapses at once.
    pub(crate) fn row_style(self, exit: &Transition) -> Style {
        if !matches!(self.session.get(), machine::Swipe::Released { .. }) {
            return style();
        }
        let theme = use_theme().get();
        let row = theme
            .ext::<crate::UiTokens>()
            .toast
            .row_motion
            .transition
            .clone();
        match (row, exit.duration_ms()) {
            (Some(row), Some(wait)) => {
                let delay = row.delay_ms + wait;
                style().transition(TransitionProp::Size, row.delay(delay))
            }
            _ => style(),
        }
    }
    pub(crate) fn style(self, transition: &Transition) -> Style {
        let swipe = self.session.get();
        let timing = if matches!(swipe, machine::Swipe::Dragging { .. }) {
            Transition::tween(0.0, kivact_core::motion::Easing::Linear)
        } else {
            transition.clone()
        };
        let base = style().transition(TransitionProp::All, timing);
        let (travel, height, base) = match swipe {
            machine::Swipe::Dragging { contact, offset } => (offset, contact.height, base),
            // Only a qualifying swipe slides the card off; any other
            // dismissal keeps the card's own exit.
            machine::Swipe::Released { travel, height } => (
                travel,
                height,
                base.exit(
                    style()
                        .opacity(0.0)
                        .translate_y(pct(120.0 * edge_sign(self.edge)))
                        .scale(1.0),
                ),
            ),
            _ => return base,
        };
        let progress = (travel / height).clamp(0.0, 1.0);
        base.translate_y(travel * edge_sign(self.edge))
            .opacity(1.0 - 0.4 * progress)
            .scale(1.0 - 0.02 * progress)
    }
}
