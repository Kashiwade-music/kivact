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
use kivact::interaction::{
    DragCoordinates, DragUpdate, EditSession, RangeThumb, Scalar, ScalarState, use_drag,
    use_range_scalar, use_scalar,
};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::{
    Handled, KeyEvent, MouseButton, PointerKind, ReadingDirection, provide_direction,
};

use EditPhase::{Begin, Cancel, Commit};

/// The phases an observer saw, each with the values at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

/// The track is 200 by 20 and the thumb 20 by 20, so the thumb's center
/// spans 180 pixels from 10 to 190.
const TRACK: f32 = 200.0;
const THUMB: f32 = 20.0;

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
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
        let at = state.get().fraction as f32 * (TRACK - THUMB);
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
fn a_vertical_track_grows_upward_and_a_right_to_left_one_from_the_right() {
    let log = Log::default();
    let (seen, seen_rtl) = (log.clone(), log.clone());
    let (screen, (up, back)) = mount_with(real_window(), move || {
        let up = signal(0.0);
        let back = signal(0.0);
        let (_, vertical) = slider("Up", up, &seen, Orientation::Vertical, false);
        let mirrored = provide_direction(ReadingDirection::Rtl, move || {
            let (_, view) = slider("Back", back, &seen_rtl, Orientation::Horizontal, true);
            view.test_id("rtl")
        });
        ((up, back), row((vertical, mirrored)))
    });
    let rect = track_rect(&screen);
    // 10 + 45 pixels above the bottom is a quarter of the way up.
    screen.mouse_move(rect.x + 10.0, rect.y + TRACK - 55.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(up.get_untracked(), 25.0);
    let vertical = screen.get_by_role(Role::Slider, "Up");
    vertical.focus();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(
        up.get_untracked(),
        24.0,
        "a vertical slider keeps the arrows"
    );

    let rtl = screen.get_by_test_id("rtl").rect().expect("laid out");
    // 10 + 45 pixels from the right edge is a quarter.
    screen.mouse_move(rtl.x + TRACK - 55.0, rtl.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(back.get_untracked(), 25.0);
    screen.press_key(Key::ArrowLeft);
    assert_eq!(
        back.get_untracked(),
        26.0,
        "right to left, ArrowLeft increases"
    );
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::ArrowRight);
    assert_eq!(back.get_untracked(), 24.0);
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

#[test]
fn a_track_that_keeps_the_focus_moves_the_nearest_unfocusable_thumb() {
    let (screen, (low, high)) = mount_with(real_window(), || {
        let low = signal(20.0);
        let high = signal(60.0);
        let price = use_range_scalar(low, high, range());
        let unfocusable = |name: &str, thumb: RangeThumb| {
            node()
                .part(price.thumb(thumb))
                .focusable(false)
                .name(name.to_string())
                .style(style().size(THUMB, THUMB).position(Position::Absolute))
        };
        let track = node()
            .part(price.track(Orientation::Horizontal))
            .focusable(true)
            .name("Price")
            .test_id("track")
            .style(style().size(TRACK, THUMB))
            .child((
                unfocusable("Low", RangeThumb::Low),
                unfocusable("High", RangeThumb::High),
            ));
        ((low, high), track)
    });
    let track = screen.get_by_test_id("track");
    let rect = track_rect(&screen);
    // Nearest to 80: the high thumb, which cannot take the focus.
    screen.mouse_move(rect.x + 10.0 + 144.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!((low.get_untracked(), high.get_untracked()), (20.0, 80.0));
    assert!(track.is_focused(), "the track keeps the focus");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_drag_given_an_edit_session_begins_commits_and_cancels_it() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(10.0);
        let origin = signal(0.0);
        let edit = EditSession::new(move |start| value.set(start)).on_edit(observer(value, &seen));
        let source = use_drag::<()>(DragCoordinates::Window)
            .edit(edit, move || value.get_untracked())
            .on_start(move |_| origin.set(value.get_untracked()))
            .on_update(move |update: DragUpdate<()>| {
                value.set(origin.get_untracked() + f64::from(update.delta.x));
            })
            .source(());
        let handle = node()
            .part(source)
            .name("Handle")
            .test_id("handle")
            .focusable(true)
            .style(style().size(40, 40));
        (value, handle)
    });
    let rect = screen.get_by_test_id("handle").rect().expect("laid out");
    let (x, y) = (rect.x + 20.0, rect.y + 20.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 30.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 40.0)]);
    log.borrow_mut().clear();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 50.0, y);
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 40.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 40.0), (Cancel, 40.0)]);
    log.borrow_mut().clear();
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_move(contact, x + 5.0, y);
    screen.contact_cancel(contact);
    assert_eq!(value.get_untracked(), 40.0);
    assert_eq!(*log.borrow(), [(Begin, 40.0), (Cancel, 40.0)]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_scrub_is_an_edit_of_the_scalar_and_escape_on_any_source_of_it_cancels() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, value) = mount_with(real_window(), move || {
        let value = signal(10.0);
        let origin = signal(0.0);
        let scalar = use_scalar(value, range()).on_edit(observer(value, &seen));
        let scrub = scalar
            .scrub(use_drag::<u8>(DragCoordinates::Window))
            .on_start(move |_| origin.set(value.get_untracked()))
            .on_update(move |update: DragUpdate<u8>| {
                scalar.set(origin.get_untracked() + f64::from(update.delta.x));
            });
        let field = node()
            .part(scrub.source(0))
            .role(Role::Slider)
            .name("Field")
            .test_id("field")
            .focusable(true)
            .on_key(move |event: KeyEvent| {
                if event.key == Key::Backspace {
                    scalar.once(|| scalar.set(0.0));
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .style(style().size(60, 40))
            .child(
                node()
                    .part(scrub.source(1))
                    .test_id("grip")
                    .style(style().size(10, 10)),
            );
        (value, field)
    });
    let field = screen.get_by_test_id("field");
    field.focus();
    let grip = screen.get_by_test_id("grip").rect().expect("laid out");
    let (x, y) = (grip.x + 5.0, grip.y + 5.0);
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 30.0, y);
    assert_eq!(value.get_untracked(), 40.0);
    assert!(field.is_focused(), "the focus is on the other source");
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 10.0, "the value before the press");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Cancel, 10.0)]);
    log.borrow_mut().clear();
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 30.0)]);
    log.borrow_mut().clear();
    field.press_key(Key::Backspace);
    assert_eq!(value.get_untracked(), 0.0);
    assert_eq!(
        *log.borrow(),
        [(Begin, 30.0), (Commit, 0.0)],
        "a write of the component's own key is one edit"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The range and the two ends of a range slider.
type RangeState = (Signal<ScalarRange>, Signal<f64>, Signal<f64>);

/// A range slider over a reactive range, with its thumbs made in the order
/// `low_first` says.
fn range_slider(low_first: bool) -> (Screen, RangeState) {
    mount_with(real_window(), move || {
        let bounds = signal(range());
        let (low, high) = (signal(20.0), signal(80.0));
        let price = use_range_scalar(low, high, bounds);
        let (first, second) = if low_first {
            let first = thumb(
                "Low",
                price.thumb(RangeThumb::Low),
                Orientation::Horizontal,
                false,
            );
            let second = thumb(
                "High",
                price.thumb(RangeThumb::High),
                Orientation::Horizontal,
                false,
            );
            (first, second)
        } else {
            let second = thumb(
                "High",
                price.thumb(RangeThumb::High),
                Orientation::Horizontal,
                false,
            );
            let first = thumb(
                "Low",
                price.thumb(RangeThumb::Low),
                Orientation::Horizontal,
                false,
            );
            (first, second)
        };
        let track = node()
            .part(price.track(Orientation::Horizontal))
            .style(style().size(TRACK, THUMB))
            .child((first, second));
        ((bounds, low, high), track)
    })
}

#[test]
fn a_moving_range_pulls_both_ends_in_without_crossing_whatever_order_the_thumbs_were_made() {
    for low_first in [true, false] {
        let (screen, (bounds, low, high)) = range_slider(low_first);
        let new_range = |min: f64, max: f64| ScalarRange::new(min, max).expect("finite range");
        let ends = || (low.get_untracked(), high.get_untracked());
        // Up, past both ends: both land on the nearest bound.
        bounds.set(new_range(90.0, 100.0));
        screen.run_until_idle();
        assert_eq!(ends(), (90.0, 90.0), "up, low_first={low_first}");
        // Down, past both ends again.
        bounds.set(new_range(0.0, 100.0));
        low.set(20.0);
        high.set(80.0);
        bounds.set(new_range(0.0, 10.0));
        screen.run_until_idle();
        assert_eq!(ends(), (10.0, 10.0), "down, low_first={low_first}");
        // Past one end only: the other stays.
        bounds.set(new_range(0.0, 100.0));
        low.set(20.0);
        high.set(80.0);
        bounds.set(new_range(0.0, 50.0));
        screen.run_until_idle();
        assert_eq!(ends(), (20.0, 50.0), "one end, low_first={low_first}");
        bounds.set(new_range(30.0, 100.0));
        screen.run_until_idle();
        assert_eq!(ends(), (30.0, 50.0), "other end, low_first={low_first}");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
