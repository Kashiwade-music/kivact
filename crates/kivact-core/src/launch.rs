//! Role: Shared application launch and read-only inspection boundary.
mod close;
mod inspect;
mod runtime;
mod settings;
pub use inspect::{Inspect, LaunchHandle};
pub(crate) use runtime::run_native;
pub use runtime::{ClientMode, Launch, TestTiming};
pub use settings::AppOptions;

/// An adapter's identity, as the graphics APIs report it.
#[doc(inline)]
pub use kivact_types::AdapterMatch;
/// A graphics API the window's GPU device can use.
#[doc(inline)]
pub use kivact_types::GpuBackend;
/// Which kind of adapter an App prefers ([`App::gpu_preference`](crate::App::gpu_preference)).
#[doc(inline)]
pub use kivact_types::GpuPower;
/// What an App wishes of the window's GPU device ([`App::gpu_preference`](crate::App::gpu_preference)).
#[doc(inline)]
pub use kivact_types::GpuPreference;
/// When the window's GPU device is started (`GpuPreference::start`).
#[doc(inline)]
pub use kivact_types::gpu::GpuStart;
