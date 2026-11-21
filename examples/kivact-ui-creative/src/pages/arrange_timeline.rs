//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::audio_values::TransportState;
use kivact::state::Signal;
use super::common::{bind, col, keyed_options, line, note, separator};
use kivact::prelude::{Element, IntoView, rgb, row, signal, spacer};
use kivact::style::style;
use kivact::view::Node;
use kivact_ui_creative as ui;
use super::arrange::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("arrange-timeline", "Arrange Timeline", "Animation", include_str!("arrange_timeline.rs"))
        .description("Ruler, loop, tracks, clips and automation.")
        .example(Example::new("arrange-timeline", "Arrange Timeline", "", demo))
}

// example: arrange-timeline
fn demo() -> impl IntoView {
    let state = transport();
    let status = signal(String::new());
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
                selection: selected,
                transform,
                tool,
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
// end example
