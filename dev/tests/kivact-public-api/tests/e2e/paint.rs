//! Role: Tests for CSS paint properties and rendering.
//!
//! Element paint as CSS paints a box: gradients and background layers
//! (tiles, sizes, positions, images), per-side and dashed borders,
//! percentage and elliptical radii, inset and stacked shadows, rounded
//! overflow clips and clip paths, which also clip hit testing. Checked on
//! the frame's pixels (scale 1: a logical pixel is a physical one).

use kivact::draw::{Shadow, conic_gradient, linear_gradient, radial_gradient};
use kivact::prelude::*;
use kivact::style::{BorderStyle, Position};
use kivact::style::{Color, Style, pct};
use kivact::test::{Screenshot, render};

pub(crate) const RED: u32 = 0xff0000;
pub(crate) const BLUE: u32 = 0x0000ff;
pub(crate) const GREEN: u32 = 0x00ff00;
pub(crate) const WHITE: u32 = 0xffffff;
pub(crate) const BLACK: u32 = 0x000000;

/// A white 800 × 600 canvas holding `children`, each placed with [`at`].
pub(crate) fn canvas_of(children: impl IntoView) -> Node {
    node()
        .child(children)
        .style(style().size(800, 600).background(Color::WHITE))
}

/// A box of `w × h` at `(x, y)` of the canvas.
pub(crate) fn at(x: i32, y: i32, w: i32, h: i32) -> Style {
    style()
        .position(Position::Absolute)
        .left(x)
        .top(y)
        .size(w, h)
}

/// The color of a pixel (0xRRGGBB).
#[track_caller]
pub(crate) fn pixel(shot: &Screenshot, x: u32, y: u32) -> u32 {
    shot.pixel(x, y).expect("in the frame")
}

/// The largest channel difference between two colors.
pub(crate) fn distance(a: u32, b: u32) -> u32 {
    (0..3)
        .map(|i| ((a >> (8 * i)) & 0xff).abs_diff((b >> (8 * i)) & 0xff))
        .max()
        .unwrap_or(0)
}

/// The pixel at `(x, y)` is `want`, within `tolerance` per channel.
#[track_caller]
pub(crate) fn expect(shot: &Screenshot, x: u32, y: u32, want: u32, tolerance: u32, what: &str) {
    let got = pixel(shot, x, y);
    assert!(
        distance(got, want) <= tolerance,
        "{what}: pixel ({x}, {y}) is {got:06x}, want {want:06x} (±{tolerance})"
    );
}

#[test]
fn gradients_place_their_stops_as_css_does() {
    let screen = render(|| {
        canvas_of((
            node().style(
                at(0, 0, 200, 20).background(linear_gradient(90).colors([rgb(RED), rgb(BLUE)])),
            ),
            // A hard stop: two stops at the same position.
            node().style(
                at(0, 30, 200, 20).background(
                    linear_gradient(90)
                        .stop(rgb(RED), pct(50))
                        .stop(rgb(BLUE), pct(50)),
                ),
            ),
            // CSS `repeating-linear-gradient(90deg, black 0 2px, white 2px 6px)`;
            // a stop before an earlier one moves up to it.
            node().style(
                at(0, 60, 60, 10).background(
                    linear_gradient(90)
                        .repeating()
                        .stop(rgb(BLACK), 0)
                        .stop(rgb(BLACK), 2)
                        .stop(rgb(WHITE), 0)
                        .stop(rgb(WHITE), 6),
                ),
            ),
            // CSS `linear-gradient(red, blue)` runs to bottom (180).
            node().style(
                at(300, 0, 20, 100).background(linear_gradient(180).colors([rgb(RED), rgb(BLUE)])),
            ),
            // CSS `radial-gradient(circle 20px, red 10px, white 10px)`.
            node().style(
                at(0, 100, 60, 60).background(
                    radial_gradient()
                        .radius(20)
                        .stop(rgb(RED), 10)
                        .stop(rgb(WHITE), 10),
                ),
            ),
            // CSS `conic-gradient(red 0 90deg, blue 90deg)`.
            node().style(
                at(100, 100, 60, 60).background(
                    conic_gradient(0)
                        .stop(rgb(RED), 0)
                        .stop(rgb(RED), 90)
                        .stop(rgb(BLUE), 90)
                        .stop(rgb(BLUE), 360),
                ),
            ),
            // A repeating radial gradient: rings 4 px wide.
            node().style(
                at(200, 100, 60, 60).background(
                    radial_gradient()
                        .circle()
                        .repeating()
                        .stop(rgb(BLACK), 0)
                        .stop(rgb(BLACK), 4)
                        .stop(rgb(WHITE), 4)
                        .stop(rgb(WHITE), 8),
                ),
            ),
        ))
    });
    let shot = screen.screenshot();
    expect(&shot, 0, 10, RED, 4, "linear start");
    expect(&shot, 199, 10, BLUE, 4, "linear end");
    expect(&shot, 100, 10, 0x800080, 6, "linear middle");
    expect(&shot, 98, 40, RED, 0, "hard stop, left");
    expect(&shot, 101, 40, BLUE, 0, "hard stop, right");
    for (x, want) in [
        (0, BLACK),
        (1, BLACK),
        (3, WHITE),
        (6, BLACK),
        (9, WHITE),
        (13, BLACK),
    ] {
        expect(&shot, x, 65, want, 0, "repeating period");
    }
    expect(&shot, 310, 1, RED, 6, "to bottom: red on top");
    expect(&shot, 310, 98, BLUE, 6, "to bottom: blue below");
    expect(&shot, 30, 130, RED, 0, "radial center");
    expect(&shot, 30 + 14, 130, WHITE, 0, "radial beyond 10 px");
    expect(&shot, 140, 120, RED, 0, "conic: up to the right");
    expect(&shot, 140, 140, BLUE, 0, "conic: down to the right");
    expect(&shot, 120, 140, BLUE, 0, "conic: down to the left");
    expect(&shot, 232, 130, BLACK, 0, "rings: 2 px out");
    expect(&shot, 236, 130, WHITE, 0, "rings: 6 px out");
    expect(&shot, 240, 130, BLACK, 0, "rings: 10 px out");
}

#[test]
fn borders_have_sides_styles_and_fractional_widths() {
    let screen = render(|| {
        canvas_of((
            node().style(at(0, 0, 40, 40).border_left(3, rgb(RED))),
            node().style(
                at(50, 0, 40, 40)
                    .border_top(4, rgb(RED))
                    .border_bottom(4, rgb(BLUE)),
            ),
            node().style(
                at(100, 0, 60, 20)
                    .border(2, rgb(BLACK))
                    .border_style(BorderStyle::Dashed),
            ),
            node().style(
                at(200, 0, 60, 20)
                    .border(4, rgb(BLACK))
                    .border_style(BorderStyle::Dotted),
            ),
            node().style(at(300, 0, 40, 40).border(1.5, rgb(BLACK))),
            // Per-side widths and colors with rounded corners.
            node().style(
                at(400, 0, 60, 60)
                    .radius(12)
                    .border_left(6, rgb(RED))
                    .border_top(2, rgb(BLUE)),
            ),
            // A state style changes the color and keeps the width.
            node().test_id("hovered").style(
                at(500, 0, 40, 40)
                    .border(3, rgb(BLACK))
                    .hover(style().border_color(rgb(RED))),
            ),
        ))
    });
    let shot = screen.screenshot();
    expect(&shot, 1, 20, RED, 0, "left border");
    expect(&shot, 4, 20, WHITE, 0, "inside the left border");
    expect(&shot, 20, 1, WHITE, 0, "no top border");
    expect(&shot, 70, 1, RED, 0, "top border");
    expect(&shot, 70, 38, BLUE, 0, "bottom border");
    expect(&shot, 51, 20, WHITE, 0, "no left border");
    let row =
        |y: u32, x0: u32, x1: u32| -> Vec<u32> { (x0..x1).map(|x| pixel(&shot, x, y)).collect() };
    let dashes = row(0, 108, 152);
    assert!(
        dashes.contains(&BLACK) && dashes.contains(&WHITE),
        "dashed: dashes and gaps along the top {dashes:06x?}"
    );
    let dots = row(2, 208, 252);
    assert!(
        dots.iter().any(|p| distance(*p, BLACK) < 40) && dots.contains(&WHITE),
        "dotted: dots and gaps along the top"
    );
    expect(&shot, 300, 20, BLACK, 0, "1.5 px border, first pixel");
    let half = pixel(&shot, 301, 20) & 0xff;
    assert!(
        (60..200).contains(&half),
        "1.5 px border, half a pixel: gray {half}"
    );
    expect(&shot, 402, 30, RED, 0, "rounded left side");
    expect(&shot, 430, 0, BLUE, 0, "rounded top side");
    expect(&shot, 430, 30, WHITE, 0, "inside");
    expect(&shot, 401, 1, WHITE, 0, "outside the rounded corner");
    screen.get_by_test_id("hovered").hover();
    let shot = screen.screenshot();
    expect(&shot, 501, 20, RED, 0, "hovered: the color changed");
    expect(&shot, 504, 20, WHITE, 0, "hovered: the width kept");
}

#[test]
fn shadows_inset_negative_spread_stacked_and_shaped() {
    let screen = render(|| {
        canvas_of((
            node().style(
                at(20, 20, 60, 60)
                    .background(Color::WHITE)
                    .shadow(Shadow::new(0, 0, 0, rgb(BLACK)).spread(4).inset()),
            ),
            node().style(
                at(120, 20, 40, 40)
                    .background(Color::WHITE)
                    .shadow(Shadow::new(0, 10, 0, rgb(BLACK)).spread(-5)),
            ),
            // The first shadow is on top.
            node().style(
                at(220, 20, 40, 40)
                    .background(Color::WHITE)
                    .shadow(Shadow::new(0, 10, 0, rgb(RED)))
                    .shadow(Shadow::new(0, 10, 0, rgb(BLUE)).spread(2)),
            ),
            // A glow around an ellipse follows it.
            node().style(
                at(320, 20, 40, 40)
                    .radius(pct(50))
                    .background(Color::WHITE)
                    .shadow(Shadow::new(0, 0, 0, rgb(RED)).spread(4)),
            ),
            // An inset shadow with a blur.
            node().style(
                at(420, 20, 60, 60)
                    .background(Color::WHITE)
                    .shadow(Shadow::new(0, 0, 12, rgb(BLACK)).inset()),
            ),
        ))
    });
    let shot = screen.screenshot();
    expect(&shot, 22, 50, BLACK, 0, "inset band");
    expect(&shot, 30, 50, WHITE, 0, "inside the inset band");
    expect(&shot, 17, 50, WHITE, 0, "nothing outside an inset shadow");
    expect(&shot, 140, 62, BLACK, 0, "shrunk shadow below the box");
    expect(&shot, 140, 67, WHITE, 0, "shrunk by 5");
    expect(&shot, 123, 62, WHITE, 0, "shrunk at the side");
    expect(&shot, 240, 65, RED, 0, "the first shadow on top");
    expect(&shot, 218, 65, BLUE, 0, "the second, wider, beneath");
    expect(&shot, 340, 17, RED, 0, "glow above the circle");
    expect(&shot, 318, 18, WHITE, 0, "no glow in the corner");
    let edge = pixel(&shot, 421, 50) & 0xff;
    let middle = pixel(&shot, 450, 50) & 0xff;
    assert!(
        edge < middle && middle > 240,
        "a blurred inset shadow darkens the edge ({edge}) and fades inwards ({middle})"
    );
}
