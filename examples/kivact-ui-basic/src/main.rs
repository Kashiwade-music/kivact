//! Role: Entry point of the Basic kit catalog app.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_ui_basic_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Basic")
        .window_size(1440, 900)
        .fonts(fonts().iter().copied())
        .run(catalog)
}
