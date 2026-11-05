//! Role: Track a held menu pointer and activate the eligible row at release.
use super::press_machine::{Event, Input, PressGesture, Source};
use super::press_state::Press;
use super::session::Session;
use super::state::MenuOrigin;
use crate::PopupCloseReason;
use kivact_core::custom::clock_now;
use kivact_core::view::{Handled, MouseButton, PointerEvent, PointerKind, PointerPhase};

const SETTLE_NS: u64 = 300_000_000;

impl Press {
    fn begin<K: kivact_core::state::ItemKey>(
        self,
        session: Session<K>,
        source: Source,
        event: PointerEvent,
        button: MouseButton,
    ) -> Handled {
        let position = event.window_pos();
        if !event.is_primary() {
            self.cancel();
            return Handled::No;
        }
        if button != MouseButton::Primary
            || !matches!(self.machine.get().gesture, PressGesture::Idle)
        {
            return Handled::No;
        }
        let opened = session.is_open().get_untracked();
        if (source == Source::Menu) != opened
            || (source == Source::Context && event.kind() == PointerKind::Mouse)
        {
            return Handled::No;
        }
        let input = Input {
            id: event.id(),
            kind: event.kind(),
            source,
        };
        if source == Source::Menu || input.kind == PointerKind::Mouse {
            event.suppress_activation();
            if source == Source::Trigger {
                session.open_from_button();
            } else {
                self.machine
                    .send(Event::Highlight(self.hit(position).1.into()));
            }
            self.machine.send(Event::Open(
                input,
                position,
                clock_now(),
                source == Source::Menu,
            ));
            return Handled::Yes;
        }
        let callback = kivact_core::state::callback(move |()| {
            if session.is_open().get_untracked() {
                self.cancel();
                return;
            }
            event.suppress_activation();
            if source == Source::Context {
                session.open_at(MenuOrigin::Pointer(position));
            } else {
                session.open_from_button();
            }
        });
        self.machine.send(Event::Pending(input, position, callback));
        Handled::No
    }

    pub(super) fn event<K: kivact_core::state::ItemKey>(
        self,
        session: Session<K>,
        source: Source,
        event: PointerEvent,
    ) -> Handled {
        let position = event.window_pos();
        if let PointerPhase::Down(press) = event.phase() {
            return self.begin(session, source, event, press.button);
        }
        let gesture = self.machine.get().gesture;
        let input = match gesture {
            PressGesture::Idle => return Handled::No,
            PressGesture::Pending {
                source,
                kind,
                origin,
                ..
            } => Input {
                id: source,
                kind,
                source: origin,
            },
            PressGesture::Holding { input, .. } | PressGesture::Dragging { input, .. } => input,
        };
        if input.id != event.id() {
            return Handled::No;
        }
        if matches!(event.phase(), PointerPhase::Cancel) {
            self.cancel();
            return Handled::No;
        }
        match (event.phase(), gesture) {
            (PointerPhase::Move, PressGesture::Pending { from, .. }) => {
                let drift = kivact_core::style::use_theme()
                    .get()
                    .ext::<crate::InteractionPolicy>()
                    .menu
                    .drift
                    .clone();
                let drift = if input.source == Source::Context {
                    drift.context
                } else {
                    drift.other
                };
                if (position.x - from.x).hypot(position.y - from.y) > drift {
                    event.suppress_activation();
                    self.cancel();
                }
            }
            (PointerPhase::Up(_), PressGesture::Pending { .. }) => self.cancel(),
            (PointerPhase::Move, PressGesture::Holding { .. } | PressGesture::Dragging { .. }) => {
                let (inside, row) = self.hit(position);
                if inside || matches!(gesture, PressGesture::Dragging { .. }) {
                    self.machine.send(Event::Move(position, row.into()));
                }
                return Handled::Yes;
            }
            (
                PointerPhase::Up(press),
                PressGesture::Holding { since, .. } | PressGesture::Dragging { since, .. },
            ) if press.button == MouseButton::Primary => {
                event.suppress_activation();
                let (inside, row) = self.hit(position);
                self.cancel();
                if !settled(
                    matches!(gesture, PressGesture::Dragging { .. }),
                    clock_now().saturating_sub(since),
                ) {
                    return Handled::Yes;
                }
                if let Some(row) = row {
                    row.activate();
                } else if input.kind == PointerKind::Mouse
                    && !inside
                    && !session.anchor.hit_test(position)
                {
                    session.close(PopupCloseReason::OutsidePress);
                }
                return Handled::Yes;
            }
            _ => {}
        }
        Handled::No
    }
}

fn settled(entered: bool, elapsed_ns: u64) -> bool {
    entered || elapsed_ns >= SETTLE_NS
}

#[cfg(test)]
mod tests {
    //! Role: Check when drag release can pick a menu item.
    use super::{SETTLE_NS, settled};

    #[test]
    fn an_opening_press_can_pick_after_entry_or_the_settle_window() {
        assert!(settled(true, 0));
        assert!(!settled(false, SETTLE_NS - 1));
        assert!(settled(false, SETTLE_NS));
    }
}
