//! Role: Token typography and shape samples for the Pop catalogue.

use kivact::draw::Shadow;
use kivact::prelude::{ColorToken, Node, column, pct, text};
use kivact::style::use_theme;
use kivact::style::{Align, Justify, style};
use kivact_ui_pop as ui;

pub(super) fn type_samples() -> Vec<Node> {
    [
        ("Display 32", 32, "Barlow Italic 800"),
        (
            "見出し H2 / 20",
            20,
            "Inter + Noto Sans JP 600 · tracking −2%",
        ),
        ("見出し H3 / 15", 15, "600"),
        (
            "本文テキスト 14 — 読みやすく、やわらかく。",
            14,
            "400 · line-height 1.5",
        ),
        ("補足テキスト 12 — muted", 12, "500"),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (label, n, sub))| {
        column((
            text(label)
                .style(style().font_size(n).weight(if i == 0 {
                    800
                } else if i < 3 {
                    600
                } else {
                    400
                }))
                .style(if i == 0 {
                    style()
                        .font_family("Barlow")
                        .italic(true)
                        .color(ColorToken::Accent)
                } else {
                    style()
                }),
            text(sub).style(style().font_size(12).color(ColorToken::TextMuted)),
        ))
        .style(
            style()
                .gap(2)
                .padding_bottom(12)
                .border_bottom(1, ColorToken::Border),
        )
    })
    .collect::<Vec<_>>()
}

pub(super) fn shape_samples() -> Vec<Node> {
    [
        (4, "sm · checkbox"),
        (8, "md · control"),
        (10, "lg · popover"),
        (14, "xl · card"),
    ]
    .into_iter()
    .map(|(radius, hint)| {
        column((
            text(radius.to_string()).style(style().font_size(16).weight(600)),
            text(hint).style(style().font_size(12).color(ColorToken::TextMuted)),
        ))
        .style(
            style()
                .height(88)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(ColorToken::SurfaceAlt)
                .border(1, ColorToken::Border)
                .radius(radius),
        )
    })
    .chain(
        [
            (1, "shadow-sm", "card"),
            (2, "shadow-md", "menu"),
            (3, "shadow-lg", "dialog / toast"),
        ]
        .into_iter()
        .map(|(n, label, hint)| {
            column((
                text(label).style(style().weight(600)),
                text(hint).style(style().font_size(12).color(ColorToken::TextMuted)),
            ))
            .style(
                style()
                    .height(88)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, ColorToken::Border)
                    .background(ColorToken::Surface)
                    .radius(14)
                    .shadow(Shadow::new(0, n, n * 4, ColorToken::Shadow)),
            )
        }),
    )
    .collect::<Vec<_>>()
}

pub(super) fn cut_corner_sample() -> Node {
    column((
        text("cut").style(style().weight(600)),
        text("corner accent").style(style().font_size(12).color(ColorToken::TextMuted)),
    ))
    .style(move || {
        let theme = &use_theme().get();
        style().background(theme.ext::<ui::Tokens>().hue(ui::Hue::Blue).0)
    })
    .style(
        style()
            .height(88)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .radius(14)
            .clip_path(
                kivact::draw::Shape::polygon()
                    .point(0, 0)
                    .point(pct(100) - 20, 0)
                    .point(pct(100), 20)
                    .point(pct(100), pct(100))
                    .point(0, pct(100)),
            ),
    )
}
