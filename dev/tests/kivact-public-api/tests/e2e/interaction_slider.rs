//! Role: Slider tracks, thumbs, ranges and their edits through the public API.
//!
//! `use_scalar(..).track(axis)` maps the pointer to the value (horizontal,
//! vertical, right to left, minus the thumb's length), each key press and
//! track drag is one `EditPhase` edit, and Escape, an engine Cancel and the
//! focus leaving the thumb restore the value from before the drag.
//! `use_range_scalar` keeps its two thumbs from crossing.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::{RangeThumb, Scalar, ScalarState, use_range_scalar, use_scalar};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::test::{MirrorMode, RenderOptions, Screen};
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::{MouseButton, PointerKind};

use EditPhase::{Begin, Cancel, Commit};

/// The phases an observer saw, each with the values at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

/// The track is 200 by 20 and the thumb 20 by 20, so the thumb's center
/// spans 180 pixels from 10 to 190.
const TRACK: f64 = 200.0;
const THUMB: f64 = 20.0;

fn real_window() -> RenderOptions {
    RenderOptions::from_env().mirror(MirrorMode::Required)
}

fn range() -> ScalarRange {
    ScalarRange::new(0.0, 100.0).expect("invariant: finite range")
}

fn observer(value: Signal<f64>, log: &Log) -> impl Fn(EditPhase) + 'static {
    let log = log.clone();
    move |phase| log.borrow_mut().push((phase, value.get_untracked()))
}

/// A thumb drawn at its fraction along the track, horizontal or vertical
/// (from the bottom); right to left, it runs from the right edge.
fn thumb(
    name: &str,
    part: kivact::primitive::Part<ScalarState>,
    axis: Orientation,
    rtl: bool,
) -> Node {
    let state = part.state();
    node().part(part).name(name.to_string()).style(move || {
        let at = state.get().fraction * (TRACK - THUMB);
        let s = style().size(THUMB, THUMB).position(Position::Absolute);
        match (axis, rtl) {
            (Orientation::Vertical, _) => s.left(0).top(TRACK - THUMB - at),
            (_, true) => s.top(0).left(TRACK - THUMB - at),
            _ => s.top(0).left(at),
        }
    })
}

fn slider(
    name: &str,
    value: Signal<f64>,
    log: &Log,
    axis: Orientation,
    rtl: bool,
) -> (Scalar, Node) {
    let scalar = use_scalar(value, range()).on_edit(observer(value, log));
    let (width, height) = match axis {
        Orientation::Vertical => (THUMB, TRACK),
        _ => (TRACK, THUMB),
    };
    let track = node()
        .part(scalar.track(axis))
        .test_id("track")
        .style(style().size(width, height))
        .child(thumb(name, scalar.thumb(), axis, rtl));
    (scalar, track)
}

fn track_rect(screen: &Screen) -> kivact::draw::Rect {
    screen.get_by_test_id("track").rect().expect("laid out")
}

#[test]
fn a_track_press_moves_the_value_focuses_the_thumb_and_a_drag_is_one_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(0.0);
        let (_, view) = slider("Volume", value, &seen, Orientation::Horizontal, false);
        (
            value,
            column((
                view,
                node()
                    .role(Role::Button)
                    .name("Other")
                    .focusable(true)
                    .test_id("other")
                    .style(style().size(40, 20)),
            )),
        )
    });
    let rect = track_rect(&screen);
    // The center of the thumb at 50 is 10 + 90 pixels from the left edge.
    screen.mouse_move(rect.x + 100.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 50.0);
    let thumb = screen.get_by_role(Role::Slider, "Volume");
    assert!(thumb.is_focused(), "the press focuses the thumb");
    screen.mouse_move(rect.x + 190.0, rect.y + 10.0);
    assert_eq!(value.get_untracked(), 100.0);
    screen.mouse_move(rect.x - 40.0, rect.y + 10.0);
    assert_eq!(value.get_untracked(), 0.0, "past the edge clamps");
    screen.mouse_move(rect.x + 55.0, rect.y + 10.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 25.0);
    assert_eq!(*log.borrow(), [(Begin, 0.0), (Commit, 25.0)]);
    log.borrow_mut().clear();
    // A press on the thumb keeps the pointer's distance from its center.
    let at = thumb.rect().expect("laid out");
    screen.mouse_move(at.x + 4.0, at.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(
        value.get_untracked(),
        25.0,
        "grabbing the thumb does not move it"
    );
    screen.mouse_move(at.x + 4.0 + 18.0, at.y + 10.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 35.0);
    assert_eq!(*log.borrow(), [(Begin, 25.0), (Commit, 35.0)]);
    log.borrow_mut().clear();
    // Each key press is its own edit.
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Home);
    assert_eq!(
        *log.borrow(),
        [(Begin, 35.0), (Commit, 36.0), (Begin, 36.0), (Commit, 0.0)]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_a_canceled_pointer_and_the_focus_leaving_restore_the_value() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(20.0);
        let (_, view) = slider("Volume", value, &seen, Orientation::Horizontal, false);
        (
            value,
            column((
                view,
                node()
                    .role(Role::Button)
                    .name("Other")
                    .focusable(true)
                    .test_id("other")
                    .style(style().size(40, 20)),
            )),
        )
    });
    let rect = track_rect(&screen);
    screen.mouse_move(rect.x + 100.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(rect.x + 150.0, rect.y + 10.0);
    assert!(value.get_untracked() > 50.0);
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 20.0, "Escape writes the start back");
    screen.mouse_move(rect.x + 180.0, rect.y + 10.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        value.get_untracked(),
        20.0,
        "the canceled drag stops following"
    );
    assert_eq!(*log.borrow(), [(Begin, 20.0), (Cancel, 20.0)]);
    log.borrow_mut().clear();

    let contact = screen.contact_down(PointerKind::Touch, rect.x + 100.0, rect.y + 10.0);
    screen.contact_move(contact, rect.x + 190.0, rect.y + 10.0);
    assert_eq!(value.get_untracked(), 100.0);
    screen.contact_cancel(contact);
    assert_eq!(
        value.get_untracked(),
        20.0,
        "an engine Cancel writes the start back"
    );
    assert_eq!(*log.borrow(), [(Begin, 20.0), (Cancel, 20.0)]);
    log.borrow_mut().clear();

    screen.mouse_move(rect.x + 100.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 50.0);
    screen.get_by_test_id("other").focus();
    assert_eq!(
        value.get_untracked(),
        20.0,
        "the focus leaving the thumb cancels"
    );
    assert_eq!(*log.borrow(), [(Begin, 20.0), (Cancel, 20.0)]);
    screen.mouse_up(MouseButton::Primary);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_range_keeps_its_thumbs_apart_and_moves_the_nearest_one() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (low, high)) = mount_with(real_window(), move || {
        let low = signal(20.0);
        let high = signal(60.0);
        let price = use_range_scalar(low, high, range()).on_edit(observer(low, &seen));
        let track = node()
            .part(price.track(Orientation::Horizontal))
            .test_id("track")
            .style(style().size(TRACK, THUMB))
            .child((
                thumb(
                    "Low",
                    price.thumb(RangeThumb::Low),
                    Orientation::Horizontal,
                    false,
                ),
                thumb(
                    "High",
                    price.thumb(RangeThumb::High),
                    Orientation::Horizontal,
                    false,
                ),
            ));
        ((low, high), track)
    });
    let low_thumb = screen.get_by_role(Role::Slider, "Low");
    let high_thumb = screen.get_by_role(Role::Slider, "High");
    assert_eq!(low_thumb.value_range(), (Some(0.0), Some(60.0), Some(20.0)));
    assert_eq!(
        high_thumb.value_range(),
        (Some(20.0), Some(100.0), Some(60.0))
    );
    low_thumb.focus();
    screen.press_key(Key::End);
    assert_eq!(
        low.get_untracked(),
        60.0,
        "the low end stops at the high one"
    );
    screen.press_key(Key::Home);
    high_thumb.focus();
    screen.press_key(Key::Home);
    assert_eq!(
        high.get_untracked(),
        0.0,
        "the high end stops at the low one"
    );
    assert_eq!(low.get_untracked(), 0.0);
    screen.press_key(Key::End);
    let rect = track_rect(&screen);
    // Nearest to 80: the high thumb.
    screen.mouse_move(rect.x + 10.0 + 144.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!((low.get_untracked(), high.get_untracked()), (0.0, 80.0));
    assert!(high_thumb.is_focused(), "the moved thumb takes the focus");
    // Dragging the high thumb past the low one stops on it.
    screen.mouse_move(rect.x - 30.0, rect.y + 10.0);
    assert_eq!((low.get_untracked(), high.get_untracked()), (0.0, 0.0));
    screen.press_key(Key::Escape);
    assert_eq!(
        (low.get_untracked(), high.get_untracked()),
        (0.0, 100.0),
        "Escape writes both ends back"
    );
    screen.mouse_up(MouseButton::Primary);
    // Nearest to 10: the low thumb.
    screen.mouse_move(rect.x + 10.0 + 18.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!((low.get_untracked(), high.get_untracked()), (10.0, 100.0));
    assert!(low_thumb.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
