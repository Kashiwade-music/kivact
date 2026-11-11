# 色・イージング曲線・グラデーションを編集する

[English](../../../en/creative/how-to-guide/color-curves-and-gradients.md)

目的: 色、イージング曲線、グラデーションを利用者が編集できるようにします。値は自分のシグナルに持ち、編集ごとに元に戻す手順を 1 つ記録します。

このガイドの 3 つのエディタは、バインディングで渡された*1 つの値*を編集します。値は `Color`、`CubicBezier`、`Vec<GradientStop>` です。エディタは値のコピーを持たないので、自分のシグナルから読む値は常に最新です。このガイドは、[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## 色を選ぶ

最小の解決方法は `color_popup` です。押すとカラーホイールがポップアップで開く、スウォッチのボタンです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn fill_picker() -> impl IntoView {
    let fill = signal(rgb(0x8b7cff));
    ui::property_row("塗り", ui::color_popup("塗り", fill))
}
```

`Esc` を押すとポップアップが閉じ、開いたときの色に戻ります。外側をクリックすると閉じて、変更が確定します。スウォッチは常に現在の色を表示し、アクセシビリティ上の名前には渡したラベルが使われます。

常に表示しておくエディタには `color_wheel` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::view::EditPhase;
use kivact_ui_creative as ui;

fn grading_wheel() -> impl IntoView {
    let color = signal(rgb(0x8b7cff));
    let model = signal(ui::ColorModel::Rgb);
    let palette = signal(vec![rgb(0xffffff), rgb(0x000000)]);
    let edits = signal(0);

    column((
        ui::color_wheel(color, model)
            .wheel_size(200.0)
            .alpha(false)
            .swatches(palette)
            .on_edit(move |phase| {
                if phase == EditPhase::Commit {
                    edits.update(|n| *n += 1);
                }
            }),
        text(move || format!("色の編集 {} 回", edits.get())),
    ))
    .style(style().gap(8).padding(12).width(260))
}
```

`color_wheel(color, model)` は 2 つのバインディングを取ります。`color` が値で、`model` は数値フィールドが表示するチャンネルの種類（`ColorModel::Rgb`・`Hsv`・`Hex`）です。`model` を切り替えても変わるのはフィールドの表示だけで、色は変わりません。専用のシグナルに持つか、複数のホイールで 1 つのシグナルを共有して、同じモデルを表示させてください。

| オプション | 効果 |
|------------|------|
| `.wheel_size(px)` | ホイール領域の高さ（論理ピクセル）です。最小は 96 で、既定は 220 です。 |
| `.alpha(false)` | アルファのストリップとフィールドを隠します。色のアルファはそのまま保たれます。 |
| `.fields(false)` | モデルの選択欄と数値フィールドを隠します。 |
| `.swatches(binding)` | 利用者が色を追加・削除できるパレットです。スウォッチの選択は編集ではありません。 |
| `.on_edit(f)` | ドラッグ・キー操作・入力ごとの[`EditPhase`](#編集ごとに元に戻す手順を-1-つ記録する)です。 |

`alpha`・`fields`・`wheel_size` は、シグナルやクロージャも受け取るので、アプリの実行中に変えられます。彩度がゼロになったとき、ホイールは最後の色相を覚えているので、グレーから外へドラッグしても色相が飛びません。

## 編集ごとに元に戻す手順を 1 つ記録する

このガイドのエディタは、数値フィールドと同じく、`on_edit` で編集を 3 つの段階として報告します。

- `EditPhase::Begin` は最初の書き込みの前に届きます。ここで値を覚えておきます。
- `EditPhase::Commit` は最後の書き込みの後に届きます。ここで元に戻す手順を確定します。
- `EditPhase::Cancel` は、エディタが開始時の値を書き戻した後に届きます（利用者が `Esc` を押した、または編集の途中でエディタが隠された・取り除かれた場合です）。覚えておいた値を捨てます。バインディングにはすでに元の値が入っています。

ドラッグは、押してから離すまでが 1 回の編集です。キー 1 回、入力 1 回もそれぞれ 1 回の編集で、支援技術からの操作も同じ段階を通ります。次は、`Clone` できる任意の値に使える小さな履歴です。

```rust,no_run
use kivact::prelude::*;
use kivact::view::EditPhase;

/// `value` の `(編集前, 編集後)` の組を記録する。
fn history<T: Clone + PartialEq + 'static>(
    value: Signal<T>,
) -> (impl Fn(EditPhase) + 'static, Signal<Vec<(T, T)>>) {
    let start = signal(None::<T>);
    let steps = signal(Vec::<(T, T)>::new());
    let on_edit = move |phase: EditPhase| match phase {
        EditPhase::Begin => start.set(Some(value.get_untracked())),
        EditPhase::Commit => {
            if let Some(before) = start.update(|saved| saved.take()) {
                steps.update(|all| all.push((before, value.get_untracked())));
            }
        }
        EditPhase::Cancel => start.set(None),
    };
    (on_edit, steps)
}
```

返ってきた `on_edit` を、同じ値を編集するエディタに渡します。たとえば `color_wheel(color, model).on_edit(on_edit)` です。元に戻すときは、最後の組を取り出して `before` をシグナルに書き込みます。

## イージング曲線を編集する

`bezier_curve` は `CubicBezier` を編集します。固定された両端 (0, 0) と (1, 1) の間にある、3 次イージング曲線の 2 つの内側の制御点です。`curve_preview` は読み取り専用のサンプルを描き、`progress` はアニメーションのコードで曲線の値を計算します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn easing_editor() -> impl IntoView {
    let easing = signal(ui::CubicBezier::new(0.25, 0.1, 0.25, 1.0));
    column((
        ui::bezier_curve(easing),
        ui::curve_preview(move || easing.get().samples(64)).style(style().size(240, 120)),
        text(move || format!("中間点: {:.3}", easing.get().progress(0.5))),
    ))
    .style(style().gap(8).padding(12))
}
```

2 つのハンドルをドラッグするか、1 つにフォーカスして矢印キーで動かします（1 回で `0.01`、`Shift` で `0.1`、`Ctrl` で `0.001`）。エディタとプレビューは同じ `easing` を読むので、プレビューは編集のたびに追従します。

- `CubicBezier::new(x1, y1, x2, y2)` は `x1` と `x2` を `0..=1` に収めるので、時間は必ず前へ進みます。`y1` と `y2` は `0..=1` の外へ出せます。そのとき曲線は行き過ぎ（オーバーシュート）を持ち、バウンスや予備動作のイージングはこれで作ります。
- `progress(t)` は、`0..=1` の時間 `t` に対するイージング後の値を返します。範囲外の時間はその範囲に収められます。結果は制限されないので、オーバーシュートが残ります。アニメーションでは、正規化した時間で `start + (end - start) * easing.progress(t)` のように計算します。
- `samples(n)` は、等間隔の `n` 個の点を返します（2〜513）。`curve_preview` は任意の `Point` のリストを受け取るので、ほかの曲線を描くこともできます。
- バインディングの値が不正でも（たとえば壊れたファイル由来の `NaN`）、描画のために整えられますが、利用者が編集するまであなたのシグナルは書き換えられません。

## グラデーションを編集する

`gradient_editor(stops, selected)` は、ドラッグできるストップを持つカラーランプを表示し、選択中のストップには位置のフィールドとカラーホイールを表示します。

```rust,no_run
use kivact::draw::linear_gradient;
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn gradient_panel() -> impl IntoView {
    let stops = signal(vec![
        ui::GradientStop::new(0.0, rgb(0x1b1c22)),
        ui::GradientStop::new(1.0, rgb(0x8b7cff)),
    ]);
    // `stops` の中で選択中のストップのインデックス（なければ `None`）。
    let selected = signal(Some(0));

    column((
        ui::gradient_editor(stops, selected).name("背景のグラデーション"),
        // ストップの使い方の例: プレビューの帯。
        node().style(move || {
            let mut ordered = stops.get();
            ordered.sort_by(|a, b| a.position.total_cmp(&b.position));
            let mut ramp = linear_gradient(90);
            for stop in ordered {
                ramp = ramp.stop(stop.color, pct(stop.position * 100.0));
            }
            style().height(24).radius(6).background(ramp)
        }),
    ))
    .style(style().gap(10).padding(12).width(300))
}
```

- **ストップに ID はありません。** ストップは、ベクタの中のインデックスで識別します。`selected` はそのインデックス（または `None`）です。ストップを追加・削除すると、選択中のインデックスはエディタが調整します。
- **ベクタは並べ替えられません。** エディタは位置の順にストップを描きますが、データの中の順序には手を触れません。順序が必要なときは、上のプレビューのようにコピーを並べ替えてください。
- **キー操作:** ストップにフォーカスがあるとき、`←`/`↓` と `→`/`↑` で `0.01` ずつ動かし（`Shift` で `0.1`）、`Home` と `End` で `0` と `1` へ送り、`Delete` か `Backspace` で削除します。ストップの追加と削除は、エディタのボタンでもできます。
- **選択の変更は編集ではありません。** `on_edit` が呼ばれるのは、移動・追加・削除・色の変更だけです。
- **色空間。** 保存する色が画面の色空間と違う場合は、`.color_transform(to_display, from_display)` で、ストップを表示するときと書き戻すときに変換できます。

## 関連項目

- [アプリが描いたフレームを表示する](show-app-rendered-images.md): 自分で描いたヒストグラムやスコープを、カラーホイールの隣に置けます。
- [Creative キットのテーマ・フォント・アイコンを設定する](theme-and-icons.md)
- [チュートリアル ステップ 2: 値を編集する](../getting-started/02-edit-values.md): 数値フィールドでの同じ `EditPhase`。
- API の全体: `cargo doc -p kivact-ui-creative --open`
- カタログのグレーディングとアニメーションの章: `cargo run -p kivact-ui-creative-catalog`
