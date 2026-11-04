//! Role: Re-export unmounted descriptions and reusable factory contracts.

mod description;
mod factory;
mod sealed_view_fn;

pub use description::{IntoView, View, view_is_empty};
pub(crate) use description::{Mcx, mount, view_fn};
pub use factory::{IntoViewFn, ViewFn};
