//! Role: Tests for CSS color spaces, color parsing and gradient interpolation.
//!
//! Colors as the public API paints them. A pixel is compared with a value
//! the test computes from the formulas of CSS Color 4 (written out below,
//! in `reference`), or with a known number of the design. The tier of each
//! comparison (§3.3) is named where it is made: tier 2 is ±2/255 per channel
//! (the compositor), tier 3 is the raster tolerance of
//! `Screenshot::raster_difference` (Vello's gradient lookup table).

use super::paint::{BLACK, BLUE, RED, WHITE, at, canvas_of, expect, pixel};
use kivact::draw::linear_gradient;
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::{Color, ColorSpace, TransitionProp, lab, oklab, oklch, pct, rgb};
use kivact::test::{RenderOptions, Screen, render, render_with};

/// The formulas of CSS Color 4 that the expected values are computed with.
/// The matrices are §5.4.1's (CSS Color 4 §18); nothing here calls the
/// implementation under test.
mod reference {
    pub(super) const SRGB_TO_XYZ: [[f64; 3]; 3] = [
        [0.41239079926595934, 0.357584339383878, 0.1804807884018343],
        [0.21263900587151027, 0.715168678767756, 0.07219231536073371],
        [0.01933081871559182, 0.11919477979462598, 0.9505321522496607],
    ];
    pub(super) const XYZ_TO_SRGB: [[f64; 3]; 3] = [
        [3.2409699419045226, -1.537383177570094, -0.4986107602930034],
        [-0.9692436362808796, 1.8759675015077202, 0.04155505740717559],
        [
            0.05563007969699366,
            -0.20397695888897652,
            1.0569715142428786,
        ],
    ];
    pub(super) const P3_TO_XYZ: [[f64; 3]; 3] = [
        [0.4865709486482162, 0.26566769316909306, 0.1982172852343625],
        [0.2289745640697488, 0.6917385218365064, 0.079286914093745],
        [0.0, 0.04511338185890264, 1.043944368900976],
    ];
    pub(super) const XYZ_TO_LMS: [[f64; 3]; 3] = [
        [0.8190224379967030, 0.3619062600528904, -0.1288737815209879],
        [0.0329836539323885, 0.9292868615863434, 0.0361446663506424],
        [0.0481771893596242, 0.2642395317527308, 0.6335478284694309],
    ];
    pub(super) const CBRT_LMS_TO_OKLAB: [[f64; 3]; 3] = [
        [0.2104542683093140, 0.7936177747023054, -0.0040720430116193],
        [1.9779985324311684, -2.4285922420485799, 0.4505937096174110],
        [0.0259040424655478, 0.7827717124575296, -0.8086757549230774],
    ];
    pub(super) const OKLAB_TO_CBRT_LMS: [[f64; 3]; 3] = [
        [1.0, 0.3963377773761749, 0.2158037573099136],
        [1.0, -0.1055613458156586, -0.0638541728258133],
        [1.0, -0.0894841775298119, -1.2914855480194092],
    ];
    pub(super) const LMS_TO_XYZ: [[f64; 3]; 3] = [
        [1.2268798758459243, -0.5578149944602171, 0.2813910456659647],
        [-0.0405757452148008, 1.1122868032803170, -0.0717110580655164],
        [-0.0763729366746601, -0.4214933324022432, 1.5869240198367816],
    ];

    pub(super) type V3 = [f64; 3];

    pub(super) fn mul(m: &[[f64; 3]; 3], v: V3) -> V3 {
        let row = |r: &[f64; 3]| r[0] * v[0] + r[1] * v[1] + r[2] * v[2];
        [row(&m[0]), row(&m[1]), row(&m[2])]
    }

    /// The sRGB transfer function (sign preserving).
    pub(super) fn to_linear(v: f64) -> f64 {
        let a = v.abs();
        let l = if a <= 0.04045 {
            a / 12.92
        } else {
            ((a + 0.055) / 1.055).powf(2.4)
        };
        l.copysign(v)
    }

    /// The inverse of `to_linear`.
    pub(super) fn to_encoded(v: f64) -> f64 {
        let a = v.abs();
        let e = if a <= 0.0031308 {
            12.92 * a
        } else {
            1.055 * a.powf(1.0 / 2.4) - 0.055
        };
        e.copysign(v)
    }

    pub(super) fn xyz_to_oklab(xyz: V3) -> V3 {
        let lms = mul(&XYZ_TO_LMS, xyz).map(f64::cbrt);
        mul(&CBRT_LMS_TO_OKLAB, lms)
    }

    pub(super) fn oklab_to_xyz(lab: V3) -> V3 {
        let lms = mul(&OKLAB_TO_CBRT_LMS, lab).map(|v| v * v * v);
        mul(&LMS_TO_XYZ, lms)
    }

    /// Oklab of sRGB values (encoded, may lie outside 0..1).
    pub(super) fn encoded_to_oklab(enc: V3) -> V3 {
        let xyz = mul(&SRGB_TO_XYZ, enc.map(to_linear));
        xyz_to_oklab(xyz)
    }

    /// Unclipped sRGB (encoded) of an Oklab color.
    pub(super) fn oklab_to_encoded(lab: V3) -> V3 {
        mul(&XYZ_TO_SRGB, oklab_to_xyz(lab)).map(to_encoded)
    }

    /// Oklab of an Oklch color (hue in degrees).
    pub(super) fn oklch_to_oklab(l: f64, c: f64, h: f64) -> V3 {
        let (sin, cos) = h.to_radians().sin_cos();
        [l, c * cos, c * sin]
    }

    pub(super) fn oklab_to_oklch(lab: V3) -> V3 {
        let c = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
        let h = lab[2].atan2(lab[1]).to_degrees().rem_euclid(360.0);
        [lab[0], c, h]
    }

    pub(super) fn delta_e(a: V3, b: V3) -> f64 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    }

    /// The Display P3 color `enc` (encoded) as unclipped sRGB (encoded).
    pub(super) fn p3_to_srgb(enc: V3) -> V3 {
        let xyz = mul(&P3_TO_XYZ, enc.map(to_linear));
        mul(&XYZ_TO_SRGB, xyz).map(to_encoded)
    }

    fn in_srgb(enc: V3) -> bool {
        enc.iter().all(|v| (0.0..=1.0).contains(v))
    }

    fn clip(enc: V3) -> V3 {
        enc.map(|v| v.clamp(0.0, 1.0))
    }

    /// CSS Color 4 §13.2.2 gamut mapping of an unclipped sRGB color
    /// (encoded), as the design's §5.4.4 states it.
    pub(super) fn gamut_map(enc: V3) -> V3 {
        if in_srgb(enc) {
            return enc;
        }
        const JND: f64 = 0.02;
        const EPS: f64 = 0.0001;
        let current = oklab_to_oklch(encoded_to_oklab(enc));
        let (l, c, h) = (current[0], current[1], current[2]);
        if l >= 1.0 {
            return [1.0; 3];
        }
        if l <= 0.0 {
            return [0.0; 3];
        }
        let at_chroma = |chroma: f64| oklch_to_oklab(l, chroma, h);
        let clip_of = |chroma: f64| {
            let lab = at_chroma(chroma);
            (lab, clip(oklab_to_encoded(lab)))
        };
        let (lab0, mut clipped) = clip_of(c);
        if delta_e(encoded_to_oklab(clipped), lab0) < JND {
            return clipped;
        }
        let (mut min, mut max) = (0.0, c);
        let mut min_in_gamut = true;
        while max - min > EPS {
            let chroma = (min + max) / 2.0;
            let lab = at_chroma(chroma);
            if min_in_gamut && in_srgb(oklab_to_encoded(lab)) {
                min = chroma;
                continue;
            }
            clipped = clip(oklab_to_encoded(lab));
            let e = delta_e(encoded_to_oklab(clipped), lab);
            if e < JND {
                if JND - e < EPS {
                    return clipped;
                }
                min_in_gamut = false;
                min = chroma;
            } else {
                max = chroma;
            }
        }
        clipped
    }

    /// A pixel (0xRRGGBB) as sRGB values in 0..1.
    pub(super) fn pixel_encoded(p: u32) -> V3 {
        [(p >> 16) & 255, (p >> 8) & 255, p & 255].map(|c| f64::from(c) / 255.0)
    }

    /// The 8-bit value of an encoded channel, rounded.
    pub(super) fn channel(v: f64) -> u32 {
        (v * 255.0).round().clamp(0.0, 255.0) as u32
    }

    pub(super) fn packed(enc: V3) -> u32 {
        (channel(enc[0]) << 16) | (channel(enc[1]) << 8) | channel(enc[2])
    }
}

use reference::{V3, encoded_to_oklab, gamut_map, oklab_to_encoded};

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
    // Tier 2 (the paint of a color, ±1/255 for the known sRGB values and
    // ±2/255 for the gamut mapped display-p3 value). The forward conversions
    // (sRGB to Oklab, to Lab) and the unclipped sRGB of display-p3 red
    // (1.093066, −0.226742, −0.150135, ±1e-4) are internal values that the
    // public API does not expose; they are checked only through what is
    // painted here.
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
    // mapping of its unclipped sRGB, computed here from the §5.4.1 matrices.
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
    // 0.02°, a ruling of B-18); chroma below the source's.
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
    // The CSS gamut mapping of the same color, as §5.4.4 states it.
    let want = reference::packed(gamut_map(oklab_to_encoded(reference::oklch_to_oklab(
        0.7, 0.4, 30.0,
    ))));
    expect(&screen.screenshot(), 20, 20, want, 2, "gamut mapped value");
}

#[test]
fn wide_gamut_gradient_interpolates_source_colors() {
    // Tier 3: a gradient of Display P3 green to oklch(0.6 0.3 300). The
    // expected value interpolates the two source colors' Oklab values
    // unclipped, then gamut maps the result (§9.6).
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
    // The examples of §11.4.1 parse to the colors they name.
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
    // These are errors (§11.4.1 does not use them).
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
    // Tier 3: the known values of §5.4.1 for the midpoint of red and blue
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
    // Tier 3: the LUT of §9.6 allows a few levels; the hint's midpoint is
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
        assert_eq!(row.len(), width as usize);
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
