//! Editors and input controls for creative editing.
use super::audio_values::TransportState;
use kivact::state::Signal;

use super::common::{bind, col, keyed_options, line, note, spec, specs};
use super::controls::num;
use super::controls::tool_palette;
use super::editor_images as images;
use kivact::draw::Path;
use kivact::geometry::Rect;
use kivact::geometry::{Point, Size};
use kivact::interaction::MenuEntry;
use kivact::prelude::{
    Element, IntoView, NodeRef, Role, dynamic, memo, node, rgb, row, signal, spacer, text,
};
use kivact::style::Position;
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::view::Node;
use kivact_ui_creative as ui;

pub(super) fn playhead(state: Signal<TransportState>) -> Signal<f64> {
    bind(
        move || state.get().frame,
        move |v| state.update(|s| s.frame = v.max(0.0)),
    )
}

/// The grading viewer's display: graded, bypassed, or wiped.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Grade {
    Graded,
    Bypassed,
    Wipe,
}

pub(super) fn scene() -> Node {
    node()
        .role(Role::Image)
        .name("Procedural lakeside grading test image")
        .style(style().min_width(0))
        .draw(|d, size| {
            let w = size.width;
            let h = size.height;
            let horizon = h * 0.66;
            scene_sky(d, w, horizon);
            for i in 0..3 {
                let mut path = Path::new().move_to(Point::new(0.0, horizon));
                for x in 0..26 {
                    let px = x as f64 * w / 25.0;
                    let ridge =
                        horizon - (0.13 + (x as f64 * 1.9 + i as f64).sin().abs() * 0.12) * h;
                    path = path.line_to(Point::new(px, ridge + i as f64 * h * 0.04));
                }
                d.fill_path(
                    &path.line_to(Point::new(w, horizon)).close(),
                    [rgb(0x565570), rgb(0x405b61), rgb(0x194c44)][i],
                );
            }
            for y in 0..32 {
                let t = y as f64 / 31.0;
                d.fill_rect(
                    Rect::new(
                        0.0,
                        horizon + t * (h - horizon),
                        w,
                        (h - horizon) / 31.0 + 1.0,
                    ),
                    rgb(0x4e94ae).mix(rgb(0x053957), t, &kivact::style::use_theme().get()),
                );
            }
            d.fill_path(
                &Path::new()
                    .move_to(Point::new(0.0, h * 0.56))
                    .line_to(Point::new(w * 0.2, h * 0.65))
                    .line_to(Point::new(w * 0.3, h))
                    .line_to(Point::new(0.0, h))
                    .close(),
                rgb(0x075532),
            );
            for i in 0..12 {
                let x = i as f64 * w * 0.025;
                let top = h * (0.43 + (i as f64 * 2.4).sin().abs() * 0.12);
                d.fill_path(
                    &Path::new()
                        .move_to(Point::new(x, top))
                        .line_to(Point::new(x - w * 0.025, h * 0.65))
                        .line_to(Point::new(x + w * 0.025, h * 0.65))
                        .close(),
                    if i % 2 == 0 {
                        rgb(0x279837)
                    } else {
                        rgb(0x46b934)
                    },
                );
            }
            d.fill_rect(
                Rect::new(w * 0.2, h * 0.68, w * 0.05, h * 0.06),
                rgb(0xf43835),
            );
            d.rounded_rect(
                Rect::new(w * 0.84, h * 0.8, h * 0.1, h * 0.1),
                h * 0.05,
                rgb(0xffcc82),
            );
            d.fill_rect(
                Rect::new(w * 0.79, h * 0.9, w * 0.035, h * 0.045),
                rgb(0x43e4df),
            );
            d.fill_rect(
                Rect::new(w * 0.845, h * 0.92, w * 0.025, h * 0.035),
                rgb(0xffd451),
            );
        })
}

pub(super) fn picker_channel(
    color: Signal<Color>,
    index: usize,
    label: &str,
    name: &str,
) -> impl IntoView {
    ui::value_slider(
        label.to_owned(),
        bind(
            move || color.get().components(&kivact::style::use_theme().get())[index],
            move |v| {
                let mut c = color
                    .get_untracked()
                    .components(&kivact::style::use_theme().get());
                c[index] = v;
                color.set(Color::from_rgba_f64(c[0], c[1], c[2], c[3]));
            },
        ),
    )
    .name(name.to_owned())
    .range(0.0..=1.0)
    .step(0.001)
    .precision(3)
    .style(style().width(248))
    .accent_color(rgb([0xff5c6c, 0xa3e635, 0x48c8ff, 0x8b7cff][index]))
}
pub(super) fn channel(color: Signal<Color>, index: usize, label: &str) -> impl IntoView {
    ui::number(
        label.to_string(),
        bind(
            move || color.get().components(&kivact::style::use_theme().get())[index],
            move |v| {
                let mut c = color
                    .get_untracked()
                    .components(&kivact::style::use_theme().get());
                c[index] = v;
                color.set(Color::from_rgba_f64(c[0], c[1], c[2], c[3]));
            },
        ),
    )
    .range(0.0..=1.0)
    .step(0.01)
    .precision(3)
    .style(style().flex_grow(1))
}
pub(super) fn color_swatch(color: Signal<Color>) -> impl IntoView {
    ui::color_popup("Edit color", color).style(style().width(130))
}

pub(super) fn apply_time(keys: &mut Vec<ui::Keyframe>, request: &ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(values) => {
            for key in keys {
                if let Some(value) = values.iter().find(|value| value.id == key.id) {
                    key.time = value.time;
                    key.duration = value.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => keys.retain(|key| !ids.contains(&key.id)),
    }
}

pub(super) fn empty_pixels() -> kivact::draw::Pixels {
    kivact::draw::Pixels::new(1, 1, vec![0; 4])
}

pub(super) fn animation_curves() -> Vec<ui::Curve> {
    let curve_specs: [&[(f64, f64)]; 3] = [
        &[(0.0, 0.0), (60.0, 2.2), (120.0, -1.4), (180.0, 0.0)],
        &[(0.0, 0.0), (90.0, 1.6), (180.0, 0.0)],
        &[
            (0.0, 0.0),
            (40.0, 1.2),
            (80.0, 0.0),
            (120.0, 0.8),
            (160.0, 0.0),
        ],
    ];
    let mut curves = curve_specs
        .into_iter()
        .enumerate()
        .map(|(channel, points)| {
            let keys = points
                .iter()
                .enumerate()
                .map(|(i, &(frame, value))| {
                    let prev = i.checked_sub(1).and_then(|i| points.get(i));
                    let next = points.get(i + 1);
                    let dl = prev.map_or(10.0, |p| (frame - p.0) / 3.0);
                    let dr = next.map_or(10.0, |n| (n.0 - frame) / 3.0);
                    let slope = match (prev, next) {
                        (Some(p), Some(n))
                            if !((value >= p.1 && value >= n.1)
                                || (value <= p.1 && value <= n.1)) =>
                        {
                            (n.1 - p.1) / (n.0 - p.0)
                        }
                        _ => 0.0,
                    };
                    let mut key = ui::CurveKey::new((channel * 8 + i) as u64 + 1, frame, value);
                    key.in_tangent = ui::CurveTangent::new(-dl, -(slope * dl));
                    key.out_tangent = ui::CurveTangent::new(dr, slope * dr);
                    key
                })
                .collect();
            ui::Curve::new(
                channel as u64,
                ["X Location", "Y Location", "Z Location"][channel],
                keys,
            )
            .color([rgb(0xff5c6c), rgb(0x8fd94a), rgb(0x4d9bff)][channel])
        })
        .collect::<Vec<_>>();
    curves.push(
        ui::Curve::new(
            3,
            "Focal Length",
            [0.0, 120.0, 196.0]
                .into_iter()
                .enumerate()
                .map(|(i, time)| ui::CurveKey::new(4096 + i as u64, time, 0.0))
                .collect(),
        )
        .color(rgb(0xc6c8d0)),
    );
    curves
}

pub(super) fn animation_graph_editor(
    state: Signal<TransportState>,
    graph: Signal<Vec<ui::Curve>>,
    graph_selected: Signal<Vec<u64>>,
    channels: [Signal<bool>; 3],
    graph_view: Signal<ui::GraphView>,
) -> impl IntoView {
    ui::graph_editor(
        graph,
        ui::GraphState {
            view: graph_view,
            selection: graph_selected,
            playhead: playhead(state),
        },
    )
    .on_edit(super::animation_model::graph_edits(graph))
    .lanes(move || {
        channels
            .iter()
            .enumerate()
            .filter(|(_, visible)| visible.get())
            .map(|(id, _)| ui::Lane::new(id as u64, ["X Location", "Y Location", "Z Location"][id]))
            .collect::<Vec<_>>()
    })
    .style(style().height(320))
    .test_id("creative-graph")
}

pub(super) fn animation_graph_toolbar(
    graph: Signal<Vec<ui::Curve>>,
    channels: [Signal<bool>; 3],
    graph_view: Signal<ui::GraphView>,
) -> Node {
    line((
        text("Graph Editor").style(style().font_weight(600)),
        kivact::ui::radio_group(
            "Interpolation",
            keyed_options([
                ("constant", "Constant"),
                ("linear", "Linear"),
                ("bezier", "Bezier"),
            ]),
            bind(
                move || {
                    graph.with(|g| {
                        Some(
                            g.first()
                                .and_then(|curve| curve.keys.first())
                                .map_or("bezier", |p| match p.interpolation {
                                    ui::Interpolation::Constant => "constant",
                                    ui::Interpolation::Linear => "linear",
                                    ui::Interpolation::Bezier => "bezier",
                                })
                                .to_owned(),
                        )
                    })
                },
                move |key: Option<String>| {
                    graph.update(|curves| {
                        let ids = curves
                            .iter()
                            .flat_map(|curve| &curve.keys)
                            .map(|key| key.id)
                            .collect();
                        super::animation_model::apply_graph(
                            curves,
                            &ui::GraphRequest::Interpolate(
                                ids,
                                match key.as_deref() {
                                    Some("constant") => ui::Interpolation::Constant,
                                    Some("linear") => ui::Interpolation::Linear,
                                    _ => ui::Interpolation::Bezier,
                                },
                            ),
                        );
                    })
                },
            ),
        )
        .orientation(kivact::view::Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
        .style(style().width(166)),
        kivact::ui::toggle_button("X", channels[0]),
        kivact::ui::toggle_button("Y", channels[1]),
        kivact::ui::toggle_button("Z", channels[2]),
        kivact::ui::button("Frame All  Home")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .on_press(move || {
                graph_view.set(ui::GraphView {
                    time: (-20.0, 200.0),
                    value: (-2.0, 3.0),
                })
            }),
        spacer(),
        note("Z Location · f 40 · 1.200 · bezier"),
    ))
    .style(style().height(38).padding_x(10))
}

pub(super) fn animation_dope_sheet(
    state: Signal<TransportState>,
    keys: Signal<Vec<ui::Keyframe>>,
    selected: Signal<Vec<u64>>,
    initial_keys: Signal<super::txn::Txn<Vec<ui::Keyframe>>>,
    transform: Signal<kivact::interaction::ViewTransform>,
) -> impl IntoView {
    ui::dope_sheet(
        keys,
        ui::TimeState {
            selection: selected,
            playhead: playhead(state),
            transform,
        },
    )
    .on_edit(move |event| match event {
        ui::EditorEvent::Begin => super::txn::Txn::begin(initial_keys, keys.get()),
        ui::EditorEvent::Request(request) => {
            let mut next = keys.get();
            apply_time(&mut next, &request);
            keys.set(next);
        }
        ui::EditorEvent::Commit => super::txn::Txn::commit(initial_keys),
        ui::EditorEvent::Cancel => {
            if let Some(initial) = super::txn::Txn::cancel(initial_keys) {
                keys.set(initial);
            }
        }
    })
    .zoom_range(0.25..=10.0)
    .lanes(
        [
            "Summary",
            "□ Cube",
            "  Object Transforms",
            "   ● X Location",
            "   ● Y Location",
            "   ● Z Location",
            "▣ Camera",
            "   ● Focal Length",
        ]
        .into_iter()
        .enumerate()
        .map(|(id, label)| ui::Lane::new(id as u64, label))
        .collect::<Vec<_>>(),
    )
    .style(style().height(252))
    .test_id("creative-dopesheet")
}

pub(super) fn animation_dope_toolbar(
    state: Signal<TransportState>,
    keys: Signal<Vec<ui::Keyframe>>,
) -> Node {
    line((
        text("Dope Sheet").style(style().font_weight(600)),
        kivact::ui::select(
            keyed_options([
                ("dope-sheet", "Dope Sheet"),
                ("action-editor", "Action Editor"),
                ("shape-key-editor", "Shape Key Editor"),
            ]),
            signal(Some("dope-sheet".to_owned())),
        )
        .label("Selection")
        .placeholder("")
        .style(style().width(110)),
        spacer(),
        note("Click / Shift / Drag · Box select · G move · Del"),
        kivact::ui::button("Insert Key")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("diamond"))
            .on_press(move || {
                let mut next = keys.get();
                {
                    let keys = &mut next;
                    let id = keys.iter().map(|x| x.id).max().unwrap_or(0) + 1;
                    keys.push(
                        ui::Keyframe::new(id, 3, state.get().frame, "Inserted key")
                            .color(rgb(0x8b7cff)),
                    );
                }
                keys.set(next);
            }),
    ))
    .style(style().height(38).padding_x(10))
}

pub(super) fn picker_hsv(color: Signal<Color>, index: usize, label: &str) -> impl IntoView {
    ui::value_slider(
        ["H", "S", "V"][index],
        bind(
            move || {
                let (h, s, v) = ui::color_to_hsv(color.get(), &kivact::style::use_theme().get());
                [h / 360.0, s, v][index]
            },
            move |value| {
                let (h, s, v) =
                    ui::color_to_hsv(color.get_untracked(), &kivact::style::use_theme().get());
                let mut c = [h, s, v];
                c[index] = if index == 0 { value * 360.0 } else { value };
                color.set(ui::color_from_hsv(
                    c[0],
                    c[1],
                    c[2],
                    color
                        .get_untracked()
                        .components(&kivact::style::use_theme().get())[3],
                ));
            },
        ),
    )
    .name(label.to_owned())
    .range(0.0..=1.0)
    .step(0.001)
    .precision(3)
}

pub(super) fn picker_popover() -> Node {
    spec(
        "Swatch Field",
        "Click for a Popover",
        col((
            ui::property_row("Diffuse", color_swatch(signal(rgb(0xd24b3a)))),
            ui::property_row("Specular Tint", color_swatch(signal(rgb(0xf2e6c8)))),
            ui::property_row(
                "Fog (RGBA)",
                color_swatch(signal(rgb(0x5a8ccf).with_alpha(0.6))),
            ),
            ui::property_row("Track Color", color_swatch(signal(rgb(0x3fbf8f)))),
            note("Changes apply live while dragging; Escape restores the original color."),
        ))
        .role(kivact::view::Role::List),
    )
    .style(style().min_height(516))
}

pub(super) fn picker_inline(
    color: Signal<Color>,
    mode: Signal<Option<String>>,
    hex: Signal<String>,
) -> Node {
    spec(
        "Inline Picker",
        "Inline",
        col((
            ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                .wheel_size(200.0)
                .style(style().width(248))
                .test_id("creative-color-picker"),
            kivact::ui::radio_group(
                "Color model",
                keyed_options([("rgb", "RGB"), ("hsv", "HSV")]),
                mode,
            )
            .orientation(kivact::view::Orientation::Horizontal)
            .variant(kivact::ui::RadioGroupVariant::Segmented)
            .style(style().width(248)),
            dynamic(mode, move |mode| {
                if mode.as_deref() == Some("rgb") {
                    col((
                        picker_channel(color, 0, "R", "Picker R"),
                        picker_channel(color, 1, "G", "Picker G"),
                        picker_channel(color, 2, "B", "Picker B"),
                        picker_channel(color, 3, "A", "Picker Alpha"),
                    ))
                } else {
                    col((
                        picker_hsv(color, 0, "Picker Hue"),
                        picker_hsv(color, 1, "Picker Saturation"),
                        picker_hsv(color, 2, "Picker Value"),
                        picker_channel(color, 3, "A", "Picker Alpha"),
                    ))
                }
            }),
            ui::property_row(
                "Hex",
                kivact::ui::text_input(hex)
                    .label("HEX Color")
                    .style(style().font_family("Geist Mono")),
            ),
            row((
                node().style(move || {
                    style()
                        .height(24)
                        .flex_grow(1)
                        .border_radius(5)
                        .background(color.get())
                }),
                kivact::ui::button("Eyedropper preview")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .hide_label(true)
                    .icon(kivact::view::icon("color-picker"))
                    .style(style().size(24, 24).padding(0)),
            )),
            row([
                0x8b7cff, 0xa3e635, 0x3ad3e0, 0xff5c6c, 0xffb32c, 0x5aa9ff, 0xee8ac4, 0x3fc87d,
                0xffffff, 0x000000, 0x8a879b, 0x2c7cb0, 0xb05c42, 0xa6b076, 0x463e78, 0x9eb6a4,
                0x3f474a, 0xe0e0e0, 0x808080, 0x242528,
            ]
            .into_iter()
            .map(|c| {
                kivact::ui::button("")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .name(format!("Swatch #{c:06X}"))
                    .style(
                        style()
                            .size(20, 20)
                            .border_radius(10)
                            .padding(0)
                            .background(rgb(c)),
                    )
                    .on_press(move || color.set(rgb(c)))
            })
            .collect::<Vec<_>>())
            .style(style().width(248).flex_wrap(true).gap(5)),
        ))
        .role(kivact::view::Role::List)
        .style(style().width(248)),
    )
    .style(style().min_height(516))
}

pub(super) fn viewport_toolbar(
    shading: Signal<usize>,
    grid: Signal<bool>,
    reference: NodeRef,
    transform: Signal<kivact::interaction::ViewTransform>,
) -> Node {
    line((
        text("Application image"),
        spacer(),
        kivact::ui::toggle_button("Grid", grid),
        kivact::ui::radio_group(
            "Shading",
            keyed_options([
                ("wire", "Wire"),
                ("solid", "Solid"),
                ("material", "Material"),
                ("rendered", "Rendered"),
            ]),
            bind(
                move || {
                    Some(
                        match shading.get() {
                            0 => "wire",
                            1 => "solid",
                            2 => "material",
                            _ => "rendered",
                        }
                        .to_owned(),
                    )
                },
                move |key: Option<String>| {
                    shading.set(match key.as_deref() {
                        Some("wire") => 0,
                        Some("solid") => 1,
                        Some("material") => 2,
                        _ => 3,
                    })
                },
            ),
        )
        .orientation(kivact::view::Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
        .style(style().width(218)),
        kivact::ui::button("Frame All")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .on_press(move || {
                if let Some(r) = reference.rect().get() {
                    transform.set(kivact::interaction::ViewTransform::fit(
                        Size::new(640.0, 360.0),
                        Size::new(r.width, r.height),
                    ));
                }
            }),
    ))
}

pub(super) fn grading_scopes(
    samples: kivact::state::Value<Vec<Color>>,
    upper: Signal<Option<String>>,
    lower: Signal<Option<String>>,
) -> Node {
    col((
        kivact::ui::select(
            keyed_options([
                ("parade", "Parade"),
                ("waveform", "Waveform"),
                ("histogram", "Histogram"),
            ]),
            upper,
        )
        .label("Selection")
        .placeholder("")
        .style(style().width(110)),
        kivact::view::viewport(move || {
            images::histogram(
                &samples.get(),
                upper.get().as_deref().unwrap_or("histogram"),
            )
        })
        .name("Application histogram")
        .style(style().height(340)),
        kivact::ui::select(
            keyed_options([("vectorscope", "Vectorscope"), ("histogram", "Histogram")]),
            lower,
        )
        .label("Selection")
        .placeholder("")
        .style(style().width(110)),
        kivact::view::viewport(move || {
            images::histogram(
                &samples.get(),
                lower.get().as_deref().unwrap_or("histogram"),
            )
        })
        .name("Application channel histogram")
        .style(style().height(340)),
    ))
    .style(style().flex_grow(1).gap(6).padding(8))
}

/// One of the grading viewer's two display toggles over [`Grade`].
pub(super) fn grade_toggle(grade: Signal<Grade>, value: Grade) -> Signal<bool> {
    kivact::state::binding(
        move || grade.get() == value,
        move |on: bool| grade.set(if on { value } else { Grade::Graded }),
    )
}

pub(super) fn grading_viewer(
    worker_image: Signal<Option<kivact::draw::Pixels>>,
    bypass: Signal<bool>,
    wipe: Signal<bool>,
) -> Node {
    col((
        line((
            text("Viewer · 01:00:12:08").style(style().font_family("Geist Mono").font_size(10)),
            spacer(),
            kivact::ui::toggle_button("Bypass", bypass),
            kivact::ui::toggle_button("Wipe", wipe),
        )),
        node()
            .child(
                kivact::view::viewport(move || worker_image.get().unwrap_or_else(empty_pixels))
                    .name("Worker rendered gradient")
                    .style(style().size(720, 405).max_width(pct(100))),
            )
            .style(
                style()
                    .flex_grow(1)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .padding(16),
            ),
    ))
    .style(style().width(pct(56)).padding(10))
}

pub(super) fn scene_sky(d: &mut kivact::draw::Draw, w: f64, horizon: f64) {
    for y in 0..64 {
        let t = y as f64 / 63.0;
        let sky = if t < 0.7 {
            rgb(0x153565).mix(rgb(0xf89973), t / 0.7, &kivact::style::use_theme().get())
        } else {
            rgb(0xf89973).mix(
                rgb(0xffd481),
                (t - 0.7) / 0.3,
                &kivact::style::use_theme().get(),
            )
        };
        d.fill_rect(
            Rect::new(0.0, y as f64 * horizon / 64.0, w, horizon / 64.0 + 1.0),
            sky,
        );
    }
}
