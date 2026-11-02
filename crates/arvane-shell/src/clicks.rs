//! Role: Click counting.
//! Presses of the same
//! button within the system's double-click time and distance of the last
//! one count up here, and a release carries its press's count. Touch and
//! pen contacts count their taps by the same rule (a contact's `Down` is
//! its press), apart from the mouse's presses.
//!
//! A press counts up from the last when it is the same button, sooner than
//! the double-click time after it and within the double-click distance of
//! it on each axis; otherwise it is 1. The settings are read at each press,
//! in logical pixels; where they cannot be read they are Windows' defaults,
//! 500 ms and 4 px ([`DEFAULT_DOUBLE_CLICK`], which test windows count
//! with too). Each held button keeps the count it was pressed with for its
//! release, so the buttons of a chord release with their own counts. A
//! contact's later events carry the count of its `Down`.

use crate::events::ShellEvent;
use kivact_types::{MouseButton, Point, PointerId, PointerKind, PointerPhase};

/// Windows' default double-click time (ns, `GetDoubleClickTime`) and
/// distance (pixels per axis, `SM_CXDOUBLECLK` / `SM_CYDOUBLECLK`): what a
/// window that cannot read the system's settings counts with.
pub const DEFAULT_DOUBLE_CLICK: (u64, (f32, f32)) = (500_000_000, (4.0, 4.0));

/// The last mouse press.
#[derive(Clone, Copy, Debug)]
struct Press {
    button: MouseButton,
    at: Point,
    time_ns: u64,
    count: u8,
}

/// The last touch or pen contact's `Down`.
#[derive(Clone, Copy, Debug)]
struct Tap {
    id: PointerId,
    at: Point,
    time_ns: u64,
    count: u8,
}

/// Counts the presses of the pointer events a window delivers.
#[derive(Clone, Debug, Default)]
pub struct ClickCounter {
    /// The last mouse press.
    last: Option<Press>,
    /// The count each held button was pressed with (`None`: not held), by
    /// [`slot`].
    pressed: [Option<u8>; 5],
    /// The last contact's `Down`.
    tap: Option<Tap>,
}

/// The index of `button` in [`ClickCounter::pressed`].
fn slot(button: MouseButton) -> usize {
    match button {
        MouseButton::Primary => 0,
        MouseButton::Secondary => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
    }
}

/// Whether a press at `at` and `now` follows the one at `then_at` and
/// `then` within `double_click` (ns and logical pixels per axis).
fn follows(
    at: Point,
    now: u64,
    then_at: Point,
    then: u64,
    double_click: (u64, (f32, f32)),
) -> bool {
    let (time, (dx, dy)) = double_click;
    now.saturating_sub(then) < time
        && (at.x - then_at.x).abs() <= dx
        && (at.y - then_at.y).abs() <= dy
}

impl ClickCounter {
    /// Fill in the click count of `event`'s press or release, or of its
    /// contact's event. A press counts up when it is the same button as
    /// the last press, less than the double-click time after it and within
    /// the double-click distance of it per axis (`double_click`: ns and
    /// logical pixels, read at the press), else it is a single click; a
    /// release carries the count its button was pressed with. A contact's
    /// `Down` counts up the same way from the last contact's `Down`.
    /// `now_ns` is read at a press or a `Down`.
    pub fn count(
        &mut self,
        event: &mut ShellEvent,
        now_ns: impl FnOnce() -> u64,
        double_click: impl FnOnce() -> (u64, (f32, f32)),
    ) {
        let ShellEvent::Pointer(input) = event else {
            return;
        };
        if input.kind != PointerKind::Mouse {
            if input.phase == PointerPhase::Down {
                let (now, settings) = (now_ns(), double_click());
                let count = self
                    .tap
                    .filter(|tap| follows(input.position, now, tap.at, tap.time_ns, settings))
                    .map_or(1, |tap| tap.count.saturating_add(1));
                self.tap = Some(Tap {
                    id: input.id,
                    at: input.position,
                    time_ns: now,
                    count,
                });
                input.clicks = count;
            } else {
                input.clicks = self
                    .tap
                    .filter(|tap| tap.id == input.id)
                    .map_or(1, |tap| tap.count);
            }
            return;
        }
        match (input.phase, input.button) {
            (PointerPhase::Down, Some(button)) => {
                let (now, settings) = (now_ns(), double_click());
                let count = self
                    .last
                    .filter(|last| {
                        last.button == button
                            && follows(input.position, now, last.at, last.time_ns, settings)
                    })
                    .map_or(1, |last| last.count.saturating_add(1));
                self.last = Some(Press {
                    button,
                    at: input.position,
                    time_ns: now,
                    count,
                });
                self.pressed[slot(button)] = Some(count);
                input.clicks = count;
            }
            (PointerPhase::Up, Some(button)) => {
                input.clicks = self.pressed[slot(button)].take().unwrap_or(1);
            }
            (PointerPhase::Cancel, _) => {
                self.pressed = [None; 5];
                input.clicks = 1;
            }
            _ => input.clicks = 1,
        }
    }
}
