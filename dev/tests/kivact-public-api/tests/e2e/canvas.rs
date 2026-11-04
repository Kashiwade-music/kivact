//! Role: Tests for canvas pixels, shaping, compositing, and hit regions.
use super::paint::{BLACK, BLUE, RED, WHITE, expect, pixel};
use kivact::draw::{BlendMode, Filter, LineCap, Shadow, Shape, Stroke, Transform, radial_gradient};
use kivact::draw::{FillRule, ImageOptions, Layer, Path};
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::test::render;
use kivact::view::canvas;

#[test]
fn scaled_canvas_svg_raster_matches_an_explicitly_sized_image() {
    static SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path d="M0 0L10 10H0Z"/></svg>"#;
    let asset = kivact::view::Asset::from(SVG);
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
    let source = kivact::view::Asset::from(IMAGE);
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
                .svg_arc_to((15, 15), 0, false, true, Point::new(40., 50.))
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

#[test]
fn path_contains_matches_fill_rule() {
    let path = Path::new()
        .rect(Rect::new(5., 5., 60., 60.))
        .rect(Rect::new(20., 20., 30., 30.));
    let screen = render(move || {
        let path = path.clone();
        canvas(move |d, _| {
            for (offset, rule) in [(0., FillRule::NonZero), (80., FillRule::EvenOdd)] {
                let path = path
                    .transformed(Transform::new().translate(offset, 0))
                    .with_fill_rule(rule);
                assert!(path.contains(Point::new(offset + 10., 10.), rule));
                assert_eq!(
                    path.contains(Point::new(offset + 30., 30.), rule),
                    rule == FillRule::NonZero
                );
                assert!(!path.contains(Point::new(offset + 70., 30.), rule));
                d.fill_path(&path, Color::BLACK);
            }
        })
        .style(style().size(160, 80).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    assert_ne!(pixel(&shot, 30, 30), WHITE);
    assert_eq!(pixel(&shot, 110, 30), WHITE);
    assert_ne!(pixel(&shot, 90, 10), WHITE);
    assert!(screen.errors().is_empty());
}

#[test]
fn canvas_arc_to_rounds_a_corner() {
    let screen = render(|| {
        canvas(|d, _| {
            let path = Path::new()
                .move_to(Point::new(10., 10.))
                .arc_to(Point::new(60., 10.), Point::new(60., 60.), 20)
                .line_to(Point::new(60., 60.));
            let stroke = Stroke::new(6);
            assert!(path.stroke_contains(Point::new(30., 10.), &stroke));
            assert!(path.stroke_contains(Point::new(60., 45.), &stroke));
            assert!(!path.stroke_contains(Point::new(60., 10.), &stroke));
            d.stroke_path(&path, &stroke, Color::BLACK);
        })
        .style(style().size(80, 80).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    assert_ne!(pixel(&shot, 30, 10), WHITE);
    assert_ne!(pixel(&shot, 60, 45), WHITE);
    assert_eq!(pixel(&shot, 60, 10), WHITE);
    assert!(screen.errors().is_empty());
}

#[test]
fn parse_svg_draws_the_same_path() {
    let parsed = Path::parse_svg("M10 10h40v40H10z m5 5 l10 0 0 10 -10 0z").expect("valid SVG");
    assert!(Path::parse_svg("M10 10 C20").is_err());
    for data in [
        "M0 0 Q10 20 20 0 T40 0 C45 0 45 10 50 10 S60 20 65 10 A5 5 0 0 1 70 15Z",
        "m0 0 q10 20 20 0 t20 0 c5 0 5 10 10 10 s10 10 15 0 a5 5 0 0 1 5 5z",
    ] {
        assert!(Path::parse_svg(data).is_ok());
    }
    let reference = Path::new()
        .rect(Rect::new(10., 10., 40., 40.))
        .rect(Rect::new(15., 15., 10., 10.));
    let screen = render(move || {
        let parsed = parsed.clone();
        let reference = reference.clone();
        canvas(move |d, _| {
            d.fill_path(&parsed, Color::BLACK);
            d.fill_path(
                &reference.transformed(Transform::new().translate(70, 0)),
                Color::BLACK,
            );
        })
        .style(style().size(140, 70).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    for (x, y, inside) in [
        (5, 5, false),
        (30, 30, true),
        (55, 30, false),
        (20, 20, true),
    ] {
        assert_eq!(pixel(&shot, x, y) != WHITE, inside);
        assert_eq!(pixel(&shot, x + 70, y) != WHITE, inside);
    }
    assert!(screen.errors().is_empty());
}
