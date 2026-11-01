//! Role: Device status monitor: a live board of sensors.

mod data;
mod detail;

pub use data::{Device, HISTORY, Status, fleet, tick};
pub use detail::device_detail;

use kivact::draw::{Draw, Size};
use kivact::prelude::*;
use kivact::style::current_theme;
use kivact_external_component::skin::{self, Skin, look};
use kivact_external_component::{lazy_list, provide_skin, search_box, sparkline};

use crate::Rng;

/// Root of the device monitor over `devices`, starting in `skin`. Every
/// `tick_ms` the sensors get a simulated sample (`None` = static data).
pub fn app(devices: Vec<Device>, skin: Skin, tick_ms: Option<u64>) -> impl IntoView {
    let skin = signal(skin);
    provide_skin(skin, move || board(devices, skin, tick_ms))
}

fn board(devices: Vec<Device>, skin: Signal<Skin>, tick_ms: Option<u64>) -> impl IntoView {
    let look = look();
    let devices = signal(devices);
    let query = signal(String::new());
    let alarms_only = signal(false);
    let selected = signal(Some(0u32));
    if let Some(ms) = tick_ms {
        let rng = var(Rng::new(17));
        spawn::<()>(async move {
            loop {
                kivact::task::sleep(std::time::Duration::from_millis(ms)).await;
                let mut r = rng.get();
                devices.update(|d| tick(d, &mut r));
                rng.set(r);
            }
        });
    }
    let shown = memo(move || {
        let needle = query.with(|q| q.trim().to_lowercase());
        let only = alarms_only.get();
        devices.with(|all| {
            all.iter()
                .filter(|d| !only || matches!(d.status, Status::Alarm | Status::Offline))
                .filter(|d| {
                    needle.is_empty()
                        || d.name.contains(&needle)
                        || d.zone.to_lowercase().contains(&needle)
                })
                .map(|d| d.id)
                .collect::<Vec<u32>>()
        })
    });
    let counts = memo(move || {
        devices.with(|all| {
            let mut c = [0usize; 4];
            for d in all {
                c[d.status as usize] += 1;
            }
            c
        })
    });
    let summary = move || {
        let c = counts.get();
        format!(
            "{} devices · OK {} · WARN {} · ALARM {} · OFFLINE {}",
            devices.with(|d| d.len()),
            c[0],
            c[1],
            c[2],
            c[3]
        )
    };
    let shown_label = memo(move || format!("{} shown", shown.with(|s| s.len())));
    let list = lazy_list(
        "Devices",
        shown,
        |id: &u32| *id,
        move |_id: u32, id: Value<u32>| device_row(devices, id),
    )
    .row_height(30.0)
    .selection(selected)
    .item_name(|id: &u32| format!("device {id}"))
    // Fixed viewport: the number of mounted (visible) rows, hence the
    // semantics, must not depend on the skin's type size.
    .style(style().height(480).shrink(0))
    .test_id("devices");
    column((
        row((
            text("Kivact Device Monitor").style(style().font_size(18).weight(600)),
            spacer(),
            text(summary).role(Role::Status).test_id("summary"),
            button(move || format!("Skin: {}", skin.get().name()))
                .variant(ButtonVariant::Secondary)
                .on_press(move || skin.update(|s| *s = s.other()))
                .test_id("skin"),
        ))
        .style(style().align_items(Align::Center).gap(12)),
        row((
            column((
                search_box("Find device", query)
                    .placeholder("name or zone")
                    .status(shown_label)
                    .test_id("find"),
                checkbox("Alarms only", alarms_only).test_id("alarms"),
                list,
            ))
            .style(move || {
                let t = &current_theme();
                skin::panel(look(), t)
            })
            .style(style().width(420).shrink(0).min_height(0)),
            show_some(selected, move |id: Value<u32>| device_detail(devices, id))
                .fallback(|| text("Select a device")),
        ))
        .style(style().gap(12).grow(1).min_height(0)),
    ))
    .style(move || {
        let t = &current_theme();
        style()
            .padding(12)
            .gap(12)
            .grow(1)
            .background(ColorToken::SurfaceAlt)
            .color(ColorToken::Text)
            .font_size(t.text().size)
    })
}

/// One device row: status chip, name, zone, temperature and a sparkline
/// drawn by the timeline's drawing function in a local draw layer.
fn device_row(devices: Signal<Vec<Device>>, id: Value<u32>) -> impl IntoView {
    let look = look();
    let dev = memo(move || {
        let i = id.get() as usize;
        devices.with(|all| all.get(i).cloned())
    });
    let status =
        memo(move || dev.with(|d| d.as_ref().map(|d| d.status).unwrap_or(Status::Offline)));
    (
        text(move || status.get().label().to_string())
            .style(move || {
                let t = &current_theme();
                skin::chip(look(), t, status.get().token())
            })
            .style(style().width(64).shrink(0).text_wrap(false)),
        text(move || dev.with(|d| d.as_ref().map(|d| d.name.clone()).unwrap_or_default()))
            .style(style().width(96).shrink(0).text_wrap(false)),
        text(move || {
            dev.with(|d| d.as_ref().map(|d| d.zone).unwrap_or(""))
                .to_string()
        })
        .style(
            style()
                .color(ColorToken::TextMuted)
                .width(90)
                .shrink(0)
                .text_wrap(false),
        ),
        text(move || {
            dev.with(|d| {
                d.as_ref()
                    .map(|d| format!("{:.1} °C", d.temp_now()))
                    .unwrap_or_default()
            })
        })
        .style(style().width(64).shrink(0).text_wrap(false)),
        node().style(style().width(60).height(18).shrink(0)).draw(
            move |d: &mut Draw, size: Size| {
                let token = status.get().token();
                dev.with(|x| {
                    if let Some(x) = x {
                        sparkline(d, size, &x.temp[x.temp.len().saturating_sub(16)..], token)
                    }
                })
            },
        ),
    )
}
