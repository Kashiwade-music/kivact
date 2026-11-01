//! Role: Tests for canvas gradient drawing and placement.
//!
//! Gradients placed by coordinates (`CanvasGradient`: HTML Canvas
//! `createLinearGradient`, `createRadialGradient`, `createConicGradient`),
//! checked on the frame's pixels.
use super::paint::{BLUE, RED, WHITE, expect, pixel};
use kivact::control::canvas;
use kivact::draw::{CanvasGradient, GradientExtend, Transform};
use kivact::draw::{Layer, Path};
use kivact::draw::{Point, Rect};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::test::render;

fn p(x: f32, y: f32) -> Point {
    Point::new(x, y)
}

fn red_to_blue(g: CanvasGradient) -> CanvasGradient {
    g.stop(0.0, rgb(RED)).stop(1.0, rgb(BLUE))
}

fn black_to_white(g: CanvasGradient) -> CanvasGradient {
    g.stop(0.0, Color::BLACK).stop(1.0, Color::WHITE)
}

const PURPLE: u32 = 0x800080;

#[test]
fn a_linear_gradient_is_placed_by_its_points_not_by_the_shape() {
    let screen = render(|| {
        canvas(|d, _| {
            let g = red_to_blue(CanvasGradient::linear(p(0., 0.), p(100., 0.)));
            d.fill_rect(Rect::new(0., 0., 200., 20.), g.clone());
            // Another shape with the same gradient shows the same colors.
            d.fill_rect(Rect::new(50., 30., 100., 20.), g.clone());
            // And a path.
            d.fill_path(&Path::new().rect(Rect::new(0., 60., 200., 20.)), g);
        })
        .style(style().size(200, 80).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 1, 10, RED, 8, "offset 0 is the first point");
    expect(&shot, 50, 10, PURPLE, 8, "half way");
    expect(&shot, 98, 10, BLUE, 8, "offset 1 is the second point");
    expect(&shot, 150, 10, BLUE, 0, "padded beyond it");
    expect(
        &shot,
        50,
        40,
        PURPLE,
        8,
        "not moved to the shape's own left edge",
    );
    expect(
        &shot,
        148,
        40,
        BLUE,
        8,
        "the shape ends where the gradient has",
    );
    expect(&shot, 25, 70, 0xbf0040, 8, "a path, a quarter in");
    assert!(screen.errors().is_empty());
}

#[test]
fn pushed_transforms_apply_to_a_gradient_when_it_is_drawn() {
    let screen = render(|| {
        canvas(|d, _| {
            let g = red_to_blue(CanvasGradient::linear(p(0., 0.), p(50., 0.)));
            d.layer(Layer::new().transform(Transform::new().scale(2)), |d| {
                d.fill_rect(Rect::new(0., 0., 100., 10.), g.clone());
            });
            d.layer(
                Layer::new().transform(Transform::new().translate(0, 30)),
                |d| {
                    d.fill_rect(Rect::new(0., 0., 100., 10.), g);
                },
            );
        })
        .style(style().size(200, 40).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(
        &shot,
        50,
        10,
        PURPLE,
        8,
        "scaled: the gradient is 100 px long",
    );
    expect(&shot, 2, 10, RED, 12, "scaled: the start");
    expect(&shot, 25, 35, PURPLE, 8, "moved: half of 50 px");
    expect(&shot, 75, 35, BLUE, 0, "moved: padded");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_radial_gradient_runs_between_two_circles() {
    let screen = render(|| {
        canvas(|d, _| {
            // Concentric: red inside 20, blue outside 40.
            let rings = red_to_blue(CanvasGradient::radial(p(50., 50.), 20., p(50., 50.), 40.));
            d.fill_rect(Rect::new(0., 0., 100., 100.), rings);
            // From a point at the left to a circle: offset 1 is the circle.
            let cone = red_to_blue(CanvasGradient::radial(p(130., 50.), 0., p(150., 50.), 40.));
            d.fill_rect(Rect::new(100., 0., 100., 100.), cone);
        })
        .style(style().size(200, 100).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 50, 50, RED, 0, "inside the first circle");
    expect(
        &shot,
        65,
        50,
        RED,
        8,
        "inside the first circle, off the center",
    );
    expect(&shot, 80, 50, PURPLE, 8, "half way between the circles");
    expect(&shot, 95, 50, BLUE, 0, "outside the second circle");
    expect(&shot, 130, 50, RED, 8, "the point");
    // On the circle of offset 2/3: center 143.3, radius 26.7.
    expect(&shot, 170, 50, 0x5500aa, 12, "two thirds along the cone");
    expect(&shot, 189, 50, BLUE, 12, "the second circle's far side");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_conic_gradient_turns_clockwise_from_its_start_angle() {
    let screen = render(|| {
        canvas(|d, _| {
            d.fill_rect(
                Rect::new(0., 0., 100., 100.),
                red_to_blue(CanvasGradient::conic(0., p(50., 50.))),
            );
            // Starting at the bottom (90 degrees): its start is below.
            d.fill_rect(
                Rect::new(100., 0., 100., 100.),
                red_to_blue(CanvasGradient::conic(90., p(150., 50.))),
            );
        })
        .style(style().size(200, 100).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 90, 53, RED, 16, "to the right, the start");
    expect(&shot, 50, 90, 0xbf0040, 12, "a quarter turn: below");
    expect(&shot, 10, 50, PURPLE, 12, "half a turn: left");
    expect(&shot, 50, 10, 0x4000bf, 12, "three quarters: above");
    expect(&shot, 148, 90, RED, 16, "started at 90 degrees: below");
    expect(
        &shot,
        110,
        52,
        0xbf0040,
        12,
        "a quarter turn from there: left",
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn repeat_and_reflect_continue_the_stops_beyond_the_points() {
    let screen = render(|| {
        canvas(|d, _| {
            let line = || CanvasGradient::linear(p(0., 0.), p(40., 0.));
            d.fill_rect(
                Rect::new(0., 0., 120., 10.),
                black_to_white(line()).extend(GradientExtend::Reflect),
            );
            d.fill_rect(
                Rect::new(0., 20., 120., 10.),
                black_to_white(line()).extend(GradientExtend::Repeat),
            );
            d.fill_rect(Rect::new(0., 40., 120., 10.), black_to_white(line()));
        })
        .style(style().size(120, 50).background(rgb(BLUE)))
    });
    let shot = screen.screenshot();
    let grey = |level: u32| level * 0x010101;
    expect(&shot, 10, 5, grey(0x40), 10, "reflect: a quarter");
    expect(
        &shot,
        50,
        5,
        grey(0xc0),
        10,
        "reflect: coming back, offset 1.25",
    );
    expect(&shot, 70, 5, grey(0x40), 10, "reflect: offset 1.75");
    expect(
        &shot,
        90,
        5,
        grey(0x40),
        10,
        "reflect: offset 2.25 is 0.25 again",
    );
    expect(&shot, 50, 25, grey(0x40), 10, "repeat: offset 1.25 is 0.25");
    expect(&shot, 70, 25, grey(0xc0), 10, "repeat: offset 1.75 is 0.75");
    expect(&shot, 100, 45, WHITE, 0, "pad: the last color goes on");
    assert!(screen.errors().is_empty());
}

#[test]
fn two_stops_at_one_offset_make_a_hard_edge_in_the_order_added() {
    let screen = render(|| {
        canvas(|d, _| {
            let g = CanvasGradient::linear(p(0., 0.), p(100., 0.))
                .stop(0.0, rgb(RED))
                .stop(0.5, rgb(RED))
                .stop(0.5, rgb(BLUE))
                .stop(1.0, rgb(BLUE));
            d.fill_rect(Rect::new(0., 0., 100., 10.), g);
            // Added out of order: Canvas orders the stops by offset.
            let unordered = CanvasGradient::linear(p(0., 0.), p(100., 0.))
                .stop(1.0, rgb(BLUE))
                .stop(0.0, rgb(RED))
                .stop(0.5, rgb(RED))
                .stop(0.5, rgb(BLUE));
            d.fill_rect(Rect::new(0., 20., 100., 10.), unordered);
        })
        .style(style().size(100, 30).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    for y in [5, 25] {
        expect(&shot, 48, y, RED, 0, "before the edge");
        expect(&shot, 52, y, BLUE, 0, "after the edge");
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn a_theme_token_stop_is_the_themes_color() {
    let screen = render(|| {
        canvas(|d, _| {
            let g = CanvasGradient::linear(p(0., 0.), p(100., 0.))
                .stop(0.0, ColorToken::Accent)
                .stop(1.0, ColorToken::Accent);
            d.fill_rect(Rect::new(0., 0., 100., 10.), g);
            d.fill_rect(Rect::new(0., 20., 100., 10.), ColorToken::Accent);
        })
        .style(style().size(100, 30).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    let accent = pixel(&shot, 50, 25);
    assert_ne!(accent, WHITE, "the accent is drawn");
    expect(
        &shot,
        50,
        5,
        accent,
        2,
        "the gradient's token resolves to the same color",
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn a_gradient_without_valid_stops_paints_nothing() {
    let screen = render(|| {
        canvas(|d, _| {
            let line = || CanvasGradient::linear(p(0., 0.), p(100., 0.));
            d.fill_rect(Rect::new(0., 0., 100., 10.), line());
            d.fill_rect(Rect::new(0., 20., 100., 10.), line().stop(1.5, rgb(RED)));
            d.fill_rect(
                Rect::new(0., 40., 100., 10.),
                line().stop(f32::NAN, rgb(RED)),
            );
        })
        .style(style().size(100, 50).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    for y in [5, 25, 45] {
        expect(&shot, 50, y, WHITE, 0, "nothing is painted");
    }
    assert!(screen.errors().is_empty());
}
