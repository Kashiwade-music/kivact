//! Role: X11 implementation of the child window that fills a foreign window.

use arvane_shell::ShellError;
use kivact_types::NativeWindow;
use std::num::NonZeroU32;
use std::rc::Rc;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::COPY_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ConfigureWindowAux, ConnectionExt, CreateWindowAux, InputFocus, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::xcb_ffi::XCBConnection;

/// The loop's connection for foreign children, opened at the first one.
#[derive(Default)]
pub(crate) struct ForeignConnections {
    connection: Option<Rc<XCBConnection>>,
}

impl ForeignConnections {
    /// An empty loop state; the connection opens at the first child.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn connection(&mut self) -> Result<Rc<XCBConnection>, ShellError> {
        if let Some(connection) = &self.connection {
            return Ok(Rc::clone(connection));
        }
        let (connection, _) =
            XCBConnection::connect(None).map_err(|error| ShellError::new(error.to_string()))?;
        let connection = Rc::new(connection);
        self.connection = Some(Rc::clone(&connection));
        Ok(connection)
    }
}

/// The X11 child window of one foreign window.
pub(crate) struct ForeignOs {
    connection: Rc<XCBConnection>,
    child: u32,
}

impl ForeignOs {
    /// Create the child of `window`, `width` x `height` physical pixels.
    pub(crate) fn create(
        window: &winit::window::Window,
        width: u32,
        height: u32,
        connections: &mut ForeignConnections,
    ) -> Result<Self, ShellError> {
        let parent = parent_window(window)?;
        let connection = connections.connection()?;
        let child = connection
            .generate_id()
            .map_err(|error| ShellError::new(error.to_string()))?;
        let (width, height) = (
            u16::try_from(width.max(1)).unwrap_or(u16::MAX),
            u16::try_from(height.max(1)).unwrap_or(u16::MAX),
        );
        connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                child,
                parent,
                0,
                0,
                width,
                height,
                0,
                WindowClass::INPUT_OUTPUT,
                COPY_FROM_PARENT,
                &CreateWindowAux::new().background_pixel(0).border_pixel(0),
            )
            .map_err(|error| ShellError::new(error.to_string()))?
            .check()
            .map_err(|error| ShellError::new(error.to_string()))?;
        connection
            .map_window(child)
            .map_err(|error| ShellError::new(error.to_string()))?
            .check()
            .map_err(|error| ShellError::new(error.to_string()))?;
        connection
            .flush()
            .map_err(|error| ShellError::new(error.to_string()))?;
        Ok(Self { connection, child })
    }

    /// Resize the child to `width` x `height` physical pixels.
    pub(crate) fn fit(&mut self, width: u32, height: u32) -> Result<(), ShellError> {
        let (width, height) = (
            u16::try_from(width.max(1)).unwrap_or(u16::MAX),
            u16::try_from(height.max(1)).unwrap_or(u16::MAX),
        );
        self.connection
            .configure_window(
                self.child,
                &ConfigureWindowAux::new()
                    .width(u32::from(width))
                    .height(u32::from(height)),
            )
            .map_err(|error| ShellError::new(error.to_string()))?;
        self.connection
            .flush()
            .map_err(|error| ShellError::new(error.to_string()))
    }

    /// Give the keyboard focus to the foreign UI's own child, if it has one.
    pub(crate) fn focus_into(&self) {
        let Ok(cookie) = self.connection.query_tree(self.child) else {
            return;
        };
        let Ok(tree) = cookie.reply() else {
            return;
        };
        let Some(first) = tree.children.first() else {
            return;
        };
        let _ = self
            .connection
            .set_input_focus(InputFocus::PARENT, *first, x11rb::CURRENT_TIME);
        let _ = self.connection.flush();
    }

    /// The raw child handle given to the foreign UI as its parent.
    pub(crate) fn native(&self) -> NativeWindow {
        let child =
            NonZeroU32::new(self.child).expect("invariant: a generated X11 window id is nonzero");
        NativeWindow::X11(child)
    }
}

impl Drop for ForeignOs {
    fn drop(&mut self) {
        if let Ok(cookie) = self.connection.destroy_window(self.child) {
            let _ = cookie.check();
        }
        let _ = self.connection.flush();
    }
}

fn parent_window(window: &winit::window::Window) -> Result<u32, ShellError> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window
        .window_handle()
        .map_err(|error| ShellError::new(error.to_string()))?;
    match handle.as_raw() {
        RawWindowHandle::Xlib(handle) => u32::try_from(handle.window)
            .ok()
            .filter(|id| *id != 0)
            .ok_or_else(|| ShellError::unsupported("the X11 window id does not fit 32 bits")),
        RawWindowHandle::Xcb(handle) => Ok(handle.window.get()),
        _ => Err(ShellError::unsupported(
            "the window has no X11 handle for a foreign child",
        )),
    }
}
