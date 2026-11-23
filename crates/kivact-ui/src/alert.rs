//! Role: Present persistent alerts and dismissible feedback banners.
pub(crate) mod look;
pub use look::AlertVariant;
mod view;
pub use view::{Alert, alert};
