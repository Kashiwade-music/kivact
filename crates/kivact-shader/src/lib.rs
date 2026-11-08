//! Role: Validate user WGSL and assemble Kivact shader modules.
//!
//! A user writes a paint shader (`fn paint`) or a filter shader (`fn effect`)
//! in WGSL. [`assemble`] checks that source with naga, places it between the
//! fixed Kivact prelude and the generated entry points, and returns the
//! complete module with the byte layout of its `Params` uniform block.

mod assemble;
mod entry;
mod error;
mod hash;
mod layout;
mod prelude;
mod source;

pub use assemble::assemble;
pub use error::ShaderError;
