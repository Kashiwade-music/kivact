//! Role: Tests for path rendering and transformation.
//! paths and stroke outlines are what pixels and hit regions show.
use super::paint::{BLACK, RED, WHITE, expect};
use kivact::control::canvas;
use kivact::draw::Path;
use kivact::draw::Point;
use kivact::draw::{Stroke, Transform};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::test::render;

fn circle(center: Point, radius: f32) -> Path {
    Path::new().arc(center, radius, 0, 360, false).close()
}

fn line(from: (f32, f32), to: (f32, f32)) -> Path {
    Path::new()
        .move_to(Point::new(from.0, from.1))
        .line_to(Point::new(to.0, to.1))
}

#[test]
fn transformed_paths_keep_their_curves_and_move_where_they_are_drawn() {
    let screen = render(|| {
        canvas(|d, _| {
            let disc = circle(Point::new(20., 20.), 10.);
            d.fill_path(&disc, rgb(RED));
            let moved = disc.transformed(Transform::new().translate(60, 0).scale(2));
            d.fill_path(&moved, Color::BLACK);
        })
        .style(style().size(140, 80).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 20, 20, RED, 0, "the original");
    // The copy is centred at (100, 40) with radius 20, round and not a polygon
    // of its control points.
    expect(&shot, 100, 40, BLACK, 0, "the copy's centre");
    expect(&shot, 100, 22, BLACK, 0, "inside the copy's top");
    expect(&shot, 100, 17, WHITE, 0, "above the copy");
    expect(&shot, 112, 52, BLACK, 0, "inside the copy's diagonal");
    expect(&shot, 118, 58, WHITE, 0, "outside the copy's diagonal");
    expect(&shot, 60, 40, WHITE, 0, "the copy left the original place");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_trimmed_stroke_covers_only_its_stretch() {
    let path = line((0., 10.), (100., 10.));
    let screen = render(move || {
        canvas(move |d, _| {
            let stroke = Stroke::new(6);
            d.stroke_path(&path.trim(0.25, 0.5, 0.0), &stroke, Color::BLACK);
            // Turned by half the path: 50..75.
            d.stroke_path(
                &path
                    .trim(0.0, 0.25, 0.5)
                    .transformed(Transform::new().translate(0, 20)),
                &stroke,
                rgb(RED),
            );
            // The end wraps around to the start: 90..100 and 0..15.
            d.stroke_path(
                &path
                    .trim(0.0, 0.25, 0.9)
                    .transformed(Transform::new().translate(0, 40)),
                &stroke,
                Color::BLACK,
            );
        })
        .style(style().size(100, 60).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 10, 10, WHITE, 0, "before the stretch");
    expect(&shot, 30, 10, BLACK, 0, "in the stretch");
    expect(&shot, 60, 10, WHITE, 0, "after the stretch");
    expect(&shot, 40, 30, WHITE, 0, "before the turned stretch");
    expect(&shot, 60, 30, RED, 0, "in the turned stretch");
    expect(&shot, 80, 30, WHITE, 0, "after the turned stretch");
    expect(&shot, 95, 50, BLACK, 0, "the stretch's first part");
    expect(&shot, 5, 50, BLACK, 0, "the stretch wrapped to the start");
    expect(&shot, 50, 50, WHITE, 0, "between the two parts");
    assert!(screen.errors().is_empty());
}
