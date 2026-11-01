//! Role: Tests for region actions and accessibility.
//!
//! A drawn item remains identifiable when assistive technology requests an
//! action, without a component reaching into Kivact's retained tree.

use crate::support::mount;
use kivact::control::canvas;
use kivact::draw::{Rect, RegionId};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::view::A11yEvent;
use kivact::view::{A11yAction, callback};

#[test]
fn accessibility_events_distinguish_drawn_regions_and_the_root() {
    let (screen, (invoked, selected, disabled)) = mount(|| {
        let invoked = signal(None::<u64>);
        let selected = signal(None::<u64>);
        let disabled = signal(false);
        let invoke = callback(move |event: A11yEvent| {
            assert_eq!(event.action, A11yAction::Activate);
            invoked.set(event.region.map(|region| region.0));
        });
        let select = callback(move |event: A11yEvent| {
            assert_eq!(event.action, A11yAction::Select);
            selected.set(event.region.map(|region| region.0));
        });
        let mut editor = canvas(|draw, _| {
            for (id, x) in [(11, 0.0), (22, 100.0)] {
                draw.region(RegionId(id), Rect::new(x, 0.0, 90.0, 60.0));
            }
        });
        for (id, name) in [(11, "First clip"), (22, "Second clip")] {
            editor = editor.region(
                RegionId(id),
                kivact::primitive::part(|state| state)
                    .role(Role::Button)
                    .name(name)
                    .on_a11y_event(A11yAction::Activate, move |event| invoke.call(event))
                    .on_a11y_event(A11yAction::Select, move |event| select.call(event)),
            );
        }
        let editor = editor
            .name("Clip editor")
            .style(style().size(200, 60))
            .disabled(disabled)
            .on_a11y_event(A11yAction::Activate, move |event| invoke.call(event))
            .on_a11y_event(A11yAction::Select, move |event| select.call(event));
        ((invoked, selected, disabled), editor)
    });
    let root = screen.get_by_role(Role::Canvas, "Clip editor");
    let first = root.get_by_role(Role::Button, "First clip");
    let second = root.get_by_role(Role::Button, "Second clip");
    second.a11y(A11yAction::Activate);
    assert_eq!(invoked.get(), Some(22));
    first.a11y(A11yAction::Activate);
    assert_eq!(invoked.get(), Some(11));
    second.a11y(A11yAction::Select);
    assert_eq!(selected.get(), Some(22));
    root.a11y(A11yAction::Activate);
    assert_eq!(invoked.get(), None, "the root is not a drawn region");
    disabled.set(true);
    screen.run_until_idle();
    assert!(first.a11y_actions().is_empty());
    assert!(second.a11y_actions().is_empty());
    second.a11y(A11yAction::Activate);
    assert_eq!(invoked.get(), None, "disabled regions reject actions");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn explicit_toggle_has_its_own_event_and_legacy_aliases_still_activate() {
    let (screen, (explicit, legacy)) = mount(|| {
        let explicit = signal(0);
        let legacy = signal(0);
        let view = column((
            node()
                .role(Role::CheckBox)
                .name("Explicit toggle")
                .checked(false)
                .style(style().size(160, 36))
                .on_a11y_event(A11yAction::Toggle, move |event| {
                    assert_eq!(event.action, A11yAction::Toggle);
                    assert_eq!(event.region, None);
                    explicit.update(|n| *n += 1);
                }),
            node()
                .role(Role::CheckBox)
                .name("Legacy toggle")
                .checked(false)
                .style(style().size(160, 36))
                .on_press(move || legacy.update(|n| *n += 1)),
        ));
        ((explicit, legacy), view)
    });
    screen
        .get_by_role(Role::CheckBox, "Explicit toggle")
        .a11y(A11yAction::Toggle);
    // A native AccessKit Click arrives as Activate even when the component
    // offers only Toggle. It must reach the explicitly offered handler.
    screen
        .get_by_role(Role::CheckBox, "Explicit toggle")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::CheckBox, "Legacy toggle")
        .a11y(A11yAction::Toggle);
    screen
        .get_by_role(Role::CheckBox, "Legacy toggle")
        .a11y(A11yAction::Select);
    assert_eq!(explicit.get(), 2);
    assert_eq!(legacy.get(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn accessibility_set_value_preserves_the_requested_payload() {
    let (screen, received) = mount(|| {
        let received = signal(None::<String>);
        let view = node()
            .role(Role::Slider)
            .name("External range")
            .value_min(0.)
            .value_max(100.)
            .value_now(0.)
            .style(style().size(200, 36))
            .on_a11y_event(A11yAction::SetValue, move |event| {
                assert_eq!(event.action, A11yAction::SetValue);
                assert_eq!(event.region, None);
                received.set(event.value);
            });
        (received, view)
    });
    screen
        .get_by_role(Role::Slider, "External range")
        .a11y_with_value(A11yAction::SetValue, "37.5");
    assert_eq!(received.get().as_deref(), Some("37.5"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn native_click_alias_reaches_a_select_only_drawn_region() {
    let (screen, selected) = mount(|| {
        let selected = signal(None::<u64>);
        let select = callback(move |event: A11yEvent| {
            assert_eq!(event.action, A11yAction::Select);
            selected.set(event.region.map(|id| id.0));
        });
        let view = canvas(|draw, _| {
            draw.region(RegionId(42), Rect::new(0., 0., 120., 40.));
        })
        .region(
            RegionId(42),
            kivact::primitive::part(|state| state)
                .role(Role::Tab)
                .name("Third-party tab")
                .on_a11y_event(A11yAction::Select, move |event| select.call(event)),
        )
        .name("External tabs")
        .style(style().size(200, 60))
        .on_a11y_event(A11yAction::Select, move |event| select.call(event));
        (selected, view)
    });
    screen
        .get_by_role(Role::Tab, "Third-party tab")
        .a11y(A11yAction::Activate);
    assert_eq!(selected.get(), Some(42));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn drawn_region_states_update_without_remounting_the_external_component() {
    let (screen, active) = mount(|| {
        let active = signal(false);
        let toggle = callback(move |event: A11yEvent| {
            assert_eq!(event.region, Some(RegionId(17)));
            active.update(|value| *value = !*value);
        });
        let view = canvas(|draw, _| {
            draw.region(RegionId(17), Rect::new(0., 0., 100., 40.));
            draw.region(RegionId(18), Rect::new(100., 0., 100., 40.));
        })
        .region(
            RegionId(17),
            kivact::primitive::part(|state| state)
                .role(Role::CheckBox)
                .name("External track mute")
                .checked(active)
                .indeterminate(move || !active.get())
                .on_a11y_event(A11yAction::Toggle, move |event| toggle.call(event)),
        )
        .region(
            RegionId(18),
            kivact::primitive::part(|state| state)
                .role(Role::TreeItem)
                .name("External track")
                .selected(active)
                .expanded(active)
                // The previous region advertised Select without implementing it.
                .on_a11y(A11yAction::Select, || {}),
        )
        .name("External arrangement")
        .style(style().size(200, 60))
        .on_a11y_event(A11yAction::Toggle, move |event| toggle.call(event));
        (active, view)
    });
    let mute = screen.get_by_role(Role::CheckBox, "External track mute");
    let track = screen.get_by_role(Role::TreeItem, "External track");
    assert_eq!(mute.is_checked(), Some(false));
    assert!(mute.is_indeterminate());
    assert!(!track.is_selected());
    assert_eq!(track.is_expanded(), Some(false));
    mute.a11y(A11yAction::Activate);
    assert!(active.get());
    assert_eq!(mute.is_checked(), Some(true));
    assert!(!mute.is_indeterminate());
    assert!(track.is_selected());
    assert_eq!(track.is_expanded(), Some(true));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
