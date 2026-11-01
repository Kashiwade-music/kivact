//! Role: Detail view of one device with two timelines and range statistics.
//!
//! Detail of one device: two timelines (the same component twice) whose
//! selected ranges give statistics.

use kivact::prelude::*;
use kivact::style::current_theme;
use kivact_external_component::skin::{self, look};
use kivact_external_component::{Span, timeline};

use super::data::{Device, HISTORY};

fn stats(values: &[f32], span: Option<Span>, unit: &str) -> String {
    let (lo, hi) = match span {
        Some(s) => (s.start.min(values.len()), s.end.min(values.len())),
        None => (0, values.len()),
    };
    let part = &values[lo..hi];
    if part.is_empty() {
        return "no samples".into();
    }
    let min = part.iter().copied().fold(f32::INFINITY, f32::min);
    let max = part.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let avg = part.iter().sum::<f32>() / part.len() as f32;
    format!(
        "{} samples · min {min:.1} {unit} · avg {avg:.1} {unit} · max {max:.1} {unit}",
        part.len()
    )
}

/// Detail of device `id` of `devices`.
pub fn device_detail(devices: Signal<Vec<Device>>, id: Value<u32>) -> impl IntoView {
    let look = look();
    let dev = memo(move || {
        let i = id.get() as usize;
        devices.with(|all| all.get(i).cloned())
    });
    let temp = memo(move || dev.with(|d| d.as_ref().map(|d| d.temp.clone()).unwrap_or_default()));
    let volt = memo(move || dev.with(|d| d.as_ref().map(|d| d.volt.clone()).unwrap_or_default()));
    let temp_sel = signal(None::<Span>);
    let volt_sel = signal(None::<Span>);
    let sample = |i: usize| format!("t-{}", HISTORY - i);
    column((
        text(move || {
            dev.with(|d| match d {
                Some(d) => format!("{} · {} · {}", d.name, d.zone, d.status.label()),
                None => "gone".into(),
            })
        })
        .style(style().font_size(16).weight(600))
        .test_id("title"),
        text("Temperature (°C)"),
        timeline("Temperature history", temp, temp_sel)
            .bucket_label(sample)
            .height(90.0)
            .test_id("temp"),
        text(move || temp.with(|v| stats(v, temp_sel.get(), "°C"))).test_id("temp-stats"),
        text("Supply voltage (V)"),
        timeline("Voltage history", volt, volt_sel)
            .bucket_label(sample)
            .height(90.0)
            .test_id("volt"),
        text(move || volt.with(|v| stats(v, volt_sel.get(), "V"))).test_id("volt-stats"),
    ))
    .style(move || {
        let t = &current_theme();
        skin::panel(look(), t)
    })
    .style(style().grow(1).min_width(0))
}
