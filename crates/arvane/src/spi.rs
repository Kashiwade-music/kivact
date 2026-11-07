//! Role: Backend contracts for platform integrations.
pub use arvane_shell::{
    Backend, ChildHooks, CpuFormat, MakeHandler, ShellChild, ShellError, ShellEvent, ShellHandler,
    ShellWindow, Step, StepOutcome, SteppedBackend, SteppedLoop,
};

/// Make the window's GPU device lost, as a device failure does: the
/// window's exports are reported lost, and the device restarts once per session
/// (the test driver's control, §10.6).
pub fn lose_gpu_device(window: &mut crate::Window<'_>) {
    window.lose_gpu_device();
}
