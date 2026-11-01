//! Role: Editors and input controls for creative editing.
use crate::creative::audio_values::TransportState;

use super::common::{bind, col, keyed_options, line, note, spec, specs};
use super::controls::num;
use super::controls::tool_palette;
use super::editor_images as images;
use kivact::Binding;
use kivact::draw::Path;
use kivact::draw::{Point, Rect, Size};
use kivact::prelude::{
    IntoView, Node, NodeBuilder, NodeRef, Role, Signal, dynamic, memo, node, rgb, row, signal,
    spacer, text,
};
use kivact::style::Position;
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::ui::MenuEntry;
use kivact_ui_creative as ui;

fn playhead(state: Signal<TransportState>) -> Binding<f64> {
    bind(
        move || state.get().frame,
        move |v| state.update(|s| s.frame = v.max(0.0)),
    )
}

pub(super) fn animation(state: Signal<TransportState>) -> impl IntoView {
    let curve_specs: [&[(f32, f32)]; 3] = [
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
                    let mut key = ui::CurveKey::new(
                        (channel * 8 + i) as u64 + 1,
                        f64::from(frame),
                        f64::from(value),
                    );
                    key.in_tangent = ui::CurveTangent::new(-f64::from(dl), -f64::from(slope * dl));
                    key.out_tangent = ui::CurveTangent::new(f64::from(dr), f64::from(slope * dr));
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
    let graph = signal(curves);
    let graph_selected = signal(vec![18_u64]);
    let (keys, selected) = super::animation_model::bindings(graph, graph_selected);
    let initial_keys = signal(None);
    let channels = [signal(true), signal(true), signal(true)];
    let zoom = signal(6.8);
    let graph_view = signal(ui::GraphView::new((-20.0, 200.0), (-2.0, 3.0)));
    col((
        line((
            text("Dope Sheet").style(style().weight(600)),
            kivact::ui::select(
                "Selection",
                keyed_options([
                    ("dope-sheet", "Dope Sheet"),
                    ("action-editor", "Action Editor"),
                    ("shape-key-editor", "Shape Key Editor"),
                ]),
                signal(Some("dope-sheet".to_owned())),
            )
            .placeholder("")
            .style(style().width(110)),
            spacer(),
            note("Click / Shift / Drag · Box select · G move · Del"),
            kivact::ui::button("Insert Key")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .icon(kivact::control::icon("diamond"))
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
        .style(style().height(38).padding_x(10)),
        ui::dope_sheet(keys, {
            let mut value = ui::TimeState::new();
            value.pan = (signal(Point::new(12.0, 0.0))).into();
            value.selection = selected;
            value.playhead = playhead(state);
            value.zoom = zoom.into();
            value
        })
        .on_edit(move |event| match event {
            ui::EditorEvent::Begin => initial_keys.set(Some(keys.get())),
            ui::EditorEvent::Request(request) => {
                let mut next = keys.get();
                apply_time(&mut next, &request);
                keys.set(next);
            }
            ui::EditorEvent::Commit => initial_keys.set(None),
            ui::EditorEvent::Cancel => {
                if let Some(initial) = initial_keys.get() {
                    keys.set(initial);
                }
                initial_keys.set(None);
            }
            // Variants added later fall back to being ignored.
            _ => {}
        })
        .zoom_range(0.25, 10.0)
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
        .test_id("creative-dopesheet"),
        line((
            text("Graph Editor").style(style().weight(600)),
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
                                        // Variants added later fall back to bezier.
                                        _ => "bezier",
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
            .variant(ui::Segmented)
            .style(style().width(166)),
            kivact::ui::toggle("X", channels[0]),
            kivact::ui::toggle("Y", channels[1]),
            kivact::ui::toggle("Z", channels[2]),
            kivact::ui::button("Frame All  Home")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || graph_view.set(ui::GraphView::new((-20.0, 200.0), (-2.0, 3.0)))),
            spacer(),
            note("Z Location · f 40 · 1.200 · bezier"),
        ))
        .style(style().height(38).padding_x(10)),
        ui::graph_editor(graph, {
            let mut value = ui::GraphState::new();
            value.view = graph_view.into();
            value.selection = graph_selected.into();
            value.playhead = playhead(state);
            value
        })
        .on_edit(super::animation_model::graph_edits(graph))
        .lanes(move || {
            channels
                .iter()
                .enumerate()
                .filter(|(_, visible)| visible.get())
                .map(|(id, _)| {
                    ui::Lane::new(id as u64, ["X Location", "Y Location", "Z Location"][id])
                })
                .collect::<Vec<_>>()
        })
        .style(style().height(320))
        .test_id("creative-graph"),
    ))
    .style(style().gap(0))
}

pub(super) fn nodes() -> impl IntoView {
    let data = signal(vec![
        super::node_model::editor_node(1, "Render Layers", 100.0, 28.0, 0xff5c6c),
        super::node_model::editor_node(2, "Blur", 310.0, 20.0, 0x8b7cff),
        super::node_model::editor_node(3, "Image", 100.0, 230.0, 0xff5c6c),
        super::node_model::editor_node(4, "Color Balance", 310.0, 200.0, 0xffd25a),
        super::node_model::editor_node(5, "Keying", 310.0, 390.0, 0x34d399),
        super::node_model::editor_node(6, "Mix", 535.0, 150.0, 0xffd25a),
        super::node_model::editor_node(7, "Viewer", 755.0, 20.0, 0xff5c6c),
        super::node_model::editor_node(8, "Composite", 755.0, 260.0, 0xff5c6c),
    ]);
    let connections = signal(vec![
        ui::NodeLink::new(1, 0, 2, 0),
        ui::NodeLink::new(1, 2, 2, 1),
        ui::NodeLink::new(1, 1, 6, 0),
        ui::NodeLink::new(2, 0, 6, 1),
        ui::NodeLink::new(3, 0, 4, 1),
        ui::NodeLink::new(3, 0, 5, 0),
        ui::NodeLink::new(4, 0, 6, 2),
        ui::NodeLink::new(6, 0, 7, 0),
        ui::NodeLink::new(6, 0, 8, 0),
    ]);
    let selected = signal(Vec::<u64>::new());
    let transform = signal(kivact::interaction::ViewTransform::new(
        0.85,
        Point::new(300.0, 0.0),
    ));
    let mut node_state = ui::NodeState::new();
    node_state.selection = selected.into();
    node_state.transform = transform.into();
    col((node().style(style().position(Position::Relative).height(480)).child((
        ui::node_editor(data,connections, node_state).can_connect(move |link| super::node_model::can_connect(&data.get_untracked(), link)).on_edit(super::node_model::edits(data, connections)).style(style().height(480)).test_id("creative-node-editor"),
        line((kivact::ui::select("Selection", keyed_options([("compositor", "Compositor"), ("geometry-nodes", "Geometry Nodes"), ("shader-editor", "Shader Editor")]), signal(Some("compositor".to_owned()))).placeholder("").style(style().width(136)),kivact::ui::toggle("Use Nodes", signal(true)))).style(style().position(Position::Absolute).bottom(8).left(8)),
        line((kivact::ui::menu("Add", ["Image", "Blur", "Mix", "Color Balance", "Keying"].into_iter().map(|label| MenuEntry::item(label, label)).collect::<Vec<_>>(), move |label: &'static str| super::node_model::add(data, label)).offset(0.0).attach(kivact::ui::button("Add").variant(kivact::ui::ButtonVariant::Secondary).trailing_icon(kivact::control::icon("chevron-down")).into_node()),kivact::ui::button("Frame All").variant(kivact::ui::ButtonVariant::Secondary).on_press(move || transform.set(kivact::interaction::ViewTransform::new(0.85, Point::new(300.0,0.0)))))).style(style().position(Position::Absolute).top(8).right(8)),
    )), row((note(move || format!("{} nodes · {} links · {:.0}%", data.with(Vec::len), connections.with(Vec::len), transform.get().scale * 100.0)), spacer(), note("ヘッダーをドラッグ / 出力 → 入力で接続 / Del: 選択削除 / Ctrl-wheel: zoom / Middle: pan"))).style(style().padding(12))))
}

pub(super) fn grading() -> impl IntoView {
    let upper = signal(Some("parade".to_owned()));
    let lower = signal(Some("vectorscope".to_owned()));
    let worker_image = images::worker_gradient();
    let bypass = signal(false);
    let wipe = signal(false);
    let wheels = [
        signal(rgb(0x7a7a7a)),
        signal(rgb(0x909090)),
        signal(rgb(0xb4b4b4)),
        signal(rgb(0x8b7cff)),
    ];
    let samples = memo(move || {
        let tint = wheels[0].get().components();
        (0..4096)
            .map(|i| {
                let x = (i % 128) as f32 / 127.0;
                let y = (i / 128) as f32 / 31.0;
                let noise = ((i as u64 * 1664525 + 1013904223) % 65536) as f32 / 65535.0 - 0.5;
                let c = if y < 0.58 {
                    mix(rgb(0x183961), rgb(0xffb272), y / 0.58)
                } else {
                    mix(rgb(0x4e94ae), rgb(0x053957), (y - 0.58) / 0.42)
                };
                let c = if x < 0.22 && y > 0.45 {
                    rgb(0x279837)
                } else {
                    c
                };
                let mut c = c.components();
                if !bypass.get() {
                    for k in 0..3 {
                        c[k] = (c[k] + (tint[k] - 0.478) * 0.3 + noise * 0.10).clamp(0.0, 1.0);
                    }
                }
                Color::from_rgba_f32(c[0], c[1], c[2], 1.0)
            })
            .collect::<Vec<_>>()
    });
    col((
        row((
            col((
                line((
                    text("Viewer · 01:00:12:08")
                        .style(style().font_family("Geist Mono").font_size(10)),
                    spacer(),
                    kivact::ui::toggle("Bypass", bypass),
                    kivact::ui::toggle("Wipe", wipe),
                )),
                node()
                    .child(
                        ui::viewport_surface(worker_image)
                            .name("Worker rendered gradient")
                            .style(style().size(720, 405).max_width(pct(100))),
                    )
                    .style(
                        style()
                            .grow(1)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .padding(16),
                    ),
            ))
            .style(style().width(pct(56)).padding(10)),
            col((
                kivact::ui::select(
                    "Selection",
                    keyed_options([
                        ("parade", "Parade"),
                        ("waveform", "Waveform"),
                        ("histogram", "Histogram"),
                    ]),
                    upper,
                )
                .placeholder("")
                .style(style().width(110)),
                ui::viewport_surface(move || {
                    Some(images::histogram(
                        &samples.get(),
                        upper.get().as_deref().unwrap_or("histogram"),
                    ))
                })
                .name("Application histogram")
                .style(style().height(340)),
                kivact::ui::select(
                    "Selection",
                    keyed_options([("vectorscope", "Vectorscope"), ("histogram", "Histogram")]),
                    lower,
                )
                .placeholder("")
                .style(style().width(110)),
                ui::viewport_surface(move || {
                    Some(images::histogram(
                        &samples.get(),
                        lower.get().as_deref().unwrap_or("histogram"),
                    ))
                })
                .name("Application channel histogram")
                .style(style().height(340)),
            ))
            .style(style().grow(1).gap(6).padding(8)),
        ))
        .style(style().gap(1).height(772)),
        row(["Lift", "Gamma", "Gain", "Offset"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                col((
                    line((
                        text(label).style(style().weight(600).grow(1)),
                        kivact::ui::button("Reset")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || wheels[i].set(rgb(0x808080))),
                    )),
                    ui::color_wheel(wheels[i], signal(ui::ColorModel::Rgb)).wheel_size(150.0),
                    line((
                        channel(wheels[i], 0, "Grade R"),
                        channel(wheels[i], 1, "Grade G"),
                        channel(wheels[i], 2, "Grade B"),
                    )),
                ))
                .style(style().width(pct(25)).padding(12))
                .style(
                    style()
                        .border_top(1, ui::Color::Hair)
                        .border_right(1, ui::Color::Hair),
                )
            })
            .collect::<Vec<_>>()),
        line((
            num("Contrast", 1.0, 0.0, 4.0, 3, ""),
            num("Pivot", 0.435, 0.0, 1.0, 3, ""),
            num("Saturation", 1.0, 0.0, 2.0, 3, ""),
            num("Temperature", 0.0, -100.0, 100.0, 0, ""),
            num("Tint", 0.0, -100.0, 100.0, 0, ""),
        ))
        .style(style().padding(12)),
    ))
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
            for y in 0..64 {
                let t = y as f32 / 63.0;
                let sky = if t < 0.7 {
                    mix(rgb(0x153565), rgb(0xf89973), t / 0.7)
                } else {
                    mix(rgb(0xf89973), rgb(0xffd481), (t - 0.7) / 0.3)
                };
                d.fill_rect(
                    Rect::new(0.0, y as f32 * horizon / 64.0, w, horizon / 64.0 + 1.0),
                    sky,
                );
            }
            for i in 0..3 {
                let mut path = Path::new().move_to(Point::new(0.0, horizon));
                for x in 0..26 {
                    let px = x as f32 * w / 25.0;
                    let ridge =
                        horizon - (0.13 + (x as f32 * 1.9 + i as f32).sin().abs() * 0.12) * h;
                    path = path.line_to(Point::new(px, ridge + i as f32 * h * 0.04));
                }
                d.fill_path(
                    &path.line_to(Point::new(w, horizon)).close(),
                    [rgb(0x565570), rgb(0x405b61), rgb(0x194c44)][i],
                );
            }
            for y in 0..32 {
                let t = y as f32 / 31.0;
                d.fill_rect(
                    Rect::new(
                        0.0,
                        horizon + t * (h - horizon),
                        w,
                        (h - horizon) / 31.0 + 1.0,
                    ),
                    mix(rgb(0x4e94ae), rgb(0x053957), t),
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
                let x = i as f32 * w * 0.025;
                let top = h * (0.43 + (i as f32 * 2.4).sin().abs() * 0.12);
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
fn mix(a: Color, b: Color, t: f32) -> Color {
    let a = a.components();
    let b = b.components();
    Color::from_rgba_f32(
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        1.0,
    )
}

pub(super) fn picker() -> impl IntoView {
    let color = signal(rgb(0x8b7cff));
    let mode = signal(Some("rgb".to_owned()));
    let hex = bind(
        move || {
            let c = color.get().components();
            format!(
                "#{:02X}{:02X}{:02X}",
                (c[0] * 255.0).round() as u8,
                (c[1] * 255.0).round() as u8,
                (c[2] * 255.0).round() as u8
            )
        },
        move |value: String| {
            if let Ok(v) = u32::from_str_radix(value.trim().trim_start_matches('#'), 16)
                && v <= 0xffffff
            {
                let a = color.get_untracked().components()[3];
                color.set(rgb(v).with_alpha(a));
            }
        },
    );
    let hsv = move |index: usize, label: &str| {
        ui::value_slider(
            ["H", "S", "V"][index],
            bind(
                move || {
                    let (h, s, v) = ui::color_to_hsv(color.get());
                    [h / 360.0, s, v][index]
                },
                move |value| {
                    let (h, s, v) = ui::color_to_hsv(color.get_untracked());
                    let mut c = [h, s, v];
                    c[index] = if index == 0 { value * 360.0 } else { value };
                    color.set(ui::color_from_hsv(
                        c[0],
                        c[1],
                        c[2],
                        color.get_untracked().components()[3],
                    ));
                },
            ),
        )
        .name(label.to_owned())
        .range(0.0, 1.0)
        .step(0.001)
        .precision(3)
    };
    specs((
        spec(
            "Inline Picker",
            "インライン",
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
                .variant(ui::Segmented)
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
                            hsv(0, "Picker Hue"),
                            hsv(1, "Picker Saturation"),
                            hsv(2, "Picker Value"),
                            picker_channel(color, 3, "A", "Picker Alpha"),
                        ))
                    }
                }),
                ui::property_row(
                    "Hex",
                    kivact::ui::text_input("HEX Color", hex)
                        .style(style().font_family("Geist Mono")),
                ),
                row((
                    node().style(move || {
                        style().height(24).grow(1).radius(5).background(color.get())
                    }),
                    kivact::ui::button("Eyedropper preview")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .hide_label(true)
                        .icon(kivact::control::icon("color-picker"))
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
                                .radius(10)
                                .padding(0)
                                .background(rgb(c)),
                        )
                        .on_press(move || color.set(rgb(c)))
                })
                .collect::<Vec<_>>())
                .style(style().width(248).wrap(true).gap(5)),
            ))
            .style(style().width(248)),
        )
        .style(style().min_height(516)),
        spec(
            "Swatch Field",
            "クリックでポップオーバー",
            col((
                ui::property_row("Diffuse", color_swatch(signal(rgb(0xd24b3a)))),
                ui::property_row("Specular Tint", color_swatch(signal(rgb(0xf2e6c8)))),
                ui::property_row(
                    "Fog (RGBA)",
                    color_swatch(signal(rgb(0x5a8ccf).with_alpha(0.6))),
                ),
                ui::property_row("Track Color", color_swatch(signal(rgb(0x3fbf8f)))),
                note("ドラッグ中はリアルタイム反映、Escape で元の色に戻します。"),
            )),
        )
        .style(style().min_height(516)),
    ))
    .style(style().align_items(Align::Start))
}
fn picker_channel(color: Signal<Color>, index: usize, label: &str, name: &str) -> ui::Numeric {
    ui::value_slider(
        label.to_owned(),
        bind(
            move || f64::from(color.get().components()[index]),
            move |v| {
                let mut c = color.get_untracked().components();
                c[index] = v as f32;
                color.set(Color::from_rgba_f32(c[0], c[1], c[2], c[3]));
            },
        ),
    )
    .name(name.to_owned())
    .range(0.0, 1.0)
    .step(0.001)
    .precision(3)
    .style(style().width(248))
    .accent_color(rgb([0xff5c6c, 0xa3e635, 0x48c8ff, 0x8b7cff][index]))
}
fn channel(color: Signal<Color>, index: usize, label: &str) -> ui::Numeric {
    ui::number(
        label.to_string(),
        bind(
            move || f64::from(color.get().components()[index]),
            move |v| {
                let mut c = color.get_untracked().components();
                c[index] = v as f32;
                color.set(Color::from_rgba_f32(c[0], c[1], c[2], c[3]));
            },
        ),
    )
    .range(0.0, 1.0)
    .step(0.01)
    .precision(3)
    .style(style().grow(1))
}
fn color_swatch(color: Signal<Color>) -> impl IntoView {
    ui::color_popup("Edit color", color).style(style().width(130))
}

pub(super) fn viewport(
    shading: Signal<usize>,
    grid: Signal<bool>,
    reference: NodeRef,
    pointer: Signal<Point>,
) -> impl IntoView {
    let transform = signal(kivact::interaction::ViewTransform::default());
    let frame = memo(move || Some(images::gradient(640, 360, shading.get(), grid.get())));
    col((
        line((
            text("Application image"),
            spacer(),
            kivact::ui::toggle("Grid", grid),
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
            .variant(ui::Segmented)
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
        .style(style().height(38).padding_x(10)),
        row((
            tool_palette(true).style(style().width(42).padding(6)),
            ui::viewport_surface(frame)
                .transform(transform)
                .on_pointer(move |event: ui::SurfacePointer| pointer.set(event.position))
                .overlay(|draw, _| {
                    draw.stroke_rect(Rect::new(220.0, 110.0, 200.0, 140.0), 1.0, rgb(0xffd25a));
                })
                .regions(vec![ui::SurfaceRegion::new(
                    1,
                    "Sample area",
                    Role::Image,
                    Rect::new(220.0, 110.0, 200.0, 140.0),
                )])
                .style(style().height(392).grow(1))
                .node_ref(reference)
                .test_id("creative-viewport"),
        ))
        .style(style().gap(4)),
        row((
            note("CPU gradient · image coordinates"),
            spacer(),
            note("Middle drag: pan · Wheel: zoom · 0: fit"),
        ))
        .style(style().padding(12)),
    ))
}

fn apply_time(keys: &mut Vec<ui::Keyframe>, request: &ui::TimeRequest) {
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
        // Variants added later fall back to being ignored.
        _ => {}
    }
}
