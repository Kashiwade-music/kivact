//! Creative example application entry point.

use kivact::prelude::*;
use kivact_ui_creative_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    App::new("Kivact · Official UI · Creative")
        .theme(kivact::style::Theme::default().with_fonts(fonts().iter().copied()))
        .run(catalog)
}
