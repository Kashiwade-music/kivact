//! Role: Control buttons and widgets for creative application.

use super::common::{col, keyed_options, line, note, spec, specs};
use kivact::interaction::MenuEntry;
use kivact::prelude::{ColorToken, Element, IntoView, Node, Signal, node, rgb, row, signal, text};
use kivact::style::GridPlacement;
use kivact::style::{Align, Color, pct, style};
use kivact_ui_creative as ui;

pub(super) fn buttons(status: Signal<String>) -> impl IntoView {
    let snap = signal(true);
    let interpolation = signal(Some("bezier".to_owned()));
    let curve = signal(ui::CubicBezier::default());
    specs((
        buttons_static_collections(),
        buttons_standard_indicators(),
        buttons_cubic_easing(curve),
        buttons_push_button(status),
        buttons_glyph_button(),
        buttons_toggle_button(snap),
        buttons_segmented_enum_row(interpolation),
        buttons_split_menu_button(status),
        buttons_tool_palette().style(style().grid_column(GridPlacement::span(2)).min_height(360)),
    ))
}

/// The button name of an icon of the kit's set: its words in title case
/// (`"zoom-in"` is `ZoomIn`), and `Close` for the close mark `"x"`.
fn icon_label(icon: &str) -> String {
    if icon == "x" {
        return "Close".into();
    }
    icon.split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                .unwrap_or_default()
        })
        .collect()
}

pub(super) fn tool_palette(vertical: bool) -> Node {
    tool_palette_size(vertical, false)
}
pub(super) fn tool_palette_size(vertical: bool, compact: bool) -> Node {
    let selected = signal(0_usize);
    let tools = if vertical {
        vec![
            "pointer",
            "crosshair",
            "move",
            "rotate",
            "scale",
            "pencil",
            "ruler",
        ]
    } else if compact {
        vec!["pointer", "move", "rotate", "scale", "scissors", "pencil"]
    } else {
        vec![
            "pointer", "range", "scissors", "link", "eraser", "zoom-in", "hand",
        ]
    };
    let buttons = tools
        .into_iter()
        .enumerate()
        .flat_map(|(i, icon)| {
            let name = format!("Tool {}", icon_label(icon));
            let button = kivact::ui::button(name)
                .hide_label(true)
                .icon(kivact::view::icon(icon))
                .variant(kivact::ui::ButtonVariant::Ghost)
                .with_node(|node| node.selected(move || selected.get() == i))
                .on_press(move || selected.set(i))
                .style(
                    style().selected(
                        style()
                            .background(ui::ColorName::Accent)
                            .color(ui::ColorName::AccentInk),
                    ),
                )
                .style(style().size(if compact { 22 } else { 30 }, if compact { 22 } else { 30 }))
                .into_view();
            if !compact && ((vertical && matches!(i, 1 | 4)) || (!vertical && i == 3)) {
                vec![
                    button,
                    node()
                        .style(if vertical {
                            style().height(1).width(pct(100)).margin_y(5)
                        } else {
                            style().width(1).height(22).margin_x(4)
                        })
                        .style(style().background(ui::ColorName::HairStrong))
                        .into_view(),
                ]
            } else {
                vec![button]
            }
        })
        .collect::<Vec<_>>();
    let mut root = node()
        .child(buttons)
        .style(
            style()
                .direction(if vertical {
                    kivact::style::Direction::Column
                } else {
                    kivact::style::Direction::Row
                })
                .gap(2)
                .padding(3)
                .radius(11)
                .align_self(Align::Start),
        )
        .style(
            style()
                .background(ColorToken::Raised)
                .border(1, ui::ColorName::Hair),
        );
    if !vertical && !compact {
        root = root.style(style().width(pct(100)).max_width(548));
    }
    root
}

pub(super) fn num(
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    precision: usize,
    unit: &str,
) -> ui::Numeric {
    ui::number(label.to_owned(), signal(value))
        .range(min..=max)
        .precision(precision)
        .step(if precision == 0 { 1.0 } else { 0.01 })
        .unit(unit)
        .reset_to(value)
        .style(style().width(pct(100)))
}
pub(super) fn slide(label: &str, value: f64) -> ui::Numeric {
    ui::value_slider(label.to_owned(), signal(value))
        .reset_to(value)
        .style(style().width(pct(100)))
}

pub(super) fn selection() -> impl IntoView {
    let radio = signal(Some("median".to_owned()));
    specs((
        selection_checkbox_radio(radio).style(style().min_height(190)),
        selection_switch(),
        selection_enum_dropdown(),
        selection_text_field(),
        selection_data_block_selector(),
        selection_script_expression().style(style().min_height(202)),
    ))
}

pub(super) fn chip(label: &str, color: Color) -> Node {
    node().child(text(label.to_owned())).style(
        style()
            .padding_x(8)
            .padding_y(3)
            .radius(5)
            .font_family("Geist Mono")
            .font_size(10)
            .color(color)
            .background(color.with_alpha(0.12))
            .border(1, color.with_alpha(0.2)),
    )
}

fn buttons_static_collections() -> Node {
    spec(
        "Static collections",
        "一覧と表の意味を標準部品で公開",
        col((
            kivact::ui::table((
                kivact::ui::table_header((kivact::ui::table_row((
                    kivact::ui::table_head("Asset"),
                    kivact::ui::table_head("State"),
                )),)),
                kivact::ui::table_body((kivact::ui::table_row((
                    kivact::ui::table_cell("Camera"),
                    kivact::ui::table_cell("Ready"),
                )),)),
            )),
            kivact::ui::item_list((
                kivact::ui::item_row("Render queue").description("All frames are ready"),
            )),
        )),
    )
}

fn buttons_standard_indicators() -> Node {
    spec(
        "Standard indicators",
        "状態・読み込み・ショートカット",
        col((
            line((
                kivact::ui::badge("Ready").tone(kivact::interaction::Tone::Success),
                kivact::ui::avatar("Ada Lovelace")
                    .size(kivact::ui::ControlSize::Small)
                    .status(Some(kivact::interaction::Tone::Success)),
                kivact::ui::spinner().size(kivact::ui::ControlSize::Small),
                kivact::ui::kbd("mod+shift+p"),
            )),
            kivact::ui::skeleton()
                .lines(2)
                .shape(kivact::ui::SkeletonShape::Text),
        )),
    )
}

fn buttons_cubic_easing(curve: Signal<ui::CubicBezier>) -> Node {
    spec(
        "Cubic easing",
        "同じ曲線を編集・プレビュー",
        col((
            ui::bezier_curve(curve),
            ui::curve_preview(move || curve.get().samples(64)),
        )),
    )
}

fn buttons_push_button(status: Signal<String>) -> Node {
    spec(
        "Push Button",
        "押しボタン",
        col((
            line((
                kivact::ui::button("Apply")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .on_press(move || status.set("Applied".into())),
                kivact::ui::button("Render")
                    .icon(kivact::view::icon("aperture"))
                    .variant(kivact::ui::ButtonVariant::Primary),
                kivact::ui::button("Delete")
                    .icon(kivact::view::icon("trash"))
                    .variant(kivact::ui::ButtonVariant::Danger),
            )),
            kivact::ui::button("Disabled")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .disabled(true)
                .style(style().align_self(Align::Start)),
            line((
                kivact::ui::button("Small")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .size(kivact::ui::ControlSize::Small),
                kivact::ui::button("Regular").variant(kivact::ui::ButtonVariant::Secondary),
                kivact::ui::button("Large 30px")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .size(kivact::ui::ControlSize::Large),
            )),
            kivact::ui::button("Add Modifier")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .icon(kivact::view::icon("plus"))
                .style(style().width(pct(100))),
        )),
    )
}

fn buttons_glyph_button() -> Node {
    spec(
        "Glyph Button",
        "アイコンボタン",
        col((
            line(
                ["copy", "link", "settings", "more", "trash"]
                    .into_iter()
                    .map(|i| {
                        let name = icon_label(i);
                        kivact::ui::button(name)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .hide_label(true)
                            .icon(kivact::view::icon(i))
                    })
                    .collect::<Vec<_>>(),
            ),
            line(
                ["eye", "lock", "camera", "x", "plus"]
                    .into_iter()
                    .map(|i| {
                        let name = format!("Ghost {}", icon_label(i));
                        kivact::ui::button(name)
                            .hide_label(true)
                            .icon(kivact::view::icon(i))
                            .variant(kivact::ui::ButtonVariant::Ghost)
                    })
                    .collect::<Vec<_>>(),
            ),
        )),
    )
}

fn buttons_toggle_button(snap: Signal<bool>) -> Node {
    spec(
        "Toggle Button",
        "トグル",
        col((
            line((
                kivact::ui::toggle_button("Snap", snap).icon(kivact::view::icon("magnet")),
                kivact::ui::toggle_button("Ripple", signal(false)).icon(kivact::view::icon("wave")),
                kivact::ui::toggle_button("Auto Key", signal(false)),
            )),
            line((
                kivact::ui::toggle_button("◎", signal(false)),
                kivact::ui::toggle_button("▣", signal(true)),
                kivact::ui::toggle_button("≋", signal(false)),
                kivact::ui::toggle_button("◇", signal(false)),
            )),
        )),
    )
}

fn buttons_segmented_enum_row(interpolation: Signal<Option<String>>) -> Node {
    spec(
        "Segmented / Enum Row",
        "排他ボタン列",
        col((
            kivact::ui::radio_group(
                "Mode",
                keyed_options([
                    ("object", "Object"),
                    ("edit", "Edit"),
                    ("sculpt", "Sculpt"),
                    ("paint", "Paint"),
                ]),
                signal(Some("object".to_owned())),
            )
            .orientation(kivact::view::Orientation::Horizontal)
            .variant(kivact::ui::RadioGroupVariant::Segmented),
            kivact::ui::radio_group(
                "Pivot",
                keyed_options([
                    ("median", "◎"),
                    ("individual", "◯"),
                    ("cursor", "◉"),
                    ("axis-x", "X"),
                    ("axis-y", "Y"),
                    ("axis-z", "Z"),
                ]),
                signal(Some("median".to_owned())),
            )
            .orientation(kivact::view::Orientation::Horizontal)
            .variant(kivact::ui::RadioGroupVariant::Segmented),
            kivact::ui::radio_group(
                "Interpolation",
                keyed_options([
                    ("constant", "Constant"),
                    ("linear", "Linear"),
                    ("bezier", "Bezier"),
                ]),
                interpolation,
            )
            .orientation(kivact::view::Orientation::Horizontal)
            .variant(kivact::ui::RadioGroupVariant::Segmented),
        )),
    )
}

fn buttons_split_menu_button(status: Signal<String>) -> Node {
    spec(
        "Split / Menu Button",
        "分割ボタン",
        col((
            ui::split_button(
                "Add to Render Queue",
                vec![
                    MenuEntry::item("image", "Render Image"),
                    MenuEntry::item("animation", "Render Animation"),
                    MenuEntry::item("selected", "Queue Selected"),
                ],
            )
            .on_press(move || status.set("Queued render".into()))
            .on_press(move || status.set("Queued render".into()))
            .on_activate(move |key: &'static str| status.set(format!("Queue {key}")))
            .style(style().width(197)),
            line((
                kivact::ui::menu(
                    "Save",
                    vec![
                        MenuEntry::item("save", "Save"),
                        MenuEntry::item("save-as", "Save As..."),
                    ],
                )
                .on_activate(|_: &'static str| ())
                .offset(0.0)
                .attach(
                    kivact::ui::button("Save")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .trailing(kivact::view::icon("chevron-down")),
                ),
                kivact::ui::menu(
                    "Add",
                    vec![
                        MenuEntry::item("object", "Object"),
                        MenuEntry::item("camera", "Camera"),
                    ],
                )
                .on_activate(|_: &'static str| ())
                .offset(0.0)
                .attach(
                    kivact::ui::button("Add")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .trailing(kivact::view::icon("chevron-down")),
                ),
            )),
        )),
    )
}

fn buttons_tool_palette() -> Node {
    spec(
        "Tool Palette",
        "ツールボックス",
        col((
            tool_palette(true),
            tool_palette(false),
            note("選択・押下はアクセント色の半透明ティント。Ctrl で微調整 / Shift で粗調整。"),
        )),
    )
}

fn selection_checkbox_radio(radio: Signal<Option<String>>) -> Node {
    spec(
        "Checkbox / Radio",
        "チェック・ラジオ",
        row((
            col((
                kivact::ui::checkbox("Shadows", signal(true)),
                kivact::ui::checkbox("Ambient Occlusion", signal(false)),
                kivact::ui::checkbox("Mixed Selection", signal(false)).indeterminate(true),
                kivact::ui::checkbox("Disabled", signal(false)).disabled(true),
            )),
            kivact::ui::radio_group(
                "Pivot point",
                vec![
                    kivact::interaction::ListItem::new("median".to_owned(), "Median Point"),
                    kivact::interaction::ListItem::new(
                        "individual".to_owned(),
                        "Individual Origins",
                    ),
                    kivact::interaction::ListItem::new("cursor".to_owned(), "3D Cursor"),
                    kivact::interaction::ListItem::new("active".to_owned(), "Active Element")
                        .disabled(true),
                ],
                radio,
            )
            .style(style().gap(8).min_width(0)),
        ))
        .style(style().gap(24).align_items(Align::Start)),
    )
}

fn selection_switch() -> Node {
    spec(
        "Switch",
        "スイッチ",
        col((
            kivact::ui::switch("GPU Compute", signal(true)),
            kivact::ui::switch("Low-latency Monitoring", signal(false)),
            kivact::ui::switch("Background Render", signal(true)),
        )),
    )
}

fn selection_enum_dropdown() -> Node {
    spec(
        "Enum Dropdown",
        "列挙",
        col((
            ui::property_row(
                "Render Engine",
                kivact::ui::select(
                    "Render Engine",
                    keyed_options([
                        ("path", "Path Tracer"),
                        ("raster", "Rasterizer"),
                        ("workbench", "Workbench"),
                    ]),
                    signal(Some("path".to_owned())),
                )
                .placeholder("")
                .name("Render Engine")
                .style(style().width(pct(100))),
            ),
            ui::property_row(
                "Blend Mode",
                kivact::ui::select(
                    "Blend Mode",
                    keyed_options([
                        ("normal", "Normal"),
                        ("dissolve", "Dissolve"),
                        ("darken", "Darken"),
                        ("multiply", "Multiply"),
                        ("color-burn", "Color Burn"),
                        ("linear-burn", "Linear Burn"),
                        ("lighten", "Lighten"),
                        ("screen", "Screen"),
                        ("color-dodge", "Color Dodge"),
                        ("add", "Add"),
                        ("overlay", "Overlay"),
                        ("soft-light", "Soft Light"),
                        ("hard-light", "Hard Light"),
                        ("hue", "Hue"),
                        ("saturation", "Saturation"),
                        ("color", "Color"),
                        ("luminosity", "Luminosity"),
                    ]),
                    signal(Some("screen".to_owned())),
                )
                .placeholder("")
                .name("Blend Mode")
                .style(style().width(pct(100))),
            ),
            ui::property_row(
                "Sample Rate",
                kivact::ui::select(
                    "Sample Rate",
                    keyed_options([
                        ("hz44100", "44.1 kHz"),
                        ("hz48000", "48 kHz"),
                        ("hz88200", "88.2 kHz"),
                        ("hz96000", "96 kHz"),
                        ("hz192000", "192 kHz"),
                    ]),
                    signal(Some("hz48000".to_owned())),
                )
                .placeholder("")
                .name("Sample Rate")
                .style(style().width(pct(100))),
            ),
            ui::property_row(
                "Disabled",
                kivact::ui::select(
                    "Selection",
                    keyed_options([("none", "None")]),
                    signal(Some("none".to_owned())),
                )
                .placeholder("")
                .disabled(true)
                .style(style().width(pct(100))),
            ),
        ))
        .role(kivact::view::Role::List)
        .style(style().gap(3)),
    )
}

fn selection_text_field() -> Node {
    spec(
        "Text Field",
        "テキスト",
        col((
            kivact::ui::text_input("Object Name", signal("Shot_010_Comp_v012".into())),
            kivact::ui::text_input("Search", signal(String::new()))
                .placeholder("Search effects..."),
            kivact::ui::text_input("Invalid expression", signal("frame_####.exr".into()))
                .invalid(true)
                .style(
                    style()
                        .font_family("Geist Mono")
                        .border(1, rgb(0xff5c6c))
                        .ring(2, rgb(0xff5c6c).with_alpha(0.12)),
                ),
            note("⚠ 連番パディング # が出力ディレクトリで重複しています").style(
                style()
                    .color(rgb(0xff5c6c))
                    .font_size(10.5)
                    .line_height(1.2),
            ),
        )),
    )
}

fn selection_data_block_selector() -> Node {
    spec(
        "Data-block Selector",
        "データブロック",
        col((
            row((
                kivact::ui::select(
                    "Browse Material",
                    keyed_options([
                        ("none", "⊖"),
                        ("glass", "Glass"),
                        ("skin", "Skin_SSS"),
                        ("emission", "Emission"),
                    ]),
                    signal(Some("none".to_owned())),
                )
                .placeholder("")
                .name("Browse Material")
                .style(style().padding(0)),
                kivact::ui::text_input("Material Name", signal("Metal.002".into()))
                    .style(style().width(0).grow(1)),
                kivact::ui::button("3")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .style(style().width(24).padding(0)),
                kivact::ui::toggle_button("♧", signal(true))
                    .name("Fake User")
                    .style(style().width(24).padding(0)),
                kivact::ui::button("Duplicate data-block")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .hide_label(true)
                    .icon(kivact::view::icon("copy")),
                kivact::ui::button("Unlink data-block")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .hide_label(true)
                    .icon(kivact::view::icon("x")),
            ))
            .style(style().gap(1)),
            note("File Path").style(style().font_size(11)),
            row((
                kivact::ui::text_input("Data-block path", signal("//renders/shot010/".into()))
                    .style(style().grow(1).width(0).font_family("Geist Mono")),
                kivact::ui::button("Browse File Path")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .hide_label(true)
                    .icon(kivact::view::icon("folder")),
            ))
            .style(style().gap(1)),
        )),
    )
}

fn selection_script_expression() -> Node {
    spec(
            "Script / Expression",
            "エクスプレッション",
            col((
                kivact::ui::text_area("Expression", signal("// wiggle position\nfreq = 3; amp = 25;\nw = wiggle(freq, amp);\n[w[0], value[1]]".into()))
                    .style(style().height(108)),
                line((chip("● valid",rgb(0x34d399)),note("Ln 4, Col 17 · JS").style(style().font_family("Geist Mono").font_size(10.5)))),
            )),
        )
}
