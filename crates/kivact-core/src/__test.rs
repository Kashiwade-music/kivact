//! Role: The launch contract kivact-test drives; not an App API.

pub use crate::launch::{Inspect, LaunchHandle, LaunchPolicy};
pub use crate::test_launch::{
    app_theme_has_fonts, check_window_sizes, host_options, launch, retitle, window_title,
};
