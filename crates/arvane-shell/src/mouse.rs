//! Role: Mouse event production.
//! A backend reports the mouse as separate messages (a move with a
//! position, a button without one, a leave without either); [`Mouse`]
//! keeps the last position and the held buttons, so every event it makes
//! is a complete [`PointerSample`]. Hit testing and capture stay with the
//! host.

use kivact_types::PointerSample;
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
    inside: bool,
}

impl Mouse {
    /// The last known logical position, or `None` before any position arrived.
    pub fn position(&self) -> Option<Point> {
        self.position
    }

    /// Last reported position while the mouse is inside the window.
    pub fn window_position(&self) -> Option<Point> {
        self.inside.then_some(self.position).flatten()
    }

    /// The `Cancel` that ends the gesture when the window loses focus with
    /// buttons held, or `None` when none is held (there is nothing to end).
    pub fn cancel(&mut self) -> Option<PointerSample> {
        if self.buttons.is_empty() {
            return None;
        }
        self.event(PointerPhase::Cancel, None)
    }

    /// Note a complete mouse event made elsewhere (one injected into a
    /// test window), so the next event made here continues from it.
    pub fn observe(&mut self, input: PointerSample) {
        if input.kind == PointerKind::Mouse {
            self.position = Some(input.position);
            self.buttons = input.buttons;
            self.inside = input.phase != PointerPhase::Leave;
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
    ) -> Option<PointerSample> {
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
    pub fn event(&mut self, phase: PointerPhase, position: Option<Point>) -> Option<PointerSample> {
        if let Some(position) = position {
            self.position = Some(position);
            self.inside = true;
        }
        if phase == PointerPhase::Leave {
            self.inside = false;
        }
        let at = self.position?;
        match phase {
            PointerPhase::Down(press) => self.buttons = self.buttons.with(press.button, true),
            PointerPhase::Up(press) => self.buttons = self.buttons.with(press.button, false),
            PointerPhase::Cancel => self.buttons = PointerButtons::NONE,
            PointerPhase::Move | PointerPhase::Enter | PointerPhase::Leave => {}
        }
        Some(PointerSample {
            id: PointerId::MOUSE,
            kind: PointerKind::Mouse,
            phase,
            buttons: self.buttons,
            position: at,
            pressure: None,
            tilt: None,
            button: None,
            press: None,
            time_ns: 0,
            modifiers: kivact_types::Modifiers::default(),
            is_primary: true,
        })
    }
}
