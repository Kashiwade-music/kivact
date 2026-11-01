//! Role: Compare region action advertisements with their public editing behavior.

use super::*;
use kivact::draw::Rect;
use kivact::style::Color;
use kivact::ui::EditPhase;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
enum Region {
    TimeLane,
    GraphLane,
    GraphKey,
    ColorHue,
    ColorSv,
    ColorAlpha,
    BezierPoint1,
    BezierPoint2,
    GradientStop,
    Item,
    Tangent,
    Trim,
    Socket,
    Number,
    Choice,
    Toggle,
    Link,
    Playhead,
    ArrangementPlayhead,
    TrackSwitch,
    Automation,
    LoopEndpoint,
    AddTrack,
    LoopBody,
    RangeBody,
}

#[derive(Clone, Copy)]
struct Models {
    color: Signal<Color>,
    bezier: Signal<ui::CubicBezier>,
    stops: Signal<Vec<ui::GradientStop>>,
    stop_selection: Signal<Option<usize>>,
    activated: Signal<Vec<u64>>,
    phases: Signal<Vec<EditPhase>>,
    nodes: Signal<Vec<ui::NodeData>>,
    clips: Signal<Vec<ui::Clip>>,
    curves: Signal<Vec<ui::Curve>>,
    selected: Signal<Vec<u64>>,
    links: Signal<Vec<ui::NodeLink>>,
    link: Signal<Option<ui::NodeLink>>,
    frame: Signal<f64>,
    orbit: Signal<Point>,
    tracks: Signal<Vec<ui::TimelineTrack>>,
    range: Signal<(f64, f64)>,
    cycling: Signal<bool>,
    additions: Signal<usize>,
}

impl Models {
    fn snapshot(self) -> String {
        format!(
            "{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}",
            (
                self.nodes.get_untracked(),
                self.clips.get_untracked(),
                self.curves.get_untracked()
            ),
            self.selected.get_untracked(),
            self.links.get_untracked(),
            self.link.get_untracked(),
            self.frame.get_untracked(),
            self.orbit.get_untracked(),
            self.tracks.get_untracked(),
            self.range.get_untracked(),
            self.cycling.get_untracked(),
            self.additions.get_untracked(),
        )
    }
}

fn fixture(region: Region) -> (Screen, Models) {
    kivact::test::render_with_handle(crate::options(), move || {
        let node = |id, label: &str, x| {
            ui::NodeData::new(
                id,
                label,
                vec![ui::NodeSocket::new(
                    "Value",
                    "Scalar",
                    kivact::style::rgb(0x9b9ea8),
                )],
                vec![ui::NodeSocket::new(
                    "Value",
                    "Scalar",
                    kivact::style::rgb(0x9b9ea8),
                )],
            )
            .position(Point::new(x, 60.0))
            .size(Size::new(180.0, 180.0))
            .controls(vec![
                ui::NodeControl::number("Amount", 75.0, 2.0, 0.0, 10.0, 1.0),
                ui::NodeControl::choice("Mode", 105.0, ["A", "B"], 0),
                ui::NodeControl::toggle("Enabled", 135.0, false),
            ])
        };
        let models = Models {
            nodes: signal(vec![node(1, "Source", 60.0), node(2, "Sink", 340.0)]),
            clips: signal(vec![
                ui::Clip::new(1, 0, 60.0, 180.0, "Source"),
                ui::Clip::new(2, 0, 340.0, 180.0, "Sink"),
            ]),
            curves: signal(vec![ui::Curve::new(
                1,
                "Source",
                vec![ui::CurveKey::new(1, 60.0, 60.0)],
            )]),
            selected: signal(if matches!(region, Region::Tangent | Region::Trim) {
                vec![1]
            } else {
                vec![]
            }),
            links: signal(if matches!(region, Region::Link) {
                vec![ui::NodeLink::new(1, 0, 2, 0)]
            } else {
                vec![]
            }),
            link: signal(None),
            frame: signal(10.0),
            orbit: signal(Point::new(0.65, 0.45)),
            tracks: signal(vec![
                ui::TimelineTrack::new(0, "Vox", ui::TrackKind::Audio, rgb(0x34d399))
                    .automation(vec![ui::AutomationPoint::new(1, 2.0, 0.25)]),
            ]),
            range: signal((0.0, 16.0)),
            cycling: signal(true),
            additions: signal(0),
            color: signal(ui::color_from_hsv(120.0, 0.5, 0.8, 0.6)),
            bezier: signal(ui::CubicBezier::new(0.3, 0.2, 0.7, 0.8)),
            stops: signal(vec![ui::GradientStop::new(0.5, rgb(0x4080a0))]),
            stop_selection: signal(None),
            activated: signal(Vec::new()),
            phases: signal(Vec::new()),
        };
        let is_added = matches!(
            region,
            Region::TimeLane
                | Region::GraphLane
                | Region::GraphKey
                | Region::ColorHue
                | Region::ColorSv
                | Region::ColorAlpha
                | Region::BezierPoint1
                | Region::BezierPoint2
                | Region::GradientStop
        );
        if is_added {
            models
                .clips
                .set(vec![ui::Clip::new(7, 10, 2.0, 2.0, "Clip")]);
            models.curves.set(vec![ui::Curve::new(
                10,
                "Curve",
                vec![ui::CurveKey::new(7, 2.0, 1.0)],
            )]);
            let edit = move |phase| models.phases.update(|phases| phases.push(phase));
            let activate = move |id| models.activated.update(|ids| ids.push(id));
            let added_view = match region {
                Region::TimeLane => app_timeline(
                    models.clips,
                    ui::TimeState {
                        selection: models.selected.into(),
                        ..ui::TimeState::new()
                    },
                )
                .lanes(vec![ui::Lane::new(10, "Lane")])
                .on_lane_activate(activate)
                .on_edit(time_edits(models.clips, apply_clips, edit))
                .into_view(),
                Region::GraphLane | Region::GraphKey => app_graph(
                    models.curves,
                    ui::GraphState {
                        view: (signal(ui::GraphView {
                            time: (0.0, 10.0),
                            value: (-2.0, 3.0),
                        }))
                        .into(),
                        selection: models.selected.into(),
                        ..ui::GraphState::new()
                    },
                )
                .lanes(vec![ui::Lane::new(10, "Lane")])
                .step(1.0, 0.01)
                .time_format(|time| time.to_string())
                .on_lane_activate(activate)
                .on_edit(graph_edits(models.curves, edit))
                .into_view(),
                Region::ColorHue | Region::ColorSv | Region::ColorAlpha => {
                    ui::color_wheel(models.color, signal(ui::ColorModel::Rgb))
                        .fields(false)
                        .on_edit(edit)
                        .into_view()
                }
                Region::BezierPoint1 | Region::BezierPoint2 => {
                    ui::bezier_curve(models.bezier).on_edit(edit).into_view()
                }
                Region::GradientStop => ui::gradient_editor(models.stops, models.stop_selection)
                    .on_edit(edit)
                    .into_view(),
                _ => unreachable!(),
            };
            return (models, added_view);
        }
        let view = if matches!(region, Region::RangeBody) {
            ui::range("Selection", models.range)
                .limits(0.0, 64.0)
                .interval(rgb(0x8b7cff))
                .movable(true)
                .test_id("surface")
                .into_view()
        } else if matches!(
            region,
            Region::ArrangementPlayhead
                | Region::TrackSwitch
                | Region::Automation
                | Region::LoopEndpoint
                | Region::AddTrack
                | Region::LoopBody
        ) {
            app_arrangement(
                models.clips,
                models.tracks,
                ui::ArrangementState {
                    selection: models.selected.into(),
                    playhead: models.frame.into(),
                    loop_range: models.range.into(),
                    cycling: models.cycling.into(),
                    ..ui::ArrangementState::new()
                },
            )
            .on_add_track(move || models.additions.update(|n| *n += 1))
            .style(style().size(700, 360))
            .test_id("surface")
            .into_view()
        } else if matches!(region, Region::Tangent) {
            app_graph(
                models.curves,
                ui::GraphState {
                    view: (signal(ui::GraphView {
                        time: (0.0, 100.0),
                        value: (0.0, 100.0),
                    }))
                    .into(),
                    selection: models.selected.into(),
                    ..ui::GraphState::new()
                },
            )
            .time_format(|time| format!("{time}"))
            .style(style().size(700, 360))
            .test_id("surface")
            .into_view()
        } else if matches!(region, Region::Trim | Region::Playhead) {
            app_timeline(
                models.clips,
                ui::TimeState {
                    selection: models.selected.into(),
                    playhead: models.frame.into(),
                    ..ui::TimeState::new()
                },
            )
            .style(style().size(700, 360))
            .test_id("surface")
            .into_view()
        } else {
            app_node_editor(
                models.nodes,
                models.links,
                ui::NodeState {
                    selection: models.selected.into(),
                    link: models.link.into(),
                    ..ui::NodeState::new()
                },
            )
            .style(style().size(700, 360))
            .test_id("surface")
            .into_view()
        };
        (models, view)
    })
}

fn verify_effect(screen: &Screen, models: Models, region: Region, action: A11yAction) {
    if matches!(
        region,
        Region::TimeLane
            | Region::GraphLane
            | Region::GraphKey
            | Region::ColorHue
            | Region::ColorSv
            | Region::ColorAlpha
            | Region::BezierPoint1
            | Region::BezierPoint2
            | Region::GradientStop
    ) {
        use A11yAction::{Activate, Increment, Select, SetValue};
        let selection_only = matches!(action, Select | Activate);
        let delta = if action == Increment { 0.01 } else { -0.01 };
        match region {
            Region::TimeLane | Region::GraphLane => {
                assert_eq!(models.activated.get_untracked(), [10]);
                assert!(models.selected.get_untracked().is_empty());
            }
            Region::GraphKey => {
                let key = &models.curves.get_untracked()[0].keys[0];
                assert_eq!(key.id, 7);
                if selection_only {
                    assert_eq!(models.selected.get_untracked(), [7]);
                    assert!(near(key.time, 2.0, 1e-5));
                    assert!(near(key.value, 1.0, 1e-5));
                } else {
                    assert!(near(
                        key.time,
                        if action == SetValue {
                            4.0
                        } else if action == Increment {
                            3.0
                        } else {
                            1.0
                        },
                        1e-5
                    ));
                    assert!(near(
                        key.value,
                        if action == SetValue { 2.0 } else { 1.0 },
                        1e-5
                    ));
                }
            }
            Region::ColorHue | Region::ColorSv | Region::ColorAlpha => {
                let color = models.color.get_untracked();
                let (h, s, v) = ui::color_to_hsv(color);
                let (hue, saturation, value, alpha) = if selection_only {
                    (120.0, 0.5, 0.8, 0.6)
                } else {
                    match region {
                        Region::ColorHue => (
                            if action == SetValue {
                                180.0
                            } else if action == Increment {
                                121.0
                            } else {
                                119.0
                            },
                            0.5,
                            0.8,
                            0.6,
                        ),
                        Region::ColorSv => (
                            120.0,
                            if action == SetValue { 0.7 } else { 0.5 + delta },
                            if action == SetValue { 0.6 } else { 0.8 },
                            0.6,
                        ),
                        Region::ColorAlpha => (
                            120.0,
                            0.5,
                            0.8,
                            if action == SetValue { 0.4 } else { 0.6 + delta },
                        ),
                        _ => unreachable!("color regions only"),
                    }
                };
                assert!(near(h, hue, 1e-5));
                assert!(near(s, saturation, 1e-5));
                assert!(near(v, value, 1e-5));
                assert!(near(f64::from(color.components()[3]), alpha, 1e-5));
            }
            Region::BezierPoint1 | Region::BezierPoint2 => {
                let curve = models.bezier.get_untracked();
                let first = matches!(region, Region::BezierPoint1);
                let (initial_x, initial_y) = if first { (0.3, 0.2) } else { (0.7, 0.8) };
                let (x, y) = if first {
                    (curve.x1, curve.y1)
                } else {
                    (curve.x2, curve.y2)
                };
                assert!(near(
                    x,
                    if selection_only {
                        initial_x
                    } else if action == SetValue {
                        0.4
                    } else {
                        initial_x + delta
                    },
                    1e-5
                ));
                assert!(near(
                    y,
                    if !selection_only && action == SetValue {
                        1.2
                    } else {
                        initial_y
                    },
                    1e-5
                ));
                assert_eq!(
                    if first {
                        (curve.x2, curve.y2)
                    } else {
                        (curve.x1, curve.y1)
                    },
                    if first { (0.7, 0.8) } else { (0.3, 0.2) }
                );
            }
            Region::GradientStop => {
                assert_eq!(models.stop_selection.get_untracked(), Some(0));
                let stops = models.stops.get_untracked();
                assert_eq!(stops.len(), 1);
                assert_eq!(stops[0].color, rgb(0x4080a0));
                assert!(near(
                    stops[0].position,
                    if selection_only {
                        0.5
                    } else if action == SetValue {
                        0.7
                    } else {
                        0.5 + delta
                    },
                    1e-5
                ));
            }
            _ => unreachable!(),
        }
        let no_edit = selection_only || matches!(region, Region::TimeLane | Region::GraphLane);
        assert_eq!(
            models.phases.get_untracked(),
            if no_edit {
                vec![]
            } else {
                vec![EditPhase::Begin, EditPhase::Commit]
            },
            "{region:?} {action:?}"
        );
        if !matches!(region, Region::TimeLane | Region::GraphLane) {
            assert!(models.activated.get_untracked().is_empty());
        }
        return;
    }
    match region {
        Region::Item | Region::Trim => {
            assert_eq!(models.selected.get_untracked(), vec![1])
        }
        Region::Tangent => {
            match action {
                A11yAction::Select | A11yAction::Activate => {
                    let before = models.curves.get_untracked()[0].keys[0].in_tangent;
                    screen.get_by_test_id("surface").press_key(Key::ArrowLeft);
                    assert_ne!(models.curves.get_untracked()[0].keys[0].in_tangent, before);
                }
                A11yAction::SetValue => assert_eq!(
                    models.curves.get_untracked()[0].keys[0].in_tangent,
                    ui::CurveTangent::new(0.5, 0.5)
                ),
                A11yAction::Increment => assert_eq!(
                    models.curves.get_untracked()[0].keys[0].in_tangent,
                    ui::CurveTangent::new(0.0, 0.0)
                ),
                A11yAction::Decrement => assert_eq!(
                    models.curves.get_untracked()[0].keys[0].in_tangent,
                    ui::CurveTangent::new(-2.0, 0.0)
                ),
                _ => unreachable!(),
            }
            assert_eq!(
                (
                    models.curves.get_untracked()[0].keys[0].time,
                    models.curves.get_untracked()[0].keys[0].value
                ),
                (60.0, 60.0)
            );
        }
        Region::Socket => {
            screen
                .get_by_role(Role::Button, "Sink input Value")
                .a11y(A11yAction::Activate);
            assert_eq!(
                models.links.get_untracked(),
                vec![ui::NodeLink::new(1, 0, 2, 0)]
            );
        }
        Region::Number => {
            let ui::NodeControlValue::Number { value, .. } =
                models.nodes.get_untracked()[0].controls[0].value
            else {
                panic!("number control")
            };
            assert_eq!(
                value,
                match action {
                    A11yAction::SetValue => 0.5,
                    A11yAction::Increment => 3.0,
                    _ => 1.0,
                }
            );
        }
        Region::Choice => {
            screen.get_by_role(Role::Option, "B").click();
            assert!(matches!(
                models.nodes.get_untracked()[0].controls[1].value,
                ui::NodeControlValue::Choice { selected: 1, .. }
            ));
        }
        Region::Toggle => assert!(matches!(
            models.nodes.get_untracked()[0].controls[2].value,
            ui::NodeControlValue::Toggle(true)
        )),
        Region::Link => assert_eq!(
            models.link.get_untracked(),
            Some(ui::NodeLink::new(1, 0, 2, 0))
        ),
        Region::Playhead | Region::ArrangementPlayhead => assert_eq!(
            models.frame.get_untracked(),
            match action {
                A11yAction::SetValue => 0.5,
                A11yAction::Increment => 11.0,
                _ => 9.0,
            }
        ),
        Region::TrackSwitch => assert!(models.tracks.get_untracked()[0].muted),
        Region::Automation if action == A11yAction::Select => {
            screen.get_by_test_id("surface").press_key(Key::Delete);
            assert!(models.tracks.get_untracked()[0].automation.is_empty());
            assert_eq!(models.clips.get_untracked().len(), 2);
        }
        Region::Automation => assert_eq!(models.tracks.get_untracked()[0].automation[0].value, 0.5),
        Region::LoopEndpoint => assert_eq!(models.range.get_untracked(), (0.5, 16.0)),
        Region::AddTrack => assert_eq!(models.additions.get_untracked(), 1),
        Region::LoopBody => assert!(!models.cycling.get_untracked()),
        Region::RangeBody => {
            screen.get_by_test_id("surface").press_key(Key::ArrowRight);
            assert_eq!(models.range.get_untracked(), (1.0, 17.0));
        }
        _ => unreachable!(),
    }
}

#[test]
fn every_generated_region_advertises_exactly_the_actions_it_processes() {
    use A11yAction::{Activate, Decrement, Increment, Select, SetValue, Toggle as ToggleAction};
    use Region::*;
    // AccessKit maps Activate/Select/Toggle through one Click action. Compare
    // explicit semantic actions here; the host's activation aliases are equivalent.
    // Focus, when present, belongs to the host's tab-stop contract.
    let rows: &[(Region, Role, &str, &[A11yAction])] = &[
        (Item, Role::Button, "Source", &[Select, Activate]),
        (
            Tangent,
            Role::Button,
            "Source key 60 incoming tangent",
            &[Select, Activate, SetValue, Increment, Decrement],
        ),
        (Trim, Role::Button, "Source start trim", &[Activate]),
        (Socket, Role::Button, "Source output Value", &[Activate]),
        (
            Number,
            Role::SpinButton,
            "Source Amount",
            &[SetValue, Increment, Decrement],
        ),
        (Choice, Role::Button, "Source Mode", &[Activate]),
        (
            Region::Toggle,
            Role::Button,
            "Source Enabled",
            &[ToggleAction, Activate],
        ),
        (
            Link,
            Role::Button,
            "Link Source output 0 to Sink input 0",
            &[Select, Activate],
        ),
        (
            Playhead,
            Role::Slider,
            "Playhead",
            &[SetValue, Increment, Decrement],
        ),
        (ArrangementPlayhead, Role::Slider, "Playhead", &[SetValue]),
        (TrackSwitch, Role::Button, "Vox Mute", &[Activate]),
        (
            Automation,
            Role::Slider,
            "Vox automation point 1",
            &[SetValue, Select],
        ),
        (LoopEndpoint, Role::Slider, "Loop start", &[SetValue]),
        (AddTrack, Role::Button, "Add Track", &[Activate]),
        (LoopBody, Role::Button, "Loop range", &[Activate]),
        (RangeBody, Role::Button, "Selection interval", &[Select]),
    ];
    for &(region, role, name, handled) in rows {
        for &action in handled {
            let (screen, models) = fixture(region);
            let element = screen.get_by_role(role, name);
            let advertised = element.a11y_actions();
            assert!(
                advertised
                    .iter()
                    .all(|action| { *action == A11yAction::Focus || handled.contains(action) }),
                "{region:?}: unexpected advertised actions {advertised:?}"
            );
            for candidate in [
                Select,
                Activate,
                ToggleAction,
                SetValue,
                Increment,
                Decrement,
            ] {
                assert_eq!(
                    advertised.contains(&candidate),
                    handled.contains(&candidate),
                    "{region:?}: {candidate:?}"
                );
            }
            // Unsupported numeric actions must not bubble to the surface and
            // edit its playhead or a parameter through the root handler.
            for candidate in [SetValue, Increment, Decrement] {
                if !handled.contains(&candidate) {
                    let before = models.snapshot();
                    element.a11y_with_value(candidate, "0.5");
                    assert_eq!(
                        models.snapshot(),
                        before,
                        "{region:?}: unadvertised {candidate:?}"
                    );
                }
            }
            element.a11y_with_value(
                action,
                if matches!((region, action), (Region::Tangent, A11yAction::SetValue)) {
                    "0.5,0.5"
                } else {
                    "0.5"
                },
            );
            verify_effect(&screen, models, region, action);
            clean(&screen);
        }
    }
}

#[test]
fn added_regions_advertise_exactly_the_actions_that_select_activate_or_edit() {
    use A11yAction::{Activate, Decrement, Increment, Select, SetValue};
    use Region::*;
    let numeric = &[Select, Activate, SetValue, Increment, Decrement][..];
    let rows: &[(Region, Role, &str, &[A11yAction])] = &[
        (TimeLane, Role::Group, "Lane", &[Activate]),
        (GraphLane, Role::Group, "Lane", &[Activate]),
        (GraphKey, Role::Button, "Curve key 2", numeric),
        (ColorHue, Role::Slider, "Hue", numeric),
        (ColorSv, Role::Slider, "Saturation and value", numeric),
        (ColorAlpha, Role::Slider, "Alpha", numeric),
        (BezierPoint1, Role::Slider, "Control point 1", numeric),
        (BezierPoint2, Role::Slider, "Control point 2", numeric),
        (GradientStop, Role::Slider, "Gradient stop 0", numeric),
    ];
    // The native bridge normalizes Activate/Select/Toggle activation aliases to
    // Click, as in the existing matrix. Compare explicit advertised semantics;
    // no native-only alias is an extra kit action. Focus belongs to the host.
    for &(row, role, name, expected) in rows {
        let (screen, data) = fixture(row);
        let original = (
            data.clips.get_untracked(),
            data.curves.get_untracked(),
            data.color.get_untracked(),
            data.bezier.get_untracked(),
            data.stops.get_untracked(),
        );
        for &action in expected {
            data.clips.set(original.0.clone());
            data.curves.set(original.1.clone());
            data.color.set(original.2);
            data.bezier.set(original.3);
            data.stops.set(original.4.clone());
            data.selected.set(Vec::new());
            data.stop_selection.set(None);
            data.activated.set(Vec::new());
            data.phases.set(Vec::new());
            screen.run_until_idle();
            let element = screen.get_by_role(role, name);
            let advertised = element
                .a11y_actions()
                .into_iter()
                .filter(|action| *action != A11yAction::Focus)
                .collect::<HashSet<_>>();
            assert_eq!(
                advertised,
                expected.iter().copied().collect::<HashSet<_>>(),
                "{row:?}: missing or extra advertisement"
            );
            let value = match row {
                GraphKey => "4,2",
                ColorHue => "180",
                ColorSv => "0.7,0.6",
                ColorAlpha => "0.4",
                BezierPoint1 | BezierPoint2 => "0.4,1.2",
                GradientStop => "0.7",
                TimeLane | GraphLane => "",
                _ => unreachable!(),
            };
            element.a11y_with_value(action, value);
            verify_effect(&screen, data, row, action);
            if matches!(action, Select | Activate)
                && matches!(
                    row,
                    ColorHue | ColorSv | ColorAlpha | BezierPoint1 | BezierPoint2 | GradientStop
                )
            {
                assert!(
                    element.is_focused(),
                    "{row:?}: selection focuses the target"
                );
            }
            clean(&screen);
        }
    }
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::viewport_surface(None).regions(vec![ui::SurfaceRegion::new(
                7,
                "Application region",
                Role::Group,
                Rect::new(10.0, 10.0, 40.0, 30.0),
            )]),
        )
    });
    assert!(
        screen
            .get_by_role(Role::Group, "Application region")
            .a11y_actions()
            .is_empty()
    );
    clean(&screen);
}
