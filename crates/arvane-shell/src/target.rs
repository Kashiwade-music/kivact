//! Role: Defines retained native targets for UI-thread GPU surface creation.

/// A window a GPU surface can present to (window and display handles).
pub trait GpuWindow:
    raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync
{
}

impl<T> GpuWindow for T where
    T: raw_window_handle::HasWindowHandle
        + raw_window_handle::HasDisplayHandle
        + Send
        + Sync
        + ?Sized
{
}

/// A window as a GPU surface's target. It keeps the native window alive on
/// any thread that holds it. Access its native handles only inside the owning
/// UI thread's `ShellWindow::with_gpu_target` callback; retention does not
/// transfer platform handle-access permission to a worker.
pub type GpuTarget = std::sync::Arc<dyn GpuWindow>;
