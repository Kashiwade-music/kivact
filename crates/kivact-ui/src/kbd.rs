//! Role: Present literal and typed shortcuts with platform command-key glyphs.

pub(crate) mod labels;
pub(crate) mod look;
mod view;

pub use view::{Kbd, KbdCombo, kbd, kbd_combo};
