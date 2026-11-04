//! Role: Binding-backed scalar operations and semantic control parts.
//!
//! A [`ScalarApi`] has no value of its own: the finite range, the min-anchored
//! step, the key step and assistive technology all write through the
//! caller's binding. A slider attaches [`ScalarApi::thumb`] and
//! [`ScalarApi::track`]; a number field attaches [`ScalarApi::input`] to its text
//! field; a canvas declares the thumb as a region and maps the pointer itself
//! with [`ScalarApi::scrub`]. The slider's thumb and the spin button share this
//! private implementation, except that the spin button leaves the value text
//! to the native editor. [`RangeScalarApi`](crate::RangeScalarApi) builds two ends
//! on the same plumbing. The reading direction a horizontal track mirrors in
//! comes from `kivact::view::ReadingDirection` (`provide_direction`,
//! `use_direction`).
//!
//! A spin button takes Up/Down, PageUp/PageDown (ten steps) and unmodified
//! Home/End; Left/Right and modified Home/End stay the native editor's caret
//! keys. In the engine's native editor, unmodified Home/End skip native
//! editing and go to the key handlers only when the editor, or a
//! non-focusable ancestor, is a `SpinButton` with an explicit SetValue
//! action; a `TextBox` inside a focusable numeric control (Creative's), an
//! editor that only has the `SpinButton` role and an ordinary `TextBox` keep
//! native caret movement. Assistive technology's SetValue on an editor that
//! is itself such a spin button goes to the explicit action handler (the
//! numeric writer) without passing through text input; an ordinary native
//! SetValue is edited by the engine and notified as `TextChanged`, and the
//! core's `run_a11y` hands a host SetValue to the explicit action handler
//! without sending it to text input again. The kits put the part on the real
//! field or on a
//! non-focusable row around it: the standard `number_input` makes the field
//! itself the spin button, and a component that keeps the field a `TextBox`
//! makes the non-focusable row the spin button (with the name and range,
//! without a value text); both focus the field and share these keys.
//!
//! Integer editing projects the binding to `f64` and uses a [`ScalarRange`]
//! with step 1: a finite assistive technology value is rounded to the
//! nearest whole step from the minimum and clamped, a non-finite one is not
//! written, and the field's typed text is parsed as `i32` by the kit. After
//! an external write left the value outside the range, a step first brings
//! the current value inside the range and then moves it.
//!
//! Invariants:
//! - A range change writes the value back only when it falls outside the new
//!   range, through one effect per scalar however many parts are attached.
//! - A key press, an assistive technology action and a track drag are each
//!   one edit of the scalar's [`EditSession`]; Escape, an engine Cancel and
//!   the focus leaving the thumb during a drag write the starting value back.

mod data;
mod edit_state;
mod handle;
mod ops;
mod range;
mod touch;
mod track;

pub use data::ScalarState;
pub(crate) use edit_state::{Edit, Shared};
pub use handle::{ScalarApi, use_scalar};
pub(crate) use ops::ScalarOps;

pub(crate) use range::{follow_range, pulled_inside};
pub use touch::TouchSliderGesture;
pub(crate) use track::Grip;
