//! Role: Asset loading and format detection tests.
//!
//! The theme's fonts (`Theme::with_fonts`) on real windows: they register
//! before the first layout, so the first frame already measures text with
//! them, a rejected one is a diagnostic located at the theme call, and a
//! theme switched later registers its own. The rest of the image and font
//! behaviour is checked through the public API (`kivact`'s
//! `tests/assets.rs`); only the App's theme fonts need the crate-internal
//! `Screen::start`.

use kivact::prelude::{IntoView, column, signal, style, text};
use kivact::style::provide_theme;
use kivact::style::{Align, Theme};
use kivact::test::{RenderOptions, Screen, render_with, render_with_handle};
use kivact::view::Asset;
use kivact::{ErrorKind, UiError};

static TUFFY: &[u8] = include_bytes!("../../../../../crates/arvane/assets/fonts/Tuffy.ttf");
static NOT_A_FONT: &[u8] = b"wOFF\0\x01\0\0";
/// Tuffy's file in the repository (the same bytes as `TUFFY`).
const TUFFY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../crates/arvane/assets/fonts/Tuffy.ttf"
);

/// A screen whose App theme carries `fonts`.
#[track_caller]
fn screen<V: IntoView>(fonts: Vec<Asset>, root: impl FnOnce() -> V + 'static) -> Screen {
    render_with(
        RenderOptions::default().theme(Theme::default().with_fonts(fonts)),
        root,
    )
}

fn asset_errors(s: &Screen) -> Vec<UiError> {
    s.errors()
        .into_iter()
        .filter(|e| *e.kind() == ErrorKind::Asset)
        .collect()
}

fn tuffy_text() -> impl IntoView {
    column((text("WWWWWWWW").test_id("t"),))
        .style(style().font_family("Tuffy").align_items(Align::Start))
}

#[test]
fn theme_fonts_register_before_the_first_layout() {
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
        "the first layout measured with the theme's font"
    );
}

#[test]
fn a_rejected_theme_font_is_located_at_the_theme_call() {
    let line = line!() + 1;
    let s = screen(vec![Asset::from(NOT_A_FONT)], || text("x"));
    let errors = asset_errors(&s);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message().contains("TTF or OTF"), "{errors:?}");
    assert_eq!(errors[0].location().map(|l| l.line()), Some(line));
}

#[test]
fn a_switched_theme_registers_its_fonts() {
    let (s, theme) = render_with_handle(RenderOptions::default(), || {
        let theme = signal(Theme::default());
        (theme, provide_theme(theme, tuffy_text))
    });
    let width = |s: &Screen| s.get_by_test_id("t").rect().map(|r| r.width);
    let before = width(&s);
    theme.set(Theme::default().with_fonts([TUFFY]));
    s.run_until_idle();
    assert!(asset_errors(&s).is_empty(), "{:?}", s.errors());
    assert_ne!(
        width(&s),
        before,
        "the switched theme's font measures the text"
    );
}

#[test]
fn theme_fonts_from_owned_bytes_and_a_path_register() {
    let root = || {
        column((text("WWWWWWWW").test_id("t"),))
            .style(style().font_family("Tuffy").align_items(Align::Start))
    };
    let owned = screen(
        vec![Asset::from(
            std::fs::read(TUFFY_PATH).expect("the font file is readable"),
        )],
        root,
    );
    let path = screen(
        vec![Asset::from(std::path::PathBuf::from(TUFFY_PATH))],
        root,
    );
    let plain = screen(Vec::new(), root);
    assert!(asset_errors(&owned).is_empty(), "{:?}", owned.errors());
    assert!(asset_errors(&path).is_empty(), "{:?}", path.errors());
    let width = |s: &Screen| s.get_by_test_id("t").rect().map(|r| r.width);
    assert_ne!(width(&owned), width(&plain), "owned bytes measure the text");
    assert_ne!(width(&path), width(&plain), "a path measures the text");
}
