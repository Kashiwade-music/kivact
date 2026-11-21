//! Numeric controls and sample display for creative.
use super::audio_values::TransportState;
use kivact::state::Signal;

use super::common::{bind, col, note, spec, specs};
use super::controls::{num, slide};
use kivact::prelude::{Element, IntoView, column, rgb, row, signal};
use kivact::style::{pct, style};
use kivact_ui_creative as ui;

pub(super) fn tc(label: &str, state: Signal<f64>) -> ui::Number {
    ui::number(label.to_owned(), state)
        .range(0.0..=1_000_000.0)
        .timecode(24)
        .label_visible(false)
        .style(style().width(pct(100)))
}
pub(super) fn axis_num(
    label: &str,
    value: f64,
    unit: &str,
    precision: usize,
    color: u32,
) -> ui::Number {
    num(label, value, -65536.0, 65536.0, precision, unit).style(
        style()
            .border_radius(3)
            .border_left(3, rgb(color).with_alpha(0.65)),
    )
}
pub(super) fn animation_property(
    label: &str,
    initial: f64,
    tint: ui::PropertyState,
) -> ui::PropertyRow {
    let name = label.to_owned();
    let value = signal(initial);
    let keyed = signal(tint == ui::PropertyState::Keyed);
    let state = move || {
        if keyed.get() {
            ui::PropertyState::Keyed
        } else if tint == ui::PropertyState::Keyed {
            ui::PropertyState::Plain
        } else {
            tint
        }
    };
    let field: kivact::prelude::View = if tint == ui::PropertyState::Animated {
        ui::value_slider(name, value)
            .range(0.0..=1.0)
            .property_state(state)
            .label_visible(false)
            .style(style().width(pct(100)))
            .into_view()
    } else {
        ui::number(name, value)
            .range(-65536.0..=65536.0)
            .property_state(state)
            .label_visible(false)
            .style(style().width(pct(100)))
            .into_view()
    };
    ui::property_row(label.to_owned(), field).leading({
        let label = label.to_owned();
        {
            ui::property_state_button(label, state)
                .on_press(move || keyed.update(|keyed| *keyed = !*keyed))
        }
    })
}
