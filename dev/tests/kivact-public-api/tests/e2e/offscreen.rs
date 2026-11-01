//! Role: Tests for offscreen rendering and drawing consistency.
//!
//! One drawing function through a canvas in a window and through
//! `Offscreen`: the same pixels (the windowless behaviour of `Offscreen`
//! is `tests/offscreen.rs`).

use std::sync::Arc;

use kivact::Asset;
use kivact::control::canvas;
use kivact::draw::Transform;
use kivact::draw::{Draw, Layer, Offscreen, RgbaImage, TextBaseline, TextStyle};
use kivact::draw::{Point, Rect, Size};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::style::{TextAlign, Theme};
use kivact::test::{RenderOptions, TestFonts, render_with};

use crate::paint::{distance, pixel};

static TUFFY: &[u8] = include_bytes!("../../../../../crates/arvane/assets/fonts/Tuffy.ttf");
static SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#00ff00"/></svg>"##;

fn theme() -> Theme {
    let theme = Theme::default();
    let text = kivact::style::theme::Typography {
        family: Some("Tuffy".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

fn artwork(draw: &mut Draw, size: Size, image: &Asset) {
    draw.fill_rect(size.to_rect(), Color::WHITE);
    draw.layer(
        Layer::new().clip_rect(Rect::new(0.0, 0.0, 96.0, 64.0)),
        |draw| {
            draw.layer(
                Layer::new().transform(Transform::new().translate(8, 4).rotate(5)),
                |draw| {
                    draw.layer(Layer::new().opacity(0.5), |draw| {
                        draw.fill_rect(Rect::new(0.0, 0.0, 30.0, 24.0), rgb(0xff0000));
                        draw.fill_rect(Rect::new(10.0, 8.0, 30.0, 24.0), rgb(0xff0000));
                    });
                },
            );
            draw.image(image, Rect::new(70.0, 4.0, 16.0, 16.0));
            draw.text(
                "Output",
                Point::new(45.0, 48.0),
                TextStyle {
                    size: 16.0,
                    color: Color::BLACK,
                    align: TextAlign::Center,
                    baseline: TextBaseline::Middle,
                    ..Default::default()
                },
            );
        },
    );
}

fn sample(image: &RgbaImage, x: u32, y: u32) -> u32 {
    let [r, g, b, a] = image.rgba[((y * image.width + x) * 4) as usize..][..4]
        .try_into()
        .expect("invariant: four bytes per pixel");
    assert_eq!(a, 255, "an opaque drawing at {x},{y}");
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

#[test]
fn a_drawing_function_gives_the_same_pixels_on_a_canvas_and_in_an_offscreen_image() {
    let image = Asset::from(SVG);
    let preview_image = image.clone();
    let screen = render_with(
        RenderOptions {
            width: 96.0,
            height: 64.0,
            theme: theme(),
            fonts: TestFonts::Only(vec![Arc::from(TUFFY)]),
            ..Default::default()
        },
        move || {
            canvas(move |draw, size| artwork(draw, size, &preview_image))
                .style(style().size(96, 64))
        },
    );
    let mut renderer = Offscreen::new().theme(theme());
    renderer.font(TUFFY).unwrap();
    let output = renderer
        .render(96, 64, |draw, size| artwork(draw, size, &image))
        .unwrap();
    assert_eq!(
        (output.width, output.height, output.rgba.len()),
        (96, 64, 96 * 64 * 4)
    );
    let preview = screen.screenshot();
    let dark = (0..64)
        .flat_map(|y| (0..96).map(move |x| (x, y)))
        .filter(|(x, y)| distance(sample(&output, *x, *y), 0x000000) <= 60)
        .count();
    assert!(dark > 20, "the text drew {dark} dark pixels");
    for y in 0..64 {
        for x in 0..96 {
            assert!(
                distance(sample(&output, x, y), pixel(&preview, x, y)) <= 2,
                "canvas/Offscreen at {x},{y}"
            );
        }
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
