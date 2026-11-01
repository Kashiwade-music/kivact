//! Role: `cargo run -p kivact-ext-apps --bin log-viewer`
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_ext_apps::log_viewer::{LogViewerOptions, app};

fn main() -> kivact::Result<()> {
    App::new("Kivact Log Viewer").run(|| app(LogViewerOptions::default()))
}
