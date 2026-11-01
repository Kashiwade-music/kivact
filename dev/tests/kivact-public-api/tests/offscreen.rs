//! Role: Tests for offscreen image drawing without windows or apps.
//!
//! `kivact::draw::Offscreen`: fixed-resolution RGBA images drawn without an
//! App or a window. This binary opens no window (the comparison with a
//! canvas in a window is `e2e/offscreen.rs`).

use kivact::diagnostics::ErrorKind;
use kivact::draw::{Offscreen, RgbaImage, TextStyle};
use kivact::draw::{Point, Rect, Size};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::style::Theme;
use std::cell::Cell;

static TUFFY: &[u8] = include_bytes!("../../../../crates/arvane/assets/fonts/Tuffy.ttf");

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
    assert!(metrics.size.width > 0.0 && metrics.size.height > 0.0);
    let mut inked = 0;
    for y in 0..60 {
        for x in 0..90 {
            if px(&image, x, y) != [255, 255, 255, 255] {
                inked += 1;
                assert!(
                    (x as f32) >= 9.0
                        && (x as f32) <= 10.0 + metrics.size.width + 1.0
                        && (y as f32) >= 9.0
                        && (y as f32) <= 10.0 + metrics.size.height + 1.0,
                    "ink at {x},{y} outside the measured {metrics:?}"
                );
            }
        }
    }
    assert!(inked > 20, "text drew {inked} pixels");
}
