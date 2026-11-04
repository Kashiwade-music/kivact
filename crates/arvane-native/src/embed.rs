//! Role: Native child window creation in the host event loop.
//!
//! Win32 uses the host GUI thread; X11 owns a joined connection thread.
//! Other operating systems report Unsupported without constructing a handler.
use arvane_shell::{ChildHooks, EmbedOptions, MakeHandler, ShellChild, ShellError};
use kivact_types::ParentWindow;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod x11;
#[cfg(all(windows, feature = "testing"))]
pub use windows::TestParent;
#[cfg(all(target_os = "linux", feature = "testing"))]
pub use x11::test_parent::TestParent;
/// Open a child of `parent`, constructing its handler on the native window's
/// UI thread.
pub fn open(
    parent: ParentWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    #[cfg(windows)]
    {
        windows::open(parent, options, hooks, make)
    }
    #[cfg(target_os = "linux")]
    {
        x11::open(parent, options, hooks, make)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (parent, options, hooks, make);
        Err(ShellError::unsupported(
            "embedding a window is not implemented on this platform yet",
        ))
    }
}

#[cfg(any(windows, target_os = "linux"))]
mod geometry;

#[cfg(any(windows, target_os = "linux"))]
mod child;
#[cfg(any(windows, target_os = "linux"))]
mod session;
