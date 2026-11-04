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
                    kivact::view::node().name("inner").part(
                        kivact::interaction::use_text_edit(kivact::state::signal(String::new())).control(),
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
                        kivact::view::node().name("name").part(
                            kivact::interaction::use_text_edit(kivact::state::signal(String::new()))
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
                .modal()
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
