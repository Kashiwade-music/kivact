//! Open the Soft component catalog with application-owned fonts.
use kivact::App;
use kivact_ui_soft_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Soft")
        .window_size(1440, 900)
        .theme(kivact::style::Theme::default().with_fonts(fonts().iter().copied()))
        .run(catalog)
}
