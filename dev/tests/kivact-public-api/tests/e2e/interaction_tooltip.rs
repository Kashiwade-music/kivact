//! Role: Tests for tooltip display and accessibility.
//!
//! The unstyled `tooltip`: when it shows (hover or focus after a delay), what
//! hides it, one at a time per App, the skip delay and its semantics. The
//! tests draw the hint themselves, as a kit does.

use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::interaction::{TooltipConfig, use_tooltip};
use kivact::prelude::*;
use kivact::primitive::NodeExt;

use kivact::test::Screen;
use kivact::view::MouseButton;
use kivact::view::{Handled, KeyEvent, OverlayAlign, Placement, overlay};

const DELAY: Duration = Duration::from_millis(500);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A button-like trigger. A `focusable` one takes the keyboard focus.
fn trigger(name: &str, focusable: bool) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .test_id(name)
        .focusable(focusable)
        .style(style().size(100, 30).shrink(0).margin_right(20))
        .child(text(name))
}

/// A row of the triggers "A" and "B", each with a tooltip, and "Other", a
/// focusable button without one.
fn pair(focusable: bool) -> View {
    column((
        row((
            tipped("Tip A", trigger("A", focusable)),
            tipped("Tip B", trigger("B", focusable)),
            trigger("Other", true),
        )),
        node().style(style().height(200)),
    ))
    .into_view()
}

/// `trigger` with a tooltip saying `label` as a text at most 320 px wide,
/// below it. The trigger is described by the label unless it has its own
/// description.
fn tipped(label: &'static str, trigger: Node) -> View {
    tip_with(label, Placement::Below, trigger, move || {
        text(label).style(style().padding_x(8).padding_y(4).max_width(320))
    })
}

/// `trigger` with a tooltip named `label` that shows `body` on `placement`.
fn tip_with<V: IntoView>(
    label: &'static str,
    placement: Placement,
    mut trigger: Node,
    body: impl Fn() -> V + 'static,
) -> View {
    let tip = use_tooltip();
    let part = tip.trigger();
    let anchor = part.node_ref();
    let described = trigger.has_description();
    let trigger = trigger.part(part);
    let trigger = if described {
        trigger
    } else {
        trigger.description(label)
    };
    let hint = show(tip.shown(), move || {
        overlay(anchor, placement, body())
            .align(OverlayAlign::Center)
            .offset(6)
            .part(tip.content())
            .name(label)
    });
    (trigger, hint).into_view()
}

fn shown(screen: &Screen, label: &str) -> bool {
    screen.query_by_role(Role::Tooltip, label).is_some()
}

fn none_shown(screen: &Screen) -> bool {
    !shown(screen, "Tip A") && !shown(screen, "Tip B")
}

#[test]
fn hovering_shows_it_after_the_delay_and_leaving_hides_it_at_once() {
    let (screen, ()) = mount(|| ((), pair(false)));
    let a = screen.get_by_test_id("A");
    a.hover();
    screen.advance(ms(499));
    assert!(none_shown(&screen), "not before the delay");
    screen.advance(ms(1));
    assert!(shown(&screen, "Tip A"));
    let tip = screen.get_by_role(Role::Tooltip, "Tip A");
    let anchor = a.rect().unwrap();
    assert!(tip.rect().unwrap().y >= anchor.y + anchor.height);
    screen.mouse_leave();
    assert!(none_shown(&screen), "hides without a delay");
    a.hover();
    screen.advance(ms(100));
    screen.mouse_leave();
    screen.advance(DELAY);
    assert!(none_shown(&screen), "a shorter visit cancels the deadline");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_overlay_is_a_named_tooltip_and_the_trigger_is_described_by_the_label() {
    let (screen, ()) = mount(|| {
        (
            (),
            row((
                tipped("Tip A", trigger("A", true)),
                tipped("Tip B", trigger("B", true).description("Own description")),
            )),
        )
    });
    assert_eq!(screen.get_by_test_id("A").description(), "Tip A");
    assert_eq!(
        screen.get_by_test_id("B").description(),
        "Own description",
        "a description the trigger has stays"
    );
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    let tip = screen.get_by_role(Role::Tooltip, "Tip A");
    assert_eq!(tip.role(), Some(Role::Tooltip));
    assert_eq!(tip.name(), "Tip A");
    assert!(!tip.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn keyboard_focus_shows_it_and_losing_focus_hides_it() {
    let (screen, ()) = mount(|| ((), pair(true)));
    screen.get_by_test_id("A").focus();
    screen.advance(ms(499));
    assert!(none_shown(&screen));
    screen.advance(ms(1));
    assert!(shown(&screen, "Tip A"));
    screen.get_by_test_id("Other").focus();
    assert!(none_shown(&screen), "focus moved away");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_child_with_its_own_pointer_handler_does_not_hide_the_hover() {
    let (screen, ()) = mount(|| {
        (
            (),
            tipped(
                "Tip A",
                node()
                    .role(Role::Button)
                    .name("A")
                    .style(style().size(100, 30))
                    .child(
                        node()
                            .test_id("inner")
                            .style(style().size(100, 30))
                            .on_pointer(|_| Handled::Yes),
                    ),
            ),
        )
    });
    screen.get_by_test_id("inner").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_trigger_with_its_own_node_ref_still_works() {
    let (screen, own_hover) = mount(|| {
        let own = node_ref();
        let own_hover = own.hovered();
        (
            own_hover,
            tipped("Tip A", trigger("A", false).node_ref(own)),
        )
    });
    screen.get_by_test_id("A").hover();
    assert!(own_hover.get_untracked());
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_hides_it_only_while_shown_and_it_waits_for_the_condition_to_start_again() {
    let escapes = Rc::new(RefCell::new(0));
    let seen = escapes.clone();
    let (screen, ()) = mount(move || {
        (
            (),
            row(tipped("Tip A", trigger("A", true))).on_key(move |event: KeyEvent| {
                if event.key == Key::Escape {
                    *seen.borrow_mut() += 1;
                }
                Handled::No
            }),
        )
    });
    screen.get_by_test_id("A").focus();
    screen.press_key(Key::Escape);
    assert_eq!(*escapes.borrow(), 1, "not shown: Escape goes on");
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.press_key(Key::Escape);
    assert!(none_shown(&screen));
    assert_eq!(*escapes.borrow(), 1, "shown: the tooltip consumed Escape");
    screen.advance(ms(2000));
    assert!(none_shown(&screen), "it stays hidden while the focus stays");
    screen.press_key(Key::Escape);
    assert_eq!(*escapes.borrow(), 2, "hidden: Escape goes on");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn escape_hides_a_tooltip_shown_by_hover_while_the_focus_is_elsewhere() {
    let (screen, ()) = mount(|| ((), pair(false)));
    screen.get_by_test_id("Other").focus();
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.press_key(Key::Escape);
    assert!(none_shown(&screen));
    screen.advance(ms(2000));
    assert!(none_shown(&screen));
    screen.mouse_leave();
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"), "the hover started again");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_press_on_the_trigger_hides_it_until_the_pointer_comes_back() {
    let (screen, ()) = mount(|| ((), pair(false)));
    let a = screen.get_by_test_id("A");
    a.hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.mouse_down(MouseButton::Primary);
    assert!(none_shown(&screen));
    screen.mouse_up(MouseButton::Primary);
    screen.advance(ms(2000));
    assert!(none_shown(&screen), "it stays hidden under the pointer");
    screen.mouse_leave();
    a.hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_wheel_over_the_trigger_hides_it_until_the_pointer_comes_back() {
    let (screen, ()) = mount(|| ((), pair(false)));
    let a = screen.get_by_test_id("A");
    a.hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.scroll(0.0, 10.0);
    assert!(none_shown(&screen));
    screen.advance(ms(2000));
    assert!(none_shown(&screen));
    screen.mouse_leave();
    a.hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tab_and_other_keys_do_not_hide_it() {
    let (screen, ()) = mount(|| ((), pair(false)));
    screen.get_by_test_id("Other").focus();
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.press_key(Key::Tab);
    assert!(shown(&screen, "Tip A"), "Tab does not hide it");
    assert!(
        !screen.get_by_role(Role::Tooltip, "Tip A").is_focused(),
        "the tooltip takes no focus"
    );
    screen.press_key(Key::Char('x'));
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn only_one_tooltip_is_shown_and_a_new_one_replaces_the_old_without_a_delay() {
    let (screen, ()) = mount(|| ((), pair(true)));
    screen.get_by_test_id("A").focus();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.get_by_test_id("B").hover();
    assert!(shown(&screen, "Tip B"), "another one is showing: no delay");
    assert!(!shown(&screen, "Tip A"), "one at a time");
    assert_eq!(screen.all_by_role(Role::Tooltip, "Tip B").len(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_tooltip_within_the_skip_delay_of_the_last_one_shows_without_a_delay() {
    let (screen, ()) = mount(|| ((), pair(false)));
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.mouse_leave();
    screen.advance(ms(299));
    screen.get_by_test_id("B").hover();
    assert!(shown(&screen, "Tip B"), "within 300 ms: no delay");
    screen.mouse_leave();
    screen.advance(ms(301));
    screen.get_by_test_id("A").hover();
    screen.advance(ms(499));
    assert!(
        none_shown(&screen),
        "after the skip delay the delay is back"
    );
    screen.advance(ms(1));
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_same_trigger_does_not_skip_its_own_delay() {
    let (screen, ()) = mount(|| ((), pair(false)));
    let a = screen.get_by_test_id("A");
    a.hover();
    screen.advance(DELAY);
    screen.mouse_leave();
    screen.advance(ms(100));
    a.hover();
    screen.advance(ms(499));
    assert!(none_shown(&screen));
    screen.advance(ms(1));
    assert!(shown(&screen, "Tip A"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tooltip_config_overrides_the_delay_and_the_skip_delay() {
    let (screen, ()) = mount(|| {
        (
            (),
            provide(
                TooltipConfig {
                    delay: ms(100),
                    skip_delay: ms(0),
                },
                || pair(false),
            ),
        )
    });
    screen.get_by_test_id("A").hover();
    screen.advance(ms(99));
    assert!(none_shown(&screen));
    screen.advance(ms(1));
    assert!(shown(&screen, "Tip A"));
    screen.mouse_leave();
    screen.advance(ms(1));
    screen.get_by_test_id("B").hover();
    assert!(none_shown(&screen), "no skip with a zero skip delay");
    screen.advance(ms(100));
    assert!(shown(&screen, "Tip B"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_placement_and_the_content_can_be_chosen() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                node().style(style().height(100)),
                tip_with("Tip A", Placement::Above, trigger("A", false), || {
                    text("Custom body")
                }),
            )),
        )
    });
    let a = screen.get_by_test_id("A");
    a.hover();
    screen.advance(DELAY);
    let tip = screen.get_by_role(Role::Tooltip, "Tip A");
    screen.get_by_text("Custom body");
    let tip = tip.rect().unwrap();
    assert!(tip.y + tip.height <= a.rect().unwrap().y);
    assert_eq!(a.description(), "Tip A");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_long_label_wraps_at_the_maximum_width() {
    let long = "A long explanation that goes on and on, well past what fits in three hundred \
                and twenty logical pixels of one line, so that it has to wrap.";
    let (screen, ()) = mount(move || ((), tipped(long, trigger("A", false))));
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    let rect = screen.get_by_role(Role::Tooltip, long).rect().unwrap();
    assert!(rect.width <= 320.5, "{rect:?}");
    assert!(rect.width > 200.0, "{rect:?}");
    assert!(rect.height > 30.0, "wrapped onto several lines: {rect:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn removing_the_trigger_while_shown_leaves_nothing_behind() {
    let (screen, alive) = mount(|| {
        let alive = signal(true);
        (
            alive,
            column((
                show(alive, || tipped("Tip A", trigger("A", false))),
                tipped("Tip B", trigger("B", false)),
            )),
        )
    });
    screen.get_by_test_id("A").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    alive.set(false);
    screen.run_until_idle();
    assert!(none_shown(&screen));
    screen.mouse_leave();
    screen.advance(ms(1000));
    screen.get_by_test_id("B").hover();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip B"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
