//! Role: Mouse event production.
//! A backend reports the mouse as separate messages (a move with a
//! position, a button without one, a leave without either); [`Mouse`]
//! keeps the last position and the held buttons, so every event it makes
//! is a complete [`PointerInput`]. Hit testing and capture stay with the
//! host.

use kivact_types::protocol::PointerInput;
use kivact_types::{
    ButtonPress, MouseButton, Point, PointerButtons, PointerId, PointerKind, PointerPhase,
};

/// One window's mouse: where it last was (logical pixels) and the buttons
/// held. Leaving the window keeps both, so a button released outside is
/// reported where the mouse left; a `Cancel` releases every button.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mouse {
    position: Option<Point>,
    buttons: PointerButtons,
}

impl Mouse {
    /// The last known logical position, or `None` before any position arrived.
    pub fn position(&self) -> Option<Point> {
        self.position
    }

    /// The `Cancel` that ends the gesture when the window loses focus with
    /// buttons held, or `None` when none is held (there is nothing to end).
    pub fn cancel(&mut self) -> Option<PointerInput> {
        if self.buttons.is_empty() {
            return None;
        }
        self.event(PointerPhase::Cancel, None)
    }

    /// Note a complete mouse event made elsewhere (one injected into a
    /// test window), so the next event made here continues from it.
    pub fn observe(&mut self, input: PointerInput) {
        if input.kind == PointerKind::Mouse {
            self.position = Some(input.position);
            self.buttons = input.buttons;
        }
    }

    /// The `Down` (`pressed`) or `Up` of `button` at `position` (`None`: where
    /// the mouse last was), counted as a single click until a `ClickCounter`
    /// counts it. `None` as for [`event`](Self::event).
    pub fn button(
        &mut self,
        button: MouseButton,
        pressed: bool,
        position: Option<Point>,
    ) -> Option<PointerInput> {
        let press = ButtonPress { button, clicks: 1 };
        let phase = if pressed {
            PointerPhase::Down(press)
        } else {
            PointerPhase::Up(press)
        };
        self.event(phase, position)
    }

    /// The mouse event of `phase` at `position` (`None`: where the mouse
    /// last was). A `Down` adds its button to the held ones, an `Up` removes
    /// it, a `Cancel` releases all.
    ///
    /// Returns `None` when `position` is `None` and no position arrived yet: the platform reported a
    /// button or leave of a mouse it never placed in the window, so there is no point to report it at
    /// (its buttons stay unchanged).
    pub fn event(&mut self, phase: PointerPhase, position: Option<Point>) -> Option<PointerInput> {
        if let Some(position) = position {
            self.position = Some(position);
        }
        let at = self.position?;
        match phase {
            PointerPhase::Down(press) => self.buttons = self.buttons.with(press.button, true),
            PointerPhase::Up(press) => self.buttons = self.buttons.with(press.button, false),
            PointerPhase::Cancel => self.buttons = PointerButtons::NONE,
            PointerPhase::Move | PointerPhase::Enter | PointerPhase::Leave => {}
        }
        Some(PointerInput {
            id: PointerId::MOUSE,
            kind: PointerKind::Mouse,
            phase,
            buttons: self.buttons,
            position: at,
            pressure: None,
            tilt: None,
        })
    }
}
