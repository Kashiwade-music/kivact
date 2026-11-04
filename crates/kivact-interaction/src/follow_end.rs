//! Role: Expose viewport following commands backed by an owner-scoped machine.
mod api;
mod machine;
pub use api::{FollowEndApi, use_follow_end};
