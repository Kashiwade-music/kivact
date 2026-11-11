//! Role: Preserve selection and gestures across independent editor contracts.
use crate::arrangement_model::arrangement as app_arrangement;
use crate::graph_support::app_graph;
use crate::node_support::app_node_editor;
use crate::time_support::{app_dope_sheet, app_timeline};
use kivact::geometry::{Point, Size};
use kivact::interaction::ViewTransform;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact_ui_creative as ui;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Timeline,
    DopeSheet,
    Graph,
    Nodes,
    Arrangement,
}
const KINDS: [Kind; 4] = [Kind::Timeline, Kind::DopeSheet, Kind::Graph, Kind::Nodes];
#[derive(Clone, Copy)]
struct Model {
    kind: Kind,
    clips: Signal<Vec<ui::Clip>>,
    keys: Signal<Vec<ui::Keyframe>>,
    curves: Signal<Vec<ui::Curve>>,
    nodes: Signal<Vec<ui::NodeData>>,
    selection: Signal<Vec<u64>>,
    frame: Signal<f64>,
    transform: Signal<ViewTransform>,
    view: Signal<ui::GraphView>,
    activated: Signal<Vec<u64>>,
}
#[derive(Clone, Debug, PartialEq)]
struct Item {
    position: Point,
}
impl Model {
    fn snapshot(self) -> Vec<Item> {
        match self.kind {
            Kind::Timeline | Kind::Arrangement => self
                .clips
                .get_untracked()
                .iter()
                .enumerate()
                .map(|(i, c)| Item {
                    position: Point::new(c.start, 60.0 + i as f64 * 60.0),
                })
                .collect(),
            Kind::DopeSheet => self
                .keys
                .get_untracked()
                .iter()
                .enumerate()
                .map(|(i, k)| Item {
                    position: Point::new(k.time, 60.0 + i as f64 * 60.0),
                })
                .collect(),
            Kind::Graph => self
                .curves
                .get_untracked()
                .iter()
                .flat_map(|c| {
                    c.keys.iter().map(|k| Item {
                        position: Point::new(k.time, -k.value),
                    })
                })
                .collect(),
            Kind::Nodes => self
                .nodes
                .get_untracked()
                .iter()
                .map(|n| Item {
                    position: n.position,
                })
                .collect(),
        }
    }
    fn name(self, index: usize) -> String {
        if self.kind == Kind::Graph {
            format!(
                "Item {} key {}",
                index + 1,
                self.curves.get_untracked()[index].keys[0].time
            )
        } else {
            format!("Item {}", index + 1)
        }
    }
}
fn item(screen: &Screen, model: Model, index: usize) -> kivact::test::Locator<'_> {
    screen.get_by_role(Role::Button, &model.name(index))
}
fn fixture(kind: Kind, snap: bool) -> (Screen, Model) {
    kivact::test::render_with_handle(crate::support::options(), move || {
        let model = gesture_model(kind);
        let view = gesture_view(model, snap);
        (model, view)
    })
}
fn drag_by(screen: &Screen, model: Model, index: usize, delta: Point) {
    let item = item(screen, model, index);
    let start = item.rect().expect("item region is laid out").center();
    item.drag(start, Point::new(start.x + delta.x, start.y + delta.y));
}

#[test]
fn item_drag_writes_consumer_positions_with_and_without_snap() {
    for kind in KINDS {
        for snap in [false, true] {
            let (screen, model) = fixture(kind, snap);
            let before = model.snapshot();
            drag_by(&screen, model, 0, Point::new(17.0, 7.0));
            let after = model.snapshot();
            assert_eq!(model.selection.get_untracked(), [1], "{kind:?}");
            assert_eq!(
                after[0].position.x,
                if snap && !matches!(kind, Kind::Timeline | Kind::DopeSheet) {
                    60.0
                } else {
                    57.0
                },
                "{kind:?}"
            );
            // The node editor snaps both axes; the time editors snap time only.
            let dy = match kind {
                Kind::Timeline | Kind::DopeSheet => 0.0,
                Kind::Nodes if snap => 10.0,
                _ => 7.0,
            };
            assert_eq!(after[0].position.y, before[0].position.y + dy, "{kind:?}");
            assert_eq!(&after[1..], &before[1..], "{kind:?}: other items stay put");
            assert!(screen.errors().is_empty(), "{:?}", screen.errors());
        }
    }
}

#[test]
fn marquee_selects_intersecting_items_and_escape_clears_selection() {
    for kind in KINDS {
        let (screen, model) = fixture(kind, false);
        let first = item(&screen, model, 0).rect().expect("first item");
        let last = item(&screen, model, 2).rect().expect("last item");
        let canvas = screen.get_by_role(Role::Canvas, "Gesture editor");
        canvas.drag(
            Point::new(first.x - 2.0, first.y - 2.0),
            Point::new(last.x + last.width + 2.0, last.y + last.height + 2.0),
        );
        assert_eq!(model.selection.get_untracked(), [1, 2, 3], "{kind:?}");
        canvas.press_key(Key::Escape);
        assert!(model.selection.get_untracked().is_empty(), "{kind:?}");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn playhead_arrow_keys_edit_the_time_without_moving_selected_items() {
    for kind in KINDS.into_iter().filter(|kind| *kind != Kind::Nodes) {
        let (screen, model) = fixture(kind, false);
        model.selection.set(vec![1]);
        screen.run_until_idle();
        let before = model.snapshot();
        let marker = screen.get_by_role(Role::Slider, "Playhead");
        marker.press_key(Key::ArrowRight);
        // The graph fixture sets an explicit logical-time step (`.step(10.0, 0.01)`).
        assert_eq!(
            model.frame.get_untracked(),
            if kind == Kind::Graph { 10.0 } else { 1.0 },
            "{kind:?}"
        );
        assert_eq!(model.snapshot(), before, "{kind:?}");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn timeline_trimming_updates_consumer_clip_bounds() {
    let (screen, data) = kivact::test::render_with_handle(crate::support::options(), || {
        let data = signal(vec![
            ui::Clip::new(1, 0, 30.0, 100.0, "Clip").color(rgb(0x8b7cff)),
        ]);
        (
            data,
            app_timeline(
                data,
                ui::TimeState {
                    selection: (signal(vec![1])).into_binding(),
                    ..ui::TimeState::new()
                },
            )
            .test_id("trim"),
        )
    });
    screen
        .get_by_role(Role::Button, "Clip end trim")
        .drag(Point::new(329.0, 40.0), Point::new(359.0, 40.0));
    assert_eq!(data.get_untracked()[0].duration, 130.0);
    screen
        .get_by_role(Role::Button, "Clip start trim")
        .drag(Point::new(231.0, 40.0), Point::new(251.0, 40.0));
    assert_eq!(data.get_untracked()[0].start, 50.0);
    assert_eq!(data.get_untracked()[0].duration, 110.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn gesture_model(kind: Kind) -> Model {
    let clips = gesture_clips();
    let keys = gesture_keys();
    let curves = gesture_curves();
    let nodes = gesture_nodes();
    if kind == Kind::Arrangement {
        clips.update(|items| {
            for (i, clip) in items.iter_mut().enumerate() {
                clip.start = 2.0 + i as f64 * 6.0;
                clip.duration = 3.0;
            }
        });
    }
    Model {
        kind,
        activated: signal(Vec::new()),
        clips,
        keys,
        curves,
        nodes,
        selection: signal(Vec::new()),
        frame: signal(0.0),
        transform: signal(ViewTransform {
            scale: if kind == Kind::Arrangement { 20.0 } else { 1.0 },
            ..ViewTransform::default()
        }),
        view: signal(ui::GraphView {
            time: (0.0, 440.0),
            value: (-258.0, 0.0),
        }),
    }
}
fn gesture_view(model: Model, snap: bool) -> View {
    let Model {
        kind,
        clips,
        keys,
        curves,
        nodes,
        ..
    } = model;
    let lanes = vec![
        ui::Lane::new(0, "First"),
        ui::Lane::new(1, "Second"),
        ui::Lane::new(2, "Third"),
    ];
    match kind {
        Kind::Arrangement => arrangement_view(model, snap),
        Kind::Timeline => app_timeline(
            clips,
            ui::TimeState {
                selection: model.selection.into_binding(),
                transform: model.transform.into_binding(),
                playhead: model.frame.into_binding(),
            },
        )
        .on_activate(move |id| model.activated.update(|ids| ids.push(id)))
        .snap(snap)
        .lanes(lanes)
        .name("Gesture editor")
        .style(style().size(640, 280))
        .into_view(),
        Kind::DopeSheet => app_dope_sheet(
            keys,
            ui::TimeState {
                selection: model.selection.into_binding(),
                transform: model.transform.into_binding(),
                playhead: model.frame.into_binding(),
            },
        )
        .on_activate(move |id| model.activated.update(|ids| ids.push(id)))
        .snap(snap)
        .lanes(lanes)
        .name("Gesture editor")
        .style(style().size(640, 280))
        .into_view(),
        Kind::Graph => app_graph(
            curves,
            ui::GraphState {
                selection: model.selection.into_binding(),
                view: model.view.into_binding(),
                playhead: model.frame.into_binding(),
            },
        )
        .on_activate(move |id| model.activated.update(|ids| ids.push(id)))
        .snap(snap)
        .step(10.0, 0.01)
        .time_format(|time| format!("{time}"))
        .name("Gesture editor")
        .style(style().size(640, 280))
        .into_view(),
        Kind::Nodes => app_node_editor(
            nodes,
            signal(Vec::new()),
            ui::NodeState {
                selection: model.selection.into_binding(),
                transform: model.transform.into_binding(),
                ..ui::NodeState::new()
            },
        )
        .snap(snap)
        .name("Gesture editor")
        .style(style().size(640, 280))
        .into_view(),
    }
}
fn arrangement_view(model: Model, snap: bool) -> View {
    let clips = model.clips;
    app_arrangement(
        clips,
        signal(
            (0..3)
                .map(|id| {
                    ui::TimelineTrack::new(
                        id,
                        format!("Track {id}"),
                        ui::TrackKind::Audio,
                        kivact::style::Color::from_rgba_f64(0.4, 0.5, 0.8, 1.0),
                    )
                })
                .collect::<Vec<_>>(),
        ),
        ui::ArrangementState {
            selection: model.selection.into_binding(),
            transform: model.transform.into_binding(),
            ..ui::ArrangementState::new()
        },
    )
    .snap(snap)
    .name("Gesture editor")
    .style(style().size(640, 280))
    .into_view()
}
fn gesture_clips() -> Signal<Vec<ui::Clip>> {
    signal(
        (0..3)
            .map(|i| {
                ui::Clip::new(
                    i + 1,
                    i,
                    40.0 + i as f64 * 100.0,
                    70.0,
                    format!("Item {}", i + 1),
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn gesture_keys() -> Signal<Vec<ui::Keyframe>> {
    signal(
        (0..3)
            .map(|i| {
                ui::Keyframe::new(i + 1, i, 40.0 + i as f64 * 100.0, format!("Item {}", i + 1))
            })
            .collect::<Vec<_>>(),
    )
}

fn gesture_curves() -> Signal<Vec<ui::Curve>> {
    signal(
        (0..3)
            .map(|i| {
                ui::Curve::new(
                    i + 1,
                    format!("Item {}", i + 1),
                    vec![ui::CurveKey::new(
                        i + 1,
                        40.0 + i as f64 * 100.0,
                        -60.0 - i as f64 * 60.0,
                    )],
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn gesture_nodes() -> Signal<Vec<ui::NodeData>> {
    signal(
        (0..3)
            .map(|i| {
                ui::NodeData::new(i + 1, format!("Item {}", i + 1), vec![], vec![])
                    .position(Point::new(40.0 + i as f64 * 100.0, 60.0 + i as f64 * 60.0))
                    .size(Size::new(80.0, 40.0))
            })
            .collect::<Vec<_>>(),
    )
}
