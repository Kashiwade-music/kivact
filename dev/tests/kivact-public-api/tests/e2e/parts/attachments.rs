//! Role: Part attachments preserve lifetimes and ancestor input routing.

use kivact::primitive::NodeExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::{NodeRefExt, part};
use kivact::view::{Handled, Placement, callback, overlay};
use kivact::view::{KeyCode, Modifiers};

#[test]
fn descendant_region_keys_and_wheels_bubble_to_ancestor_parts_without_retargeting() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = calls.clone();
    let (screen, (consume, parent_state, child_state)) = mount(move || {
        let consume = signal(false);
        let record = callback(move |entry| output.borrow_mut().push(entry));
        let parent = part(|state| state)
            .on_key(move |event| {
                record.call(("parent key", event.region));
                Handled::No
            })
            .on_key_up(move |event| {
                record.call(("parent up", event.region));
                Handled::No
            })
            .on_wheel(move |event| {
                record.call(("parent wheel", event.region()));
                Handled::No
            });
        let parent_state = parent.state();
        let child = part(|state| state)
            .role(Role::Button)
            .name("Nested input region")
            .focusable(true)
            .on_key(move |event| {
                record.call(("child key", event.region));
                Handled::No
            })
            .on_key_up(move |event| {
                record.call(("child up", event.region));
                Handled::No
            })
            .on_wheel(move |event| {
                record.call(("child wheel", event.region()));
                Handled::No
            });
        let child_state = child.state();
        let child_owner = part(|state| state)
            .on_key(move |event| {
                record.call(("child owner key", event.region));
                Handled::No
            })
            .on_key_up(move |event| {
                record.call(("child owner up", event.region));
                Handled::No
            })
            .on_wheel(move |event| {
                record.call(("child owner wheel", event.region()));
                Handled::No
            });
        let unrelated = part(|state| state)
            .role(Role::Button)
            .name("Ancestor own region")
            .focusable(true)
            .on_key(move |event| {
                record.call(("own key", event.region));
                Handled::No
            })
            .on_key_up(move |event| {
                record.call(("own up", event.region));
                Handled::No
            })
            .on_wheel(move |event| {
                record.call(("own wheel", event.region()));
                Handled::No
            });
        let nested = canvas(|draw, _| {
            draw.region(RegionId(7), Rect::new(0.0, 0.0, 80.0, 40.0));
        })
        .region(RegionId(7), child)
        .part(child_owner)
        .on_key(move |_| {
            if consume.get() {
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .on_key_up(move |_| {
            if consume.get() {
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .on_wheel(move |_| {
            if consume.get() {
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .style(style().size(100, 40));
        let view = canvas(|draw, _| {
            draw.region(RegionId(7), Rect::new(130.0, 0.0, 80.0, 40.0));
        })
        .region(RegionId(7), unrelated)
        .part(parent)
        .child(nested)
        .style(style().size(240, 60));
        ((consume, parent_state, child_state), view)
    });
    let child = screen.get_by_role(Role::Button, "Nested input region");
    child.focus();
    screen.key_down(Key::Escape, KeyCode::Escape, Modifiers::default());
    screen.key_up(Key::Escape, KeyCode::Escape, Modifiers::default());
    assert_eq!(
        &*calls.borrow(),
        &[
            ("child key", Some(RegionId(7))),
            ("parent key", Some(RegionId(7))),
            ("child up", Some(RegionId(7))),
            ("parent up", Some(RegionId(7))),
        ]
    );
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(child_state.get().pressed);
    assert!(
        !parent_state.get().pressed,
        "ancestor defaults do not observe descendant key state"
    );
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    calls.borrow_mut().clear();
    child.scroll(0.0, 24.0);
    assert_eq!(
        &*calls.borrow(),
        &[
            ("child wheel", Some(RegionId(7))),
            ("parent wheel", Some(RegionId(7)))
        ]
    );
    calls.borrow_mut().clear();
    consume.set(true);
    screen.key_down(Key::Escape, KeyCode::Escape, Modifiers::default());
    screen.key_up(Key::Escape, KeyCode::Escape, Modifiers::default());
    child.scroll(0.0, 24.0);
    assert!(
        calls.borrow().is_empty(),
        "consumed child input stops ancestor defaults"
    );
    consume.set(false);
    let own = screen.get_by_role(Role::Button, "Ancestor own region");
    own.focus();
    screen.key_down(Key::Escape, KeyCode::Escape, Modifiers::default());
    screen.key_up(Key::Escape, KeyCode::Escape, Modifiers::default());
    own.scroll(0.0, 24.0);
    assert_eq!(
        &*calls.borrow(),
        &[
            ("own key", Some(RegionId(7))),
            ("own up", Some(RegionId(7))),
            ("own wheel", Some(RegionId(7))),
        ],
        "an owner Part never receives defaults from its own canvas region"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn composed_parts_keep_both_states_refs_constraints_and_callback_lifetimes() {
    for region in [false, true] {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let cleanups = Rc::new(Cell::new(0));
        let (log, cleaned) = (calls.clone(), cleanups.clone());
        let (screen, (first, second, first_ref, second_ref, disabled, hidden, consume, visible)) =
            mount(move || {
                let disabled = signal(false);
                let hidden = signal(false);
                let consume = signal(false);
                let visible = signal(true);
                let own = log.clone();
                let first_part = part(|state| state)
                    .role(Role::Button)
                    .name("Receiver")
                    .focusable(true)
                    .on_key(move |_| {
                        own.borrow_mut().push("first");
                        Handled::No
                    });
                let first_part =
                    first_part.map_state(|state| (state.pressed, state.disabled, state.hidden));
                let first = first_part.state();
                let first_ref = first_part.node_ref();
                let own = cleaned.clone();
                let first_part = first_part.on_bind(move |_| {
                    let own = own.clone();
                    on_cleanup(move || own.set(own.get() + 1));
                });
                let other = log.clone();
                let second_part = part(|state| state)
                    .name("Composed target")
                    .focusable(true)
                    .disabled(disabled)
                    .hidden(hidden)
                    .on_key(move |_| {
                        other.borrow_mut().push("second");
                        Handled::No
                    });
                let second = second_part.state();
                let second_ref = second_part.node_ref();
                let second_part = second_part.on_bind(move |_| {
                    let cleaned = cleaned.clone();
                    on_cleanup(move || cleaned.set(cleaned.get() + 1));
                });
                let combined = first_part.with(second_part);
                let target = if region {
                    canvas(|draw, _| {
                        draw.region(RegionId(1), Rect::new(0.0, 0.0, 100.0, 40.0));
                    })
                    .region(RegionId(1), combined)
                } else {
                    node().part(combined)
                };
                let target = target
                    .on_key(move |_| {
                        log.borrow_mut().push("caller");
                        if consume.get() {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    })
                    .style(style().size(140, 60));
                let target = RefCell::new(Some(target));
                (
                    (
                        first, second, first_ref, second_ref, disabled, hidden, consume, visible,
                    ),
                    show(visible, move || {
                        target.borrow_mut().take().expect("opens once")
                    }),
                )
            });
        assert_eq!(first_ref.mounted_node(), second_ref.mounted_node());
        first_ref.focus();
        screen.run_until_idle();
        assert!(second.get().focused);
        second_ref.focus();
        screen.run_until_idle();
        assert!(
            screen
                .get_by_role(Role::Button, "Composed target")
                .is_focused()
        );
        screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
        assert!(first.get().0 && second.get().pressed);
        assert_eq!(&*calls.borrow(), &["caller", "first", "second"]);
        screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
        assert!(!first.get().0 && !second.get().pressed);
        calls.borrow_mut().clear();
        consume.set(true);
        screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
        assert!(first.get().0 && second.get().pressed);
        assert_eq!(&*calls.borrow(), &["caller"]);
        screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
        disabled.set(true);
        screen.run_until_idle();
        assert!(first.get().1 && second.get().disabled);
        disabled.set(false);
        hidden.set(true);
        screen.run_until_idle();
        assert!(first.get().2 && second.get().hidden);
        visible.set(false);
        screen.run_until_idle();
        assert_eq!(cleanups.get(), 2);
        assert!(first_ref.mounted_node().is_none() && second_ref.mounted_node().is_none());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_part_modal_requirement_confines_focus_and_restores_the_opener() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let opener = node_ref();
        let view = column((
            node()
                .role(Role::Button)
                .name("Part modal opener")
                .focusable(true)
                .node_ref(opener)
                .on_press(move || open.set(true))
                .style(style().size(150, 40)),
            node()
                .role(Role::Button)
                .name("Outside modal")
                .focusable(true)
                .style(style().size(150, 40)),
            show(open, move || {
                overlay(
                    opener,
                    Placement::Below,
                    column((
                        kivact::control::node().name("Part modal first").part(
                            kivact::interaction::use_text_edit(kivact::signal(String::new()))
                                .control(),
                        ),
                        node()
                            .role(Role::Button)
                            .name("Part modal close")
                            .focusable(true)
                            .on_press(move || open.set(false))
                            .style(style().size(150, 40)),
                    )),
                )
                .part(part(|state| state).modal(true))
                .modal(false)
            }),
        ));
        (open, view)
    });
    screen
        .get_by_role(Role::Button, "Part modal opener")
        .click();
    assert!(open.get());
    assert!(
        screen
            .get_by_role(Role::TextBox, "Part modal first")
            .is_focused()
    );
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::Button, "Part modal close")
            .is_focused()
    );
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::TextBox, "Part modal first")
            .is_focused()
    );
    screen.get_by_role(Role::Button, "Part modal close").click();
    assert!(!open.get());
    assert!(
        screen
            .get_by_role(Role::Button, "Part modal opener")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn modal_parts_on_nodes_and_regions_report_unsupported_without_trapping_focus() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                node()
                    .part(part(|state| state).modal(true))
                    .role(Role::Button)
                    .name("Unsupported modal node")
                    .focusable(true)
                    .style(style().size(180, 40)),
                canvas(|draw, _| {
                    draw.region(RegionId(1), Rect::new(0.0, 0.0, 80.0, 40.0));
                })
                .region(
                    RegionId(1),
                    part(|state| state)
                        .modal(true)
                        .role(Role::Button)
                        .name("Unsupported modal region")
                        .focusable(true),
                )
                .style(style().size(180, 50)),
                node()
                    .role(Role::Button)
                    .name("After unsupported modal")
                    .focusable(true)
                    .style(style().size(180, 40)),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Unsupported modal node")
        .focus();
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::Button, "Unsupported modal region")
            .is_focused()
    );
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::Button, "After unsupported modal")
            .is_focused()
    );
    assert_eq!(screen.errors().len(), 2);
    assert!(
        screen
            .errors()
            .iter()
            .all(|error| matches!(error.kind(), kivact::diagnostics::ErrorKind::Unsupported))
    );
}
