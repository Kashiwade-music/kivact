//! Role: Track a held menu pointer and activate the eligible row at release.
use super::press_state::{Gesture, Phase, Press, Source};
use super::state::{MenuOrigin, Session};
use crate::PopupCloseReason;
use kivact_core::primitive::clock_now;
use kivact_core::task::{sleep, spawn};
use kivact_core::view::{Handled, MouseButton, PointerEvent, PointerKind, PointerPhase};
use std::hash::Hash;
use std::time::Duration;

const HOLD: Duration = Duration::from_millis(500);
const SETTLE_NS: u64 = 300_000_000;

impl Press {
    pub(super) fn event<K: Clone + Eq + Hash + 'static>(
        self,
        session: Session<K>,
        source: Source,
        event: PointerEvent,
    ) -> Handled {
        let position = event.window_pos();
        if let PointerPhase::Down(press) = event.phase() {
            if !event.is_primary() {
                self.cancel();
                return Handled::No;
            }
            if press.button != MouseButton::Primary || self.gesture.get().is_some() {
                return Handled::No;
            }
            let opened = session.shown().get_untracked();
            if (source == Source::Menu) != opened
                || (source == Source::Context && event.kind() == PointerKind::Mouse)
            {
                return Handled::No;
            }
            let id = event.id();
            let kind = event.kind();
            let phase = if source == Source::Menu || kind == PointerKind::Mouse {
                event.suppress_activation();
                if source == Source::Trigger {
                    session.open_from_button();
                } else {
                    self.highlight.set(Some(self.hit(position).1));
                }
                Phase::Open {
                    at: clock_now(),
                    entered: source == Source::Menu,
                }
            } else {
                let timer = spawn(async move {
                    sleep(HOLD).await;
                    if let Some(mut gesture) = self.gesture.get()
                        && gesture.id == id
                        && matches!(gesture.phase, Phase::Pending { .. })
                        && !session.shown().get_untracked()
                    {
                        event.suppress_activation();
                        gesture.phase = Phase::Open {
                            at: clock_now(),
                            entered: false,
                        };
                        self.gesture.set(Some(gesture));
                        if source == Source::Context {
                            session.open_at(MenuOrigin::Pointer(position));
                        } else {
                            session.open_from_button();
                        }
                    }
                });
                Phase::Pending {
                    timer,
                    from: position,
                }
            };
            self.gesture.set(Some(Gesture {
                id,
                kind,
                phase,
                source,
            }));
            return if matches!(phase, Phase::Open { .. }) {
                Handled::Yes
            } else {
                Handled::No
            };
        }
        let Some(mut gesture) = self
            .gesture
            .get()
            .filter(|gesture| gesture.id == event.id())
        else {
            return Handled::No;
        };
        if matches!(event.phase(), PointerPhase::Cancel) {
            self.cancel();
            return Handled::No;
        }
        match (event.phase(), gesture.phase) {
            (PointerPhase::Move, Phase::Pending { from, .. }) => {
                let drift = if gesture.source == Source::Context {
                    10.0
                } else {
                    13.0
                };
                if (position.x - from.x).hypot(position.y - from.y) > drift {
                    event.suppress_activation();
                    self.cancel();
                }
            }
            (PointerPhase::Up(_), Phase::Pending { .. }) => self.cancel(),
            (PointerPhase::Move, Phase::Open { at, entered }) => {
                let (inside, row) = self.hit(position);
                if inside || entered {
                    gesture.phase = Phase::Open { at, entered: true };
                    self.gesture.set(Some(gesture));
                    self.highlight.set(Some(row));
                }
                return Handled::Yes;
            }
            (PointerPhase::Up(press), Phase::Open { at, entered })
                if press.button == MouseButton::Primary =>
            {
                event.suppress_activation();
                let (inside, row) = self.hit(position);
                self.cancel();
                if !settled(entered, clock_now().saturating_sub(at)) {
                    return Handled::Yes;
                }
                if let Some(row) = row {
                    row.activate();
                } else if gesture.kind == PointerKind::Mouse
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
