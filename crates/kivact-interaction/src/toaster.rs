//! Role: Queue toast notifications and own their deadlines.
mod api;
mod column;
mod data;
mod timer;
pub use api::{ToastState, ToasterApi, use_toaster};
pub use data::{Toast, ToastId, ToastOptions};
