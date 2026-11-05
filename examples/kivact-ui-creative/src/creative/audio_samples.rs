//! Role: Audio sample management for the creative application.
use crate::creative::audio_values::TransportState;

use super::audio_values::{level, smpte};
use super::common::{bind, col, line, note, spec, specs};
use super::controls::chip;
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::prelude::{
    ColorToken, IntoText, IntoView, Node, NodeBuilder, Orientation, Signal, column, effect, grid,
    node, rgb, row, signal, spacer, text,
};
use kivact::state::{Binding, Machine, Step, machine};
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::style::{GridPlacement, Track};
use kivact_ui_creative as ui;

pub(super) fn transport_section(state: Signal<TransportState>) -> impl IntoView {
    let format = signal(0_usize);
    specs((
        transport_section_transport_panel(state).style(style().min_height(245)),
        transport_section_lcd_time_display(state, format),
        transport_section_locators(state),
    ))
}
fn lcd_note(value: impl IntoText) -> kivact::view::Node {
    note(value).style(
        style()
            .font_size(9.5)
            .font_family("Geist Mono")
            .line_height(1.0),
    )
}
fn locator(tag: &str, label: &str, value: Binding<f64>) -> Node {
    row((
        chip(tag, rgb(if tag == "L" { 0x5aa9ff } else { 0xff5c6c }))
            .style(style().padding_x(5).padding_y(3)),
        ui::number(label.to_owned(), value)
            .timecode(24)
            .label_visible(false)
            .range(0.0..=100000.0)
            .style(style().width(0).grow(1)),
    ))
    .style(style().gap(8))
}

pub(super) fn knobs() -> impl IntoView {
    let range = signal((24.0, 180.0));
    specs((
        knobs_rotary_knob().style(style().grid_column(GridPlacement::span(2)).min_height(240)),
        knobs_channel_fader(),
        knobs_horizontal_range(range),
    ))
}

pub(super) fn meters(state: Signal<TransportState>) -> impl IntoView {
    let integrated = machine(MeasurementState::Unmeasured, || Measurement, |_, ()| {});
    effect(move || {
        if state.get().playing {
            integrated.send(MeasurementEvent::Sample);
        }
    });
    specs((
        meters_peak_meter(state).style(style().min_height(298)),
        meters_horizontal_meter(state),
        meters_loudness(integrated),
    ))
}
fn loudness(name: &str, value: impl IntoText, unit: &str, color: Option<Color>) -> Node {
    col((
        lcd_note(name.to_owned()).style(style().font_size(8)),
        row((
            text(value).style(
                style()
                    .font_family("Geist Mono")
                    .font_size(18)
                    .weight(600)
                    .color(color.unwrap_or(rgb(0xe4e5e9))),
            ),
            lcd_note(unit.to_owned()).style(style().font_size(8)),
        )),
    ))
    .style(style().padding(8).gap(3).min_width(82).radius(10))
    .style(
        style()
            .background(ColorToken::Raised)
            .border(1, ui::ColorName::Hair),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MeasurementState {
    Unmeasured,
    Measured,
}
enum MeasurementEvent {
    Sample,
    Reset,
}
struct Measurement;
impl Machine for Measurement {
    type State = MeasurementState;
    type Event = MeasurementEvent;
    type Effect = ();
    fn transition(
        &self,
        _state: &MeasurementState,
        event: MeasurementEvent,
    ) -> Step<MeasurementState, MeasurementEvent, ()> {
        Step::to(match event {
            MeasurementEvent::Sample => MeasurementState::Measured,
            MeasurementEvent::Reset => MeasurementState::Unmeasured,
        })
    }
}

fn transport_section_transport_panel(state: Signal<TransportState>) -> Node {
    spec(
        "Transport Panel",
        "トランスポート",
        col((
            row((
                kivact::ui::button("To Start")
                    .hide_label(true)
                    .icon(kivact::view::icon("start"))
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || state.update(|s| s.frame = 0.0)),
                kivact::ui::button("◀◀")
                    .name("Rewind")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || state.update(|s| s.frame = (s.frame - 48.0).max(0.0))),
                kivact::ui::button("▶▶")
                    .name("Forward")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || state.update(|s| s.frame += 48.0)),
                kivact::ui::button("▶|")
                    .name("To End")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || state.update(|s| s.frame = s.loop_end)),
                kivact::ui::toggle_button(
                    "",
                    bind(
                        move || state.get().cycling,
                        move |v| state.update(|s| s.cycling = v),
                    ),
                )
                .icon(kivact::view::icon("repeat"))
                .name("Cycle"),
                kivact::ui::button("Stop")
                    .hide_label(true)
                    .icon(kivact::view::icon("stop"))
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || state.update(|s| s.playing = false)),
                kivact::ui::button("Play")
                    .hide_label(true)
                    .icon(kivact::view::icon("play"))
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .with_node(|node| node.selected(move || state.get().playing))
                    .on_press(move || state.update(|s| s.playing = !s.playing)),
                kivact::ui::button("Record")
                    .hide_label(true)
                    .icon(kivact::view::icon("record"))
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .style(style().color(rgb(0xff5c6c)))
                    .with_node(|node| node.selected(move || state.get().recording))
                    .on_press(move || state.update(|s| s.recording = !s.recording)),
            ))
            .style(style().height(34).gap(2).padding(3).radius(10))
            .style(style().background(ui::ColorName::Canvas)),
            line((
                kivact::ui::toggle_button("♬", signal(false)).name("Metronome"),
                kivact::ui::toggle_button("⇥", signal(false)).name("Pre-roll"),
                kivact::ui::toggle_button("↦", signal(false)).name("Punch In"),
                kivact::ui::toggle_button("↔", signal(false)).name("Follow"),
            ))
            .style(style().height(30).width(pct(100)).radius(9))
            .style(style().background(ui::ColorName::Canvas)),
            note(
                "Space 再生／停止、Home 先頭、, . で1フレーム移動。全エディタの再生ヘッドが同期します。",
            ),
        )),
    )
}

fn transport_section_lcd_time_display(
    state: Signal<TransportState>,
    format: Signal<usize>,
) -> Node {
    spec(
        "LCD Time Display",
        "タイム表示",
        col((
            kivact::ui::button("")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .name("Primary Time Display")
                .child(
                    col((
                        row((
                            lcd_note("PRIMARY"),
                            spacer(),
                            lcd_note(move || {
                                ["SMPTE 24", "BARS+BEATS", "SECONDS", "FRAMES"][format.get()]
                                    .to_owned()
                            }),
                        )),
                        text(move || match format.get() {
                            0 => smpte(state.get().frame),
                            1 => format!(
                                "{}.{:.0}.1.0",
                                (state.get().frame / 48.0).floor() + 1.0,
                                (state.get().frame / 12.0).floor() % 4.0 + 1.0
                            ),
                            2 => format!("{:.3} s", state.get().frame / 24.0),
                            _ => format!("{:06}", state.get().frame as u64),
                        })
                        .style(style().font_family("Geist Mono").font_size(30).weight(600)),
                        row((
                            lcd_note(move || {
                                format!(
                                    "BARS {:.0}.{:.0}.1.0",
                                    (state.get().frame / 48.0).floor() + 1.0,
                                    (state.get().frame / 12.0).floor() % 4.0 + 1.0
                                )
                            }),
                            spacer(),
                            lcd_note(move || format!("SEC {:.3}", state.get().frame / 24.0)),
                        )),
                    ))
                    .style(style().gap(3)),
                )
                .style(style().height(88).padding(12).radius(12))
                .style(
                    style()
                        .background(ui::ColorName::Canvas)
                        .border(1, ui::ColorName::Hair),
                )
                .on_press(move || format.update(|v| *v = (*v + 1) % 4)),
            col((
                row((lcd_note("TEMPO"), spacer(), lcd_note("SIGNATURE"))),
                row((
                    text("120.000")
                        .style(style().font_family("Geist Mono").font_size(22).weight(600)),
                    spacer(),
                    text("4/4").style(style().font_family("Geist Mono").font_size(22).weight(600)),
                )),
                row((lcd_note("TRACK TEMPO"), spacer(), lcd_note("SYNC INT"))),
            ))
            .style(style().height(82).padding(12).gap(3).radius(12))
            .style(
                style()
                    .background(ui::ColorName::Canvas)
                    .border(1, ui::ColorName::Hair),
            ),
        ))
        .style(style().gap(10)),
    )
}

fn transport_section_locators(state: Signal<TransportState>) -> Node {
    spec(
        "Locators",
        "ロケーター",
        col((
            locator(
                "L",
                "Start",
                bind(
                    move || state.get().loop_start,
                    move |v| state.update(|s| s.loop_start = v),
                ),
            ),
            locator(
                "R",
                "End",
                bind(
                    move || state.get().loop_end,
                    move |v| state.update(|s| s.loop_end = v),
                ),
            ),
            line((
                kivact::ui::button("Set from Selection")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .size(kivact::ui::ControlSize::Small)
                    .on_press(move || {
                        state.update(|s| {
                            s.loop_start = s.frame;
                            s.loop_end = s.frame + 192.0;
                        })
                    }),
                kivact::ui::button("Reset")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .size(kivact::ui::ControlSize::Small)
                    .on_press(move || {
                        state.update(|s| {
                            s.loop_start = 0.0;
                            s.loop_end = 192.0;
                        })
                    }),
            )),
            grid(
                [
                    ("FPS", "24"),
                    ("PPQ", "480"),
                    ("LATENCY", "5.33 ms"),
                    ("BUFFER", "256 smp"),
                ]
                .into_iter()
                .flat_map(|(name, v)| {
                    [
                        lcd_note(name),
                        text(v).style(
                            style()
                                .font_size(10.5)
                                .weight(600)
                                .font_family("Geist Mono"),
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
            )
            .style(
                style()
                    .grid_template_columns((Track::minmax(50, 50), Track::fr(1)))
                    .gap(5)
                    .margin_top(6),
            ),
        ))
        .style(style().gap(5)),
    )
}

fn knobs_rotary_knob() -> Node {
    spec(
        "Rotary Knob",
        "ノブ",
        col((
            line((
                ui::knob("Gain", signal(0.0))
                    .range(-24.0..=24.0)
                    .precision(1)
                    .unit(" dB")
                    .bipolar(true)
                    .accent_color(rgb(0x9b9ea8)),
                ui::knob("Pan", signal(0.0))
                    .range(-100.0..=100.0)
                    .step(1.0)
                    .precision(0)
                    .bipolar(true)
                    .accent_color(rgb(0x5aa9ff)),
                ui::knob("Freq", signal(1.2))
                    .range(0.02..=20.0)
                    .reset_to(1.0)
                    .logarithmic(true)
                    .precision(2)
                    .unit(" kHz")
                    .accent_color(rgb(0x34d399)),
                ui::knob("Q", signal(0.71))
                    .range(0.1..=18.0)
                    .reset_to(0.71)
                    .logarithmic(true)
                    .precision(2)
                    .accent_color(rgb(0x9b9ea8)),
                ui::knob("Drive", signal(35.0))
                    .range(0.0..=100.0)
                    .precision(0)
                    .unit(" %")
                    .accent_color(rgb(0xff5c6c)),
                ui::knob("Mix", signal(100.0))
                    .range(0.0..=100.0)
                    .reset_to(100.0)
                    .precision(0)
                    .unit(" %"),
            ))
            .style(style().gap(10)),
            line((
                note("EQ BAND").style(style().font_size(10.5).width(52)),
                ui::knob("Lo", signal(80.0))
                    .range(20.0..=2000.0)
                    .logarithmic(true)
                    .precision(0)
                    .unit(" Hz")
                    .accent_color(rgb(0x34d399))
                    .style(style().size(40, 80)),
                ui::knob("G", signal(3.0))
                    .range(-24.0..=24.0)
                    .precision(1)
                    .bipolar(true)
                    .style(style().size(40, 80)),
                ui::knob("Mid", signal(1.5))
                    .range(0.02..=20.0)
                    .logarithmic(true)
                    .precision(2)
                    .unit(" kHz")
                    .accent_color(rgb(0x34d399))
                    .style(style().size(40, 80)),
                ui::knob("G", signal(-2.5))
                    .range(-24.0..=24.0)
                    .precision(1)
                    .bipolar(true)
                    .style(style().size(40, 80)),
                ui::knob("Hi", signal(8.0))
                    .range(0.02..=20.0)
                    .logarithmic(true)
                    .precision(2)
                    .unit(" kHz")
                    .accent_color(rgb(0x34d399))
                    .style(style().size(40, 80)),
                ui::knob("G", signal(1.5))
                    .range(-24.0..=24.0)
                    .precision(1)
                    .bipolar(true)
                    .style(style().size(40, 80)),
            ))
            .style(style().gap(4)),
        ))
        .style(style().gap(2)),
    )
}

fn knobs_channel_fader() -> Node {
    spec(
        "Channel Fader",
        "フェーダー",
        row((
            ui::fader("Volume", signal(0.0)).range(-60.0..=6.0),
            ui::meter("Fader Peak", 0.65).style(style().size(20, 180)),
            col((
                note("VOLUME").style(style().font_size(10)),
                text("0.0 dB").style(style().font_family("Geist Mono").font_size(18)),
                note("dB カーブ（−60〜+6）。Ctrl 微調整。"),
            )),
        ))
        .style(style().align_items(Align::Start).gap(4)),
    )
}

fn knobs_horizontal_range(range: Signal<(f64, f64)>) -> Node {
    spec(
        "Horizontal / Range",
        "横フェーダー・範囲",
        col((
            row((
                lcd_note("A"),
                spacer(),
                lcd_note("CROSSFADE"),
                spacer(),
                lcd_note("B"),
            )),
            ui::fader("Crossfade", signal(0.0))
                .orientation(Orientation::Horizontal)
                .range(-1.0..=1.0)
                .precision(2)
                .style(style().width(pct(100))),
            row((
                lcd_note("IN / OUT"),
                spacer(),
                lcd_note(move || format!("{:.0} – {:.0}", range.get().0, range.get().1)),
            )),
            ui::range("In Out", range),
            lcd_note("SEND LEVEL"),
            ui::value_slider("FX 1 · Reverb", signal(-12.0))
                .range(-60.0..=6.0)
                .precision(1)
                .unit(" dB")
                .style(style().width(pct(100))),
            ui::value_slider("FX 2 · Delay", signal(-24.0))
                .range(-60.0..=6.0)
                .precision(1)
                .unit(" dB")
                .style(style().width(pct(100))),
        ))
        .style(style().gap(5)),
    )
}

fn meters_peak_meter(state: Signal<TransportState>) -> Node {
    spec(
        "Peak Meter",
        "ピーク",
        row((
            ui::meter("Stereo Peak", level(state, 0)).style(style().size(18, 220)),
            column(
                [0, -3, -6, -12, -18, -24, -40, -60]
                    .into_iter()
                    .map(|db| lcd_note(db.to_string()))
                    .collect::<Vec<_>>(),
            )
            .style(style().height(220).justify_content(Justify::SpaceBetween)),
            ui::meter("Bus Peak", level(state, 1))
                .channels(6)
                .style(style().size(52, 200)),
            col((
                lcd_note("GR"),
                ui::meter("Gain Reduction", 0.0)
                    .channels(1)
                    .style(style().size(8, 140)),
                lcd_note("−0.0"),
            ))
            .style(style().gap(8)),
        ))
        .style(style().gap(12).align_items(Align::Start)),
    )
}

fn meters_horizontal_meter(state: Signal<TransportState>) -> Node {
    spec(
        "Horizontal Meter",
        "横型",
        col((
            ui::meter("Stereo Horizontal", level(state, 0))
                .orientation(Orientation::Horizontal)
                .style(style().size(280, 24)),
            lcd_note("−60     −40     −24     −12     −6      0"),
            lcd_note("PHASE CORRELATION").style(style().margin_top(6)),
            node()
                .style(style().height(9).radius(4))
                .style(style().background(ui::ColorName::Field))
                .draw(|d, s| {
                    d.rounded_rect(Rect::new(s.width * 0.74, 0.5, 7.0, 7.0), 3.5, rgb(0x34d399));
                }),
            row((
                lcd_note("−1"),
                spacer(),
                lcd_note("0"),
                spacer(),
                lcd_note("+1"),
            )),
        ))
        .style(style().gap(6)),
    )
}

fn meters_loudness(integrated: kivact::state::MachineHandle<Measurement>) -> Node {
    spec(
        "Loudness",
        "ラウドネス",
        col((
            line((
                loudness("MOMENTARY", "−22.8", "LUFS", None),
                loudness("SHORT-TERM", "−21.1", "LUFS", None),
                loudness(
                    "INTEGRATED",
                    move || {
                        if integrated.state().get() == MeasurementState::Measured {
                            "−20.7".to_owned()
                        } else {
                            "−∞".to_owned()
                        }
                    },
                    "LUFS",
                    Some(rgb(0x34d399)),
                ),
            ))
            .style(style().gap(5)),
            line((
                loudness("TRUE PEAK", "0.2", "dBTP", Some(rgb(0xfbbf24))),
                loudness("RANGE", "0.0", "LU", None),
            ))
            .style(style().gap(5)),
            node()
                .style(style().height(64).radius(10))
                .style(
                    style()
                        .background(ui::ColorName::Canvas)
                        .border(1, ui::ColorName::Hair),
                )
                .draw(|d, s| {
                    for i in 1..4 {
                        d.line(
                            Point::new(0.0, s.height * i as f64 / 4.0),
                            Point::new(s.width, s.height * i as f64 / 4.0),
                            1.0,
                            d.theme().ext::<ui::Tokens>().hair,
                        );
                    }
                    d.line(
                        Point::new(0.0, 20.0),
                        Point::new(s.width, 20.0),
                        1.0,
                        rgb(0x34d399).with_alpha(0.3),
                    );
                }),
            line((
                kivact::ui::button("⟳ Reset")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .size(kivact::ui::ControlSize::Small)
                    .on_press(move || integrated.send(MeasurementEvent::Reset)),
                note("再生中のみ積算").style(style().font_size(10.5)),
            )),
        ))
        .style(style().gap(8)),
    )
}
