//! Role: List Parts retain semantics and exact input across caller targets.

use crate::support::mount_with;
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::interaction::{ListPolicy, ListSelection, use_list};
use kivact::prelude::*;
use kivact::primitive::part;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::ui::ListItem;
use kivact::view::{A11yAction, Handled};
use kivact::view::{MouseButton, PointerPhase};

fn mount<H: 'static, V: IntoView>(f: impl FnOnce() -> (H, V) + 'static) -> (Screen, H) {
    mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        f,
    )
}

#[test]
fn regions_and_nodes_share_one_roving_registry_and_reactive_labels() {
    let (screen, (label, selected, first, second, list)) = mount(|| {
        let label = signal("First".to_owned());
        let selected = signal(Some(1u64));
        let list = use_list(
            vec![ListItem::new(1, label), ListItem::new(2, "Second")],
            ListSelection::Single(selected.into()),
        );
        let first = list.item(1);
        let first_state = first.state();
        let second = list.item(2);
        let second_state = second.state();
        let view = column((
            canvas(|draw, _| draw.region(RegionId(1), Rect::new(0., 0., 100., 40.)))
                .region(RegionId(1), first)
                .style(style().size(100, 40)),
            node().part(second).style(style().size(100, 40)),
        ))
        .part(list.list());
        ((label, selected, first_state, second_state, list), view)
    });
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "First").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Second").is_focused());
    assert_eq!(selected.get_untracked(), Some(2));
    assert!(!first.get_untracked().active);
    assert!(second.get_untracked().active);
    label.set("Renamed".to_owned());
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Option, "Renamed").is_some());
    assert!(list.choose(1));
    screen.run_until_idle();
    assert!(first.get_untracked().input.focused);
    assert_eq!(selected.get_untracked(), Some(1));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn final_disabled_constraints_exclude_canvas_regions() {
    let (screen, (disabled, owner_disabled, selected, list)) = mount(|| {
        let disabled = signal(true);
        let owner_disabled = signal(false);
        let selected = signal(None::<u64>);
        let list = use_list(
            vec![ListItem::new(1, "First"), ListItem::new(2, "Second")],
            ListSelection::Single(selected.into()),
        );
        let view = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 90., 40.));
            draw.region(RegionId(2), Rect::new(100., 0., 90., 40.));
        })
        .region(RegionId(1), list.item(1).disabled(disabled))
        .region(RegionId(2), list.item(2))
        .disabled(owner_disabled)
        .style(style().size(200, 40));
        ((disabled, owner_disabled, selected, list), view)
    });
    assert!(!list.choose(1));
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "Second").is_focused());
    disabled.set(false);
    screen.run_until_idle();
    assert!(list.choose(1));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), Some(1));
    assert!(screen.get_by_role(Role::Option, "First").is_focused());
    owner_disabled.set(true);
    screen.run_until_idle();
    assert!(
        !list.choose(2),
        "canvas owner constraints also gate regions"
    );
    assert!(!list.focus(2));
    owner_disabled.set(false);
    screen.run_until_idle();
    assert!(list.choose(2));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), Some(2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn subsequent_drag_part_observes_selection_and_custom_consumption_stays_first() {
    let (screen, (selected, observed, consumed)) = mount(|| {
        let selected = signal(Vec::<u64>::new());
        let observed = signal(Vec::new());
        let consumed = signal(false);
        let list = use_list(
            vec![ListItem::new(1, "Row")],
            ListSelection::Multiple(selected.into()),
        );
        let drag = part(|input| input).on_pointer(move |event| {
            if event.phase == PointerPhase::Down && event.button == Some(MouseButton::Primary) {
                observed.set(selected.get_untracked());
                Handled::Yes
            } else {
                Handled::No
            }
        });
        let view = node()
            .part(list.item(1))
            .part(drag)
            .on_pointer(move |event| {
                if consumed.get_untracked() && event.phase == PointerPhase::Down {
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .style(style().size(120, 40));
        ((selected, observed, consumed), view)
    });
    screen.get_by_role(Role::Option, "Row").click();
    assert_eq!(observed.get_untracked(), [1]);
    selected.set(Vec::new());
    observed.set(Vec::new());
    consumed.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Option, "Row").click();
    assert!(selected.get_untracked().is_empty());
    assert!(observed.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_consuming_nested_control_keeps_its_focus_and_does_not_select_the_row() {
    let (screen, selected) = mount(|| {
        let selected = signal(Vec::<u64>::new());
        let list = use_list(
            vec![ListItem::new(1, "Row")],
            ListSelection::Multiple(selected.into()),
        );
        let inner = node()
            .role(Role::Button)
            .name("Inner")
            .focusable(true)
            .on_pointer(|_| Handled::Yes)
            .on_press(|| ())
            .style(style().size(80, 30));
        (
            selected,
            node()
                .part(list.item(1))
                .child(inner)
                .style(style().size(160, 60)),
        )
    });
    screen.get_by_role(Role::Button, "Inner").click();
    assert!(screen.get_by_role(Role::Button, "Inner").is_focused());
    assert!(selected.get_untracked().is_empty());
    screen.press_key(Key::Space);
    assert!(selected.get_untracked().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pending_virtual_focus_is_cancelled_when_the_user_focuses_outside() {
    let (screen, (shown, scrolls, list)) = mount(|| {
        let shown = signal(false);
        let scrolls = signal(Vec::new());
        let list = use_list(
            vec![ListItem::new(1, "First"), ListItem::new(2, "Later")],
            ListSelection::Single(signal(None).into()),
        )
        .scroll_with(move |key| scrolls.update(|keys| keys.push(key)));
        let outside = node()
            .role(Role::Button)
            .name("Outside")
            .focusable(true)
            .on_press(|| ())
            .style(style().size(100, 30));
        let view = column((
            node().part(list.item(1)).style(style().size(100, 30)),
            outside,
            show(shown, move || {
                node().part(list.item(2)).style(style().size(100, 30))
            }),
        ));
        ((shown, scrolls, list), view)
    });
    screen.get_by_role(Role::Option, "First").focus();
    assert!(list.focus(2));
    assert_eq!(scrolls.get_untracked(), [2]);
    screen.get_by_role(Role::Button, "Outside").focus();
    shown.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "Outside").is_focused());
    assert_eq!(scrolls.get_untracked(), [2]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn unsupported_policy_preserves_the_previous_complete_policy() {
    let (screen, selected) = mount(|| {
        let selected = signal(None::<u64>);
        let list = use_list(
            vec![ListItem::new(1, "First"), ListItem::new(2, "Second")],
            ListSelection::Single(selected.into()),
        )
        .policy(ListPolicy::tabs())
        .policy(ListPolicy {
            action: A11yAction::Expand,
            ..ListPolicy::default()
        });
        (
            selected,
            row((
                node().part(list.item(1)).style(style().size(100, 30)),
                node().part(list.item(2)).style(style().size(100, 30)),
            ))
            .part(list.list()),
        )
    });
    assert_eq!(
        screen.get_by_role(Role::Option, "First").a11y_actions(),
        [A11yAction::Activate, A11yAction::Focus]
    );
    screen.press_tab(false);
    screen.press_key(Key::ArrowRight);
    assert_eq!(
        selected.get_untracked(),
        Some(2),
        "the horizontal automatic Activate policy survives"
    );
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(matches!(
        errors[0].kind(),
        kivact::diagnostics::ErrorKind::Unsupported
    ));
}

#[test]
fn attached_constraints_and_ancestor_hidden_gate_mounts_but_ancestor_disabled_is_local() {
    let (screen, (disabled, hidden, selected, list)) = mount(|| {
        let disabled = signal(true);
        let hidden = signal(true);
        let selected = signal(None::<u64>);
        let list = use_list(
            vec![
                ListItem::new(1, "Attached"),
                ListItem::new(2, "Parented"),
                ListItem::new(3, "Available"),
            ],
            ListSelection::Single(selected.into()),
        )
        .scroll_with(|_| ());
        let attached = node()
            .part(list.item(1))
            .part(part(|input| input).disabled(disabled).hidden(hidden))
            .style(style().size(100, 30));
        let parented = column(node().part(list.item(2)).style(style().size(100, 30)))
            .part(part(|input| input).disabled(disabled).hidden(hidden));
        let available = node().part(list.item(3)).style(style().size(100, 30));
        (
            (disabled, hidden, selected, list),
            column((attached, parented, available)).part(list.list()),
        )
    });
    assert!(!list.choose(1));
    assert!(!list.focus(1));
    assert!(!list.choose(2));
    assert!(!list.focus(2));
    assert!(
        list.select(1, kivact::interaction::ListGesture::Replace),
        "logical selection ignores mount constraints"
    );
    assert_eq!(selected.get_untracked(), Some(1));
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "Available").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::Option, "Available").is_focused());
    disabled.set(false);
    screen.run_until_idle();
    assert!(
        !list.choose(1),
        "an attached Part hidden constraint still excludes"
    );
    assert!(
        !list.choose(2),
        "a parent Part hidden constraint still excludes"
    );
    disabled.set(true);
    hidden.set(false);
    screen.run_until_idle();
    assert!(
        !list.choose(1),
        "an attached Part disabled constraint still excludes"
    );
    assert!(list.choose(2), "ancestor Disabled is node-local");
    assert!(list.focus(2));
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Option, "Parented").is_focused());
    disabled.set(false);
    screen.run_until_idle();
    assert!(list.choose(1));
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Option, "Attached").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Parented").is_focused());
    assert_eq!(selected.get_untracked(), Some(2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn virtual_focus_survives_unbinding_the_source_before_binding_the_target() {
    let (screen, (list, shown, scrolls)) = mount(|| {
        let shown = signal(1u64);
        let scrolls = signal(Vec::new());
        let list = use_list(
            vec![ListItem::new(1, "First"), ListItem::new(2, "Later")],
            ListSelection::Single(signal(None).into()),
        )
        .scroll_with(move |key| {
            scrolls.update(|keys| keys.push(key));
            shown.set(key);
        });
        let view = column(for_each(
            move || vec![shown.get()],
            |key| *key,
            move |key, _| node().part(list.item(key)).style(style().size(100, 30)),
        ));
        ((list, shown, scrolls), view)
    });
    screen.get_by_role(Role::Option, "First").focus();
    assert!(list.focus(2));
    screen.run_until_idle();
    assert_eq!(shown.get_untracked(), 2);
    assert_eq!(scrolls.get_untracked(), [2]);
    assert!(screen.query_by_role(Role::Option, "First").is_none());
    assert!(screen.get_by_role(Role::Option, "Later").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn bound_source_losing_focus_cancels_a_virtual_request() {
    let (screen, (list, hidden, shown)) = mount(|| {
        let hidden = signal(false);
        let shown = signal(false);
        let list = use_list(
            vec![ListItem::new(1, "First"), ListItem::new(2, "Later")],
            ListSelection::Single(signal(None).into()),
        )
        .scroll_with(|_| ());
        let view = column((
            node()
                .part(list.item(1))
                .hidden(hidden)
                .style(style().size(100, 30)),
            show(shown, move || {
                node().part(list.item(2)).style(style().size(100, 30))
            }),
        ));
        ((list, hidden, shown), view)
    });
    screen.get_by_role(Role::Option, "First").focus();
    assert!(list.focus(2));
    // Hiding leaves the original reference bound but removes its native focus.
    hidden.set(true);
    screen.run_until_idle();
    assert!(screen.focused().is_none());
    hidden.set(false);
    shown.set(true);
    screen.run_until_idle();
    assert!(!screen.get_by_role(Role::Option, "Later").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_foreign_list_part_is_rejected_before_its_sources_or_refs_are_attached() {
    let (first, (foreign, foreign_state, foreign_ref, hooks, actions, selected)) = mount(|| {
        let selected = signal(Some(1u64));
        let hooks = signal(0u32);
        let actions = signal(0u32);
        let list = use_list(
            vec![ListItem::new(1, "Source")],
            ListSelection::Single(selected.into()),
        )
        .policy(ListPolicy::radio());
        let foreign = list
            .item(1)
            .role(Role::RadioButton)
            .name("Foreign Part")
            .focusable(true)
            .on_a11y(A11yAction::Increment, move || {
                actions.update(|count| *count += 1)
            })
            .on_bind(move |_| hooks.update(|count| *count += 1));
        let foreign_state = foreign.state();
        let foreign_ref = foreign.node_ref();
        let view = node()
            .part(list.item(1).role(Role::Tab).name("Own"))
            .child(text("Own"))
            .style(style().size(120, 28));
        (
            (
                foreign,
                foreign_state,
                foreign_ref,
                hooks,
                actions,
                selected,
            ),
            view,
        )
    });
    let (second, local_actions) = mount(move || {
        let local_actions = signal(0u32);
        let view = node()
            .role(Role::Tab)
            .name("Caller")
            .selected(false)
            .focusable(false)
            .on_press(move || local_actions.update(|count| *count += 1))
            .child(text("Caller"))
            .style(style().size(120, 28))
            .part(foreign);
        (local_actions, view)
    });
    // ChoiceMember used a disabled sentinel; rejected Parts now leave callers intact.
    let caller = second.get_by_role(Role::Tab, "Caller");
    assert!(!caller.is_disabled());
    assert!(!caller.is_selected());
    assert_eq!(caller.a11y_actions(), [A11yAction::Activate]);
    assert!(
        second
            .query_by_role(Role::RadioButton, "Foreign Part")
            .is_none()
    );
    caller.a11y(A11yAction::Activate);
    assert_eq!(local_actions.get_untracked(), 1);
    assert_eq!(hooks.get_untracked(), 0);
    assert_eq!(actions.get_untracked(), 0);
    assert!(foreign_ref.rect().get_untracked().is_none());
    assert!(!foreign_state.get_untracked().input.focused);
    let errors = second.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(matches!(
        errors[0].kind(),
        kivact::diagnostics::ErrorKind::WrongApp
    ));
    first.get_by_role(Role::Tab, "Own").click();
    assert!(first.get_by_role(Role::Tab, "Own").is_focused());
    assert_eq!(selected.get_untracked(), Some(1));
    assert!(first.errors().is_empty(), "{:?}", first.errors());
}
