//! Role: Arrangement and timeline UI for the creative application.
use crate::creative::audio_values::TransportState;
use kivact::state::IntoBinding;

use super::common::{bind, col, keyed_options, line, note, separator};

use kivact::prelude::{Element, IntoView, Node, Signal, rgb, row, signal, spacer};
use kivact::style::style;
use kivact_ui_creative as ui;

pub(super) fn timeline(state: Signal<TransportState>, status: Signal<String>) -> impl IntoView {
    let clips = signal(initial_clips());
    let tracks = signal(initial_tracks());
    tracks.update(|tracks| tracks[4].automation_label = "Volume".into());
    let selected = signal(vec![10u64]);
    let snap = signal(true);
    let transform = signal(kivact::interaction::ViewTransform {
        scale: 26.0,
        ..Default::default()
    });
    let zoom = transform.project(|pose| &pose.scale, |pose| &mut pose.scale);
    let tool = signal(ui::TimelineTool::Select);
    let unit = signal(Some("beat".to_owned()));
    let loop_range = signal((0.0, 16.0));
    col((
        timeline_toolbar(tool, snap, unit, zoom, selected).style(style().height(38).padding_x(10)),
        ui::arrangement(
            clips,
            tracks,
            ui::ArrangementState {
                selection: selected.into_binding(),
                transform: transform.into_binding(),
                tool: tool.into_binding(),
                playhead: bind(
                    move || state.get().frame / 12.0,
                    move |v| state.update(|s| s.frame = v * 12.0),
                ),
                loop_range: bind(
                    move || loop_range.get(),
                    move |v| {
                        loop_range.set(v);
                        state.update(|s| {
                            s.loop_start = v.0 * 12.0;
                            s.loop_end = v.1 * 12.0;
                        });
                    },
                ),
                cycling: bind(
                    move || state.get().cycling,
                    move |v| state.update(|s| s.cycling = v),
                ),
            },
        )
        .length(64.0)
        .on_edit(super::arrangement_model::edits(clips, tracks, |_| {}))
        .snap(snap)
        .step(move || {
            unit.with(|key| match key.as_deref() {
                Some("bar") => 4.0,
                Some("1-8") => 0.5,
                Some("1-16") => 0.25,
                _ => 1.0,
            })
        })
        .markers(move || {
            vec![
                ui::TimelineMarker::new(0.0, "Intro", rgb(0xa9a9ca)),
                ui::TimelineMarker::new(16.0, "Verse", rgb(0xffd25a)),
                ui::TimelineMarker::new(32.0, "Chorus", rgb(0xff5c6c)),
                ui::TimelineMarker::new(48.0, "Bridge", rgb(0x34d399)),
            ]
        })
        .on_add_track(move || {
            status.set("Add Track: Audio / Instrument / Video / Folder (demo)".into())
        })
        .test_id("creative-timeline"),
    ))
    .style(style().gap(0))
}
fn clip(id: u64, label: &str, beat: f64, length: f64, lane: usize, color: u32) -> ui::Clip {
    ui::Clip::new(id, lane as u64, beat, length, label).color(rgb(color))
}

fn initial_clips() -> Vec<ui::Clip> {
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

fn initial_tracks() -> Vec<ui::TimelineTrack> {
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

fn timeline_toolbar(
    tool: Signal<ui::TimelineTool>,
    snap: Signal<bool>,
    unit: Signal<Option<String>>,
    zoom: kivact::state::Binding<f64>,
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
            "Timeline grid",
            keyed_options([
                ("bar", "Bar"),
                ("beat", "Beat"),
                ("1-8", "1/8"),
                ("1-16", "1/16"),
            ]),
            unit,
        )
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
