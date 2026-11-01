//! Role: These tests verify native focus and input identity for canvas parts.

use crate::support::mount_with;
use kivact::control::canvas;
use kivact::draw::{Point, Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::{NodeExt, part};
use kivact::test::{RenderOptions, WindowMode};
use kivact::view::{A11yAction, Handled, Placement};

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
}

#[test]
fn regions_have_independent_native_tab_pointer_and_accessibility_focus() {
    let (screen, (keys, released, invoked, focused, within, visible)) =
        mount_with(real_window(), || {
            let keys = signal(Vec::new());
            let released = signal(Vec::new());
            let invoked = signal(Vec::new());
            let reference = node_ref();
            let focused = reference.focused();
            let within = reference.focus_within();
            let visible = reference.focus_visible();
            let owner = canvas(|d, _| {
                for (id, x) in [(22, 0.0), (11, 100.0)] {
                    d.region(RegionId(id), Rect::new(x, 0.0, 90.0, 40.0));
                }
            })
            .region(
                RegionId(22),
                part(|state| state)
                    .role(Role::Button)
                    .name("First")
                    .description("First region")
                    .focusable(true)
                    .on_a11y_event(A11yAction::Activate, move |event| {
                        invoked.update(|v| v.push(event.region))
                    }),
            )
            .region(
                RegionId(11),
                part(|state| state)
                    .role(Role::Button)
                    .name("Second")
                    .description("Second region")
                    .focusable(true)
                    .on_a11y_event(A11yAction::Activate, move |event| {
                        invoked.update(|v| v.push(event.region))
                    }),
            )
            .name("Owner")
            .node_ref(reference)
            .focusable(true)
            .style(style().size(200, 50))
            .on_key(move |e| {
                keys.update(|v| v.push(e.region));
                Handled::No
            })
            .on_key_up(move |e| {
                released.update(|v| v.push(e.region));
            })
            .on_a11y_event(A11yAction::Activate, move |e| {
                invoked.update(|v| v.push(e.region))
            });
            (
                (keys, released, invoked, focused, within, visible),
                column((
                    owner,
                    node()
                        .role(Role::Button)
                        .name("After")
                        .focusable(true)
                        .style(style().size(80, 30)),
                )),
            )
        });
    let owner = screen.get_by_role(Role::Canvas, "Owner");
    let first = screen.get_by_role(Role::Button, "First");
    let second = screen.get_by_role(Role::Button, "Second");
    owner.focus();
    assert!(focused.get());
    assert!(visible.get());
    screen.press_tab(false);
    assert!(first.is_focused());
    assert!(!owner.is_focused());
    assert!(!second.is_focused());
    assert!(!focused.get());
    assert!(within.get());
    assert_eq!(screen.focused().unwrap().name(), "First");
    assert_eq!(first.description(), "First region");
    screen.press_key(Key::Char('x'));
    assert_eq!(keys.get().last(), Some(&Some(RegionId(22))));
    assert_eq!(released.get().last(), Some(&Some(RegionId(22))));
    screen.press_key(Key::Enter);
    screen.press_key(Key::Space);
    assert_eq!(invoked.get(), vec![Some(RegionId(22)), Some(RegionId(22))]);
    screen.press_tab(false);
    assert!(second.is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    screen.press_tab(true);
    assert!(second.is_focused());
    first.click();
    assert!(first.is_focused());
    second.focus();
    assert!(second.is_focused());
    assert_eq!(screen.focused().unwrap().name(), "Second");
    owner.click();
    owner.focus();
    assert!(visible.get());
    // Pointer focus on the owner uses a location outside either region.
    let rect = owner.rect().unwrap();
    screen.click_at(rect.x + 195.0, rect.y + 47.0);
    assert!(focused.get());
    assert!(!visible.get());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn removed_disabled_and_hidden_regions_cannot_retain_focus_or_actions() {
    let (screen, (show_first, disabled, hidden, owner_disabled, invoked)) =
        mount_with(real_window(), || {
            let show_first = signal(true);
            let disabled = signal(false);
            let hidden = signal(false);
            let owner_disabled = signal(false);
            let invoked = signal(0);
            let owner = canvas(move |d, _| {
                if show_first.get() {
                    d.region(RegionId(1), Rect::new(0.0, 0.0, 90.0, 40.0));
                }
                d.region(RegionId(2), Rect::new(100.0, 0.0, 90.0, 40.0));
            })
            .region(
                RegionId(1),
                part(|state| state)
                    .role(Role::Button)
                    .name("First")
                    .focusable(true)
                    .on_press(move || invoked.update(|v| *v += 1)),
            )
            .region(
                RegionId(2),
                part(|state| state)
                    .role(Role::Button)
                    .name("Second")
                    .focusable(true)
                    .disabled(disabled)
                    .hidden(hidden)
                    .on_press(move || invoked.update(|v| *v += 1)),
            )
            .name("Owner")
            .disabled(owner_disabled)
            .style(style().size(200, 50))
            .on_a11y_event(A11yAction::Activate, move |_| invoked.update(|v| *v += 1));
            (
                (show_first, disabled, hidden, owner_disabled, invoked),
                column((
                    node()
                        .role(Role::Button)
                        .name("Before")
                        .focusable(true)
                        .style(style().size(80, 30)),
                    owner,
                    node()
                        .role(Role::Button)
                        .name("After")
                        .focusable(true)
                        .style(style().size(80, 30)),
                )),
            )
        });
    let first = screen.get_by_role(Role::Button, "First");
    let second = screen.get_by_role(Role::Button, "Second");
    first.focus();
    show_first.set(false);
    screen.run_until_idle();
    assert!(!first.exists());
    assert!(second.exists());
    assert!(second.is_focused());
    disabled.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(second.a11y_actions().is_empty());
    second.a11y(A11yAction::Activate);
    assert_eq!(invoked.get(), 0);
    disabled.set(false);
    screen.run_until_idle();
    second.focus();
    hidden.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(second.a11y_actions().is_empty());
    hidden.set(false);
    screen.run_until_idle();
    second.focus();
    owner_disabled.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(!second.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn modal_cleanup_restores_the_exact_region_and_keeps_it_confined_while_open() {
    let (screen, open) = mount_with(real_window(), || {
        let open = signal(false);
        let owner = canvas(|d, _| {
            for (id, x) in [(1, 0.0), (2, 100.0)] {
                d.region(RegionId(id), Rect::new(x, 0.0, 90.0, 40.0));
            }
        })
        .region(
            RegionId(1),
            part(|state| state)
                .role(Role::Button)
                .name("First")
                .focusable(true)
                .on_press(move || open.set(true)),
        )
        .region(
            RegionId(2),
            part(|state| state)
                .role(Role::Button)
                .name("Second")
                .focusable(true)
                .on_press(move || open.set(true)),
        )
        .name("Owner")
        .style(style().size(200, 50))
        .on_a11y_event(A11yAction::Activate, move |_| open.set(true));
        (
            open,
            column((
                owner,
                show(open, move || {
                    kivact::view::overlay_at(
                        Point::new(240.0, 80.0),
                        Placement::Center,
                        node()
                            .role(Role::Button)
                            .name("Inside")
                            .focusable(true)
                            .style(style().size(100, 40)),
                    )
                    .modal(true)
                }),
            )),
        )
    });
    let first = screen.get_by_role(Role::Button, "First");
    let second = screen.get_by_role(Role::Button, "Second");
    second.focus();
    screen.press_key(Key::Enter);
    let inside = screen.get_by_role(Role::Button, "Inside");
    assert!(inside.is_focused());
    screen.press_tab(false);
    assert!(inside.is_focused());
    assert!(!first.is_focused());
    assert!(!second.is_focused());
    open.set(false);
    screen.run_until_idle();
    assert!(second.is_focused());
    assert_eq!(screen.focused().unwrap().name(), "Second");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn removing_a_region_owner_moves_focus_to_the_next_surviving_node() {
    let (screen, alive) = mount_with(real_window(), || {
        let alive = signal(true);
        (
            alive,
            column((
                show(alive, || {
                    canvas(|d, _| {
                        d.region(RegionId(7), Rect::new(0.0, 0.0, 80.0, 40.0));
                    })
                    .region(
                        RegionId(7),
                        part(|state| state)
                            .role(Role::Button)
                            .name("Temporary region")
                            .focusable(true),
                    )
                    .name("Temporary owner")
                    .style(style().size(100, 50))
                }),
                node()
                    .role(Role::Button)
                    .name("Survivor")
                    .focusable(true)
                    .style(style().size(100, 40)),
            )),
        )
    });
    let region = screen.get_by_role(Role::Button, "Temporary region");
    region.focus();
    alive.set(false);
    screen.run_until_idle();
    assert!(!region.exists());
    assert!(!region.is_focused());
    assert!(screen.get_by_role(Role::Button, "Survivor").is_focused());
    assert_eq!(screen.focused().unwrap().name(), "Survivor");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_node_reference_can_first_request_its_rect_from_an_input_handler() {
    let (screen, rect) = mount_with(real_window(), || {
        let reference = node_ref();
        let rect = std::rc::Rc::new(std::cell::Cell::new(None));
        let output = rect.clone();
        (
            rect,
            node()
                .role(Role::Button)
                .name("Measure on press")
                .node_ref(reference)
                .style(style().size(100, 40))
                .on_press(move || output.set(Some(reference.rect()))),
        )
    });
    screen.get_by_role(Role::Button, "Measure on press").click();
    assert!(
        rect.get()
            .expect("the handler requested geometry")
            .get()
            .is_some()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
