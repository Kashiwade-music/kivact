//! Role: Render options, the clean-screen check and tolerance shared by tests.

use kivact::test::RenderOptions;
use kivact_ui_creative as ui;

pub(crate) fn options() -> RenderOptions {
    RenderOptions::with_theme(ui::theme(kivact::style::ThemeMode::Dark)).reduced_motion()
}

pub(crate) fn near(a: impl Into<f64>, b: impl Into<f64>, tolerance: f64) -> bool {
    (a.into() - b.into()).abs() < tolerance
}
