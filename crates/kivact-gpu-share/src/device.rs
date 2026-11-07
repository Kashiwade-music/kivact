//! Role: The device requests shared by the import and export sides of shared textures.
//!
//! The import side's implementation (`import::device`) asks for the Vulkan
//! extensions both sides need; this module is the path both sides name.

pub use crate::import::{Opened, identity, request_device};
