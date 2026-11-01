//! Role: Public ancestor visibility and roving-focus contracts.

use super::*;

#[test]
fn ancestor_hidden_siblings_are_skipped_by_roving_navigation() {
    let (screen, (selected, group)) = mount_real(|| {
        let selected = signal(Some("a"));
        let group = use_list(
            vec![
                ListItem::new("a", ""),
                ListItem::new("b", ""),
                ListItem::new("c", ""),
            ],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio());
        (
            (selected, group),
            node().part(group.list().role(Role::TabList)).child((
                node()
                    .part(group.item("a").role(Role::Tab).name("a"))
                    .child(text("a")),
                node().hidden(true).child(
                    node()
                        .part(group.item("b").role(Role::Tab).name("b"))
                        .child(text("b")),
                ),
                node()
                    .part(group.item("c").role(Role::Tab).name("c"))
                    .child(text("c")),
            )),
        )
    });
    screen.get_by_role(Role::Tab, "a").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), Some("c"));
    assert_eq!(group.active().get_untracked(), Some("c"));
    assert!(screen.get_by_role(Role::Tab, "c").is_focused());
    assert!(!group.focus("b"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn ancestor_visibility_changes_restore_roving_candidates_without_stealing_focus() {
    let (screen, (hide_b, hide_all, selected, group)) = mount_real(|| {
        let hide_b = signal(false);
        let hide_all = signal(false);
        let selected = signal(Some("b"));
        let group = use_list(
            vec![
                ListItem::new("a", ""),
                ListItem::new("b", ""),
                ListItem::new("c", ""),
            ],
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio());
        (
            (hide_b, hide_all, selected, group),
            column((
                node()
                    .part(group.list().role(Role::TabList))
                    .child((
                        node()
                            .part(group.item("a").role(Role::Tab).name("a"))
                            .child(text("a")),
                        node().hidden(hide_b).child(
                            node()
                                .part(group.item("b").role(Role::Tab).name("b"))
                                .child(text("b")),
                        ),
                        node()
                            .part(group.item("c").role(Role::Tab).name("c"))
                            .child(text("c")),
                    ))
                    .hidden(hide_all),
                action("Outside"),
            )),
        )
    });
    screen.get_by_role(Role::Tab, "b").focus();
    hide_b.set(true);
    screen.run_until_idle();
    assert_eq!(group.active().get_untracked(), Some("a"));
    assert_eq!(selected.get_untracked(), Some("b"));
    assert!(screen.get_by_role(Role::Tab, "a").is_focused());
    hide_b.set(false);
    screen.run_until_idle();
    assert_eq!(group.active().get_untracked(), Some("a"));
    assert!(screen.get_by_role(Role::Tab, "a").is_focused());
    assert!(group.focus("b"));
    screen.run_until_idle();
    hide_all.set(true);
    screen.run_until_idle();
    assert_eq!(group.active().get_untracked(), None);
    assert_eq!(selected.get_untracked(), Some("b"));
    let outside = screen.get_by_role(Role::Button, "Outside");
    outside.click();
    hide_all.set(false);
    screen.run_until_idle();
    assert_eq!(group.active().get_untracked(), Some("b"));
    assert!(outside.is_focused());
    screen.press_tab(true);
    assert!(screen.get_by_role(Role::Tab, "b").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn terminal_constraints_preserve_disabled_hidden_and_one_roving_tab_stop() {
    fn decorate<V: NodeBuilder>(view: V, disabled: Signal<bool>) -> V {
        view.disabled(disabled)
    }
    let (screen, (disabled, hidden, selected, group)) = mount_real(|| {
        let disabled = signal(false);
        let hidden = signal(false);
        let selected = signal(Some(0u64));
        let group = use_list(
            (0..6)
                .map(|key| ListItem::new(key, "").disabled(key == 0))
                .collect::<Vec<_>>(),
            ListSelection::Single((selected).into()),
        )
        .policy(ListPolicy::radio())
        .policy(ListPolicy::tabs());
        let logical = node()
            .part(group.item(0).role(Role::Tab).name("logical disabled"))
            .disabled(false)
            .child(text("logical disabled"))
            .style(style().size(160, 28));
        let member = group.item(1);
        let member_state = member.state();
        let before_apply = (node()
            .role(Role::Tab)
            .name("before apply")
            .disabled(true)
            .selected(memo(move || member_state.get().selected))
            .child(text("before apply"))
            .style(style().size(160, 28)))
        .part(member.role(Role::Tab).name("before apply"));
        let generic = decorate(
            node()
                .part(group.item(2).role(Role::Tab).name("generic disabled"))
                .child(text("generic disabled"))
                .style(style().size(160, 28)),
            disabled,
        );
        let hidden_item = node()
            .part(group.item(3).role(Role::Tab).name("hidden"))
            .hidden(hidden)
            .child(text("hidden"))
            .style(style().size(160, 28));
        let mut permission = node()
            .part(group.item(4).role(Role::Tab).name("permission"))
            .child(text("permission"))
            .style(style().size(160, 28));
        let root = std::mem::replace(permission.node_mut(), node());
        *permission.node_mut() = root.focusable(false);
        let explicit_true = node()
            .part(group.item(5).role(Role::Tab).name("last"))
            .focusable(true)
            .child(text("last"))
            .style(style().size(160, 28));
        (
            (disabled, hidden, selected, group),
            column((
                action("before"),
                node().part(group.list().role(Role::TabList)).child((
                    logical,
                    before_apply,
                    generic,
                    hidden_item,
                    permission,
                    explicit_true,
                )),
                action("after"),
            )),
        )
    });
    assert!(
        screen
            .get_by_role(Role::Tab, "logical disabled")
            .is_disabled()
    );
    assert!(screen.get_by_role(Role::Tab, "before apply").is_disabled());
    assert!(!group.focus(0));
    assert!(!group.choose(1));
    assert!(!group.focus(4), "explicit false excludes focus permission");
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::Tab, "generic disabled")
            .is_focused()
    );
    screen.press_tab(false);
    assert!(
        screen.get_by_role(Role::Button, "after").is_focused(),
        "caller true does not create a second tab stop"
    );
    screen.press_tab(true);
    disabled.set(true);
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::Tab, "generic disabled")
            .is_disabled()
    );
    assert!(
        screen.get_by_role(Role::Tab, "hidden").is_focused(),
        "reactive disabled fallback"
    );
    hidden.set(true);
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::Tab, "last").is_focused(),
        "hidden key and focus-forbidden key are skipped"
    );
    assert!(!group.focus(3));
    assert_eq!(
        selected.get_untracked(),
        Some(0),
        "constraints never clear app selection"
    );
    disabled.set(false);
    hidden.set(false);
    screen.run_until_idle();
    assert!(
        screen.get_by_role(Role::Tab, "last").is_focused(),
        "re-enabled views do not steal focus"
    );
    screen.press_key(Key::Home);
    assert!(
        screen
            .get_by_role(Role::Tab, "generic disabled")
            .is_focused()
    );
    assert_eq!(selected.get_untracked(), Some(2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
