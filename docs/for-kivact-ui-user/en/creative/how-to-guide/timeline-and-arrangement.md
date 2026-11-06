# Edit a timeline, dope sheet, graph or arrangement

[日本語](../../../jp/creative/how-to-guide/timeline-and-arrangement.md)

Goal: show your clips, keys or curves in a timeline-style editor, let the user move, trim and delete them, and keep the time (the playhead) under your control.

These editors draw and interpret gestures; they do not store anything. You pass your items in, and the editor reports what the user *asks for*. You decide whether to apply it. This guide assumes you finished the [tutorial](../getting-started/README.md).

## Show clips and apply edit requests

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact_ui_creative as ui;

/// Would a clip at `start..start + duration` on `lane` collide with another clip?
fn collides(clips: &[ui::Clip], id: u64, lane: u64, start: f64, duration: f64) -> bool {
    clips.iter().any(|other| {
        other.id != id
            && other.lane == lane
            && start < other.start + other.duration
            && other.start < start + duration
    })
}

fn apply(clips: &mut Vec<ui::Clip>, request: ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(times) => {
            for time in times {
                let Some(index) = clips.iter().position(|c| c.id == time.id) else {
                    continue;
                };
                let lane = clips[index].lane;
                // The app's rules: no negative start, no overlap in a lane.
                if time.time >= 0.0 && !collides(clips, time.id, lane, time.time, time.duration) {
                    clips[index].start = time.time;
                    clips[index].duration = time.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => clips.retain(|c| !ids.contains(&c.id)),
    }
}

fn timeline_panel() -> impl IntoView {
    let clips = signal(vec![
        ui::Clip::new(1, 10, 0.0, 48.0, "Intro").color(rgb(0x8b7cff)),
        ui::Clip::new(2, 10, 60.0, 72.0, "Verse").color(rgb(0x3ad3e0)),
        ui::Clip::new(3, 20, 24.0, 96.0, "Voice").color(rgb(0xffb224)),
    ]);
    let lanes = signal(vec![ui::Lane::new(10, "Video 1"), ui::Lane::new(20, "Audio 1")]);

    // View state the app owns: the playhead is the app's time.
    let playhead = signal(0.0);
    let state = ui::TimeState {
        playhead: playhead.into_binding(),
        ..ui::TimeState::new()
    };

    // One undo step per gesture.
    let snapshot = signal(None::<Vec<ui::Clip>>);
    let undo = signal(Vec::<Vec<ui::Clip>>::new());
    let on_edit = move |event: ui::EditorEvent<ui::TimeRequest>| match event {
        ui::EditorEvent::Begin => snapshot.set(Some(clips.get_untracked())),
        ui::EditorEvent::Request(request) => clips.update(|all| apply(all, request)),
        ui::EditorEvent::Commit => {
            if let Some(before) = snapshot.update(|saved| saved.take()) {
                undo.update(|steps| steps.push(before));
            }
        }
        ui::EditorEvent::Cancel => {
            if let Some(before) = snapshot.update(|saved| saved.take()) {
                clips.set(before);
            }
        }
    };

    column((
        ui::timeline(clips, state)
            .lanes(lanes)
            .step(1.0)
            .snap(true)
            .time_format(|frame| format!("{:.1}s", frame / 24.0))
            .on_edit(on_edit)
            .name("Timeline"),
        text(move || format!("Frame {:.0}, {} undo step(s)", playhead.get(), undo.get().len())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Timeline")
        .theme(ui::theme(ThemeMode::Dark))
        .run(timeline_panel)
}
```

Drag a clip, trim an edge, click the ruler to move the playhead, select a clip and press `Delete`. Drag a clip over another one: the request arrives, `collides` refuses it, and the clip does not move.

### How the contract works

- **Items are read-only input.** `ui::timeline(clips, state)` takes a signal, a closure or a plain `Vec<Clip>`. The editor never writes to it.
- **`on_edit` receives an `EditorEvent`.** A gesture is `Begin`, any number of `Request(..)`, then `Commit` (the user released) or `Cancel` (the user pressed `Esc`, or the gesture was interrupted). A single action, such as a key press, sends the same sequence with one request.
- **Requests are absolute.** A `Retime` carries the new start and duration of every item involved, computed from the items as they were at `Begin`. You do not have to accumulate deltas, and applying only the latest request is enough. Ignoring a request is allowed; the item simply keeps drawing where your model puts it.
- **Take the snapshot at `Begin`.** On `Cancel`, put the snapshot back. The editor restores the values it holds itself (playhead, zoom, scroll), but your items are yours to restore.
- **`Begin` is sent at the first change, not at the press.** A press that does not move selects the clip, and a double click activates it; neither sends events.
- **IDs are stable keys.** `Clip::id` and `Lane::id` are numbers you assign. They do not change when rows are reordered; they are not indexes.

### Time belongs to the app

The editors never advance time. The playhead is the binding in `TimeState` that *you* provide:

- When the user moves the playhead, the editor writes it. Read it, for example to seek your player.
- When your clock runs, you write it. Drive it from your player while playing:

```rust,no_run
use kivact::prelude::*;
use std::time::Duration;

async fn run_clock(playing: Signal<bool>, playhead: Signal<f64>) {
    loop {
        kivact::task::sleep(Duration::from_millis(1000 / 24)).await;
        if playing.get_untracked() {
            playhead.update(|frame| *frame += 1.0);
        }
    }
}

fn start_clock(playing: Signal<bool>, playhead: Signal<f64>) {
    spawn(run_clock(playing, playhead));
}
```

Call `start_clock` once from a component's setup. The task ends with the component that started it. `TimeState` also holds `selection` and `transform`; replace any field with your own signal to share it with other parts of the app (`TimeState { selection: picked.into(), ..ui::TimeState::new() }`). The time unit is yours: a frame number, a tick or a second. `time_format` only controls how the ruler and the playhead print it.

### Other builder options

| Method | Effect |
|--------|--------|
| `.lanes(..)` | Rows and their labels, in order. A row exists only if listed; an empty list hides every item. Without it, lanes come from the items' `lane` values. |
| `.snap(true)` and `.step(n)` | Round pointer edits to multiples of `n`. Arrow keys move by `n`, `Shift` by `10 n`. |
| `.zoom_range(min, max)` | Allowed zoom (default `0.25..=4`). |
| `.min_duration(n)` | Shortest clip a trim may propose (default `1`). |
| `.ruler_ticks(ui::RulerTicks::new([30.0, 60.0, 150.0], 4))` | Your own major tick intervals and minor marks. |
| `.lane_cell(width, build)` and `.context_menu(entries, choose)` | A control in each lane header, and a right-click menu whose entries are built from an `EditorContext` (item, lane, time). |

## Keyframes: the dope sheet

`ui::dope_sheet(keys, state)` is the same editor over `Keyframe`s. It reports the same `TimeRequest`s, so one `on_edit` shape serves both; only the field names differ (`time` instead of `start`).

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn keys_panel() -> impl IntoView {
    let keys = signal(vec![
        ui::Keyframe::new(1, 1, 0.0, "Start"),
        ui::Keyframe::new(2, 1, 24.0, "Hold").duration(12.0),
        ui::Keyframe::new(3, 2, 12.0, "Pop").glyph(ui::KeyGlyph::Circle),
    ]);
    ui::dope_sheet(keys, ui::TimeState::new()).on_edit(move |event| {
        if let ui::EditorEvent::Request(request) = event {
            keys.update(|all| match request {
                ui::TimeRequest::Retime(times) => {
                    for time in times {
                        if let Some(key) = all.iter_mut().find(|k| k.id == time.id) {
                            key.time = time.time;
                            key.duration = time.duration;
                        }
                    }
                }
                ui::TimeRequest::Delete(ids) => all.retain(|k| !ids.contains(&k.id)),
            });
        }
    })
}
```

A key with a positive `duration` draws as a bar. This version skips snapshots; add them as in the timeline above if you want `Cancel` to restore the keys.

## Curves: the graph editor

`ui::graph_editor(curves, state)` shows curves with the value on the vertical axis. A `CurveKey` has a time, a value, an `Interpolation` (`Constant`, `Linear` or `Bezier`) and relative tangent handles.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn apply(curves: &mut [ui::Curve], request: &ui::GraphRequest) {
    for key in curves.iter_mut().flat_map(|curve| curve.keys.iter_mut()) {
        match request {
            ui::GraphRequest::Move(moves) => {
                if let Some(m) = moves.iter().find(|m| m.id == key.id) {
                    key.time = m.time;
                    key.value = m.value;
                    key.in_tangent = m.in_tangent;
                    key.out_tangent = m.out_tangent;
                }
            }
            ui::GraphRequest::Interpolate(ids, mode) if ids.contains(&key.id) => {
                key.interpolation = *mode;
            }
            _ => {}
        }
    }
    if let ui::GraphRequest::Delete(ids) = request {
        for curve in curves.iter_mut() {
            curve.keys.retain(|key| !ids.contains(&key.id));
        }
    }
}

fn graph_panel() -> impl IntoView {
    let curves = signal(vec![ui::Curve::new(
        1,
        "Opacity",
        vec![ui::CurveKey::new(1, 0.0, 0.0), ui::CurveKey::new(2, 48.0, 1.0)],
    )]);
    ui::graph_editor(curves, ui::GraphState::new())
        .step(1.0, 0.01)
        .on_edit(move |event| {
            if let ui::EditorEvent::Request(request) = event {
                curves.update(|all| apply(all, &request));
            }
        })
}
```

Key IDs must be unique across all curves of the graph. `GraphState::view` holds the visible time and value ranges (`GraphView { time, value }`). Press `F` to fit everything and `Home` to frame the selected keys; `Ctrl` plus the wheel zooms time, and `Ctrl+Shift` plus the wheel zooms value.

## A song-style arrangement

`ui::arrangement(clips, tracks, state)` is a music-sequencer layout: track headers with mute, solo and record arm, clips in beats, a loop range, markers and automation lanes. A clip's `lane` is the id of its track.

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn apply(
    clips: &mut Vec<ui::Clip>,
    tracks: &mut [ui::TimelineTrack],
    request: ui::ArrangementRequest,
) {
    use ui::ArrangementRequest as R;
    match request {
        R::Retime(times) => {
            for time in times {
                if let Some(clip) = clips.iter_mut().find(|c| c.id == time.id) {
                    clip.start = time.time;
                    clip.duration = time.duration;
                }
            }
        }
        R::Delete(ids) => clips.retain(|c| !ids.contains(&c.id)),
        R::Split { clip, at } => {
            // The app owns the new clip's id and name.
            let new_id = clips.iter().map(|c| c.id).max().unwrap_or(0) + 1;
            if let Some(left) = clips.iter_mut().find(|c| c.id == clip) {
                if at > left.start && at < left.start + left.duration {
                    let mut right = left.clone();
                    right.id = new_id;
                    right.start = at;
                    right.duration = left.start + left.duration - at;
                    left.duration = at - left.start;
                    clips.push(right);
                }
            }
        }
        R::TrackFlag { track, flag, value } => {
            if let Some(t) = tracks.iter_mut().find(|t| t.id == track) {
                match flag {
                    ui::TrackFlag::Mute => t.muted = value,
                    ui::TrackFlag::Solo => t.solo = value,
                    ui::TrackFlag::Arm => t.armed = value,
                    ui::TrackFlag::AutomationRead => t.automation_read = value,
                }
            }
        }
        // Erase, ToggleClipMute, AddPoint, MovePoint and DeletePoint are the
        // rest of the requests; this app does not support them yet.
        _ => {}
    }
}

fn arrangement_panel() -> impl IntoView {
    let clips = signal(vec![
        ui::Clip::new(1, 1, 0.0, 8.0, "Drums"),
        ui::Clip::new(2, 2, 4.0, 8.0, "Bass"),
    ]);
    let tracks = signal(vec![
        ui::TimelineTrack::new(1, "Drums", ui::TrackKind::Audio, rgb(0x8b7cff)),
        ui::TimelineTrack::new(2, "Bass", ui::TrackKind::Midi, rgb(0x3ad3e0)),
    ]);
    let state = ui::ArrangementState::new();
    ui::arrangement(clips, tracks, state)
        .length(32.0)
        .markers(vec![ui::TimelineMarker::new(16.0, "Chorus", rgb(0xffb224))])
        .on_edit(move |event| {
            if let ui::EditorEvent::Request(request) = event {
                clips.update(|c| tracks.update(|t| apply(c, t, request)));
            }
        })
}
```

Positions are in beats; the zoom in `ArrangementState` is pixels per beat. `ArrangementState` also holds the loop range, the `cycling` flag and the tool (`TimelineTool::Select`, `Split`, `Mute` or `Erase`). Choosing a tool only changes what a click *requests*: the Split tool sends `Split { clip, at }`, and you decide whether and how to cut. Combine this example with the snapshot pattern of the timeline for undo.

## Playback controls and meters

The kit draws transport buttons, channel strips and meters, but it plays no audio. You feed them state and react to changes.

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn mixer_panel() -> impl IntoView {
    let transport = signal(ui::TransportState::default());
    let drums = ui::ChannelState::new();
    let level = signal(0.4); // the final level your audio engine measures, 0..1
    row((
        ui::transport(transport).on_to_start(move || {
            // Seek your player to the start here.
        }),
        ui::mixer_strip("Drums", rgb(0x8b7cff), drums)
            .gain_range(kivact::interaction::ScalarRange::new(-60.0, 6.0).expect("valid range"))
            .level(level),
        ui::meter("Master", level).orientation(Orientation::Horizontal),
    ))
    .style(style().gap(12).padding(12))
}
```

`TransportState` holds only `playing`, `recording` and `cycling`. `ChannelState` holds `gain`, `pan`, `mute` and `solo` as bindings. The strip shows the `level` you give it unchanged, without applying gain or mute, so apply DSP in your engine and pass the result. Routing choices and insert names are also yours: `.routes(labels, route)` shows a selector only if `labels` is not empty, and `.inserts(names)` lists names.

## See also

- [Show frames your application renders](show-app-rendered-images.md)
- [Build docks, panel groups and outliners](panels-docks-and-outliners.md)
- [Connect nodes in a node graph](node-graph.md)
- Full API: `cargo doc -p kivact-ui-creative --open`
- The catalog's timeline, animation and mixer chapters: `cargo run -p kivact-ui-creative-catalog`

Time and arrangement editors use a single `transform: Binding<kivact::interaction::ViewTransform>`. Its `scale` is horizontal pixels per logical time unit (pixels per beat for arrangement), and its `offset` is pixel scrolling. Lane heights remain unscaled. A setup-time `transform.project(|pose| &pose.scale, |pose| &mut pose.scale)` binding can connect a zoom slider to the same pose. Domain wheel mappings and sidebar anchors remain part of the time editor.
