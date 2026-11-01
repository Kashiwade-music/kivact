//! Role: The render options every game consumer test opens its window with.

use kivact::test::RenderOptions;
use kivact_ui_game as g;

pub(crate) fn options(reduced_motion: bool) -> RenderOptions {
    RenderOptions {
        theme: g::theme(kivact::style::ThemeMode::Light),
        reduced_motion,
        ..RenderOptions::from_env()
    }
}
