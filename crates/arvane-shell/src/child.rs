//! Role: Child window options and the host event loop SPI.

use crate::{ShellError, ShellHandler};

/// The parent, first logical size, and optional host-decided scale.
#[derive(Clone)]
pub struct ChildOptions {
    /// The host's window.
    pub parent: ui_types::ParentWindow,
    /// Initial width in logical pixels.
    pub width: f32,
    /// Initial height in logical pixels.
    pub height: f32,
    /// Physical pixels per logical pixel, or the OS scale.
    pub scale: Option<f32>,
    /// Ask the host for a physical content size.
    pub on_resize_request: Option<std::sync::Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
}

impl std::fmt::Debug for ChildOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildOptions")
            .field("parent", &self.parent)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale", &self.scale)
            .field("on_resize_request", &self.on_resize_request.is_some())
            .finish()
    }
}

/// Builds the window's handler on its UI thread, once.
pub type MakeHandler = Box<dyn FnOnce() -> Box<dyn ShellHandler> + Send>;

/// A child in a host event loop. Drop closes it using the backend's thread rules.
pub trait ShellChild: Send + Sync {
    /// Resize in physical pixels.
    fn set_size(&self, width: u32, height: u32);
    /// Last applied physical size.
    fn size(&self) -> (u32, u32);
    /// Change scale, keeping the physical size.
    fn set_scale(&self, scale: f32);
    /// Last applied scale.
    fn scale(&self) -> f32;
    /// Why the window stopped, or `None` while running.
    fn failure(&self) -> Option<ShellError>;
}
