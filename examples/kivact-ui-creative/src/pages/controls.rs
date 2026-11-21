//! Control buttons and widgets for creative application.

use super::common::{col, keyed_options, line, note, spec};
use kivact::interaction::MenuEntry;
use kivact::prelude::{Element, IntoView, Signal, node, rgb, row, signal, text};
use kivact::style::{Align, Color, pct, style};
use kivact::view::Node;
use kivact_ui_creative as ui;

/// The button name of an icon of the kit's set: its words in title case
/// (`"zoom-in"` is `ZoomIn`), and `Close` for the close mark `"x"`.
pub(super) fn icon_label(icon: &str) -> String {
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
                            .background(ui::color::ACCENT)
                            .color(ui::color::ACCENT_INK),
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
                        .style(style().background(ui::color::HAIR_STRONG))
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
                .flex_direction(if vertical {
                    kivact::style::FlexDirection::Column
                } else {
                    kivact::style::FlexDirection::Row
                })
                .gap(2)
                .padding(3)
                .border_radius(11)
                .align_self(Align::Start),
        )
        .style(
            style()
                .background(kivact::style::color::RAISED)
                .border(1, ui::color::HAIR),
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
) -> ui::Number {
    ui::number(label.to_owned(), signal(value))
        .range(min..=max)
        .precision(precision)
        .step(if precision == 0 { 1.0 } else { 0.01 })
        .unit(unit)
        .reset_to(value)
        .style(style().width(pct(100)))
}
pub(super) fn slide(label: &str, value: f64) -> ui::ValueSlider {
    ui::value_slider(label.to_owned(), signal(value))
        .reset_to(value)
        .style(style().width(pct(100)))
}

pub(super) fn chip(label: &str, color: Color) -> Node {
    node().child(text(label.to_owned())).style(
        style()
            .padding_x(8)
            .padding_y(3)
            .border_radius(5)
            .font_family("Geist Mono")
            .font_size(10)
            .color(color)
            .background(color.with_alpha(0.12))
            .border(1, color.with_alpha(0.2)),
    )
}

pub(super) fn buttons_static_collections() -> Node {
    spec(
        "Static collections",
        "Lists and tables exposed with standard components",
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

pub(super) fn buttons_standard_indicators() -> Node {
    spec(
        "Standard indicators",
        "States, Loading and Shortcuts",
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

pub(super) fn buttons_cubic_easing(curve: Signal<ui::CubicBezier>) -> Node {
    spec(
        "Cubic easing",
        "Edit and preview the same curve",
        col((
            ui::bezier_curve(curve),
            ui::curve_preview(move || curve.get().samples(64)),
        )),
    )
}

pub(super) fn buttons_push_button(status: Signal<String>) -> Node {
    spec(
        "Push Button",
        "Push Buttons",
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

pub(super) fn buttons_glyph_button() -> Node {
    spec(
        "Glyph Button",
        "Icon Buttons",
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

pub(super) fn buttons_toggle_button(snap: Signal<bool>) -> Node {
    spec(
        "Toggle Button",
        "Toggles",
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

pub(super) fn buttons_segmented_enum_row(interpolation: Signal<Option<String>>) -> Node {
    spec(
        "Segmented / Enum Row",
        "Exclusive Button Row",
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

pub(super) fn buttons_split_menu_button(status: Signal<String>) -> Node {
    spec(
        "Split / Menu Button",
        "Split Buttons",
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

pub(super) fn buttons_tool_palette() -> Node {
    spec(
        "Tool Palette",
        "Tool Palette",
        col((
            tool_palette(true),
            tool_palette(false),
            note(
                "Selected and pressed use a translucent accent tint. Ctrl for fine, Shift for coarse.",
            ),
        )),
    )
}

pub(super) fn selection_checkbox_radio(radio: Signal<Option<String>>) -> Node {
    spec(
        "Checkbox / Radio",
        "Checkbox and Radio",
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

pub(super) fn selection_switch() -> Node {
    spec(
        "Switch",
        "Switch",
        col((
            kivact::ui::switch("GPU Compute", signal(true)),
            kivact::ui::switch("Low-latency Monitoring", signal(false)),
            kivact::ui::switch("Background Render", signal(true)),
        )),
    )
}

pub(super) fn selection_enum_dropdown() -> Node {
    spec(
        "Enum Dropdown",
        "Enum",
        col((
            ui::property_row(
                "Render Engine",
                kivact::ui::select(
                    keyed_options([
                        ("path", "Path Tracer"),
                        ("raster", "Rasterizer"),
                        ("workbench", "Workbench"),
                    ]),
                    signal(Some("path".to_owned())),
                )
                .label("Render Engine")
                .placeholder("")
                .name("Render Engine")
                .style(style().width(pct(100))),
            ),
            ui::property_row(
                "Blend Mode",
                kivact::ui::select(
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
                .label("Blend Mode")
                .placeholder("")
                .name("Blend Mode")
                .style(style().width(pct(100))),
            ),
            ui::property_row(
                "Sample Rate",
                kivact::ui::select(
                    keyed_options([
                        ("hz44100", "44.1 kHz"),
                        ("hz48000", "48 kHz"),
                        ("hz88200", "88.2 kHz"),
                        ("hz96000", "96 kHz"),
                        ("hz192000", "192 kHz"),
                    ]),
                    signal(Some("hz48000".to_owned())),
                )
                .label("Sample Rate")
                .placeholder("")
                .name("Sample Rate")
                .style(style().width(pct(100))),
            ),
            ui::property_row(
                "Disabled",
                kivact::ui::select(
                    keyed_options([("none", "None")]),
                    signal(Some("none".to_owned())),
                )
                .label("Selection")
                .placeholder("")
                .disabled(true)
                .style(style().width(pct(100))),
            ),
        ))
        .role(kivact::view::Role::List)
        .style(style().gap(3)),
    )
}

pub(super) fn selection_text_field() -> Node {
    spec(
        "Text Field",
        "Text",
        col((
            kivact::ui::text_input(signal("Shot_010_Comp_v012".into())).label("Object Name"),
            kivact::ui::text_input(signal(String::new()))
                .label("Search")
                .placeholder("Search effects..."),
            kivact::ui::text_input(signal("frame_####.exr".into()))
                .label("Invalid expression")
                .invalid(true)
                .style(
                    style()
                        .font_family("Geist Mono")
                        .border(1, rgb(0xff5c6c))
                        .outline(2, rgb(0xff5c6c).with_alpha(0.12)),
                ),
            note("⚠ The sequence padding # duplicates a name in the output folder").style(
                style()
                    .color(rgb(0xff5c6c))
                    .font_size(10.5)
                    .line_height(1.2),
            ),
        )),
    )
}

pub(super) fn selection_data_block_selector() -> Node {
    spec(
        "Data-block Selector",
        "Data Block",
        col((
            row((
                kivact::ui::select(
                    keyed_options([
                        ("none", "⊖"),
                        ("glass", "Glass"),
                        ("skin", "Skin_SSS"),
                        ("emission", "Emission"),
                    ]),
                    signal(Some("none".to_owned())),
                )
                .label("Browse Material")
                .placeholder("")
                .name("Browse Material")
                .style(style().padding(0)),
                kivact::ui::text_input(signal("Metal.002".into()))
                    .label("Material Name")
                    .style(style().width(0).flex_grow(1)),
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
                kivact::ui::text_input(signal("//renders/shot010/".into()))
                    .label("Data-block path")
                    .style(style().flex_grow(1).width(0).font_family("Geist Mono")),
                kivact::ui::button("Browse File Path")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .hide_label(true)
                    .icon(kivact::view::icon("folder")),
            ))
            .style(style().gap(1)),
        )),
    )
}

pub(super) fn selection_script_expression() -> Node {
    spec(
        "Script / Expression",
        "Expression",
        col((
            kivact::ui::text_area(signal(
                "// wiggle position\nfreq = 3; amp = 25;\nw = wiggle(freq, amp);\n[w[0], value[1]]"
                    .into(),
            ))
            .label("Expression")
            .style(style().height(108)),
            line((
                chip("● valid", rgb(0x34d399)),
                note("Ln 4, Col 17 · JS").style(style().font_family("Geist Mono").font_size(10.5)),
            )),
        )),
    )
}
