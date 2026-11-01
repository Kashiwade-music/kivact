//! Role: Verify shared controllers in a third-party device setup screen.

use super::render;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::ui::ListItem;
use kivact::view::A11yAction;
use official_ui_external_consumer::interactions::{
    DeviceKit, calibration, device_picker, official_device_choices,
};

#[test]
fn external_device_controls_share_identity_and_values_across_skins() {
    let screen = render(|| {
        let amount = signal(25.0);
        let selected = signal(Some("USB:front".to_owned()));
        let devices = signal(vec!["USB:front".to_owned(), "USB:rear".to_owned()]);
        column((
            calibration(amount, rgb(0x2b8bd8)).test_id("blue-calibration"),
            calibration(amount, rgb(0xcf782d)).test_id("orange-calibration"),
            device_picker(devices, selected),
            super::external_action("Reverse devices")
                .on_press(move || devices.update(|items| items.reverse())),
            text(move || selected.get().unwrap_or_default()).test_id("device-key"),
        ))
        .style(style().gap(12).padding(16))
    });
    let blue = screen.get_by_test_id("blue-calibration");
    let orange = screen.get_by_test_id("orange-calibration");
    blue.press_key(Key::ArrowRight);
    assert_eq!(blue.value_range().2, Some(26.0));
    assert_eq!(orange.value_range().2, Some(26.0));
    orange.a11y_with_value(A11yAction::SetValue, "73");
    assert_eq!(blue.value_range().2, Some(73.0));
    let rear = screen.get_by_role(Role::RadioButton, "USB:rear");
    rear.click();
    assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
    assert_eq!(rear.is_checked(), Some(true));
    screen.get_by_role(Role::Button, "Reverse devices").click();
    assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
    assert_eq!(
        screen
            .get_by_role(Role::RadioButton, "USB:rear")
            .is_checked(),
        Some(true)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

const FRONT: &str = "Front microphone";
const REAR: &str = "Rear microphone";
const MONITOR: &str = "Monitor (unavailable)";

fn device_options() -> Vec<ListItem<String>> {
    vec![
        ListItem::new("USB:front".to_owned(), FRONT),
        ListItem::new("USB:rear".to_owned(), REAR),
        ListItem::new("USB:monitor".to_owned(), MONITOR).disabled(true),
    ]
}

fn assert_front_is_the_only_group_tab_stop(screen: &Screen) {
    screen.get_by_role(Role::Button, "Before choices").focus();
    screen.press_tab(false);
    assert!(
        screen
            .get_by_test_id("official-tabs")
            .get_by_role(Role::Tab, FRONT)
            .is_focused()
    );
    screen.press_tab(false);
    assert!(
        screen
            .get_by_test_id("official-radios")
            .get_by_role(Role::RadioButton, FRONT)
            .is_focused()
    );
    screen.press_tab(false);
    assert!(screen.get_by_test_id("official-select").is_focused());
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::Button, "After choices")
            .is_focused()
    );
}

#[test]
fn all_official_styled_keyed_routes_preserve_missing_selection_and_roving_focus() {
    for kit in [DeviceKit::Pop, DeviceKit::Game, DeviceKit::Creative] {
        let screen = render(move || {
            let selected = signal(Some("USB:rear".to_owned()));
            let devices = signal(device_options());
            let filter_rear = signal(false);
            let visible = memo(move || {
                let filtered = filter_rear.get();
                devices.with(|items| {
                    items
                        .iter()
                        .filter(|item| !(filtered && *item.key() == "USB:rear"))
                        .cloned()
                        .collect::<Vec<_>>()
                })
            });
            column((
                super::external_action("Before choices"),
                official_device_choices(kit, visible, selected),
                super::external_action("After choices"),
                row((
                    super::external_action("Reverse devices")
                        .on_press(move || devices.update(|items| items.reverse())),
                    super::external_action("Disable rear").on_press(move || {
                        devices.update(|items| {
                            items.iter_mut().for_each(|item| {
                                if *item.key() == "USB:rear" {
                                    *item = item.clone().disabled(true);
                                }
                            })
                        })
                    }),
                    super::external_action("Enable rear").on_press(move || {
                        devices.update(|items| {
                            items.iter_mut().for_each(|item| {
                                if *item.key() == "USB:rear" {
                                    *item = item.clone().disabled(false);
                                }
                            })
                        })
                    }),
                )),
                row((
                    super::external_action("Filter rear").on_press(move || filter_rear.set(true)),
                    super::external_action("Reveal rear").on_press(move || filter_rear.set(false)),
                    super::external_action("Delete rear").on_press(move || {
                        devices.update(|items| items.retain(|item| *item.key() != "USB:rear"))
                    }),
                    super::external_action("Readd rear").on_press(move || {
                        devices
                            .update(|items| items.push(ListItem::new("USB:rear".to_owned(), REAR)))
                    }),
                )),
                text(move || selected.get().unwrap_or_default()).test_id("device-key"),
            ))
            .style(style().padding(16).gap(8))
        });
        let tabs = screen.within(screen.get_by_test_id("official-tabs"));
        let radios = screen.within(screen.get_by_test_id("official-radios"));
        assert!(tabs.get_by_role(Role::Tab, REAR).is_selected(), "{kit:?}");
        assert_eq!(
            radios.get_by_role(Role::RadioButton, REAR).is_checked(),
            Some(true),
            "{kit:?}"
        );
        let created = screen.stats().creates;
        screen
            .get_by_role(Role::Button, "Reverse devices")
            .a11y(A11yAction::Activate);
        assert_eq!(screen.stats().creates, created, "{kit:?}: keyed reorder");
        assert!(tabs.get_by_role(Role::Tab, REAR).is_selected());
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");

        screen
            .get_by_role(Role::Button, "Disable rear")
            .a11y(A11yAction::Activate);
        assert!(tabs.get_by_role(Role::Tab, REAR).is_disabled());
        assert!(radios.get_by_role(Role::RadioButton, REAR).is_disabled());
        assert_front_is_the_only_group_tab_stop(&screen);
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
        screen
            .get_by_role(Role::Button, "Enable rear")
            .a11y(A11yAction::Activate);
        assert!(
            screen
                .get_by_role(Role::Button, "After choices")
                .is_focused()
        );

        let old_rear = tabs.get_by_role(Role::Tab, REAR);
        screen
            .get_by_role(Role::Button, "Filter rear")
            .a11y(A11yAction::Activate);
        assert!(!old_rear.exists());
        assert!(tabs.query_by_role(Role::Tab, REAR).is_none());
        assert!(radios.query_by_role(Role::RadioButton, REAR).is_none());
        assert_front_is_the_only_group_tab_stop(&screen);
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
        screen
            .get_by_role(Role::Button, "Reveal rear")
            .a11y(A11yAction::Activate);
        assert!(tabs.get_by_role(Role::Tab, REAR).is_selected());
        assert_eq!(
            radios.get_by_role(Role::RadioButton, REAR).is_checked(),
            Some(true)
        );
        assert!(
            screen
                .get_by_role(Role::Button, "After choices")
                .is_focused()
        );

        let owners = screen.stats().live_owners;
        let objects = screen.stats().live_objects;
        let old_rear = radios.get_by_role(Role::RadioButton, REAR);
        screen
            .get_by_role(Role::Button, "Delete rear")
            .a11y(A11yAction::Activate);
        assert!(!old_rear.exists());
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
        assert_front_is_the_only_group_tab_stop(&screen);
        screen
            .get_by_role(Role::Button, "Readd rear")
            .a11y(A11yAction::Activate);
        assert!(tabs.get_by_role(Role::Tab, REAR).is_selected());
        assert_eq!(
            radios.get_by_role(Role::RadioButton, REAR).is_checked(),
            Some(true)
        );
        assert!(
            screen
                .get_by_role(Role::Button, "After choices")
                .is_focused()
        );
        assert_eq!(screen.stats().live_owners, owners, "{kit:?}: row cleanup");
        assert_eq!(screen.stats().live_objects, objects, "{kit:?}: row cleanup");

        let trigger = screen.get_by_test_id("official-select");
        trigger.a11y(A11yAction::Expand);
        assert!(screen.get_by_role(Role::Option, MONITOR).is_disabled());
        screen
            .get_by_role(Role::Option, FRONT)
            .a11y(A11yAction::Select);
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:front");
        assert!(tabs.get_by_role(Role::Tab, FRONT).is_selected());
        assert_eq!(
            radios.get_by_role(Role::RadioButton, FRONT).is_checked(),
            Some(true)
        );
        assert!(screen.query_by_role(Role::Option, FRONT).is_none());
        assert!(trigger.is_focused());
        // Reopening mounts new option owners against the current shared key.
        trigger.a11y(A11yAction::Expand);
        assert!(screen.get_by_role(Role::Option, FRONT).is_selected());
        trigger.a11y(A11yAction::Collapse);
        assert!(screen.query_by_role(Role::Option, FRONT).is_none());
        assert!(screen.errors().is_empty(), "{kit:?}: {:?}", screen.errors());
    }
}

#[test]
fn all_official_keyed_routes_offer_each_write_once_without_claiming_acceptance() {
    for kit in [DeviceKit::Pop, DeviceKit::Game, DeviceKit::Creative] {
        let screen = render(move || {
            let selected = signal(Some("USB:rear".to_owned()));
            let deny = signal(true);
            let fail = signal(false);
            let offers = signal(Vec::<Option<String>>::new());
            let choice = kivact::binding(
                selected.into_value(),
                move |key: Option<String>| -> kivact::Result<()> {
                    offers.update(|offers| offers.push(key.clone()));
                    if fail.get_untracked() {
                        return Err(UiError::msg("device selection rejected"));
                    }
                    if !deny.get_untracked() {
                        selected.set(key);
                    }
                    Ok(())
                },
            );
            column((
                official_device_choices(kit, device_options(), choice),
                super::external_action("Fail writes").on_press(move || {
                    deny.set(false);
                    fail.set(true);
                }),
                super::external_action("Accept writes").on_press(move || {
                    deny.set(false);
                    fail.set(false);
                }),
                text(move || selected.get().unwrap_or_default()).test_id("device-key"),
                text(move || offers.with(|offers| offers.len().to_string())).test_id("offers"),
                text(move || {
                    offers.with(|offers| offers.last().cloned().flatten().unwrap_or_default())
                })
                .test_id("offered-key"),
            ))
            .style(style().padding(16).gap(8))
        });
        let tabs = screen.within(screen.get_by_test_id("official-tabs"));
        let radios = screen.within(screen.get_by_test_id("official-radios"));
        let front_radio = radios.get_by_role(Role::RadioButton, FRONT);
        front_radio.a11y(A11yAction::Select);
        assert_eq!(screen.get_by_test_id("offers").text(), "1", "{kit:?}");
        assert_eq!(screen.get_by_test_id("offered-key").text(), "USB:front");
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
        assert_eq!(front_radio.is_checked(), Some(false));
        assert!(front_radio.is_focused());
        assert!(tabs.get_by_role(Role::Tab, REAR).is_selected());
        assert!(screen.errors().is_empty());

        screen
            .get_by_role(Role::Button, "Fail writes")
            .a11y(A11yAction::Activate);
        let front_tab = tabs.get_by_role(Role::Tab, FRONT);
        front_tab.a11y(A11yAction::Select);
        assert_eq!(screen.get_by_test_id("offers").text(), "2");
        assert_eq!(screen.errors().len(), 1);
        assert!(front_tab.is_focused());
        assert!(!front_tab.is_selected());
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
        assert_eq!(
            radios.get_by_role(Role::RadioButton, REAR).is_checked(),
            Some(true)
        );

        let trigger = screen.get_by_test_id("official-select");
        trigger.a11y(A11yAction::Expand);
        screen
            .get_by_role(Role::Option, FRONT)
            .a11y(A11yAction::Select);
        assert_eq!(screen.get_by_test_id("offers").text(), "3");
        assert_eq!(screen.errors().len(), 2);
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:rear");
        assert!(screen.query_by_role(Role::Option, FRONT).is_none());
        assert!(trigger.is_focused());

        screen
            .get_by_role(Role::Button, "Accept writes")
            .a11y(A11yAction::Activate);
        front_radio.a11y(A11yAction::Select);
        assert_eq!(screen.get_by_test_id("offers").text(), "4");
        assert_eq!(screen.get_by_test_id("device-key").text(), "USB:front");
        assert_eq!(front_radio.is_checked(), Some(true));
        assert!(front_tab.is_selected());
        assert_eq!(screen.errors().len(), 2, "{kit:?}");
    }
}
