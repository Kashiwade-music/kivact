//! Role: E2E checks of `Offscreen::render_svg` without windows.
//!
//! The export writes a drawing as SVG 1.1 markup, from the same recording,
//! theme and fonts as `Offscreen::render`. These tests check the elements,
//! attributes and nesting it names, that the document is well formed, that
//! the text sits on the measured baseline, that one `Offscreen` serves the
//! pixels and the document, and that what SVG cannot express is an error.
//! They compare no pixels of the document (Q-01).

use kivact::ErrorKind;
use kivact::draw::{
    BlendMode, Draw, ImageOptions, Layer, Offscreen, Pixels, TextFormat, Transform, linear_gradient,
};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::theme::Typography;
use kivact::style::{Color, Theme, rgb};
use std::cell::Cell;

static TUFFY: &[u8] = include_bytes!("../../../../../crates/arvane/assets/fonts/Tuffy.ttf");

/// Whether every start tag of `svg` has its end tag, in order.
fn well_formed(svg: &str) -> bool {
    let mut open: Vec<&str> = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find('<') {
        let Some(end) = rest[start..].find('>') else {
            return false;
        };
        let tag = &rest[start + 1..start + end];
        rest = &rest[start + end + 1..];
        if tag.starts_with('?') {
            continue;
        }
        if let Some(name) = tag.strip_prefix('/') {
            if open.pop() != Some(name.trim()) {
                return false;
            }
            continue;
        }
        if !tag.ends_with('/') {
            let name = tag.split(|c: char| c.is_whitespace() || c == '/').next();
            open.push(name.unwrap_or_default());
        }
    }
    open.is_empty()
}

#[test]
fn a_drawing_is_written_as_well_formed_svg_with_its_elements() {
    let mut renderer = Offscreen::new();
    let pixels = Pixels::new(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 128]);
    let out = renderer
        .render_svg(200.0, 100.0, |draw, _size| {
            draw.fill_rect(Rect::new(0.0, 0.0, 200.0, 100.0), rgb(0xffffff));
            draw.rounded_rect(Rect::new(10.0, 10.0, 50.0, 20.0), 4.0, rgb(0x00ff00));
            draw.stroke_rect(Rect::new(70.0, 10.0, 50.0, 20.0), 2.0, rgb(0x0000ff));
            draw.line(
                Point::new(10.0, 40.0),
                Point::new(120.0, 40.0),
                1.5,
                rgb(0x000000),
            );
            draw.layer(
                Layer::new().clip_rect(Rect::new(0.0, 50.0, 100.0, 50.0)),
                |draw| {
                    draw.layer(
                        Layer::new().transform(Transform::default().translate(5.0, 5.0)),
                        |draw| {
                            let style = TextFormat {
                                size: 14.0,
                                color: Color::BLACK,
                                ..Default::default()
                            };
                            draw.text("Hi & <bye>", Point::new(10.0, 60.0), style);
                        },
                    );
                },
            );
            draw.pixels(
                &pixels,
                Rect::new(150.0, 50.0, 40.0, 20.0),
                ImageOptions::default(),
            );
        })
        .expect("a drawing SVG can express");

    assert!(out.starts_with("<?xml version=\"1.0\""));
    assert!(out.contains(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"200\" height=\"100\" viewBox=\"0 0 200 100\">"
    ));
    assert!(out.contains(
        "<rect x=\"10\" y=\"10\" width=\"50\" height=\"20\" rx=\"4\" ry=\"4\" fill=\"#00ff00\"/>"
    ));
    assert!(out.contains("fill=\"none\" stroke=\"#0000ff\" stroke-width=\"2\""));
    assert!(out.contains(
        "<line x1=\"10\" y1=\"40\" x2=\"120\" y2=\"40\" stroke=\"#000000\" stroke-width=\"1.5\" stroke-linecap=\"butt\"/>"
    ));
    assert!(out.contains(
        "<clipPath id=\"clip1\"><rect x=\"0\" y=\"50\" width=\"100\" height=\"50\"/></clipPath>"
    ));
    assert!(out.contains("<g clip-path=\"url(#clip1)\">"));
    assert!(out.contains("<g transform=\"matrix(1 0 0 1 5 5)\">"));
    assert!(out.contains("Hi &amp; &lt;bye&gt;</text>"));
    assert!(out.contains("font-size=\"14\""));
    assert!(out.contains("<image x=\"150\" y=\"50\" width=\"40\" height=\"20\""));
    assert!(out.contains("href=\"data:image/png;base64,"));
    assert!(out.ends_with("</svg>\n"));
    assert!(well_formed(&out), "{out}");
}

#[test]
fn a_linear_gradient_fill_defines_its_stops() {
    let mut renderer = Offscreen::new();
    let out = renderer
        .render_svg(100.0, 50.0, |draw, _size| {
            draw.fill_rect(
                Rect::new(0.0, 0.0, 100.0, 50.0),
                linear_gradient(90.0).colors([rgb(0xff0000), rgb(0x0000ff)]),
            );
        })
        .expect("a linear gradient is expressible");
    assert!(out.contains("<linearGradient id=\"grad1\" gradientUnits=\"userSpaceOnUse\""));
    assert!(out.contains("<stop offset=\"0\" stop-color=\"#ff0000\"/>"));
    assert!(out.contains("<stop offset=\"1\" stop-color=\"#0000ff\"/>"));
    assert!(out.contains("fill=\"url(#grad1)\""));
    assert!(well_formed(&out));
}

#[test]
fn text_sits_on_the_baseline_the_engine_measures() {
    let mut renderer = Offscreen::new();
    renderer.font(TUFFY).expect("the bundled font registers");
    let mut baseline = None;
    let out = renderer
        .render_svg(120.0, 60.0, |draw, _size| {
            let style = TextFormat {
                size: 20.0,
                color: Color::BLACK,
                family: Some("Tuffy".into()),
                ..Default::default()
            };
            baseline = Some(draw.measure_text("Hi", &style).baseline);
            draw.text("Hi", Point::new(10.0, 10.0), style);
        })
        .expect("text is expressible");
    let expected = 10.0 + baseline.expect("the paint closure ran");
    let y = out
        .split("<text x=\"10\" y=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|value| value.parse::<f64>().ok())
        .expect("a text element with a y position");
    assert!((y - expected).abs() < 1e-3, "y {y} expected {expected}");
    assert!(out.contains("font-family=\"'Tuffy', sans-serif\""));
    assert!(well_formed(&out));
}

#[test]
fn an_invalid_size_is_an_error_and_never_calls_paint() {
    let mut renderer = Offscreen::new();
    let called = Cell::new(false);
    for (width, height) in [
        (0.0, 1.0),
        (1.0, 0.0),
        (-1.0, 1.0),
        (f64::NAN, 1.0),
        (f64::INFINITY, 1.0),
    ] {
        let error = renderer
            .render_svg(width, height, |_, _| called.set(true))
            .expect_err("an invalid size");
        assert_eq!(error.kind(), &ErrorKind::Message, "{width}x{height}");
        assert!(
            error
                .location()
                .is_some_and(|at| at.file().ends_with("offscreen_svg.rs"))
        );
    }
    assert!(!called.get());
}

#[test]
fn a_layer_svg_cannot_express_is_an_error() {
    let mut renderer = Offscreen::new();
    let error = renderer
        .render_svg(10.0, 10.0, |draw, _size| {
            draw.layer(Layer::new().blend(BlendMode::Multiply), |_| {});
        })
        .expect_err("a blend mode has no SVG 1.1 form");
    assert_eq!(error.kind(), &ErrorKind::Unsupported);
}

/// One drawing function for both exports: a background and a text that
/// takes the theme's family.
fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0xffffff));
    let style = TextFormat {
        size: 20.0,
        color: Color::BLACK,
        ..Default::default()
    };
    draw.text("Hi", Point::new(10.0, 10.0), style);
}

#[test]
fn one_offscreen_with_its_font_exports_the_pixels_and_the_document_of_one_drawing() {
    let theme = Theme::default().with_text(Typography {
        family: Some("Tuffy".into()),
        ..Theme::default().text().clone()
    });
    let mut renderer = Offscreen::new().theme(theme);
    renderer.font(TUFFY).expect("the bundled font registers");

    let image = renderer.render(120, 60, artwork).expect("pixels");
    let document = renderer.render_svg(120.0, 60.0, artwork).expect("svg");

    assert_eq!(image.rgba.len(), 120 * 60 * 4);
    assert!(
        image.rgba.chunks(4).any(|px| px[3] == 255 && px[0] < 64),
        "the text is drawn in the pixels"
    );
    assert!(document.contains("font-family=\"'Tuffy', sans-serif\""));
    assert!(document.contains("fill=\"#ffffff\""));
    assert!(well_formed(&document));
}
