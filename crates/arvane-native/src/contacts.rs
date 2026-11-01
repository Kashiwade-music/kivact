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
use kivact_types::{
    MouseButton, Point, PointerButtons, PointerId, PointerInput, PointerKind, PointerPhase,
};
use winit::event::{Touch, TouchPhase};

/// Map physical native coordinates and cancellation metadata before normalization.
pub(crate) fn map(touch: Touch, scale: f64, notes: &[ContactNote]) -> PointerInput {
    let note = notes.iter().find(|n| n.id == touch.id).copied();
    let canceled = touch.phase == TouchPhase::Cancelled || note.is_some_and(|n| n.canceled);
    let phase = match touch.phase {
        TouchPhase::Started => PointerPhase::Down,
        TouchPhase::Moved => PointerPhase::Move,
        TouchPhase::Ended | TouchPhase::Cancelled if canceled => PointerPhase::Cancel,
        TouchPhase::Ended | TouchPhase::Cancelled => PointerPhase::Up,
    };
    PointerInput {
        id: PointerId(touch.id),
        kind: note.map_or(PointerKind::Touch, |n| n.kind),
        phase,
        position: Point {
            x: (touch.location.x / scale) as f32,
            y: (touch.location.y / scale) as f32,
        },
        // A contact presses the primary button while it lasts.
        buttons: PointerButtons::NONE.with(
            MouseButton::Primary,
            matches!(phase, PointerPhase::Down | PointerPhase::Move),
        ),
        button: matches!(phase, PointerPhase::Down | PointerPhase::Up)
            .then_some(MouseButton::Primary),
        pressure: touch.force.map(|force| force.normalized() as f32),
        tilt: note.and_then(|n| n.tilt),
        clicks: 1,
    }
}
