//! Role: Tests for offscreen image drawing without windows or apps.
//!
//! `kivact::draw::Offscreen`: fixed-resolution RGBA images drawn without an
//! App or a window. This binary opens no window (the comparison with a
//! canvas in a window is `e2e/offscreen.rs`).

use kivact::Asset;
use kivact::diagnostics::ErrorKind;
use kivact::draw::{Draw, ImageOptions, Layer, Offscreen, Path, Pixels, RgbaImage, TextStyle};
use kivact::draw::{Point, Rect, Size};
use kivact::draw::{Shadow, Shape, Stroke, linear_gradient};
use kivact::prelude::*;
use kivact::style::{Color, rgba};
use kivact::style::{ColorToken, LineHeight, Theme};
use std::cell::Cell;
use std::sync::Arc;

/// The kind of the error `result` holds; panics on an image.
#[track_caller]
fn kind<T: std::fmt::Debug>(result: kivact::Result<T>) -> ErrorKind {
    result.expect_err("an error").kind().clone()
}

static TUFFY: &[u8] = include_bytes!("../../../../crates/arvane/assets/fonts/Tuffy.ttf");
static SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#00ff00"/></svg>"##;

fn px(image: &RgbaImage, x: usize, y: usize) -> [u8; 4] {
    image.rgba[(y * image.width as usize + x) * 4..][..4]
        .try_into()
        .expect("invariant: four bytes per pixel")
}

fn tuffy_theme() -> Theme {
    let theme = Theme::default();
    let text = kivact::style::theme::Typography {
        family: Some("Tuffy".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

#[test]
fn a_size_outside_what_windows_draw_is_invalid_and_never_calls_paint() {
    let mut renderer = Offscreen::new();
    let called = Cell::new(false);
    for (width, height) in [
        (0, 1),
        (1, 0),
        (0, 0),
        (16_385, 1),
        (1, 16_385),
        (4097, 4096),
        (u32::MAX, 1),
    ] {
        let error = renderer
            .render(width, height, |_, _| called.set(true))
            .expect_err("an invalid size");
        assert_eq!(error.kind(), &ErrorKind::Message, "{width}x{height}");
        assert!(
            error
                .location()
                .is_some_and(|at| at.file().ends_with("offscreen.rs"))
        );
    }
    assert!(!called.get());
    for (width, height) in [(16_384, 1), (1, 16_384)] {
        let image = renderer.render(width, height, |_, _| {}).unwrap();
        assert_eq!(image.rgba.len(), (width * height * 4) as usize);
    }
}

#[test]
fn the_image_is_exactly_sized_transparent_and_draw_knows_its_size() {
    let mut renderer = Offscreen::new();
    let mut seen = None;
    let image = renderer
        .render(5, 3, |draw, size| seen = Some((draw.size(), size)))
        .unwrap();
    assert_eq!(seen, Some((Size::new(5.0, 3.0), Size::new(5.0, 3.0))));
    assert_eq!((image.width, image.height, image.rgba.len()), (5, 3, 60));
    assert!(
        image.rgba.iter().all(|byte| *byte == 0),
        "starts transparent"
    );
}

#[test]
fn colors_are_straight_alpha_srgb() {
    let mut renderer = Offscreen::new();
    let image = renderer
        .render(8, 8, |draw, _| {
            draw.fill_rect(Rect::new(0.0, 0.0, 4.0, 8.0), rgba(0xff0000, 0.5));
            draw.fill_rect(Rect::new(4.0, 0.0, 4.0, 8.0), rgb(0x3366cc));
        })
        .unwrap();
    let [r, g, b, a] = px(&image, 2, 4);
    assert!(
        r >= 254 && g <= 1 && b <= 1,
        "not premultiplied: {r} {g} {b}"
    );
    assert!(a.abs_diff(128) <= 1, "alpha {a}");
    assert_eq!(px(&image, 6, 4), [0x33, 0x66, 0xcc, 255]);
}

#[test]
fn rects_and_text_are_drawn_where_they_are_placed() {
    let mut renderer = Offscreen::new().theme(tuffy_theme());
    renderer.font(TUFFY).unwrap();
    let mut measured = None;
    let image = renderer
        .render(120, 60, |draw, size| {
            draw.fill_rect(size.to_rect(), Color::WHITE);
            draw.fill_rect(Rect::new(100.0, 40.0, 10.0, 10.0), rgb(0xff0000));
            let style = TextStyle {
                size: 20.0,
                color: Color::BLACK,
                ..Default::default()
            };
            measured = Some(draw.measure_text("Hi", &style));
            draw.text("Hi", Point::new(10.0, 10.0), style);
        })
        .unwrap();
    assert_eq!(px(&image, 104, 44), [255, 0, 0, 255]);
    assert_eq!(px(&image, 90, 44), [255, 255, 255, 255]);
    let metrics = measured.unwrap();
    assert!(metrics.width > 0.0 && metrics.height > 0.0);
    let mut inked = 0;
    for y in 0..60 {
        for x in 0..90 {
            if px(&image, x, y) != [255, 255, 255, 255] {
                inked += 1;
                assert!(
                    (x as f32) >= 9.0
                        && (x as f32) <= 10.0 + metrics.width + 1.0
                        && (y as f32) >= 9.0
                        && (y as f32) <= 10.0 + metrics.height + 1.0,
                    "ink at {x},{y} outside the measured {metrics:?}"
                );
            }
        }
    }
    assert!(inked > 20, "text drew {inked} pixels");
}

#[test]
fn the_themes_tokens_text_family_and_line_height_apply() {
    let theme = tuffy_theme();
    let colors = kivact::style::theme::Palette {
        accent: rgb(0x336699),
        ..theme.colors().clone()
    };
    let text = kivact::style::theme::Typography {
        line_height: LineHeight::px(40),
        ..theme.text().clone()
    };
    let theme = theme.with_colors(colors).with_text(text);
    let mut renderer = Offscreen::new().theme(theme);
    renderer.font(TUFFY).unwrap();
    let (mut inherited, mut explicit) = (None, None);
    let image = renderer
        .render(8, 8, |draw, size| {
            draw.fill_rect(size.to_rect(), ColorToken::Accent);
            let own = TextStyle {
                family: Some("Tuffy".into()),
                line_height: Some(LineHeight::px(40)),
                ..Default::default()
            };
            inherited = Some(draw.measure_text("Wide text", &TextStyle::default()));
            explicit = Some(draw.measure_text("Wide text", &own));
        })
        .unwrap();
    assert_eq!(px(&image, 4, 4), [0x33, 0x66, 0x99, 255]);
    let (inherited, explicit) = (inherited.unwrap(), explicit.unwrap());
    assert_eq!(
        inherited, explicit,
        "family and line height are the theme's"
    );
    assert_eq!(inherited.height, 40.0);
    // The default theme's line height is not 40 px.
    let mut plain = Offscreen::new();
    let mut height = None;
    plain
        .render(8, 8, |draw, _| {
            height = Some(draw.measure_text("Wide text", &TextStyle::default()).height)
        })
        .unwrap();
    assert_ne!(height, Some(40.0));
}

#[test]
fn fonts_fail_loudly_and_stay_registered() {
    let mut renderer = Offscreen::new().theme(tuffy_theme());
    assert_eq!(kind(renderer.font(&b"not a font"[..])), ErrorKind::Asset);
    let families = renderer.font(TUFFY).unwrap();
    assert!(
        families.iter().any(|family| family == "Tuffy"),
        "{families:?}"
    );
    // The registration serves every later frame.
    for _ in 0..2 {
        let image = renderer
            .render(40, 24, |draw, _| {
                let style = TextStyle {
                    size: 20.0,
                    color: Color::BLACK,
                    ..Default::default()
                };
                draw.text("A", Point::new(4.0, 2.0), style);
            })
            .unwrap();
        assert!(
            image.rgba.as_chunks::<4>().0.iter().any(|p| p[3] > 0),
            "the glyph drew"
        );
    }
}

#[test]
fn a_failed_resource_is_an_error_each_time_and_the_renderer_stays_usable() {
    let mut renderer = Offscreen::new();
    let bad = Asset::from(&b"not an image"[..]);
    for _ in 0..2 {
        let result = renderer.render(8, 8, |draw, size| draw.image(&bad, size.to_rect()));
        assert_eq!(kind(result), ErrorKind::Asset);
    }
    // A command a canvas drops asks for no resource, so it is no failure.
    renderer
        .render(8, 8, |draw, _| {
            draw.image(&bad, Rect::new(0.0, 0.0, 0.0, 8.0));
        })
        .unwrap();
    let svg = Asset::from(SVG);
    let image = renderer
        .render(8, 8, |draw, size| draw.image(&svg, size.to_rect()))
        .unwrap();
    assert_eq!(px(&image, 4, 4), [0, 255, 0, 255]);
    // The image outlives the renderer and its assets.
    drop(renderer);
    assert_eq!(px(&image, 1, 1), [0, 255, 0, 255]);
}

#[test]
fn commands_draw_rejects_are_an_error_not_silently_dropped() {
    let mut renderer = Offscreen::new();
    let mismatched = renderer.render(8, 8, |draw, size| {
        draw.pixels(
            &Pixels::new(2, 1, Arc::from([0u8; 4])),
            size.to_rect(),
            ImageOptions::default(),
        )
    });
    assert_eq!(kind(mismatched), ErrorKind::Unsupported);
    let spread = renderer.render(8, 8, |draw: &mut Draw, _| {
        let shadow = Shadow {
            spread: 2.0,
            ..Shadow::new(0.0, 0.0, 4.0, Color::BLACK)
        };
        draw.layer(Layer::new().shadow(shadow), |_| {});
    });
    assert_eq!(kind(spread), ErrorKind::Unsupported);
    // And the next frame draws.
    let ok = renderer
        .render(2, 2, |draw, size| {
            draw.fill_rect(size.to_rect(), Color::WHITE)
        })
        .unwrap();
    assert_eq!(px(&ok, 1, 1), [255, 255, 255, 255]);
}

#[test]
fn values_the_painting_drops_are_invalid_draws_not_silent_omissions() {
    let mut renderer = Offscreen::new();
    // Stops so far apart that the gradient line overflows: Arvane would
    // reject the command, so it is dropped and reported (on a rectangle,
    // a shape and a mask), never left a hole in a successful image.
    let gradient = linear_gradient(90)
        .stop(rgb(0xff0000), -1.5e38f32)
        .stop(rgb(0x0000ff), 1.5e38f32);
    let narrow = Rect::new(0., 0., 0.5, 8.);
    for mode in ["rect", "shape", "mask"] {
        let result = renderer.render(8, 8, |draw, _| match mode {
            "rect" => draw.fill_rect(narrow, gradient.clone()),
            "shape" => draw.fill_shape(narrow, &Shape::circle(2), gradient.clone()),
            _ => {
                draw.layer(Layer::new().area(narrow).mask(gradient.clone()), |draw| {
                    draw.fill_rect(narrow, Color::WHITE)
                });
            }
        });
        assert_eq!(kind(result), ErrorKind::Unsupported, "{mode}");
    }
    // A gradient with sane stops is fine.
    renderer
        .render(8, 8, |draw, size| {
            draw.fill_rect(
                size.to_rect(),
                linear_gradient(90).colors([rgb(0xff0000), rgb(0x0000ff)]),
            )
        })
        .unwrap();
    let arcs = [
        Path::new().arc(Point::new(4., 4.), f32::NAN, 0, 360, false),
        Path::new().arc(Point::new(4., 4.), 3, f32::NAN, 90, false),
        Path::new().move_to(Point::new(0., 0.)).arc_to(
            (f32::NAN, 4),
            0,
            false,
            true,
            Point::new(8., 8.),
        ),
    ];
    for arc in &arcs {
        for mode in 0..2 {
            let result = renderer.render(8, 8, |draw, _| {
                if mode == 0 {
                    draw.fill_path(arc, rgb(0xff0000));
                } else {
                    draw.stroke_path(arc, &Stroke::new(2), rgb(0xff0000));
                }
            });
            assert_eq!(kind(result), ErrorKind::Unsupported);
        }
    }
    // Non-finite and negative rects, and a negative stroke.
    for rect in [
        Rect::new(f32::NAN, 0., 4., 4.),
        Rect::new(0., 0., -4., 4.),
        Rect::new(0., 0., f32::INFINITY, 4.),
    ] {
        assert_eq!(
            kind(renderer.render(8, 8, |draw, _| draw.fill_rect(rect, rgb(0xff0000)))),
            ErrorKind::Unsupported,
            "{rect:?}"
        );
    }
    // Valid commands around it do not hide the error, and the next frame draws.
    let result = renderer.render(8, 8, |draw, size| {
        draw.fill_rect(size.to_rect(), Color::WHITE);
        draw.stroke_rect(Rect::new(0., 0., 4., 4.), f32::NAN, Color::BLACK);
    });
    assert_eq!(kind(result), ErrorKind::Unsupported);
    let ok = renderer
        .render(2, 2, |draw, size| {
            draw.fill_rect(size.to_rect(), Color::WHITE)
        })
        .unwrap();
    assert_eq!(px(&ok, 0, 0), [255, 255, 255, 255]);
}
