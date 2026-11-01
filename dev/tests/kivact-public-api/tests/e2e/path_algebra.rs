//! Role: Tests for path rendering and transformation.
//! paths and stroke outlines are what pixels and hit regions show.
use super::paint::{BLACK, RED, WHITE, expect};
use kivact::control::canvas;
use kivact::draw::Path;
use kivact::draw::{LineCap, Stroke, Transform};
use kivact::draw::{Point, Rect, RegionId};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::test::render;
use kivact::view::{Handled, PointerEvent};
use std::{cell::Cell, rc::Rc};

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
fn appended_paths_fill_as_one_and_the_bounds_cover_both() {
    let a = Path::new().rect(Rect::new(5., 5., 20., 20.));
    let b = Path::new().rect(Rect::new(45., 5., 20., 20.));
    let both = a.append(&b);
    assert_eq!(both.bounds(), Some(Rect::new(5., 5., 60., 20.)));
    let screen = render(move || {
        canvas(move |d, _| d.fill_path(&both, Color::BLACK))
            .style(style().size(80, 40).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 15, 15, BLACK, 0, "first");
    expect(&shot, 55, 15, BLACK, 0, "second");
    expect(&shot, 35, 15, WHITE, 0, "between them");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_transformed_path_is_the_hit_region_it_is_drawn_as() {
    let seen = Rc::new(Cell::new(None));
    let events = seen.clone();
    let screen = render(move || {
        canvas(|d, _| {
            let disc = circle(Point::new(0., 0.), 10.)
                .transformed(Transform::new().translate(30, 30).scale_xy(3, 1));
            d.region_path(RegionId(1), disc.bounds().unwrap_or_default(), &disc);
        })
        .region(
            RegionId(1),
            kivact::primitive::part(|state| state)
                .role(Role::Button)
                .name("ellipse"),
        )
        .on_pointer(move |e: PointerEvent| {
            if e.phase == kivact::view::PointerPhase::Down {
                events.set(e.region());
            }
            Handled::Yes
        })
        .style(style().size(100, 60))
    });
    // Inside along the stretched axis, outside along the other.
    screen.click_at(55., 30.);
    assert_eq!(seen.get(), Some(RegionId(1)));
    screen.click_at(30., 45.);
    assert_eq!(seen.get(), None);
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

#[test]
fn a_closed_path_trims_around_its_start() {
    let square = Path::new().rect(Rect::new(10., 10., 40., 40.));
    // A quarter of the perimeter starting a quarter in: the right edge.
    let part = square.trim(0.0, 0.25, 0.25);
    let screen = render(move || {
        canvas(move |d, _| d.stroke_path(&part, &Stroke::new(6), Color::BLACK))
            .style(style().size(60, 60).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 50, 30, BLACK, 0, "the right edge");
    expect(&shot, 30, 10, WHITE, 0, "the top edge is left out");
    expect(&shot, 30, 50, WHITE, 0, "the bottom edge is left out");
    expect(&shot, 10, 30, WHITE, 0, "the left edge is left out");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_stroke_outline_fills_what_the_stroke_covers() {
    let dashed = Stroke::new(8).cap(LineCap::Butt).dash([20., 10.]);
    let path = line((10., 10.), (110., 10.));
    let outline = path.stroke_outline(&dashed);
    let screen = render(move || {
        canvas(move |d, _| {
            d.stroke_path(&path, &dashed, Color::BLACK);
            d.fill_path(
                &outline.transformed(Transform::new().translate(0, 30)),
                Color::BLACK,
            );
        })
        .style(style().size(120, 50).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    for y in [10, 40] {
        expect(&shot, 20, y, BLACK, 0, "a dash");
        expect(&shot, 35, y, WHITE, 0, "a gap");
        expect(&shot, 50, y, BLACK, 0, "the next dash");
        expect(&shot, 20, y - 7, WHITE, 0, "above the line");
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn flattened_paths_expose_contours_curves_and_fill_rules() {
    use kivact::draw::{FillRule, PathContour};
    let square = Path::new().rect(Rect::new(10., 10., 20., 20.));
    assert_eq!(square.fill_rule(), FillRule::NonZero);
    let square = square.with_fill_rule(FillRule::EvenOdd);
    assert_eq!(square.fill_rule(), FillRule::EvenOdd);
    assert_eq!(
        square.flattened(0.25),
        vec![PathContour {
            points: vec![
                Point::new(10., 10.),
                Point::new(30., 10.),
                Point::new(30., 30.),
                Point::new(10., 30.)
            ],
            closed: true,
        }]
    );
    let open = line((40., 10.), (60., 30.));
    let both = square.append(&open).flattened(0.25);
    assert_eq!(both.len(), 2);
    assert!(both[0].closed);
    assert_eq!(
        both[1],
        PathContour {
            points: vec![Point::new(40., 10.), Point::new(60., 30.)],
            closed: false
        }
    );
    let cubic = Path::new().move_to(Point::new(10., 50.)).cubic_to(
        Point::new(10., 10.),
        Point::new(60., 10.),
        Point::new(60., 50.),
    );
    let coarse = cubic.flattened(2.0);
    let fine = cubic.flattened(0.1);
    assert!(!fine[0].closed);
    assert_eq!(fine[0].points.first(), Some(&Point::new(10., 50.)));
    assert_eq!(fine[0].points.last(), Some(&Point::new(60., 50.)));
    assert!(fine[0].points.len() > coarse[0].points.len());
    assert!(fine[0].points.iter().any(|p| p.y < 25.));
    let arc = Path::new().arc(Point::new(80., 30.), 20, 0, 180, false);
    let contour = &arc.flattened(0.25)[0];
    assert!(!contour.closed);
    assert!(contour.points.len() > 2);
    assert_eq!(contour.points.first(), Some(&Point::new(100., 30.)));
    let end = contour.points.last().unwrap();
    assert!((end.x - 60.).abs() < 0.01 && (end.y - 30.).abs() < 0.01);
    for tolerance in [0., -1., f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(cubic.flattened(tolerance), cubic.flattened(0.25));
    }
    assert!(Path::new().flattened(0.25).is_empty());
    assert!(
        Path::new()
            .move_to(Point::new(f32::NAN, 0.))
            .flattened(0.25)
            .is_empty()
    );
    let screen = render(move || {
        canvas(move |draw, _| {
            for path in [&square, &open, &cubic, &arc] {
                draw.stroke_path(path, &Stroke::new(1), Color::BLACK);
            }
        })
        .style(style().size(120, 80))
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
