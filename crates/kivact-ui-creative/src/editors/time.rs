//! Role: Shared time-item geometry, drawing and gestures for clips and keys.

pub(crate) mod a11y;
mod builder;
pub(crate) mod edit;
pub(crate) mod gesture;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;
mod view;
pub(crate) mod view_state;

pub(crate) use builder::time_editor_builder;
pub(crate) use view::TimeEditor;
