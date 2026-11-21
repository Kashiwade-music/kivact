//! Visual indicators and status displays.

use super::common::{col, line, note, spec};
use super::controls::chip;
use kivact::prelude::{Element, node, rgb, row, show, signal, spacer, text};
use kivact::style::{Align, style};
use kivact::style::{Overflow, TextAlign};
use kivact::view::Node;

pub(super) fn led(c: u32, square: bool) -> Node {
    node().style(
        style()
            .size(if square { 10 } else { 7 }, if square { 5 } else { 7 })
            .border_radius(if square { 2 } else { 4 })
            .background(rgb(c))
            .shadow(kivact::style::Shadow::new(0, 0, 5, rgb(c).with_alpha(0.5))),
    )
}

pub(super) fn indicators_led() -> Node {
    spec(
        "LED",
        "Indicators",
        col((
            line(
                [
                    ("Off", 0x2c2e36),
                    ("Sync", 0x34d399),
                    ("Busy", 0xfbbf24),
                    ("REC", 0xff5c6c),
                    ("MIDI", 0x5aa9ff),
                ]
                .into_iter()
                .map(|(name, c)| {
                    row((led(c, false), text(name)))
                        .style(style().gap(5).align_items(Align::Center))
                })
                .collect::<Vec<_>>(),
            )
            .style(style().gap(14)),
            line((
                led(0x34d399, true),
                led(0xfbbf24, true),
                led(0xff5c6c, true),
            ))
            .style(style().gap(4)),
        )),
    )
}

pub(super) fn indicators_status_chip() -> Node {
    spec(
        "Status Chip",
        "States",
        col((
            line(
                [
                    ("Online", 0x34d399),
                    ("Proxy", 0xfbbf24),
                    ("Offline", 0xff5c6c),
                    ("Cached", 0x5aa9ff),
                    ("Rendering", 0x8b7cff),
                    ("Draft", 0x9b9ea8),
                    ("24p", 0x9b9ea8),
                    ("ACEScg", 0x9b9ea8),
                    ("4K DCI", 0x9b9ea8),
                ]
                .into_iter()
                .map(|(s, c)| {
                    chip(s, rgb(c)).style(
                        style()
                            .border_radius(9)
                            .padding_y(2)
                            .padding_x(6)
                            .font_family("Geist"),
                    )
                })
                .collect::<Vec<_>>(),
            )
            .style(style().gap(5)),
            line(
                [
                    ("Hero", 0xe5484d),
                    ("BG Plate", 0x4a8fe7),
                    ("VFX", 0x9b9ea8),
                ]
                .into_iter()
                .map(|(s, c)| {
                    let visible = signal(true);
                    show(visible, move || {
                        kivact::ui::button(format!("● {s} ×"))
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .style(
                                style()
                                    .height(22)
                                    .border_radius(10)
                                    .padding_x(8)
                                    .color(rgb(c)),
                            )
                            .on_press(move || visible.set(false))
                    })
                })
                .collect::<Vec<_>>(),
            )
            .style(style().gap(5)),
            line((
                text("Inbox"),
                chip("12", rgb(0x8b7cff)).style(style().border_radius(10).padding_y(1)),
            )),
        ))
        .style(style().gap(5)),
    )
}

pub(super) fn indicators_shortcuts() -> Node {
    spec(
        "Shortcuts",
        "Key Caps",
        line(
            [
                ("Ctrl + Shift + S", "Save As"),
                ("F12", "Render"),
                ("Space", "Play"),
                ("Alt + ◉", "Orbit"),
                ("I", "Insert Key"),
                ("Num /", "Cycle"),
            ]
            .into_iter()
            .map(|(key, label)| {
                row((
                    chip(key, rgb(0x9b9ea8)).style(style().padding_x(5).padding_y(2).font_size(9)),
                    text(label).style(style().font_size(11)),
                ))
                .style(style().gap(6).align_items(Align::Center))
            })
            .collect::<Vec<_>>(),
        )
        .style(style().gap(12)),
    )
}

pub(super) fn indicators_label_colors(
    picked: kivact::state::Signal<usize>,
    colors: [(&'static str, u32); 16],
) -> Node {
    spec(
        "Label Colors",
        "Label Colors",
        col((
            note(move || colors[picked.get()].0.to_owned())
                .style(style().text_align(TextAlign::End).font_size(9)),
            line(
                colors
                    .into_iter()
                    .enumerate()
                    .map(|(i, (name, c))| {
                        kivact::ui::button("")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .name(name)
                            .with_node(|node| node.selected(move || picked.get() == i))
                            .style(
                                style()
                                    .size(26, 26)
                                    .border_radius(13)
                                    .padding(0)
                                    .background(rgb(c))
                                    .selected(style().outline(2, rgb(c))),
                            )
                            .on_press(move || picked.set(i))
                    })
                    .collect::<Vec<_>>(),
            )
            .style(style().gap(9)),
        ))
        .style(style().gap(2)),
    )
}

pub(super) fn indicators_cache_render_bar() -> Node {
    spec(
        "Cache / Render Bar",
        "Cache",
        col((
            note("RAM preview (green) / disk cache (blue) / uncached (red)")
                .style(style().font_size(10.5).line_height(1.1)),
            row([
                (22, 0x34d399),
                (4, 0xb53838),
                (30, 0x5aa9ff),
                (10, 0x34d399),
                (34, 0xb53838),
            ]
            .into_iter()
            .map(|(width, c)| node().style(style().flex_grow(width).height(4).background(rgb(c))))
            .collect::<Vec<_>>())
            .style(
                style()
                    .height(4)
                    .border_radius(2)
                    .overflow(Overflow::Hidden),
            ),
            row((note("0f"), spacer(), note("120f"), spacer(), note("240f")))
                .style(style().font_family("Geist Mono").font_size(10)),
        ))
        .style(style().gap(3)),
    )
}

pub(super) fn indicators_badges_on_controls() -> Node {
    spec(
        "Badges on Controls",
        "Combinations",
        line((
            kivact::ui::button("♧ Messages ❸").variant(kivact::ui::ButtonVariant::Secondary),
            kivact::ui::button("● Timecode Sync").variant(kivact::ui::ButtonVariant::Secondary),
            kivact::ui::toggle_button("● Arm", signal(false)),
        )),
    )
}
