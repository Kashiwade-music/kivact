//! Role: Tests for keyboard focus and focus management.
//! `NodeRef::focused` and `NodeRef::focus_within` follow Tab, clicks,
//! `NodeRef::focus`, unmounting and re-binding, and a focus move notifies
//! only the two affected nodes.

use crate::support::mount;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::View;
use kivact::view::{Handled, Placement, overlay};

fn states(values: &[Value<bool>]) -> Vec<bool> {
    values.iter().map(|v| v.get_untracked()).collect()
}

/// A global shortcut can identify externally authored form controls even
/// when their focused nodes are inside reusable components.
#[test]
fn bubbling_keys_preserve_the_original_targets_role() {
    let (screen, seen) = mount(|| {
        let seen = signal(Vec::new());
        let fields: Vec<View> = [
            Role::CheckBox,
            Role::RadioButton,
            Role::Slider,
            Role::ComboBox,
            Role::SpinButton,
            Role::Button,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, role)| {
            node()
                .role(role)
                .name(format!("field {index}"))
                .test_id(format!("field-{index}"))
                .focusable(true)
                .style(style().size(80, 24))
                .into_view()
        })
        .collect();
        (
            seen,
            node()
                .role(Role::Group)
                .on_key(move |event| {
                    if event.key == Key::Char('/') {
                        seen.update(|roles| roles.push(event.target_role));
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .child(column((
                    column(fields),
                    kivact::control::node().name("readonly target").part(
                        kivact::interaction::use_text_edit(signal(String::new()))
                            .read_only(true)
                            .control(),
                    ),
                    node()
                        .test_id("untyped target")
                        .focusable(true)
                        .style(style().size(80, 24)),
                ))),
        )
    });
    for index in 0..6 {
        screen.get_by_test_id(&format!("field-{index}")).focus();
        screen.press_key(Key::Char('/'));
    }
    screen.get_by_role(Role::TextBox, "readonly target").focus();
    screen.press_key(Key::Char('/'));
    screen.get_by_test_id("untyped target").click();
    screen.press_key(Key::Char('/'));
    assert_eq!(
        seen.get_untracked(),
        [
            Some(Role::CheckBox),
            Some(Role::RadioButton),
            Some(Role::Slider),
            Some(Role::ComboBox),
            Some(Role::SpinButton),
            Some(Role::Button),
            Some(Role::TextBox),
            None,
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reactive_focusability_supports_roving_tab_stops_without_remounting() {
    let (screen, selected) = mount(|| {
        let selected = signal(0usize);
        let tabs: Vec<View> = (0..3)
            .map(|i| {
                node()
                    .role(Role::Tab)
                    .name(format!("tab {i}"))
                    .test_id(format!("tab-{i}"))
                    .focusable(move || selected.get() == i)
                    .selected(move || selected.get() == i)
                    .child(text(format!("tab {i}")))
                    .into_view()
            })
            .collect();
        (
            selected,
            column((
                {
                    let label = ("before").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                },
                row(tabs),
                {
                    let label = ("after").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                },
            )),
        )
    });
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(screen.get_by_test_id("tab-0").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "after").is_focused());
    let created = screen.stats().creates;
    selected.set(2);
    screen.run_until_idle();
    screen.press_tab(true);
    assert!(screen.get_by_test_id("tab-2").is_focused());
    screen.press_tab(true);
    assert!(screen.get_by_role(Role::Button, "before").is_focused());
    assert_eq!(
        screen.stats().creates,
        created,
        "changing the tab stop does not remount"
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn focused_follows_tab_click_and_focus_requests() {
    let (screen, values) = mount(|| {
        let refs = [node_ref(), node_ref(), node_ref()];
        let values: Vec<Value<bool>> = refs.iter().map(|r| r.focused()).collect();
        let b = refs[1];
        let view = column((
            {
                let label = ("a").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(refs[0]),
            {
                let label = ("b").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(refs[1]),
            {
                let label = ("c").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(refs[2]),
            node()
                .test_id("request")
                .on_pointer(move |_| {
                    b.focus();
                    Handled::Yes
                })
                .child(text("request")),
        ));
        (values, view)
    });
    assert_eq!(states(&values), [false, false, false]);
    screen.press_tab(false);
    assert_eq!(states(&values), [true, false, false], "Tab");
    screen.press_tab(false);
    assert_eq!(states(&values), [false, true, false], "Tab again");
    screen.press_tab(true);
    assert_eq!(states(&values), [true, false, false], "Shift+Tab");
    screen.get_by_role(Role::Button, "c").click();
    assert_eq!(states(&values), [false, false, true], "click");
    // The box is not focusable: the click only runs the pointer handler,
    // which asks for the focus through the NodeRef.
    screen.get_by_test_id("request").click();
    assert_eq!(states(&values), [false, true, false], "NodeRef::focus");
}

#[test]
fn focus_within_follows_focus_into_and_out_of_a_subtree() {
    let (screen, (within, focused)) = mount(|| {
        let panel = node_ref();
        let within = panel.focus_within();
        let focused = panel.focused();
        let view = column((
            {
                let label = ("before").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
            node()
                .node_ref(panel)
                .focusable(true)
                .name("panel")
                .child(column((
                    kivact::control::node().name("inner").part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    ),
                    {
                        let label = ("inner button").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    },
                ))),
            {
                let label = ("after").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
        ));
        ((within, focused), view)
    });
    assert!(!within.get_untracked());
    screen.get_by_role(Role::Button, "before").click();
    assert!(!within.get_untracked(), "focus before the panel");
    screen.press_tab(false);
    assert!(
        within.get_untracked() && focused.get_untracked(),
        "the panel itself"
    );
    screen.press_tab(false);
    assert!(
        within.get_untracked() && !focused.get_untracked(),
        "a field inside"
    );
    screen.get_by_role(Role::Button, "inner button").click();
    assert!(within.get_untracked(), "a button inside");
    screen.press_tab(false);
    assert!(!within.get_untracked(), "focus moved out");
    screen.press_tab(true);
    assert!(within.get_untracked(), "back in with Shift+Tab");
}

#[test]
fn focus_within_includes_overlays_mounted_inside() {
    let (screen, within) = mount(|| {
        let menu = node_ref();
        let within = menu.focus_within();
        let view = column((
            node().node_ref(menu).child(column((
                {
                    let label = ("open").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                },
                overlay(menu, Placement::Below, {
                    let label = ("item").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }),
            ))),
            {
                let label = ("elsewhere").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
        ));
        (within, view)
    });
    screen.get_by_role(Role::Button, "item").click();
    assert!(within.get_untracked(), "focus in the overlay is inside");
    screen.get_by_role(Role::Button, "elsewhere").click();
    assert!(!within.get_untracked());
}

#[test]
fn unmounted_node_is_not_focused_and_a_rebound_ref_follows_the_new_node() {
    let (screen, (open, focused, within)) = mount(|| {
        let r = node_ref();
        // Asked before the ref is bound.
        let focused = r.focused();
        let within = r.focus_within();
        let open = signal(true);
        let view = column((
            show(open, move || {
                {
                    let label = ("target").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(r)
            }),
            {
                let label = ("other").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
        ));
        ((open, focused, within), view)
    });
    screen.get_by_role(Role::Button, "target").click();
    assert!(focused.get_untracked() && within.get_untracked());
    open.set(false);
    screen.run_until_idle();
    assert!(
        !focused.get_untracked() && !within.get_untracked(),
        "unmounted"
    );
    open.set(true);
    screen.run_until_idle();
    assert!(
        !focused.get_untracked(),
        "a new node: not focused until it gets the focus"
    );
    screen.get_by_role(Role::Button, "target").click();
    assert!(
        focused.get_untracked() && within.get_untracked(),
        "re-bound"
    );
    screen.get_by_role(Role::Button, "other").click();
    assert!(!focused.get_untracked());
}

#[test]
fn a_focus_move_reruns_only_the_two_affected_values() {
    const ROWS: usize = 30;
    let (screen, values) = mount(|| {
        let refs: Vec<NodeRef> = (0..ROWS).map(|_| node_ref()).collect();
        let values: Vec<Value<bool>> = refs.iter().map(|r| r.focused()).collect();
        let rows: Vec<View> = refs
            .iter()
            .zip(&values)
            .enumerate()
            .map(|(i, (r, f))| {
                let f = *f;
                node()
                    .node_ref(*r)
                    .focusable(true)
                    .name(format!("row {i}"))
                    .child(text(move || if f.get() { "*" } else { " " }.to_string()))
                    .into_view()
            })
            .collect();
        (values, column(rows))
    });
    // The only memos created in this file are the `focused` values.
    let memo_runs = |screen: &Screen, from: usize| -> usize {
        screen.trace()[from..]
            .iter()
            .flat_map(|t| t.reruns.iter())
            .filter(|r| r.kind == "memo" && r.at.file() == file!())
            .count()
    };
    let before = screen.trace().len();
    screen.press_tab(false);
    assert_eq!(
        memo_runs(&screen, before),
        1,
        "first focus: one node gains it"
    );
    for step in 0..5 {
        let before = screen.trace().len();
        screen.press_tab(false);
        assert_eq!(
            memo_runs(&screen, before),
            2,
            "focus move {step}: the node that lost and the node that gained it"
        );
    }
    let on: Vec<usize> = states(&values)
        .iter()
        .enumerate()
        .filter(|(_, f)| **f)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(on, [5]);
}

/// Turns since `from` that delivered a focus result (the answer to a focus
/// request).
fn focus_results(screen: &Screen, from: usize) -> usize {
    screen.trace()[from..]
        .iter()
        .flat_map(|t| t.causes.iter())
        .filter(|c| c.starts_with("FocusResult"))
        .count()
}

fn focused_name(screen: &Screen) -> Option<String> {
    screen.focused().map(|e| e.name())
}

#[test]
fn closing_a_modal_gives_the_focus_back_to_the_opener() {
    let (screen, (open, presses)) = mount(|| {
        let open = signal(false);
        let presses = signal(0);
        let anchor = node_ref();
        let view = column((
            {
                let label = ("open").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(anchor)
            .on_press(move || open.set(true)),
            {
                let label = ("behind").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || presses.update(|n| *n += 1)),
            show(open, move || {
                overlay(
                    anchor,
                    Placement::Below,
                    column((
                        kivact::control::node().name("name").part(
                            kivact::interaction::use_text_edit(kivact::signal(String::new()))
                                .control(),
                        ),
                        {
                            let label = ("close").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || open.set(false)),
                    )),
                )
                .modal(true)
            }),
            // Where the host itself would move the focus on removal.
            {
                let label = ("after").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
        ));
        ((open, presses), view)
    });
    screen.get_by_role(Role::Button, "open").click();
    assert!(open.get_untracked());
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("name"),
        "the host moves the focus into the modal"
    );
    // The modal blocks the window behind it: a press there lands on its
    // backdrop (a checked `click` would refuse the covered button).
    screen.get_by_role(Role::Button, "behind").force_click();
    assert_eq!(presses.get_untracked(), 0);
    assert_eq!(focused_name(&screen).as_deref(), Some("name"));
    screen.press_tab(false);
    assert_eq!(focused_name(&screen).as_deref(), Some("close"));
    screen.press_tab(false);
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("name"),
        "Tab stays inside"
    );
    let before = screen.trace().len();
    screen.get_by_role(Role::Button, "close").click();
    assert!(!open.get_untracked());
    assert_eq!(focused_name(&screen).as_deref(), Some("open"));
    assert_eq!(focus_results(&screen, before), 1, "one restore request");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.get_by_role(Role::Button, "behind").click();
    assert_eq!(presses.get_untracked(), 1, "input reaches the window again");
}

#[test]
fn a_removed_opener_gets_no_focus_request_and_no_error() {
    for same_turn in [false, true] {
        let (screen, (open, opener)) = mount(move || {
            let open = signal(false);
            let opener = signal(true);
            let anchor = node_ref();
            let view = column((
                show(opener, move || {
                    {
                        let label = ("open").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }
                    .node_ref(anchor)
                    .on_press(move || open.set(true))
                }),
                {
                    let label = ("rest").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                },
                show(open, move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("close").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || {
                            if same_turn {
                                opener.set(false);
                            }
                            open.set(false);
                        }),
                    )
                    .modal(true)
                }),
            ));
            ((open, opener), view)
        });
        screen.get_by_role(Role::Button, "open").click();
        assert_eq!(focused_name(&screen).as_deref(), Some("close"));
        if !same_turn {
            opener.set(false);
            screen.run_until_idle();
        }
        let before = screen.trace().len();
        screen.get_by_role(Role::Button, "close").click();
        assert!(!open.get_untracked() && !opener.get_untracked());
        assert_eq!(
            focus_results(&screen, before),
            0,
            "same turn {same_turn}: no request"
        );
        assert!(
            screen.errors().is_empty(),
            "same turn {same_turn}: {:?}",
            screen.errors()
        );
        assert_ne!(focused_name(&screen).as_deref(), Some("open"));
    }
}

#[test]
fn an_app_focus_request_in_the_closing_turn_wins_over_the_restore() {
    for focus_first in [false, true] {
        let (screen, open) = mount(move || {
            let open = signal(false);
            let anchor = node_ref();
            let other = node_ref();
            let view = column((
                {
                    let label = ("open").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(anchor)
                .on_press(move || open.set(true)),
                kivact::control::node()
                    .name("other")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .node_ref(other),
                show(open, move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("close").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || {
                            if focus_first {
                                other.focus();
                                open.set(false);
                            } else {
                                open.set(false);
                                other.focus();
                            }
                        }),
                    )
                    .modal(true)
                }),
            ));
            (open, view)
        });
        screen.get_by_role(Role::Button, "open").click();
        assert_eq!(focused_name(&screen).as_deref(), Some("close"));
        let before = screen.trace().len();
        screen.get_by_role(Role::Button, "close").click();
        assert!(!open.get_untracked());
        assert_eq!(
            focused_name(&screen).as_deref(),
            Some("other"),
            "focus first {focus_first}: the app's request wins"
        );
        assert_eq!(
            focus_results(&screen, before),
            1,
            "focus first {focus_first}: only the app's request is sent"
        );
        assert!(
            screen.errors().is_empty(),
            "focus first {focus_first}: {:?}",
            screen.errors()
        );
    }
}

/// Tab skips static text, hidden subtrees and disabled buttons and wraps
/// both ways; when the focused node is removed, disabled or hidden, the
/// host moves the focus to the next shown tab stop after it at the end of
/// the batch and scrolls it into view.
#[test]
fn a_lost_focus_moves_to_the_next_shown_tab_stop_and_is_revealed() {
    let (screen, (x_on, y_off, z_hidden)) = mount(|| {
        let (x_on, y_off, z_hidden) = (signal(true), signal(false), signal(false));
        let view = column((
            {
                let label = ("a").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
            show(x_on, || {
                let label = ("x").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }),
            node().hidden(true).child({
                let label = ("hidden").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }),
            text("static"),
            {
                let label = ("disabled").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .disabled(true),
            scroll(column((
                node().style(style().height(300).shrink(0.0)),
                {
                    let label = ("y").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .disabled(y_off),
            )))
            .test_id("pane")
            .style(style().height(100)),
            node().hidden(z_hidden).child({
                let label = ("z").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }),
        ));
        ((x_on, y_off, z_hidden), view)
    });
    let mut order = Vec::new();
    for _ in 0..5 {
        screen.press_tab(false);
        order.push(focused_name(&screen).unwrap_or_default());
    }
    assert_eq!(order, ["a", "x", "y", "z", "a"], "Tab wraps");
    screen.press_tab(true);
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("z"),
        "Shift+Tab wraps"
    );

    let pane = screen.get_by_test_id("pane");
    pane.scroll(0.0, -1000.0);
    let inside = |id: &str| {
        let (p, r) = (
            pane.rect().expect("laid out"),
            screen
                .get_by_role(Role::Button, id)
                .rect()
                .expect("laid out"),
        );
        r.y >= p.y - 0.5 && r.y + r.height <= p.y + p.height + 0.5
    };
    assert!(!inside("y"), "scrolled away");
    screen.get_by_role(Role::Button, "x").click();
    x_on.set(false);
    screen.run_until_idle();
    assert_eq!(focused_name(&screen).as_deref(), Some("y"), "removed");
    assert!(inside("y"), "the moved focus is scrolled into view");
    y_off.set(true);
    screen.run_until_idle();
    assert_eq!(focused_name(&screen).as_deref(), Some("z"), "disabled");
    z_hidden.set(true);
    screen.run_until_idle();
    assert_eq!(focused_name(&screen).as_deref(), Some("a"), "hidden");
}

/// Explicit focus reveals a scroll-clipped target before normal pointer input,
/// including a target which already owns focus but was scrolled out of view.
#[test]
fn explicit_focus_reveals_a_scroll_clipped_button() {
    let (screen, presses) = mount(|| {
        let presses = signal(0);
        let view = scroll(column((
            node().style(style().height(400).shrink(0.0)),
            node()
                .role(Role::Button)
                .focusable(true)
                .name("offscreen target")
                .style(style().height(32).shrink(0.0))
                .child(text("offscreen target"))
                .on_press(move || presses.update(|n| *n += 1)),
        )))
        .test_id("focus reveal pane")
        .style(style().height(100).shrink(0.0));
        (presses, view)
    });
    let pane = screen.get_by_test_id("focus reveal pane");
    let target = screen.get_by_role(Role::Button, "offscreen target");
    let inside = || {
        let p = pane.rect().expect("pane is laid out");
        let r = target.rect().expect("button is laid out");
        r.y >= p.y - 0.5 && r.y + r.height <= p.y + p.height + 0.5
    };
    assert!(!inside(), "the target starts below the scroll viewport");
    target.focus();
    assert!(inside(), "explicit focus reveals the complete button");
    assert_eq!(focused_name(&screen).as_deref(), Some("offscreen target"));
    target.click();
    assert_eq!(
        presses.get(),
        1,
        "the revealed button accepts a normal click"
    );
    pane.scroll(0.0, -1000.0);
    assert!(!inside(), "scrolling can hide the focused button");
    target.focus();
    assert!(inside(), "requesting the same focus reveals it again");
}

/// A reveal keeps the scroll container's padding as a gutter for focus
/// rings and scrollbars instead of pushing the target to the border edge.
#[test]
fn a_reveal_keeps_the_scroll_containers_padding() {
    let (screen, ()) = mount(|| {
        let view = scroll(column((
            node().style(style().height(400).shrink(0.0)),
            node()
                .role(Role::Button)
                .focusable(true)
                .name("padded target")
                .style(style().height(32).shrink(0.0))
                .child(text("padded target")),
            node().style(style().height(400).shrink(0.0)),
        )))
        .test_id("padded pane")
        .style(style().height(120).padding(20).shrink(0.0));
        ((), view)
    });
    let pane = screen.get_by_test_id("padded pane");
    let target = screen.get_by_role(Role::Button, "padded target");
    target.focus();
    let p = pane.rect().expect("pane is laid out");
    let r = target.rect().expect("button is laid out");
    assert!(
        r.y >= p.y + 20.0 - 0.5 && r.y + r.height <= p.y + p.height - 20.0 + 0.5,
        "the target stays inside the content box: pane {p:?} target {r:?}"
    );
}

/// A modal's focus stays inside it: removing its focused node moves the
/// focus to its next tab stop, a stacked modal confines Tab until it
/// closes, and a hidden modal confines nothing.
#[test]
fn stacked_and_hidden_modals_keep_the_focus_where_it_belongs() {
    let (screen, (second_on, hide_dialog)) = mount(|| {
        let (second_on, inner, hide_dialog) = (signal(true), signal(false), signal(false));
        let presses = signal(0);
        let view = column((
            {
                let label = ("behind").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || presses.update(|n| *n += 1)),
            text(move || format!("presses {}", presses.get())),
            overlay(
                node_ref(),
                Placement::Center,
                column((
                    {
                        let label = ("first").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    },
                    show(second_on, || {
                        let label = ("second").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }),
                    {
                        let label = ("open inner").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }
                    .on_press(move || inner.set(true)),
                )),
            )
            .modal(true)
            .hidden(hide_dialog),
            show(inner, move || {
                overlay(
                    node_ref(),
                    Placement::Center,
                    column((
                        {
                            let label = ("inner a").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        },
                        {
                            let label = ("inner close").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || inner.set(false)),
                    )),
                )
                .modal(true)
            }),
        ));
        ((second_on, hide_dialog), view)
    });
    screen.get_by_role(Role::Button, "second").click();
    second_on.set(false);
    screen.run_until_idle();
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("open inner"),
        "next stop inside"
    );
    screen.press_tab(false);
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("first"),
        "Tab wraps inside"
    );

    screen.get_by_role(Role::Button, "open inner").click();
    let mut order = Vec::new();
    for _ in 0..3 {
        screen.press_tab(false);
        order.push(focused_name(&screen).unwrap_or_default());
    }
    assert!(
        order.iter().all(|n| n.starts_with("inner")),
        "the topmost modal confines Tab: {order:?}"
    );
    screen.get_by_role(Role::Button, "inner close").click();
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("open inner"),
        "restored"
    );
    screen.get_by_role(Role::Button, "behind").force_click();
    screen.get_by_text("presses 0");

    hide_dialog.set(true);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "first").is_none());
    assert_eq!(
        focused_name(&screen).as_deref(),
        Some("behind"),
        "the focus leaves the hidden modal for the page"
    );
    screen.get_by_role(Role::Button, "behind").click();
    screen.get_by_text("presses 1");
}
