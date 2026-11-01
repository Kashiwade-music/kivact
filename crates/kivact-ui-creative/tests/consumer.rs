//! Role: An ordinary external consumer: only the public Kivact facade is imported.

use kivact::Callback;
use kivact::binding;
use kivact::draw::{Point, Size};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::test::{RenderOptions, Screen, render_with};
use kivact::ui::{MenuEntry, toaster};
use kivact::view::A11yAction;
use kivact::view::MouseButton;
use kivact::view::{KeyCode, Modifiers, callback};
use kivact_ui_creative as ui;
use ui::ThemeOptions;

fn choice_options(
    entries: &[(&'static str, &'static str)],
) -> Vec<kivact::ui::ListItem<&'static str>> {
    entries
        .iter()
        .map(|(key, label)| kivact::ui::ListItem::new(*key, *label))
        .collect()
}
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[path = "consumer/table_rows.rs"]
mod table_rows;
use table_rows::{Line, line, lines};
#[path = "consumer/cleanup.rs"]
mod cleanup;
#[path = "consumer/tables.rs"]
mod tables;

fn apply_clips(items: &mut Vec<ui::Clip>, request: &ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(values) => {
            for item in items {
                if let Some(value) = values.iter().find(|value| value.id == item.id) {
                    item.start = value.time;
                    item.duration = value.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => items.retain(|item| !ids.contains(&item.id)),
    }
}
fn apply_keys(items: &mut Vec<ui::Keyframe>, request: &ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(values) => {
            for item in items {
                if let Some(value) = values.iter().find(|value| value.id == item.id) {
                    item.time = value.time;
                    item.duration = value.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => items.retain(|item| !ids.contains(&item.id)),
    }
}
trait TestTime: Clone + PartialEq + 'static {
    fn timing(&self) -> ui::ItemTime;
}
impl TestTime for ui::Clip {
    fn timing(&self) -> ui::ItemTime {
        ui::ItemTime {
            id: self.id,
            time: self.start,
            duration: self.duration,
        }
    }
}
impl TestTime for ui::Keyframe {
    fn timing(&self) -> ui::ItemTime {
        ui::ItemTime {
            id: self.id,
            time: self.time,
            duration: self.duration,
        }
    }
}
fn time_edits<T: TestTime, R: kivact::view::HandlerResult>(
    items: impl Into<kivact::Binding<Vec<T>>>,
    apply: fn(&mut Vec<T>, &ui::TimeRequest),
    observer: impl Fn(kivact::ui::EditPhase) -> R + 'static,
) -> impl Fn(ui::EditorEvent<ui::TimeRequest>) + 'static {
    let items = items.into();
    let initial = signal(None::<Vec<T>>);
    let observer = callback(observer);
    move |event| {
        use kivact::ui::EditPhase;
        match event {
            ui::EditorEvent::Begin => {
                initial.set(Some(items.get_untracked()));
                observer.call(EditPhase::Begin);
            }
            ui::EditorEvent::Request(request) => {
                let mut next = items.get_untracked();
                apply(&mut next, &request);
                items.set(next);
            }
            ui::EditorEvent::Commit => {
                initial.set(None);
                observer.call(EditPhase::Commit);
            }
            ui::EditorEvent::Cancel => {
                if let Some(value) = initial.get_untracked() {
                    let mut next = items.get_untracked();
                    apply(
                        &mut next,
                        &ui::TimeRequest::Retime(value.iter().map(TestTime::timing).collect()),
                    );
                    items.set(next);
                }
                initial.set(None);
                observer.call(EditPhase::Cancel);
            }
        }
    }
}
fn app_timeline(
    items: impl Into<kivact::Binding<Vec<ui::Clip>>>,
    state: ui::TimeState,
) -> ui::Timeline {
    let items = items.into();
    ui::timeline(items, state).on_edit(time_edits(items, apply_clips, |_| {}))
}
fn app_dope_sheet(
    items: impl Into<kivact::Binding<Vec<ui::Keyframe>>>,
    state: ui::TimeState,
) -> ui::DopeSheet {
    let items = items.into();
    ui::dope_sheet(items, state).on_edit(time_edits(items, apply_keys, |_| {}))
}
fn time_observer<R: kivact::view::HandlerResult>(
    clips: Signal<Vec<ui::Clip>>,
    keys: Signal<Vec<ui::Keyframe>>,
    observer: impl Fn(kivact::ui::EditPhase) -> R + 'static,
) -> impl Fn(ui::EditorEvent<ui::TimeRequest>) + 'static {
    let clips = time_edits(clips, apply_clips, observer);
    let keys = time_edits(keys, apply_keys, |_| {});
    move |event: ui::EditorEvent<ui::TimeRequest>| {
        keys(event.clone());
        clips(event);
    }
}
macro_rules! creative_time_fixture {
    ($is_keys:expr, $clips:expr, $keys:expr, $state:expr, $($builder:tt)*) => {
        if $is_keys {
            app_dope_sheet($keys, $state) $($builder)* .into_view()
        } else {
            app_timeline($clips, $state) $($builder)* .into_view()
        }
    };
}

fn near(a: impl Into<f64>, b: impl Into<f64>, tolerance: f64) -> bool {
    (a.into() - b.into()).abs() < tolerance
}
fn value(clips: Signal<Vec<ui::Clip>>, keys: Signal<Vec<ui::Keyframe>>, is_keys: bool) -> f64 {
    if is_keys {
        keys.get_untracked()[0].time
    } else {
        clips.get_untracked()[0].start
    }
}
fn inspect(screen: &Screen, name: &str, item: Option<u64>, lane: Option<u64>, time: f64) {
    screen.get_by_role(Role::Menu, name);
    screen
        .get_by_role(Role::MenuItem, &format!("Inspect {item:?} {lane:?} {time}"))
        .click();
}

fn clean(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn default_fonts_and_user_modifiers_need_no_assets() {
    let (screen, count) = kivact::test::render_with_handle(crate::options(), || {
        let count = signal(0);
        (
            count,
            column((
                kivact::ui::button("Apply")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .icon(kivact::control::icon("check"))
                    .style(style().height(40))
                    .on_press(move || count.update(|v| *v += 1)),
                kivact::ui::button("Disabled")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .disabled(true),
                ui::number("Gain", signal(0.0)),
                ui::knob("Pan", signal(0.0)),
                ui::fader("Volume", signal(0.0)).range(-60.0, 6.0),
            )),
        )
    });
    let apply = screen.get_by_role(Role::Button, "Apply");
    assert_eq!(apply.rect().map(|r| r.height), Some(40.0));
    apply.click();
    apply.press_key(Key::Enter);
    assert_eq!(count.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    clean(&screen);
}

#[test]
fn toasts_are_live_pause_for_the_pointer_and_expire_on_the_host_clock() {
    let (screen, (toaster, (undone, undo))) =
        kivact::test::render_with_handle(crate::options(), || {
            let undone = signal(false);
            let undo = kivact::ui::Toast::new("Muted")
                .tone(kivact::ui::Tone::Info)
                .body("Track 2")
                .action("Undo", move || undone.set(true))
                .duration(None);
            (
                (toaster(), (undone, undo)),
                kivact::ui::toast_region(toaster()),
            )
        });
    toaster.push(
        kivact::ui::Toast::new("Exported")
            .tone(kivact::ui::Tone::Success)
            .body("Wrote 3 files"),
    );
    toaster.push(
        kivact::ui::Toast::new("Render failed")
            .tone(kivact::ui::Tone::Danger)
            .body("Out of memory"),
    );
    toaster.push(undo);
    screen.run_until_idle();
    screen.get_by_role(Role::Status, "Exported");
    screen.get_by_role(Role::Alert, "Render failed");
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups[0].live_region(), kivact::view::LiveRegion::Polite);
    assert_eq!(groups[1].live_region(), kivact::view::LiveRegion::Assertive);
    screen.get_by_text("Wrote 3 files").hover();
    screen.advance(Duration::from_secs(30));
    assert!(
        screen.query_by_text("Exported").is_some(),
        "held by the pointer"
    );
    assert!(screen.query_by_text("Render failed").is_none());
    assert!(
        screen.query_by_text("Muted").is_some(),
        "no duration: stays"
    );
    screen.mouse_leave();
    screen.advance(Duration::from_secs(5));
    assert!(screen.query_by_text("Exported").is_none());
    screen.get_by_role(Role::Button, "Undo").click();
    assert!(undone.get_untracked());
    assert!(screen.query_by_text("Muted").is_none());
    toaster.push(
        kivact::ui::Toast::new("Clipping")
            .tone(kivact::ui::Tone::Warning)
            .body("Master bus"),
    );
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Close").click();
    assert!(screen.query_by_text("Clipping").is_none());
    clean(&screen);
}

#[test]
fn mixed_check_switch_and_roving_segment_are_public_semantics() {
    let (screen, (check, choice)) = kivact::test::render_with_handle(crate::options(), || {
        let check = signal(false);
        let choice = signal(Some("constant"));
        (
            (check, choice),
            column((
                kivact::ui::checkbox("Mixed Selection", check).indeterminate(true),
                kivact::ui::switch("Enabled", signal(true)),
                kivact::ui::radio_group(
                    "Interpolation",
                    choice_options(&[
                        ("constant", "Constant"),
                        ("linear", "Linear"),
                        ("bezier", "Bezier"),
                    ]),
                    choice,
                )
                .orientation(kivact::view::Orientation::Horizontal)
                .variant(ui::Segmented),
                kivact::ui::button("After").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let mixed = screen.get_by_role(Role::CheckBox, "Mixed Selection");
    assert!(mixed.is_indeterminate());
    mixed.a11y(A11yAction::Toggle);
    assert!(check.get_untracked());
    let first = screen.get_by_role(Role::RadioButton, "Constant");
    first.focus();
    first.press_key(Key::ArrowRight);
    assert_eq!(choice.get_untracked(), Some("linear"));
    assert_eq!(screen.focused().map(|e| e.name()), Some("Linear".into()));
    screen.press_key(Key::Tab);
    assert_eq!(screen.focused().map(|e| e.name()), Some("After".into()));
    clean(&screen);
}

#[test]
fn check_switch_toggle_and_disclosure_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, bold, open)) =
        kivact::test::render_with_handle(crate::options(), || {
            let check = signal(false);
            let switch = signal(false);
            let bold = signal(false);
            let open = signal(false);
            (
                (check, switch, bold, open),
                column((
                    kivact::ui::checkbox("Visible", check),
                    kivact::ui::switch("Enabled", switch),
                    kivact::ui::toggle("Bold", bold),
                    kivact::ui::disclosure("Transform", open, || text("Position"))
                        .keep_mounted(true),
                    kivact::ui::checkbox("Locked", signal(false)).disabled(true),
                )),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Visible");
    let switch_node = screen.get_by_role(Role::Switch, "Enabled");
    let button = screen.get_by_role(Role::Button, "Bold");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked());
    button.press_key(Key::Enter);
    assert!(bold.get_untracked(), "Enter flips a toggle button");
    assert_eq!(button.is_checked(), Some(true));
    for node in [&check_box, &switch_node, &button] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked() && !bold.get_untracked());
    check_box.a11y(A11yAction::Activate);
    switch_node.a11y(A11yAction::Toggle);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.is_checked(), Some(false));

    let transform = screen.get_by_role(Role::Button, "Transform");
    assert_eq!(transform.is_expanded(), Some(false));
    transform.press_key(Key::Enter);
    assert!(open.get_untracked());
    transform.press_key(Key::Space);
    assert!(!open.get_untracked());
    transform.a11y(A11yAction::Toggle);
    assert_eq!(transform.is_expanded(), Some(true));
    transform.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    clean(&screen);
}

#[test]
fn all_numeric_geometries_accept_finite_clamped_accessibility_values() {
    let (screen, values) = kivact::test::render_with_handle(crate::options(), || {
        let values = [signal(0.5), signal(0.5), signal(0.5), signal(-6.0)];
        (
            values,
            row((
                ui::number("Exposure", values[0]).range(0.0, 1.0).step(0.1),
                ui::value_slider("Mix", values[1]),
                ui::knob("Drive", values[2]),
                ui::fader("Volume", values[3])
                    .range(-60.0, 6.0)
                    .precision(3),
            )),
        )
    });
    for (i, (role, name)) in [
        (Role::SpinButton, "Exposure"),
        (Role::Slider, "Mix"),
        (Role::Slider, "Drive"),
        (Role::Slider, "Volume"),
    ]
    .into_iter()
    .enumerate()
    {
        let control = screen.get_by_role(role, name);
        control.a11y_with_value(A11yAction::SetValue, "0.75");
        assert_eq!(values[i].get_untracked(), 0.75);
        control.a11y_with_value(A11yAction::SetValue, "999");
        let maximum = if i == 3 { 6.0 } else { 1.0 };
        assert_eq!(values[i].get_untracked(), maximum);
        for invalid in ["NaN", "inf", "no value"] {
            control.a11y_with_value(A11yAction::SetValue, invalid);
            assert_eq!(values[i].get_untracked(), maximum);
        }
    }
    clean(&screen);
}

#[test]
fn numeric_arithmetic_submit_cancel_and_blur_commit() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(0.25);
        (
            value,
            column((
                ui::number("Gain", value).range(-10.0, 10.0).precision(3),
                kivact::ui::button("Outside").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    let editor = screen.get_by_role(Role::TextBox, "Gain");
    editor.type_text("(2 + 3) / 4");
    screen.press_key(Key::ArrowLeft);
    assert_eq!(editor.text_selection(), Some((10, 10)));
    screen.press_key(Key::Home);
    assert_eq!(editor.text_selection(), Some((0, 0)));
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::End);
    assert_eq!(editor.text_selection(), Some((11, 11)));
    assert_eq!(value.get_untracked(), 0.25);
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 1.25);
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("1 / 0");
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 1.25);
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("3 * 2");
    screen.get_by_role(Role::Button, "Outside").click();
    assert_eq!(value.get_untracked(), 6.0);
    clean(&screen);
}

#[test]
fn a_value_slider_in_a_kept_disclosure_keeps_its_consumer_value() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(0.5);
        (
            value,
            column((kivact::ui::disclosure("Transform", signal(true), move || {
                ui::value_slider("Scale", value)
            })
            .keep_mounted(true),)),
        )
    });
    screen
        .get_by_role(Role::Slider, "Scale")
        .a11y_with_value(A11yAction::SetValue, "0.75");
    let disclosure = screen.get_by_role(Role::Button, "Transform");
    disclosure.click();
    disclosure.click();
    assert_eq!(value.get_untracked(), 0.75);
    clean(&screen);
}

fn test_node(
    id: u64,
    label: &str,
    x: f32,
    inputs: Vec<ui::NodeSocket>,
    outputs: Vec<ui::NodeSocket>,
) -> ui::NodeData {
    ui::NodeData::new(id, label, inputs, outputs)
        .position(Point::new(x, 30.0))
        .size(Size::new(160.0, 180.0))
        .color(rgb(0x8b7cff))
}

#[test]
fn typed_multiple_sockets_validate_reconnect_and_delete_nodes() {
    let (screen, (data, links)) = kivact::test::render_with_handle(crate::options(), || {
        let socket = ui::NodeSocket::new;
        let data = signal(vec![
            test_node(
                1,
                "Image",
                30.0,
                vec![],
                vec![
                    socket("Image", "Image", kivact::style::rgb(0xffd25a)),
                    socket("Alpha", "Scalar", kivact::style::rgb(0x9b9ea8)),
                ],
            ),
            test_node(
                2,
                "Mix",
                300.0,
                vec![
                    socket("Image", "Image", kivact::style::rgb(0xffd25a)),
                    socket("Factor", "Scalar", kivact::style::rgb(0x9b9ea8)),
                ],
                vec![socket("Image", "Image", kivact::style::rgb(0xffd25a))],
            ),
        ]);
        let links = signal(Vec::<ui::NodeLink>::new());
        (
            (data, links),
            app_node_editor(data, links, ui::NodeState::new())
                .style(style().height(300))
                .test_id("nodes"),
        )
    });
    screen
        .get_by_role(Role::Button, "Image output Alpha")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::Button, "Mix input Image")
        .a11y(A11yAction::Activate);
    assert!(links.get_untracked().is_empty());
    screen
        .get_by_role(Role::Button, "Image output Image")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::Button, "Mix input Image")
        .a11y(A11yAction::Activate);
    assert_eq!(links.get_untracked(), vec![ui::NodeLink::new(1, 0, 2, 0)]);
    screen
        .get_by_role(Role::Button, "Image output Alpha")
        .a11y(A11yAction::Activate);
    screen
        .get_by_role(Role::Button, "Mix input Factor")
        .a11y(A11yAction::Activate);
    assert_eq!(links.get_untracked().len(), 2);
    screen
        .get_by_role(Role::Button, "Image")
        .a11y(A11yAction::Select);
    screen.get_by_test_id("nodes").press_key(Key::Delete);
    assert_eq!(data.get_untracked().len(), 1);
    assert!(links.get_untracked().is_empty());
    clean(&screen);
}

#[test]
fn graph_tangents_are_draggable_and_keyboard_editable() {
    let (screen, data) = kivact::test::render_with_handle(crate::options(), || {
        let mut key = ui::CurveKey::new(1, 100.0, 198.0);
        key.out_tangent = ui::CurveTangent::new(60.0, 0.0);
        let data = signal(vec![
            ui::Curve::new(1, "Frame 20", vec![key]).color(rgb(0xff5c6c)),
        ]);
        (
            data,
            app_graph(
                data,
                ui::GraphState {
                    selection: (signal(vec![1])).into(),
                    view: (signal(ui::GraphView {
                        time: (0.0, 400.0),
                        value: (0.0, 278.0),
                    }))
                    .into(),
                    ..ui::GraphState::new()
                },
            )
            .style(style().size(600, 300))
            .test_id("graph"),
        )
    });
    let handle = screen.get_by_role(Role::Button, "Frame 20 key 100 outgoing tangent");
    let start = handle.rect().expect("tangent").center();
    handle.drag(start, Point::new(start.x + 30.0, start.y + 20.0));
    assert_eq!(
        data.get_untracked()[0].keys[0].out_tangent,
        ui::CurveTangent::new(90.0, -20.0)
    );
    screen.get_by_test_id("graph").press_key(Key::ArrowRight);
    assert_eq!(data.get_untracked()[0].keys[0].out_tangent.time, 91.0);
    assert_eq!(
        (
            data.get_untracked()[0].keys[0].time,
            data.get_untracked()[0].keys[0].value
        ),
        (100.0, 198.0)
    );
    clean(&screen);
}

#[test]
fn hsv_picker_value_and_alpha_are_independent() {
    let (screen, color) = kivact::test::render_with_handle(crate::options(), || {
        let color = signal(rgb(0x8b7cff));
        (
            color,
            ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                .wheel_size(200.0)
                .style(style().width(248))
                .test_id("picker"),
        )
    });
    let area = screen.get_by_role(Role::Slider, "Saturation and value");
    let r = area.rect().expect("SV square");
    area.drag(r.center(), Point::new(r.x + r.width, r.y));
    let (_, s, v) = ui::color_to_hsv(color.get_untracked());
    assert!((s - 1.0).abs() < 0.01 && (v - 1.0).abs() < 0.01);
    let alpha = screen.get_by_role(Role::Slider, "Alpha");
    let r = alpha.rect().expect("alpha strip");
    alpha.drag(r.center(), Point::new(r.x + r.width * 0.25, r.center().y));
    assert!((color.get_untracked().components()[3] - 0.25).abs() < 0.01);
    clean(&screen);
}

#[test]
fn viewport_surface_forwards_application_image_coordinates() {
    let (screen, position) = kivact::test::render_with_handle(crate::options(), || {
        let position = signal(Point::new(0.0, 0.0));
        (
            position,
            ui::viewport_surface(None)
                .transform(signal(kivact::interaction::ViewTransform {
                    offset: Point::new(10.0, 20.0),
                    scale: 2.0,
                }))
                .on_pointer(move |event: ui::SurfacePointer| position.set(event.position))
                .style(style().size(300, 200))
                .test_id("viewport"),
        )
    });
    screen.get_by_test_id("viewport").hover();
    screen.mouse_move(110.0, 120.0);
    assert_eq!(position.get_untracked(), Point::new(50.0, 50.0));
    clean(&screen);
}

#[test]
fn modal_contains_input_and_restores_opener_focus() {
    let (screen, _) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                {
                    let open = signal(false);
                    (
                        kivact::ui::button("Export...")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .expanded(open)
                            .on_press(move || open.set(true)),
                        kivact::ui::dialog("Export...", open)
                            .closable(false)
                            .content(|| {
                                kivact::ui::text_input("File name", signal("shot.exr".into()))
                                    .into_view()
                            })
                            .footer(move || {
                                kivact::ui::button(kivact::i18n::message(
                                    "creative.dialog.close",
                                    "Close",
                                ))
                                .variant(kivact::ui::ButtonVariant::Secondary)
                                .on_press(move || open.set(false))
                            }),
                    )
                },
                kivact::ui::button("Behind").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Export...");
    opener.focus();
    opener.click();
    screen.get_by_role(Role::TextBox, "File name").focus();
    screen.press_key(Key::Escape);
    assert_eq!(screen.focused().map(|e| e.name()), Some("Export...".into()));
    clean(&screen);
}

#[test]
fn node_parameters_accept_finite_set_value_drag_and_toggle() {
    let (screen, data) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(vec![
            ui::NodeData::new(1, "Mix", vec![], vec![])
                .position(Point::new(20.0, 20.0))
                .size(Size::new(180.0, 180.0))
                .color(rgb(0xffd25a))
                .controls(vec![
                    ui::NodeControl::number("Factor", 60.0, 0.5, 0.0, 1.0, 0.01),
                    ui::NodeControl::toggle("Use Alpha", 90.0, true),
                ]),
        ]);
        (
            data,
            app_node_editor(data, signal(Vec::new()), ui::NodeState::new())
                .test_id("node-parameters"),
        )
    });
    let value = screen.get_by_role(Role::SpinButton, "Mix Factor");
    value.a11y_with_value(A11yAction::SetValue, "0.75");
    assert_eq!(value.value_range().2, Some(0.75));
    value.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(value.value_range().2, Some(0.75));
    value.a11y_with_value(A11yAction::SetValue, "8");
    assert_eq!(value.value_range().2, Some(1.0));
    let alpha = screen.get_by_role(Role::Button, "Mix Use Alpha");
    assert_eq!(alpha.is_checked(), Some(true));
    alpha.a11y(A11yAction::Toggle);
    assert_eq!(alpha.is_checked(), Some(false));
    assert!(matches!(
        data.get_untracked()[0].controls[1].value,
        ui::NodeControlValue::Toggle(false)
    ));
    alpha.a11y(A11yAction::Activate);
    assert_eq!(alpha.is_checked(), Some(true));
    value.drag(Point::new(60.0, 90.0), Point::new(40.0, 90.0));
    assert_eq!(value.value_range().2, Some(0.8));
    clean(&screen);
}

#[test]
fn timeline_trimming_updates_consumer_clip_bounds() {
    let (screen, data) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(vec![
            ui::Clip::new(1, 0, 30.0, 100.0, "Clip").color(rgb(0x8b7cff)),
        ]);
        (
            data,
            app_timeline(
                data,
                ui::TimeState {
                    selection: (signal(vec![1])).into(),
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
    clean(&screen);
}

#[test]
fn nested_menu_keys_choose_a_leaf_key_and_delayed_tooltip_dismisses() {
    let (screen, selection) = kivact::test::render_with_handle(crate::options(), || {
        let selection = signal(None::<&'static str>);
        (
            selection,
            row((
                kivact::ui::menu(
                    "Add",
                    vec![
                        MenuEntry::submenu(
                            "mesh",
                            "Mesh",
                            vec![
                                MenuEntry::item("plane", "Plane"),
                                MenuEntry::item("cube", "Cube"),
                            ],
                        ),
                        MenuEntry::item("camera", "Camera"),
                    ],
                    move |key: &'static str| selection.set(Some(key)),
                )
                .offset(0.0)
                .attach(
                    kivact::ui::button("Add")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .trailing_icon(kivact::control::icon("chevron-down"))
                        .into_node(),
                ),
                kivact::ui::tooltip("Camera View")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .content(move || {
                        column((
                            text("Camera View").style(style().weight(600)),
                            text("Preview active camera").style(style().font_size(11)),
                        ))
                    })
                    .attach(
                        kivact::ui::button("Help")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .test_id("tooltip-target"),
                    ),
            )),
        )
    });
    screen.get_by_role(Role::Button, "Add").click();
    screen
        .get_by_role(Role::MenuItem, "Mesh")
        .press_key(Key::ArrowRight);
    screen
        .get_by_role(Role::MenuItem, "Plane")
        .press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(selection.get_untracked(), Some("cube"));
    assert!(screen.query_by_role(Role::Menu, "Add").is_none());
    screen.get_by_test_id("tooltip-target").hover();
    screen.advance(Duration::from_millis(499));
    assert!(screen.query_by_role(Role::Tooltip, "Camera View").is_none());
    screen.advance(Duration::from_millis(1));
    assert!(screen.get_by_role(Role::Tooltip, "Camera View").exists());
    screen.get_by_role(Role::Button, "Help").click();
    assert!(screen.query_by_role(Role::Tooltip, "Camera View").is_none());
    clean(&screen);
}

#[path = "consumer/advanced.rs"]
mod advanced;
#[path = "consumer/dock.rs"]
mod dock;
#[path = "consumer/dock_model.rs"]
mod dock_model;
#[path = "consumer/glyphs.rs"]
mod glyphs;
#[cfg(all(feature = "gpu-interop", any(windows, target_os = "linux")))]
#[path = "consumer/gpu_interop.rs"]
mod gpu_interop;
#[path = "consumer/interactions.rs"]
mod interactions;
#[path = "consumer/layout_parts.rs"]
mod layout_parts;

#[path = "consumer/node_regions.rs"]
mod node_regions;

#[path = "consumer/editor_actions.rs"]
mod editor_actions;
#[path = "consumer/editor_gestures.rs"]
mod editor_gestures;
#[path = "consumer/keyed.rs"]
mod keyed;
#[path = "consumer/messages.rs"]
mod messages;
#[path = "consumer/numeric_edit.rs"]
mod numeric_edit;
#[path = "consumer/outliner.rs"]
mod outliner;
#[path = "consumer/picker.rs"]
mod picker;
#[path = "consumer/range.rs"]
mod range;
#[path = "consumer/ruler.rs"]
mod ruler;

#[path = "consumer/table_stage3.rs"]
mod table_stage3;

#[test]
fn rendering_factories_accept_nodes_and_pointer_observers_accept_closures() {
    let (screen, pointers) = kivact::test::render_with_handle(crate::options(), || {
        let pointers = signal(0usize);
        (
            pointers,
            column((
                lines(
                    &["Name"],
                    signal(vec![line(1, ["Layer"])]),
                    signal(Vec::new()),
                )
                .cell_controls(|row, column| text(move || row.get().cells[column].clone())),
                ui::window_button("Node content", ui::WindowOptions::default(), |_| {
                    text("Window node")
                }),
                ui::viewport_surface(None)
                    .on_pointer(move |_: ui::SurfacePointer| pointers.update(|n| *n += 1))
                    .style(style().size(300, 100))
                    .test_id("pointer observer"),
            )),
        )
    });
    screen.get_by_text("Layer");
    screen.get_by_test_id("pointer observer").hover();
    assert!(pointers.get_untracked() > 0);
    screen.get_by_role(Role::Button, "Node content").click();
    screen.get_by_text("Window node");
    clean(&screen);
}

#[path = "consumer/arrangement_edit.rs"]
mod arrangement_edit;
#[path = "consumer/bezier.rs"]
mod bezier;
#[path = "consumer/color_wheel.rs"]
mod color_wheel;
#[path = "consumer/editor_edit.rs"]
mod editor_edit;
#[path = "consumer/gradient.rs"]
mod gradient;
#[path = "consumer/graph_domain.rs"]
mod graph_domain;
#[path = "consumer/graph_extensions.rs"]
mod graph_extensions;
#[path = "consumer/node_boundary.rs"]
mod node_boundary;
#[path = "consumer/node_edit.rs"]
mod node_edit;
#[path = "consumer/node_support.rs"]
mod node_support;
use node_support::app_node_editor;
#[path = "consumer/reactive_options.rs"]
mod reactive_options;
#[path = "consumer/time_extensions.rs"]
mod time_extensions;

#[path = "consumer/editor_extensions.rs"]
mod editor_extensions;

#[path = "consumer/time_step.rs"]
mod time_step;

#[path = "consumer/graph_support.rs"]
mod graph_support;
use graph_support::{app_graph, graph_edits};
#[path = "consumer/arrangement_model.rs"]
mod arrangement_model;
use arrangement_model::arrangement as app_arrangement;

#[path = "consumer/arrangement_boundary.rs"]
mod arrangement_boundary;

#[path = "consumer/audio_boundary.rs"]
mod audio_boundary;

fn options() -> RenderOptions {
    RenderOptions {
        theme: ui::theme(kivact::style::ThemeMode::Dark),
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}
