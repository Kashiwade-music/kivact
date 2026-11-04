//! Role: Tests for images and fonts through the public API.
//!
//! Images and fonts through the public API: every `Asset`
//! source, natural sizes and aspect ratios on the Arvane host, font
//! registration at run time, and the located diagnostic of a missing image.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use crate::support::mount;
use kivact::ErrorKind;
use kivact::draw::IntrinsicSize;
use kivact::geometry::Size;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Screen, render};
use kivact::view::Asset;
use kivact::view::ImageFit;
use kivact::view::error_boundary;

static SVG_4X2: &[u8] = include_bytes!("../fixtures/shapes.svg");
static PNG_2X2: &[u8] = include_bytes!("../fixtures/rgba.png");

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(path)
}

fn size(screen: &Screen, id: &str) -> (f64, f64) {
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
fn an_image_error_handler_can_show_a_fallback_and_observes_retried_sources() {
    let missing = Asset::from("missing-retry.png");
    let first = missing.clone();
    let (screen, source) = mount(move || {
        let source = signal(first);
        let failures = signal(0);
        let view = column((
            image(source).on_image_error(move |_| failures.update(|count| *count += 1)),
            text(move || format!("Fallback count {}", failures.get())),
        ));
        (source, view)
    });
    screen.get_by_text("Fallback count 1");
    source.set(Asset::from(PNG_2X2));
    screen.run_until_idle();
    source.set(missing);
    screen.run_until_idle();
    screen.get_by_text("Fallback count 2");
    assert!(asset_errors(&screen).is_empty());
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
