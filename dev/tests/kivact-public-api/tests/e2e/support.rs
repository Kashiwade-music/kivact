//! Role: Fixtures shared by the end-to-end tests: mounting and the random driver.
//! Mounting helpers and the deterministic random driver of the model tests.

mod mount;
pub(crate) mod random;

pub(crate) use mount::{mount, mount_with};
