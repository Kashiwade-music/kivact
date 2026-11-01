//! Role: Normalizes platform touch and pen contacts into shell pointer events.
//!
//! Backends supply logical coordinates, pressure, tilt and cancellation in
//! `PointerInput`. The source id identifies a platform contact until it ends;
//! this normalizer assigns a stable identity for its delivered lifetime.
//!
//! Invariants:
//! - Every contact gets a new [`PointerId`] at its `Down` (from 2 on; 1 is
//!   the mouse), kept until its `Up` or `Cancel`; ids are not reused.
//! - A contact's kind is fixed at its `Down`.
//! - Every contact that began ends with one `Up` or `Cancel`: a `Down` of
//!   a platform id still in contact (its end was lost) cancels the old
//!   contact before the new one begins, and a window losing the focus
//!   cancels every contact in progress.
//! - Moves of a pointer that is not in contact (a pen hovering) and ends of
//!   contacts that never began are not reported.
//! - A test window delivers its injected contacts by the same rules
//!   ([`Contacts::deliver`], [`Contacts::cancel`]).
//!
use crate::events::ShellEvent;
use kivact_types::{Point, PointerButtons, PointerId, PointerInput, PointerKind, PointerPhase};

/// A contact in progress.
#[derive(Debug)]
struct Live {
    /// The platform's id of the contact.
    platform: u64,
    /// The contact's own pointer id and kind.
    id: PointerId,
    kind: PointerKind,
    /// Where it was last reported (logical window pixels).
    at: Point,
}

/// The touch and pen contacts in progress of one window: the platform's
/// contacts as the window's pointers (see the module invariants). A native
/// window supplies decoded platform input; a test window the contacts a
/// test injects (`arvane::test_display`).
#[derive(Debug)]
pub struct Contacts {
    live: Vec<Live>,
    /// The id the next contact gets.
    next: u64,
}

impl Default for Contacts {
    fn default() -> Self {
        Self {
            live: Vec::new(),
            next: PointerId::MOUSE.0 + 1,
        }
    }
}

impl Contacts {
    /// The events the window delivers for `platform`, a contact as the
    /// platform reports it: its `id` is the platform's id of the contact
    /// (which the platform may reuse once the contact ended), its `kind`
    /// counts at its `Down` only (a mouse is taken for a touch). The events
    /// carry the contact's own pointer id and kind: none for a move or an
    /// end of a contact that never began; a `Cancel` of the old contact
    /// before the new one's `Down` for a `Down` of a platform id still in
    /// contact (the platform lost its end).
    pub fn deliver(&mut self, platform: PointerInput) -> Vec<ShellEvent> {
        let at = self.live.iter().position(|c| c.platform == platform.id.0);
        let mut events = Vec::new();
        let (id, kind) = match (platform.phase, at) {
            (PointerPhase::Down, at) => {
                if let Some(at) = at {
                    events.push(cancel(self.live.remove(at)));
                }
                let id = PointerId(self.next);
                self.next += 1;
                let kind = match platform.kind {
                    PointerKind::Pen => PointerKind::Pen,
                    PointerKind::Touch | PointerKind::Mouse => PointerKind::Touch,
                };
                self.live.push(Live {
                    platform: platform.id.0,
                    id,
                    kind,
                    at: platform.position,
                });
                (id, kind)
            }
            (PointerPhase::Move, Some(at)) => {
                let live = &mut self.live[at];
                live.at = platform.position;
                (live.id, live.kind)
            }
            (PointerPhase::Up | PointerPhase::Cancel, Some(at)) => {
                let live = self.live.remove(at);
                (live.id, live.kind)
            }
            // A window reports contacts, not a pen hovering (`Enter`,
            // `Leave`), nor a move or an end of one that never began.
            (PointerPhase::Enter | PointerPhase::Leave, _)
            | (PointerPhase::Move | PointerPhase::Up | PointerPhase::Cancel, None) => {
                return events;
            }
        };
        events.push(ShellEvent::Pointer(PointerInput {
            id,
            kind,
            ..platform
        }));
        events
    }

    /// The window lost the focus (and with it the pointer capture): every
    /// contact in progress ends with a `Cancel` where it was last reported.
    pub fn cancel(&mut self) -> Vec<ShellEvent> {
        self.live.drain(..).map(cancel).collect()
    }
}

/// The `Cancel` of `live` where it was last reported.
fn cancel(live: Live) -> ShellEvent {
    ShellEvent::Pointer(PointerInput {
        id: live.id,
        kind: live.kind,
        phase: PointerPhase::Cancel,
        position: live.at,
        buttons: PointerButtons::NONE,
        button: None,
        pressure: None,
        tilt: None,
        clicks: 1,
    })
}
