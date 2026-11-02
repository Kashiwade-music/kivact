//! Role: Tests for extension primitives and custom rendering.
//!
//! Extension primitives (`kivact::primitive`, experimental) on real
//! windows: the layout sizes a primitive by its measure function and
//! places its siblings by that size, new content changes that geometry,
//! its draw layer runs again when its size changes,
//! it takes focus, keys, the pointer and accessibility actions and is
//! found by role and test id like any element, and unmounting it drops
//! its measure function.

use kivact::primitive::NodeExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::draw::{Point, Rect, Size};
use kivact::prelude::*;
use kivact::primitive::{Constraints, primitive};
use kivact::style::Align;
use kivact::test::Screen;
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent};

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// A row of `n` cells of 10 × 8 px: the primitive is as wide as its
/// content says, its sibling follows it, and a new content moves the
/// sibling. A repaint, time and equal content preserve both boxes.
#[test]
fn a_primitive_is_laid_out_at_its_measured_size_and_new_content_relayouts() {
    let (screen, (n, tint)) = mount(move || {
        let n = signal(3u32);
        let tint = signal(false);
        let view = row((
            primitive()
                .measure(n, move |n: &u32, _: Constraints| {
                    Size::new(*n as f32 * 10.0, 8.0)
                })
                .draw(move |d, size| {
                    let c = if tint.get() {
                        ColorToken::Accent
                    } else {
                        ColorToken::Border
                    };
                    d.fill_rect(size.to_rect(), c);
                })
                .test_id("cells"),
            node()
                .test_id("after")
                .style(style().size(10, 10).shrink(0.0)),
        ))
        .style(style().align_items(Align::Start));
        ((n, tint), view)
    });
    let (cells, after) = (rect(&screen, "cells"), rect(&screen, "after"));
    assert_eq!((cells.width, cells.height), (30.0, 8.0));
    assert_eq!(
        after.x,
        cells.x + 30.0,
        "the sibling follows the measured width"
    );
    tint.set(true);
    screen.run_until_idle();
    assert_eq!(rect(&screen, "cells"), cells);
    assert_eq!(rect(&screen, "after"), after);
    screen.advance(Duration::from_millis(100));
    assert_eq!(rect(&screen, "cells"), cells);
    assert_eq!(rect(&screen, "after"), after);
    n.set(3);
    screen.run_until_idle();
    assert_eq!(rect(&screen, "cells"), cells);
    assert_eq!(rect(&screen, "after"), after);

    n.set(5);
    screen.run_until_idle();
    assert_eq!(rect(&screen, "cells").width, 50.0);
    assert_eq!(
        rect(&screen, "after").x,
        cells.x + 50.0,
        "and moves the sibling"
    );
}

/// A primitive that fills the width it is given draws again at its new
/// size when the window is resized.
#[test]
fn a_primitives_draw_layer_runs_again_when_its_size_changes() {
    let drawn: Rc<RefCell<Vec<Size>>> = Rc::new(RefCell::new(Vec::new()));
    let log = drawn.clone();
    let (screen, ()) = mount(move || {
        let view = column((primitive()
            .measure((), |_: &(), c: Constraints| {
                Size::new(c.max_width().unwrap_or(100.0), 20.0)
            })
            .draw(move |d, size| {
                log.borrow_mut().push(size);
                d.fill_rect(size.to_rect(), ColorToken::Accent);
            })
            .test_id("bar"),));
        ((), view)
    });
    let first = rect(&screen, "bar");
    assert_eq!(
        drawn.borrow().last().copied(),
        Some(Size::new(first.width, 20.0))
    );
    assert!(screen.get_by_test_id("bar").scene_len() > 0, "it drew");
    screen.resize(first.width - 100.0, 300.0);
    screen.run_until_idle();
    let second = rect(&screen, "bar");
    assert_eq!(
        second.width,
        first.width - 100.0,
        "stretched across the new width"
    );
    assert_eq!(
        drawn.borrow().last().copied(),
        Some(Size::new(second.width, 20.0)),
        "drawn again at the new size"
    );
}

/// A primitive is a focusable, accessible slider found by role and by test
/// id; Tab focuses it, keys and the pointer (in its own coordinates) reach
/// its handlers and assistive technology invokes its actions.
#[test]
fn a_primitive_takes_focus_input_and_accessibility_like_any_element() {
    let pressed_at = Rc::new(Cell::new(None));
    let seen = pressed_at.clone();
    let (screen, level) = mount(move || {
        let level = signal(1u32);
        let view = column((
            {
                let label = ("before").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
            primitive()
                .measure((), |_: &(), _: Constraints| Size::new(100.0, 20.0))
                .draw(move |d, size| {
                    let w = size.width * level.get() as f32 / 10.0;
                    d.fill_rect(Rect::new(0.0, 0.0, w, size.height), ColorToken::Accent);
                })
                .role(Role::Slider)
                .name("level")
                .value_text(move || level.get().to_string())
                .focusable(true)
                .test_id("level")
                .on_key(move |e: KeyEvent| match e.key {
                    Key::ArrowRight => {
                        level.update(|l| *l += 1);
                        Handled::Yes
                    }
                    _ => Handled::No,
                })
                .on_pointer(move |e: PointerEvent| {
                    if matches!(e.phase, kivact::view::PointerPhase::Down(_)) {
                        seen.set(Some(e.pos));
                        level.set((e.pos.x / 10.0) as u32);
                    }
                })
                .on_a11y(A11yAction::Increment, move || level.update(|l| *l += 1)),
        ));
        (level, view)
    });
    let slider = screen.get_by_role(Role::Slider, "level");
    assert_eq!(slider.value().as_deref(), Some("1"));
    assert!(slider.a11y_actions().contains(&A11yAction::Increment));
    screen.get_by_role(Role::Button, "before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_test_id("level").is_focused(), "a tab stop");
    screen.press_key(Key::ArrowRight);
    assert_eq!(level.get_untracked(), 2);
    slider.a11y(A11yAction::Increment);
    assert_eq!(level.get_untracked(), 3);
    assert_eq!(slider.value().as_deref(), Some("3"));
    let r = slider.rect().expect("laid out");
    screen.click_at(r.x + 75.0, r.y + 10.0);
    assert_eq!(pressed_at.get(), Some(Point::new(75.0, 10.0)), "node-local");
    assert_eq!(level.get_untracked(), 7);
}
