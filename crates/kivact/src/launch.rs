//! Role: What an App wishes of its window's GPU device at launch.
//!
//! Give a [`GpuPreference`](crate::launch::GpuPreference) to
//! `App::gpu_preference`. A build without the
//! feature `gpu` draws on the CPU only and ignores it.

#[doc(inline)]
pub use kivact_core::launch::{AdapterMatch, GpuBackend, GpuPower, GpuPreference, GpuStart};
