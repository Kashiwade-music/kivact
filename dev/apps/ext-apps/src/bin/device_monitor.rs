//! Role: `cargo run -p kivact-ext-apps --bin device-monitor`
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_ext_apps::device_monitor::{app, fleet};
use kivact_external_component::skin::Skin;

fn main() -> kivact::Result<()> {
    App::new("Kivact Device Monitor").run(|| app(fleet(5, 1_000), Skin::Business, Some(1_000)))
}
