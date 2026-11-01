//! Role: Tests for focus behavior with pointer input.
//!
//! The focus a pointer press leaves: a primary press on nothing focusable
//! blurs on its `Down` (committing a composition first), except on a
//! modal's backdrop and inside a popup anchored to the focused input; a
//! press of another button there and a later button of a captured gesture
//! keep the focus.

use crate::support::mount;
use kivact::prelude::*;
use kivact::view::MouseButton;
use kivact::view::{Handled, Placement, overlay};

#[test]
fn a_primary_press_on_nothing_focusable_blurs_except_in_an_anchored_popup() {
    let (screen, (value, open)) = mount(|| {
        let value = signal(String::new());
        let open = signal(false);
        let anchor = node_ref();
        (
            (value, open),
            column((
                kivact::control::node()
                    .name("Draft")
                    .part(kivact::interaction::use_text_edit(value).control())
                    .node_ref(anchor)
                    .style(style().size(180, 30)),
                node().test_id("blank").style(style().size(180, 40)),
                node()
                    .role(Role::Button)
                    .name("Target")
                    .focusable(true)
                    .style(style().size(180, 30))
                    .child(text("Target child")),
                node()
                    .role(Role::Slider)
                    .name("Capture")
                    .focusable(true)
                    .style(style().size(180, 30))
                    .on_pointer(|_| Handled::Yes),
                show(open, move || {
                    overlay(
                        anchor,
                        Placement::End,
                        node().test_id("popup surface").style(style().size(100, 40)),
                    )
                }),
            )),
        )
    });
    let draft = screen.get_by_role(Role::TextBox, "Draft");
    draft.compose("未確定");
    assert!(screen.ime_text().is_some_and(|t| t.composition.is_some()));
    let blank = screen
        .get_by_test_id("blank")
        .rect()
        .expect("laid out")
        .center();
    screen.mouse_move(blank.x, blank.y);
    screen.mouse_down(MouseButton::Primary);
    assert!(
        !draft.is_focused(),
        "the blur happens on Down, before the release"
    );
    assert_eq!(
        value.get_untracked(),
        "未確定",
        "the blur commits the composition"
    );
    assert!(screen.focused().is_none());
    screen.mouse_up(MouseButton::Primary);
    screen.press_tab(false);
    assert!(
        draft.is_focused(),
        "Tab starts from the first tab stop after the blur"
    );

    screen
        .get_by_test_id("blank")
        .click_with(MouseButton::Secondary);
    assert!(
        draft.is_focused(),
        "a secondary press on nothing focusable keeps the focus"
    );

    open.set(true);
    screen.run_until_idle();
    screen.get_by_test_id("popup surface").click();
    assert!(
        draft.is_focused(),
        "a press in a popup anchored to the focused input keeps the focus"
    );
    open.set(false);
    screen.run_until_idle();

    screen.get_by_text("Target child").click();
    assert!(
        screen.get_by_role(Role::Button, "Target").is_focused(),
        "a press focuses the nearest focusable ancestor"
    );
    let capture = screen.get_by_role(Role::Slider, "Capture");
    let start = capture.rect().expect("laid out").center();
    screen.mouse_move(start.x, start.y);
    screen.mouse_down(MouseButton::Primary);
    assert!(capture.is_focused());
    screen.mouse_move(blank.x, blank.y);
    screen.mouse_down(MouseButton::Secondary);
    assert!(
        capture.is_focused(),
        "a later button of a captured gesture keeps the focus"
    );
    screen.mouse_up(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Primary);
    assert!(capture.is_focused(), "the release does not blur either");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_press_on_a_modal_backdrop_keeps_the_focus_in_the_modal() {
    let (screen, ()) = mount(|| {
        let anchor = node_ref();
        (
            (),
            column((
                node()
                    .test_id("behind")
                    .node_ref(anchor)
                    .style(style().size(100, 40)),
                overlay(
                    anchor,
                    Placement::Below,
                    kivact::control::node()
                        .name("Modal draft")
                        .part(kivact::interaction::use_text_edit(signal(String::new())).control())
                        .style(style().size(180, 30)),
                )
                .modal(true),
            )),
        )
    });
    let draft = screen.get_by_role(Role::TextBox, "Modal draft");
    draft.focus();
    assert!(draft.is_focused());
    screen.get_by_test_id("behind").force_click();
    assert!(
        draft.is_focused(),
        "the backdrop is not a blank spot of the modal"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
