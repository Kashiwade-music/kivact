//! Role: Tests for the compositor: damage, caches, backdrops and layers.
//!
//! The frame is composed from layers. These tests check that partial
//! redraws, cached layers, backdrops, layer composites and the cull and
//! inline rules draw what a fresh mount draws. Pixels of compositor ops
//! are compared within ±2/255 per channel (tier 2) unless a test says
//! otherwise; the frame statistics come from `Screen::frame_stats`.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use kivact::draw::RegionId;
use kivact::draw::{BlendMode, Composite, Filter, Layer, PaintShader, Transform};
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact::style::{Isolation, TransformStyle, WillChange, rgba};
use kivact::test::{RenderOptions, Screenshot, render, render_with};
use kivact::view::canvas;

use crate::paint::{BLACK, BLUE, GREEN, RED, WHITE, at, canvas_of, distance, expect, pixel};
use crate::support::mount;

type Log = Rc<RefCell<Vec<String>>>;

/// The window of the tests: 800 × 600 device px.
const W: u32 = 800;
const H: u32 = 600;

/// Every pixel of `got` is within `tolerance` of `want` (per channel).
#[track_caller]
fn assert_same_pixels(got: &Screenshot, want: &Screenshot, tolerance: u32, what: &str) {
    let mut differing = 0u32;
    let mut first = None;
    for y in 0..H {
        for x in 0..W {
            let (g, w) = (pixel(got, x, y), pixel(want, x, y));
            if distance(g, w) > tolerance {
                differing += 1;
                first.get_or_insert((x, y, g, w));
            }
        }
    }
    assert!(
        differing == 0,
        "{what}: {differing} pixels differ by more than ±{tolerance}; first {first:?}"
    );
}

/// A button named `name` that logs its activations.
fn button(name: String, log: &Log) -> Node {
    let log = log.clone();
    let shown = name.clone();
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || log.borrow_mut().push(shown.clone()))
}

/// A 10 × 10 grid of boxes; the box 37 takes the colour `hot` holds.
fn grid(hot: Signal<u32>) -> impl IntoView {
    let boxes: Vec<Node> = (0..100u32)
        .map(|i| {
            let (x, y) = (20 + (i % 10) as i32 * 60, 20 + (i / 10) as i32 * 55);
            if i == 37 {
                node().style(move || at(x, y, 50, 40).background(rgb(hot.get())))
            } else {
                let colour = 0x101010 * ((i % 15) + 1);
                node().style(at(x, y, 50, 40).background(rgb(colour)))
            }
        })
        .collect();
    canvas_of(boxes)
}

#[test]
fn partial_redraw_matches_full_redraw() {
    let (screen, hot) = mount(|| {
        let hot = signal(BLUE);
        (hot, grid(hot))
    });
    screen.screenshot();
    hot.set(RED);
    let partial = screen.screenshot();
    let damage = screen.frame_stats().damage_pixels;
    assert!(
        damage < u64::from(W * H) / 10,
        "the damage is under a tenth of the window: {damage} px"
    );
    let fresh = render(|| grid(signal(RED))).screenshot();
    assert_same_pixels(&partial, &fresh, 2, "the partial redraw");
}

/// A 120 × 80 box at (40, 40) inside a blurred parent, and a glass panel
/// over a box at (40, 40), both coloured by `colour`.
fn blurred_parent(colour: Signal<u32>) -> impl IntoView {
    canvas_of(
        node()
            .style(at(40, 40, 300, 200).filter(Filter::new().blur(4)))
            .child(node().style(move || style().size(120, 80).background(rgb(colour.get())))),
    )
}

fn glass_over(colour: Signal<u32>) -> impl IntoView {
    canvas_of((
        node().style(move || at(40, 40, 200, 160).background(rgb(colour.get()))),
        node().style(at(100, 80, 300, 200).backdrop_blur(6)),
    ))
}

#[test]
fn damage_reaches_through_blur_and_backdrop() {
    // The blur of a parent is repainted when a child inside changes.
    let (screen, colour) = mount(|| {
        let colour = signal(BLUE);
        (colour, blurred_parent(colour))
    });
    screen.screenshot();
    colour.set(RED);
    let changed = screen.screenshot();
    let fresh = render(|| blurred_parent(signal(RED))).screenshot();
    assert_same_pixels(&changed, &fresh, 2, "a child under a blur");
    // The glass repaints when the content under it changes.
    let (screen, colour) = mount(|| {
        let colour = signal(BLUE);
        (colour, glass_over(colour))
    });
    screen.screenshot();
    colour.set(RED);
    let changed = screen.screenshot();
    let fresh = render(|| glass_over(signal(RED))).screenshot();
    assert_same_pixels(&changed, &fresh, 2, "the backdrop under a glass");
}

/// A 40 × 40 translucent box at `x`.
fn mover(x: Signal<i32>) -> impl IntoView {
    canvas_of(node().style(move || at(x.get(), 100, 40, 40).background(rgb(RED)).opacity(0.5)))
}

#[test]
fn translucent_move_leaves_no_trail() {
    let (screen, x) = mount(|| {
        let x = signal(100);
        (x, mover(x))
    });
    screen.screenshot();
    x.set(300);
    let moved = screen.screenshot();
    let fresh = render(|| mover(signal(300))).screenshot();
    assert_same_pixels(&moved, &fresh, 2, "the moved translucent box");
}

/// The red box of the cached layer at `dx` (`will_change(Transform)`).
fn cached(dx: Signal<f64>) -> impl IntoView {
    canvas_of(node().style(move || {
        at(100, 100, 120, 120)
            .background(rgb(RED))
            .will_change(WillChange::Transform)
            .transform(Transform::new().translate(dx.get(), 0.0))
    }))
}

#[test]
fn cached_layer_moves_without_raster() {
    let (screen, dx) = mount(|| {
        let dx = signal(0.0);
        (dx, cached(dx))
    });
    screen.screenshot();
    // 16 ms frames; the box moves from 0 to 37.25 px over 160 ms.
    // Each frame's statistics are kept, so a failure names the frames.
    let mut report = Vec::new();
    for frame in 1..=10 {
        let at_px = 37.25 * f64::from(frame) / 10.0;
        dx.set(at_px);
        screen.advance(Duration::from_millis(16));
        let advanced = screen.frame_stats();
        let shot = screen.screenshot();
        let stats = screen.frame_stats();
        report.push(format!(
            "frame {frame} dx {at_px:.3}: after advance chunks {} pixels {} damage {}; \
             after screenshot chunks {} pixels {} damage {}",
            advanced.chunks_rastered,
            advanced.pixels_rastered,
            advanced.damage_pixels,
            stats.chunks_rastered,
            stats.pixels_rastered,
            stats.damage_pixels,
        ));
        // The left edge of the red box on row 160, within 1 device px.
        let left = (0..W)
            .find(|&x| distance(pixel(&shot, x, 160), RED) <= 8)
            .expect("the box is drawn");
        let want = 100.0 + at_px;
        assert!(
            (f64::from(left) - want).abs() <= 1.0,
            "frame {frame}: the left edge at {left} is not {want}"
        );
    }
    let rastered: Vec<&String> = report
        .iter()
        .filter(|line| {
            !line.contains("after advance chunks 0 ")
                || !line.contains("after screenshot chunks 0 ")
        })
        .collect();
    assert!(
        rastered.is_empty(),
        "frames that rastered a chunk: {rastered:#?}\nall frames: {report:#?}"
    );
    let moved = screen.screenshot();
    let fresh = render(|| cached(signal(37.25))).screenshot();
    assert_same_pixels(&moved, &fresh, 2, "the cached layer at its final place");
}

/// Two blue-on-white stripes behind a glass panel at (300, 100) that
/// blurs what is beneath with `layers` copies stacked.
fn stripes_under_glass(layers: u32) -> impl IntoView {
    let panels: Vec<Node> = (0..layers)
        .map(|_| node().style(at(300, 100, 200, 200).backdrop_blur(8)))
        .collect();
    canvas_of((
        node().style(at(0, 0, 400, 600).background(rgb(RED))),
        node().style(at(400, 0, 400, 600).background(rgb(BLUE))),
        panels,
    ))
}

#[test]
fn backdrop_sees_backdrop() {
    let one = render(|| stripes_under_glass(1)).screenshot();
    let two = render(|| stripes_under_glass(2)).screenshot();
    // 10 px inside the blue side: a second blur spreads the edge further.
    let (a, b) = (pixel(&one, 410, 150), pixel(&two, 410, 150));
    assert!(
        distance(a, b) > 4,
        "two glass layers blur more than one: {a:06x} and {b:06x}"
    );
}

#[test]
fn backdrop_in_overlay_sees_the_app() {
    // The panel in the overlay is the same glass as a plain panel at the
    // same place (the overlay adds only its layer, which does not change the
    // backdrop): the pixels inside the panel are
    // the blurred stripes in both. Tier 2 for the blur.
    let plain = render(|| {
        canvas_of((
            node().style(at(0, 0, 400, 600).background(rgb(RED))),
            node().style(at(400, 0, 400, 600).background(rgb(BLUE))),
            node().style(at(300, 100, 200, 100).backdrop_blur(8)),
        ))
    })
    .screenshot();
    let (screen, rect) = mount(|| {
        let r = node_ref();
        let panel = node()
            .node_ref(r)
            .style(style().size(200, 100).backdrop_blur(8));
        let view = canvas_of((
            node().style(at(0, 0, 400, 600).background(rgb(RED))),
            node().style(at(400, 0, 400, 600).background(rgb(BLUE))),
            kivact::view::overlay_at(
                Point::new(300.0, 100.0),
                kivact::view::Placement::Below,
                panel,
            ),
        ));
        (r.rect(), view)
    });
    let overlaid = screen.screenshot();
    // The panel really is where the test looks: it contains (405, 150).
    let drawn = rect.get_untracked().expect("the panel is laid out");
    assert!(
        (drawn.x..drawn.x + drawn.width).contains(&405.0)
            && (drawn.y..drawn.y + drawn.height).contains(&150.0),
        "the panel is at {drawn:?}, not around (405, 150)"
    );
    for (x, y) in [(320, 120), (405, 150), (470, 170)] {
        let (a, b) = (pixel(&plain, x, y), pixel(&overlaid, x, y));
        assert!(
            distance(a, b) <= 2,
            "the overlay panel at ({x}, {y}): {a:06x} against {b:06x}"
        );
    }
}

#[test]
fn destination_out_cuts_a_hole() {
    // Compositor tier: the hole shows the canvas background beneath.
    let screen = render(|| {
        canvas(|d, _| {
            d.fill_rect(Rect::new(0.0, 0.0, 200.0, 200.0), rgb(RED));
            d.layer(Layer::new().composite(Composite::DestinationOut), |d| {
                d.circle(Point::new(100.0, 100.0), 40.0, rgb(BLACK))
            });
        })
        .style(style().size(200, 200).background(Color::WHITE))
    });
    let shot = screen.screenshot();
    expect(&shot, 100, 100, WHITE, 2, "inside the hole: the background");
    expect(&shot, 10, 10, RED, 2, "outside the hole: the red");
}

/// A button per index, 8 × 8 each, in a grid of 100 columns from (0, 0),
/// each on its own layer (`will_change`), logging its name on press.
fn cached_buttons(log: &Log, count: u32) -> Vec<Node> {
    (0..count)
        .map(|i| {
            let (x, y) = (((i % 100) * 8) as i32, ((i / 100) * 8) as i32);
            button(format!("b{i}"), log).style(
                at(x, y, 8, 8)
                    .background(rgb(BLUE))
                    .will_change(WillChange::Transform),
            )
        })
        .collect()
}

#[test]
fn many_cache_layers_keep_content() {
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || canvas_of(cached_buttons(&log, 5000))
    });
    let shot = screen.screenshot();
    for i in 0..5000u32 {
        let (x, y) = ((i % 100) * 8 + 4, (i / 100) * 8 + 4);
        assert!(
            distance(pixel(&shot, x, y), WHITE) > 8,
            "button {i} is drawn at its centre"
        );
    }
    screen.click_at(796.0, 396.0);
    assert_eq!(*log.borrow(), ["b4999"], "the last button is clicked");
}

#[test]
fn cull_keeps_blur_reaching_into_view() {
    // The box is 20 px left of the window; its blur reaches the window's
    // first 5 px. Shifted 40 px in a window 40 px wider, the same scene
    // gives the same pixels (Compositor tier, ±2/255).
    let narrow = render_with(RenderOptions::from_env().window_size(800.0, 600.0), || {
        blurred_box(800, -20)
    })
    .screenshot();
    let wide = render_with(RenderOptions::from_env().window_size(840.0, 600.0), || {
        blurred_box(840, 20)
    })
    .screenshot();
    for y in 0..H {
        for x in 0..6 {
            let (a, b) = (pixel(&narrow, x, y), pixel(&wide, x + 40, y));
            assert!(
                distance(a, b) <= 2,
                "pixel ({x}, {y}) of the narrow window: {a:06x} against {b:06x}"
            );
        }
    }
}

/// A white canvas of `width` holding a 10 px black box at `x`, blurred.
fn blurred_box(width: i32, x: i32) -> impl IntoView {
    canvas_of(
        node().style(
            at(x, 100, 10, 100)
                .background(rgb(BLACK))
                .filter(Filter::new().blur(10)),
        ),
    )
    .style(style().size(width, 600).background(Color::WHITE))
}

#[test]
fn inline_keeps_isolation_and_blend() {
    // Past the 1024 targets, the isolated group and its Multiply child are
    // not inlined: the child multiplies with the group's own backdrop.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            let group = node()
                .style(at(100, 450, 200, 100).isolation(Isolation::Isolate))
                .child(
                    node().style(
                        at(0, 0, 200, 100)
                            .background(rgb(BLUE))
                            .mix_blend_mode(BlendMode::Multiply),
                    ),
                );
            node()
                .style(style().size(800, 600).background(rgb(RED)))
                .child((cached_buttons(&log, 5000), group))
        }
    });
    let shot = screen.screenshot();
    expect(&shot, 200, 500, BLUE, 2, "the Multiply child is blue");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn coplanar_draw_order_is_kept_in_3d() {
    // Three coplanar draws in one Preserve3d canvas: the last one (green)
    // is on top, and the regions of the three draws are hit with it on top.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas(|d, _| {
                let rect = Rect::new(20.0, 20.0, 100.0, 80.0);
                d.region(RegionId(1), rect);
                d.region(RegionId(2), rect);
                d.region(RegionId(3), rect);
                d.fill_rect(rect, rgb(RED));
                d.layer(Layer::new().transform(Transform::new().scale_z(2.0)), |d| {
                    d.fill_rect(rect, rgb(BLUE))
                });
                d.fill_rect(rect, rgb(GREEN));
            })
            .style(
                style()
                    .size(800, 600)
                    .background(Color::WHITE)
                    .transform_style(TransformStyle::Preserve3d),
            )
            .region(RegionId(1), region_part("red", &log))
            .region(RegionId(2), region_part("blue", &log))
            .region(RegionId(3), region_part("green", &log))
        }
    });
    let shot = screen.screenshot();
    expect(&shot, 70, 60, GREEN, 2, "the last draw is on top");
    screen.click_at(70.0, 60.0);
    assert_eq!(*log.borrow(), ["green"], "the region drawn last is hit");
}

/// A region part that logs `name` on press.
fn region_part(name: &'static str, log: &Log) -> kivact::custom::Part<kivact::custom::PartState> {
    let attachment = kivact::custom::part(|state| state);
    let log = log.clone();
    kivact::custom::Part::new(attachment.state(), move |node| {
        let log = log.clone();
        node.part(attachment)
            .role(Role::Button)
            .name(name)
            .on_press(move || log.borrow_mut().push(name.to_string()))
    })
}

#[test]
fn crowded_3d_context_hits_what_is_drawn() {
    // 300 planes through one centre: more than MAX_CONTEXT_PLANES, so they
    // overlap flat in DOM order. The centre is the last button's colour.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            let planes: Vec<Node> = (0..300)
                .map(|i| {
                    let colour = if i == 299 { GREEN } else { BLUE };
                    button(format!("b{i}"), &log).style(
                        at(0, 0, 120, 80)
                            .background(rgb(colour))
                            .rotate_y(f64::from(i) * 2.0),
                    )
                })
                .collect();
            canvas_of(
                node()
                    .style(at(100, 100, 120, 80).transform_style(TransformStyle::Preserve3d))
                    .child(planes),
            )
        }
    });
    let shot = screen.screenshot();
    expect(
        &shot,
        160,
        140,
        GREEN,
        2,
        "the centre is the last button's colour",
    );
    screen.click_at(160.0, 140.0);
    assert_eq!(
        *log.borrow(),
        ["b299"],
        "the last button in DOM order is hit"
    );
}

/// A paint shader that returns blue, for the translucent fallback test.
const SHADER: &str = "fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {\n    return vec4<f32>(0.0, 0.0, 1.0, 1.0);\n}\n";

/// A translucent red shader fallback in the root's top-left corner, and a box beside it.
fn shader_beside_box(colour: Signal<u32>) -> impl IntoView {
    let shader = PaintShader::wgsl(SHADER).expect("a valid paint shader");
    canvas_of((
        canvas(move |d, _| {
            d.fill_rect(
                Rect::new(0.0, 0.0, 100.0, 100.0),
                shader.paint().fallback(rgba(RED, 0.5)),
            );
        })
        .style(at(0, 0, 100, 100)),
        node().style(move || at(300, 300, 60, 60).background(rgb(colour.get()))),
    ))
}

#[test]
fn root_shader_fallback_is_not_blended_again_outside_the_damage() {
    // The shader's fallback is drawn only where the window is damaged: frames that
    // change the box keep the pixels under the shader, which is blended once.
    let (screen, colour) = mount(|| {
        let colour = signal(BLUE);
        (colour, shader_beside_box(colour))
    });
    screen.screenshot();
    colour.set(RED);
    screen.screenshot();
    colour.set(GREEN);
    screen.screenshot();
    colour.set(RED);
    let changed = screen.screenshot();
    // The reference is mounted on the same lane: on the GPU the shader paints itself.
    let (fresh_screen, _) = mount(|| {
        let colour = signal(RED);
        (colour, shader_beside_box(colour))
    });
    let fresh = fresh_screen.screenshot();
    assert_same_pixels(
        &changed,
        &fresh,
        2,
        "the shader fallback after partial frames",
    );
}
