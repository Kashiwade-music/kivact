//! Role: Shared keyed selection and controlled popup focus.

use kivact::primitive::NodeExt;
use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::Binding;
use kivact::binding;
use kivact::interaction::{
    FocusReturn, List, ListCommandModifier, ListGesture, ListMissingAnchor, ListPolicy,
    ListRangeMode, ListRangePolicy, ListSelection, PopupCloseReason, use_list, use_popup,
};
use kivact::prelude::*;
use kivact::ui::ListItem;
use kivact::view::Modifiers;
use kivact::view::{A11yAction, Placement, overlay};

fn selection_items(
    prefix: &'static str,
    visible: Signal<Vec<String>>,
    selected: Binding<Vec<String>>,
    anchor: Binding<Option<String>>,
) -> (List<String>, View) {
    let selection = use_list(
        move || {
            visible
                .get()
                .into_iter()
                .map(|key| ListItem::new(key.clone(), key))
                .collect::<Vec<_>>()
        },
        ListSelection::Multiple(selected),
    )
    .anchor(anchor);
    let view = column(for_each(visible, Clone::clone, move |key, _| {
        node()
            .part(selection.item(key.clone()).name(format!("{prefix}-{key}")))
            .style(style().size(140, 24))
    }))
    .part(selection.list())
    .into_view();
    (selection, view)
}

#[test]
fn table_and_thumbnail_share_keys_across_order_filter_and_gestures() {
    let (screen, (selected, visible, selection, table, thumb)) = mount(|| {
        let selected = signal(vec!["offscreen".to_string()]);
        let visible = signal(vec!["a".into(), "b".into(), "c".into(), "d".into()]);
        let anchor = signal(None::<String>);
        let selection = use_list(
            move || {
                visible
                    .get()
                    .into_iter()
                    .map(|key| ListItem::new(key, ""))
                    .collect::<Vec<_>>()
            },
            ListSelection::Multiple(selected.into()),
        )
        .anchor(anchor);
        let (table, table_view) = selection_items("table", visible, selected.into(), anchor.into());
        let (thumb, thumb_view) = selection_items("thumb", visible, selected.into(), anchor.into());
        (
            (selected, visible, selection, table, thumb),
            row((table_view, thumb_view)),
        )
    });
    screen.get_by_role(Role::Option, "table-b").click();
    assert_eq!(selected.get_untracked(), ["b"]);
    assert!(screen.get_by_role(Role::Option, "thumb-b").is_selected());
    assert!(thumb.focus("d".to_owned()));
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "thumb-d").press_key_with(
        Key::Space,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(selected.get_untracked(), ["b", "c", "d"]);
    assert_eq!(
        selection.anchor_value().get_untracked().as_deref(),
        Some("b")
    );
    selected.set(vec!["offscreen".into(), "b".into()]);
    screen.run_until_idle();
    assert!(table.focus("d".to_owned()));
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "table-d").press_key_with(
        Key::Space,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(selected.get_untracked(), ["offscreen", "b", "c", "d"]);
    assert!(thumb.focus("c".to_owned()));
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "thumb-c").press_key_with(
        Key::Space,
        Modifiers {
            meta: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(selected.get_untracked(), ["offscreen", "b", "d"]);
    visible.set(vec!["d".into(), "a".into()]);
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), ["offscreen", "b", "d"]);
    assert!(table.focus("a".to_owned()));
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "table-a").press_key_with(
        Key::Space,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(
        selected.get_untracked(),
        ["a"],
        "missing anchor starts at target"
    );
    visible.set(vec!["a".into(), "d".into(), "b".into()]);
    screen.run_until_idle();
    screen
        .get_by_role(Role::Option, "thumb-b")
        .a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), ["b"]);
    assert!(screen.get_by_role(Role::Option, "table-b").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn selection_explicit_policy_keeps_hidden_keys_and_uses_visible_fallback() {
    let (screen, selected) = mount(|| {
        let selected = signal(vec!["hidden".to_owned(), "b".to_owned(), "b".to_owned()]);
        let selection = use_list(
            move || {
                (vec![
                    "c".to_owned(),
                    "b".to_owned(),
                    "b".to_owned(),
                    "a".to_owned(),
                ])
                .into_iter()
                .map(|key| ListItem::new(key, ""))
                .collect::<Vec<_>>()
            },
            ListSelection::Multiple((selected).into()),
        )
        .policy(ListPolicy {
            range: ListRangePolicy {
                shift: ListRangeMode::Add,
                command_shift: ListRangeMode::Replace,
                missing_anchor: ListMissingAnchor::FirstSelected,
                command: ListCommandModifier::Meta,
            },
            ..ListPolicy::default()
        });
        (
            selected,
            column((
                {
                    let label = ("Extend visible").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    selection.select(
                        "a".into(),
                        ListGesture::Extend {
                            mode: ListRangeMode::Add,
                        },
                    );
                }),
                {
                    let label = ("Replace command range").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    selection.select(
                        "c".into(),
                        ListGesture::Extend {
                            mode: ListRangeMode::Replace,
                        },
                    );
                }),
                {
                    let label = ("Missing target").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    selection.select("missing".into(), ListGesture::Replace);
                }),
                {
                    let label = ("Clear shared selection").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || selection.clear()),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Extend visible").click();
    assert_eq!(selected.get_untracked(), ["hidden", "b", "a"]);
    screen
        .get_by_role(Role::Button, "Replace command range")
        .click();
    assert_eq!(selected.get_untracked(), ["c", "b"]);
    screen.get_by_role(Role::Button, "Missing target").click();
    assert_eq!(selected.get_untracked(), ["c", "b"]);
    screen
        .get_by_role(Role::Button, "Clear shared selection")
        .click();
    assert!(selected.get_untracked().is_empty());
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message().contains("duplicate logical"));
}

#[test]
fn a_missing_selection_target_does_not_offer_a_pair_binding_write_or_change_the_anchor() {
    let (screen, (selected, offers, selection)) = mount(|| {
        let selected = signal(vec!["hidden".to_owned()]);
        let offers = signal(0);
        let binding = binding(selected.into_value(), move |requested| {
            offers.update(|count| *count += 1);
            selected.set(requested);
        });
        let selection = use_list(
            move || {
                (vec!["b".to_owned(), "c".to_owned()])
                    .into_iter()
                    .map(|key| ListItem::new(key, ""))
                    .collect::<Vec<_>>()
            },
            ListSelection::Multiple(binding),
        );
        (
            (selected, offers, selection),
            column((
                {
                    let label = ("Establish selection anchor").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    selection.select("b".into(), ListGesture::Replace);
                }),
                {
                    let label = ("Offer absent selection target").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    for gesture in [
                        ListGesture::Replace,
                        ListGesture::Toggle,
                        ListGesture::Extend {
                            mode: ListRangeMode::Replace,
                        },
                        ListGesture::Extend {
                            mode: ListRangeMode::Add,
                        },
                    ] {
                        selection.select("missing".into(), gesture);
                    }
                }),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Establish selection anchor")
        .click();
    assert_eq!(offers.get_untracked(), 1);
    assert_eq!(
        selection.anchor_value().get_untracked().as_deref(),
        Some("b")
    );
    offers.set(0);
    selected.set(vec!["hidden".into(), "b".into(), "b".into()]);
    screen.run_until_idle();
    screen
        .get_by_role(Role::Button, "Offer absent selection target")
        .click();
    assert_eq!(
        offers.get_untracked(),
        0,
        "missing targets never call the writer"
    );
    assert_eq!(
        selection.anchor_value().get_untracked().as_deref(),
        Some("b")
    );
    assert_eq!(selected.get_untracked(), ["hidden", "b", "b"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_popup_trigger_part_toggles_controls_the_surface_and_takes_focus_back() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let menu = use_popup(open);
        let trigger = menu.trigger();
        let anchor = trigger.node_ref();
        (
            open,
            column((
                node()
                    .part(trigger)
                    .name("Options")
                    .style(style().size(120, 30))
                    .child(text("Options")),
                show(menu.shown(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        node()
                            .role(Role::Button)
                            .name("Inside")
                            .focusable(true)
                            .child(text("Inside")),
                    )
                    .part(menu.content())
                    .role(Role::Dialog)
                    .name("Options panel")
                }),
            )),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Options");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(
        trigger.controls().is_empty(),
        "nothing to control while closed"
    );
    trigger.focus();
    trigger.click();
    assert!(open.get_untracked());
    assert_eq!(trigger.is_expanded(), Some(true));
    let controlled: Vec<String> = trigger.controls().iter().map(|e| e.name()).collect();
    assert_eq!(controlled, ["Options panel"]);
    trigger.click();
    assert!(!open.get_untracked(), "a second activation closes it");
    trigger.a11y(A11yAction::Expand);
    assert!(open.get_untracked());
    trigger.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    trigger.press_key(Key::Enter);
    assert!(open.get_untracked());
    screen.get_by_role(Role::Button, "Inside").focus();
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(
        trigger.is_focused(),
        "Escape returns the focus to the trigger"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn popup_reopening_cleans_content_and_close_reasons_preserve_focus() {
    let local_values = Rc::new(RefCell::new(Vec::new()));
    let locals = local_values.clone();
    let (screen, (open, mounted, cleaned)) = mount(move || {
        let open = signal(false);
        let mounted = signal(0);
        let cleaned = signal(0);
        let anchor = node_ref();
        let first = node_ref();
        let popup = use_popup(open).anchor(anchor).initial_focus(first);
        (
            (open, mounted, cleaned),
            column((
                {
                    let label = ("Open popup").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(anchor)
                .on_press(move || open.set(true)),
                show(popup.shown(), move || {
                    mounted.update(|n| *n += 1);
                    let local = signal(mounted.get_untracked());
                    locals.borrow_mut().push(local);
                    on_cleanup(move || cleaned.update(|n| *n += 1));
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("Choose popup item").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(first)
                        .on_press(move || popup.close(PopupCloseReason::Selection)),
                    )
                    .part(popup.content())
                    .role(Role::Menu)
                    .name("Shared popup")
                    .offset(8)
                }),
                {
                    let label = ("Popup outside").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .style(style().margin_top(100)),
            )),
        )
    });
    for reason in 0..7 {
        screen.get_by_role(Role::Button, "Open popup").click();
        assert!(
            screen
                .get_by_role(Role::Button, "Choose popup item")
                .is_focused()
        );
        match reason {
            0 => screen.press_key(Key::Escape),
            1 => screen
                .get_by_role(Role::Menu, "Shared popup")
                .a11y(A11yAction::Dismiss),
            2 => screen
                .get_by_role(Role::Button, "Choose popup item")
                .click(),
            3 => screen.get_by_role(Role::Button, "Popup outside").click(),
            4 => screen.press_tab(false),
            5 => {
                screen.get_by_role(Role::Button, "Open popup").focus();
                assert!(
                    open.get_untracked(),
                    "the exact anchor stays inside the popup boundary"
                );
                screen.press_key(Key::Escape);
            }
            _ => {
                screen.get_by_role(Role::Button, "Open popup").focus();
                assert!(
                    open.get_untracked(),
                    "returning to the anchor keeps content mounted"
                );
                screen.get_by_role(Role::Button, "Popup outside").focus();
            }
        }
        assert!(!open.get_untracked());
        assert!(screen.query_by_role(Role::Menu, "Shared popup").is_none());
        let target = if reason < 3 || reason == 5 {
            "Open popup"
        } else {
            "Popup outside"
        };
        assert!(screen.get_by_role(Role::Button, target).is_focused());
        assert_eq!(mounted.get_untracked(), reason + 1);
        assert_eq!(cleaned.get_untracked(), reason + 1);
        assert!(local_values.borrow().last().unwrap().try_get().is_none());
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn nested_popup_dismisses_topmost_and_modal_restores_the_actual_opener() {
    let (screen, (parent_open, child_open)) = mount(|| {
        let parent_open = signal(false);
        let child_open = signal(false);
        let parent_anchor = node_ref();
        let child_anchor = node_ref();
        let child_first = node_ref();
        let parent = use_popup(parent_open)
            .anchor(parent_anchor)
            .initial_focus(child_anchor);
        let child = use_popup(child_open)
            .anchor(child_anchor)
            .initial_focus(child_first);
        (
            (parent_open, child_open),
            column((
                {
                    let label = ("Modal anchor").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(parent_anchor),
                {
                    let label = ("Actual modal opener").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || parent_open.set(true)),
                show(parent.shown(), move || {
                    overlay(
                        parent_anchor,
                        Placement::Below,
                        column((
                            {
                                let label = ("Nested popup anchor").into_value();
                                node()
                                    .role(Role::Button)
                                    .focusable(true)
                                    .name(label)
                                    .child(text(label))
                            }
                            .node_ref(child_anchor)
                            .on_press(move || child_open.set(true)),
                            show(child.shown(), move || {
                                overlay(
                                    child_anchor,
                                    Placement::End,
                                    {
                                        let label = ("Nested popup choice").into_value();
                                        node()
                                            .role(Role::Button)
                                            .focusable(true)
                                            .name(label)
                                            .child(text(label))
                                    }
                                    .node_ref(child_first),
                                )
                                .part(child.content())
                                .role(Role::Menu)
                                .name("Nested popup")
                            }),
                        )),
                    )
                    .part(parent.content())
                    .role(Role::Dialog)
                    .name("Parent popup")
                    .modal(true)
                }),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Actual modal opener")
        .click();
    screen
        .get_by_role(Role::Button, "Nested popup anchor")
        .click();
    assert!(
        screen
            .get_by_role(Role::Button, "Nested popup choice")
            .is_focused()
    );
    screen.press_key(Key::Escape);
    assert!(parent_open.get_untracked());
    assert!(!child_open.get_untracked());
    assert!(
        screen
            .get_by_role(Role::Button, "Nested popup anchor")
            .is_focused()
    );
    screen.press_key(Key::Escape);
    assert!(!parent_open.get_untracked());
    assert!(
        screen
            .get_by_role(Role::Button, "Actual modal opener")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn popup_anchor_removal_and_a_rejected_close_do_not_request_stale_focus() {
    let (screen, open) = mount(|| {
        let mounted = signal(true);
        let open = signal(false);
        let anchor = node_ref();
        let first = node_ref();
        let popup = use_popup(open).anchor(anchor).initial_focus(first);
        (
            open,
            column((
                show(mounted, move || {
                    {
                        let label = ("Removable anchor").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }
                    .node_ref(anchor)
                    .on_press(move || open.set(true))
                }),
                show(popup.shown(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("Remove popup anchor").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(first)
                        .on_press(move || mounted.set(false)),
                    )
                    .part(popup.content())
                    .role(Role::Menu)
                    .name("Removed anchor popup")
                }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Removable anchor").click();
    screen
        .get_by_role(Role::Button, "Remove popup anchor")
        .click();
    assert!(!open.get_untracked());
    assert!(
        screen
            .query_by_role(Role::Menu, "Removed anchor popup")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());

    let (screen, open) = mount(|| {
        let open = signal(false);
        let binding = binding(open.into_value(), move |requested| {
            if requested {
                open.set(true);
            }
        });
        let anchor = node_ref();
        let first = node_ref();
        let popup = use_popup(binding).anchor(anchor).initial_focus(first);
        (
            open,
            column((
                {
                    let label = ("Controlled popup anchor").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(anchor)
                .on_press(move || binding.set(true)),
                show(popup.shown(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("Controlled popup choice").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(first),
                    )
                    .part(popup.content())
                    .role(Role::Menu)
                    .name("Controlled popup")
                }),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Controlled popup anchor")
        .click();
    screen.press_key(Key::Escape);
    assert!(open.get_untracked());
    assert!(
        screen
            .get_by_role(Role::Button, "Controlled popup choice")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn popup_preserves_user_dismissal_errors_and_explicit_focus_policy() {
    let (screen, (open, dismissed)) = mount(|| {
        let open = signal(false);
        let dismissed = signal(0);
        let anchor = node_ref();
        let first = node_ref();
        let popup = use_popup(open).anchor(anchor).initial_focus(first);
        (
            (open, dismissed),
            column((
                {
                    let label = ("Custom dismissal anchor").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(anchor)
                .on_press(move || open.set(true)),
                show(popup.shown(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("Custom dismissal choice").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(first),
                    )
                    .on_dismiss(move |_| {
                        dismissed.update(|count| *count += 1);
                        open.set(false);
                    })
                    .part(popup.content())
                    .role(Role::Menu)
                    .name("Custom dismissal popup")
                }),
                {
                    let label = ("Custom dismissal outside").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .style(style().margin_top(80)),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Custom dismissal anchor")
        .click();
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert_eq!(dismissed.get_untracked(), 1);
    assert!(
        screen
            .get_by_role(Role::Button, "Custom dismissal anchor")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());

    let (screen, open) = mount(|| {
        let open = signal(false);
        let fail = signal(true);
        let anchor = node_ref();
        let first = node_ref();
        let popup = use_popup(open)
            .anchor(anchor)
            .initial_focus(first)
            .focus_return(FocusReturn::preserve());
        (
            open,
            column((
                {
                    let label = ("Error popup anchor").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(anchor)
                .on_press(move || open.set(true)),
                show(popup.shown(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("Error popup choice").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(first)
                        .on_press(move || fail.set(false)),
                    )
                    .on_dismiss(move |_| -> Result<(), UiError> {
                        if fail.get_untracked() {
                            Err(UiError::msg("popup dismissal failed"))
                        } else {
                            Ok(())
                        }
                    })
                    .part(popup.content())
                    .role(Role::Menu)
                    .name("Error popup")
                }),
                {
                    let label = ("Error popup destination").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .style(style().margin_top(80)),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Error popup anchor")
        .click();
    screen.press_key(Key::Escape);
    assert!(open.get_untracked());
    assert!(
        screen
            .get_by_role(Role::Button, "Error popup choice")
            .is_focused()
    );
    assert_eq!(screen.errors().len(), 1);
    assert_eq!(screen.errors()[0].message(), "popup dismissal failed");
    screen
        .get_by_role(Role::Button, "Error popup choice")
        .click();
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(
        screen
            .get_by_role(Role::Button, "Error popup destination")
            .is_focused()
    );
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn popup_survives_retained_tab_focus_handoff_and_closes_on_external_focus() {
    let (screen, (open, selected)) = mount(|| {
        let open = signal(false);
        let selected = signal("first");
        let anchor = node_ref();
        let first_field = node_ref();
        let popup = use_popup(open).anchor(anchor).initial_focus(first_field);
        let options = vec![
            ListItem::new("first", "First tab"),
            ListItem::new("second", "Second tab"),
        ]
        .into_value();
        let tabs = kivact::interaction::use_tabs(options, selected);
        (
            (open, selected),
            column((
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name("Retained tabs opener")
                    .node_ref(anchor)
                    .on_press(move || open.set(true))
                    .child(text("Open tabs")),
                show(popup.shown(), move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        column((
                            row(for_each(
                                options,
                                |item| *item.key(),
                                move |key, item| {
                                    node()
                                        .part(tabs.item(key))
                                        .child(text(move || item.get().label()))
                                        .style(style().size(100, 30))
                                },
                            ))
                            .part(tabs.list()),
                            node().part(tabs.panel("first")).child(
                                kivact::control::node()
                                    .name("First retained field")
                                    .part(
                                        kivact::interaction::use_text_edit(signal(String::new()))
                                            .control(),
                                    )
                                    .node_ref(first_field),
                            ),
                            node().part(tabs.panel("second")).child(
                                kivact::control::node().name("Second retained field").part(
                                    kivact::interaction::use_text_edit(signal(String::new()))
                                        .control(),
                                ),
                            ),
                        )),
                    )
                    .part(popup.content())
                    .role(Role::Dialog)
                    .name("Retained tabs popup")
                }),
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name("External destination")
                    .style(style().margin_top(150))
                    .child(text("Outside")),
            )),
        )
    });
    screen
        .get_by_role(Role::Button, "Retained tabs opener")
        .click();
    let first = screen.get_by_role(Role::TextBox, "First retained field");
    assert!(first.is_focused());
    screen.get_by_role(Role::Tab, "Second tab").click();
    assert!(
        open.get_untracked(),
        "a transient absent focus during the tab handoff must not close the popup"
    );
    assert_eq!(selected.get_untracked(), "second");
    assert!(screen.get_by_role(Role::Tab, "Second tab").is_focused());
    assert!(!first.is_visible());
    let destination = screen.get_by_role(Role::Button, "External destination");
    destination.focus();
    assert!(
        !open.get_untracked(),
        "a concrete outside focus destination closes the popup"
    );
    assert!(destination.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
