//! Role: Parts share semantic, input and lifecycle contracts across targets.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::part;
use kivact::view::{A11yAction, Handled};
use kivact::view::{KeyCode, Modifiers, MouseButton, PointerPhase};

#[test]
fn node_part_hover_includes_children_with_their_own_pointer_listener() {
    let (screen, state) = mount(|| {
        let behavior = part(|state| state);
        let state = behavior.state();
        (
            state,
            node()
                .part(behavior)
                .role(Role::Group)
                .name("Hover parent")
                .child(
                    node()
                        .role(Role::Button)
                        .name("Hover child")
                        .on_pointer(|_| Handled::No)
                        .style(style().size(80, 40)),
                )
                .style(style().size(180, 80)),
        )
    });
    let child = screen
        .get_by_role(Role::Button, "Hover child")
        .rect()
        .unwrap();
    screen.mouse_move(child.x + 10.0, child.y + 10.0);
    assert!(state.get().hovered);
    screen.mouse_leave();
    assert!(!state.get().hovered);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn region_pointer_capture_and_hover_use_independent_targets_and_shared_refs() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    let (screen, (first, second, hovered, consume)) = mount(move || {
        let consume = signal(false);
        let reference = node_ref();
        let hover = reference.hovered();
        let out_first = out.clone();
        let a = part(|state| state)
            .role(Role::Button)
            .name("Drag A")
            .reference(reference)
            .on_pointer(move |event| {
                out_first
                    .borrow_mut()
                    .push((1, event.phase(), event.region()));
                Handled::No
            });
        let first = a.state();
        let b = part(|state| state)
            .role(Role::Button)
            .name("Drag B")
            .on_pointer(move |event| {
                out.borrow_mut().push((2, event.phase(), event.region()));
                Handled::No
            });
        let second = b.state();
        let view = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0.0, 0.0, 80.0, 40.0));
            draw.region(RegionId(2), Rect::new(100.0, 0.0, 80.0, 40.0));
        })
        .role(Role::Canvas)
        .name("Drag canvas")
        .region(RegionId(1), a)
        .region(RegionId(2), b)
        .on_pointer(move |_| {
            if consume.get() {
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .style(style().size(240, 70));
        ((first, second, hover, consume), view)
    });
    let rect = screen
        .get_by_role(Role::Canvas, "Drag canvas")
        .rect()
        .unwrap();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    assert!(first.get().hovered && hovered.get());
    screen.mouse_move(rect.x + 120.0, rect.y + 20.0);
    assert!(!first.get().hovered && !hovered.get());
    assert!(second.get().hovered);
    screen.mouse_move(rect.x + 210.0, rect.y + 20.0);
    assert!(!second.get().hovered);
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    calls.borrow_mut().clear();
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(rect.x + 120.0, rect.y + 20.0);
    assert!(first.get().pressed && !second.get().pressed);
    assert!(!first.get().hovered && second.get().hovered);
    screen.mouse_move(rect.x + 280.0, rect.y + 90.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(!first.get().pressed && !second.get().pressed);
    assert!(calls.borrow().iter().any(|(id, phase, region)| *id == 1
        && matches!(phase, PointerPhase::Up(_))
        && *region == Some(RegionId(1))));
    assert!(
        !calls.borrow().iter().any(|(id, phase, _)| *id == 2
            && matches!(phase, PointerPhase::Down(_) | PointerPhase::Up(_)))
    );
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    screen.mouse_down(MouseButton::Primary);
    consume.set(true);
    screen.mouse_move(rect.x + 120.0, rect.y + 20.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(!first.get().pressed && !second.get().pressed);
    screen.mouse_leave();
    assert!(!first.get().hovered && !second.get().hovered && !hovered.get());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn node_parts_keep_customizations_first_and_observe_consumed_input() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let out = calls.clone();
    let (screen, (state, selected)) = mount(move || {
        let selected = signal(false);
        let behavior_calls = out.clone();
        let behavior = part(|state| state)
            .role(Role::Button)
            .name("Behavior name")
            .selected(selected)
            .focusable(true)
            .on_key(move |_| {
                behavior_calls.borrow_mut().push("behavior key");
                Handled::No
            });
        let state = behavior.state();
        let behavior_press = out.clone();
        let behavior =
            behavior.on_press(move || behavior_press.borrow_mut().push("behavior press"));
        let custom_key = out.clone();
        let custom_press = out.clone();
        let control = node()
            .part(behavior)
            .name("Caller name")
            .on_key(move |_| {
                custom_key.borrow_mut().push("caller key");
                Handled::Yes
            })
            .on_pointer(|_| Handled::Yes)
            .on_press(move || custom_press.borrow_mut().push("caller press"))
            .style(style().size(140, 40));
        ((state, selected), control)
    });
    let button = screen.get_by_role(Role::Button, "Caller name");
    button.focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(
        state.get().pressed,
        "observation precedes the consuming key handler"
    );
    assert_eq!(&*calls.borrow(), &["caller key"]);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(!state.get().pressed);
    calls.borrow_mut().clear();
    button.a11y(A11yAction::Activate);
    assert_eq!(&*calls.borrow(), &["caller press", "behavior press"]);
    selected.set(true);
    screen.run_until_idle();
    assert!(button.is_selected());
    let rect = button.rect().expect("the button is laid out");
    screen.mouse_move(rect.x + 10.0, rect.y + 10.0);
    screen.mouse_down(MouseButton::Primary);
    assert!(
        state.get().pressed,
        "consumed pointer input still updates state"
    );
    screen.mouse_up(MouseButton::Primary);
    assert!(!state.get().pressed);
    screen.mouse_leave();
    assert!(!state.get().hovered);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn canvas_parts_have_independent_focus_actions_input_state_and_geometry_lifetimes() {
    let (screen, (first_state, second_state, first_calls, second_calls, visible, disabled)) =
        mount(|| {
            let visible = signal(true);
            let disabled = signal(false);
            let first_calls = signal(0);
            let second_calls = signal(0);
            let shared_first = node_ref();
            let first = part(|state| state)
                .role(Role::Button)
                .name("First region")
                .reference(shared_first)
                .focusable(true)
                .on_press(move || first_calls.update(|n| *n += 1));
            let second = part(|state| state)
                .role(Role::Button)
                .name("Second region")
                .focusable(true)
                .disabled(disabled)
                .on_press(move || second_calls.update(|n| *n += 1));
            let first_state = first.state();
            let second_state = second.state();
            let first_ref = shared_first;
            let second_ref = second.node_ref();
            let root = canvas(move |draw, _| {
                if visible.get() {
                    draw.region(RegionId(2), Rect::new(100.0, 0.0, 80.0, 40.0));
                }
                draw.region(RegionId(1), Rect::new(0.0, 0.0, 80.0, 40.0));
            })
            .region(RegionId(1), first)
            .region(RegionId(2), second)
            .region(
                RegionId(3),
                part(|state| state)
                    .role(Role::Button)
                    .name("Undrawn")
                    .focusable(true),
            )
            .part(part(|state| state).role(Role::Canvas).name("Parts canvas"))
            .style(style().size(200, 50));
            let view = column((
                root,
                node()
                    .role(Role::Button)
                    .name("Focus first")
                    .focusable(true)
                    .on_press(move || first_ref.focus())
                    .style(style().size(100, 30)),
                node()
                    .role(Role::Button)
                    .name("Focus second")
                    .focusable(true)
                    .on_press(move || second_ref.focus())
                    .style(style().size(100, 30)),
            ));
            (
                (
                    first_state,
                    second_state,
                    first_calls,
                    second_calls,
                    visible,
                    disabled,
                ),
                view,
            )
        });
    screen
        .get_by_role(Role::Button, "Focus first")
        .a11y(A11yAction::Activate);
    assert!(first_state.get().focused);
    assert!(!second_state.get().focused);
    screen.press_key(Key::Enter);
    assert_eq!(first_calls.get(), 1);
    assert_eq!(second_calls.get(), 0);
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(first_state.get().pressed);
    assert!(!second_state.get().pressed);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(!first_state.get().pressed);
    screen.press_key(Key::Tab);
    assert!(
        second_state.get().focused,
        "Tab follows declaration order, not paint order"
    );
    assert!(screen.query_by_role(Role::Button, "Undrawn").is_none());
    screen
        .get_by_role(Role::Button, "Focus second")
        .a11y(A11yAction::Activate);
    assert!(second_state.get().focused);
    assert!(!first_state.get().focused);
    screen
        .get_by_role(Role::Button, "Second region")
        .a11y(A11yAction::Activate);
    assert_eq!(second_calls.get(), 1);
    disabled.set(true);
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::Button, "Second region")
            .is_disabled()
    );
    visible.set(false);
    screen.run_until_idle();
    assert!(!second_state.get().focused);
    assert!(
        screen
            .query_by_role(Role::Button, "Second region")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
