//! Role: Foreign child windows on platforms without a native implementation.

use arvane_shell::ShellError;
use kivact_types::NativeWindow;

/// No foreign children exist here.
#[derive(Default)]
pub(crate) struct ForeignConnections;

impl ForeignConnections {
    /// No connection state here.
    pub(crate) fn new() -> Self {
        Self
    }
}

/// A foreign child cannot be created here.
pub(crate) struct ForeignOs;

impl ForeignOs {
    pub(crate) fn create(
        _window: &winit::window::Window,
        _width: u32,
        _height: u32,
        _connections: &mut ForeignConnections,
    ) -> Result<Self, ShellError> {
        Err(ShellError::unsupported(
            "foreign windows are only implemented on Windows and X11",
        ))
    }

    pub(crate) fn fit(&mut self, _width: u32, _height: u32) -> Result<(), ShellError> {
        Ok(())
    }

    pub(crate) fn focus_into(&self) {}

    pub(crate) fn native(&self) -> NativeWindow {
        unreachable!("invariant: no foreign child exists on this platform")
    }
}
