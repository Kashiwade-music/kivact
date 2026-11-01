//! Role: Numeric controls and sample display for creative.
use crate::creative::audio_values::TransportState;

use super::common::{bind, col, note, spec, specs};
use super::controls::{num, slide};
use kivact::Binding;
use kivact::prelude::{IntoView, NodeBuilder, Signal, column, rgb, row, signal};
use kivact::style::{pct, style};
use kivact_ui_creative as ui;

pub(super) fn numeric(transport: Signal<TransportState>) -> impl IntoView {
    specs((
        spec("Drag Number", "ドラッグ数値", col((
            num("Focal Length", 50.0, 1.0, 5000.0, 0, " mm"),
            num("Exposure", 0.0, -10.0, 10.0, 3, " EV"),
            num("Samples", 128.0, 1.0, 4096.0, 0, ""),
            num("Frame Rate", 24.0, 1.0, 240.0, 3, " fps").label_visible(false).step(0.001),
            note("左右ドラッグ／Ctrl 微調整／Shift 粗調整／クリックで入力（数式可: 50*2）／ホバー中 Backspace で既定値／ホイール+Ctrl で増減。"),
        ))).style(style().min_height(258)),
        spec("Slider Field", "スライダー", col((
            ui::value_slider("Opacity", signal(100.0)).range(0.0,100.0).precision(1).step(0.1).unit(" %").reset_to(100.0).style(style().width(pct(100))),
            slide("Roughness", 0.5).step(0.001),
            slide("Strength", 0.82).step(0.001).reset_to(1.0),
            slide("Disabled", 0.3).precision(2).disabled(true),
        ))),
        spec("Vector (Column)", "ベクトル入力", col((
            ui::property_row("Location X", column((
                axis_num("X",0.0," m",3,0xff5c6c).label_visible(false),
                axis_num("Y",1.25," m",3,0x34d399),
                axis_num("Z",0.5," m",3,0x5aa9ff),
            )).style(style().gap(1))),
            ui::property_row("Rotation", row((
                axis_num("Rotation X",90.0,"°",1,0xff5c6c).label_visible(false).style(style().width(0).grow(1)),
                axis_num("Rotation Y",0.0,"°",1,0x34d399).label_visible(false).style(style().width(0).grow(1)),
                axis_num("Rotation Z",-35.0,"°",1,0x5aa9ff).label_visible(false).style(style().width(0).grow(1)),
            )).style(style().gap(1))),
        )).style(style().gap(3))),
        spec("Animatable Property", "キーフレーム状態", col((
            animation_property("Scale",1.0,ui::PropertyState::Plain),
            animation_property("Keyed",0.75,ui::PropertyState::Keyed),
            animation_property("Animated",0.4,ui::PropertyState::Animated),
            animation_property("Driven",2.5,ui::PropertyState::Driven),
            note("◇ をクリックでキー挿入（黄）／再クリックで削除。未キーのフレームは緑、ドライバー駆動は紫。"),
        )).style(style().gap(3))),
        spec("Timecode Field", "タイムコード", col((
            ui::property_row("Start", tc("Start timecode", signal(0.0).into())),
            ui::property_row("End", tc("End timecode", signal(5760.0).into())),
            ui::property_row("Duration", tc("Duration timecode", signal(86.0).into())),
            ui::property_row("Current", tc("Current timecode", bind(move || transport.get().frame, move |frame| transport.update(|state| state.frame = frame)))),
            note("ドラッグでフレーム単位スクラブ／クリックで HH:MM:SS:FF 入力。+12 のような相対入力も可。"),
        )).style(style().gap(3))),
        spec("Inline Pairs", "横並び", col((
            ui::property_row("Resolution", column((
                num("X",1920.0,4.0,65536.0,0," px").style(style().radius(3)),
                num("Y",1080.0,4.0,65536.0,0," px").style(style().radius(3)),
                ui::value_slider("%", signal(100.0)).range(1.0,100.0).precision(0).step(1.0).unit(" %").style(style().width(pct(100)).radius(3)),
            )).style(style().gap(1))),
            ui::property_row("Frame Range", row((
                num("Range start",1.0,0.0,65536.0,0,"").label_visible(false).style(style().width(0).grow(1)),
                num("Range end",250.0,0.0,65536.0,0,"").label_visible(false).style(style().width(0).grow(1)),
            )).style(style().gap(1))),
            ui::property_row("Tempo", num("BPM",120.0,20.0,300.0,3,"")),
        )).style(style().gap(3))).style(style().min_height(190)),
    ))
}
fn tc(label: &str, state: Binding<f64>) -> ui::Numeric {
    ui::number(label.to_owned(), state)
        .range(0.0, 1_000_000.0)
        .timecode(24)
        .label_visible(false)
        .style(style().width(pct(100)))
}
fn axis_num(label: &str, value: f64, unit: &str, precision: usize, color: u32) -> ui::Numeric {
    num(label, value, -65536.0, 65536.0, precision, unit).style(
        style()
            .radius(3)
            .border_left(3, rgb(color).with_alpha(0.65)),
    )
}
fn animation_property(label: &str, initial: f64, tint: ui::PropertyState) -> ui::PropertyRow {
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
    let field = if tint == ui::PropertyState::Animated {
        ui::value_slider(name, value).range(0.0, 1.0)
    } else {
        ui::number(name, value).range(-65536.0, 65536.0)
    };
    ui::property_row(
        label.to_owned(),
        field
            .property_state(state)
            .label_visible(false)
            .style(style().width(pct(100))),
    )
    .leading({
        let label = label.to_owned();
        move || {
            ui::property_state_button(label, state)
                .on_press(move || keyed.update(|keyed| *keyed = !*keyed))
        }
    })
}
