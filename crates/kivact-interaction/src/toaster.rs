//! Role: Queue toast notifications and own their deadlines.
mod api;
mod column;
mod data;
mod timer;
pub use api::{ToastState, ToasterApi, toaster};
pub use data::{Toast, ToastId, ToastOptions};
