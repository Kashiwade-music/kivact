//! Role: Native child window creation in the host event loop.
//!
//! Win32 uses the host GUI thread; X11 owns a joined connection thread.
//! Other operating systems report Unsupported without constructing a handler.
#[cfg(all(windows, feature = "embed"))]
use crate::port::win32 as windows;
#[cfg(all(target_os = "linux", feature = "embed"))]
use crate::port::x11;
use arvane_shell::{ChildHooks, EmbedOptions, MakeHandler, ShellChild, ShellError};
use kivact_types::ParentWindow;
#[cfg(all(windows, feature = "test"))]
pub use windows::TestParent;
#[cfg(all(target_os = "linux", feature = "test"))]
pub use x11::test_parent::TestParent;
/// Open a child of `parent`, constructing its handler on the native window's
/// UI thread.
pub fn open(
    parent: ParentWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    #[cfg(all(windows, feature = "embed"))]
    {
        windows::open(parent, options, hooks, make)
    }
    #[cfg(all(target_os = "linux", feature = "embed"))]
    {
        x11::open(parent, options, hooks, make)
    }
    #[cfg(not(all(feature = "embed", any(windows, target_os = "linux"))))]
    {
        let _ = (parent, options, hooks, make);
        Err(ShellError::unsupported(
            "embedding a window is not enabled or supported on this platform",
        ))
    }
}
