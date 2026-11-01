//! Role: Tests for images and fonts through the public API.
//!
//! Images and fonts through the public API: every `Asset`
//! source, natural sizes and aspect ratios on the Arvane host, font
//! registration at run time, and the located diagnostic of a missing image.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use kivact::Asset;
use kivact::control::ImageFit;
use kivact::diagnostics::ErrorKind;
use kivact::draw::{IntrinsicSize, Size};
use kivact::prelude::*;
use kivact::style::Align;
#[cfg(windows)]
use kivact::test::{RenderOptions, TestFonts, render_with};
use kivact::test::{Screen, render};
use kivact::view::error_boundary;

static SVG_4X2: &[u8] = include_bytes!("../fixtures/shapes.svg");
static PNG_2X2: &[u8] = include_bytes!("../fixtures/rgba.png");

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(path)
}

fn size(screen: &Screen, id: &str) -> (f32, f32) {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    (r.width, r.height)
}

fn asset_errors(screen: &Screen) -> Vec<UiError> {
    screen
        .errors()
        .into_iter()
        .filter(|e| *e.kind() == ErrorKind::Asset)
        .collect()
}

#[test]
fn image_accepts_every_asset_source() {
    let svg = fixture("fixtures/shapes.svg");
    let screen = render(move || {
        let path = signal(svg.clone());
        column((
            image(include_bytes!("../fixtures/rgba.png")).test_id("include_bytes"),
            image(SVG_4X2).test_id("static slice"),
            image(PNG_2X2.to_vec()).test_id("vec"),
            image(Arc::<[u8]>::from(SVG_4X2)).test_id("arc"),
            image(svg.clone()).test_id("pathbuf"),
            image(svg.as_path()).test_id("path"),
            image(svg.display().to_string()).test_id("string"),
            image(Asset::from(PNG_2X2)).test_id("asset"),
            image(move || Asset::from(path.get())).test_id("closure"),
        ))
        .style(style().align_items(Align::Start))
    });
    for (id, expected) in [
        ("include_bytes", (2.0, 2.0)),
        ("static slice", (4.0, 2.0)),
        ("vec", (2.0, 2.0)),
        ("arc", (4.0, 2.0)),
        ("pathbuf", (4.0, 2.0)),
        ("path", (4.0, 2.0)),
        ("string", (4.0, 2.0)),
        ("asset", (2.0, 2.0)),
        ("closure", (4.0, 2.0)),
    ] {
        assert_eq!(size(&screen, id), expected, "{id}");
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_relative_str_path_is_a_file() {
    // `&'static str` reads a file at run time (relative to the current
    // directory, the crate's directory under `cargo test`).
    let screen = render(|| {
        column((image("tests/fixtures/shapes.svg").test_id("svg"),))
            .style(style().align_items(Align::Start))
    });
    assert_eq!(size(&screen, "svg"), (4.0, 2.0));
}

#[test]
fn images_keep_their_aspect_ratio_and_take_style_sizes() {
    let screen = render(|| {
        column((
            image(SVG_4X2).test_id("natural"),
            image(SVG_4X2).test_id("width").style(style().width(40)),
            image(SVG_4X2).test_id("height").style(style().height(10)),
            image(SVG_4X2)
                .test_id("both")
                .fit(ImageFit::Cover)
                .style(style().size(30, 30)),
            image(SVG_4X2)
                .test_id("hint")
                .intrinsic_size(IntrinsicSize {
                    preferred: Some(Size::new(16.0, 16.0)),
                    ..IntrinsicSize::default()
                }),
        ))
        .style(style().align_items(Align::Start))
    });
    assert_eq!(size(&screen, "natural"), (4.0, 2.0));
    assert_eq!(size(&screen, "width"), (40.0, 20.0));
    assert_eq!(size(&screen, "height"), (20.0, 10.0));
    assert_eq!(size(&screen, "both"), (30.0, 30.0));
    assert_eq!(size(&screen, "hint"), (16.0, 16.0));
}

#[test]
fn a_missing_image_is_one_diagnostic_at_its_call_with_the_absolute_path() {
    let line = line!() + 4;
    let screen = render(|| {
        let n = signal(0);
        column((
            image("no/such/dir/missing.png").test_id("missing"),
            button_like(n),
            text(move || format!("{}", n.get())),
        ))
    });
    screen.get_by_test_id("bump").click();
    screen.get_by_test_id("bump").click();
    let errors = asset_errors(&screen);
    assert_eq!(errors.len(), 1, "{errors:?}");
    let e = &errors[0];
    let at = e.location().expect("located at image(..)");
    assert!(at.file().ends_with("assets.rs"), "{at}");
    assert_eq!(at.line(), line);
    let absolute = std::path::absolute("no/such/dir/missing.png").expect("absolute path");
    assert!(e.message().contains(&absolute.display().to_string()), "{e}");
    let cwd = std::env::current_dir().expect("current directory");
    assert!(e.message().contains(&cwd.display().to_string()), "{e}");
    // The node draws nothing and the App goes on.
    assert_eq!(
        screen.get_by_test_id("missing").rect().map(|r| r.height),
        Some(0.0)
    );
    screen.get_by_text("2");
}

/// A clickable box that bumps `n` (using the public primitives); it has a size
/// so that a pointer can hit it.
fn button_like(n: Signal<i32>) -> Node {
    node()
        .test_id("bump")
        .style(style().width(40.0).height(20.0))
        .on_press(move || n.update(|v| *v += 1))
}

#[test]
fn switching_to_a_missing_file_reports_it_once() {
    let good = fixture("fixtures/shapes.svg");
    let screen = render(move || {
        let src = signal(Asset::from(good.clone()));
        column((
            image(src).test_id("img"),
            node()
                .test_id("bump")
                .style(style().width(40.0).height(20.0))
                .on_press(move || {
                    src.set(Asset::from("missing-after-switch.png"));
                }),
        ))
    });
    assert!(asset_errors(&screen).is_empty());
    screen.get_by_test_id("bump").click();
    screen.get_by_test_id("bump").click();
    let errors = asset_errors(&screen);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message().contains("missing-after-switch.png"));
}

/// What an `error_boundary` saw: the errors its `on_error` received.
type Caught = Rc<RefCell<Vec<UiError>>>;

#[test]
fn a_failed_image_under_an_error_boundary_keeps_the_content_shown() {
    // An asset failure draws nothing for that node (an image) and the App
    // goes on: it is a diagnostic of the default sink, not an error of the
    // boundary's subtree, so the boundary neither swaps in its fallback nor
    // calls `on_error`.
    let caught: Caught = Rc::default();
    let seen = caught.clone();
    let image_line = line!() + 6;
    let screen = render(move || {
        let seen = seen.clone();
        error_boundary(|| {
            column((
                text("important content"),
                image("missing.png").name("thumb"),
            ))
        })
        .on_error(move |e| seen.borrow_mut().push(e))
        .fallback(|e, _reset| text(format!("fallback: {e}")))
    });
    screen.run_until_idle();
    screen.get_by_text("important content");
    screen.get_by_role(Role::Image, "thumb");
    assert!(caught.borrow().is_empty(), "{:?}", caught.borrow());
    let errors = asset_errors(&screen);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].location().map(|l| l.line()),
        Some(image_line),
        "{errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.message().contains("missing.png"))
            .count(),
        1,
        "{errors:?}"
    );
}

/// Emoji draw in color from the platform's color emoji font (Windows ships
/// Segoe UI Emoji), not in one color from a symbol font that also has
/// them; text without emoji keeps its font list.
///
/// Windows only: it needs a color emoji font among the system fonts, which
/// every Windows has; Linux distributions may have none (WSL's Ubuntu has
/// none), and macOS cannot open test windows.
#[cfg(windows)]
#[test]
fn emoji_draw_in_color_from_the_system_emoji_font() {
    let screen = render_with(
        RenderOptions {
            fonts: TestFonts::System,
            ..RenderOptions::from_env()
        },
        || {
            column((
                text("😀👍❤️")
                    .test_id("emoji")
                    .style(style().font_size(32.0)),
                text("plain ▾")
                    .test_id("plain")
                    .style(style().font_size(32.0)),
            ))
        },
    );
    let colored = |id: &str| {
        screen
            .get_by_test_id(id)
            .screenshot()
            .pixels
            .iter()
            .filter(|p| {
                let (r, g, b) = ((*p >> 16) & 0xff, (*p >> 8) & 0xff, *p & 0xff);
                r.abs_diff(g) > 40 || g.abs_diff(b) > 40
            })
            .count()
    };
    assert!(
        colored("emoji") > 200,
        "emoji in color: {}",
        colored("emoji")
    );
    assert_eq!(colored("plain"), 0, "plain text and symbols in one color");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
