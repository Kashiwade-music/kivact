//! Role: The functions `kivact_core::__test` exposes over a private App.

use crate::app::App;
use crate::launch::{Launch, LaunchHandle, LaunchPolicy};
use crate::view::View;

/// Launch `app` under `policy` with `root`, returning the engine client and
/// the inspection handle.
#[track_caller]
pub fn launch(
    app: App,
    policy: LaunchPolicy,
    root: impl FnOnce() -> View + 'static,
) -> (kivact_engine::api::Client, LaunchHandle) {
    Launch::new(app.into_launch(), policy, root).into_engine_client()
}
pub fn host_options(app: &App) -> kivact_engine::api::HostOptions {
    app.host_options()
}
pub fn window_title(app: &App) -> &str {
    app.window_title()
}
#[track_caller]
pub fn check_window_sizes(app: &App) -> crate::Result<()> {
    app.check_window_sizes()
}
pub fn retitle(app: App, title: impl Into<String>) -> App {
    app.retitle(title)
}
/// Whether the App's fixed theme carries fonts (the test host then starts without any).
pub fn app_theme_has_fonts(app: &App) -> bool {
    app.theme
        .fixed()
        .is_some_and(|theme| !theme.fonts().is_empty())
}
