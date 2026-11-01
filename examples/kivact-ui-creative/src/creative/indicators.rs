//! Role: Visual indicators and status displays.

use super::common::{col, line, note, spec, specs};
use super::controls::chip;
use kivact::prelude::{IntoView, Node, NodeBuilder, node, rgb, row, show, signal, spacer, text};
use kivact::style::{Align, style};
use kivact::style::{Overflow, TextAlign};

pub(super) fn indicators() -> impl IntoView {
    let picked = signal(0usize);
    let colors = [
        ("Red", 0xb53838),
        ("Yellow", 0xe4d84c),
        ("Aqua", 0xa9cbc7),
        ("Pink", 0xe5bcc9),
        ("Lavender", 0xa9a9ca),
        ("Peach", 0xe7c19e),
        ("Sea Foam", 0xb3c7b3),
        ("Blue", 0x7c8cff),
        ("Green", 0x4aa44c),
        ("Purple", 0x8e2c9a),
        ("Orange", 0xe8920d),
        ("Brown", 0x7f452a),
        ("Fuchsia", 0xf46dd6),
        ("Cyan", 0x3ad3e0),
        ("Sandstone", 0xa89677),
        ("Dark Green", 0x1e401e),
    ];
    specs((
        spec(
            "LED",
            "インジケーター",
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
        .style(style().min_height(167)),
        spec(
            "Status Chip",
            "状態",
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
                                .radius(9)
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
                                .style(style().height(22).radius(10).padding_x(8).color(rgb(c)))
                                .on_press(move || visible.set(false))
                        })
                    })
                    .collect::<Vec<_>>(),
                )
                .style(style().gap(5)),
                line((
                    text("Inbox"),
                    chip("12", rgb(0x8b7cff)).style(style().radius(10).padding_y(1)),
                )),
            ))
            .style(style().gap(5)),
        ),
        spec(
            "Shortcuts",
            "キーキャップ",
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
                        chip(key, rgb(0x9b9ea8))
                            .style(style().padding_x(5).padding_y(2).font_size(9)),
                        text(label).style(style().font_size(11)),
                    ))
                    .style(style().gap(6).align_items(Align::Center))
                })
                .collect::<Vec<_>>(),
            )
            .style(style().gap(12)),
        ),
        spec(
            "Label Colors",
            "ラベル色",
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
                                .selected(move || picked.get() == i)
                                .style(
                                    style()
                                        .size(26, 26)
                                        .radius(13)
                                        .padding(0)
                                        .background(rgb(c))
                                        .selected(style().ring(2, rgb(c))),
                                )
                                .on_press(move || picked.set(i))
                        })
                        .collect::<Vec<_>>(),
                )
                .style(style().gap(9)),
            ))
            .style(style().gap(2)),
        ),
        spec(
            "Cache / Render Bar",
            "キャッシュ",
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
                .map(|(width, c)| node().style(style().grow(width).height(4).background(rgb(c))))
                .collect::<Vec<_>>())
                .style(style().height(4).radius(2).overflow(Overflow::Hidden)),
                row((note("0f"), spacer(), note("120f"), spacer(), note("240f")))
                    .style(style().font_family("Geist Mono").font_size(10)),
            ))
            .style(style().gap(3)),
        ),
        spec(
            "Badges on Controls",
            "組み合わせ",
            line((
                kivact::ui::button("♧ Messages ❸").variant(kivact::ui::ButtonVariant::Secondary),
                kivact::ui::button("● Timecode Sync").variant(kivact::ui::ButtonVariant::Secondary),
                kivact::ui::toggle("● Arm", signal(false)),
            )),
        )
        .style(style().min_height(116)),
    ))
}
fn led(c: u32, square: bool) -> Node {
    node().style(
        style()
            .size(if square { 10 } else { 7 }, if square { 5 } else { 7 })
            .radius(if square { 2 } else { 4 })
            .background(rgb(c))
            .shadow(kivact::draw::Shadow::new(0, 0, 5, rgb(c).with_alpha(0.5))),
    )
}
