# 2. 値を編集する

[English](../../../en/creative/getting-started/02-edit-values.md)

このステップでは、インスペクターを小さなプロパティエディタに育てます。1 つの値を共有する位置と不透明度のフィールド、スクラブ・キー操作・入力のたびに反応する「元に戻す」ボタン、ポップアップで選ぶ塗りの色、ドラッグで形を変えるイージング曲線を加えます。

[ステップ 1](01-set-up-the-kit.md) のプロジェクトを使います。`src/main.rs` を次の内容に置き換えてください。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact::view::EditPhase;
use kivact_ui_creative as ui;

fn inspector() -> impl IntoView {
    let x = signal(120.0);
    let opacity = signal(0.8);
    let fill = signal(rgb(0x8b7cff));
    let easing = signal(ui::CubicBezier::default());

    // X の編集 1 回につき元に戻す手順を 1 つ記録する: (編集前の値, 編集後の値)
    let started = signal(0.0);
    let history = signal(Vec::<(f64, f64)>::new());
    let record = move |phase: EditPhase| match phase {
        EditPhase::Begin => started.set(x.get_untracked()),
        EditPhase::Commit => {
            history.update(|steps| steps.push((started.get_untracked(), x.get_untracked())));
        }
        EditPhase::Cancel => {}
    };
    let undo = move || {
        if let Some((before, _)) = history.update(|steps| steps.pop()) {
            x.set(before);
        }
    };

    ui::panel(
        "インスペクター",
        column((
            ui::property_row(
                "X",
                ui::number("X", x)
                    .range(kivact::interaction::ScalarRange::new(-1000.0, 1000.0).expect("valid range"))
                    .step(1.0)
                    .precision(1)
                    .unit(" px")
                    .label_visible(false)
                    .reset_to(0.0)
                    .on_edit(record),
            ),
            ui::property_row(
                "不透明度",
                ui::number("不透明度", opacity)
                    .range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range"))
                    .step(0.01)
                    .precision(2)
                    .label_visible(false),
            ),
            ui::property_row(
                "フェード",
                ui::value_slider("フェード", opacity)
                    .range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range"))
                    .step(0.01)
                    .precision(2)
                    .label_visible(false),
            ),
            ui::property_row("塗り", ui::color_popup("塗り", fill)),
            ui::bezier_curve(easing),
            text(move || {
                let e = easing.get();
                format!("イージング {:.2}, {:.2}, {:.2}, {:.2}", e.x1, e.y1, e.x2, e.y2)
            }),
            row((
                button("X を元に戻す")
                    .disabled(move || history.get().is_empty())
                    .on_press(undo),
                text(move || format!("元に戻せる手順: {} 件", history.get().len())),
            ))
            .style(style().gap(8)),
        ))
        .style(style().gap(6).padding(10)),
    )
    .style(style().width(300))
}

fn main() -> kivact::Result<()> {
    App::new("レイヤーインスペクター")
        .theme(ui::theme(ThemeMode::Dark))
        .run(inspector)
}
```

実行してください。ステップ 1 のパネルに行が増え、2 つのハンドルを持つイージング曲線と、**X を元に戻す** ボタン（今は無効）が表示されます。ボタンの横には `元に戻せる手順: 0 件` と出ます。

## 試してみる

1. **スクラブ。** **X** フィールドを押したまま右へドラッグし、離します。値はポインターに合わせて変わりますが、表示は 1 ピクセルごとではなく、最後に 1 回だけ `元に戻せる手順: 1 件` になります。**X を元に戻す** を押すと、ドラッグを始めた位置の値に戻ります。
2. **キャンセル。** もう一度ドラッグし、離す前に `Esc` を押します。値はドラッグを始めた時点に戻り、件数は増えません。キャンセルした編集は元に戻す手順になりません。
3. **キーと入力。** `Tab` でフィールドにフォーカスして `→` を押します。このキー 1 回が 1 回の編集です。`Enter` でテキスト入力を開き、`10 + 5` と入力して `Enter` で確定すると、それがもう 1 回の編集です。`Backspace`（またはダブルクリック）で、`reset_to` に渡した `0` に戻ります。
4. **1 つの値を 2 つのエディタで。** **フェード** のスライダーをドラッグすると **不透明度** のフィールドが追従し、逆も同じです。どちらにも同じ `opacity` シグナルを渡していて、どちらも値のコピーを持たないので、同期のための処理は要りません。
5. **色。** **塗り** のスウォッチをクリックすると、カラーホイールのポップアップが開きます。ホイールをドラッグしてから `Esc` を押すと元の色に戻ります。ポップアップの外をクリックすると、新しい色が確定します。
6. **曲線。** イージング曲線の 2 つの丸いハンドルのどちらかをドラッグします。下の数値が追従します。ドラッグ中に `Esc` を押すと曲線は元に戻ります。ハンドルにフォーカスがあるときは、矢印キーで 0.01 ずつ（`Shift` で 0.1）動かせます。

## 書いたコードの説明

- **値はアプリが持ちます。** どの部品も、編集する値を引数で受け取ります。キットはデータを保存せず、渡されたシグナルを読み書きするだけです。React の制御コンポーネント（controlled component）と同じ考え方です。
- **`on_edit` は編集を 3 つの段階で報告します。** `EditPhase::Begin` は編集の最初の書き込みの*前*に届くので、開始時の値を覚えておく場所です。`Commit` は最後の書き込みの後に届き、元に戻す手順を確定する場所です。`Cancel` は、部品が開始時の値をすでに書き戻した後に届きます。元に戻すものはなく、覚えておいた値を捨てるだけです。ドラッグは長くても 1 回の編集で、キー 1 回、ホイール 1 目盛り、入力 1 回もそれぞれ 1 回の編集です。支援技術からの操作も同じ段階を通るので、これらを使った履歴は値を変えるあらゆる方法をカバーします。
- **`move` クロージャ。** `record` と `undo` は `move` を付けて、シグナルのハンドルを所有します。ハンドルは `Copy` なので、クロージャへ move した後でも同じ `x` を使えます。
- **`color_popup(label, color)`** は、押すと `color_wheel` のポップアップを開くスウォッチのボタンです。1 つのトランザクションとして働き、`Esc` で開いたときの色に戻り、外側のクリックで確定します。
- **`bezier_curve(binding)`** は `CubicBezier` を編集します。両端が (0, 0) と (1, 1) に固定されたイージング曲線の制御点です。これも値を引数で受け取り、`on_edit` で段階を報告します。
- **`value_slider`** は、塗りつぶしたトラックの形をした数値フィールドです。`ui::knob` と `ui::fader` は同じ部品の別の形で、同じ引数を取ります。つまみ付きの通常のスライダーは `kivact::ui::slider` です。

## 次へ

[3. ドックに置く](03-dock-and-outliner.md)
