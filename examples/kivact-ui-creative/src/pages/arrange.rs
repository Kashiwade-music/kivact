//! Arrangement and timeline UI for the creative application.
use super::audio_values::TransportState;
use kivact::state::Signal;

use super::common::{bind, col, keyed_options, line, note, separator};

use kivact::prelude::{Element, IntoView, rgb, row, signal, spacer};
use kivact::style::style;
use kivact::view::Node;
use kivact_ui_creative as ui;

pub(super) fn clip(
    id: u64,
    label: &str,
    beat: f64,
    length: f64,
    lane: usize,
    color: u32,
) -> ui::Clip {
    ui::Clip::new(id, lane as u64, beat, length, label).color(rgb(color))
}

pub(super) fn initial_clips() -> Vec<ui::Clip> {
    vec![
        clip(1, "Plate_A_v03", 0.0, 16.0, 0, 0x7c8cff),
        clip(2, "Plate_B_v01", 16.0, 12.0, 0, 0x7c8cff),
        clip(3, "Plate_C_v02", 28.0, 22.0, 0, 0x7c8cff),
        clip(4, "Kick_01", 0.0, 32.0, 1, 0xff5c6c),
        clip(5, "Kick_02", 32.0, 32.0, 1, 0xff5c6c),
        clip(6, "Bass Line A", 8.0, 24.0, 2, 0xffd25a),
        clip(7, "Bass Line B", 36.0, 28.0, 2, 0xffd25a),
        clip(8, "Chords", 16.0, 16.0, 3, 0xa397ff),
        clip(9, "Chords 2", 40.0, 16.0, 3, 0xa397ff),
        clip(10, "Vox_Take3", 20.0, 14.0, 4, 0x34d399),
        clip(11, "Vox_Take5", 40.0, 18.0, 4, 0x34d399),
        clip(12, "Reverb Tail", 24.0, 40.0, 5, 0x3ad3e0),
    ]
}

pub(super) fn initial_tracks() -> Vec<ui::TimelineTrack> {
    vec![
        ui::TimelineTrack::new(0, "V1 · Plate", ui::TrackKind::Video, rgb(0x7c8cff)),
        ui::TimelineTrack::new(1, "Kick", ui::TrackKind::Audio, rgb(0xff5c6c)),
        ui::TimelineTrack::new(2, "Bass", ui::TrackKind::Midi, rgb(0xffd25a)),
        ui::TimelineTrack::new(3, "Keys", ui::TrackKind::Midi, rgb(0xa397ff)),
        ui::TimelineTrack::new(4, "Vox", ui::TrackKind::Audio, rgb(0x34d399)).automation(
            [
                (0.0, 0.7),
                (18.0, 0.7),
                (20.0, 0.92),
                (33.0, 0.86),
                (37.0, 0.5),
                (41.0, 0.82),
                (64.0, 0.82),
            ]
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| ui::AutomationPoint::new(i as u64 + 1, x, y))
            .collect(),
        ),
        ui::TimelineTrack::new(5, "FX Return", ui::TrackKind::Audio, rgb(0x3ad3e0)),
    ]
}

pub(super) fn timeline_toolbar(
    tool: Signal<ui::TimelineTool>,
    snap: Signal<bool>,
    unit: Signal<Option<String>>,
    zoom: kivact::state::Signal<f64>,
    selected: Signal<Vec<u64>>,
) -> Node {
    line((
        row([
            ("Select", ui::TimelineTool::Select, "pointer"),
            ("Split", ui::TimelineTool::Split, "scissors"),
            ("Mute", ui::TimelineTool::Mute, "wave"),
            ("Erase", ui::TimelineTool::Erase, "eraser"),
        ]
        .into_iter()
        .map(|(label, kind, icon)| {
            kivact::ui::button(format!("Timeline {label}"))
                .variant(kivact::ui::ButtonVariant::Secondary)
                .hide_label(true)
                .icon(kivact::view::icon(icon))
                .with_node(|node| node.selected(move || tool.get() == kind))
                .on_press(move || tool.set(kind))
        })
        .collect::<Vec<_>>())
        .style(style().gap(2)),
        separator(),
        kivact::ui::toggle_button("Snap", snap).icon(kivact::view::icon("magnet")),
        kivact::ui::select(
            keyed_options([
                ("bar", "Bar"),
                ("beat", "Beat"),
                ("1-8", "1/8"),
                ("1-16", "1/16"),
            ]),
            unit,
        )
        .label("Timeline grid")
        .placeholder("")
        .name("Timeline grid")
        .style(style().width(70)),
        separator(),
        note("Zoom"),
        ui::value_slider("Timeline zoom", zoom)
            .range(8.0..=80.0)
            .step(1.0)
            .precision(0)
            .unit(" px/beat")
            .style(style().width(120).height(18)),
        spacer(),
        note(move || {
            format!(
                "{} selected · grid Beat · 16 bars · 120 BPM",
                selected.with(Vec::len)
            )
        }),
    ))
}
