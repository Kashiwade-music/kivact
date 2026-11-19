//! Role: Wire a visible label, hint and error to the control of a form field.
//!
//! A field is a column of the label, the control it is given, an optional
//! hint and an optional error message. The wiring is the field's: the label
//! is the control's accessible name (`labelled_by`, and the name itself),
//! the hint and the error describe it (`described_by`), and a present error
//! makes the control invalid for assistive technology and for its own
//! recipe. The error message and its relation exist only while the error is
//! `Some`. Pressing the label activates the control through the shared activation authority.
pub(crate) mod look;
mod view;

pub use view::{Field, field};
