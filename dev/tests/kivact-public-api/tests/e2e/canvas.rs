//! Role: Tests for canvas pixels, shaping, compositing, and hit regions.
use super::paint::{BLACK, BLUE, RED, WHITE, expect, pixel};
use kivact::control::canvas;
use kivact::draw::{
    BlendMode, Filter, LineCap, LineJoin, Paint, Shadow, Shape, Stroke, Transform, radial_gradient,
};
use kivact::draw::{FillRule, ImageOptions, Layer, Path, Pixels, TextBaseline, TextStyle};
use kivact::draw::{Point, Rect, RegionId};
use kivact::prelude::*;
use kivact::style::TextAlign;
use kivact::style::{Color, pct};
use kivact::test::render;
use kivact::view::{Handled, PointerEvent};
use std::{cell::Cell, rc::Rc, sync::Arc};

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
fn invalid_raw_pixel_dimensions_report_unsupported_without_losing_the_frame() {
    let screen = render(|| {
        canvas(|d, _| {
            for (width, height) in [(u32::MAX, u32::MAX), (0, 1), (2, 1)] {
                d.pixels(
                    &Pixels::new(width, height, Arc::from([])),
                    Rect::new(0., 0., 20., 20.),
                    ImageOptions::default(),
                );
            }
            d.fill_rect(Rect::new(0., 0., 20., 20.), rgb(BLUE));
        })
        .style(style().size(20, 20))
    });
    assert_eq!(screen.errors().len(), 1, "diagnostic once per draw layer");
    assert_eq!(
        screen.errors()[0].kind(),
        &kivact::diagnostics::ErrorKind::Unsupported
    );
    expect(
        &screen.screenshot(),
        10,
        10,
        BLUE,
        0,
        "valid commands still draw",
    );
}

#[test]
fn cropped_svg_keeps_the_destination_pixel_resolution() {
    static WHOLE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path d="M0 0L10 10H0Z"/></svg>"#;
    static HALF: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path d="M0 0L10 5V10H0Z"/></svg>"#;
    let whole = kivact::Asset::from(WHOLE);
    let half = kivact::Asset::from(HALF);
    let screen = render(move || {
        row((
            canvas(move |d, _| {
                d.image_with(
                    &half,
                    Rect::new(0., 0., 40., 40.),
                    ImageOptions {
                        pixelated: true,
                        ..Default::default()
                    },
                );
            })
            .style(style().size(40, 40)),
            canvas(move |d, _| {
                d.image_with(
                    &whole,
                    Rect::new(0., 0., 40., 40.),
                    ImageOptions {
                        source: Some(Rect::new(0., 0., 5., 10.)),
                        pixelated: true,
                        ..Default::default()
                    },
                );
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
                "SVG crop raster at {x},{y}"
            );
        }
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn shape_paint_percentages_resolve_against_the_supplied_rect() {
    let screen = render(|| {
        canvas(|d, _| {
            d.fill_shape(
                Rect::new(0., 0., 100., 40.),
                &Shape::circle(10),
                kivact::draw::linear_gradient(90)
                    .stop(Color::BLACK, pct(60))
                    .stop(Color::WHITE, pct(60)),
            );
        })
        .style(style().size(100, 40).background(rgb(BLUE)))
    });
    expect(
        &screen.screenshot(),
        55,
        20,
        BLACK,
        0,
        "60% is x=60, not the shape's x=52",
    );
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
fn rotated_canvas_text_image_and_clip_keep_their_paint() {
    let screen = render(|| {
        canvas(|d, _| {
            d.layer(
                Layer::new().transform(Transform::new().translate(80, 10).rotate(90)),
                |d| {
                    d.layer(Layer::new().clip_rect(Rect::new(0., 0., 40., 40.)), |d| {
                        d.pixels(
                            &Pixels::new(1, 1, Arc::from([255u8, 0, 0, 255])),
                            Rect::new(0., 0., 20., 10.),
                            ImageOptions::default(),
                        );
                        d.text(
                            "M",
                            Point::new(22., 0.),
                            TextStyle {
                                size: 18.,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        );
                    });
                },
            );
        })
        .style(style().size(120, 80).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 75, 15, RED, 0, "image is rotated");
    expect(
        &shot,
        85,
        15,
        WHITE,
        0,
        "image leaves its original orientation",
    );
    assert!(
        (32..52).any(|y| (56..80).any(|x| pixel(&shot, x, y) != WHITE)),
        "rotated glyph paints"
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

/// A path with a non-finite arc is dropped before Arvane validates it; the
/// frame and the commands around it survive, and the drop is reported once
/// as an `Unsupported` error (never silently).
#[test]
fn non_finite_arcs_are_dropped_without_losing_the_frame() {
    let screen = render(|| {
        canvas(|d, _| {
            let bad = [
                Path::new().arc(Point::new(20., 20.), f32::NAN, 0, 360, false),
                Path::new().arc(Point::new(20., 20.), 10, f32::NAN, 90, false),
                Path::new().arc(Point::new(20., 20.), f32::INFINITY, 0, 90, false),
                Path::new().move_to(Point::new(0., 0.)).arc_to(
                    (f32::NAN, 4),
                    0,
                    false,
                    true,
                    Point::new(8., 8.),
                ),
            ];
            for path in &bad {
                d.fill_path(path, rgb(RED));
                d.stroke_path(path, &Stroke::new(2), rgb(RED));
                d.fill_path(path, radial_gradient().colors([rgb(RED), rgb(BLUE)]));
            }
            d.fill_rect(Rect::new(40., 0., 20., 40.), rgb(BLUE));
        })
        .style(style().size(60, 40).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 10, 10, WHITE, 0, "the invalid arcs draw nothing");
    expect(&shot, 50, 20, BLUE, 0, "valid commands still draw");
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].kind(),
        &kivact::diagnostics::ErrorKind::Unsupported
    );
    assert!(
        errors[0].message().contains("draw command"),
        "{}",
        errors[0].message()
    );
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
fn group_opacity_fades_images_raw_pixels_and_image_pattern_fills_once() {
    static RED_IMAGE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12"><rect width="12" height="12" fill="#ff0000"/></svg>"##;
    let image = kivact::Asset::from(RED_IMAGE);
    let blue = Pixels::new(1, 1, Arc::from([0u8, 0, 255, 255]));
    // Public Screen rendering is CPU-backed; GPU selection is internal.
    let screen = kivact::test::render_with(
        kivact::test::RenderOptions {
            width: 100.,
            height: 40.,
            ..Default::default()
        },
        move || {
            canvas(move |d, _| {
                d.layer(Layer::new().opacity(0.5), |d| {
                    d.image(&image, Rect::new(0., 0., 20., 20.));
                    d.image_with(
                        &image,
                        Rect::new(20., 0., 20., 20.),
                        ImageOptions {
                            source: Some(Rect::new(0., 0., 12., 12.)),
                            opacity: 0.5,
                            pixelated: true,
                        },
                    );
                    d.pixels(
                        &blue,
                        Rect::new(40., 0., 20., 20.),
                        ImageOptions {
                            opacity: 0.5,
                            ..Default::default()
                        },
                    );
                    d.fill_shape(
                        Rect::new(60., 0., 20., 20.),
                        &Shape::circle(8),
                        kivact::draw::Paint::image(image.clone()).size(6, 6),
                    );
                    d.fill_path(
                        &Path::new().rect(Rect::new(80., 0., 20., 20.)),
                        kivact::draw::Paint::image(image.clone()).size(6, 6),
                    );
                    d.image(&image, Rect::new(0., 20., 30., 20.));
                    d.pixels(
                        &blue,
                        Rect::new(20., 20., 20., 20.),
                        ImageOptions::default(),
                    );
                });
                d.image(&image, Rect::new(60., 20., 20., 20.));
            })
            .style(style().size(100, 40).background(Color::WHITE))
        },
    );
    let shot = screen.screenshot();
    expect(
        &shot,
        10,
        10,
        0xff8080,
        1,
        "ordinary image gets group opacity",
    );
    expect(
        &shot,
        30,
        10,
        0xffbfbf,
        1,
        "sampled image opacity composes with the group",
    );
    expect(
        &shot,
        50,
        10,
        0xbfbfff,
        1,
        "raw pixels opacity composes with the group",
    );
    expect(
        &shot,
        70,
        10,
        0xff8080,
        1,
        "shape image pattern gets group opacity",
    );
    expect(
        &shot,
        90,
        10,
        0xff8080,
        1,
        "path image pattern gets group opacity",
    );
    expect(&shot, 10, 30, 0xff8080, 1, "group image without overlap");
    expect(
        &shot,
        25,
        30,
        0x8080ff,
        1,
        "opaque blue wins before the group fades once",
    );
    expect(
        &shot,
        35,
        30,
        0x8080ff,
        1,
        "overlap has the same alpha as blue alone",
    );
    expect(
        &shot,
        70,
        30,
        RED,
        0,
        "the end of the layer restores full image opacity",
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn raw_pixels_update_each_frame_crop_pixelated_and_release_old_allocations() {
    let slot = Rc::new(Cell::new(None));
    let setup = slot.clone();
    let first: Arc<[u8]> = Arc::from([255u8, 0, 0, 255, 0, 0, 255, 255]);
    let weak = Arc::downgrade(&first);
    let screen = render(move || {
        let pixels = signal(Pixels::new(2, 1, first));
        setup.set(Some(pixels));
        canvas(move |d, _| {
            d.pixels(
                &pixels.get(),
                Rect::new(0., 0., 40., 20.),
                ImageOptions {
                    source: Some(Rect::new(1., 0., 1., 1.)),
                    pixelated: true,
                    opacity: 0.5,
                },
            );
            d.pixels(
                &pixels.get(),
                Rect::new(40., 0., 40., 20.),
                ImageOptions {
                    pixelated: true,
                    ..Default::default()
                },
            );
        })
        .style(style().size(80, 40).background(Color::WHITE))
    });
    expect(
        &screen.screenshot(),
        10,
        10,
        0x8080ff,
        1,
        "cropped blue pixel faded",
    );
    let shot = screen.screenshot();
    expect(&shot, 59, 10, RED, 0, "nearest keeps the last red pixel");
    expect(
        &shot,
        60,
        10,
        BLUE,
        0,
        "nearest starts the blue pixel without blending",
    );
    let pixels = slot.get().expect("mounted");
    for _ in 0..4 {
        pixels.set(Pixels::new(
            2,
            1,
            Arc::from([255u8, 0, 0, 255, 0, 255, 0, 255]),
        ));
        screen.run_until_idle();
        let _ = screen.screenshot();
    }
    expect(
        &screen.screenshot(),
        10,
        10,
        0x80ff80,
        1,
        "new pixels visible",
    );
    assert!(weak.upgrade().is_none(), "old frame allocation is released");
    assert!(screen.errors().is_empty());
}

#[test]
fn canvas_text_alignment_matches_measured_position() {
    let screen = render(|| {
        row((
            canvas(|d, _| {
                d.text(
                    "Label",
                    Point::new(60., 30.),
                    TextStyle {
                        align: TextAlign::Center,
                        baseline: TextBaseline::Middle,
                        color: Color::BLACK,
                        ..Default::default()
                    },
                );
            })
            .style(style().size(120, 60)),
            canvas(|d, _| {
                let style = TextStyle {
                    color: Color::BLACK,
                    ..Default::default()
                };
                let m = d.measure_text("Label", &style);
                d.text(
                    "Label",
                    Point::new(60. - m.width / 2., 30. - m.height / 2.),
                    style,
                );
            })
            .style(style().size(120, 60)),
        ))
        .style(style().size(240, 60).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    // Both canvases stretch to equal halves of the row.
    for y in 0..60 {
        for x in 0..120 {
            assert_eq!(pixel(&shot, x, y), pixel(&shot, x + 120, y));
        }
    }
}

#[test]
fn rotated_rect_and_circle_regions_hit_their_actual_shape() {
    let seen = Rc::new(Cell::new(None));
    let events = seen.clone();
    let screen = render(move || {
        canvas(|d, _| {
            d.layer(
                Layer::new().transform(Transform::new().translate(50, 10).rotate(45)),
                |d| {
                    d.region(RegionId(1), Rect::new(0., 0., 20., 20.));
                },
            );
            d.region_shape(
                RegionId(2),
                Rect::new(80., 0., 40., 40.),
                &Shape::circle(20),
            );
        })
        .region(
            RegionId(1),
            kivact::primitive::part(|state| state)
                .role(Role::Button)
                .name("diamond"),
        )
        .region(
            RegionId(2),
            kivact::primitive::part(|state| state)
                .role(Role::Button)
                .name("circle"),
        )
        .on_pointer(move |e: PointerEvent| {
            if e.phase == kivact::view::PointerPhase::Down {
                events.set(e.region());
            }
            Handled::Yes
        })
        .style(style().size(140, 70))
    });
    screen.click_at(37., 12.);
    assert_eq!(seen.get(), None);
    screen.click_at(50., 24.);
    assert_eq!(seen.get(), Some(RegionId(1)));
    screen.click_at(81., 1.);
    assert_eq!(seen.get(), None);
    screen.click_at(100., 20.);
    assert_eq!(seen.get(), Some(RegionId(2)));
}

#[test]
fn extended_lines_and_rectangles_match_stroked_paths() {
    let screen = render(|| {
        row(Vec::from([true, false].map(|direct| {
            canvas(move |d, _| {
                let strokes = [
                    Stroke::new(4)
                        .cap(LineCap::Round)
                        .dash([7., 3.])
                        .dash_offset(2),
                    Stroke::new(4)
                        .cap(LineCap::Square)
                        .join(LineJoin::Bevel)
                        .miter_limit(2),
                    Stroke::new(4),
                ];
                let paints = [
                    Paint::from(Color::BLACK),
                    Paint::from(radial_gradient().colors([rgb(RED), rgb(BLUE)])),
                ];
                let mut y = 10.0;
                for (i, stroke) in strokes.iter().enumerate() {
                    for (j, paint) in paints.iter().enumerate() {
                        if i == 2 && j == 0 {
                            continue;
                        }
                        let from = Point::new(10., y);
                        let to = Point::new(100., y);
                        let rect = Rect::new(10., y + 12., 90., 24.);
                        if direct {
                            d.line(from, to, stroke.clone(), paint.clone());
                            d.stroke_rect(rect, stroke.clone(), paint.clone());
                        } else {
                            d.stroke_path(
                                &Path::new().move_to(from).line_to(to),
                                stroke,
                                paint.clone(),
                            );
                            d.stroke_path(&Path::new().rect(rect), stroke, paint.clone());
                        }
                        y += 60.;
                    }
                }
            })
            .style(style().size(120, 310))
        })))
        .style(style().size(240, 310).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 20, 10, BLACK, 0, "extended line is drawn");
    for y in 0..310 {
        for x in 0..120 {
            assert_eq!(pixel(&shot, x, y), pixel(&shot, x + 120, y), "at {x}, {y}");
        }
    }
    assert!(screen.errors().is_empty());
}
