//! Role: Toast presence with a kit's theme installed.

use kivact::interaction::toaster;
use kivact::motion::{Easing, Transition};
use std::time::Duration;

use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::test::{RenderOptions, Screen};
use kivact::view::PointerKind;
use kivact_public_api_tests::support::mount_with;

fn toast_options() -> RenderOptions {
    RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light))
}

#[test]
fn toast_swipe_dismiss_exits_from_the_drag_offset() {
    let theme =
        kivact_ui_soft::theme(ThemeMode::Light).with(kivact::ui::UiRecipes::default().toast(|r| {
            r.toast(|_| {
                |_: &_, _: &_| {
                    style()
                        .size(240, 100)
                        .padding(12)
                        .background(rgb(0xcc0000))
                        .exit(style().opacity(0.0).translate_y(120))
                        .transition(TransitionProp::All, Transition::tween(180, Easing::Linear))
                }
            })
        }));
    let (screen, _) = mount_with(RenderOptions::from_env().theme(theme), || {
        let queue = toaster();
        queue.push(Toast::new("Swipe").duration(None));
        ((), kivact::ui::toast_region(queue))
    });
    advance(&screen, 300);
    let rect = screen.get_by_role(Role::Status, "Swipe").rect().unwrap();
    let contact = screen.contact_down(PointerKind::Touch, rect.x + 6.0, rect.y + 5.0);
    screen.contact_move(contact, rect.x + 6.0, rect.y + 65.0);
    advance(&screen, 16);
    let drag_top = red_top(&screen).expect("invariant: drag paint is visible");
    assert!(drag_top as f64 > rect.y + 40.0);
    screen.contact_up(contact);
    assert!(screen.query_by_role(Role::Status, "Swipe").is_none());
    let exit_top = red_top(&screen).expect("invariant: exit retains drag paint");
    assert!(exit_top >= drag_top.saturating_sub(2));
    // One display frame into the exit: the card fades as it slides on from
    // the drag offset, so its paint is red tinted rather than red.
    advance(&screen, 16);
    assert!(
        tinted_top(&screen).is_some_and(|top| top + 2 >= exit_top),
        "exit must animate instead of vanishing"
    );
    advance(&screen, 400);
    assert!(tinted_top(&screen).is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The first row of the screenshot with a pixel `matches` accepts.
fn top_where(screen: &Screen, matches: impl Fn(u32, u32, u32) -> bool) -> Option<u32> {
    let shot = screen.screenshot();
    shot.pixels
        .iter()
        .position(|pixel| matches((pixel >> 16) & 255, (pixel >> 8) & 255, pixel & 255))
        .map(|index| index as u32 / shot.width)
}

fn red_top(screen: &Screen) -> Option<u32> {
    top_where(screen, |red, green, blue| {
        red > 80 && green < red / 3 && blue < red / 3
    })
}

/// The top of the red card's paint, faded or not.
fn tinted_top(screen: &Screen) -> Option<u32> {
    top_where(screen, |red, green, blue| {
        red > green + 40 && red > blue + 40
    })
}

#[test]
fn toast_row_shrinks_while_it_exits() {
    let (screen, (queue, id)) = mount_with(toast_options(), || {
        let queue = toaster();
        queue.push(Toast::new("Earlier").duration(None));
        let id = queue.push(Toast::new("Leaving").duration(None));
        ((queue, id), kivact::ui::toast_region(queue))
    });
    advance(&screen, 300);
    let y = || {
        screen
            .get_by_role(Role::Status, "Earlier")
            .rect()
            .unwrap()
            .y
    };
    let before = y();
    queue.dismiss(id);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Status, "Leaving").is_none());
    advance(&screen, 40);
    let middle = y();
    advance(&screen, 300);
    assert!(before < middle && middle < y());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn advance(screen: &Screen, ms: u64) {
    screen.advance(Duration::from_millis(ms));
}
