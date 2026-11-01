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
use std::rc::{Rc, Weak};
use std::time::Duration;

use crate::support::mount;
use kivact::draw::{Point, Rect, Size};
use kivact::prelude::*;
use kivact::primitive::{Constraints, NodeRefExt, primitive};
use kivact::style::Align;
use kivact::test::Screen;
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent, error_boundary};

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// Public component focus actions cannot schedule restoration from a memo.
#[test]
fn component_focus_restoration_rejects_pure_computations() {
    let (screen, (restore_token, restore_inside)) = mount(|| {
        let target = node_ref();
        let token = var(None);
        on_mount(move || token.set(target.mounted_node()));
        let restore_token = signal(false);
        let restore_inside = signal(false);
        let token_value = memo(move || {
            if restore_token.get() {
                token.get().expect("target mounted").restore_focus();
            }
            "token".to_string()
        });
        let inside_value = memo(move || {
            if restore_inside.get() {
                target.focus_first_inside();
            }
            "inside".to_string()
        });
        (
            (restore_token, restore_inside),
            column((
                node()
                    .test_id("keeper")
                    .role(Role::Button)
                    .name("Keeper")
                    .focusable(true)
                    .style(style().size(80, 24)),
                node()
                    .node_ref(target)
                    .role(Role::Button)
                    .name("Target")
                    .focusable(true)
                    .style(style().size(80, 24)),
                text(token_value),
                text(inside_value),
            )),
        )
    });
    screen.get_by_test_id("keeper").focus();
    assert!(screen.errors().is_empty());
    for trigger in [restore_token, restore_inside] {
        trigger.set(true);
        screen.run_until_idle();
        assert!(screen.get_by_test_id("keeper").is_focused());
    }
    let errors = screen.errors();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors.iter().all(|error| {
            *error.kind() == kivact::diagnostics::ErrorKind::WriteInPureContext
                && error.location().is_some_and(|at| at.file() == file!())
        }),
        "{errors:?}"
    );
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
                    if e.phase == kivact::view::PointerPhase::Down {
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

/// Unmounting a primitive drops its measure function (and what it
/// captured): the engine keeps nothing of a removed leaf.
#[test]
fn an_unmounted_primitive_drops_its_measure_function() {
    let token = Rc::new(());
    let weak: Weak<()> = Rc::downgrade(&token);
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let view = column((show(shown, move || {
            let held = weak.upgrade();
            primitive()
                .measure(1u32, move |n: &u32, _: Constraints| {
                    let _ = &held;
                    Size::new(10.0, *n as f32 * 12.0)
                })
                .test_id("leaf")
        }),));
        (shown, view)
    });
    assert_eq!(rect(&screen, "leaf").height, 12.0);
    assert_eq!(Rc::strong_count(&token), 2, "held by the mounted leaf");
    shown.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_test_id("leaf").is_none());
    assert_eq!(Rc::strong_count(&token), 1, "dropped with the leaf");
}

/// A panic in a measure function sizes the primitive 0 × 0 and is reported
/// once to the nearest error boundary; the App keeps running.
#[test]
fn a_panicking_measure_function_is_reported_to_the_boundary() {
    let errors: Rc<RefCell<Vec<UiError>>> = Rc::new(RefCell::new(Vec::new()));
    let seen = errors.clone();
    let (screen, ()) = mount(move || {
        let view = column((
            error_boundary(|| {
                primitive()
                    .measure(1u32, |_: &u32, _: Constraints| -> Size {
                        panic!("cannot measure")
                    })
                    .test_id("broken")
            })
            .on_error(move |e| seen.borrow_mut().push(e)),
            text("still here"),
        ));
        ((), view)
    });
    screen.run_until_idle();
    screen.get_by_text("still here");
    let errors = errors.borrow();
    assert_eq!(errors.len(), 1, "reported once: {errors:?}");
    assert!(
        errors[0].to_string().contains("cannot measure"),
        "{}",
        errors[0]
    );
    let broken = rect(&screen, "broken");
    assert_eq!(broken.height, 0.0, "measures as 0 high");
}

/// A measure function runs pure and untracked: a signal write in it is
/// rejected (`WriteInPureContext`, reported once) and does not happen, and
/// the primitive measures as 0 high (its width is stretched by the column).
#[test]
fn a_measure_function_that_writes_a_signal_is_rejected_and_reported_once() {
    let (screen, count) = mount(|| {
        let count = signal(0);
        let view = primitive()
            .measure(1u32, move |_: &u32, _: Constraints| {
                count.set(5);
                Size::new(30.0, 10.0)
            })
            .test_id("writer");
        (count, view)
    });
    screen.run_until_idle();
    assert_eq!(count.get(), 0, "the write did not happen");
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "reported once: {errors:?}");
    assert_eq!(
        *errors[0].kind(),
        kivact::diagnostics::ErrorKind::WriteInPureContext
    );
    assert!(errors[0].location().is_some_and(|at| at.file() == file!()));
    let rect = rect(&screen, "writer");
    assert_eq!(rect.height, 0.0, "measures as 0 high");
}

/// The error of a panicking measure function reaches the boundary's
/// handlers outside the engine's layout: `on_error` reads a node's props
/// through the engine (`why`) and writes a signal, and the fallback
/// replaces the subtree, all without a further error.
#[test]
fn a_measure_panic_handler_may_read_the_tree_and_swap_the_subtree() {
    let labels: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let seen = labels.clone();
    let (screen, failures) = mount(move || {
        let failures = signal(0u32);
        let label = node_ref();
        let view = column((
            text(move || format!("failures: {}", failures.get())).node_ref(label),
            error_boundary(|| {
                primitive()
                    .measure(1u32, |_: &u32, _: Constraints| -> Size {
                        panic!("cannot measure")
                    })
                    .test_id("broken")
            })
            .on_error(move |_| {
                let why = kivact::diagnostics::why(&label).expect("mounted");
                seen.borrow_mut().push(why.label);
                failures.update(|n| *n += 1);
            })
            .fallback(|e, _| {
                let measured = e.to_string().contains("cannot measure");
                text(if measured {
                    "fallback shown"
                } else {
                    "other error"
                })
            }),
        ));
        (failures, view)
    });
    screen.run_until_idle();
    assert_eq!(failures.get_untracked(), 1, "on_error ran once");
    assert_eq!(
        labels.borrow().len(),
        1,
        "the handler read the label's props"
    );
    assert!(
        labels.borrow()[0].contains("failures: 0"),
        "{:?}",
        labels.borrow()
    );
    assert!(screen.query_by_test_id("broken").is_none(), "swapped out");
    screen.get_by_text("failures: 1");
    screen.get_by_text("fallback shown");
    assert!(
        screen.errors().is_empty(),
        "no error reached the default sink: {:?}",
        screen.errors()
    );
}

/// What a measure function holds is dropped where user code may run: a
/// content whose `Drop` reads the tree through the engine (`why`) is
/// dropped without a borrow error, when a new content replaces it and
/// when the primitive unmounts.
#[test]
fn a_measure_content_is_dropped_outside_the_engines_borrow() {
    #[derive(Clone)]
    struct Probe {
        n: u32,
        label: NodeRef,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl PartialEq for Probe {
        fn eq(&self, other: &Self) -> bool {
            self.n == other.n
        }
    }
    impl Drop for Probe {
        fn drop(&mut self) {
            if let Some(why) = kivact::diagnostics::why(&self.label) {
                self.log.borrow_mut().push(why.label);
            }
        }
    }
    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let dropped = log.clone();
    let (screen, (n, shown)) = mount(move || {
        let n = signal(1u32);
        let shown = signal(true);
        let label = node_ref();
        let log = dropped.clone();
        let content = move || Probe {
            n: n.get(),
            label,
            log: log.clone(),
        };
        let view = column((
            text("label").node_ref(label),
            show(shown, move || {
                primitive()
                    .measure(content.clone(), |p: &Probe, _: Constraints| {
                        Size::new(10.0, p.n as f32 * 8.0)
                    })
                    .test_id("probe")
            }),
        ));
        ((n, shown), view)
    });
    assert_eq!(rect(&screen, "probe").height, 8.0);
    n.set(2);
    screen.run_until_idle();
    assert_eq!(rect(&screen, "probe").height, 16.0);
    shown.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_test_id("probe").is_none());
    assert!(
        !log.borrow().is_empty(),
        "the contents were dropped and read the tree"
    );
    assert!(
        screen.errors().is_empty(),
        "no error reached the default sink: {:?}",
        screen.errors()
    );
}

/// `base_style` is the component author's layer: below every caller `.style`
/// whatever the order of the calls, and several base styles merge in the
/// order they were added.
#[test]
fn a_base_style_is_below_every_caller_style_whatever_the_call_order() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                // The component's base style comes last, the caller's first.
                node()
                    .test_id("after")
                    .style(style().width(30).height(30))
                    .base_style(style().width(10).height(10).min_height(5)),
                // The caller's style comes last.
                node()
                    .test_id("before")
                    .base_style(style().width(10).height(10))
                    .style(style().width(40)),
                // Two base styles merge in call order; a reactive one follows its signal.
                node()
                    .test_id("stacked")
                    .base_style(style().width(10))
                    .base_style(style().width(20).height(12)),
            ))
            .style(style().align_items(Align::Start)),
        )
    });
    let size = |id: &str| {
        let r = rect(&screen, id);
        (r.width, r.height)
    };
    assert_eq!(size("after"), (30.0, 30.0), "the caller's style wins");
    assert_eq!(
        size("before"),
        (40.0, 10.0),
        "the base fills what the caller leaves"
    );
    assert_eq!(
        size("stacked"),
        (20.0, 12.0),
        "a later base style wins over an earlier one"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
