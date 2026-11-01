//! Role: Device controls compose shared behavior and themed standard UI.

use kivact::Binding;
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, ListPolicy, ListSelection, use_drag, use_list,
    use_scalar,
};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::ui::{ListItem, ScalarRange};

/// Shared input state with an application-specific skin. The scalar supplies
/// the slider semantics; this consumer owns the local pointer mapping.
pub fn calibration(value: impl Into<Binding<f64>>, color: Color) -> Node {
    let range = ScalarRange::new(0.0, 100.0)
        .expect("invariant: finite ordered bounds")
        .with_step(1.0)
        .expect("invariant: positive finite step");
    let control = use_scalar(value, range);
    // The value at the press; a drag moves it by the pointer's travel.
    let origin = signal(0.0);
    let follow = move |update: DragUpdate<()>| {
        if update.phase != DragPhase::Cancel {
            control.set(origin.get_untracked() + f64::from(update.delta.x));
        }
    };
    let pointer = use_drag::<()>(DragCoordinates::Local)
        .on_start(move |_| origin.set(control.value().get_untracked()))
        .on_update(follow)
        .on_end(follow);
    node()
        .part(control.thumb())
        .part(pointer.source(()))
        .name("Device calibration")
        .style(style().size(220, 40).background(color))
        .child(text(move || format!("{}%", control.value().get())))
}

/// An application-owned identity survives item reordering. Labels are owned
/// at the builder's entry; the caller does not need to own a `'static` string.
pub fn device_picker(
    devices: impl IntoValue<Vec<String>>,
    selected: impl Into<Binding<Option<String>>>,
) -> impl IntoView {
    let devices = devices.into_value();
    let group = use_list(
        move || {
            devices
                .get()
                .into_iter()
                .map(|key| ListItem::new(key.clone(), key))
                .collect::<Vec<_>>()
        },
        ListSelection::Single(selected.into()),
    )
    .policy(ListPolicy::radio());
    node()
        .role(Role::RadioGroup)
        .name("Input device")
        .child(for_each(devices, Clone::clone, move |id, key| {
            {
                let member = group.item(id.clone()).role(Role::RadioButton).name(id);
                let state = member.state();
                node().part(member.checked(move || state.get().selected))
            }
            .style(style().height(36))
            .child(text(key))
        }))
}

/// Official styled routes over the same application-owned device IDs.
#[derive(Clone, Copy, Debug)]
pub enum DeviceKit {
    Pop,
    Game,
    Creative,
}

/// Each theme skins standard tabs and select beside a kit radio group. The app
/// passes String keys unchanged and shares one binding across all three views.
pub fn official_device_choices(
    kit: DeviceKit,
    devices: impl IntoValue<Vec<ListItem<String>>>,
    selected: impl Into<Binding<Option<String>>>,
) -> Node {
    let devices = devices.into_value();
    let selected = selected.into();
    let theme = match kit {
        DeviceKit::Pop => kivact_ui_pop::theme(kivact::style::ThemeMode::Light),
        DeviceKit::Game => kivact_ui_game::theme(kivact::style::ThemeMode::Light),
        DeviceKit::Creative => kivact_ui_creative::theme(kivact::style::ThemeMode::Dark),
    };
    node().child(kivact::view::provide_theme(theme, move || {
        // The tabs always hold a device; the radio group and select may hold none.
        let tab = kivact::binding(
            move || selected.get().unwrap_or_default(),
            move |device: String| selected.set(Some(device)),
        );
        let tabs = kivact::ui::tabs("Devices", devices, tab)
            .test_id("official-tabs")
            .into_view();
        let radio = match kit {
            DeviceKit::Pop => kivact::ui::radio_group("Device", devices, selected)
                .orientation(kivact::view::Orientation::Horizontal)
                .variant(kivact_ui_pop::Segmented)
                .test_id("official-radios")
                .into_view(),
            DeviceKit::Game => kivact::ui::radio_group("Device", devices, selected)
                .orientation(kivact::view::Orientation::Horizontal)
                .variant(kivact_ui_game::Segmented)
                .test_id("official-radios")
                .into_view(),
            DeviceKit::Creative => kivact::ui::radio_group("Device", devices, selected)
                .test_id("official-radios")
                .into_view(),
        };
        let select = kivact::ui::select("Input device", devices, selected)
            .test_id("official-select")
            .into_view();
        column((tabs, radio, select)).style(style().gap(8).width(320))
    }))
}
