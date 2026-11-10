//! Role: Tests for the frame sources the host keeps in an index.
//!
//! An animated image and a display-rate shader clock ask for frames only
//! while their node is shown, attached and not under reduced motion (an
//! essential shader keeps its clock); a duplicate layout id is reported
//! once; frames and layouts built from partial updates are what a fresh
//! window draws and lays out.

use std::time::Duration;

use super::images_animated::gif;
use super::paint::{GREEN, RED, at, canvas_of, expect};
use crate::support::mount;
use kivact::draw::PaintShader;
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::View;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The frame ticks the screen counts while it advances by `d`.
fn ticks_over(screen: &Screen, d: Duration) -> u64 {
    let before = screen.frame_ticks();
    screen.advance(d);
    screen.frame_ticks() - before
}

/// A 40 × 40 GIF of three 100 ms frames (red, green, blue), hidden while
/// `hidden` holds.
fn hideable_gif(hidden: Signal<bool>) -> Node {
    canvas_of(
        image(gif())
            .test_id("gif")
            .hidden(hidden)
            .style(at(0, 0, 40, 40)),
    )
}

#[test]
fn hidden_gif_stops_and_restarts_from_its_first_frame() {
    let (screen, hidden) = mount(|| {
        let hidden = signal(false);
        (hidden, hideable_gif(hidden))
    });
    expect(&screen.screenshot(), 20, 20, RED, 2, "frame 0");
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 after 100 ms",
    );
    hidden.set(true);
    // The frame asked for before the change still ticks once.
    screen.advance(ms(100));
    assert_eq!(
        ticks_over(&screen, ms(300)),
        0,
        "a hidden GIF asks for no frame"
    );
    hidden.set(false);
    screen.run_until_idle();
    expect(
        &screen.screenshot(),
        20,
        20,
        RED,
        2,
        "shown again at frame 0",
    );
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 100 ms later",
    );
}

#[test]
fn removed_gif_stops_requesting_frames() {
    let (screen, on) = mount(|| {
        let on = signal(true);
        let view = canvas_of(show(on, || {
            image(gif()).test_id("gif").style(at(0, 0, 40, 40))
        }));
        (on, view)
    });
    assert!(
        ticks_over(&screen, ms(200)) > 0,
        "a playing GIF asks for frames"
    );
    on.set(false);
    screen.run_until_idle();
    screen.advance(ms(500));
    assert_eq!(
        ticks_over(&screen, ms(300)),
        0,
        "a removed GIF asks for no frame"
    );
}

#[test]
fn reduced_motion_toggle_stops_and_resumes_gif() {
    let screen = render(|| canvas_of(image(gif()).test_id("gif").style(at(0, 0, 40, 40))));
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        GREEN,
        2,
        "frame 1 after 100 ms",
    );
    screen.set_reduced_motion(true);
    screen.advance(ms(100));
    expect(
        &screen.screenshot(),
        20,
        20,
        RED,
        2,
        "the first frame under reduced motion",
    );
    assert_eq!(
        ticks_over(&screen, ms(300)),
        0,
        "no frame under reduced motion"
    );
    expect(
        &screen.screenshot(),
        20,
        20,
        RED,
        2,
        "still the first frame",
    );
    screen.set_reduced_motion(false);
    assert!(
        ticks_over(&screen, ms(100)) > 0,
        "frames go on without reduced motion"
    );
}

/// A display-rate shader whose red channel follows its clock.
const CLOCK: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(fract(context.time), 0.0, 0.0, 1.0);\n}\n";

/// A 100 × 100 canvas filled with the clocked shader (`essential`: it runs
/// under reduced motion too), hidden while `hidden` holds.
fn clocked(hidden: Signal<bool>, essential: bool) -> Node {
    let shader = PaintShader::wgsl(CLOCK).expect("a valid paint shader");
    canvas_of(
        canvas(move |d, _| {
            let paint = shader.paint().animate(FrameRate::Display);
            let paint = if essential { paint.essential() } else { paint };
            d.fill_rect(Rect::new(0.0, 0.0, 100.0, 100.0), paint);
        })
        .hidden(hidden)
        .style(at(0, 0, 100, 100)),
    )
}

#[test]
fn display_shader_clock_follows_hidden_and_reduced_motion() {
    let (screen, hidden) = mount(|| {
        let hidden = signal(false);
        (hidden, clocked(hidden, false))
    });
    assert!(
        ticks_over(&screen, ms(100)) >= 4,
        "a display-rate clock ticks every display frame"
    );
    hidden.set(true);
    screen.advance(ms(100));
    assert_eq!(ticks_over(&screen, ms(100)), 0, "hidden: no clock");
    hidden.set(false);
    screen.run_until_idle();
    assert!(ticks_over(&screen, ms(100)) >= 4, "shown again: it ticks");
    screen.set_reduced_motion(true);
    screen.advance(ms(100));
    assert_eq!(
        ticks_over(&screen, ms(100)),
        0,
        "reduced motion stops a decorative clock"
    );

    let (screen, _) = mount(|| {
        let hidden = signal(false);
        (hidden, clocked(hidden, true))
    });
    screen.set_reduced_motion(true);
    assert!(
        ticks_over(&screen, ms(100)) >= 4,
        "an essential clock ticks under reduced motion"
    );
}

#[test]
fn duplicate_layout_id_names_the_later_node_once() {
    let screen = render(|| {
        canvas_of((
            node().layout_id("dup").style(at(0, 0, 20, 20)),
            node()
                .layout_id("dup")
                .test_id("second")
                .style(at(40, 0, 20, 20)),
        ))
    });
    let reports = |screen: &Screen| {
        screen
            .errors()
            .iter()
            .filter(|e| e.message().contains("dup"))
            .count()
    };
    assert_eq!(reports(&screen), 1, "{:?}", screen.errors());
    screen.advance(ms(100));
    screen.advance(ms(100));
    assert_eq!(reports(&screen), 1, "reported once: {:?}", screen.errors());
}

/// 50 rows of a label and a button in a 300 px scroll box; the first
/// label ends in `bumps` exclamation marks.
fn bumped_rows(bumps: Signal<usize>) -> View {
    let rows: Vec<View> = (0..50usize)
        .map(|i| {
            let label = move || {
                if i == 0 {
                    format!("Row 0{}", "!".repeat(bumps.get()))
                } else {
                    format!("Row {i}")
                }
            };
            row((text(label), button("x")))
                .style(style().gap(4))
                .into_view()
        })
        .collect();
    scroll(column(rows))
        .style(style().height(300).shrink(0.0))
        .into_view()
}

#[test]
fn incremental_frames_match_a_fresh_render() {
    let (screen, bumps) = mount(|| {
        let bumps = signal(0usize);
        (bumps, bumped_rows(bumps))
    });
    screen.screenshot();
    for n in 1..=20 {
        bumps.set(n);
        screen.screenshot();
    }
    let fresh = render(|| bumped_rows(signal(20)));
    assert_eq!(
        screen.screenshot().hash(),
        fresh.screenshot().hash(),
        "20 partial frames end in the pixels of a fresh frame"
    );
}

#[test]
fn percent_and_absolute_layouts_survive_parent_resize() {
    let (screen, wide) = mount(|| {
        let wide = signal(false);
        let view = row((
            node()
                .test_id("pct")
                .style(style().width(pct(50)).height(20).shrink(0.0)),
            node()
                .test_id("px")
                .style(style().width(100).height(20).shrink(0.0)),
        ))
        .style(move || style().width(if wide.get() { 600 } else { 400 }).height(20));
        (wide, view)
    });
    let width = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    for (on, parent) in [(true, 600.0), (false, 400.0)] {
        wide.set(on);
        screen.run_until_idle();
        assert_eq!(width("pct"), parent / 2.0, "50 % of a {parent} px row");
        assert_eq!(width("px"), 100.0, "100 px in a {parent} px row");
    }
}
