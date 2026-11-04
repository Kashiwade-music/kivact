//! Role: Tests for tooltip display and accessibility.
//!
//! The unstyled `tooltip`: when it shows (hover or focus after a delay), what
//! hides it, one at a time per App, the skip delay and its semantics. The
//! tests draw the hint themselves, as a kit does.

use kivact::view::View;
use std::time::Duration;

use crate::support::mount;
use kivact::interaction::use_tooltip;
use kivact::prelude::*;

use kivact::test::Screen;
use kivact::view::{OverlayAlign, Placement, overlay};

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
fn a_tooltip_taken_over_shows_again_when_the_other_hides() {
    let (screen, ()) = mount(|| ((), pair(true)));
    screen.get_by_test_id("A").focus();
    screen.advance(DELAY);
    assert!(shown(&screen, "Tip A"));
    screen.get_by_test_id("B").hover();
    screen.advance(ms(1));
    assert!(
        shown(&screen, "Tip B"),
        "the warm one shows without a delay"
    );
    assert!(!shown(&screen, "Tip A"), "one at a time");
    screen.mouse_leave();
    screen.advance(ms(1));
    assert!(
        shown(&screen, "Tip A"),
        "the one it preempted is still engaged"
    );
    assert!(!shown(&screen, "Tip B"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
