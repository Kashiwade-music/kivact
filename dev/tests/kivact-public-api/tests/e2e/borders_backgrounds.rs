//! Role: Tests for border styles, border images and background layers.
//!
//! The bands of the border styles: dark is two thirds of the colour, light is
//! the colour itself. The border image follows the nine
//! slices of CSS Backgrounds 3 §6. Flat fills have crisp edges, so their
//! pixels are checked within ±4/255 (tier 2); the image pixels are
//! sampled at tile centres (tier 2).

use kivact::draw::{BackgroundClip, BackgroundOrigin, BlendMode, Paint, linear_gradient};
use kivact::prelude::*;
use kivact::style::{BorderImage, BorderImageRepeat, BorderStyle};
use kivact::test::render;
use kivact::view::Asset;

use crate::paint::{RED, WHITE, at, canvas_of, expect};

/// The border colour of the tests: light is it, dark is two thirds of it.
const EDGE: u32 = 0x906030;
const DARK: u32 = 0x604020;

/// A 40 × 40 box at `(x, y)` with a 9 px border of `style`.
fn bordered(x: i32, y: i32, style: BorderStyle) -> Node {
    node().style(at(x, y, 40, 40).border(9, rgb(EDGE)).border_style(style))
}

#[test]
fn border_styles_draw_their_bands() {
    // Each box is 40 px with a 9 px border; the samples sit in the outer
    // and inner halves (or thirds, for double) of the top and bottom sides.
    let screen = render(|| {
        canvas_of(vec![
            bordered(20, 20, BorderStyle::Double),
            bordered(80, 20, BorderStyle::Groove),
            bordered(140, 20, BorderStyle::Ridge),
            bordered(200, 20, BorderStyle::Inset),
            bordered(260, 20, BorderStyle::Outset),
        ])
    });
    let shot = screen.screenshot();
    // Double: outer third and inner third, a gap of background between.
    expect(&shot, 40, 22, EDGE, 4, "double: outer band");
    expect(&shot, 40, 24, WHITE, 4, "double: the gap between the bands");
    expect(&shot, 40, 27, EDGE, 4, "double: inner band");
    // Groove: dark outside, light inside; the bottom is the reverse.
    expect(&shot, 100, 22, DARK, 4, "groove: dark outer top");
    expect(&shot, 100, 27, EDGE, 4, "groove: light inner top");
    expect(&shot, 100, 58, EDGE, 4, "groove: light outer bottom");
    expect(&shot, 100, 53, DARK, 4, "groove: dark inner bottom");
    // Ridge: the groove's colours swapped.
    expect(&shot, 160, 22, EDGE, 4, "ridge: light outer top");
    expect(&shot, 160, 27, DARK, 4, "ridge: dark inner top");
    expect(&shot, 160, 58, DARK, 4, "ridge: dark outer bottom");
    expect(&shot, 160, 53, EDGE, 4, "ridge: light inner bottom");
    // Inset: the top and left dark; the bottom and right light.
    expect(&shot, 220, 22, DARK, 4, "inset: dark top outer");
    expect(&shot, 220, 27, DARK, 4, "inset: dark top inner");
    expect(&shot, 220, 58, EDGE, 4, "inset: light bottom outer");
    expect(&shot, 220, 53, EDGE, 4, "inset: light bottom inner");
    // Outset: the top and left light; the bottom and right dark.
    expect(&shot, 280, 22, EDGE, 4, "outset: light top outer");
    expect(&shot, 280, 27, EDGE, 4, "outset: light top inner");
    expect(&shot, 280, 58, DARK, 4, "outset: dark bottom outer");
    expect(&shot, 280, 53, DARK, 4, "outset: dark bottom inner");
}

#[test]
fn border_styles_per_side() {
    // A 6 px border, a different style on each side: top double (a gap
    // between two bands), left groove (dark outside, light inside), right
    // solid, bottom inset (light: the bottom and right of an inset are
    // light).
    let screen = render(|| {
        canvas_of(
            node().style(
                at(20, 20, 40, 40)
                    .border(6, rgb(EDGE))
                    .border_top_style(BorderStyle::Double)
                    .border_left_style(BorderStyle::Groove)
                    .border_right_style(BorderStyle::Solid)
                    .border_bottom_style(BorderStyle::Inset),
            ),
        )
    });
    let shot = screen.screenshot();
    expect(&shot, 40, 21, EDGE, 4, "top: the outer band");
    expect(&shot, 40, 23, WHITE, 4, "top: the gap of the double");
    expect(&shot, 40, 25, EDGE, 4, "top: the inner band");
    expect(&shot, 21, 40, DARK, 4, "left: dark outside of the groove");
    expect(&shot, 24, 40, EDGE, 4, "left: light inside of the groove");
    expect(&shot, 58, 40, EDGE, 4, "right: solid");
    expect(&shot, 40, 58, EDGE, 4, "bottom: the light of an inset");
    expect(
        &shot,
        40,
        56,
        EDGE,
        4,
        "bottom: the light of an inset, inner",
    );
}

/// A 10 × 10 tile per cell of a 3 × 3 picture. The top middle has a green
/// half and a blue half, so the tiles of its edge are seen.
static NINE_SLICE: &[u8] =
    br##"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 30 30">
<rect x="0" y="0" width="10" height="10" fill="#ff0000"/>
<rect x="10" y="0" width="5" height="10" fill="#00ff00"/>
<rect x="15" y="0" width="5" height="10" fill="#0000ff"/>
<rect x="20" y="0" width="10" height="10" fill="#ffff00"/>
<rect x="0" y="10" width="10" height="10" fill="#ff00ff"/>
<rect x="10" y="10" width="10" height="10" fill="#808080"/>
<rect x="20" y="10" width="10" height="10" fill="#00ffff"/>
<rect x="0" y="20" width="10" height="10" fill="#000000"/>
<rect x="10" y="20" width="10" height="10" fill="#804000"/>
<rect x="20" y="20" width="10" height="10" fill="#008080"/>
</svg>"##;

#[test]
fn border_image_nine_slice() {
    // A 120 × 60 box at (20, 20). Slices of 10 px: the corners keep their
    // colours. The top edge is 100 px long and its tile 10 px, so it has
    // 10 round tiles: the green and blue halves repeat at 10 px steps.
    let screen = render(|| {
        canvas_of(
            node().style(
                at(20, 20, 120, 60).border_image(
                    BorderImage::new(Asset::from(NINE_SLICE))
                        .slice(10)
                        .width(10)
                        .repeat(BorderImageRepeat::Round),
                ),
            ),
        )
    });
    let shot = screen.screenshot();
    expect(&shot, 25, 25, RED, 4, "the top-left corner");
    expect(&shot, 135, 25, 0xffff00, 4, "the top-right corner");
    expect(&shot, 25, 75, 0x000000, 4, "the bottom-left corner");
    expect(&shot, 135, 75, 0x008080, 4, "the bottom-right corner");
    expect(
        &shot,
        31,
        25,
        0x00ff00,
        4,
        "the top edge: the first tile's green",
    );
    expect(
        &shot,
        36,
        25,
        0x0000ff,
        4,
        "the top edge: the first tile's blue",
    );
    expect(
        &shot,
        40,
        25,
        0x00ff00,
        4,
        "the top edge: the second tile starts",
    );
    expect(
        &shot,
        121,
        25,
        0x00ff00,
        4,
        "the top edge: the last tile's green",
    );
    expect(
        &shot,
        129,
        25,
        0x0000ff,
        4,
        "the top edge: the last tile's blue",
    );
    expect(&shot, 25, 50, 0xff00ff, 4, "the left edge");
    expect(&shot, 135, 50, 0x00ffff, 4, "the right edge");
    expect(&shot, 80, 75, 0x804000, 4, "the bottom edge");
    expect(&shot, 80, 50, WHITE, 4, "the middle is not filled");
}

#[test]
fn background_clip_text_paints_glyphs_only() {
    // The background is a red-to-blue gradient clipped to the glyphs, and
    // the text colour is transparent: the glyphs show the gradient, the
    // rest of the box shows the canvas.
    let screen = render(|| {
        canvas_of(
            text("I").style(
                at(20, 20, 200, 80)
                    .font_size(64)
                    .color(Color::TRANSPARENT)
                    .background(
                        Paint::from(linear_gradient(90).colors([rgb(RED), rgb(0x0000ff)]))
                            .clip(BackgroundClip::Text),
                    ),
            ),
        )
    });
    let shot = screen.screenshot();
    let stem: Vec<u32> = (20..60)
        .filter(|&x| (0..80).any(|y| dark_enough(shot.pixel(x, 20 + y).unwrap_or(WHITE))))
        .collect();
    assert!(!stem.is_empty(), "the glyph is painted");
    // Inside the stem (its first pixels), the gradient's red end.
    let inside = shot.pixel(stem[0] + 3, 60).expect("in the frame");
    let (r, b) = ((inside >> 16) & 0xff, inside & 0xff);
    assert!(
        r > 180 && b < 120,
        "the glyph shows the gradient's red end: {inside:06x}"
    );
    expect(&shot, 150, 60, WHITE, 2, "beside the glyph: the canvas");
    expect(
        &shot,
        215,
        25,
        WHITE,
        2,
        "the box's corner away from the glyph",
    );
}

/// Whether a pixel is clearly not the canvas.
fn dark_enough(p: u32) -> bool {
    p != WHITE && p.abs_diff(WHITE) > 32
}

#[test]
fn background_origin_and_blend() {
    // A 10 × 10 red picture in a 60 × 60 box with 10 px padding: at the
    // content-box origin it sits 10 px further in than at the padding-box
    // origin. A grey (0.5) multiplied over red gives (128, 0, 0).
    let square = Asset::from(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#ff0000"/></svg>"##
            .as_slice(),
    );
    let screen = render(move || {
        let content = square.clone();
        let padding = content.clone();
        canvas_of((
            node().style(
                at(20, 20, 60, 60).padding(10).background(
                    Paint::image(content)
                        .origin(BackgroundOrigin::ContentBox)
                        .no_repeat(),
                ),
            ),
            node().style(
                at(100, 20, 60, 60).padding(10).background(
                    Paint::image(padding)
                        .origin(BackgroundOrigin::PaddingBox)
                        .no_repeat(),
                ),
            ),
            node().style(at(200, 20, 60, 60).background((
                Paint::from(rgb(0x808080)).blend(BlendMode::Multiply),
                Paint::from(rgb(RED)),
            ))),
        ))
    });
    let shot = screen.screenshot();
    expect(
        &shot,
        25,
        25,
        WHITE,
        2,
        "content origin: the padding is clear",
    );
    expect(
        &shot,
        35,
        35,
        RED,
        2,
        "content origin: the picture is inset",
    );
    expect(
        &shot,
        105,
        25,
        RED,
        2,
        "padding origin: the picture at the corner",
    );
    expect(&shot, 115, 35, WHITE, 2, "padding origin: not inset");
    expect(&shot, 230, 50, 0x800000, 2, "multiply: grey over red");
}
