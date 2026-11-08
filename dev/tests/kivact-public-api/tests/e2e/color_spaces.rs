//! Role: Tests for CSS color spaces, color parsing and gradient interpolation.
//!
//! Colors as the public API paints them. A pixel is compared with a value
//! the test computes from the formulas of CSS Color 4 (written out below,
//! in `reference`), or with a known number listed in the test. The tier of
//! each comparison is named where it is made; the tiers are defined in
//! `support`.

use super::paint::{BLACK, BLUE, RED, WHITE, at, canvas_of, expect, pixel};
use kivact::draw::linear_gradient;
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::{Color, ColorSpace, TransitionProp, lab, oklab, oklch, pct, rgb};
use kivact::test::{RenderOptions, Screen, render, render_with};

use crate::support::color_reference::{
    self as reference, V3, encoded_to_oklab, gamut_map, oklab_to_encoded,
};

/// A window of `w × h` holding one box of the given paint at `(0, 0)`.
fn one_box(w: i32, h: i32, paint: Color) -> Screen {
    render(move || canvas_of(node().style(at(0, 0, w, h).background(paint))))
}

/// The packed pixel at the center of a box at `(0, 0)` of size `w × h`.
fn center(screen: &Screen, w: u32, h: u32) -> u32 {
    pixel(&screen.screenshot(), w / 2, h / 2)
}

/// The expected pixel of `t` along the gradient that runs from `a` to `b`
/// in Oklab, with `a` and `b` unclipped (encoded sRGB, Oklab lerp).
fn oklab_lerp_at(a: V3, b: V3, t: f64) -> u32 {
    let (la, lb) = (encoded_to_oklab(a), encoded_to_oklab(b));
    let mid = [0, 1, 2].map(|i| la[i] + (lb[i] - la[i]) * t);
    reference::packed(gamut_map(oklab_to_encoded(mid)))
}

#[test]
fn color_conversion_fixtures() {
    // The reference formulas first: they must give the known values
    // (±1e-4), since the expected pixels below are computed from them.
    let close = |got: V3, want: V3, what: &str| {
        for i in 0..3 {
            assert!(
                (got[i] - want[i]).abs() < 1e-4,
                "{what}: {got:?}, want {want:?}"
            );
        }
    };
    close(
        reference::encoded_to_oklab([1.0, 0.0, 0.0]),
        [0.627955, 0.224863, 0.125846],
        "reference oklab of sRGB red",
    );
    close(
        reference::encoded_to_oklab([1.0, 1.0, 1.0]),
        [1.0, 0.0, 0.0],
        "reference oklab of white",
    );
    close(
        reference::encoded_to_lab([1.0, 0.0, 0.0]),
        [54.2905, 80.8049, 69.8910],
        "reference lab (D50) of sRGB red",
    );
    close(
        reference::p3_to_srgb([1.0, 0.0, 0.0]),
        [1.093066, -0.226742, -0.150135],
        "reference unclipped sRGB of display-p3 red",
    );

    // Tier 2 (the paint of a color, ±1/255 for the known sRGB values and
    // ±2/255 for the gamut mapped display-p3 value). The public API does not
    // expose the converted values, so they are checked through the paint.
    let red = one_box(40, 40, oklab(0.627955, 0.224863, 0.125846));
    expect(&red.screenshot(), 20, 20, RED, 1, "oklab of sRGB red");
    let white = one_box(40, 40, oklab(1.0, 0.0, 0.0));
    expect(&white.screenshot(), 20, 20, WHITE, 1, "oklab white");
    let lab_red = one_box(40, 40, lab(54.2905, 80.8049, 69.8910));
    expect(
        &lab_red.screenshot(),
        20,
        20,
        RED,
        1,
        "lab (D50) of sRGB red",
    );

    // `color(display-p3 1 0 0)` is out of sRGB; it paints the CSS gamut
    // mapping of its unclipped sRGB, computed here from the CSS Color 4 §18
    // matrices.
    let p3 = Color::css("color(display-p3 1 0 0)").expect("a color() value");
    let p3_red = one_box(40, 40, p3);
    let unclipped = reference::p3_to_srgb([1.0, 0.0, 0.0]);
    let mapped = reference::packed(gamut_map(unclipped));
    expect(&p3_red.screenshot(), 20, 20, mapped, 2, "display-p3 red");
}

#[test]
fn oklch_out_of_gamut_maps_like_css() {
    // Tier 2 for the value. The lightness and hue are read back from the
    // pixel: L within ±0.02; hue within ±1.0° (8-bit pixels cannot resolve
    // 0.02°); chroma below the source's.
    let screen = one_box(40, 40, oklch(0.7, 0.4, 30.0));
    let p = center(&screen, 40, 40);
    let enc = reference::pixel_encoded(p);
    assert!(enc.iter().all(|v| (0.0..=1.0).contains(v)), "in sRGB");
    let back = reference::oklab_to_oklch(encoded_to_oklab(enc));
    assert!(
        (back[0] - 0.7).abs() <= 0.02,
        "lightness {} vs 0.7",
        back[0]
    );
    assert!(back[1] < 0.4, "chroma reduced: {}", back[1]);
    let hue_error = (back[2] - 30.0).abs();
    assert!(hue_error <= 1.0, "hue {} vs 30 (±{hue_error})", back[2]);
    // The CSS gamut mapping of the same color (CSS Color 4 §13.2.2).
    let want = reference::packed(gamut_map(oklab_to_encoded(reference::oklch_to_oklab(
        0.7, 0.4, 30.0,
    ))));
    expect(&screen.screenshot(), 20, 20, want, 2, "gamut mapped value");
}

#[test]
fn wide_gamut_gradient_interpolates_source_colors() {
    // Tier 3: a gradient of Display P3 green to oklch(0.6 0.3 300). The
    // expected value interpolates the two source colors' Oklab values
    // unclipped, then gamut maps the result.
    let width: i32 = 200;
    let screen = render(move || {
        canvas_of(
            node().style(
                at(0, 0, width, 40).background(
                    linear_gradient(90)
                        .colors([
                            Color::css("color(display-p3 0 1 0)").expect("a color() value"),
                            oklch(0.6, 0.3, 300.0),
                        ])
                        .interpolate(ColorSpace::Oklab),
                ),
            ),
        )
    });
    let shot = screen.screenshot();
    let a = reference::xyz_to_oklab(reference::mul(&reference::P3_TO_XYZ, [0.0, 1.0, 0.0]));
    let b = reference::oklch_to_oklab(0.6, 0.3, 300.0);
    for x in [50u32, 99, 100, 150] {
        let t = (f64::from(x) + 0.5) / f64::from(width);
        let mid = [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t);
        let want = reference::packed(gamut_map(oklab_to_encoded(mid)));
        expect(&shot, x, 20, want, 3, "gradient point");
    }
}

#[test]
fn css_color_parsing() {
    // The examples of the supported forms parse to the colors they name.
    assert_eq!(
        Color::css("hsl(120deg 50% 25% / 0.5)").expect("hsl"),
        Color::from_space(ColorSpace::Hsl, 120.0, 0.5, 0.25).with_alpha(0.5)
    );
    assert_eq!(
        Color::css("rgb(255 0 0 / 50%)").expect("rgb"),
        kivact::style::rgba(0xff0000, 0.5)
    );
    // Names and the case of keywords do not matter.
    assert_eq!(
        Color::css("RED").expect("a name"),
        Color::css("red").expect("a name")
    );
    // rgb() channels, and hsl() and hwb() saturation, lightness, whiteness
    // and blackness, are clamped to their ranges when parsed (CSS Color 4
    // §4.2.1, §7.1 and §8.1). The color() spaces are not clamped.
    let css = |s: &str| Color::css(s).expect(s);
    assert_eq!(css("rgb(300 -20 128)"), css("rgb(255 0 128)"));
    assert_eq!(css("rgb(150% -5% 50%)"), css("rgb(100% 0% 50%)"));
    assert_eq!(css("hsl(120 150% -10%)"), css("hsl(120 100% 0%)"));
    assert_eq!(css("hwb(0 120% 0%)"), css("hwb(0 100% 0%)"));
    assert_ne!(css("color(srgb 1.2 -0.1 0.5)"), css("color(srgb 1 0 0.5)"));
    // These are errors: the forms are not supported.
    for input in [
        "none",
        "calc(255) 0 0",
        "rgb(calc(255) 0 0)",
        "currentcolor",
    ] {
        assert!(Color::css(input).is_err(), "{input} must be an error");
    }
}

#[test]
fn gradient_interpolation_spaces() {
    // Tier 3: the known values for the midpoint of red and blue
    // (the window is 201 px wide so x = 100 has t = 0.5). Srgb: (128, 0,
    // 128) ±2; Oklab: (140, 83, 162) ±3.
    for (space, want, tolerance) in [
        (ColorSpace::Srgb, 0x800080, 2),
        (ColorSpace::Oklab, 0x8c53a2, 3),
    ] {
        let screen = render(move || {
            canvas_of(
                node().style(
                    at(0, 0, 201, 40).background(
                        linear_gradient(90)
                            .colors([RED, BLUE].map(rgb))
                            .interpolate(space),
                    ),
                ),
            )
        });
        expect(
            &screen.screenshot(),
            100,
            20,
            want,
            tolerance,
            &format!("midpoint in {space:?}"),
        );
    }
}

#[test]
fn gradient_hint_moves_the_midpoint() {
    // Tier 3: the gradient lookup table allows a few levels; the hint's
    // midpoint is
    // 128 ±2, the row never decreases, and near the end the values are the
    // formula's (about 49 at x = 3990, about 234 at x = 3999). The hint sits
    // between the black and the white stop (a hint after the last stop has
    // no interval to act on).
    let width: u32 = 4000;
    for hint in [25.0, 0.5] {
        let screen = render_with(
            RenderOptions::from_env().window_size(f64::from(width), 40.0),
            move || {
                node().style(
                    style().size(width as i32, 40).background(
                        linear_gradient(90)
                            .colors([rgb(BLACK)])
                            .hint(pct(hint))
                            .colors([rgb(WHITE)])
                            .interpolate(ColorSpace::Srgb),
                    ),
                )
            },
        );
        let shot = screen.screenshot();
        let at_hint = (hint / 100.0 * f64::from(width)) as u32;
        let p = pixel(&shot, at_hint, 20) & 255;
        assert!(
            (126..=130).contains(&p),
            "hint {hint}%: midpoint {p} at x {at_hint}"
        );
        let row: Vec<u32> = (0..width).map(|x| pixel(&shot, x, 20) & 255).collect();
        assert!(
            row.windows(2).all(|w| w[0] <= w[1]),
            "hint {hint}%: the row decreases"
        );
    }
    let screen = render_with(
        RenderOptions::from_env().window_size(f64::from(width), 40.0),
        move || {
            node().style(
                style().size(width as i32, 40).background(
                    linear_gradient(90)
                        .colors([rgb(BLACK)])
                        .hint(pct(99.9))
                        .colors([rgb(WHITE)])
                        .interpolate(ColorSpace::Srgb),
                ),
            )
        },
    );
    let shot = screen.screenshot();
    let near_end = pixel(&shot, 3990, 20) & 255;
    let last = pixel(&shot, 3999, 20) & 255;
    assert!(near_end < 64, "x 3990 is {near_end}, want < 64");
    assert!(last >= 200, "x 3999 is {last}, want >= 200");
}

#[test]
fn same_color_gradient_matches_solid() {
    // Tier 3: a solid fill and a gradient whose stops are both the same
    // gamut mapped color.
    let color = oklch(0.7, 0.4, 30.0);
    let solid = one_box(100, 100, color);
    let gradient = render(move || {
        canvas_of(
            node().style(at(0, 0, 100, 100).background(linear_gradient(90).colors([color, color]))),
        )
    });
    let diff = solid.screenshot().raster_difference(&gradient.screenshot());
    assert!(diff.within_raster_tolerance(), "{diff:?}");
}

#[test]
fn wide_gamut_to_white_gradient_maps_each_point() {
    // Tier 2: the sRGB interpolation of the unclipped color and white, then
    // the gamut mapping of each point (the end points are not mapped first).
    let width = 201;
    let start = oklch(0.7, 0.4, 30.0);
    let screen = render(move || {
        canvas_of(
            node().style(
                at(0, 0, width, 40).background(
                    linear_gradient(90)
                        .colors([start, Color::WHITE])
                        .interpolate(ColorSpace::Srgb),
                ),
            ),
        )
    });
    let shot = screen.screenshot();
    let unclipped = oklab_to_encoded(reference::oklch_to_oklab(0.7, 0.4, 30.0));
    for x in [40u32, 100, 160] {
        let t = (f64::from(x) + 0.5) / f64::from(width as u32);
        let mix = [0, 1, 2].map(|i| unclipped[i] + (1.0 - unclipped[i]) * t);
        let want = reference::packed(gamut_map(mix));
        expect(&shot, x, 20, want, 3, "sRGB mix, then mapped");
    }
    // The end-point-mapped value differs at the middle by about 59 in G.
    let start_mapped = gamut_map(unclipped);
    let wrong = reference::packed([0, 1, 2].map(|i| (start_mapped[i] + 1.0) / 2.0));
    let center = pixel(&shot, 100, 20);
    assert!(
        (center >> 8 & 255).abs_diff(wrong >> 8 & 255) > 20,
        "the middle is not the mapped end points"
    );
}

#[test]
fn transition_interpolates_in_oklab() {
    // Tier 2: a background transition is a paint of one color at each
    // time; at 50 ms of a 100 ms linear tween it is the Oklab midpoint of
    // red and blue (CSS's default for transitions is not linear, so the
    // tween names its easing). The state is drawn at frame ticks, so the
    // frames are 1 ms apart: at the 60 Hz default the last frame before
    // 50 ms is at 33.3 ms, which is progress 1/3 and not the midpoint.
    let (screen, color) =
        crate::support::mount_with(RenderOptions::from_env().frame_period_ns(1_000_000), || {
            let color = signal(rgb(RED));
            let view = canvas_of(node().test_id("box").style(move || {
                at(0, 0, 100, 100).background(color.get()).transition(
                    TransitionProp::Background,
                    Transition::tween(100, Easing::Linear),
                )
            }));
            (color, view)
        });
    color.set(rgb(BLUE));
    screen.run_until_idle();
    screen.advance(std::time::Duration::from_millis(50));
    let mid = oklab_lerp_at(
        reference::pixel_encoded(RED),
        reference::pixel_encoded(BLUE),
        0.5,
    );
    expect(&screen.screenshot(), 50, 50, mid, 2, "transition midpoint");
    // The end of the transition is the target.
    screen.advance(std::time::Duration::from_millis(100));
    expect(&screen.screenshot(), 50, 50, BLUE, 2, "transition end");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
