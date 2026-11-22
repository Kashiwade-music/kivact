//! Role: A state style applies every property to its node (ST-05).
//! A paint callback of a state replaces the base's while the state applies.

use std::cell::Cell;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact_public_api_tests::paint::{BLUE, GREEN, RED, WHITE, at, canvas_of, expect};
use kivact_public_api_tests::support::mount;

/// A style whose paint callback fills the box with `color`.
fn filled(color: u32) -> Style {
    style().paint(move |draw, size| {
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(color));
    })
}

/// The width of the node `id`, once laid out.
fn width_of(screen: &Screen, id: &str) -> f64 {
    screen.get_by_test_id(id).rect().expect("laid out").width
}

/// The height of the node `id`, once laid out.
fn height_of(screen: &Screen, id: &str) -> f64 {
    screen.get_by_test_id(id).rect().expect("laid out").height
}

#[test]
fn hover_applies_a_layout_property() {
    let screen = render(|| {
        canvas_of(
            node()
                .test_id("box")
                .style(at(0, 0, 40, 20).hover(style().width(80))),
        )
    });
    assert_eq!(width_of(&screen, "box"), 40.0, "before the hover");
    screen.get_by_test_id("box").hover();
    assert_eq!(width_of(&screen, "box"), 80.0, "hovered");
    screen.mouse_leave();
    assert_eq!(width_of(&screen, "box"), 40.0, "left");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn active_applies_a_layout_property() {
    let screen = render(|| {
        canvas_of(
            node()
                .test_id("box")
                .style(at(0, 0, 40, 20).active(style().height(40))),
        )
    });
    screen.get_by_test_id("box").hover();
    assert_eq!(height_of(&screen, "box"), 20.0, "hovered, not pressed");
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(height_of(&screen, "box"), 40.0, "pressed");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(height_of(&screen, "box"), 20.0, "released");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn focus_applies_a_text_property() {
    let screen = render(|| {
        canvas_of(
            node()
                .test_id("field")
                .role(Role::Button)
                .name("field")
                .focusable(true)
                .style(at(0, 0, 200, 60).focus(style().font_size(30)))
                .child(text("Aa").test_id("label")),
        )
    });
    let before = height_of(&screen, "label");
    screen.get_by_test_id("field").focus();
    assert!(
        height_of(&screen, "label") > before,
        "the focused text is larger: {before} before"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_paint_only_state_stays_a_host_delta() {
    let screen = render(|| {
        canvas_of(
            node().test_id("box").style(
                at(0, 0, 40, 40)
                    .background(rgb(WHITE))
                    .hover(style().background(rgb(RED))),
            ),
        )
    });
    screen.get_by_test_id("box").hover();
    expect(&screen.screenshot(), 20, 20, RED, 0, "hovered background");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hover_draws_its_paint_callback() {
    let screen = render(|| {
        canvas_of(
            node()
                .test_id("box")
                .style(at(0, 0, 40, 40).hover(filled(RED))),
        )
    });
    expect(&screen.screenshot(), 20, 20, WHITE, 0, "not hovered");
    screen.get_by_test_id("box").hover();
    expect(&screen.screenshot(), 20, 20, RED, 0, "hovered");
    screen.mouse_leave();
    expect(&screen.screenshot(), 20, 20, WHITE, 0, "left");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn active_paint_replaces_hover_paint() {
    let screen = render(|| {
        canvas_of(
            node()
                .test_id("box")
                .style(at(0, 0, 40, 40).hover(filled(BLUE)).active(filled(RED))),
        )
    });
    screen.get_by_test_id("box").hover();
    expect(&screen.screenshot(), 20, 20, BLUE, 0, "hovered");
    screen.mouse_down(MouseButton::Primary);
    expect(&screen.screenshot(), 20, 20, RED, 0, "pressed over hover");
    screen.mouse_up(MouseButton::Primary);
    expect(
        &screen.screenshot(),
        20,
        20,
        BLUE,
        0,
        "released, still hovered",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn focus_draws_its_paint_callback() {
    let screen = render(|| {
        canvas_of((
            node()
                .test_id("box")
                .role(Role::Button)
                .name("box")
                .focusable(true)
                .style(at(0, 0, 40, 40).focus(filled(RED))),
            node()
                .test_id("other")
                .role(Role::Button)
                .name("other")
                .focusable(true)
                .style(at(100, 0, 40, 40)),
        ))
    });
    expect(&screen.screenshot(), 20, 20, WHITE, 0, "not focused");
    screen.get_by_test_id("box").focus();
    expect(&screen.screenshot(), 20, 20, RED, 0, "focused");
    screen.get_by_test_id("other").focus();
    expect(&screen.screenshot(), 20, 20, WHITE, 0, "blurred");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disabled_and_selected_draw_their_paint_callbacks() {
    let (screen, (disabled, selected)) = mount(|| {
        let (disabled, selected) = (signal(false), signal(false));
        let view = canvas_of(
            node()
                .test_id("box")
                .disabled(disabled)
                .selected(selected)
                .style(
                    at(0, 0, 40, 40)
                        .disabled(filled(RED))
                        .selected(filled(GREEN)),
                ),
        );
        ((disabled, selected), view)
    });
    expect(&screen.screenshot(), 20, 20, WHITE, 0, "neither");
    disabled.set(true);
    screen.run_until_idle();
    expect(&screen.screenshot(), 20, 20, RED, 0, "disabled");
    disabled.set(false);
    selected.set(true);
    screen.run_until_idle();
    expect(&screen.screenshot(), 20, 20, GREEN, 0, "selected");
    selected.set(false);
    screen.run_until_idle();
    expect(&screen.screenshot(), 20, 20, WHITE, 0, "neither again");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_base_paint_comes_back_after_the_state() {
    let screen = render(|| {
        canvas_of(
            node()
                .test_id("box")
                .style(at(0, 0, 40, 40).merge(&filled(BLUE)).hover(filled(RED))),
        )
    });
    expect(&screen.screenshot(), 20, 20, BLUE, 0, "base paint");
    screen.get_by_test_id("box").hover();
    expect(&screen.screenshot(), 20, 20, RED, 0, "hovered");
    screen.mouse_leave();
    expect(&screen.screenshot(), 20, 20, BLUE, 0, "base paint returns");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pressing_again_does_not_run_the_style_layers_again() {
    let runs = Rc::new(Cell::new(0));
    let counted = Rc::clone(&runs);
    let screen = render(move || {
        canvas_of(node().test_id("box").style(move || {
            counted.set(counted.get() + 1);
            at(0, 0, 40, 20).active(style().height(40))
        }))
    });
    screen.get_by_test_id("box").hover();
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    let settled = runs.get();
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(height_of(&screen, "box"), 40.0, "pressed again");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(height_of(&screen, "box"), 20.0, "released again");
    assert_eq!(
        runs.get(),
        settled,
        "the layers run again only when what they read changes, not on a press"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
