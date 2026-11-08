//! Role: Fixtures shared by the end-to-end tests: mounting, colors, random.
//! Mounting helpers, the CSS color formulas of the color tests and the
//! deterministic random driver of the model tests.

pub(crate) mod color_reference;
mod mount;
pub(crate) mod random;

pub(crate) use mount::{mount, mount_with};
