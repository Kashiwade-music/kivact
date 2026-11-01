//! Role: Catalog specimen framing and token demonstration tiles.
use super::common::{cards, line, list};
use super::slot::{Slot, slot};
use kivact::Value;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::draw::{Layer, Path, Point, Shape, Transform};
use kivact::platform;
use kivact::prelude::{Signal, rgb};
use kivact::style::{Align, BorderStyle, ColorToken, Justify, current_theme, pct, style};
use kivact::task::spawn;
use kivact::ui::{Toast, Tone, toaster};
use kivact::view::{IntoText, IntoValue, IntoView, Role, View};
use kivact_ui_game as g;
use kivact_ui_game::{Color, Tokens, cut_corner, hud, mono};
use std::time::Duration;
/// Supported prototype shape motifs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShapeKind {
    Cut,
    Skew,
    Bracket,
    Stripe,
}

/// A catalog specimen frame. Created by [`specimen`].
pub(super) struct Specimen {
    root: Node,
    id: Value<String>,
    title: Value<String>,
    caption: Value<String>,
    content: View,
    note: Option<Slot>,
}

/// A catalog specimen frame, usable for diagnostics and documentation apps.
///
/// [`Specimen::caption`] adds a right-aligned machine caption to the header.
#[track_caller]
pub(super) fn specimen(
    id: impl IntoText,
    title: impl IntoText,
    content: impl IntoView,
) -> Specimen {
    Specimen {
        root: hud(()),
        id: id.into_text_value(),
        title: title.into_text_value(),
        caption: String::new().into_value(),
        content: content.into_view(),
        note: None,
    }
}
impl Specimen {
    /// A right-aligned machine caption in the header; empty by default.
    pub(super) fn caption(mut self, caption: impl IntoText) -> Self {
        self.caption = caption.into_text_value();
        self
    }
    /// A full-width note under the content, set off by a dashed separator;
    /// built once by `note` when the specimen mounts.
    pub(super) fn note<V: IntoView>(mut self, note: impl FnOnce() -> V + 'static) -> Self {
        self.note = Some(slot(note));
        self
    }
}
impl NodeBuilder for Specimen {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Specimen {
    fn into_view(self) -> View {
        self.root
            .child(column((
                row((
                    mono(self.id).style(style().color(ColorToken::Accent)),
                    text(self.title).style(style().weight(700).font_size(12.5)),
                    node().style(style().grow(1)),
                    mono(self.caption)
                        .style(style().font_size(10.5).weight(500).color(Color::Text3)),
                ))
                .style(
                    style()
                        .height(36)
                        .padding_x(14)
                        .gap(10)
                        .align_items(Align::Center)
                        .background(ColorToken::SurfaceAlt)
                        .border_bottom(1, ColorToken::Border),
                ),
                node()
                    .style(move || {
                        let t = &current_theme();
                        style().padding(t.get::<Tokens>().padding)
                    })
                    .child(self.content),
                self.note.map(|note| {
                    node().child(note()).style(move || {
                        let t = &current_theme();
                        style()
                            .padding_x(14)
                            .padding_y(10)
                            .font_size(11.5)
                            .color(t.get::<Tokens>().text_3)
                            .border_top(1, t.colors().border)
                            .border_style(BorderStyle::Dashed)
                    })
                }),
            )))
            .into_view()
    }
}

/// Token swatch; activation copies the explicit hexadecimal label.
#[track_caller]
pub(super) fn swatch(
    name: impl IntoText,
    hex: impl IntoText,
    color: impl IntoValue<kivact::style::Color>,
) -> Node {
    let hex = hex.into_text_value();
    let color = color.into_value();
    column((
        node().style(move || {
            style().height(54).background(color.get()).clip_path(
                Shape::polygon()
                    .point(0, 0)
                    .point(pct(100), 0)
                    .point(pct(100), pct(100) - 12)
                    .point(pct(100) - 12, pct(100))
                    .point(0, pct(100)),
            )
        }),
        column((
            text(name).style(style().font_size(12).weight(700)),
            mono(hex).style(style().color(ColorToken::TextMuted)),
        ))
        .style(style().padding_x(10).padding_y(10).gap(4)),
    ))
    .role(Role::Button)
    .name(hex)
    .focusable(true)
    .on_press(move || {
        kivact::task::spawn(kivact::platform::copy_text(hex.get()));
    })
    .style(
        style()
            .border(1, ColorToken::Border)
            .background(ColorToken::Surface)
            .hover(style().translate(0, -2).border_color(ColorToken::Accent))
            .padding(1),
    )
}

/// Shape specimen, also useful as a small application callout.
#[track_caller]
pub(super) fn shape_tile(kind: ShapeKind, label: impl IntoText) -> Node {
    let label = mono(label);
    let b = node().style(
        style()
            .size(92, 64)
            .align_items(Align::Center)
            .justify_content(Justify::Center),
    );
    match kind {
        ShapeKind::Cut => b
            .style(
                style()
                    .clip_path(cut_corner(14.))
                    .background(ColorToken::Selection)
                    .color(ColorToken::Accent),
            )
            .child(label.style(style().color(ColorToken::Accent))),
        ShapeKind::Skew => b
            .style(
                style()
                    .transform(Transform::new().skew_x(-12))
                    .background(ColorToken::Accent)
                    .color(ColorToken::AccentText),
            )
            .child(
                label.style(
                    style()
                        .transform(Transform::new().skew_x(12))
                        .color(ColorToken::AccentText),
                ),
            ),
        ShapeKind::Bracket => hud(label).style(
            style()
                .size(92, 64)
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        ),
        ShapeKind::Stripe => b.draw(|d, s| {
            d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                let p = d.theme().get::<Tokens>();
                d.fill_rect(s.to_rect(), p.navy);
                let mut x = -s.height;
                while x < s.width {
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(x, s.height))
                            .line_to(Point::new(x + s.height, 0.))
                            .line_to(Point::new(x + s.height + 8., 0.))
                            .line_to(Point::new(x + 8., s.height))
                            .close(),
                        p.yellow,
                    );
                    x += 16.;
                }
            });
        }),
    }
}

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
        crate::game::specimens::swatch(name, move || format!("#{:06X}", color()), move || rgb(color()))
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
        ))), specimen("TKN-03", "形状言語", list((line((crate::game::specimens::shape_tile(crate::game::specimens::ShapeKind::Cut, "CUT 14"), crate::game::specimens::shape_tile(crate::game::specimens::ShapeKind::Skew, "SKEW -12°"),
            crate::game::specimens::shape_tile(crate::game::specimens::ShapeKind::Bracket, "BRACKET"), crate::game::specimens::shape_tile(crate::game::specimens::ShapeKind::Stripe, "STRIPE"))),
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
