//! Role: The Foundations section: tokens, type scale and shape specimens.

use super::common::{cards, line, list};
use super::specimens::{ShapeKind, shape_tile, specimen, swatch};
use kivact::interaction::{Toast, Tone, toaster};
use kivact::platform;
use kivact::prelude::{Signal, rgb};
use kivact::style::{Align, ColorToken, style};
use kivact::task::spawn;
use kivact::view::IntoView;
use kivact::view::{Node, NodeBuilder, row, text};
use kivact_ui_game as g;
use std::time::Duration;

pub(super) fn foundations(mode: Signal<kivact::style::ThemeMode>) -> Node {
    let swatches = [
        ("Primary", "--c-primary", 0x1283f0, 0x2b8ceb),
        ("Primary Strong", "--c-primary-strong", 0x0a62c2, 0x1a6cc4),
        ("Signal Cyan", "--c-cyan", 0x2ec4f0, 0x2ec4f0),
        ("Accent Yellow", "--c-accent", 0xffc928, 0xffc928),
        ("Command Navy", "--c-navy", 0x0c1f36, 0x1a3150),
        ("Success", "--c-success", 0x12a06e, 0x2cc48a),
        ("Warning", "--c-warning", 0xd98300, 0xf5a524),
        ("Danger", "--c-danger", 0xe0364d, 0xff5a6e),
        ("Surface", "--surface", 0xffffff, 0x0f1c2c),
        ("Canvas", "--bg", 0xedf2f7, 0x08111c),
        ("Line Strong", "--line-strong", 0xafc0d2, 0x33506f),
        ("Ink", "--text", 0x13233a, 0xe4edf7),
    ];
    list((specimen("TKN-01", "カラートークン", cards(swatches.into_iter().map(|(name, token, light, dark)| {
        let color = move || if mode.get() == kivact::style::ThemeMode::Light { light } else { dark };
        swatch(name, move || format!("#{:06X}", color()), move || rgb(color()))
            .test_id(format!("game-swatch-{token}"))
            .on_press(move || {
                let value = format!("var({token})");
                let label = format!("{value} = #{:06X}", color());
                spawn(async move {
                    if platform::copy_text(value.clone()).await.is_ok() {
                        toaster().push(Toast::new("コピーしました").body(label).tone(Tone::Success).duration(Some(Duration::from_millis(2400))));
                    } else {
                        toaster().push(Toast::new("コピーできませんでした").body(value).tone(Tone::Warning).duration(Some(Duration::from_millis(3000))));
                    }
                });
            })
    }).collect::<Vec<_>>(), 140.).style(style().gap(10))),
        cards((specimen("TKN-02", "タイポグラフィ", list((
            type_row("DISPLAY", text("MISSION 07").style(style().font_family("Barlow Condensed").font_size(36).italic(true).weight(800))),
            type_row("H1 / 22", text("作戦ログの一括承認").style(style().font_size(22).weight(900))),
            type_row("H2 / 16", text("申請ワークフロー設定").style(style().font_size(16).weight(700))),
            type_row("BODY / 13", text("業務画面の本文は 13px を基準とし、補助情報は 11.5px に落とす。")),
            type_row("MONO / 12", g::mono("OPS-2041 • 2026-10-03T09:40:00+09:00").style(style().font_size(12))),
        ))), specimen("TKN-03", "形状言語", list((line((shape_tile(ShapeKind::Cut, "CUT 14"), shape_tile(ShapeKind::Skew, "SKEW -12°"),
            shape_tile(ShapeKind::Bracket, "BRACKET"), shape_tile(ShapeKind::Stripe, "STRIPE"))),
            text("角丸は使わない。主要操作は -12° の平行四辺形、面はカットコーナー、注目領域は HUD ブラケット、危険領域は警告ストライプで示す。")
                .style(style().color(ColorToken::TextMuted)),
        )))), 460.)))
}

fn type_row(label: &str, content: impl IntoView) -> Node {
    row((
        g::mono(label.to_owned()).style(style().width(88).color(ColorToken::TextMuted)),
        content,
    ))
    .style(
        style()
            .gap(14)
            .padding_y(8)
            .align_items(Align::Baseline)
            .border_bottom(1, ColorToken::Border),
    )
}
