//! Role: Preserve selection and gestures across independent editor contracts.
mod edges;
use super::*;
use kivact::interaction::ViewTransform;
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
    pan: Signal<Point>,
    zoom: Signal<f64>,
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
    fn time_at(self, x: f32) -> f64 {
        match self.kind {
            Kind::Graph => {
                let v = self.view.get_untracked();
                v.time.0 + f64::from(x - 200.0) / 440.0 * (v.time.1 - v.time.0)
            }
            Kind::Nodes => f64::from(
                self.transform
                    .get_untracked()
                    .to_content(Point::new(x, 0.0))
                    .x,
            ),
            Kind::Arrangement => {
                f64::from(x - 220.0 - self.pan.get_untracked().x)
                    / self.zoom.get_untracked().clamp(8.0, 80.0)
            }
            _ => f64::from(x - 200.0 - self.pan.get_untracked().x) / self.zoom.get_untracked(),
        }
    }
    fn scale(self) -> f64 {
        match self.kind {
            Kind::Graph => {
                let v = self.view.get_untracked();
                440.0 / (v.time.1 - v.time.0)
            }
            Kind::Nodes => self.transform.get_untracked().scale,
            _ => self.zoom.get_untracked(),
        }
    }

    fn snapshot(self) -> Vec<Item> {
        match self.kind {
            Kind::Timeline | Kind::Arrangement => self
                .clips
                .get_untracked()
                .iter()
                .enumerate()
                .map(|(i, c)| Item {
                    position: Point::new(c.start as f32, 60.0 + i as f32 * 60.0),
                })
                .collect(),
            Kind::DopeSheet => self
                .keys
                .get_untracked()
                .iter()
                .enumerate()
                .map(|(i, k)| Item {
                    position: Point::new(k.time as f32, 60.0 + i as f32 * 60.0),
                })
                .collect(),
            Kind::Graph => self
                .curves
                .get_untracked()
                .iter()
                .flat_map(|c| {
                    c.keys.iter().map(|k| Item {
                        position: Point::new(k.time as f32, -k.value as f32),
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
fn item(screen: &Screen, model: Model, index: usize) -> kivact::test::Element<'_> {
    screen.get_by_role(Role::Button, &model.name(index))
}
fn fixture(kind: Kind, snap: bool) -> (Screen, Model) {
    kivact::test::render_with_handle(crate::options(), move || {
        let clips = signal(
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
        );
        let keys = signal(
            (0..3)
                .map(|i| {
                    ui::Keyframe::new(i + 1, i, 40.0 + i as f64 * 100.0, format!("Item {}", i + 1))
                })
                .collect::<Vec<_>>(),
        );
        let curves = signal(
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
        );
        let nodes = signal(
            (0..3)
                .map(|i| {
                    ui::NodeData::new(i + 1, format!("Item {}", i + 1), vec![], vec![])
                        .position(Point::new(40.0 + i as f32 * 100.0, 60.0 + i as f32 * 60.0))
                        .size(Size::new(80.0, 40.0))
                })
                .collect::<Vec<_>>(),
        );
        if kind == Kind::Arrangement {
            clips.update(|items| {
                for (i, clip) in items.iter_mut().enumerate() {
                    clip.start = 2.0 + i as f64 * 6.0;
                    clip.duration = 3.0;
                }
            });
        }
        let model = Model {
            kind,
            activated: signal(Vec::new()),
            clips,
            keys,
            curves,
            nodes,
            selection: signal(Vec::new()),
            pan: signal(Point::new(0.0, 0.0)),
            zoom: signal(if kind == Kind::Arrangement { 20.0 } else { 1.0 }),
            frame: signal(0.0),
            transform: signal(ViewTransform::default()),
            view: signal(ui::GraphView {
                time: (0.0, 440.0),
                value: (-258.0, 0.0),
            }),
        };
        let lanes = vec![
            ui::Lane::new(0, "First"),
            ui::Lane::new(1, "Second"),
            ui::Lane::new(2, "Third"),
        ];
        let view = match kind {
            Kind::Arrangement => app_arrangement(
                clips,
                signal(
                    (0..3)
                        .map(|id| {
                            ui::TimelineTrack::new(
                                id,
                                format!("Track {id}"),
                                ui::TrackKind::Audio,
                                kivact::style::Color::from_rgba_f32(0.4, 0.5, 0.8, 1.0),
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
                ui::ArrangementState {
                    selection: model.selection.into(),
                    pan: model.pan.into(),
                    zoom: model.zoom.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .snap(snap)
            .name("Gesture editor")
            .style(style().size(640, 280))
            .into_view(),
            Kind::Timeline => app_timeline(
                clips,
                ui::TimeState {
                    selection: model.selection.into(),
                    pan: model.pan.into(),
                    zoom: model.zoom.into(),
                    playhead: model.frame.into(),
                },
            )
            .on_item_activate(move |id| model.activated.update(|ids| ids.push(id)))
            .snap(snap)
            .lanes(lanes)
            .name("Gesture editor")
            .style(style().size(640, 280))
            .into_view(),
            Kind::DopeSheet => app_dope_sheet(
                keys,
                ui::TimeState {
                    selection: model.selection.into(),
                    pan: model.pan.into(),
                    zoom: model.zoom.into(),
                    playhead: model.frame.into(),
                },
            )
            .on_item_activate(move |id| model.activated.update(|ids| ids.push(id)))
            .snap(snap)
            .lanes(lanes)
            .name("Gesture editor")
            .style(style().size(640, 280))
            .into_view(),
            Kind::Graph => app_graph(
                curves,
                ui::GraphState {
                    selection: model.selection.into(),
                    view: model.view.into(),
                    playhead: model.frame.into(),
                },
            )
            .on_item_activate(move |id| model.activated.update(|ids| ids.push(id)))
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
                    selection: model.selection.into(),
                    transform: model.transform.into(),
                    ..ui::NodeState::new()
                },
            )
            .snap(snap)
            .name("Gesture editor")
            .style(style().size(640, 280))
            .into_view(),
        };
        (model, view)
    })
}
fn shift(screen: &Screen, down: bool) {
    let modifiers = Modifiers {
        shift: down,
        ..Modifiers::default()
    };
    if down {
        screen.key_down(Key::Unidentified, KeyCode::ShiftLeft, modifiers);
    } else {
        screen.key_up(Key::Unidentified, KeyCode::ShiftLeft, modifiers);
    }
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
            let dy = if matches!(kind, Kind::Timeline | Kind::DopeSheet) {
                0.0
            } else {
                7.0
            };
            assert_eq!(after[0].position.y, before[0].position.y + dy, "{kind:?}");
            assert_eq!(&after[1..], &before[1..], "{kind:?}: other items stay put");
            clean(&screen);
        }
    }
}

#[test]
fn shift_adds_items_and_drag_moves_the_complete_selection() {
    // Timeline's selected drag and click-collapse rule is already fixed in ruler.rs.
    for kind in KINDS {
        let (screen, model) = fixture(kind, false);
        item(&screen, model, 0).click();
        shift(&screen, true);
        item(&screen, model, 1).click();
        item(&screen, model, 2).click();
        shift(&screen, false);
        assert_eq!(model.selection.get_untracked(), [1, 2, 3], "{kind:?}");
        if kind != Kind::Timeline {
            let before = model.snapshot();
            drag_by(&screen, model, 1, Point::new(20.0, 0.0));
            assert_eq!(model.selection.get_untracked(), [1, 2, 3], "{kind:?}");
            for (old, new) in before.iter().zip(model.snapshot()) {
                assert_eq!(
                    new.position,
                    Point::new(old.position.x + 20.0, old.position.y),
                    "{kind:?}"
                );
            }
            item(&screen, model, 1).click();
            assert_eq!(model.selection.get_untracked(), [2], "{kind:?}");
        }
        clean(&screen);
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
        clean(&screen);
    }
}

#[test]
fn middle_drag_pans_ctrl_wheel_zooms_and_home_resets_the_view() {
    for kind in KINDS {
        let (screen, model) = fixture(kind, false);
        let canvas = screen.get_by_role(Role::Canvas, "Gesture editor");
        let start = canvas.rect().expect("editor").center();
        screen.mouse_move(start.x, start.y);
        screen.mouse_down(MouseButton::Middle);
        screen.mouse_move(start.x + 23.0, start.y + 11.0);
        screen.mouse_up(MouseButton::Middle);
        // Node uses the core zoom_pan contract; Graph stores both axes in GraphView.
        match kind {
            Kind::Nodes => assert_eq!(
                model.transform.get_untracked().offset,
                Point::new(23.0, 11.0)
            ),
            Kind::Graph => assert_ne!(model.view.get_untracked().time, (0.0, 440.0)),
            _ => assert_eq!(model.pan.get_untracked(), Point::new(23.0, 11.0)),
        }
        screen.key_down(
            Key::Unidentified,
            KeyCode::ControlLeft,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
        canvas.scroll(0.0, -40.0);
        screen.key_up(
            Key::Unidentified,
            KeyCode::ControlLeft,
            Modifiers::default(),
        );
        match kind {
            Kind::Nodes => {
                assert!((model.transform.get_untracked().scale - 0.04_f64.exp()).abs() < 1e-6);
                canvas.press_key(Key::Char('0'));
                let mut fit = ViewTransform::fit(Size::new(280.0, 160.0), Size::new(640.0, 280.0));
                fit.offset.x -= 40.0 * fit.scale as f32;
                fit.offset.y -= 60.0 * fit.scale as f32;
                assert_eq!(model.transform.get_untracked(), fit);
            }
            Kind::Graph => {
                let view = model.view.get_untracked();
                assert!(view.time.1 - view.time.0 < 440.0);
                canvas.press_key(Key::Char('f'));
                let view = model.view.get_untracked();
                assert!(view.time.0 <= 40.0 && view.time.1 >= 240.0);
            }
            _ => {
                assert!((model.zoom.get_untracked() - 1.1).abs() < 1e-9);
                canvas.press_key(Key::Home);
                assert_eq!(model.pan.get_untracked(), Point::new(0.0, 0.0));
                assert_eq!(model.zoom.get_untracked(), 1.0);
            }
        }
        assert!(model.selection.get_untracked().is_empty());
        clean(&screen);
    }
}

#[test]
fn playhead_pointer_drag_and_accessibility_write_the_time_binding() {
    for kind in KINDS.into_iter().filter(|kind| *kind != Kind::Nodes) {
        let (screen, model) = fixture(kind, false);
        let marker = screen.get_by_role(Role::Slider, "Playhead");
        let ruler = marker.rect().expect("playhead ruler");
        let start = Point::new(ruler.x + 5.0, ruler.center().y);
        marker.drag(start, Point::new(ruler.x + 35.0, start.y));
        assert!((model.frame.get_untracked() - 35.0).abs() < 1.0, "{kind:?}");
        if kind != Kind::Timeline {
            marker.a11y_with_value(A11yAction::SetValue, "42");
            assert_eq!(model.frame.get_untracked(), 42.0, "{kind:?}");
        }
        clean(&screen);
    }
}

#[test]
fn color_surfaces_keep_the_consumers_accessible_name_and_canvas_role() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::color_wheel(signal(rgb(0x8b7cff)), signal(ui::ColorModel::Rgb))
                .name("Color surface")
                .style(style().width(300)),
        )
    });
    assert!(
        screen
            .get_by_role(Role::Group, "Color surface")
            .is_visible()
    );
    assert!(screen.get_by_role(Role::Canvas, "Color wheel").is_visible());
    screen.screenshot();
    clean(&screen);
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
        // Graph uses the explicit logical-time step configured by the D4 fixture.
        assert_eq!(
            model.frame.get_untracked(),
            if kind == Kind::Graph { 10.0 } else { 1.0 },
            "{kind:?}"
        );
        assert_eq!(model.snapshot(), before, "{kind:?}");
        clean(&screen);
    }
}

#[test]
fn escape_during_item_drag_restores_the_values_before_the_edit() {
    for kind in KINDS {
        let (screen, model) = fixture(kind, false);
        let before = model.snapshot();
        let item = item(&screen, model, 0);
        let start = item.rect().expect("item").center();
        screen.mouse_move(start.x, start.y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(start.x + 20.0, start.y);
        assert_ne!(model.snapshot(), before, "{kind:?}: edit began");
        screen.press_key(Key::Escape);
        assert_eq!(model.snapshot(), before, "{kind:?}: cancel restores values");
        screen.mouse_up(MouseButton::Primary);
        clean(&screen);
    }
}

#[test]
fn surface_pixels_overlay_regions_and_pointer_share_the_image_transform() {
    use kivact::draw::{Pixels, Rect};
    use kivact::interaction::ViewTransform;
    let (screen, (transform, events)) = kivact::test::render_with_handle(crate::options(), || {
        let image = Pixels::new(10, 10, std::sync::Arc::from([0, 0, 255, 255].repeat(100)));
        let transform = signal(ViewTransform {
            offset: Point::new(20.0, 10.0),
            scale: 2.0,
        });
        let events = signal(Vec::<ui::SurfacePointer>::new());
        (
            (transform, events),
            ui::viewport_surface(Some(image))
                .transform(transform)
                .pixelated(true)
                .on_pointer(move |event| events.update(|events| events.push(event)))
                .overlay(|draw, _| draw.fill_rect(Rect::new(1.0, 1.0, 2.0, 2.0), rgb(0xff0000)))
                .regions(vec![ui::SurfaceRegion::new(
                    u64::MAX,
                    "Image target",
                    Role::Button,
                    Rect::new(1.0, 1.0, 3.0, 3.0),
                )])
                .name("Application image")
                .style(style().size(200, 100)),
        )
    });
    let target = screen.get_by_role(Role::Button, "Image target");
    let bounds = target.rect().expect("transformed image region");
    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (22.0, 12.0, 6.0, 6.0)
    );
    assert_eq!(screen.screenshot().pixel(24, 14), Some(0xff0000));
    assert_eq!(screen.screenshot().pixel(30, 20), Some(0x0000ff));
    screen.mouse_move(24.0, 14.0);
    let event = events
        .get_untracked()
        .last()
        .copied()
        .expect("image pointer event");
    assert_eq!(event.position, Point::new(2.0, 2.0));
    assert_eq!(event.region, Some(u64::MAX));
    transform.set(ViewTransform {
        offset: Point::new(30.0, 20.0),
        scale: 3.0,
    });
    screen.run_until_idle();
    assert_eq!(target.rect().expect("updated region").x, 33.0);
    clean(&screen);
}

#[test]
fn surface_fits_pixels_and_reports_deduplicated_logical_and_physical_size() {
    use kivact::draw::Pixels;
    let (screen, (image, sizes)) = kivact::test::render_with_handle(crate::options(), || {
        let image = signal(Some(Pixels::new(
            10,
            10,
            std::sync::Arc::from([0, 0, 255, 255].repeat(100)),
        )));
        let sizes = signal(Vec::<ui::SurfaceSize>::new());
        (
            (image, sizes),
            ui::viewport_surface(image)
                .on_resize(move |size| sizes.update(|sizes| sizes.push(size)))
                .name("Fit image")
                .style(style().size(200, 100)),
        )
    });
    assert_eq!(screen.screenshot().pixel(55, 5), Some(0x0000ff));
    assert_ne!(screen.screenshot().pixel(5, 5), Some(0x0000ff));
    assert_eq!(
        sizes.get_untracked().last().expect("first size").logical,
        Size::new(200.0, 100.0)
    );
    let calls = sizes.get_untracked().len();
    image.set(None);
    screen.run_until_idle();
    assert_eq!(sizes.get_untracked().len(), calls);
    assert_ne!(screen.screenshot().pixel(55, 5), Some(0x0000ff));
    screen.set_scale(2.0);
    let last = sizes.get_untracked().last().copied().expect("scaled size");
    assert_eq!(
        (last.physical_width, last.physical_height, last.scale_factor),
        (400, 200, 2.0)
    );
    clean(&screen);
}

#[test]
fn surface_pixelated_sampling_only_changes_enlarged_images() {
    use kivact::draw::Pixels;
    let (screen, pixelated) = kivact::test::render_with_handle(crate::options(), || {
        let pixelated = signal(false);
        let pixels = Pixels::new(
            2,
            1,
            std::sync::Arc::from([0, 0, 0, 255, 255, 255, 255, 255]),
        );
        (
            pixelated,
            ui::viewport_surface(Some(pixels))
                .transform(signal(ViewTransform {
                    offset: Point::new(10.0, 10.0),
                    scale: 10.0,
                }))
                .pixelated(pixelated)
                .style(style().size(100, 80)),
        )
    });
    let smooth = screen.screenshot().pixel(19, 15).expect("boundary pixel");
    pixelated.set(true);
    screen.run_until_idle();
    let nearest = screen.screenshot().pixel(19, 15).expect("nearest pixel");
    assert_ne!(smooth, nearest);
    assert_eq!(nearest, 0x000000);
    clean(&screen);
}
#[test]
fn surface_removed_regions_retire_locators_and_role_changes_replace_semantics() {
    use kivact::draw::Rect;
    let (screen, regions) = kivact::test::render_with_handle(crate::options(), || {
        let regions = signal(vec![ui::SurfaceRegion::new(
            u64::MAX,
            "Image region",
            Role::Button,
            Rect::new(10.0, 10.0, 40.0, 20.0),
        )]);
        (
            regions,
            ui::viewport_surface(None)
                .regions(regions)
                .style(style().size(200, 100)),
        )
    });
    let old = screen.get_by_role(Role::Button, "Image region");
    regions.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(old.name(), "");
    regions.set(vec![ui::SurfaceRegion::new(
        u64::MAX,
        "Image region",
        Role::Image,
        Rect::new(10.0, 10.0, 40.0, 20.0),
    )]);
    screen.run_until_idle();
    assert_eq!(old.name(), "");
    assert!(screen.get_by_role(Role::Image, "Image region").is_visible());
    clean(&screen);
}

#[test]
fn editor_operation_rules_are_consistent() {
    #[derive(Clone, Copy, Debug)]
    enum Operation {
        MetaAll,
        Escape,
        Marquee,
        ShiftMarquee,
        CtrlMarquee,
        MetaMarquee,
        SnapPreserves,
        CtrlWheel,
        MetaWheel,
        DoubleClick,
        DragBetween,
        CancelEdit,
        RejectedContact,
        NarrowOnRelease,
    }
    let kinds = [
        Kind::Timeline,
        Kind::DopeSheet,
        Kind::Graph,
        Kind::Nodes,
        Kind::Arrangement,
    ];
    let operations = [
        Operation::MetaAll,
        Operation::Escape,
        Operation::Marquee,
        Operation::ShiftMarquee,
        Operation::CtrlMarquee,
        Operation::MetaMarquee,
        Operation::SnapPreserves,
        Operation::CtrlWheel,
        Operation::MetaWheel,
        Operation::DoubleClick,
        Operation::DragBetween,
        Operation::CancelEdit,
        Operation::RejectedContact,
        Operation::NarrowOnRelease,
    ];
    let mut failures = Vec::new();
    for kind in kinds {
        for operation in operations {
            // The arrangement has no activation API, so there is nothing to check.
            if kind == Kind::Arrangement && matches!(operation, Operation::DoubleClick) {
                continue;
            }
            let (screen, model) = fixture(kind, matches!(operation, Operation::SnapPreserves));
            model.selection.set(vec![1]);
            screen.run_until_idle();
            let mut before = model.snapshot();
            let canvas = screen.get_by_role(Role::Canvas, "Gesture editor");
            canvas.focus();
            match operation {
                Operation::MetaAll => screen.key_down(
                    Key::Char('a'),
                    KeyCode::Unidentified,
                    Modifiers {
                        meta: true,
                        ..Modifiers::default()
                    },
                ),
                Operation::Escape => canvas.press_key(Key::Escape),
                Operation::DoubleClick => {
                    if kind == Kind::Nodes {
                        model.nodes.set(super::node_edit::nodes());
                        screen.run_until_idle();
                        before = model.snapshot();
                        let control = screen.get_by_role(Role::SpinButton, "Source Amount");
                        control.click();
                        let p = control.rect().unwrap().center();
                        screen.mouse_move(p.x, p.y);
                        screen.mouse_down(MouseButton::Primary);
                        if screen.query_by_role(Role::SpinButton, "Amount").is_some() {
                            failures.push("Nodes x DoubleClick: popup opened on Down".into());
                        }
                        screen.mouse_up(MouseButton::Primary);
                        if screen.query_by_role(Role::SpinButton, "Amount").is_none() {
                            failures.push("Nodes x DoubleClick: popup missing after Up".into());
                        }
                    } else {
                        item(&screen, model, 0).double_click();
                        if model.activated.get_untracked() != [1] {
                            failures.push(format!(
                                "{kind:?} x DoubleClick: {:?}",
                                model.activated.get_untracked()
                            ));
                        }
                    }
                }
                Operation::DragBetween => {
                    let a = item(&screen, model, 0).rect().unwrap().center();
                    let b = item(&screen, model, 1).rect().unwrap().center();
                    screen.mouse_move(a.x, a.y);
                    screen.mouse_down(MouseButton::Primary);
                    screen.mouse_move(b.x, b.y);
                    screen.mouse_up(MouseButton::Primary);
                    if !model.activated.get_untracked().is_empty() {
                        failures.push(format!("{kind:?} x DragBetween: moved press activated"));
                    }
                    let after = model.snapshot();
                    if after[0] == before[0] || after[1..] != before[1..] {
                        failures.push(format!(
                            "{kind:?} x DragBetween: {before:?} -> {after:?}, only the first item moves"
                        ));
                    }
                }
                Operation::CancelEdit | Operation::RejectedContact => {
                    model.selection.set(vec![1, 2]);
                    screen.run_until_idle();
                    before = model.snapshot();
                    let a = item(&screen, model, 0).rect().unwrap().center();
                    screen.mouse_move(a.x, a.y);
                    screen.mouse_down(MouseButton::Primary);
                    if matches!(operation, Operation::RejectedContact) {
                        // The contact is a pointer of its own: the graph keeps the mouse's capture and ignores it.
                        let contact =
                            screen.contact_down(kivact::view::PointerKind::Touch, a.x, a.y);
                        screen.contact_move(contact, a.x + 90.0, a.y + 30.0);
                        screen.contact_up(contact);
                    }
                    screen.mouse_move(a.x + 17.0, a.y);
                    if matches!(operation, Operation::CancelEdit) {
                        screen.press_key(Key::Escape);
                        if model.snapshot() != before {
                            failures.push(format!("{kind:?} x CancelEdit: rollback changed"));
                        }
                    } else if model.snapshot() == before || model.snapshot()[2] != before[2] {
                        failures.push(format!(
                            "{kind:?} x RejectedContact: original capture lost or the unselected item moved"
                        ));
                    }
                    screen.mouse_up(MouseButton::Primary);
                }
                Operation::CtrlWheel | Operation::MetaWheel => {
                    let before = model.time_at(400.0);
                    let scale = model.scale();
                    screen.key_down(
                        Key::Unidentified,
                        KeyCode::Unidentified,
                        Modifiers {
                            ctrl: matches!(operation, Operation::CtrlWheel),
                            meta: matches!(operation, Operation::MetaWheel),
                            ..Modifiers::default()
                        },
                    );
                    let r = canvas.rect().unwrap();
                    screen.mouse_move(r.x + 400.0, r.y + 230.0);
                    screen.scroll(0.0, -40.0);
                    if !near(model.time_at(400.0), before, 1e-4) || model.scale() <= scale {
                        failures.push(format!(
                            "{kind:?} x {operation:?}: anchor {before} -> {}, scale {scale} -> {}",
                            model.time_at(400.0),
                            model.scale()
                        ));
                    }
                }
                Operation::SnapPreserves => {
                    model.selection.set(vec![1, 2]);
                    screen.run_until_idle();
                    before = model.snapshot();
                    drag_by(&screen, model, 0, Point::new(1.0, 0.0));
                    let after = model.snapshot();
                    // A one-pixel drag moves the selected items by one unit of the time
                    // editors, and snaps back to where it was in the others (graph step
                    // 10, node and arrangement grids larger than a pixel); the third item
                    // never moves.
                    let moved = match kind {
                        Kind::Timeline | Kind::DopeSheet => 1.0,
                        _ => 0.0,
                    };
                    for (index, (old, new)) in before.iter().zip(&after).enumerate() {
                        let dx = if index < 2 { moved } else { 0.0 };
                        if new.position != Point::new(old.position.x + dx, old.position.y) {
                            failures.push(format!(
                                "{kind:?} x SnapPreserves: item {index} {:?} -> {:?}, expected dx {dx}",
                                old.position, new.position
                            ));
                        }
                    }
                }
                Operation::NarrowOnRelease => {
                    model.selection.set(vec![1, 2]);
                    screen.run_until_idle();
                    before = model.snapshot();
                    item(&screen, model, 0).click();
                }
                _ => {
                    let modifiers = Modifiers {
                        shift: matches!(operation, Operation::ShiftMarquee),
                        ctrl: matches!(operation, Operation::CtrlMarquee),
                        meta: matches!(operation, Operation::MetaMarquee),
                        ..Modifiers::default()
                    };
                    screen.key_down(Key::Unidentified, KeyCode::Unidentified, modifiers);
                    let r = item(&screen, model, 1).rect().unwrap();
                    canvas.drag(
                        Point::new(r.x - 2.0, r.y - 2.0),
                        Point::new(r.x + r.width + 2.0, r.y + r.height + 2.0),
                    );
                }
            }
            let expected: &[u64] = match operation {
                Operation::MetaAll => &[1, 2, 3],
                Operation::Escape => &[],
                Operation::Marquee => &[2],
                Operation::CtrlWheel
                | Operation::MetaWheel
                | Operation::DoubleClick
                | Operation::DragBetween
                | Operation::NarrowOnRelease => &[1],
                _ => &[1, 2],
            };
            if model.selection.get_untracked() != expected {
                failures.push(format!(
                    "{kind:?} x {operation:?}: {:?} != {expected:?}",
                    model.selection.get_untracked()
                ));
            }
            // Only a drag moves items; every other row leaves them where they were.
            if matches!(
                operation,
                Operation::MetaAll
                    | Operation::Escape
                    | Operation::Marquee
                    | Operation::ShiftMarquee
                    | Operation::CtrlMarquee
                    | Operation::MetaMarquee
                    | Operation::CtrlWheel
                    | Operation::MetaWheel
                    | Operation::DoubleClick
                    | Operation::NarrowOnRelease
            ) && model.snapshot() != before
            {
                failures.push(format!(
                    "{kind:?} x {operation:?}: items moved {before:?} -> {:?}",
                    model.snapshot()
                ));
            }
            clean(&screen);
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn select_all_needs_ctrl_or_meta_without_alt() {
    for kind in [
        Kind::Timeline,
        Kind::DopeSheet,
        Kind::Graph,
        Kind::Nodes,
        Kind::Arrangement,
    ] {
        for (key, modifiers, all) in [
            (
                Key::Char('A'),
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
                true,
            ),
            (
                Key::Char('a'),
                Modifiers {
                    ctrl: true,
                    alt: true,
                    ..Modifiers::default()
                },
                false,
            ),
            (Key::Char('a'), Modifiers::default(), false),
        ] {
            let (screen, model) = fixture(kind, false);
            model.selection.set(vec![1]);
            screen.run_until_idle();
            let canvas = screen.get_by_role(Role::Canvas, "Gesture editor");
            canvas.focus();
            canvas.press_key_with(key, modifiers);
            let expected: &[u64] = if all { &[1, 2, 3] } else { &[1] };
            assert_eq!(
                model.selection.get_untracked(),
                expected,
                "{kind:?} {modifiers:?}"
            );
            clean(&screen);
        }
    }
}
