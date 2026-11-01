//! Role: Tests for canvas pixels, shaping, compositing, and hit regions.
use super::paint::{BLACK, BLUE, RED, WHITE, expect, pixel};
use kivact::control::canvas;
use kivact::draw::{BlendMode, Filter, LineCap, Shadow, Shape, Stroke, Transform, radial_gradient};
use kivact::draw::{FillRule, ImageOptions, Layer, Path};
use kivact::draw::{Point, Rect};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::test::render;

#[test]
fn scaled_canvas_svg_raster_matches_an_explicitly_sized_image() {
    static SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path d="M0 0L10 10H0Z"/></svg>"#;
    let asset = kivact::Asset::from(SVG);
    let screen = render(move || {
        let reference = asset.clone();
        row((
            canvas(move |d, _| {
                d.image_with(
                    &reference,
                    Rect::new(0., 0., 40., 40.),
                    ImageOptions {
                        pixelated: true,
                        ..Default::default()
                    },
                );
            })
            .style(style().size(40, 40)),
            canvas(move |d, _| {
                d.layer(Layer::new().transform(Transform::new().scale(4)), |d| {
                    d.image_with(
                        &asset,
                        Rect::new(0., 0., 10., 10.),
                        ImageOptions {
                            pixelated: true,
                            ..Default::default()
                        },
                    );
                });
            })
            .style(style().size(40, 40)),
        ))
        .style(style().gap(0).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    for y in 0..40 {
        for x in 0..40 {
            assert_eq!(
                pixel(&shot, x, y),
                pixel(&shot, x + 40, y),
                "SVG raster at {x},{y}"
            );
        }
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn asset_source_rect_and_path_helpers_are_drawn() {
    static IMAGE:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="10" height="10" fill="#ff0000"/><rect x="10" width="10" height="10" fill="#0000ff"/></svg>"##;
    let source = kivact::Asset::from(IMAGE);
    let screen = render(move || {
        canvas(move |d, _| {
            d.image_with(
                &source,
                Rect::new(0., 0., 40., 20.),
                ImageOptions {
                    source: Some(Rect::new(10., 0., 10., 10.)),
                    pixelated: true,
                    ..Default::default()
                },
            );
            let ellipse = Path::new()
                .ellipse(Point::new(65., 15.), (15, 10), 0, (0, 360), false)
                .close();
            d.fill_path(&ellipse, rgb(RED));
            let rounded = Path::new().rounded_rect(
                Rect::new(90., 0., 40., 30.),
                [10.into(), (15, 5).into(), 0.into(), 5.into()],
            );
            d.fill_path(&rounded, Color::BLACK);
            let svg_arc = Path::new()
                .move_to(Point::new(10., 50.))
                .arc_to((15, 15), 0, false, true, Point::new(40., 50.))
                .line_to(Point::new(10., 50.))
                .close();
            d.fill_path(&svg_arc, rgb(RED));
        })
        .style(style().size(140, 80).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 10, 10, BLUE, 0, "asset crop uses natural pixels");
    expect(&shot, 65, 15, RED, 0, "ellipse center");
    expect(&shot, 51, 6, WHITE, 0, "ellipse corner");
    expect(&shot, 90, 0, WHITE, 0, "rounded path retains corners");
    expect(&shot, 110, 15, BLACK, 0, "rounded path center");
    assert!(
        (33..50).any(|y| pixel(&shot, 25, y) == RED),
        "SVG arc encloses its contour"
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn circular_clip_radial_path_and_even_odd_holes() {
    let screen = render(|| {
        canvas(|d, _| {
            d.layer(
                Layer::new()
                    .area(Rect::new(0., 0., 40., 40.))
                    .clip(&Shape::circle(20)),
                |d| {
                    d.fill_rect(Rect::new(0., 0., 40., 40.), rgb(RED));
                },
            );
            let circle = Path::new()
                .arc(Point::new(70., 20.), 18, 0, 360, false)
                .close();
            d.fill_path(&circle, radial_gradient().colors([rgb(RED), rgb(BLUE)]));
            let hole = Path::new()
                .rect(Rect::new(100., 0., 40., 40.))
                .rect(Rect::new(110., 10., 20., 20.))
                .with_fill_rule(FillRule::EvenOdd);
            d.fill_path(&hole, Color::BLACK);
        })
        .style(style().size(150, 60).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 1, 1, WHITE, 0, "circle clips corners");
    expect(&shot, 20, 20, RED, 0, "circle center");
    let center = pixel(&shot, 70, 20);
    let edge = pixel(&shot, 85, 20);
    assert!((center >> 16 & 255) > (edge >> 16 & 255));
    assert!((edge & 255) > (center & 255));
    expect(&shot, 105, 20, BLACK, 0, "path outer contour");
    expect(&shot, 120, 20, WHITE, 0, "even odd hole");
}

#[test]
fn dashed_round_caps_arcs_glow_and_additive_layers() {
    let screen = render(|| {
        canvas(|d, _| {
            let line = Path::new()
                .move_to(Point::new(10., 10.))
                .line_to(Point::new(80., 10.));
            d.stroke_path(
                &line,
                &Stroke::new(4).cap(LineCap::Round).dash([8., 8.]),
                Color::BLACK,
            );
            d.layer(Layer::new().shadow(Shadow::new(0, 0, 8, rgb(RED))), |d| {
                d.stroke_path(
                    &Path::new().arc(Point::new(30., 50.), 15, 0, 180, false),
                    &Stroke::new(3).cap(LineCap::Round),
                    rgb(RED),
                );
            });
            d.fill_rect(Rect::new(90., 0., 30., 30.), rgb(RED));
            d.layer(Layer::new().blend(BlendMode::PlusLighter), |d| {
                d.fill_rect(Rect::new(100., 10., 30., 30.), rgb(BLUE));
            });
        })
        .style(style().size(160, 100).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 9, 10, BLACK, 1, "round cap extends");
    expect(&shot, 22, 10, WHITE, 1, "dash gap");
    expect(&shot, 110, 20, 0xff00ff, 1, "additive overlap");
    assert_ne!(pixel(&shot, 30, 69), WHITE, "glow reaches beyond arc");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_layer_applies_group_opacity_filter_and_mask_once() {
    let screen = render(|| {
        canvas(|d, _| {
            d.layer(
                Layer::new()
                    .opacity(0.5)
                    .filter(Filter::new().invert(1))
                    .mask(Color::WHITE),
                |d| {
                    d.fill_rect(Rect::new(0., 0., 30., 30.), rgb(RED));
                    d.fill_rect(Rect::new(10., 0., 30., 30.), rgb(RED));
                },
            );
        })
        .style(style().size(60, 50).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 5, 10, 0x80ffff, 1, "inverted group faded once");
    expect(&shot, 20, 10, 0x80ffff, 1, "overlap keeps group alpha");
    assert!(screen.errors().is_empty());
}
