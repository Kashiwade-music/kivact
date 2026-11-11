//! Role: Compose Creative catalog menus, accent choices and transport readouts.
use super::audio_values::{TransportState, smpte};
use super::common::line;
use kivact::geometry::Rect;
use kivact::interaction::MenuEntry;
use kivact::prelude::{Element, IntoView, Node, Role, Signal, column, node, rgb, row, text};
use kivact::style::{Align, TextAlign, style};
use kivact::view::LiveRegion;
use kivact_ui_creative as ui;

pub(super) fn accent_controls(accent: Signal<ui::Accent>) -> Vec<Node> {
    let choices = [
        ui::Accent::Iris,
        ui::Accent::Lime,
        ui::Accent::Aqua,
        ui::Accent::Coral,
        ui::Accent::Amber,
    ];
    choices
        .into_iter()
        .map(|choice| {
            node()
                .role(Role::Button)
                .name(format!("Accent {choice:?}"))
                .focusable(true)
                .with_node(|node| node.selected(move || accent.get() == choice))
                .on_press(move || accent.set(choice))
                .style(
                    style()
                        .size(14, 14)
                        .radius(7)
                        .background(choice.colors().0)
                        .shrink(0)
                        .selected(style().ring(2, choice.colors().0))
                        .hover(style().scale(1.15)),
                )
        })
        .collect::<Vec<_>>()
}

pub(super) fn application_menus(status: Signal<String>) -> Vec<kivact::view::View> {
    ["File", "Edit", "View", "Render", "Window", "Help"]
        .into_iter()
        .map(|label| {
            kivact::ui::menu(
                label,
                vec![
                    MenuEntry::item("new", "New Project").with_shortcut("Ctrl N"),
                    MenuEntry::item("open", "Open...").with_shortcut("Ctrl O"),
                    MenuEntry::separator(),
                    MenuEntry::item("preferences", "Preferences..."),
                ],
            )
            .on_activate(move |key: &'static str| status.set(format!("{label} · {key}")))
            .offset(0.0)
            .attach(
                kivact::ui::button(label)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .style(style().height(28).padding_x(8)),
            )
            .into_view()
        })
        .collect::<Vec<_>>()
}

pub(super) fn transport_readout(state: Signal<TransportState>) -> Node {
    row((
        text("TC")
            .style(style().font_size(9.5))
            .style(style().color(ui::ColorName::Accent)),
        text(move || smpte(state.get().frame))
            .test_id("catalog-playback-frame")
            .value_text(move || state.get().frame.to_string())
            .style(style().weight(600)),
        text("BAR")
            .style(style().font_size(9.5))
            .style(style().color(ui::ColorName::Accent)),
        text("1.1.1"),
    ))
    .style(
        style()
            .height(30)
            .padding_x(12)
            .gap(10)
            .radius(9)
            .font_family("Geist Mono")
            .align_items(Align::Center),
    )
    .style(style().background(ui::ColorName::Canvas))
}

pub(super) fn performance_meters() -> Node {
    column(
        [("CPU", 0.22_f64), ("DSK", 0.07_f64)]
            .into_iter()
            .map(|(label, value)| {
                line((
                    text(label).style(style().font_size(8.0)),
                    kivact::ui::progress(Some(value))
                        .name(label)
                        .style(style().size(56, 4))
                        .draw(move |d, size| {
                            d.rounded_rect(
                                Rect::new(0.0, 0.0, size.width * value, size.height),
                                2.0,
                                rgb(0x34d399),
                            )
                        }),
                ))
            })
            .collect::<Vec<_>>(),
    )
    .style(style().font_family("Geist Mono").gap(3))
}

pub(super) fn catalog_status(status: Signal<String>, transport: Signal<TransportState>) -> Node {
    row((
        text("◉ Select   ◉ Pan View   ◉ Context Menu").style(style().font_size(10.5)),
        text(status)
            .role(Role::Status)
            .live_region(LiveRegion::Polite)
            .style(style().grow(1).text_align(TextAlign::Center).font_size(11)),
        text(move || {
            format!(
                "Scene Shot 010   {:.0} f   1920×1080  24 fps   v0.1.0",
                transport.get().frame
            )
        })
        .style(style().font_family("Geist Mono").font_size(10.5)),
    ))
    .style(
        style()
            .height(26)
            .shrink(0)
            .padding_x(14)
            .align_items(Align::Center)
            .gap(16),
    )
    .style(style().color(ui::ColorName::Dim))
}
