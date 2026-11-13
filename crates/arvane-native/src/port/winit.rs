//! Role: Winit window and input port.
mod contacts;
pub(crate) mod foreign;
pub(crate) mod key_code;
mod open;
pub(crate) mod runtime;
mod shell_loop;
pub(crate) mod wheel;
mod window;
mod window_events;
mod windows;
pub use runtime::backend;
pub use window::WinitWindow;
