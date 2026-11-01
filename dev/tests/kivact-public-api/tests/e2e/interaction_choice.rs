//! Role: Tests for choice interaction contracts.
//!
//! Public keyed-choice contracts on real windows: logical identity, active
//! versus selected state, owner cleanup, virtualization and themed semantics.

use crate::support::mount_with;
use kivact::primitive::NodeExt;
use kivact::style::current_theme;

mod visibility;
use kivact::binding;
use kivact::interaction::{
    ListBoundary, ListGesture, ListPolicy, ListSelection, PopupCloseReason, use_list, use_popup,
};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::ui::{ListActivation, ListItem};
use kivact::view::provide_theme;
use kivact::view::{A11yAction, Handled, Placement, RowHeight, overlay};

fn mount_real<H: 'static, V: IntoView>(f: impl FnOnce() -> (H, V) + 'static) -> (Screen, H) {
    mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        f,
    )
}

fn action(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(|| ())
        .child(text(name))
        .style(style().size(120, 28))
}

fn item(key: &str) -> ListItem<String> {
    ListItem::new(key.to_string(), "")
}

#[test]
fn manual_choice_keeps_keys_across_reorder_delete_and_readd() {
    let (screen, (items, selected, group)) = mount_real(|| {
        let items = signal(vec![
            item("alpha"),
            item("beta").disabled(true),
            item("gamma"),
            item("delta"),
        ]);
        let selected = signal(Some("alpha".to_string()));
        let group = use_list(items, ListSelection::Single((selected).into()))
            .policy(ListPolicy::radio())
            .policy(ListPolicy {
                axis: Some(Orientation::Horizontal),
                boundary: ListBoundary::Clamp,
                activation: ListActivation::Manual,
                ..ListPolicy::radio()
            });
        let choices = for_each(
            items,
            |item| item.key().clone(),
            move |key, _| {
                node()
                    .part(group.item(key.clone()).role(Role::Tab).name(key.clone()))
                    .child(text(key))
                    .style(style().size(120, 28))
            },
        );
        (
            (items, selected, group),
            column((
                action("before"),
                node().part(group.list().role(Role::TabList)).child(choices),
                action("after"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Tab, "alpha").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Tab, "gamma").is_focused());
    assert_eq!(group.active().get_untracked().as_deref(), Some("gamma"));
    assert_eq!(selected.get_untracked().as_deref(), Some("alpha"));
    screen.press_key(Key::ArrowDown);
    assert!(
        screen.get_by_role(Role::Tab, "gamma").is_focused(),
        "axis policy"
    );
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked().as_deref(), Some("gamma"));
    screen.press_key(Key::Home);
    assert!(screen.get_by_role(Role::Tab, "alpha").is_focused());
    assert_eq!(selected.get_untracked().as_deref(), Some("gamma"));
    screen.press_key(Key::End);
    screen.press_key(Key::ArrowRight);
    assert!(
        screen.get_by_role(Role::Tab, "delta").is_focused(),
        "clamped end"
    );
    screen
        .get_by_role(Role::Tab, "gamma")
        .a11y(A11yAction::Activate);
    assert!(screen.get_by_role(Role::Tab, "gamma").is_focused());

    let created = screen.stats().creates;
    items.set(vec![
        item("delta"),
        item("gamma"),
        item("alpha"),
        item("beta").disabled(true),
    ]);
    screen.run_until_idle();
    assert_eq!(
        screen.stats().creates,
        created,
        "reordering keeps member nodes"
    );
    screen.press_key(Key::ArrowRight);
    assert!(
        screen.get_by_role(Role::Tab, "alpha").is_focused(),
        "logical order after reorder"
    );
    selected.set(Some("delta".to_string()));
    screen.run_until_idle();
    assert_eq!(
        group.active().get_untracked().as_deref(),
        Some("alpha"),
        "selection and active are independent"
    );

    items.set(vec![item("delta"), item("gamma")]);
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::Tab, "delta").is_focused(),
        "removed focused key falls back to selected"
    );
    items.set(vec![item("gamma")]);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Tab, "gamma").is_focused());
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("delta"),
        "deletion does not rewrite caller selection"
    );
    items.set(vec![item("gamma"), item("delta")]);
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::Tab, "delta").is_selected(),
        "same key is selected again"
    );
    assert!(
        screen.get_by_role(Role::Tab, "gamma").is_focused(),
        "re-add does not steal focus"
    );
    items.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(group.active().get_untracked(), None);
    assert_eq!(selected.get_untracked().as_deref(), Some("delta"));
    assert!(!group.choose("delta".to_string()));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn automatic_radio_wraps_skips_disabled_and_uses_one_invocation_writer() {
    let (screen, (selected, calls, items, group)) = mount_real(|| {
        let items = signal(vec![
            ListItem::new(10u64, ""),
            ListItem::new(20, "").disabled(true),
            ListItem::new(30, ""),
        ]);
        let selected = signal(None::<u64>);
        let calls = signal(0);
        let binding = binding(selected.into_value(), move |key| {
            calls.update(|calls| *calls += 1);
            selected.set(key);
        });
        let group = use_list(items, ListSelection::Single(binding))
            .policy(ListPolicy::radio())
            .policy(ListPolicy::radio());
        let choices = for_each(
            items,
            |item| *item.key(),
            move |key, _| {
                {
                    let member = group
                        .item(key)
                        .role(Role::RadioButton)
                        .name(format!("radio {key}"));
                    let state = member.state();
                    node().part(member.checked(move || state.get().selected))
                }
                .child(text(format!("radio {key}")))
                .style(style().size(120, 28))
            },
        );
        (
            (selected, calls, items, group),
            column((
                node()
                    .part(group.list().role(Role::RadioGroup))
                    .child(choices),
                action("after"),
            )),
        )
    });
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::RadioButton, "radio 10")
            .is_focused()
    );
    assert_eq!(
        selected.get_untracked(),
        None,
        "fallback focus never silently selects"
    );
    screen.press_key(Key::ArrowLeft);
    assert_eq!(selected.get_untracked(), Some(30));
    assert_eq!(calls.get_untracked(), 1);
    assert!(
        screen
            .get_by_role(Role::RadioButton, "radio 30")
            .is_focused()
    );
    screen.press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), Some(10));
    screen.press_key(Key::End);
    assert_eq!(selected.get_untracked(), Some(30));
    let before = calls.get_untracked();
    screen.get_by_role(Role::RadioButton, "radio 10").click();
    assert_eq!(
        calls.get_untracked(),
        before + 1,
        "pointer invocation writes once"
    );
    assert_eq!(
        screen
            .get_by_role(Role::RadioButton, "radio 10")
            .is_checked(),
        Some(true)
    );
    screen
        .get_by_role(Role::RadioButton, "radio 30")
        .a11y(A11yAction::Activate);
    assert_eq!(calls.get_untracked(), before + 2, "AT uses the same writer");
    screen.press_key(Key::Home);
    assert_eq!(selected.get_untracked(), Some(10));
    items.set(vec![
        ListItem::new(10, "").disabled(true),
        ListItem::new(30, ""),
    ]);
    screen.run_until_idle();
    assert_eq!(
        selected.get_untracked(),
        Some(10),
        "disable never overwrites selection"
    );
    assert!(
        screen
            .get_by_role(Role::RadioButton, "radio 30")
            .is_focused(),
        "disabled focus falls back"
    );
    items.set(vec![
        ListItem::new(10, "").disabled(true),
        ListItem::new(30, "").disabled(true),
    ]);
    screen.run_until_idle();
    assert_eq!(group.active().get_untracked(), None);
    assert!(!group.focus(30));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn choice_defaults_compose_with_custom_keys_and_tab_leaves_the_group() {
    let (screen, (selected, custom, stop)) = mount_real(|| {
        let selected = signal(Some(1u32));
        let custom = signal(0);
        let stop = signal(true);
        let group = use_list(
            vec![ListItem::new(1, ""), ListItem::new(2, "")],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::tabs());
        let first = node()
            .part(group.item(1).role(Role::Tab).name("first"))
            .child(text("first"))
            .style(style().size(120, 28))
            .on_key(move |event| {
                if event.key == Key::ArrowRight {
                    custom.update(|calls| *calls += 1);
                    if stop.get_untracked() {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                } else {
                    Handled::No
                }
            });
        (
            (selected, custom, stop),
            column((
                action("before"),
                node().part(group.list().role(Role::TabList)).child((
                    first,
                    node()
                        .part(group.item(2).role(Role::Tab).name("second"))
                        .child(text("second"))
                        .style(style().size(120, 28)),
                )),
                action("after"),
            )),
        )
    });
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    screen.press_key(Key::ArrowRight);
    assert_eq!(custom.get_untracked(), 1);
    assert_eq!(selected.get_untracked(), Some(1));
    stop.set(false);
    screen.press_key(Key::ArrowRight);
    assert_eq!(custom.get_untracked(), 2);
    assert_eq!(selected.get_untracked(), Some(2));
    assert!(screen.get_by_role(Role::Tab, "second").is_focused());
    screen.press_tab(false);
    assert!(
        screen.get_by_role(Role::Button, "after").is_focused(),
        "one group tab stop"
    );
    screen.press_tab(true);
    assert!(screen.get_by_role(Role::Tab, "second").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn member_registration_starts_at_mount_and_ends_with_the_node_owner() {
    let (screen, (open, selected, group)) = mount_real(|| {
        let open = signal(true);
        let selected = signal(Some("b".to_string()));
        let group = use_list(
            vec![item("unused"), item("a"), item("b")],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::tabs());
        let _never_mounted = node().part(
            group
                .item("unused".to_string())
                .role(Role::Tab)
                .name("unused"),
        );
        let branch = show(open, move || {
            node().part(group.list().role(Role::TabList)).child((
                node()
                    .part(group.item("a".to_string()).role(Role::Tab).name("a"))
                    .child(text("a"))
                    .style(style().size(120, 28)),
                node()
                    .part(group.item("b".to_string()).role(Role::Tab).name("b"))
                    .child(text("b"))
                    .style(style().size(120, 28)),
            ))
        });
        (
            (open, selected, group),
            column((action("before"), branch, action("after"))),
        )
    });
    assert!(
        !group.focus("unused".to_string()),
        "constructing a member does not mount it"
    );
    assert_eq!(group.active().get_untracked().as_deref(), Some("b"));
    let owners = screen.stats().live_owners;
    let objects = screen.stats().live_objects;
    for _ in 0..8 {
        open.set(false);
        screen.run_until_idle();
        assert_eq!(group.active().get_untracked(), None);
        assert!(!group.choose("b".to_string()));
        assert_eq!(selected.get_untracked().as_deref(), Some("b"));
        open.set(true);
        screen.run_until_idle();
        screen.get_by_role(Role::Button, "before").click();
        screen.press_tab(false);
        assert!(screen.get_by_role(Role::Tab, "b").is_focused());
        assert_eq!(screen.stats().live_owners, owners);
        assert_eq!(screen.stats().live_objects, objects);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn virtual_offscreen_members_unregister_without_losing_selected_keys() {
    let (screen, (selected, group, viewport)) = mount_real(|| {
        let rows: Vec<u64> = (0..100).collect();
        let selected = signal(Some(0u64));
        let group = use_list(
            rows.iter()
                .copied()
                .map(|key| ListItem::new(key, ""))
                .collect::<Vec<_>>(),
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::menu());
        let viewport = node_ref();
        let list = virtual_list(
            rows,
            |key| *key,
            move |key, _| {
                node()
                    .part(
                        group
                            .item(key)
                            .role(Role::Option)
                            .name(format!("row {key}")),
                    )
                    .child(text(format!("row {key}")))
                    .style(style().height(24))
            },
        )
        .row_height(RowHeight::Fixed(24.0))
        .overscan(0.0)
        .keep_focused(false)
        .node_ref(viewport)
        .test_id("choices")
        .style(style().size(240, 120));
        (
            (selected, group, viewport),
            column((
                node().part(group.list().role(Role::ListBox)).child(list),
                action("after"),
            )),
        )
    });
    screen.get_by_role(Role::Option, "row 0").click();
    let owners = screen.stats().live_owners;
    for _ in 0..4 {
        viewport.scroll_to(
            kivact::draw::Point::new(0.0, 600.0),
            kivact::view::ScrollBehavior::Instant,
        );
        screen.run_until_idle();
        assert!(screen.query_by_role(Role::Option, "row 0").is_none());
        assert_eq!(selected.get_untracked(), Some(0));
        assert!(!group.focus(0), "old NodeRef is not a focus target");
        let active = group.active().get_untracked().expect("visible fallback");
        assert!((25..=30).contains(&active));
        viewport.scroll_to(
            kivact::draw::Point::new(0.0, 0.0),
            kivact::view::ScrollBehavior::Instant,
        );
        screen.run_until_idle();
        assert!(screen.get_by_role(Role::Option, "row 0").is_selected());
        assert_eq!(selected.get_untracked(), Some(0));
        assert_eq!(
            screen.stats().live_owners,
            owners,
            "offscreen owners are released"
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn duplicate_keys_report_and_late_policy_does_not_change_members() {
    let (screen, selected) = mount_real(|| {
        let selected = signal(Some(7u64));
        let group = use_list(
            vec![
                ListItem::new(7, ""),
                ListItem::new(7, "").disabled(true),
                ListItem::new(8, ""),
            ],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::tabs());
        let first = node()
            .part(group.item(7).role(Role::Tab).name("first"))
            .child(text("first"))
            .style(style().size(120, 28));
        let duplicate = node()
            .part(group.item(7).role(Role::Tab).name("duplicate"))
            .child(text("duplicate"))
            .style(style().size(120, 28));
        let group = group.policy(ListPolicy::menu());
        (
            selected,
            node().part(group.list().role(Role::TabList)).child((
                first,
                duplicate,
                node()
                    .part(group.item(8).role(Role::Tab).name("last"))
                    .child(text("last"))
                    .style(style().size(120, 28)),
            )),
        )
    });
    assert!(screen.get_by_role(Role::Tab, "duplicate").is_disabled());
    screen.press_tab(false);
    screen.press_key(Key::ArrowDown);
    assert_eq!(
        selected.get_untracked(),
        Some(7),
        "late vertical policy is ignored"
    );
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), Some(8));
    let errors = screen.errors();
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(
        errors
            .iter()
            .any(|error| error.message().contains("duplicate logical"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message().contains("already mounted"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message().contains("before creating parts"))
    );
}

#[test]
fn mounting_duplicate_list_parts_keeps_the_first_target_intact() {
    let (screen, (selected, group)) = mount_real(|| {
        let selected = signal(Some("first".to_string()));
        let group = use_list(
            vec![item("first"), item("last")],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::tabs());
        let member = group.item("first".to_string());
        let member_state = member.state();
        let first = (node()
            .role(Role::Tab)
            .name("first")
            .selected(memo(move || member_state.get().selected))
            .child(text("first"))
            .style(style().size(120, 28)))
        .part(member.role(Role::Tab).name("first"));
        let repeated = (node()
            .role(Role::Tab)
            .name("repeated")
            .child(text("repeated"))
            .style(style().size(120, 28)))
        .part(
            group
                .item("first".to_string())
                .role(Role::Tab)
                .name("repeated"),
        );
        (
            (selected, group),
            node().part(group.list().role(Role::TabList)).child((
                first,
                repeated,
                node()
                    .part(group.item("last".to_string()).role(Role::Tab).name("last"))
                    .child(text("last"))
                    .style(style().size(120, 28)),
            )),
        )
    });
    assert!(screen.get_by_role(Role::Tab, "repeated").is_disabled());
    assert!(!screen.get_by_role(Role::Tab, "first").is_disabled());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Tab, "first").is_focused());
    assert!(screen.get_by_role(Role::Tab, "first").is_selected());
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked().as_deref(), Some("last"));
    assert!(screen.get_by_role(Role::Tab, "last").is_focused());
    assert!(group.choose("first".to_string()));
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Tab, "first").is_focused());
    assert!(screen.get_by_role(Role::Tab, "first").is_selected());
    assert!(!screen.get_by_role(Role::Tab, "first").is_disabled());
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message().contains("already mounted"));
}

#[test]
fn logical_selection_does_not_require_mounted_focus_targets() {
    let (screen, (selected, open, group)) = mount_real(|| {
        let selected = signal(Some("a".to_string()));
        let open = signal(false);
        let items = vec![item("a"), item("b").disabled(true), item("c")];
        let group = use_list(items.clone(), ListSelection::Single((selected).into()))
            .policy(ListPolicy::radio())
            .policy(ListPolicy {
                boundary: ListBoundary::Clamp,
                ..ListPolicy::tabs()
            });
        let trigger = node()
            .role(Role::ComboBox)
            .name("closed choice")
            .focusable(true)
            .on_press(move || open.set(!open.get_untracked()))
            .child(text("closed choice"))
            .style(style().size(180, 28));
        let choices = show(open, move || {
            node()
                .part(group.list().role(Role::ListBox))
                .child(for_each(
                    items.clone(),
                    |item| item.key().clone(),
                    move |key, _| {
                        node()
                            .part(group.item(key.clone()).role(Role::Option).name(key.clone()))
                            .child(text(key))
                            .style(style().size(120, 28))
                    },
                ))
        });
        ((selected, open, group), column((trigger, choices)))
    });
    screen.press_tab(false);
    assert!(group.select("c".to_string(), ListGesture::Replace));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked().as_deref(), Some("c"));
    assert!(!open.get_untracked());
    assert_eq!(group.active().get_untracked(), None);
    assert!(
        screen
            .get_by_role(Role::ComboBox, "closed choice")
            .is_focused()
    );
    assert!(
        !group.select("b".to_string(), ListGesture::Replace),
        "disabled logical key is rejected"
    );
    assert!(group.select("a".to_string(), ListGesture::Replace));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked().as_deref(), Some("a"));
    assert_eq!(group.active().get_untracked(), None);
    assert!(
        !group.choose("c".to_string()),
        "mounted choose keeps its distinct contract"
    );
    screen.get_by_role(Role::ComboBox, "closed choice").click();
    assert_eq!(group.active().get_untracked().as_deref(), Some("a"));
    screen
        .get_by_role(Role::Option, "c")
        .a11y(A11yAction::Activate);
    assert_eq!(selected.get_untracked().as_deref(), Some("c"));
    assert!(screen.get_by_role(Role::Option, "c").is_focused());
    open.set(false);
    screen.run_until_idle();
    assert_eq!(selected.get_untracked().as_deref(), Some("c"));
    assert_eq!(group.active().get_untracked(), None);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_controlled_binding_can_reject_selection_without_rewriting_active_or_meaning() {
    let (screen, (selected, requests, group)) = mount_real(|| {
        let selected = signal(Some("a".to_string()));
        let requests = signal(Vec::<Option<String>>::new());
        let binding = binding(selected.into_value(), move |key| {
            // Validation refuses the request, so the accepted source remains a.
            requests.update(|requests| requests.push(key));
        });
        let group = use_list(vec![item("a"), item("b")], ListSelection::Single(binding))
            .policy(ListPolicy::radio())
            .policy(ListPolicy::tabs());
        let choices = node().part(group.list().role(Role::RadioGroup)).child((
            {
                let member = group
                    .item("a".to_string())
                    .role(Role::RadioButton)
                    .name("a");
                let state = member.state();
                node().part(member.checked(move || state.get().selected))
            }
            .child(text("a"))
            .style(style().size(120, 28)),
            {
                let member = group
                    .item("b".to_string())
                    .role(Role::RadioButton)
                    .name("b");
                let state = member.state();
                node().part(member.checked(move || state.get().selected))
            }
            .child(text("b"))
            .style(style().size(120, 28)),
        ));
        let trigger = node()
            .role(Role::ComboBox)
            .name("logical trigger")
            .focusable(true)
            .on_key_default(move |event| {
                if event.key == Key::ArrowRight {
                    group.select("b".to_string(), ListGesture::Replace);
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .on_press(|| ())
            .child(text("logical trigger"))
            .style(style().size(160, 28));
        ((selected, requests, group), column((choices, trigger)))
    });
    assert!(
        group.choose("b".to_string()),
        "true means offered, not accepted"
    );
    screen.run_until_idle();
    assert_eq!(selected.get_untracked().as_deref(), Some("a"));
    assert_eq!(group.active().get_untracked().as_deref(), Some("b"));
    assert!(screen.get_by_role(Role::RadioButton, "b").is_focused());
    assert_eq!(
        screen.get_by_role(Role::RadioButton, "a").is_checked(),
        Some(true)
    );
    assert_eq!(
        screen.get_by_role(Role::RadioButton, "b").is_checked(),
        Some(false)
    );
    assert_eq!(requests.get_untracked().len(), 1);
    screen.press_key(Key::ArrowLeft);
    assert!(screen.get_by_role(Role::RadioButton, "a").is_focused());
    assert_eq!(
        requests.get_untracked().len(),
        2,
        "automatic arrow offers one writer call"
    );
    screen
        .get_by_role(Role::ComboBox, "logical trigger")
        .click();
    screen.press_key(Key::ArrowRight);
    assert_eq!(group.active().get_untracked().as_deref(), Some("a"));
    assert_eq!(selected.get_untracked().as_deref(), Some("a"));
    assert!(
        screen
            .get_by_role(Role::ComboBox, "logical trigger")
            .is_focused()
    );
    assert!(group.select("b".to_string(), ListGesture::Replace));
    screen.run_until_idle();
    assert_eq!(
        requests.get_untracked(),
        vec![
            Some("b".to_string()),
            Some("a".to_string()),
            Some("b".to_string()),
            Some("b".to_string())
        ]
    );
    assert_eq!(selected.get_untracked().as_deref(), Some("a"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn two_themes_share_member_semantics_selection_and_focus_without_remounting() {
    let (screen, theme) = mount_real(|| {
        let theme = signal(Theme::light());
        let selected = signal(Some("red".to_string()));
        let group = use_list(
            vec![item("red"), item("blue")],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::tabs());
        let view = provide_theme(theme, move || {
            let tabs = ["red", "blue"]
                .map(|key| {
                    node()
                        .part(group.item(key.to_string()).role(Role::Tab).name(key))
                        .child(text(key))
                        .style(move || {
                            let theme = &current_theme();
                            style()
                                .size(140, 32)
                                .background(theme.colors().surface)
                                .color(theme.colors().text)
                                .selected(style().border(2, theme.colors().accent))
                        })
                })
                .into_iter()
                .collect::<Vec<_>>();
            node().part(group.list().role(Role::TabList)).child(tabs)
        });
        (theme, view)
    });
    screen.get_by_role(Role::Tab, "red").click();
    screen.press_key(Key::ArrowRight);
    let before = screen.snapshot();
    let created = screen.stats().creates;
    let light = screen
        .get_by_role(Role::Tab, "red")
        .screenshot()
        .pixel(10, 20);
    theme.set(Theme::dark());
    screen.run_until_idle();
    let dark = screen
        .get_by_role(Role::Tab, "red")
        .screenshot()
        .pixel(10, 20);
    assert_ne!(light, dark, "both themes were actually painted");
    assert_eq!(
        screen.snapshot(),
        before,
        "style keeps the same semantic tree and focus"
    );
    assert_eq!(screen.stats().creates, created);
    assert!(screen.get_by_role(Role::Tab, "blue").is_selected());
    assert!(screen.get_by_role(Role::Tab, "blue").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(screen.get_by_role(Role::Tab, "red").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn replacing_a_menu_parts_invoke_replaces_all_native_activation_routes() {
    let (screen, (open, command_calls, choice_writes, selected)) = mount_real(|| {
        let open = signal(false);
        let command_calls = signal(0u32);
        let choice_writes = signal(0u32);
        let selected = signal(None::<u32>);
        let anchor = node_ref();
        let popup = use_popup(open).anchor(anchor);
        let menu = show(popup.shown(), move || {
            let group = use_list(
                vec![ListItem::new(1u32, "")],
                ListSelection::Single(binding(selected.into_value(), move |value| {
                    choice_writes.update(|count| *count += 1);
                    selected.set(value);
                })),
            )
            .policy(ListPolicy::radio())
            .policy(ListPolicy::menu());
            kivact::on_mount(move || {
                group.focus(1);
            });
            overlay(
                anchor,
                Placement::Below,
                node().part(group.list().role(Role::Menu)).child(
                    node()
                        .part(
                            group
                                .item(1)
                                .role(Role::MenuItem)
                                .name("Run command")
                                .on_press(move || {
                                    command_calls.update(|count| *count += 1);
                                    popup.close(PopupCloseReason::Selection);
                                }),
                        )
                        .style(style().size(140, 32))
                        .child(text("Run command")),
                ),
            )
            .part(popup.content())
        });
        (
            (open, command_calls, choice_writes, selected),
            column((
                action("Commands")
                    .node_ref(anchor)
                    .on_press(move || open.set(true)),
                menu,
            )),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Commands");
    for expected in 1..=4 {
        trigger.click();
        let item = screen.get_by_role(Role::MenuItem, "Run command");
        assert!(item.is_focused());
        match expected {
            1 => item.a11y(A11yAction::Activate),
            2 => screen.press_key(Key::Enter),
            3 => screen.press_key(Key::Space),
            _ => item.click(),
        }
        assert_eq!(command_calls.get_untracked(), expected);
        assert_eq!(choice_writes.get_untracked(), 0);
        assert_eq!(selected.get_untracked(), None);
        assert!(!open.get_untracked());
        assert!(trigger.is_focused());
        assert!(
            screen
                .query_by_role(Role::MenuItem, "Run command")
                .is_none()
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
