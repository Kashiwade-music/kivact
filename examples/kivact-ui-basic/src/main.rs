//! Entry point of the Basic kit catalog app.

use kivact::prelude::*;
use kivact_ui_basic_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Basic")
        .window_size(1440, 900)
        .theme(kivact::style::Theme::default().with_fonts(fonts().iter().copied()))
        .run(catalog)
}
