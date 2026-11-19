//! Role: Present persistent alerts and dismissible feedback banners.

pub(crate) mod look;
mod view;

pub use look::AlertVariant;
pub use view::{Alert, alert};
