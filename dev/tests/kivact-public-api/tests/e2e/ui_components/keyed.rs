//! Role: Preserve standard select wrapper identity and handler-ordering regressions.
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::ui::ListItem;
use kivact::view::A11yAction;

fn choices() -> Vec<ListItem<String>> {
    vec![
        ListItem::new("department:operations".into(), "Operations"),
        ListItem::new("department:telemetry".into(), "Telemetry"),
        ListItem::new("department:locked".into(), "Locked").disabled(true),
    ]
}

fn dropdown_options(items: &[(&str, &str)]) -> Vec<ListItem<String>> {
    items
        .iter()
        .map(|(key, label)| ListItem::new((*key).to_owned(), *label))
        .collect()
}

#[test]
fn select_keeps_identity_through_popup_navigation_filtering_and_writer_rejection() {
    let (screen, (options, selected)) = kivact::test::render_with_handle(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let options = signal(choices());
            let selected = signal(Some("department:telemetry".to_owned()));
            (
                (options, selected),
                kivact::ui::select("Select", options, selected)
                    .placeholder("")
                    .name("Department")
                    .style(style().width(280)),
            )
        },
    );
    let trigger = screen.get_by_role(Role::ComboBox, "Department");
    assert_eq!(trigger.value().as_deref(), Some("Telemetry"));
    options.update(|items| items[1] = ListItem::new(items[1].key().clone(), "Telemetry renamed"));
    screen.run_until_idle();
    assert_eq!(trigger.value().as_deref(), Some("Telemetry renamed"));
    trigger.press_key(Key::Home);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:operations")
    );
    trigger.press_key(Key::End);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:telemetry")
    );
    trigger.a11y(A11yAction::Expand);
    assert!(
        screen
            .get_by_role(Role::Option, "Telemetry renamed")
            .is_focused()
    );
    screen
        .get_by_role(Role::Option, "Telemetry renamed")
        .press_key(Key::Home);
    assert!(screen.get_by_role(Role::Option, "Operations").is_focused());
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:telemetry"),
        "popup navigation is manual"
    );
    screen
        .get_by_role(Role::Option, "Operations")
        .press_key(Key::ArrowRight);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:telemetry"),
        "an unhandled popup key cannot run the closed trigger's logical route"
    );
    assert!(screen.get_by_role(Role::Option, "Operations").is_focused());
    assert!(screen.get_by_role(Role::Option, "Locked").is_disabled());
    options.update(|items| {
        items.reverse();
        items.retain(|item| item.key() != "department:telemetry");
    });
    screen.run_until_idle();
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:telemetry")
    );
    assert_eq!(trigger.value().as_deref(), Some(""));
    assert!(
        screen
            .query_by_role(Role::Option, "Telemetry renamed")
            .is_none()
    );
    assert!(screen.get_by_role(Role::Option, "Operations").is_focused());
    options.update(|items| {
        items.push(ListItem::new(
            "department:telemetry".into(),
            "Telemetry restored",
        ))
    });
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::Option, "Telemetry restored")
            .is_selected()
    );
    assert!(screen.get_by_role(Role::Option, "Operations").is_focused());
    screen
        .get_by_role(Role::Option, "Telemetry restored")
        .a11y(A11yAction::Select);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert_eq!(trigger.value().as_deref(), Some("Telemetry restored"));
    trigger.a11y(A11yAction::Expand);
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(
        screen
            .get_by_role(Role::Option, "Telemetry restored")
            .is_focused()
    );
    assert!(!trigger.is_focused());
    trigger.a11y(A11yAction::Collapse);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(
        screen
            .query_by_role(Role::Option, "Telemetry restored")
            .is_none()
    );
    assert!(
        trigger.is_focused(),
        "accessibility collapse restores the trigger"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());

    let (screen, (selected, rejection, opens)) = kivact::test::render_with_handle(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(Some("department:operations".to_owned()));
            let rejection = signal(0);
            let opens = signal(0);
            let binding = binding(selected.into_value(), move |key| -> Result<(), UiError> {
                match rejection.get_untracked() {
                    0 => Err(UiError::msg("department write rejected")),
                    1 => Ok(()),
                    _ => {
                        selected.set(key);
                        Ok(())
                    }
                }
            });
            (
                (selected, rejection, opens),
                kivact::ui::select("Select", choices(), binding)
                    .placeholder("")
                    .name("Controlled department")
                    .on_press(move || opens.update(|count| *count += 1)),
            )
        },
    );
    let trigger = screen.get_by_role(Role::ComboBox, "Controlled department");
    trigger.click();
    assert_eq!(
        opens.get_untracked(),
        1,
        "custom trigger action runs before the popup action"
    );
    screen
        .get_by_role(Role::Option, "Telemetry")
        .a11y(A11yAction::Select);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:operations")
    );
    assert_eq!(
        trigger.is_expanded(),
        Some(false),
        "select keeps its existing close-on-offer policy"
    );
    assert!(trigger.is_focused());
    assert_eq!(screen.errors().len(), 1);
    assert_eq!(screen.errors()[0].message(), "department write rejected");
    rejection.set(1);
    screen.run_until_idle();
    trigger.click();
    assert_eq!(opens.get_untracked(), 2);
    screen
        .get_by_role(Role::Option, "Operations")
        .press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Telemetry").is_focused());
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:operations")
    );
    screen
        .get_by_role(Role::Option, "Telemetry")
        .press_key(Key::Enter);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:operations")
    );
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    rejection.set(2);
    screen.run_until_idle();
    trigger.press_key(Key::End);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:telemetry")
    );
    assert_eq!(screen.errors().len(), 1);

    let (screen, (visible, selected, offers)) = kivact::test::render_with_handle(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let visible = signal(true);
            let selected = signal(Some("department:operations".to_owned()));
            let offers = signal(0);
            let binding = binding(selected.into_value(), move |key| {
                offers.update(|count| *count += 1);
                selected.set(key);
                visible.set(false);
            });
            (
                (visible, selected, offers),
                column((
                    show(visible, move || {
                        kivact::ui::select("Select", choices(), binding)
                            .placeholder("")
                            .name("Removed department")
                    }),
                    kivact::ui::button("After removed department"),
                )),
            )
        },
    );
    screen
        .get_by_role(Role::ComboBox, "Removed department")
        .click();
    screen
        .get_by_role(Role::Option, "Telemetry")
        .a11y(A11yAction::Select);
    assert!(!visible.get_untracked());
    assert_eq!(offers.get_untracked(), 1);
    assert_eq!(
        selected.get_untracked().as_deref(),
        Some("department:telemetry")
    );
    assert!(
        screen
            .query_by_role(Role::ComboBox, "Removed department")
            .is_none()
    );
    assert!(screen.query_by_role(Role::Option, "Telemetry").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen
        .get_by_role(Role::Button, "After removed department")
        .focus();
    assert!(
        screen
            .get_by_role(Role::Button, "After removed department")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn keyed_dropdown_keeps_keys_through_open_reorder_disable_and_readd() {
    let (screen, (items, selected, writes, custom, fail)) = kivact::test::render_with_handle(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let items = signal(dropdown_options(&[
                ("a", "Alpha"),
                ("b", "Beta"),
                ("c", "Gamma"),
            ]));
            let selected = signal(Some("b".to_owned()));
            let writes = signal(0usize);
            let custom = signal(0usize);
            let fail = signal(true);
            let binding = binding(selected.into_value(), move |key: Option<String>| {
                writes.update(|count| *count += 1);
                selected.set(key);
            });
            (
                (items, selected, writes, custom, fail),
                column((
                    kivact::ui::select("Selection", items, binding)
                        .placeholder("")
                        .test_id("dropdown")
                        .on_press(move || {
                            custom.update(|count| *count += 1);
                            if fail.get_untracked() {
                                Err(kivact::UiError::msg("open failed"))
                            } else {
                                Ok(())
                            }
                        }),
                    kivact::ui::select(
                        "Selection",
                        dropdown_options(&[("none", "Unselected dropdown")]),
                        signal(None),
                    )
                    .placeholder("")
                    .test_id("unbound-dropdown"),
                )),
            )
        },
    );
    assert!(
        screen
            .get_by_test_id("unbound-dropdown")
            .value()
            .unwrap_or_default()
            .is_empty()
    );
    let trigger = screen.get_by_test_id("dropdown");
    trigger.a11y(A11yAction::Activate);
    assert_eq!(custom.get_untracked(), 1);
    assert_eq!(screen.errors().len(), 1);
    assert!(screen.query_by_role(Role::Option, "Beta").is_none());
    fail.set(false);
    trigger.click();
    assert_eq!(custom.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Option, "Beta").exists());
    items.set(dropdown_options(&[
        ("c", "Gamma"),
        ("b", "Renamed Beta"),
        ("a", "Alpha"),
    ]));
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::Option, "Renamed Beta")
            .is_focused()
    );
    screen.press_key(Key::Home);
    assert!(screen.get_by_role(Role::Option, "Gamma").is_focused());
    assert_eq!(selected.get_untracked().as_deref(), Some("b"));
    assert_eq!(writes.get_untracked(), 0);
    screen
        .get_by_role(Role::Option, "Alpha")
        .a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked().as_deref(), Some("a"));
    assert_eq!(writes.get_untracked(), 1);
    assert!(trigger.is_focused());
    trigger.a11y(A11yAction::Expand);
    items.update(|items| {
        let item = items.iter_mut().find(|item| *item.key() == "a").unwrap();
        *item = item.clone().disabled(true);
    });
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Option, "Alpha").is_disabled());
    assert_eq!(writes.get_untracked(), 1);
    screen
        .get_by_role(Role::Option, "Gamma")
        .a11y(A11yAction::Activate);
    assert_eq!(writes.get_untracked(), 2);
    assert!(trigger.is_focused());
    items.update(|items| items.retain(|item| item.key() != "c"));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked().as_deref(), Some("c"));
    items.update(|items| items.push(ListItem::new("c".to_owned(), "Gamma restored")));
    screen.run_until_idle();
    trigger.a11y(A11yAction::Expand);
    assert!(
        screen
            .get_by_role(Role::Option, "Gamma restored")
            .is_selected()
    );
    assert!(
        screen
            .get_by_role(Role::Option, "Gamma restored")
            .is_focused()
    );
    assert_eq!(writes.get_untracked(), 2);
    assert_eq!(custom.get_untracked(), 2);
    assert_eq!(screen.errors().len(), 1);
}
