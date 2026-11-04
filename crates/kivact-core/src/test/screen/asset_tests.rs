//! Role: Asset loading and format detection tests.
//!
//! The App's fonts (`App::font`) on real windows: they register before the
//! first layout, so the first frame already measures text with them, and a
//! rejected one is a diagnostic located at its call. The rest of the image
//! and font behaviour is checked through the public API (`kivact`'s
//! `tests/assets.rs`); only the App's fonts
//! need the crate-internal `Screen::start`.

use std::panic::Location;

use kivact_runtime::{ErrorKind, UiError};
use kivact_style::asset::Asset;

use super::model::Screen;
use crate::prelude::{IntoView, NodeBuilder, column, style, text};
use crate::test::{RenderOptions, TestFonts};
use kivact_style::Align;

static TUFFY: &[u8] = include_bytes!("../../../../arvane/assets/fonts/Tuffy.ttf");

/// A screen whose App registers `fonts` like `App::font`, on a host that
/// starts without any font.
#[track_caller]
fn screen<V: IntoView>(fonts: Vec<Asset>, root: impl FnOnce() -> V + 'static) -> Screen {
    let at = Location::caller();
    let fonts = fonts.into_iter().map(|f| (f, at)).collect();
    Screen::start(
        RenderOptions {
            fonts: TestFonts::Only(Vec::new()),
            ..RenderOptions::default()
        },
        false,
        fonts,
        Box::new(move || root().into_view()),
        at,
    )
}

fn asset_errors(s: &Screen) -> Vec<UiError> {
    s.errors()
        .into_iter()
        .filter(|e| *e.kind() == ErrorKind::Asset)
        .collect()
}

#[test]
fn app_fonts_register_before_the_first_layout() {
    let root = || {
        column((text("WWWWWWWW").test_id("t"),))
            .style(style().font_family("Tuffy").align_items(Align::Start))
    };
    let with = screen(vec![Asset::from(TUFFY)], root);
    assert!(asset_errors(&with).is_empty(), "{:?}", with.errors());
    let without = screen(Vec::new(), root);
    assert_eq!(asset_errors(&without).len(), 1, "the family is missing");
    let width = |s: &Screen| s.get_by_test_id("t").rect().map(|r| r.width);
    assert_ne!(
        width(&with),
        width(&without),
        "the first layout measured with the App's font"
    );
}

#[test]
fn a_rejected_app_font_is_located_at_its_call() {
    let line = line!() + 1;
    let s = screen(vec![Asset::from(&b"wOFF\0\x01\0\0"[..])], || text("x"));
    let errors = asset_errors(&s);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message().contains("TTF or OTF"), "{errors:?}");
    assert_eq!(errors[0].location().map(|l| l.line()), Some(line));
}
