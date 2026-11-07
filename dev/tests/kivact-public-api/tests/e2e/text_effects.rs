//! Role: Tests for text stroke, paint order, font axes and text paths.
//!
//! The glyph checks read the element's own pixels, on a white window. Each
//! comparison is tier 3 of §3.3 (the glyph raster): either
//! `Screenshot::raster_difference` between two frames, or a color class
//! check (stroke, fill, gradient) that a glyph's edge anti-aliasing leaves
//! unchanged.

use std::cell::RefCell;
use std::rc::Rc;

use super::paint::{BLACK, BLUE, RED, WHITE, at, canvas_of};
use kivact::draw::{Path, TextFormat, linear_gradient};
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact::style::{PaintOrder, rgb};
use kivact::test::{RenderOptions, Screenshot, render_with};
use kivact::view::Asset;

/// Inter's variable font (its `wght` and `opsz` axes), the test font.
static INTER: &[u8] = include_bytes!("../../../../../crates/arvane/tests/fonts/InterVariable.ttf");
/// The family name of `INTER`.
const FAMILY: &str = "Inter Variable";

fn inter() -> RenderOptions {
    RenderOptions::from_env().font(Asset::from(INTER))
}

/// The packed pixel at `(x, y)` of a shot, which lies inside it.
#[track_caller]
fn pixel_at(shot: &Screenshot, x: u32, y: u32) -> u32 {
    shot.pixel(x, y).expect("inside the shot")
}

/// Whether the packed pixel is a strong red (a stroke in red).
fn is_red(p: u32) -> bool {
    (p >> 16 & 255) > 200 && (p >> 8 & 255) < 60 && (p & 255) < 60
}

/// Whether the packed pixel is a strong blue (a fill in blue).
fn is_blue(p: u32) -> bool {
    (p & 255) > 200 && (p >> 16 & 255) < 60 && (p >> 8 & 255) < 60
}

/// Whether the packed pixel is dark (a fill in black).
fn is_dark(p: u32) -> bool {
    (p >> 16 & 255) < 80 && (p >> 8 & 255) < 80 && (p & 255) < 80
}

/// The middle row of a glyph's shot: the stem of an "I" crosses it.
fn middle_row(shot: &Screenshot) -> u32 {
    shot.height / 2
}

#[test]
fn text_stroke_outlines_glyphs() {
    // Tier 3 (glyph raster): the 2 px stroke is centered on the outline, so
    // its red starts 1 px outside the glyph and the black fill begins a
    // pixel or two further in.
    let screen = render_with(inter(), || {
        canvas_of(
            text("I").test_id("glyph").style(
                at(10, 10, 120, 140)
                    .font_family(FAMILY)
                    .font_size(96)
                    .color(rgb(BLACK))
                    .text_stroke(2, rgb(RED)),
            ),
        )
    });
    let shot = screen.get_by_test_id("glyph").screenshot();
    let row = middle_row(&shot);
    let first_red = (0..shot.width).find(|&x| is_red(pixel_at(&shot, x, row)));
    let first_dark = (0..shot.width).find(|&x| is_dark(pixel_at(&shot, x, row)));
    let red = first_red.expect("a red stroke along the glyph");
    let dark = first_dark.expect("a black fill inside the stroke");
    assert!(
        (1..=3).contains(&(dark - red)),
        "the stroke starts {} px before the fill (want 1 to 3)",
        dark - red
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn paint_order_stroke_first() {
    // Tier 3 (glyph raster): with the stroke first the fill covers the
    // inner half of the stroke, so no red lies inside the glyph; with the
    // fill first (the default) the inner half shows over the blue.
    let stroked = |order: PaintOrder| {
        render_with(inter(), move || {
            canvas_of(
                text("I").test_id("glyph").style(
                    at(10, 10, 120, 140)
                        .font_family(FAMILY)
                        .font_size(96)
                        .color(rgb(BLUE))
                        .text_stroke(6, rgb(RED))
                        .paint_order(order),
                ),
            )
        })
        .get_by_test_id("glyph")
        .screenshot()
    };
    let inside_red = |shot: &Screenshot| {
        let row = middle_row(shot);
        let blue: Vec<u32> = (0..shot.width)
            .filter(|&x| is_blue(pixel_at(shot, x, row)))
            .collect();
        let (Some(&first), Some(&last)) = (blue.first(), blue.last()) else {
            panic!("no blue fill in the glyph");
        };
        (first..last)
            .filter(|&x| is_red(pixel_at(shot, x, row)))
            .count()
    };
    let fill_first = stroked(PaintOrder::FillStroke);
    let stroke_first = stroked(PaintOrder::StrokeFill);
    assert!(
        inside_red(&fill_first) > 0,
        "the fill-first stroke covers the inner half"
    );
    assert_eq!(
        inside_red(&stroke_first),
        0,
        "the stroke-first glyph has no red inside it"
    );
}

#[test]
fn font_variation_wght_matches_weight() {
    // Tier 3 (glyph raster): `wght` 700 as a variation and as the weight
    // draw the same glyphs; both differ from weight 400.
    let sample = |variation: Option<f64>, weight: u16| {
        render_with(inter(), move || {
            let base = at(10, 10, 300, 80)
                .font_family(FAMILY)
                .font_size(48)
                .color(rgb(BLACK));
            let sized = match variation {
                Some(wght) => base.font_variation_settings([("wght", wght)]),
                None => base.weight(weight),
            };
            canvas_of(text("Hamburg").test_id("sample").style(sized))
        })
        .screenshot()
    };
    let as_variation = sample(Some(700.0), 400);
    let as_weight = sample(None, 700);
    let regular = sample(None, 400);
    let same = as_variation.raster_difference(&as_weight);
    assert!(same.within_raster_tolerance(), "{same:?}");
    let other = as_variation.raster_difference(&regular);
    assert!(!other.within_raster_tolerance(), "{other:?}");
}

#[test]
fn optical_sizing_changes_large_text() {
    // Tier 3 (glyph raster): at 48 px the `opsz` axis of Inter changes the
    // shapes when optical sizing follows the size and when it does not.
    let sample = |sizing: bool| {
        render_with(inter(), move || {
            canvas_of(
                text("Hamburg").test_id("sample").style(
                    at(10, 10, 300, 80)
                        .font_family(FAMILY)
                        .font_size(48)
                        .color(rgb(BLACK))
                        .font_optical_sizing(sizing),
                ),
            )
        })
        .screenshot()
    };
    let fixed = sample(false);
    let sized = sample(true);
    assert_ne!(
        fixed.pixels, sized.pixels,
        "optical sizing does not change 48 px text"
    );
}

#[test]
fn draw_text_with_gradient_and_path() {
    // Tier 3 (glyph raster): the gradient is laid over the text's bounds,
    // so the left of the text is red and the right is blue. The outline
    // path of the text lies inside the text's metrics (§9.7).
    let found: Rc<RefCell<Option<(Option<Rect>, Rect)>>> = Rc::default();
    let out = found.clone();
    let screen = render_with(inter(), move || {
        let out = out.clone();
        canvas(move |d, _| {
            let format = TextFormat {
                size: 60.0,
                weight: 700,
                family: Some(FAMILY.to_string()),
                fill: Some(linear_gradient(90).colors([rgb(RED), rgb(BLUE)]).into()),
                ..TextFormat::default()
            };
            let origin = Point::new(10.0, 10.0);
            let metrics = d.measure_text("IIIIIIII", &format);
            let path: Path = d.text_path("IIIIIIII", origin, &format);
            let inside = Rect::new(origin.x, origin.y, metrics.size.width, metrics.size.height);
            *out.borrow_mut() = Some((path.bounds(), inside));
            d.text("IIIIIIII", origin, format);
        })
        .style(style().size(400, 120).background(rgb(WHITE)))
    });
    let shot = screen.screenshot();
    let (bounds, inside) = found.borrow().clone().expect("the canvas drew the text");
    let bounds = bounds.expect("the text has an outline");
    assert!(
        bounds.x >= inside.x
            && bounds.y >= inside.y
            && bounds.x + bounds.width <= inside.x + inside.width
            && bounds.y + bounds.height <= inside.y + inside.height,
        "outline {bounds:?} lies outside the metrics {inside:?}"
    );
    let reddish = (0..shot.width / 4).any(|x| {
        (0..shot.height).any(|y| {
            let p = pixel_at(&shot, x, y);
            (p >> 16 & 255) > (p & 255) + 100
        })
    });
    let bluish = (shot.width * 3 / 4..shot.width).any(|x| {
        (0..shot.height).any(|y| {
            let p = pixel_at(&shot, x, y);
            (p & 255) > (p >> 16 & 255) + 100
        })
    });
    assert!(reddish, "the left of the gradient is red");
    assert!(bluish, "the right of the gradient is blue");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
