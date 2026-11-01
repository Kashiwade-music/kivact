//! Role: Mouse event production.
//! A backend reports the mouse as separate messages (a move with a
//! position, a button without one, a leave without either); [`Mouse`]
//! keeps the last position and the held buttons, so every event it makes
//! is a complete [`PointerInput`]. Hit testing and capture stay with the
//! host.

use kivact_types::protocol::PointerInput;
use kivact_types::{MouseButton, Point, PointerButtons, PointerId, PointerKind, PointerPhase};

/// One window's mouse: where it last was (logical pixels) and the buttons
/// held. Leaving the window keeps both, so a button released outside is
/// reported where the mouse left; a `Cancel` releases every button.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mouse {
    position: Point,
    positioned: bool,
    buttons: PointerButtons,
}

impl Mouse {
    /// The last known logical position, or `None` before any position arrived.
    pub fn position(&self) -> Option<Point> {
        self.positioned.then_some(self.position)
    }

    /// The `Cancel` that ends the gesture when the window loses focus with
    /// buttons held, or `None` when none is held (there is nothing to end).
    pub fn cancel(&mut self) -> Option<PointerInput> {
        (!self.buttons.is_empty()).then(|| self.event(PointerPhase::Cancel, None, None))
    }

    /// Note a complete mouse event made elsewhere (one injected into a
    /// test window), so the next event made here continues from it.
    pub fn observe(&mut self, input: PointerInput) {
        if input.kind == PointerKind::Mouse {
            self.position = input.position;
            self.positioned = true;
            self.buttons = input.buttons;
        }
    }

    /// The mouse event of `phase` at `position` (`None`: where the mouse
    /// last was), with `button` pressed by a `Down` or released by an
    /// `Up`; other phases ignore `button`. Its click count is 1 until a
    /// `ClickCounter` counts it.
    pub fn event(
        &mut self,
        phase: PointerPhase,
        position: Option<Point>,
        button: Option<MouseButton>,
    ) -> PointerInput {
        if let Some(position) = position {
            self.position = position;
            self.positioned = true;
        }
        let button = button.filter(|_| matches!(phase, PointerPhase::Down | PointerPhase::Up));
        match (phase, button) {
            (PointerPhase::Down, Some(button)) => self.buttons = self.buttons.with(button, true),
            (PointerPhase::Up, Some(button)) => self.buttons = self.buttons.with(button, false),
            (PointerPhase::Cancel, _) => self.buttons = PointerButtons::NONE,
            _ => {}
        }
        PointerInput {
            id: PointerId::MOUSE,
            kind: PointerKind::Mouse,
            phase,
            buttons: self.buttons,
            button,
            position: self.position,
            pressure: None,
            tilt: None,
            clicks: 1,
        }
    }
}
