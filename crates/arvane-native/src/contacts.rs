//! Role: Maps winit contacts and native metadata to portable pointer input.
//!
//! winit 0.30 reports touch and pen as Touch events, without kind or tilt,
//! and may report cancelled contacts as lifted. Windows subclass metadata
//! supplies kind, tilt and cancellation before the shell normalizes contacts.
//! Elsewhere a pen is reported as touch. Pressure comes from winit's force.
//! Pen hover, barrel buttons and erasers are not delivered. Windows pointer
//! input was checked only with synthetic injection, not physical devices;
//! cancellation flags cannot be injected and remain unchecked on devices.

use crate::native_input::state::ContactNote;
use kivact_types::PointerSample;
use kivact_types::{
    ButtonPress, MouseButton, Point, PointerButtons, PointerId, PointerKind, PointerPhase,
};
use winit::event::{Touch, TouchPhase};

/// Map physical native coordinates and cancellation metadata before normalization.
pub(crate) fn map(touch: Touch, scale: f64, notes: &[ContactNote]) -> PointerSample {
    let note = notes.iter().find(|n| n.id == touch.id).copied();
    let canceled = touch.phase == TouchPhase::Cancelled || note.is_some_and(|n| n.canceled);
    // A contact presses the primary button while it lasts.
    let press = ButtonPress {
        button: MouseButton::Primary,
        clicks: 1,
    };
    let phase = match touch.phase {
        TouchPhase::Started => PointerPhase::Down(press),
        TouchPhase::Moved => PointerPhase::Move,
        TouchPhase::Ended | TouchPhase::Cancelled if canceled => PointerPhase::Cancel,
        TouchPhase::Ended | TouchPhase::Cancelled => PointerPhase::Up(press),
    };
    PointerSample {
        id: PointerId(touch.id),
        kind: note.map_or(PointerKind::Touch, |n| n.kind),
        phase,
        position: Point {
            x: (touch.location.x / scale) as f64,
            y: (touch.location.y / scale) as f64,
        },
        buttons: PointerButtons::NONE.with(
            MouseButton::Primary,
            matches!(phase, PointerPhase::Down(_) | PointerPhase::Move),
        ),
        pressure: touch.force.map(|force| force.normalized() as f64),
        tilt: note.and_then(|n| n.tilt),
        button: None,
        press: None,
        time_ns: 0,
        modifiers: kivact_types::Modifiers::default(),
        is_primary: true,
    }
}
