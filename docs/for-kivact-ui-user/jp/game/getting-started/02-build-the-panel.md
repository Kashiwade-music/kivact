# ステップ 2: パネルを組み立てる

[English](../../../en/game/getting-started/02-build-the-panel.md)

このステップでは、枠付きのパネルを、番号付きの見出し、状態バッジ、3 種類のメーター、そのメーターを変えるボタンで満たします。

## プログラム

`src/main.rs` を次の内容に置き換えます。

```rust,no_run
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::style::{Align, FontStyle, ThemeMode};
use kivact::ui::{ButtonVariant, progress};
use kivact_ui_game::{self as game, Density, ThemeOptions};

/// A monospaced caption over a meter.
#[component]
fn labeled(label: Value<String>, meter: View) -> impl Element {
    column((game::mono(label), meter)).style(style().gap(6))
}

#[component]
fn loadout() -> impl Element {
    let armor = signal(4_usize);
    let energy = signal(76.0_f64);

    game::hud(
        column((
            row((
                text("01").style(move || {
                    let t = kivact::style::use_theme().get();
                    style()
                        .font_family(t.get(&game::fonts::DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
                        .font_size(34)
                        .line_height(0.9)
                        .font_weight(800)
                        .font_style(FontStyle::Italic)
                        .color(kivact::style::color::ACCENT)
                }),
                kivact::ui::heading("LOADOUT", 2).variant(game::HeadingVariant::Section),
                text("Squad 04").style(style().font_size(12).color(kivact::style::color::TEXT_MUTED)),
            ))
            .style(
                style()
                    .gap(14)
                    .align_items(Align::End)
                    .padding_bottom(10)
                    .border_bottom(2, kivact::style::color::TEXT),
            )
            .draw_over(|d, s| {
                d.fill_path(
                    &kivact::draw::Path::new()
                        .move_to(kivact::geometry::Point::new(s.width - 44., s.height - 4.))
                        .line_to(kivact::geometry::Point::new(s.width, s.height - 4.))
                        .line_to(kivact::geometry::Point::new(s.width - 2., s.height + 6.))
                        .line_to(kivact::geometry::Point::new(s.width - 46., s.height + 6.))
                        .close(),
                    game::color::YELLOW,
                )
            }),
            row((
                kivact::ui::badge("READY").tone(Tone::Success),
                game::mono("SLOT 2"),
            ))
            .style(style().gap(12).align_items(Align::Center)),
            row((
                labeled(
                    "ARMOR",
                    kivact::ui::progress(move || Some(armor.get() as f64 / 10.0))
                        .name("Armor")
                        .variant(kivact::ui::ProgressVariant::Gauge)
                        .variant(game::GaugeSegments(10))
                        .variant(Tone::Info),
                ),
                labeled(
                    "ENERGY",
                    kivact::ui::progress(move || Some(energy.get() / 100.0))
                        .name("ENERGY")
                        .variant(kivact::ui::ProgressVariant::Ring)
                        .variant(kivact::ui::ProgressMotion {
                            determinate_ms: 900,
                            determinate_easing: kivact::motion::Easing::CubicBezier(
                                0.2, 0.7, 0.2, 1.,
                            ),
                            ..Default::default()
                        })
                        .variant(Tone::Warning),
                ),
            ))
            .style(style().gap(32).align_items(Align::Center)),
            labeled(
                "EXPERIENCE",
                progress(Some(0.62))
                    .name("Experience")
                    .variant(kivact::ui::Striped)
                    .moving(true),
            ),
            row((
                button("Upgrade armor")
                    .variant(kivact::ui::Slant)
                    .disabled(move || armor.get() >= 10)
                    .on_press(move || armor.update(|a| *a += 1)),
                button("Recharge")
                    .variant(ButtonVariant::Secondary)
                    .on_press(move || energy.update(|e| *e = (*e + 10.0).min(100.0))),
            ))
            .style(style().gap(10)),
        ))
        .style(style().gap(18).padding(20)),
    )
    .style(style().width(420).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Loadout")
        .theme(game::theme_with(
            ThemeMode::Dark,
            ThemeOptions {
                density: Density::Comfortable,
                ..Default::default()
            },
        ))
        .run(loadout)
}
```

実行します。大きな斜体の番号が付いた見出し（下線の右端に黄色い印）、緑の READY バッジ、4 本が点灯した 10 本の斜めのバー、ENERGY の文字の上に 76% と表示された区切り付きの円、動く縞模様のバー、そして 2 つのボタンが見えるはずです。**装甲を強化**を押すとバーが 1 本点灯し、10 本目で無効になります。**充電**を押すと、リングが 10 ずつ増え、100 で止まります。

## 各部分の役割

**`game::hud(content)`** はステップ 1 の枠で、枠線と 2 つのアクセント色のカギ括弧を持つ面です。関連するコントロールのまとまりごとに 1 つ巻きます。番号、`kivact::ui::heading(title, 2).variant(game::HeadingVariant::Section)`、説明の行を組み合わせて、番号付きのセクション見出しを作ります。

**`kivact::ui::badge(label).tone(tone)`** は、四角い点の付いた小さな状態ラベルです。`Tone` は Kivact 共通の意味の enum で、`Neutral`、`Info`、`Success`、`Warning`、`Danger`、`Accent` があります。意味を伝えると、色はテーマが選びます。

**`labeled(label, meter)`** はこのプログラムの小さなコンポーネントで、等幅のキャプションをメーターの上に重ねます。`meter` は `View` なので、どのメーターも入ります。

**`progress(..).variant(ProgressVariant::Gauge)`** は斜めのバーを描きます。アプリケーションの値を全体で割ります。`armor.get() as f64 / 10.0` が 10 本のバーのうちの割合です。`game::GaugeSegments(10)` は区切りの本数を決めます（普通の `usize`）。`.name(..)` でアクセシブルな名前を付け、キャプションは別に描きます。**`ProgressVariant::Ring`** は同じ 0〜1 の割合から区切り付きの円を描きます。通常の child slot に中央の小さなキャプションを置けます。標準の進捗の所有者が弧をアニメーションさせます。

**`progress(Some(value))`** は標準のプログレスバーで、値は 0 から 1 です。Game テーマは平らなバーとして描きます。`.variant(kivact::ui::Striped)` と `.moving(true)` で、動く斜めの縞模様が加わります。バーには表示される名前もアクセシブルな名前もないため、`.name(..)` で付けます。

**ボタン。** `.variant(kivact::ui::Slant)` はボタンを斜めにし、`ButtonVariant::Secondary` は紺色のコンソールボタンにします。標準の `ButtonVariant` も引き続き使え、組み合わせもできます。型の違う variant は、先に付けたものに追加されます。

**Rust メモ。** `armor.update(|a| *a += 1)` は値をその場で書き換えます。`a` はシグナルの中の数値への可変参照です。`move ||` はクロージャにシグナルのハンドルのコピーを渡します。シグナルは `Copy` なので、これは軽い操作です。

次へ: [ステップ 3: タブとアイテム](03-tabs-and-items.md)
