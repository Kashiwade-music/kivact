//! Role: Tests for CSS effects, filters, and compositing.
//!
//! Element effects as CSS composites an element: group opacity, filters
//! (color functions over the subtree, blur, drop shadow), backdrop
//! filters, blend modes and masks; state styles and paint transitions of
//! them; theme tokens in every color position. Checked on the frame's
//! pixels (scale 1).

use std::time::Duration;

use kivact::control::ImageFit;
use kivact::draw::{BlendMode, Filter, Shadow, linear_gradient};
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::style::{Align, Color, pct, rgba};
use kivact::test::render;
use kivact::view::provide_theme;

use crate::paint::{BLACK, BLUE, RED, WHITE, at, canvas_of, distance, expect, pixel};
use crate::support::mount;

static RED_SQUARE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12"><rect width="12" height="12" fill="#ff0000"/></svg>"##;

#[test]
fn opacity_fades_a_subtree_as_one_group() {
    let screen = render(|| {
        canvas_of((
            node()
                .child(node().style(at(0, 0, 40, 40).background(rgb(BLACK))))
                .child(node().style(at(20, 0, 40, 40).background(rgb(BLACK))))
                .style(at(0, 0, 60, 40).opacity(0.5)),
            // A lone background fades by its alpha (no layer).
            node().style(at(100, 0, 40, 40).background(rgb(BLACK)).opacity(0.5)),
            // A border over a background fades as one too.
            node().style(
                at(200, 0, 40, 40)
                    .background(rgb(BLUE))
                    .border(4, rgb(RED))
                    .opacity(0.5),
            ),
        ))
    });
    let shot = screen.screenshot();
    let alone = pixel(&shot, 10, 20);
    let overlap = pixel(&shot, 30, 20);
    assert_eq!(overlap, alone, "overlapping children are not darker");
    assert!(distance(alone, 0x808080) <= 1, "half black: {alone:06x}");
    expect(&shot, 120, 20, alone, 1, "a lone faded background");
    expect(
        &shot,
        201,
        20,
        0xff8080,
        1,
        "the border, not the background beneath it",
    );
}

#[test]
fn color_filters_recolor_the_whole_subtree() {
    let screen = render(|| {
        canvas_of((
            column((
                node().style(style().size(40, 20).background(rgb(RED))),
                text("Gray").style(style().color(rgb(RED)).font_size(24.0)),
                image(RED_SQUARE)
                    .fit(ImageFit::Fill)
                    .style(style().size(24, 24)),
            ))
            .test_id("gray")
            .style(
                at(0, 0, 200, 100)
                    .align_items(Align::Start)
                    .filter(Filter::new().grayscale(1)),
            ),
            node().style(
                at(300, 0, 40, 40)
                    .background(Color::WHITE)
                    .border(4, rgb(BLUE))
                    .filter(Filter::new().brightness(0.5)),
            ),
            // In order: invert, then brightness.
            node().style(
                at(400, 0, 40, 40)
                    .background(rgb(BLACK))
                    .filter(Filter::new().invert(1).brightness(0.5)),
            ),
        ))
    });
    let shot = screen.screenshot();
    // CSS grayscale: 0.2126 R + 0.7152 G + 0.0722 B.
    expect(&shot, 20, 10, 0x363636, 1, "the red box, gray");
    let gray = screen.get_by_test_id("gray").screenshot();
    let colored = gray.pixels.iter().filter(|p| {
        let (r, g, b) = ((*p >> 16) & 0xff, (*p >> 8) & 0xff, *p & 0xff);
        r.abs_diff(g) > 2 || g.abs_diff(b) > 2
    });
    assert_eq!(colored.count(), 0, "nothing in the subtree keeps a color");
    let dark = gray.pixels.iter().filter(|p| (**p & 0xff) < 0x40).count();
    assert!(
        dark > 40,
        "the text and the image are drawn, gray ({dark} dark pixels)"
    );
    expect(&shot, 320, 20, 0x808080, 1, "half as bright");
    expect(&shot, 301, 20, 0x000080, 1, "the border too");
    expect(&shot, 420, 20, 0x808080, 1, "inverted, then darkened");
}

#[test]
fn blur_and_drop_shadow_filter_the_subtree_pixels() {
    let screen = render(|| {
        canvas_of((
            node()
                .child(node().style(at(40, 40, 20, 20).background(rgb(BLACK))))
                .style(
                    at(0, 0, 100, 100)
                        .background(Color::WHITE)
                        .filter(Filter::new().blur(4)),
                ),
            node().style(
                at(200, 0, 40, 40)
                    .radius(pct(50))
                    .background(rgb(BLUE))
                    .filter(Filter::new().drop_shadow(10, 0, 0, rgb(RED))),
            ),
        ))
    });
    let shot = screen.screenshot();
    let edge = pixel(&shot, 38, 50) & 0xff;
    assert!(
        (30..225).contains(&edge),
        "blurred across the edge: gray {edge}"
    );
    assert!(pixel(&shot, 50, 50) & 0xff < 80, "dark in the middle");
    expect(&shot, 10, 50, WHITE, 0, "far from the edge");
    expect(&shot, 247, 20, RED, 0, "the shadow beside the circle");
    expect(&shot, 220, 20, BLUE, 0, "the circle over its shadow");
    expect(
        &shot,
        247,
        3,
        WHITE,
        0,
        "the shadow is round, as the circle",
    );
}

#[test]
fn a_backdrop_filter_blurs_what_lies_behind() {
    let screen = render(|| {
        canvas_of((
            node().style(
                at(0, 0, 400, 300).background(
                    linear_gradient(90)
                        .repeating()
                        .stop(rgb(BLACK), 0)
                        .stop(rgb(BLACK), 2)
                        .stop(rgb(WHITE), 2)
                        .stop(rgb(WHITE), 4),
                ),
            ),
            node().style(
                at(100, 100, 100, 100)
                    .radius(12)
                    .backdrop_blur(6)
                    .background(rgba(0xffffff, 0.2)),
            ),
            // Glass that also saturates: grayscale behind it.
            node()
                .style(at(250, 100, 100, 100).backdrop_filter(Filter::new().blur(6).grayscale(1))),
        ))
    });
    let shot = screen.screenshot();
    let inside = pixel(&shot, 150, 150) & 0xff;
    assert!(
        (110..190).contains(&inside),
        "the stripes behind the glass blur to gray (lightened): {inside}"
    );
    let crisp = [pixel(&shot, 50, 150) & 0xff, pixel(&shot, 52, 150) & 0xff];
    assert!(
        crisp.contains(&0) && crisp.contains(&255),
        "crisp stripes beside it: {crisp:?}"
    );
    expect(
        &shot,
        101,
        101,
        BLACK,
        0,
        "a crisp stripe outside the glass's corner",
    );
    expect(&shot, 102, 101, WHITE, 0, "and the next one");
    let gray = pixel(&shot, 300, 150);
    assert!(
        distance(gray, 0x808080) < 40,
        "blurred behind the second glass: {gray:06x}"
    );
}

#[test]
fn blend_modes_mix_with_what_is_beneath() {
    let screen = render(|| {
        canvas_of((
            node()
                .child(
                    node().style(
                        at(0, 0, 40, 40)
                            .background(Color::WHITE)
                            .mix_blend_mode(BlendMode::Difference),
                    ),
                )
                .style(at(0, 0, 80, 40).background(rgb(RED))),
            node()
                .child(
                    node().style(
                        at(0, 0, 40, 40)
                            .background(rgb(0x808080))
                            .mix_blend_mode(BlendMode::Multiply),
                    ),
                )
                .style(at(100, 0, 80, 40).background(rgb(BLUE))),
        ))
    });
    let shot = screen.screenshot();
    expect(&shot, 20, 20, 0x00ffff, 0, "white minus red");
    expect(&shot, 60, 20, RED, 0, "beside it");
    expect(&shot, 120, 20, 0x000080, 1, "gray times blue");
}

#[test]
fn a_gradient_mask_fades_the_element_and_its_subtree() {
    let screen = render(|| {
        canvas_of(
            node()
                .child(node().style(at(0, 50, 100, 50).background(rgb(BLUE))))
                .style(
                    at(0, 0, 100, 100)
                        .background(rgb(RED))
                        .mask(linear_gradient(180).colors([Color::BLACK, Color::TRANSPARENT])),
                ),
        )
    });
    let shot = screen.screenshot();
    expect(&shot, 50, 0, RED, 3, "opaque at the top");
    expect(&shot, 50, 25, 0xff4040, 6, "a quarter down");
    expect(&shot, 50, 75, 0xc0c0ff, 6, "the child, three quarters down");
    expect(&shot, 50, 99, WHITE, 4, "transparent at the bottom");
}

#[test]
fn state_styles_change_filters_and_gradients() {
    let screen = render(|| {
        canvas_of(
            node().test_id("card").style(
                at(0, 0, 100, 40)
                    .background(linear_gradient(90).colors([rgb(RED), rgb(RED)]))
                    .hover(
                        style()
                            .background(linear_gradient(90).colors([rgb(BLUE), rgb(BLUE)]))
                            .filter(Filter::new().brightness(0.5)),
                    ),
            ),
        )
    });
    expect(&screen.screenshot(), 50, 20, RED, 0, "at rest");
    screen.get_by_test_id("card").hover();
    expect(
        &screen.screenshot(),
        50,
        20,
        0x000080,
        1,
        "hovered: darker blue",
    );
}

#[test]
fn paint_transitions_animate_filters_and_shadows() {
    let (screen, dark) = mount(|| {
        let dark = signal(false);
        let card = node().style(move || {
            let s = at(100, 100, 100, 40).background(Color::WHITE).transition(
                TransitionProp::Paint,
                Transition::tween(100, Easing::Linear),
            );
            if dark.get() {
                s.filter(Filter::new().brightness(0))
                    .shadow(Shadow::new(0, 0, 0, rgb(BLACK)).spread(10))
            } else {
                s
            }
        });
        (dark, canvas_of(card))
    });
    dark.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(50));
    let shot = screen.screenshot();
    let half = pixel(&shot, 150, 120) & 0xff;
    assert!((40..215).contains(&half), "on its way to black: {half}");
    let ring = pixel(&shot, 150, 97) & 0xff;
    assert!(ring < 200, "the shadow grows in: {ring}");
    expect(&shot, 150, 92, WHITE, 0, "not yet 10 px out");
    screen.advance(Duration::from_millis(100));
    let shot = screen.screenshot();
    expect(&shot, 150, 120, BLACK, 0, "done");
    expect(&shot, 150, 92, BLACK, 0, "the whole shadow");
}

#[test]
fn theme_tokens_in_every_color_position_follow_a_theme_switch() {
    let (screen, theme) = mount(|| {
        let theme = signal(Theme::light());
        let view = provide_theme(theme, || {
            canvas_of(
                node().style(
                    at(100, 100, 100, 40)
                        .background(
                            linear_gradient(90).colors([ColorToken::Accent, ColorToken::Accent]),
                        )
                        .border_top(4, ColorToken::Danger)
                        .shadow(Shadow::new(0, 10, 0, ColorToken::Success)),
                ),
            )
        });
        (theme, view)
    });
    let shot = screen.screenshot();
    expect(&shot, 150, 120, 0x2563eb, 1, "light accent");
    expect(&shot, 150, 101, 0xdc2626, 1, "light danger border");
    expect(&shot, 150, 145, 0x16a34a, 1, "light success shadow");
    let creates = screen.stats().creates;
    theme.set(Theme::dark());
    let shot = screen.screenshot();
    expect(&shot, 150, 120, 0x3b82f6, 1, "dark accent");
    expect(&shot, 150, 101, 0xf87171, 1, "dark danger border");
    expect(&shot, 150, 145, 0x4ade80, 1, "dark success shadow");
    assert_eq!(screen.stats().creates, creates, "no remount");
}

/// `Hover`, `Pressed` and `Raised` are theme tokens like the rest: a
/// custom palette sets them, and a theme switch repaints them. A translucent
/// token composites over the surface behind it.
#[test]
fn state_color_tokens_follow_the_theme() {
    let state_theme = || {
        let t = Theme::light();
        let colors = kivact::style::theme::Palette {
            hover: rgb(0x102030),
            pressed: rgb(0x203040),
            raised: rgb(0x304050),
            ..t.colors().clone()
        };
        t.with_colors(colors)
    };
    let (screen, theme) = mount(move || {
        let theme = signal(state_theme());
        let view = provide_theme(theme, || {
            canvas_of((
                node().style(at(0, 0, 50, 50).background(ColorToken::Hover)),
                node().style(at(100, 0, 50, 50).background(ColorToken::Pressed)),
                node().style(at(200, 0, 50, 50).background(ColorToken::Raised)),
                node().style(
                    at(300, 0, 50, 50).background(Color::token(ColorToken::Hover).with_alpha(0.5)),
                ),
            ))
        });
        (theme, view)
    });
    let shot = screen.screenshot();
    expect(&shot, 25, 25, 0x102030, 1, "custom hover");
    expect(&shot, 125, 25, 0x203040, 1, "custom pressed");
    expect(&shot, 225, 25, 0x304050, 1, "custom raised");
    // Half of 0x102030 over the white canvas.
    expect(&shot, 325, 25, 0x889098, 2, "half hover over white");
    let creates = screen.stats().creates;
    theme.set(Theme::dark());
    let shot = screen.screenshot();
    let dark = Theme::dark().colors().clone();
    let over_white = |c: Color| {
        let [r, g, b, a] = c.components();
        let ch = |v: f32| ((v * a + (1.0 - a)) * 255.0).round() as u32;
        (ch(r) << 16) | (ch(g) << 8) | ch(b)
    };
    expect(
        &shot,
        25,
        25,
        over_white(dark.hover),
        2,
        "dark hover over white",
    );
    expect(
        &shot,
        125,
        25,
        over_white(dark.pressed),
        2,
        "dark pressed over white",
    );
    expect(&shot, 225, 25, 0x1f2937, 1, "dark raised");
    assert_eq!(screen.stats().creates, creates, "no remount");
}

#[test]
fn filters_reaching_into_the_window_are_drawn_from_unseen_boxes() {
    let screen = render(|| {
        canvas_of((
            // A blurred parent whose child lies below the window: the blur
            // reaches into it.
            node()
                .child(node().style(at(0, 106, 200, 40).background(rgb(BLACK))))
                .style(at(0, 500, 200, 100).filter(Filter::new().blur(10))),
            // A drop shadow cast up into the window by a box below it.
            node().style(
                at(300, 610, 100, 40)
                    .background(rgb(BLUE))
                    .filter(Filter::new().drop_shadow(0, -20, 0, rgb(RED))),
            ),
            // A glass at the bottom edge sees the window only: what lies
            // below it is not blurred in, the window is mirrored there.
            node().style(at(500, 606, 200, 40).background(rgb(BLACK))),
            node().style(at(500, 560, 200, 40).backdrop_blur(10)),
        ))
    });
    let shot = screen.screenshot();
    let blurred = pixel(&shot, 100, 599) & 0xff;
    assert!(blurred < 235, "the blur reaches up: gray {blurred}");
    expect(&shot, 350, 595, RED, 0, "the drop shadow reaches up");
    let glass = pixel(&shot, 600, 599) & 0xff;
    assert!(
        glass >= 250,
        "the glass reads the window mirrored at its edge, not what lies below: gray {glass}"
    );
}
