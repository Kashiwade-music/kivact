# ステップ 2: パネルを組み立てる

[English](../../../en/game/getting-started/02-build-the-panel.md)

このステップでは、枠付きのパネルを、番号付きの見出し、状態バッジ、3 種類のメーター、そのメーターを変えるボタンで満たします。

## プログラム

`src/main.rs` を次の内容に置き換えます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::{Tone};
use kivact::ui::{progress};
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

/// メーターの上に置く等幅のキャプション。
fn labeled(label: &'static str, meter: impl IntoView) -> Node {
    column((game::mono(label), meter)).style(style().gap(6))
}

fn loadout() -> impl IntoView {
    let armor = signal(4_usize);
    let energy = signal(76.0_f64);

    game::hud(
        column((
            game::section_heading("01", "LOADOUT", "Squad 04"),
            row((
                game::badge("READY", Tone::Success),
                game::mono("SLOT 2"),
            ))
            .style(style().gap(12).align_items(Align::Center)),
            row((
                labeled(
                    "ARMOR",
                    game::gauge("Armor", armor).total(10).tone(Tone::Info),
                ),
                game::ring("ENERGY", energy, Tone::Warning),
            ))
            .style(style().gap(32).align_items(Align::Center)),
            labeled(
                "EXPERIENCE",
                progress(Some(0.62))
                    .name("Experience")
                    .variant(game::Striped)
                    .moving(true),
            ),
            row((
                button("装甲を強化")
                    .variant(game::Slant)
                    .disabled(move || armor.get() >= 10)
                    .on_press(move || armor.update(|a| *a += 1)),
                button("充電")
                    .variant(game::ButtonTone::Dark)
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
        .theme(game::theme(ThemeMode::Dark).density(Density::Comfortable))
        .run(loadout)
}
```

実行します。大きな斜体の番号が付いた見出し（下線の右端に黄色い印）、緑の READY バッジ、4 本が点灯した 10 本の斜めのバー、ENERGY の文字の上に 76% と表示された区切り付きの円、動く縞模様のバー、そして 2 つのボタンが見えるはずです。**装甲を強化**を押すとバーが 1 本点灯し、10 本目で無効になります。**充電**を押すと、リングが 10 ずつ増え、100 で止まります。

## 各部分の役割

**`game::hud(content)`** はステップ 1 の枠で、枠線と 2 つのアクセント色のカギ括弧を持つ面です。関連するコントロールのまとまりごとに 1 つ巻きます。**`game::section_heading(number, title, description)`** は、画面のセクションの番号付き見出しです。

**`game::badge(label, tone)`** は、四角い点の付いた小さな状態ラベルです。`Tone` は Kivact 共通の意味の enum で、`Neutral`、`Info`、`Success`、`Warning`、`Danger`、`Accent` があります。意味を伝えると、色はテーマが選びます。

**`game::gauge(label, value)`** は、`value` の分だけ点灯する斜めのバーの列で、全体の本数は `.total(n)` で決めます（指定しなければ 100）。第 1 引数はアクセシブルな名前で、画面には表示されません。そのためプログラムでは `labeled` で自分でキャプションを描いています。ゲージは `Signal<usize>` を読むので、バーはシグナルに追従します。**`game::ring(label, value, tone)`** は 0 から 100 の値を表す円形のメーターです。百分率とその下にラベルを中央に表示し、メーターが最初に現れたときと値が変わるたびに、弧が値まで伸びます。

**`progress(Some(value))`** は標準のプログレスバーで、値は 0 から 1 です。Game テーマは平らなバーとして描きます。`.variant(game::Striped)` と `.moving(true)` で、動く斜めの縞模様が加わります。バーには表示される名前もアクセシブルな名前もないため、`.name(..)` で付けます。

**ボタン。** `.variant(game::Slant)` はボタンを斜めにし、`game::ButtonTone::Dark` は紺色のコンソールボタンにします。標準の `ButtonVariant` も引き続き使え、組み合わせもできます。型の違う variant は、先に付けたものに追加されます。

**Rust メモ。** `armor.update(|a| *a += 1)` は値をその場で書き換えます。`a` はシグナルの中の数値への可変参照です。`move ||` はクロージャにシグナルのハンドルのコピーを渡します。シグナルは `Copy` なので、これは軽い操作です。

次へ: [ステップ 3: タブとアイテム](03-tabs-and-items.md)
