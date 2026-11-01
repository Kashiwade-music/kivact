//! Role: Parts share semantic, input and lifecycle contracts across targets.

use kivact::primitive::NodeExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::control::canvas;
use kivact::draw::{Point, Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::{NodeRefExt, PartMount, part};
use kivact::view::{A11yAction, Handled, Placement, overlay};
use kivact::view::{KeyCode, Modifiers, MouseButton, PointerPhase, ScrollBehavior};

mod attachments;
mod layers;
mod region_events;

#[test]
fn moving_a_region_between_collections_commits_one_consistent_declaration_table() {
    let (screen, first) = mount(|| {
        let first = signal(false);
        let view = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0.0, 0.0, 80.0, 40.0));
        })
        .regions(
            move || if first.get() { vec![1u64] } else { Vec::new() },
            |id| RegionId(*id),
            |_, _| {
                part(|state| state)
                    .role(Role::Slider)
                    .name("First collection")
                    .on_a11y_event(A11yAction::Increment, |_| ())
            },
        )
        .regions(
            move || if first.get() { Vec::new() } else { vec![1u64] },
            |id| RegionId(*id),
            |_, _| {
                part(|state| state)
                    .role(Role::CheckBox)
                    .name("Second collection")
                    .on_a11y_event(A11yAction::Toggle, |_| ())
            },
        )
        .style(style().size(120, 60));
        (first, view)
    });
    assert!(
        screen
            .query_by_role(Role::CheckBox, "Second collection")
            .is_some()
    );
    for _ in 0..3 {
        first.set(true);
        screen.run_until_idle();
        assert_eq!(
            screen
                .get_by_role(Role::Slider, "First collection")
                .a11y_actions(),
            vec![A11yAction::Increment]
        );
        assert!(
            screen
                .query_by_role(Role::Slider, "First collection")
                .is_some()
        );
        assert!(
            screen
                .query_by_role(Role::CheckBox, "Second collection")
                .is_none()
        );
        first.set(false);
        screen.run_until_idle();
        assert_eq!(
            screen
                .get_by_role(Role::CheckBox, "Second collection")
                .a11y_actions(),
            vec![A11yAction::Toggle]
        );
        assert!(
            screen
                .query_by_role(Role::CheckBox, "Second collection")
                .is_some()
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

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
                    .push((1, event.phase, event.region()));
                Handled::No
            });
        let first = a.state();
        let b = part(|state| state)
            .role(Role::Button)
            .name("Drag B")
            .on_pointer(move |event| {
                out.borrow_mut().push((2, event.phase, event.region()));
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
        && *phase == PointerPhase::Up
        && *region == Some(RegionId(1))));
    assert!(
        !calls.borrow().iter().any(
            |(id, phase, _)| *id == 2 && matches!(phase, PointerPhase::Down | PointerPhase::Up)
        )
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
fn dynamic_regions_retain_keyed_parts_and_cancel_removed_pointer_attachments() {
    let made = Rc::new(Cell::new(0));
    let bound = Rc::new(Cell::new(0));
    let cleaned = Rc::new(Cell::new(0));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (m, b, c, log) = (made.clone(), bound.clone(), cleaned.clone(), calls.clone());
    let (screen, (items, width)) = mount(move || {
        let items = signal(vec![
            (1u64, String::from("Dynamic A")),
            (2, String::from("Dynamic B")),
        ]);
        let width = signal(240.0);
        let view = canvas(move |draw, _| {
            for (index, (id, _)) in items.get().iter().enumerate() {
                draw.region(
                    RegionId(*id),
                    Rect::new(index as f32 * 100.0, 0.0, 80.0, 40.0),
                );
            }
        })
        .regions(
            items,
            |item| RegionId(item.0),
            move |key, item| {
                m.set(m.get() + 1);
                let (b, c, log) = (b.clone(), c.clone(), log.clone());
                part(|state| state)
                    .role(Role::Button)
                    .name(move || item.get().1)
                    .on_bind(move |_| {
                        b.set(b.get() + 1);
                        let c = c.clone();
                        on_cleanup(move || c.set(c.get() + 1));
                    })
                    .on_pointer(move |event| {
                        log.borrow_mut().push((key, event.phase));
                        Handled::No
                    })
            },
        )
        .role(Role::Canvas)
        .name("Dynamic canvas")
        .style(move || style().width(width.get()).height(70));
        ((items, width), view)
    });
    assert_eq!(made.get(), 2);
    assert_eq!(bound.get(), 2);
    let rect = screen
        .get_by_role(Role::Canvas, "Dynamic canvas")
        .rect()
        .unwrap();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    screen.mouse_down(MouseButton::Primary);
    items.set(vec![(2, String::from("Renamed B"))]);
    screen.run_until_idle();
    assert_eq!(made.get(), 2, "retained keys keep their Part transport");
    assert_eq!(cleaned.get(), 1);
    assert!(screen.query_by_role(Role::Button, "Renamed B").is_some());
    assert!(
        calls
            .borrow()
            .contains(&(RegionId(1), PointerPhase::Cancel))
    );
    items.set(vec![
        (1, String::from("New A")),
        (2, String::from("Renamed B")),
    ]);
    screen.run_until_idle();
    assert_eq!(made.get(), 3);
    calls.borrow_mut().clear();
    screen.mouse_move(rect.x + 20.0, rect.y + 20.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        !calls
            .borrow()
            .iter()
            .any(|(_, phase)| *phase == PointerPhase::Up),
        "the canceled native capture cannot reach a replacement attachment"
    );
    width.set(280.0);
    screen.run_until_idle();
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!(calls.borrow().contains(&(RegionId(1), PointerPhase::Up)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dynamic_region_references_rebind_and_follow_geometry_without_recreating_parts() {
    let made = Rc::new(Cell::new(0));
    let bound = Rc::new(Cell::new(0));
    let cleaned = Rc::new(Cell::new(0));
    let (m, b, c) = (made.clone(), bound.clone(), cleaned.clone());
    let (screen, (items, shift, reference)) = mount(move || {
        let reference = node_ref();
        let items = signal(vec![1u64, 2]);
        let shift = signal(0.0);
        let view = canvas(move |draw, _| {
            for (index, id) in items.get().iter().enumerate() {
                draw.region(
                    RegionId(*id),
                    Rect::new(index as f32 * 100.0 + shift.get(), 0.0, 80.0, 40.0),
                );
            }
        })
        .regions(
            items,
            |id| RegionId(*id),
            move |key, _| {
                m.set(m.get() + 1);
                let behavior = part(|state| state)
                    .role(Role::Button)
                    .name(format!("Ref {}", key.0))
                    .focusable(true);
                let behavior = if key.0 == 1 {
                    behavior.reference(reference)
                } else {
                    behavior
                };
                let (b, c) = (b.clone(), c.clone());
                behavior.on_bind(move |_| {
                    b.set(b.get() + 1);
                    let c = c.clone();
                    on_cleanup(move || c.set(c.get() + 1));
                })
            },
        )
        .child(node().style(style().size(300, 300).shrink(0.0)))
        .style(
            style()
                .size(320, 70)
                .overflow(kivact::style::Overflow::Scroll),
        );
        ((items, shift, reference), view)
    });
    // Both first public reads deliberately happen after setup.
    let rect = reference.rect();
    let scroll = reference.scroll_offset();
    screen.run_until_idle();
    let initial = rect.get().expect("region geometry is observed");
    assert_eq!(scroll.get(), Point::default());
    reference.focus();
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "Ref 1").is_focused());
    items.set(vec![2, 1]);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "Ref 1").is_focused());
    assert_eq!(made.get(), 2);
    assert_eq!(bound.get(), 2);
    assert!((rect.get().unwrap().x - initial.x - 100.0).abs() < 0.1);
    shift.set(10.0);
    screen.run_until_idle();
    assert!((rect.get().unwrap().x - initial.x - 110.0).abs() < 0.1);
    items.set(vec![2]);
    screen.run_until_idle();
    assert!(rect.get().is_none());
    assert_eq!(cleaned.get(), 1);
    items.set(vec![2, 1]);
    screen.run_until_idle();
    assert_eq!(made.get(), 3);
    assert_eq!(bound.get(), 3);
    assert!(rect.get().is_some());
    shift.set(20.0);
    screen.run_until_idle();
    assert!((rect.get().unwrap().x - initial.x - 120.0).abs() < 0.1);
    reference.scroll_to(Point::new(0.0, 50.0), ScrollBehavior::Instant);
    screen.run_until_idle();
    assert_eq!(
        scroll.get().y,
        50.0,
        "shared scroll callbacks survive region rebind"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn duplicate_region_adoption_rebinds_the_same_later_part_after_shadowing() {
    let bindings = Rc::new(Cell::new(0));
    let cleanups = Rc::new(Cell::new(0));
    let (b, c) = (bindings.clone(), cleanups.clone());
    let (screen, (items, reference)) = mount(move || {
        let items = signal(vec![1u64]);
        let reference = node_ref();
        let later = part(|state| state)
            .role(Role::Button)
            .name("Later declaration")
            .reference(reference)
            .on_bind(move |_| {
                b.set(b.get() + 1);
                let c = c.clone();
                on_cleanup(move || c.set(c.get() + 1));
            });
        let view = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0.0, 0.0, 80.0, 40.0));
        })
        .regions(
            items,
            |id| RegionId(*id),
            |_, _| {
                part(|state| state)
                    .role(Role::Button)
                    .name("First declaration")
            },
        )
        .region(RegionId(1), later)
        .style(style().size(120, 60));
        ((items, reference), view)
    });
    assert_eq!(bindings.get(), 0);
    assert!(
        screen
            .query_by_role(Role::Button, "First declaration")
            .is_some()
    );
    items.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(bindings.get(), 1);
    let observed = reference.rect();
    screen.run_until_idle();
    assert!(observed.get().is_some());
    items.set(vec![1]);
    screen.run_until_idle();
    assert_eq!(cleanups.get(), 1);
    assert!(observed.get().is_none());
    items.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(
        bindings.get(),
        2,
        "adoption hooks rerun without constructing another Part"
    );
    assert!(observed.get().is_some());
    assert!(
        screen
            .query_by_role(Role::Button, "Later declaration")
            .is_some()
    );
    assert_eq!(
        screen.errors().len(),
        1,
        "duplicate declarations report once"
    );
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
fn bind_hooks_receive_final_constraints_and_cleanup_with_the_mounted_branch() {
    let mounts = Rc::new(RefCell::new(Vec::<PartMount>::new()));
    let cleanups = Rc::new(Cell::new(0));
    let saved = mounts.clone();
    let cleaned = cleanups.clone();
    let (screen, visible) = mount(move || {
        let visible = signal(true);
        let behavior = part(|state| state)
            .role(Role::Button)
            .name("Constrained")
            .disabled(true)
            .focusable(false)
            .on_bind(move |mount| {
                assert!(mount.node.mounted_node().is_some());
                saved.borrow_mut().push(mount);
                let cleaned = cleaned.clone();
                on_cleanup(move || cleaned.set(cleaned.get() + 1));
            });
        let control = node()
            .part(behavior)
            .disabled(false)
            .focusable(true)
            .style(style().size(120, 36));
        (
            visible,
            show(visible, {
                let control = RefCell::new(Some(control));
                move || control.borrow_mut().take().expect("the branch opens once")
            }),
        )
    });
    assert_eq!(mounts.borrow().len(), 1);
    let constraints = mounts.borrow()[0];
    assert!(constraints.disabled.get());
    assert!(!constraints.hidden.get());
    assert!(
        !constraints.focusable.get(),
        "caller cannot override a roving stop"
    );
    assert!(
        constraints.eligible.get(),
        "roving focus is excluded from eligibility"
    );
    assert!(
        screen
            .get_by_role(Role::Button, "Constrained")
            .is_disabled()
    );
    visible.set(false);
    screen.run_until_idle();
    assert_eq!(cleanups.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn text_fields_overlays_multiple_parts_and_state_mapping_share_one_target() {
    let (screen, (field_state, panel_state, value)) = mount(|| {
        let value = signal(String::new());
        let field = part(|state| state).description("Part field description");
        let field_ref = field.node_ref();
        let field = field.map_state(|state| state.focused);
        let field_state = field.state();
        let panel = part(|state| state).role(Role::Dialog).name("Part overlay");
        let panel_state = panel.state();
        let view = column((
            kivact::control::node()
                .name("Part field")
                .part(kivact::interaction::use_text_edit(value).control())
                .part(field),
            node()
                .part(part(|state| state).role(Role::Button).name("First part"))
                .part(
                    part(|state| state)
                        .name("Second part")
                        .checked(true)
                        .expanded(true),
                )
                .style(style().size(100, 30)),
            overlay(field_ref, Placement::Below, text("Panel")).part(panel),
        ));
        ((field_state, panel_state, value), view)
    });
    let field = screen.get_by_role(Role::TextBox, "Part field");
    assert_eq!(field.description(), "Part field description");
    field.focus();
    assert!(field_state.get());
    field.type_text("hello");
    assert_eq!(value.get(), "hello");
    let second = screen.get_by_role(Role::Button, "Second part");
    assert_eq!(second.is_checked(), Some(true));
    assert_eq!(second.is_expanded(), Some(true));
    assert!(
        screen
            .get_by_role(Role::Dialog, "Part overlay")
            .is_visible()
    );
    assert!(!panel_state.get().focused);
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

#[test]
fn canvas_duplicate_declarations_unknown_ids_and_duplicate_geometry_report_once() {
    let (screen, repaint) = mount(|| {
        let repaint = signal(0);
        let view = canvas(move |draw, _| {
            repaint.get();
            draw.region(RegionId(1), Rect::new(0.0, 0.0, 40.0, 40.0));
            draw.region(RegionId(1), Rect::new(50.0, 0.0, 40.0, 40.0));
            draw.region(RegionId(9), Rect::new(100.0, 0.0, 40.0, 40.0));
        })
        .region(
            RegionId(1),
            part(|state| state).role(Role::Button).name("Accepted"),
        )
        .region(
            RegionId(1),
            part(|state| state).role(Role::Button).name("Dropped"),
        )
        .style(style().size(150, 50));
        (repaint, view)
    });
    assert!(screen.query_by_role(Role::Button, "Accepted").is_some());
    assert!(screen.query_by_role(Role::Button, "Dropped").is_none());
    assert_eq!(
        screen.errors().len(),
        3,
        "one diagnostic for each distinct misuse"
    );
    repaint.update(|n| *n += 1);
    screen.run_until_idle();
    assert_eq!(
        screen.errors().len(),
        3,
        "repainting must not repeat diagnostics"
    );
}

#[test]
fn a_bubbled_region_key_keeps_its_original_canvas_after_a_callback_moves_focus() {
    let (screen, (inner_keys, outer_keys, ordinary_keys, owner_part_keys)) = mount(|| {
        let inner_keys = signal(0);
        let outer_keys = signal(0);
        let ordinary_keys = signal(0);
        let owner_part_keys = signal(0);
        let outer = part(|state| state)
            .role(Role::Button)
            .name("Outer region")
            .focusable(true)
            .on_key(move |_| {
                outer_keys.update(|n| *n += 1);
                Handled::No
            });
        let outer_ref = outer.node_ref();
        let inner = part(|state| state)
            .role(Role::Button)
            .name("Inner region")
            .focusable(true)
            .on_key(move |_| {
                inner_keys.update(|n| *n += 1);
                Handled::No
            });
        let inner_ref = inner.node_ref();
        let nested = canvas(|draw, _| {
            draw.region(RegionId(7), Rect::new(0.0, 0.0, 80.0, 40.0));
        })
        .region(RegionId(7), inner)
        .style(style().size(100, 40))
        .on_key(move |event| {
            if event.key == Key::ArrowDown {
                outer_ref.focus();
            }
            Handled::No
        });
        let view = column((
            canvas(|draw, _| {
                draw.region(RegionId(7), Rect::new(130.0, 0.0, 80.0, 40.0));
            })
            .region(RegionId(7), outer)
            .part(part(|state| state).on_key(move |_| {
                owner_part_keys.update(|n| *n += 1);
                Handled::No
            }))
            .on_key(move |_| {
                ordinary_keys.update(|n| *n += 1);
                Handled::No
            })
            .style(style().size(240, 50))
            .child(nested),
            node()
                .role(Role::Button)
                .name("Focus inner")
                .focusable(true)
                .on_press(move || inner_ref.focus())
                .style(style().size(100, 30)),
        ));
        (
            (inner_keys, outer_keys, ordinary_keys, owner_part_keys),
            view,
        )
    });
    screen
        .get_by_role(Role::Button, "Focus inner")
        .a11y(A11yAction::Activate);
    screen.press_key(Key::ArrowDown);
    assert_eq!(inner_keys.get(), 1);
    assert_eq!(
        outer_keys.get(),
        0,
        "another canvas may reuse the same region id"
    );
    assert_eq!(
        ordinary_keys.get(),
        1,
        "ordinary ancestor handlers still bubble"
    );
    assert_eq!(
        owner_part_keys.get(),
        1,
        "ancestor node parts receive bubbled input without retargeting its region"
    );
    assert!(
        screen
            .get_by_role(Role::Button, "Outer region")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
