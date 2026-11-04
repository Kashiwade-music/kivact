//! Role: Queue toast notifications and own their deadlines.
mod api;
mod column;
mod timer;
pub use api::{Toast, ToastId, ToastOptions, ToastState, Toaster, toaster};
