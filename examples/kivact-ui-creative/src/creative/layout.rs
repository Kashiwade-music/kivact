//! Role: Layout and property panel for creative application.
use crate::creative::audio_values::TransportState;
use kivact::state::IntoBinding;

use super::common::{bind, col, keyed_options, line, note, spec, specs};
use super::controls::chip;
use super::controls::num;
use super::controls::slide;
use kivact::geometry::Point;
use kivact::prelude::{
    IntoView, Key, Node, NodeBuilder, Orientation, Signal, column, dynamic, grid, node, rgb, row,
    scroll, signal, spacer, text,
};
use kivact::style::{Align, Color, pct, style};
use kivact::style::{GridPlacement, Track};
use kivact::view::Handled;
use kivact_ui_creative as ui;

pub(super) fn properties() -> impl IntoView {
    let tab = signal(2_usize);
    let display_color = signal(rgb(0xffffff));
    let display_name = signal(false);
    let display_axes = signal(true);
    let display_wireframe = signal(false);
    let display_as = signal(Some("textured".to_owned()));
    row((
        column(
            ["camera", "settings", "box", "link", "eye"]
                .into_iter()
                .enumerate()
                .map(|(i, glyph)| {
                    kivact::ui::button(format!("Inspector {i}"))
                        .hide_label(true)
                        .icon(kivact::view::icon(glyph))
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .with_node(|node| node.selected(move || tab.get() == i))
                        .on_press(move || tab.set(i))
                })
                .collect::<Vec<_>>(),
        )
        .style(style().gap(8).padding(5).width(38)),
        scroll(col((
            text("◇ Cube  ›  Object").style(style().padding_y(8)),
            ui::panel_group(
                signal(vec![
                    ui::InspectorPanel::new(1, "Transform"),
                    ui::InspectorPanel::new(2, "Delta Transform").collapsed(),
                    ui::InspectorPanel::new(3, "Relations"),
                    ui::InspectorPanel::new(4, "Visibility"),
                    ui::InspectorPanel::new(5, "Modifiers"),
                ]),
                move |panel| match panel.get_untracked().id {
                    1 => col((
                        ui::property_row("Location X", num("X", 0.0, -100.0, 100.0, 3, " m"))
                            .test_id("creative-property-location-x"),
                        ui::property_row("Y", num("Y", 0.0, -100.0, 100.0, 3, " m"))
                            .test_id("creative-property-location-y"),
                        ui::property_row("Z", num("Z", 1.0, -100.0, 100.0, 3, " m"))
                            .test_id("axis-label-z"),
                        ui::property_row("Rotation X", num("X", 0.0, -360.0, 360.0, 1, "°")),
                        ui::property_row("Y", num("Y", 0.0, -360.0, 360.0, 1, "°")),
                        ui::property_row("Z", num("Z", 45.0, -360.0, 360.0, 1, "°"))
                            .test_id("axis-label-z"),
                        ui::property_row(
                            "Mode",
                            kivact::ui::select(
                                "Rotation Mode",
                                keyed_options([
                                    ("xyz-euler", "XYZ Euler"),
                                    ("quaternion", "Quaternion"),
                                    ("axis-angle", "Axis Angle"),
                                ]),
                                signal(Some("xyz-euler".to_owned())),
                            )
                            .placeholder("")
                            .name("Rotation Mode")
                            .style(style().width(pct(100))),
                        ),
                        ui::property_row("Scale X", num("X", 1.0, 0.0, 10.0, 3, "")),
                        ui::property_row("Y", num("Y", 1.0, 0.0, 10.0, 3, "")),
                        ui::property_row("Z", num("Z", 1.0, 0.0, 10.0, 3, ""))
                            .test_id("axis-label-z"),
                    ))
                    .role(kivact::view::Role::List)
                    .style(style().gap(3)),
                    2 => col((
                        num("X", 0.0, -100.0, 100.0, 3, " m"),
                        num("Y", 0.0, -100.0, 100.0, 3, " m"),
                    ))
                    .style(style().gap(3)),
                    3 => col((
                        ui::property_row(
                            "Parent",
                            kivact::ui::select(
                                "Selection",
                                keyed_options([
                                    ("none", "None"),
                                    ("null-1", "Null 1"),
                                    ("camera", "Camera"),
                                ]),
                                signal(Some("none".to_owned())),
                            )
                            .placeholder(""),
                        ),
                        ui::property_row("Pass Index", num("", 0.0, 0.0, 999.0, 0, "")),
                    ))
                    .role(kivact::view::Role::List)
                    .style(style().gap(3)),
                    4 => col((
                        kivact::ui::checkbox("Show in Viewport", signal(true)),
                        kivact::ui::checkbox("Show in Render", signal(true)),
                    ))
                    .style(style().gap(3)),
                    _ => col((
                        kivact::ui::checkbox("Bevel", signal(true)),
                        slide("Width", 0.12),
                        kivact::ui::checkbox("Subdivision", signal(true)),
                        num("Levels", 2.0, 0.0, 6.0, 0, ""),
                    ))
                    .style(style().gap(3)),
                },
            ),
        )))
        .style(style().width(278).height(556).padding_x(5)),
        spec(
            "Panel Behaviour",
            "パネル操作",
            col((
                note("ヘッダーをクリックで開閉。内部値を保持したまま折りたたみます。"),
                kivact::ui::disclosure("Viewport Display", signal(true)).content(move || {
                    col((
                        kivact::ui::checkbox("Name", display_name),
                        kivact::ui::checkbox("Axes", display_axes),
                        kivact::ui::checkbox("Wireframe", display_wireframe),
                        ui::property_row(
                            "Display As",
                            kivact::ui::select(
                                "Selection",
                                keyed_options([
                                    ("textured", "Textured"),
                                    ("solid", "Solid"),
                                    ("wire", "Wire"),
                                ]),
                                display_as,
                            )
                            .placeholder(""),
                        ),
                        ui::property_row("Color", ui::color_popup("Viewport color", display_color))
                            .test_id("creative-property-display-color"),
                    ))
                    .role(kivact::view::Role::List)
                }),
                kivact::ui::alert("Property Editing")
                    .tone(kivact::interaction::Tone::Info)
                    .with_node(|node| {
                        node.role(kivact::view::Role::Status)
                            .live_region(kivact::view::LiveRegion::Polite)
                    })
                    .description("ドラッグ・ホイール・直接入力・キー操作で編集。")
                    .variant(ui::NoticeVariant)
                    .dismiss_content(kivact::view::icon("x"))
                    .dismiss_label(kivact::i18n::message(
                        "creative.notice.dismiss",
                        "Dismiss notification",
                    ))
                    .icon(kivact::view::icon("info")),
            )),
        )
        .style(style().width(316)),
    ))
    .style(style().align_items(Align::Start))
}

pub(super) fn docking() -> impl IntoView {
    let document=signal("import kivact as kv\n\nscene = kv.context.scene\nfor obj in scene.objects:\n    if obj.type == \"MESH\":\n        obj.modifiers.new(\"Bevel\", \"BEVEL\")".to_string());
    let command = signal(String::new());
    let editor_sizes = signal(vec![0.55f64, 0.45]);
    let output = signal("KIVACT Console 0.1 — type help\n".to_string());
    let layout = signal(ui::DockLayout::new(ui::DockNode::split(
        ui::DockAxis::Horizontal,
        [
            (
                0.34,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("info", "Info").icon("info"),
                    ui::DockTab::new("console", "Console"),
                ]),
            ),
            (
                0.66,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("text", "Text Editor"),
                    ui::DockTab::new("image", "Image Viewer").icon("eye"),
                    ui::DockTab::new("properties", "Properties"),
                ]),
            ),
        ],
    )));
    let dock = ui::dock(layout, move |panel: &'static str| match panel {
        "info" => scroll(
            col((
                note("01:54:41  › File · Open: Shot_010_Comp_v012.kvp"),
                text("Loaded 128 clips, 14 compositions (3 media offline)")
                    .style(style().font_size(11)),
                chip("Font 'Inter Display' missing — substituted", rgb(0xfbbf24)),
                chip("GPU: compute device initialised (2 devices)", rgb(0x48c8ff)),
                chip("Shader compile error: Metal.002 line 42", rgb(0xff5c6c)),
                note("Proxy cache verified (4.1 GB)"),
            ))
            .style(style().padding(8).gap(6)),
        )
        .into_view(),
        "text" => kivact::ui::split(
            editor_sizes,
            (
                kivact::ui::text_area("Document source", document)
                    .style(style().width(pct(100)).height(pct(100))),
                ui::panel("Console", console_view(output, command)).style(style().height(pct(100))),
            ),
        )
        .orientation(Orientation::Vertical)
        .min_size(0.15)
        .into_view(),
        "console" => scroll(col((
            kivact::ui::text_area("Console output", output)
                .read_only(true)
                .style(style().height(200)),
            kivact::ui::text_input("Console command", command).on_key(move |e| {
                if e.key() == Key::Enter {
                    let cmd = command.get();
                    let answer = if cmd == "help" {
                        "help · clear · arithmetic expressions".into()
                    } else {
                        ui::evaluate_expression(&cmd).map_or_else(
                            || "SyntaxError: invalid expression".into(),
                            |v| v.to_string(),
                        )
                    };
                    if cmd == "clear" {
                        output.set(String::new())
                    } else {
                        output.update(|v| v.push_str(&format!("\n>>> {cmd}\n{answer}")))
                    }
                    command.set(String::new());
                    Handled::Yes
                } else {
                    Handled::No
                }
            }),
        )))
        .into_view(),
        "image" => scroll(super::editors::scene().style(style().height(230))).into_view(),
        "properties" => scroll(
            col((
                ui::property_row(
                    "Resolution X",
                    num("Resolution X", 1920.0, 1.0, 16384.0, 0, " px"),
                ),
                ui::property_row("Y", num("Resolution Y", 1080.0, 1.0, 16384.0, 0, " px")),
                ui::property_row(
                    "Frame Rate",
                    kivact::ui::select(
                        "Selection",
                        keyed_options([("24", "24"), ("25", "25"), ("30", "30"), ("60", "60")]),
                        signal(Some("24".to_owned())),
                    )
                    .placeholder(""),
                ),
            ))
            .role(kivact::view::Role::List)
            .style(style().padding(12)),
        )
        .into_view(),
        _ => node().into_view(),
    })
    .test_id("creative-dock-workspace")
    .style(style().height(390));
    let active = signal("comp".to_owned());
    let tabs = signal(keyed_options([
        ("comp", "Comp 1"),
        ("main-edit", "Main_Edit ●"),
        ("shot-010", "Shot_010 ●"),
        ("titles", "Titles"),
    ]));
    let next_document = kivact::state::var(5_u64);
    col((
        dock.style(style().margin(6)),
        specs((
            spec(
                "Document Tabs",
                "drag to reorder",
                col((
                    ui::document_tabs("Documents", tabs, active)
                        .on_reorder(move |(a, b): (String, String)| {
                            tabs.update(|items| {
                                if let Some(a) = items.iter().position(|item| *item.key() == a)
                                    && let Some(b) = items.iter().position(|item| *item.key() == b)
                                {
                                    items.swap(a, b);
                                }
                            });
                        })
                        .on_close(move |key: String| {
                            tabs.update(|items| items.retain(|item| *item.key() != key));
                        }),
                    grid(
                        [
                            ("DOCUMENT", "Comp 1"),
                            ("STATE", "Saved"),
                            ("DURATION", "00:00:10:00"),
                            ("FORMAT", "1920×1080 · 24 fps"),
                        ]
                        .into_iter()
                        .flat_map(|(label, value)| {
                            [
                                note(label).style(style().font_size(9.5)),
                                text(value).style(style().font_size(10.5).weight(600)),
                            ]
                        })
                        .collect::<Vec<_>>(),
                    )
                    .style(
                        style()
                            .grid_template_columns((Track::minmax(64, 64), Track::fr(1)))
                            .gap(4)
                            .margin_top(12),
                    ),
                    kivact::ui::button("New Composition")
                        .hide_label(true)
                        .icon(kivact::view::icon("plus"))
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .on_press(move || {
                            let id = next_document.with(|id| *id);
                            let next = id.checked_add(1).ok_or_else(|| {
                                kivact::UiError::msg("Document identity limit reached")
                            })?;
                            next_document.set(next);
                            let key = format!("created-document-{id}");
                            tabs.update(|items| {
                                items.push(kivact::interaction::ListItem::new(
                                    key.clone(),
                                    format!("Untitled {id}"),
                                ));
                            });
                            active.set(key);
                            Ok::<_, kivact::UiError>(())
                        })
                        .style(style().align_self(Align::End)),
                )),
            ),
            spec("Panel Tab Group", "panel tabs", effects_panel()),
        )),
    ))
}

fn console_view(output: Signal<String>, command: Signal<String>) -> Node {
    col((
        kivact::ui::text_area("Console output", output)
            .read_only(true)
            .style(style().height(96)),
        kivact::ui::text_input("Console command", command)
            .on_key(move |e| {
                if e.key() != Key::Enter {
                    return Handled::No;
                }
                let cmd = command.get();
                if cmd == "clear" {
                    output.set(String::new());
                } else {
                    let answer = if cmd == "help" {
                        "help · clear · arithmetic expressions".to_owned()
                    } else {
                        ui::evaluate_expression(&cmd).map_or_else(
                            || "SyntaxError: invalid expression".into(),
                            |v| v.to_string(),
                        )
                    };
                    output.update(|v| v.push_str(&format!("\n>>> {cmd}\n{answer}")));
                }
                command.set(String::new());
                Handled::Yes
            })
            .style(style().height(24)),
    ))
    .style(style().gap(0))
}
fn effects_panel() -> Node {
    let tab = signal("effects".to_owned());
    let query = signal(String::new());
    let effects = signal(vec![
        ui::TreeItem::new(1, None, "Blur & Sharpen", "folder"),
        ui::TreeItem::new(2, None, "Color Correction", "folder"),
        ui::TreeItem::new(3, Some(2), "Curves", "adjustments"),
        ui::TreeItem::new(4, Some(2), "Levels", "adjustments"),
        ui::TreeItem::new(5, Some(2), "Hue / Saturation", "adjustments"),
        ui::TreeItem::new(6, Some(2), "Color Balance", "adjustments"),
        ui::TreeItem::new(7, Some(2), "Exposure", "adjustments"),
        ui::TreeItem::new(8, None, "Distort", "folder"),
        ui::TreeItem::new(9, None, "Generate", "folder"),
        ui::TreeItem::new(10, None, "Keying", "folder"),
    ]);
    col((kivact::ui::tabs(
        "Effects panel",
        keyed_options([
            ("effects", "Effects & Presets"),
            ("character", "Character"),
            ("align", "Align"),
        ]),
        tab,
    )
    .content(move |key| {
        if key == "effects" {
            col((
                kivact::ui::text_input("Find effect", query).placeholder("Find effect"),
                super::layout_reducers::outliner(
                    effects,
                    ui::OutlinerState {
                        expanded: (signal(vec![2, 5])).into_binding(),
                        ..ui::OutlinerState::new(Vec::new())
                    },
                )
                .filter(query)
                .restrictions(false),
            ))
        } else if key == "character" {
            col((
                ui::property_row(
                    "Font",
                    kivact::ui::select(
                        "Selection",
                        keyed_options([("geist", "Geist"), ("geist-mono", "Geist Mono")]),
                        signal(Some("geist".to_owned())),
                    )
                    .placeholder(""),
                ),
                ui::property_row("Size", num("Font size", 12.0, 1.0, 300.0, 0, " pt")),
            ))
            .role(kivact::view::Role::List)
        } else {
            line((
                kivact::ui::button("Left").variant(kivact::ui::ButtonVariant::Secondary),
                kivact::ui::button("Center").variant(kivact::ui::ButtonVariant::Secondary),
                kivact::ui::button("Right").variant(kivact::ui::ButtonVariant::Secondary),
            ))
        }
    }),))
    .style(style().min_height(300).gap(8))
}
pub(super) fn outliner() -> impl IntoView {
    let selected = signal(vec![4u64]);
    let filter = signal(String::new());
    let renaming = signal(None);
    let nodes = signal(vec![
        ui::TreeItem::new(1, None, "Scene Collection", "box"),
        ui::TreeItem::new(2, Some(1), "Hero", "folder"),
        ui::TreeItem::new(3, Some(2), "Camera", "camera"),
        ui::TreeItem::new(23, Some(3), "Camera.001", "camera"),
        ui::TreeItem::new(4, Some(2), "Cube", "box"),
        ui::TreeItem::new(5, Some(4), "Modifiers", "settings"),
        ui::TreeItem::new(6, Some(5), "Bevel", "settings"),
        ui::TreeItem::new(7, Some(5), "Subdivision", "settings"),
        ui::TreeItem::new(8, Some(4), "Cube", "box"),
        ui::TreeItem::new(9, Some(8), "Metal.002", "palette"),
        ui::TreeItem::new(10, Some(2), "Key Light", "bulb"),
        ui::TreeItem::new(11, Some(10), "Area", "bulb"),
        ui::TreeItem::new(12, Some(2), "Fill Light", "bulb"),
        ui::TreeItem::new(13, Some(12), "Point", "bulb"),
        ui::TreeItem::new(14, Some(1), "Environment", "folder"),
        ui::TreeItem::new(15, Some(14), "Ground", "box"),
        ui::TreeItem::new(16, Some(15), "Plane", "box"),
        ui::TreeItem::new(17, Some(14), "Sky Dome", "box"),
        ui::TreeItem::new(18, Some(17), "Sphere", "box"),
        ui::TreeItem::new(19, Some(14), "Trees [Instances]", "stack"),
        ui::TreeItem::new(20, Some(1), "Rig", "folder"),
        ui::TreeItem::new(21, Some(20), "Armature", "hierarchy"),
        ui::TreeItem::new(22, Some(21), "Bones", "hierarchy"),
    ]);
    let mode = signal(Some("scene".to_owned()));
    let blocks = signal(vec![
        ui::TreeItem::new(101, None, "Kivact File", "folder"),
        ui::TreeItem::new(102, Some(101), "Scenes (1)", "stack"),
        ui::TreeItem::new(103, Some(102), "Scene", "box"),
        ui::TreeItem::new(104, Some(101), "Objects (8)", "box"),
        ui::TreeItem::new(105, Some(101), "Materials (5)", "palette"),
        ui::TreeItem::new(106, Some(105), "Metal.002", "palette"),
        ui::TreeItem::new(107, Some(101), "Images (3)", "camera"),
    ]);
    specs((
        col((
            row((kivact::ui::select("Outliner mode", keyed_options([("scene", "▧"), ("kivact-file", "Kivact File")]), mode).placeholder("").name("Outliner mode").style(style().width(32).padding(0)),kivact::ui::text_input("Search Outliner", filter).placeholder("Search").style(style().grow(1)),kivact::ui::button("Outliner filter").variant(kivact::ui::ButtonVariant::Secondary).hide_label(true).icon(kivact::view::icon("filter")),kivact::ui::button("Rename selected").variant(kivact::ui::ButtonVariant::Secondary).hide_label(true).icon(kivact::view::icon("folder")).on_press(move||renaming.set(selected.with(|v|v.first().copied())))))
                .style(style().height(40).padding_x(8).gap(6)),
            scroll(dynamic(mode,move|mode|{
                super::layout_reducers::outliner(if mode.as_deref()==Some("scene") { nodes } else { blocks }, ui::OutlinerState { expanded: (signal(if mode.as_deref()==Some("scene") { vec![1,2,4,14,19] } else { vec![101,102] })).into_binding(), selection: selected.into_binding(), renaming: renaming.into_binding()}).filter(filter).test_id("creative-scene-outliner")
            })).style(style().height(338).min_height(0)),
        )).style(style().gap(0).grid_column(GridPlacement::span(2)).height(380)),
        spec("Interaction","操作",col((
            note("クリックで選択、Ctrl 追加、Shift 範囲。ダブルクリック / F2 で改名。矢印で階層を開閉、ドラッグで移動。"),
            note(move||format!("ACTIVE       {}\nSELECTED     {}\nROWS         13",nodes.with(|v|v.iter().find(|n|selected.with(|s|s.contains(&n.id))).map_or("—",|n|n.label.as_str()).to_owned()),selected.with(Vec::len))),
        ))).style(style().height(380)),
    ))
}
pub(super) fn layers(state: Signal<TransportState>) -> impl IntoView {
    let query = signal(String::new());
    let flags = signal(
        (0..8)
            .map(|i| {
                let mut f = [false; 12];
                f[0] = i != 7;
                f[1] = i == 7;
                f[3] = i == 6;
                for x in [6, 7] {
                    f[x] = i < 6;
                }
                f
            })
            .collect::<Vec<_>>(),
    );
    let opened = signal(vec![4_u64]);
    let ranges = signal(vec![
        (0.0, 240.0),
        (24.0, 168.0),
        (0.0, 200.0),
        (0.0, 240.0),
        (40.0, 220.0),
        (0.0, 240.0),
        (0.0, 240.0),
        (0.0, 240.0),
    ]);
    let rows = signal(
        [
            ("Grade [Adjustment]", "Normal", "None", 0xb8b5d2),
            ("Title_Main", "Normal", "6. Null 1", 0xeec59d),
            ("Particles [Precomp]", "Add", "None", 0xeeb6ca),
            ("Hero_Plate.exr", "Normal", "None", 0x8b7cff),
            ("Shadow Matte", "Multiply", "None", 0xff5c6c),
            ("Null 1", "Normal", "None", 0xffd25a),
            ("Camera 1", "Normal", "None", 0xb4e2db),
            ("Music_Bed.wav", "—", "None", 0x34d399),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (name, mode, parent, color))| {
            super::common::TextRow::new(
                i as u64 + 1,
                [
                    "◉  ◉  ◇",
                    name,
                    "◦  ◦  ◇",
                    mode,
                    parent,
                    "0 f          ━━━━━━━━━━━ 240 f",
                ],
            )
            .color(rgb(color))
        })
        .collect::<Vec<_>>(),
    );
    col((
        line((
            text("Comp 1").style(style().weight(600)),
            kivact::ui::text_input("Layer search", query)
                .placeholder("Search layers")
                .style(style().width(180)),
            kivact::ui::toggle_button("Shy", signal(false)),
            kivact::ui::toggle_button("Collapse", signal(false)),
            kivact::ui::toggle_button("Motion Blur", signal(false)),
        )),
        super::common::text_table(
            &[
                ("A/V", 104.0),
                ("Layer Name", 0.0),
                ("Switches", 160.0),
                ("Mode", 110.0),
                ("Parent & Link", 120.0),
                ("Range", 260.0),
            ],
            rows,
            signal(vec![4]),
            signal(None),
        )
        .filter(query)
        .row_label(|row| row.cells.get(1).cloned().unwrap_or_default())
        .resizable(signal(Vec::new()))
        .cell_controls(move |item, column| {
            let id = item.get_untracked().id;
            match column {
                0 | 2 => row((0..if column == 0 { 4 } else { 8 })
                    .map(move |index| {
                        kivact::ui::toggle_button(
                            "",
                            bind(
                                move || {
                                    flags.get()[(id - 1) as usize]
                                        [index + if column == 0 { 0 } else { 4 }]
                                },
                                move |v| {
                                    flags.update(|f| {
                                        f[(id - 1) as usize]
                                            [index + if column == 0 { 0 } else { 4 }] = v
                                    })
                                },
                            ),
                        )
                        .icon(if column == 0 {
                            ["eye", "volume", "diamond", "lock"][index]
                        } else {
                            [
                                "eye",
                                "light",
                                "settings",
                                "adjustments",
                                "stack",
                                "wave",
                                "adjustments",
                                "box",
                            ][index]
                        })
                        .name(format!(
                            "Layer {id} {} {index}",
                            if column == 0 { "AV" } else { "Switch" }
                        ))
                        .style(
                            style()
                                .size(16, 16)
                                .padding(0)
                                .background(Color::TRANSPARENT)
                                .selected(style().background(Color::TRANSPARENT))
                                .border(0, Color::TRANSPARENT),
                        )
                    })
                    .collect::<Vec<_>>())
                .style(style().gap(2))
                .into_view(),
                1 => row((
                    node().style(style().size(12, 12).radius(6)).style(move || {
                        style().background(item.get().color.unwrap_or(Color::WHITE))
                    }),
                    text(format!("{id}")).style(style().width(16).font_size(10)),
                    kivact::ui::button(format!("Expand layer {id}"))
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .hide_label(true)
                        .icon(kivact::view::icon("chevron-right"))
                        .style(style().size(16, 16).padding(0))
                        .style(move || {
                            style().rotate(if opened.with(|v| v.contains(&id)) {
                                90
                            } else {
                                0
                            })
                        })
                        .on_press(move || {
                            opened.update(|v| {
                                if v.contains(&id) {
                                    v.retain(|x| *x != id);
                                } else {
                                    v.push(id);
                                }
                            })
                        }),
                    kivact::view::icon("stack"),
                    text(move || item.get().cells[1].clone()),
                ))
                .style(style().gap(6))
                .into_view(),
                5 => ui::range(
                    format!("Layer {id} range"),
                    bind(
                        move || ranges.get()[(id - 1) as usize],
                        move |v| ranges.update(|rs| rs[(id - 1) as usize] = v),
                    ),
                )
                .limits(0.0..=240.0)
                .interval(item.get_untracked().color.unwrap_or(Color::WHITE))
                .movable(true)
                .disabled(move || flags.get()[(id - 1) as usize][3])
                .style(style().width(pct(100)).height(22))
                .into_view(),
                3 | 4 => {
                    let labels = if column == 3 {
                        vec!["Normal", "Add", "Multiply", "Screen", "Overlay"]
                    } else {
                        vec!["None", "6. Null 1", "7. Camera 1"]
                    };
                    let options = labels
                        .into_iter()
                        .map(|label| kivact::interaction::ListItem::new(label.to_owned(), label))
                        .collect::<Vec<_>>();
                    kivact::ui::select(
                        "Selection",
                        options,
                        bind(
                            move || item.with(|row| row.cells.get(column).cloned()),
                            move |value: Option<String>| {
                                if let Some(value) = value {
                                    rows.update(|rows| {
                                        if let Some(row) = rows.iter_mut().find(|row| row.id == id)
                                        {
                                            row.cells[column] = value;
                                        }
                                    });
                                }
                            },
                        ),
                    )
                    .placeholder("")
                    .style(style().height(22).width(pct(100)))
                    .into_view()
                }
                _ => text(move || item.with(|r| r.cells.get(column).cloned().unwrap_or_default()))
                    .into_view(),
            }
        })
        .row_details(move |item| {
            let id = item.get_untracked().id;
            col((
                text("⌄  Transform").style(style().height(20).weight(600)),
                layer_property(id, "Anchor Point", vec![960.0, 540.0], "", state, false),
                layer_property(id, "Position", vec![960.0, 540.0], "", state, id == 2),
                layer_property(id, "Scale", vec![100.0, 100.0], "%", state, false),
                layer_property(id, "Rotation", vec![0.0], "°", state, false),
                layer_property(id, "Opacity", vec![100.0], "%", state, id == 2),
            ))
            .hidden(move || !opened.with(|v| v.contains(&id)))
            .style(style().padding_left(168).gap(4).padding_bottom(8))
            .into_view()
        })
        .on_reorder(move |change| super::layout_reducers::reorder(rows, change.key, change.to))
        .name("Layer Stack table"),
    ))
    .style(style().padding(8))
}

fn layer_property(
    id: u64,
    label: &str,
    initial: Vec<f64>,
    unit: &str,
    state: Signal<TransportState>,
    animated: bool,
) -> Node {
    let watch = signal(animated);
    let keys = signal(if animated {
        if label == "Position" {
            vec![24.0_f64, 72.0]
        } else {
            vec![24.0, 48.0, 150.0, 168.0]
        }
    } else {
        Vec::new()
    });
    let fields = initial
        .into_iter()
        .enumerate()
        .map(|(i, initial)| {
            let value = signal(initial);
            let written = bind(value, move |v| {
                value.set(v);
                if watch.get_untracked() {
                    let frame = state.get_untracked().frame.round();
                    keys.update(|ks| {
                        if !ks.contains(&frame) {
                            ks.push(frame);
                        }
                    });
                }
            });
            let field =
                ui::number(format!("Layer {id} {label} {i}"), written)
                    .range(
                        (if label == "Opacity" { 0.0 } else { -65536.0 })
                            ..=(if label == "Opacity" { 100.0 } else { 65536.0 }),
                    )
                    .precision(1)
                    .unit(unit.to_owned());
            field.label_visible(false).style(
                style()
                    .width(60)
                    .height(20)
                    .background(Color::TRANSPARENT)
                    .color(rgb(0xa397ff)),
            )
        })
        .collect::<Vec<_>>();
    row((
        kivact::ui::toggle_button(
            "◴",
            bind(watch, move |on| {
                watch.set(on);
                if on {
                    keys.update(|ks| ks.push(state.get_untracked().frame.round()));
                } else {
                    keys.set(Vec::new());
                }
            }),
        )
        .name(format!("Layer {id} {label} stopwatch"))
        .style(
            style()
                .size(16, 20)
                .padding(0)
                .background(Color::TRANSPARENT),
        ),
        text(label.to_owned()).style(style().width(90).font_size(11.5)),
        row(fields).style(style().gap(2)),
        spacer(),
        node()
            .style(style().height(22).width(260).shrink(0))
            .draw(move |d, s| {
                let p = d.theme().ext::<ui::Tokens>();
                d.line(Point::new(0.0, 0.0), Point::new(0.0, s.height), 1.0, p.dim);
                for frame in keys.get() {
                    let x = frame / 240.0 * s.width;
                    d.fill_path(
                        &kivact::draw::Path::new()
                            .move_to(Point::new(x, 6.0))
                            .line_to(Point::new(x + 4.0, 11.0))
                            .line_to(Point::new(x, 16.0))
                            .line_to(Point::new(x - 4.0, 11.0))
                            .close(),
                        p.accent,
                    );
                }
                let x = state.get().frame / 240.0 * s.width;
                d.line(
                    Point::new(x, 0.0),
                    Point::new(x, s.height),
                    1.0,
                    p.text_high,
                );
            }),
    ))
    .style(style().gap(5).height(22))
}
