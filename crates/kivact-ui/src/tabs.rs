//! Role: Render a shared themed tab list with optional selected panel lifetime.
//!
//! As a plain view the builder draws only the tab list. With
//! `.content(factory)` it also draws the panel of the selected valid key in a
//! new owner each time the selection changes: beside the list when vertical,
//! under it otherwise, with the recipes laying the tabs out by orientation
//! (`TabListState` and `TabState`). With `keep_mounted()` each option's
//! panel is built once and the tab panel part hides it while it is not
//! selected. A selected key that is not among the options stays in the
//! binding, but the factory is not called for it. A kit's domain tab bar
//! (close, reorder, icons, counters) moves and selects with the same
//! interaction parts and draws its ordinary parts with `UiRecipes::tabs`.
pub(crate) mod look;
mod view;

pub use view::{Tabs, tabs};
