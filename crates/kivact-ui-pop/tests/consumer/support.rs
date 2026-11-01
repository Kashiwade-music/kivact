//! Role: The render options every Pop consumer test opens its window with.

use kivact::test::RenderOptions;
use kivact_ui_pop as ui;

pub(crate) fn options() -> RenderOptions {
    RenderOptions {
        theme: ui::theme(kivact::style::ThemeMode::Light),
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}
