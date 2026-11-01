//! Role: Independently declared regions retain their own pointer activation identity.

use crate::support::mount_with;
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::part;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::view::MouseButton;

struct Fixture {
    first: Signal<u32>,
    second: Signal<u32>,
    parent: Signal<u32>,
    ordinary: Signal<u32>,
    present: Signal<bool>,
    disabled: Signal<bool>,
    hidden: Signal<bool>,
}

fn fixture() -> (Screen, Fixture) {
    mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        || {
            let first = signal(0);
            let second = signal(0);
            let parent = signal(0);
            let ordinary = signal(0);
            let present = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let surface = canvas(move |draw, _| {
                // The second region lies underneath the first where they overlap.
                draw.region(RegionId(2), Rect::new(0.0, 0.0, 180.0, 40.0));
                if present.get() {
                    draw.region(RegionId(1), Rect::new(0.0, 0.0, 80.0, 40.0));
                }
            })
            .region(
                RegionId(1),
                part(|state| state)
                    .role(Role::Button)
                    .name("First")
                    .disabled(disabled)
                    .hidden(hidden)
                    .on_press(move || first.update(|n| *n += 1)),
            )
            .region(
                RegionId(2),
                part(|state| state)
                    .role(Role::Button)
                    .name("Second")
                    .on_press(move || second.update(|n| *n += 1)),
            )
            .name("Surface")
            .style(style().size(200, 50));
            let view = column((
                node()
                    .role(Role::Button)
                    .name("Parent")
                    .on_press(move || parent.update(|n| *n += 1))
                    .child(surface),
                node()
                    .role(Role::Button)
                    .name("Ordinary")
                    .on_press(move || ordinary.update(|n| *n += 1))
                    .style(style().size(100, 40)),
            ));
            (
                Fixture {
                    first,
                    second,
                    parent,
                    ordinary,
                    present,
                    disabled,
                    hidden,
                },
                view,
            )
        },
    )
}

fn move_to(screen: &Screen, x: f32) {
    let rect = screen
        .get_by_role(Role::Canvas, "Surface")
        .rect()
        .expect("the canvas is laid out");
    screen.mouse_move(rect.x + x, rect.y + 20.0);
}

#[test]
fn regions_activate_without_an_owner_invoke_and_ordinary_buttons_still_activate() {
    let (screen, state) = fixture();
    move_to(&screen, 20.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    move_to(&screen, 120.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        (state.first.get(), state.second.get(), state.parent.get()),
        (1, 1, 0)
    );
    screen.get_by_role(Role::Button, "Ordinary").click();
    assert_eq!(state.ordinary.get(), 1);
    // A press on unclaimed canvas geometry retains the parent's own action.
    move_to(&screen, 195.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(state.parent.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn releasing_over_a_different_region_does_not_activate_either_region_or_parent() {
    let (screen, state) = fixture();
    move_to(&screen, 20.0);
    screen.mouse_down(MouseButton::Primary);
    move_to(&screen, 120.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        (state.first.get(), state.second.get(), state.parent.get()),
        (0, 0, 0)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disabled_regions_block_parent_activation_and_hidden_regions_expose_geometry_below() {
    let (screen, state) = fixture();
    state.disabled.set(true);
    screen.run_until_idle();
    move_to(&screen, 20.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        (state.first.get(), state.second.get(), state.parent.get()),
        (0, 0, 0)
    );
    state.hidden.set(true);
    screen.run_until_idle();
    move_to(&screen, 20.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        (state.first.get(), state.second.get(), state.parent.get()),
        (0, 1, 0)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn removing_or_disabling_the_pressed_region_before_release_prevents_activation() {
    let (screen, state) = fixture();
    move_to(&screen, 20.0);
    screen.mouse_down(MouseButton::Primary);
    state.present.set(false);
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        (state.first.get(), state.second.get(), state.parent.get()),
        (0, 0, 0)
    );
    state.present.set(true);
    screen.run_until_idle();
    move_to(&screen, 20.0);
    screen.mouse_down(MouseButton::Primary);
    state.disabled.set(true);
    screen.run_until_idle();
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        (state.first.get(), state.second.get(), state.parent.get()),
        (0, 0, 0)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
