//! Role: Drawables: a canvas draws a view subtree; its hits, focus and export.
//!
//! Pixel tiers (defined in `support`), named again beside each test: a position
//! or a bound is compared within ±1 device px (tier 1); a solid fill's colour
//! within ±2/255 per channel (tier 2). Expected values come from the transform
//! written in each test: `rotate_y(60)` maps x to x·cos 60° = x / 2 on z = 0,
//! and `rotate(90)` maps (x, y) to (-y, x) about the canvas origin.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use kivact::draw::{Draw, Layer, Pixels};
use kivact::geometry::{Matrix4, Point, Rect, Size};
use kivact::prelude::*;
use kivact::style::{BackfaceVisibility, TransformStyle};
use kivact::test::{Screen, Screenshot};
use kivact::view::{Drawable, DrawableFrame, DrawableShare, ShareError, ShareUpdate, viewport};

use super::paint::{BLUE, RED, distance, pixel};
use super::support::mount;

/// The stage is a canvas of this size at the window's origin.
const STAGE_W: f64 = 200.0;
const STAGE_H: f64 = 120.0;
/// Where a drawable is drawn on the stage: its top-left corner.
const AX: f64 = 20.0;
const AY: f64 = 10.0;
/// The longest a test waits for an export frame.
const FRAME_WAIT: Duration = Duration::from_secs(10);

/// A white stage that `draw` paints; the drawables are mounted on it by the caller.
fn stage(draw: impl Fn(&mut Draw, Size) + 'static) -> Node {
    canvas(draw)
        .test_id("stage")
        .style(style().size(STAGE_W, STAGE_H).background(Color::WHITE))
}

/// A red box of 40 × 20.
fn red_box() -> Node {
    node().style(style().size(40, 20).background(rgb(RED)))
}

/// A button "Apply" of 80 × 30 that sets `pressed`.
fn apply(pressed: Signal<bool>) -> impl IntoView {
    button("Apply")
        .on_press(move || pressed.set(true))
        .style(style().size(80, 30))
}

/// A red box of 40 × 20 that calls `on_press` when clicked.
fn red_button(on_press: impl Fn() + 'static) -> impl IntoView {
    node()
        .style(style().size(40, 20).background(rgb(RED)))
        .on_press(on_press)
}

/// The origin of the element `test_id`, in window px.
fn origin_of(screen: &Screen, test_id: &str) -> (f64, f64) {
    let rect = screen
        .get_by_test_id(test_id)
        .rect()
        .expect("the element is laid out");
    (rect.x, rect.y)
}

/// The screenshot of the stage alone, in stage px.
fn stage_shot(screen: &Screen) -> Screenshot {
    let (x, y) = origin_of(screen, "stage");
    screen
        .screenshot()
        .crop(x as u32, y as u32, STAGE_W as u32, STAGE_H as u32)
}

/// A click at a point of the stage, in stage px.
fn click_stage(screen: &Screen, x: f64, y: f64) {
    let (ox, oy) = origin_of(screen, "stage");
    screen.click_at(ox + x, oy + y);
}

/// The channel distance between `got` and `want` is at most `tolerance`.
fn near(got: u32, want: u32, tolerance: u32) -> bool {
    distance(got, want) <= tolerance
}

/// The inclusive bounds (x0, y0, x1, y1) of the pixels within 2/255 of
/// `want` (tier 2: a solid fill), or None.
fn bounds_of(shot: &Screenshot, want: u32) -> Option<[u32; 4]> {
    let mut found: Option<[u32; 4]> = None;
    for y in 0..shot.height {
        for x in 0..shot.width {
            if !near(pixel(shot, x, y), want, 2) {
                continue;
            }
            found = Some(match found {
                None => [x, y, x, y],
                Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
            });
        }
    }
    found
}

/// The bounds of a box are `want` within 1 device px (tier 1).
#[track_caller]
fn assert_bounds(got: Option<[u32; 4]>, want: [u32; 4], what: &str) {
    let got = got.unwrap_or_else(|| panic!("{what}: no pixel of that colour"));
    let close = |a: u32, b: u32| a.abs_diff(b) <= 1;
    assert!(
        got.iter().zip(want).all(|(&a, b)| close(a, b)),
        "{what}: bounds {got:?}, want {want:?} (±1 px)"
    );
}

/// How many errors of the screen say `needle`.
fn errors_with(screen: &Screen, needle: &str) -> usize {
    screen
        .errors()
        .iter()
        .filter(|error| error.message().contains(needle))
        .count()
}

/// The first frame after `after` of a pixel export, driving the screen
/// between the waits.
#[track_caller]
fn next_frame(screen: &Screen, share: &DrawableShare, after: u64) -> DrawableFrame {
    let start = Instant::now();
    loop {
        screen.run_until_idle();
        match share.wait(after, Duration::from_millis(5)) {
            Ok(ShareUpdate::Frame(frame)) => return frame,
            #[allow(
                unreachable_patterns,
                reason = "the textures variant exists only with the gpu-interop feature"
            )]
            Ok(other) => panic!("a pixel export gave {other:?}"),
            Err(ShareError::Timeout) => {}
            Err(error) => panic!("the export failed: {error}"),
        }
        assert!(start.elapsed() < FRAME_WAIT, "no frame after {after}");
    }
}

/// `share.wait` on a thread of its own, while this (UI) thread drives the
/// screen until the thread answers.
#[track_caller]
fn wait_on_thread(
    screen: &Screen,
    share: &DrawableShare,
    after: u64,
    timeout: Duration,
) -> Result<DrawableFrame, ShareError> {
    let (send, receive) = mpsc::channel();
    let share = share.clone();
    std::thread::spawn(move || {
        let result = match share.wait(after, timeout) {
            Ok(ShareUpdate::Frame(frame)) => Ok(frame),
            #[allow(
                unreachable_patterns,
                reason = "the textures variant exists only with the gpu-interop feature"
            )]
            Ok(other) => panic!("a pixel export gave {other:?}"),
            Err(error) => Err(error),
        };
        send.send(result).expect("the test waits for the answer");
    });
    let start = Instant::now();
    loop {
        screen.run_until_idle();
        match receive.recv_timeout(Duration::from_millis(5)) {
            Ok(result) => return result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                assert!(start.elapsed() < timeout + FRAME_WAIT, "the thread waits")
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => panic!("the waiting thread ended"),
        }
    }
}

/// The pixels of a frame of a pixel export.
fn pixels_of(frame: &DrawableFrame) -> &Pixels {
    match frame {
        DrawableFrame::Pixels { pixels, .. } => pixels,
        other => panic!("a pixel export gave {other:?}"),
    }
}

/// The straight-alpha RGBA of the pixel at (x, y) of `pixels`.
fn rgba_at(pixels: &Pixels, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * pixels.width + x) * 4) as usize;
    let bytes: &[u8] = &pixels.rgba;
    [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]
}

/// A viewport that exports its drawable: a box of 40 × 20 in `colour`
/// (scale 1 once asked). The colour is the test's handle on the content.
fn export_app() -> (Screen, (Drawable, Signal<u32>)) {
    mount(|| {
        let card = drawable();
        let colour = signal(RED);
        let view = viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
            .drawable(card, move || {
                node().style(move || style().size(40, 20).background(rgb(colour.get())))
            })
            .style(style().size(120, 80));
        ((card, colour), view)
    })
}

/// Tier 1 (geometry): the drawn box is the drawable's box moved by the
/// layer's transform. A translation, then a rotation about the canvas origin,
/// then the translation of the composed transform.
#[test]
fn drawable_is_drawn_where_the_canvas_draws_it() {
    let (screen, rotated) = mount(|| {
        let card = drawable();
        let rotated = signal(false);
        let view = stage(move |d, _| {
            let transform = if rotated.get() {
                Transform::new().translate(60.0, 0.0).rotate(90.0)
            } else {
                Transform::new().translate(30.0, 5.0)
            };
            d.layer(Layer::new().transform(transform), |d| {
                d.drawable(card, Point::new(AX, AY));
            });
        })
        .drawable(card, red_box);
        (rotated, view)
    });
    // 40 × 20 at (50, 15): the box moved by (30, 5).
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [50, 15, 89, 34],
        "translated",
    );
    rotated.set(true);
    screen.run_until_idle();
    // Rotated by 90°: 20 wide and 40 tall, moved by (60, 0) after the rotation.
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [30, 20, 49, 59],
        "rotated 90 degrees",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Tier 1 (geometry): under `rotate_y(60)` a point of the z = 0 plane maps to
/// x / 2, so the 80 × 30 button projects to x 10..50, y 10..40. A click at the
/// unprojected box (x 55) misses, and a click at the projected centre hits.
#[test]
fn drawable_button_is_clickable_through_3d() {
    let (screen, pressed) = mount(|| {
        let card = drawable();
        let pressed = signal(false);
        let view = stage(move |d, _| {
            let layer = Layer::new().transform(Transform::new().rotate_y(60.0));
            d.layer(layer, |d| d.drawable(card, Point::new(AX, AY)));
        })
        .drawable(card, move || apply(pressed));
        (pressed, view)
    });
    click_stage(&screen, 55.0, 25.0);
    assert!(!pressed.get_untracked(), "outside the projection");
    click_stage(&screen, 30.0, 25.0);
    assert!(pressed.get_untracked(), "the projected centre presses");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Focus is a test of the drawable's content: Tab from nothing reaches the
/// button inside the drawable, which the role finds.
#[test]
fn drawable_content_is_focusable_and_findable() {
    let (screen, ()) = mount(|| {
        let card = drawable();
        let view = stage(move |d, _| d.drawable(card, Point::new(AX, AY)))
            .drawable(card, || column((text("Settings"), button("Apply"))));
        ((), view)
    });
    assert!(screen.get_by_role(Role::Button, "Apply").exists());
    screen.press_tab(false);
    assert!(
        screen.get_by_role(Role::Button, "Apply").is_focused(),
        "Tab moves focus into the drawable"
    );
}

/// Tier 1 (geometry): the accessibility box is the projection of the drawn
/// box: x and width halve under `rotate_y(60)`, the box at z = 0.
#[test]
fn drawable_a11y_bounds_are_projected() {
    let (screen, ()) = mount(|| {
        let card = drawable();
        let view = stage(move |d, _| {
            let layer = Layer::new().transform(Transform::new().rotate_y(60.0));
            d.layer(layer, |d| d.drawable(card, Point::new(AX, AY)));
        })
        .drawable(card, || button("Apply").style(style().size(80, 30)));
        ((), view)
    });
    let bounds = screen
        .get_by_role(Role::Button, "Apply")
        .accessibility_bounds()
        .expect("the drawn button has bounds");
    let (ox, oy) = origin_of(&screen, "stage");
    for (got, want) in [
        (bounds.x - ox, 10.0),
        (bounds.y - oy, 10.0),
        (bounds.width, 40.0),
        (bounds.height, 30.0),
    ] {
        assert!(
            (got - want).abs() <= 1.0,
            "bounds {bounds:?}, want x 10 y 10 w 40 h 30"
        );
    }
}

/// The mount of a drawable that is already mounted is refused and reported
/// once. After the first node unmounts, a later node can mount it. A canvas
/// that draws the drawable without hosting it reports that too.
#[test]
fn drawable_mounted_twice_reports() {
    let (screen, (first, third)) = mount(|| {
        let card = drawable();
        let first = signal(true);
        let third = signal(false);
        let view = column((
            show(first, move || {
                node()
                    .test_id("host-a")
                    .drawable(card, || node().test_id("content-a"))
            }),
            node()
                .test_id("host-b")
                .drawable(card, || node().test_id("content-b")),
            show(third, move || {
                node()
                    .test_id("host-c")
                    .drawable(card, || node().test_id("content-c"))
            }),
            canvas(move |d, _| d.drawable(card, Point::new(0.0, 0.0))).test_id("other"),
        ));
        ((first, third), view)
    });
    assert_eq!(errors_with(&screen, "already mounted"), 1);
    assert_eq!(errors_with(&screen, "not hosted by this node"), 1);
    first.set(false);
    screen.run_until_idle();
    third.set(true);
    screen.run_until_idle();
    assert!(screen.query_by_test_id("content-c").is_some());
    assert_eq!(
        errors_with(&screen, "already mounted"),
        1,
        "a mount after the unmount is not refused"
    );
}

/// Tier 2 (solid fill): a drawable drawn twice is painted at its first
/// position only, and the second draw is reported once.
#[test]
fn drawing_a_drawable_twice_reports() {
    let (screen, ()) = mount(|| {
        let card = drawable();
        let view = stage(move |d, _| {
            d.drawable(card, Point::new(AX, AY));
            d.drawable(card, Point::new(100.0, 60.0));
        })
        .drawable(card, red_box);
        ((), view)
    });
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [20, 10, 59, 29],
        "the first position only",
    );
    assert_eq!(errors_with(&screen, "drawn twice"), 1);
}

/// A drawable that no canvas draws is not hit; its node stays in the
/// accessibility tree with no bounds.
#[test]
fn unplaced_drawable_is_not_hit() {
    let (screen, pressed) = mount(|| {
        let card = drawable();
        let pressed = signal(false);
        let view = stage(|_, _| {}).drawable(card, move || apply(pressed));
        (pressed, view)
    });
    click_stage(&screen, AX + 40.0, AY + 15.0);
    assert!(!pressed.get_untracked(), "an unplaced drawable is not hit");
    let button = screen.get_by_role(Role::Button, "Apply");
    assert!(button.exists(), "the node stays in the tree");
    assert_eq!(button.accessibility_bounds(), None);
}

/// Changing `draw_over` alone keeps the placement of `draw`: the button is
/// still clickable and its accessibility bounds stay the same.
#[test]
fn draw_over_update_keeps_draw_placement() {
    let (screen, (pressed, tick)) = mount(|| {
        let card = drawable();
        let pressed = signal(false);
        let tick = signal(0u32);
        let view = stage(move |d, _| d.drawable(card, Point::new(AX, AY)))
            .draw_over(move |d, _| {
                let width = 4.0 + f64::from(tick.get());
                d.fill_rect(Rect::new(0.0, 0.0, width, 4.0), rgb(BLUE));
            })
            .drawable(card, move || apply(pressed));
        ((pressed, tick), view)
    });
    click_stage(&screen, 60.0, 25.0);
    assert!(pressed.get_untracked(), "clickable before the update");
    let before = screen
        .get_by_role(Role::Button, "Apply")
        .accessibility_bounds();
    pressed.set(false);
    tick.set(1);
    screen.run_until_idle();
    assert_eq!(
        screen
            .get_by_role(Role::Button, "Apply")
            .accessibility_bounds(),
        before,
        "the a11y bounds do not change"
    );
    click_stage(&screen, 60.0, 25.0);
    assert!(pressed.get_untracked(), "clickable after the update");
}

/// The same drawable in `draw` and `draw_over`: only the `draw` position is
/// painted and hit, and the second is reported once.
#[test]
fn drawable_twice_across_slots_uses_the_first() {
    let (screen, presses) = mount(|| {
        let card = drawable();
        let presses = signal(0u32);
        let view = stage(move |d, _| d.drawable(card, Point::new(AX, AY)))
            .draw_over(move |d, _| d.drawable(card, Point::new(100.0, 60.0)))
            .drawable(card, move || {
                red_button(move || presses.set(presses.get_untracked() + 1))
            });
        (presses, view)
    });
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [20, 10, 59, 29],
        "painted at the draw position only",
    );
    click_stage(&screen, 120.0, 70.0);
    assert_eq!(
        presses.get_untracked(),
        0,
        "no hit at the draw_over position"
    );
    click_stage(&screen, 40.0, 20.0);
    assert_eq!(presses.get_untracked(), 1, "hit at the draw position");
    assert_eq!(errors_with(&screen, "drawn twice"), 1);
}

/// Tier 2 (solid fill) and tier 1 (hit): a clip at x < 50 keeps the drawable
/// painted and hit at x 20..49 only.
#[test]
fn clipped_drawable_is_not_hit_outside_the_clip() {
    let (screen, presses) = mount(|| {
        let card = drawable();
        let presses = signal(0u32);
        let view = stage(move |d, _| {
            let clip = Layer::new().clip_rect(Rect::new(0.0, 0.0, 50.0, STAGE_H));
            d.layer(clip, |d| d.drawable(card, Point::new(AX, AY)));
        })
        .drawable(card, move || {
            red_button(move || presses.set(presses.get_untracked() + 1))
        });
        (presses, view)
    });
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [20, 10, 49, 29],
        "clipped at x 50",
    );
    click_stage(&screen, 55.0, 20.0);
    assert_eq!(presses.get_untracked(), 0, "outside the clip");
    click_stage(&screen, 30.0, 20.0);
    assert_eq!(presses.get_untracked(), 1, "inside the clip");
}

/// Tier 1 (geometry): `rotate_y(180)` with a translation of 200 maps x to
/// 200 - x, so the box moves to x 140..179. With backface Hidden the back is
/// neither drawn nor hit; with Visible it is both (the control).
#[test]
fn backface_hidden_drawable_is_not_hit() {
    let (screen, (hidden, presses)) = mount(|| {
        let card = drawable();
        let hidden = signal(true);
        let presses = signal(0u32);
        let view = stage(move |d, _| {
            let backface = if hidden.get() {
                BackfaceVisibility::Hidden
            } else {
                BackfaceVisibility::Visible
            };
            let transform = Transform::new().translate(200.0, 0.0).rotate_y(180.0);
            let layer = Layer::new()
                .transform(transform)
                .backface_visibility(backface);
            d.layer(layer, |d| d.drawable(card, Point::new(AX, AY)));
        })
        .drawable(card, move || {
            red_button(move || presses.set(presses.get_untracked() + 1))
        });
        ((hidden, presses), view)
    });
    assert_eq!(
        bounds_of(&stage_shot(&screen), RED),
        None,
        "the back is not drawn"
    );
    click_stage(&screen, 160.0, 20.0);
    assert_eq!(presses.get_untracked(), 0, "the back is not hit");
    hidden.set(false);
    screen.run_until_idle();
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [140, 10, 179, 29],
        "the control: visible back",
    );
    click_stage(&screen, 160.0, 20.0);
    assert_eq!(presses.get_untracked(), 1, "the control hits");
}

/// Tier 1 (geometry) and the compositor's rule for backface: the
/// inner rotation alone shows its back in a flat parent, so the drawable is
/// neither drawn nor hit. Inside a Preserve3d parent the two rotations
/// accumulate to the identity, so it is drawn and hit at its own box.
#[test]
fn nested_flat_backface_matches_compositor() {
    let (screen, (preserve, presses)) = mount(|| {
        let card = drawable();
        let preserve = signal(false);
        let presses = signal(0u32);
        let view = stage(move |d, _| {
            let style = if preserve.get() {
                TransformStyle::Preserve3d
            } else {
                TransformStyle::Flat
            };
            let turn = || Transform::new().rotate_y(180.0);
            let outer = Layer::new().transform(turn()).transform_style(style);
            let inner = Layer::new()
                .transform(turn())
                .backface_visibility(BackfaceVisibility::Hidden);
            d.layer(outer, |d| {
                d.layer(inner, |d| d.drawable(card, Point::new(AX, AY)));
            });
        })
        .drawable(card, move || {
            red_button(move || presses.set(presses.get_untracked() + 1))
        });
        ((preserve, presses), view)
    });
    assert_eq!(
        bounds_of(&stage_shot(&screen), RED),
        None,
        "flat: not drawn"
    );
    click_stage(&screen, 40.0, 20.0);
    assert_eq!(presses.get_untracked(), 0, "flat: not hit");
    preserve.set(true);
    screen.run_until_idle();
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [20, 10, 59, 29],
        "preserve-3d: drawn at its box",
    );
    click_stage(&screen, 40.0, 20.0);
    assert_eq!(presses.get_untracked(), 1, "preserve-3d: hit");
}

/// Two requests in a row: every frame that comes after has the newer
/// generation, and `latest` holds only that generation. A frame is sent
/// only when its value changed, so the content is recoloured
/// before each later frame is awaited.
#[test]
fn pixels_frames_of_old_generation_are_dropped() {
    let (screen, (card, colour)) = export_app();
    let share = card.share();
    share.set_scale(1.0);
    share.request_pixels();
    let first = next_frame(&screen, &share, 0);
    let older = first.generation();
    share.request_pixels();
    share.request_pixels();
    let mut after = first.number();
    let mut generation = None;
    for round in 0..3 {
        if round > 0 {
            colour.set(RED + round);
            screen.run_until_idle();
        }
        let frame = next_frame(&screen, &share, after);
        let expected = *generation.get_or_insert(frame.generation());
        assert_eq!(
            frame.generation(),
            expected,
            "a frame of an older generation"
        );
        after = frame.number();
    }
    // Each request starts the next generation (drawable_share.rs), so the
    // second request's generation is the newest one, two after the first.
    assert_eq!(
        generation,
        Some(older + 2),
        "the newest request's generation survives"
    );
    let latest = share.latest().expect("a frame of the newest generation");
    assert_eq!(Some(latest.generation()), generation);
}

/// A waiter whose cursor is the old generation's frame number gets the new
/// generation's frame without a timeout, and the numbers keep growing.
#[test]
fn wait_cursor_survives_generation_change() {
    let (screen, (card, _)) = export_app();
    let share = card.share();
    share.set_scale(1.0);
    share.request_pixels();
    let first = next_frame(&screen, &share, 0);
    share.request_pixels();
    let second = wait_on_thread(&screen, &share, first.number(), FRAME_WAIT)
        .expect("a frame of the new generation, not a timeout");
    assert_ne!(second.generation(), first.generation());
    assert!(
        second.number() > first.number(),
        "numbers grow across generations"
    );
}

/// Tier 2 (solid fill): the frame the export makes of the drawable reaches a
/// thread of its own, with the red box at its place.
#[test]
fn exported_drawable_pixels_reach_another_thread() {
    let (screen, (card, _)) = export_app();
    let share = card.share();
    share.set_scale(1.0);
    share.request_pixels();
    let frame =
        wait_on_thread(&screen, &share, 0, FRAME_WAIT).expect("a frame reaches the other thread");
    let pixels = pixels_of(&frame);
    assert_eq!((pixels.width, pixels.height), (40, 20));
    let [r, g, b, a] = rgba_at(pixels, 20, 10);
    assert!(
        near(u32::from(r), 255, 2) && g <= 2 && b <= 2 && a == 255,
        "the red box at its place: {:?}",
        [r, g, b, a]
    );
}

/// Setting the drawable's geometry (the identity: drawable box to viewport
/// px) makes the drawn button clickable at the viewport's mapped position;
/// before it, the click does not reach the button.
#[test]
fn exported_drawable_geometry_enables_hits() {
    let (screen, (pressed, card)) = mount(|| {
        let card = drawable();
        let pressed = signal(false);
        let view = viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
            .drawable(card, move || apply(pressed))
            .test_id("viewport")
            .style(style().size(200, 120));
        ((pressed, card), view)
    });
    let (ox, oy) = origin_of(&screen, "viewport");
    screen.click_at(ox + 30.0, oy + 25.0);
    assert!(!pressed.get_untracked(), "no geometry, no hit");
    card.share().set_geometry(Some(Matrix4::IDENTITY));
    screen.run_until_idle();
    screen.click_at(ox + 30.0, oy + 25.0);
    assert!(
        pressed.get_untracked(),
        "the mapped position hits the button"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Tier 1 (geometry): an exported drawable's hit and accessibility bounds
/// follow the geometry the app set. The geometry maps the drawable's box into
/// the host's padding box (its origin inside the border), and nothing else
/// moves it: without geometry the button has no bounds at all.
#[test]
fn exported_drawable_hit_and_bounds_follow_its_geometry() {
    let (screen, (pressed, card)) = mount(|| {
        let card = drawable();
        let pressed = signal(false);
        let view = viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
            .drawable(card, move || apply(pressed))
            .test_id("viewport")
            .style(style().size(200, 120).padding(16));
        ((pressed, card), view)
    });
    let (ox, oy) = origin_of(&screen, "viewport");
    assert!(
        screen
            .get_by_role(Role::Button, "Apply")
            .accessibility_bounds()
            .is_none(),
        "no geometry, no bounds"
    );
    card.share()
        .set_geometry(Some(Matrix4::translate(30.0, 10.0, 0.0)));
    screen.run_until_idle();
    let bounds = screen
        .get_by_role(Role::Button, "Apply")
        .accessibility_bounds()
        .expect("the drawn button has bounds");
    for (got, want) in [
        (bounds.x - ox, 30.0),
        (bounds.y - oy, 10.0),
        (bounds.width, 80.0),
        (bounds.height, 30.0),
    ] {
        assert!(
            (got - want).abs() <= 1.0,
            "bounds {bounds:?}, want x 30 y 10 w 80 h 30 in the window"
        );
    }
    screen.click_at(ox + 50.0, oy + 30.0);
    assert!(pressed.get_untracked(), "the drawn box is hit");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Tier 1 (geometry): a drawable under a Preserve3d layer whose inner layer
/// is pushed along z is placed through the whole 3D context and flattened
/// where it ends: the box at z = 200 under `rotate_y(60)` lands at
/// x = 200 sin 60 = 173 on the stage (a flattened inner layer would give 0).
#[test]
fn preserve_3d_drawable_is_placed_through_its_context() {
    let (screen, presses) = mount(|| {
        let card = drawable();
        let presses = signal(0u32);
        let view = stage(move |d, _| {
            let outer = Layer::new()
                .transform(Transform::new().rotate_y(60.0))
                .transform_style(TransformStyle::Preserve3d);
            let inner = Layer::new().transform(Transform::new().translate_3d(0.0, 0.0, 200.0));
            d.layer(outer, |d| {
                d.layer(inner, |d| d.drawable(card, Point::new(0.0, 0.0)));
            });
        })
        .drawable(card, move || {
            node()
                .role(Role::Generic)
                .style(style().size(40, 20).background(rgb(RED)))
                .on_press(move || presses.set(presses.get_untracked() + 1))
        });
        (presses, view)
    });
    assert_bounds(
        bounds_of(&stage_shot(&screen), RED),
        [173, 0, 192, 19],
        "preserve-3d: drawn through its context",
    );
    click_stage(&screen, 180.0, 10.0);
    assert_eq!(presses.get_untracked(), 1, "hit where it is drawn");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
