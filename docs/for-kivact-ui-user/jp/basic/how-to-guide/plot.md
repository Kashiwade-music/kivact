# ズームできる軸の上にプロットを描く

[English](../../../en/basic/how-to-guide/plot.md)

このガイドでは、`kivact_ui_basic` で数値データを XY プロットとして描く方法を示します。タイトル、軸、系列を記述する `PlotModel`、ユーザーがズーム、移動、読み取りをできる `plot`、そしてユーザーが編集する範囲を扱います。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

プロットは数値を所有しません。系列のデータは `Vec<f64>` か配列で渡し、プロットは配列を参照で共有するので複製は安価です。プロットが描き直されるのは model か範囲が変わったときだけです。

## 折れ線を描く

2 つの軸から `PlotModel` を作り、`PlotSeries` を加えます。ラベルとともに `plot` へ渡します。範囲にバインディングを付けるまで、プロットは自分の範囲に従います（後述）。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::{AxisScale, PlotAxis, PlotModel, PlotSeries, TickFormat, plot};

fn response() -> impl IntoView {
    let frequencies: Vec<f64> = (1..=100).map(|i| f64::from(i) * 10.0).collect();
    let gain: Vec<f64> = frequencies
        .iter()
        .map(|f| -20.0 * (f / 500.0).log10().max(0.0))
        .collect();
    let model = PlotModel::new(
        PlotAxis::new("Frequency")
            .scale(AxisScale::Log10)
            .format(TickFormat::Si),
        PlotAxis::new("Gain").format(TickFormat::Decibel),
    )
    .title("Response")
    .series(PlotSeries::line("Gain", frequencies, gain).color(rgb(0xf2aa4c)));
    plot("Response", model).style(style().width(560))
}
```

- 系列の種類ごとにコンストラクタがあります。`PlotSeries::line`（折れ線）、`points`（円）、`stems`（下端から各点までの線）、`steps`（端点と値によるヒストグラム）、`heatmap`（端点と行優先の `z` によるセルの格子）です。どれも、名前の次にデータを取ります。
- データは `impl Into<PlotData>` です。`Vec<f64>`、`[1.0, 2.0]` のような配列、スライス（`&[f64]`、複製されます）を渡せます。イテレータは `.collect()` で作ります。
- セッターで系列を調整します。各セッターがどの種類に効くかを示します。`color` はすべての種類に効きます。`width` と `dash` は折れ線と stems に効きます。`radius` は点と stems の先の点に効きます（既定は 3 px）。`z_range` はヒートマップに効きます。
- `color` を指定しない系列は、系列の順番に対応するキットのチャートの色になるので、既定では系列ごとに色が違います。
- 軸は `AxisScale::Linear` か `AxisScale::Log10` です。対数軸では 0 以下の値は描きません。
- 目盛りの書式は `TickFormat::Plain`（有効数字 4 桁）、`Si`（`1.5k`、`10m`）、`Decibel`、`Degrees` から選びます。読み取りも同じ書式です。数字はアプリのロケールの桁と小数点で書かれます。

## ユーザーがズームとパンをできるようにする

プロットは、画像の役割を持つフォーカス可能な canvas です。名前はラベルで、説明には系列、軸のタイトル、表示中の範囲がロケールで書かれます。

- ドラッグするとパンします。内容はポインタに追従します。
- ホイールはポインタの位置を中心に x 軸をズームします。Ctrl を押すと y 軸をズームします。横方向のホイールは x 軸を移動します。多くのシステムでは、縦のホイールに Shift を押すと同じ動きになります。
- プロットにフォーカスがあるとき、`+` と `-` は x 軸を、PageUp と PageDown は y 軸をズームし、矢印キーは表示範囲の 10% ずつ移動します。`0` と Home は自動の範囲に戻します。
- ダブルクリックで自動の範囲に戻します。
- ポインタを重ねると、十字線、最も近い点の印、読み取りカードが出ます。折れ線は x が昇順のときだけ、最も近い x の点を読み取ります。

## 範囲をアプリの状態に持つ

ユーザーが選んだ範囲を読んだり、コードから設定したりするには、`PlotRange` の signal を `range` にバインドします。`PlotRange::Auto` はデータに合わせ、`PlotRange::Manual` は固定の `PlotView` を保持します。操作のたびにバインディングへ直ちに書き込まれ、別の確定用のコールバックはありません。

`on_edit` は操作を `EditPhase` として報告します。ドラッグは、ポインタが最初に動かしたときに編集を始め、ボタンを離したときに確定します。Escape やポインタのキャンセルは、編集を始めた範囲に戻し、`Cancel` を報告します。ホイールの一回の操作、キー、ダブルクリックは、始まりと確定が同時に起こる 1 回の編集です。

```rust,no_run
use kivact::prelude::*;
use kivact::view::EditPhase;
use kivact_ui_basic::{PlotModel, PlotRange, plot};

fn watched(model: PlotModel) -> impl IntoView {
    let range = signal(PlotRange::Auto);
    plot("Response", model)
        .range(range)
        .on_edit(move |phase| match phase {
            EditPhase::Begin => {}
            EditPhase::Commit => {
                // ここでは `range` に新しい PlotRange::Manual が入っています。
                // たとえばセッションファイルへ保存します。
            }
            EditPhase::Cancel => {}
        })
}
```

## ヒートマップを描く

ヒートマップは `PlotSeries::heatmap(name, x_edges, y_edges, z)` で作る系列 1 つです。`x_edges` は列数より 1 つ多い値を、`y_edges` は行数より 1 つ多い値を持ちます。値 `z` は行優先で、行数 × 列数の値が必要です。それ以外の長さの `z` は何も描きません。行と列の数は端点から決まるので、渡すのは端点と値だけです。

`z_range(min, max)` は `min` と `max` をキットのトークンのビリディス配色（`Tokens::heatmap`）に対応させます。指定しなければ、`z` の有限な最小と最大を使います。NaN のセルは透明になり、範囲の外の値は近い端の色になります。端点は昇順でも降順でもかまいません。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::{PlotAxis, PlotModel, PlotSeries, plot};

fn grid() -> impl IntoView {
    let (rows, cols) = (4_u32, 6_u32);
    let x_edges: Vec<f64> = (0..=cols).map(f64::from).collect();
    let y_edges: Vec<f64> = (0..=rows).map(f64::from).collect();
    let z: Vec<f64> = (0..rows * cols).map(f64::from).collect();
    let model = PlotModel::new(PlotAxis::new("Column"), PlotAxis::new("Row"))
        .legend(false)
        .series(PlotSeries::heatmap("Value", x_edges, y_edges, z).z_range(0.0, 23.0));
    plot("Grid", model).style(style().width(480))
}
```

## プロットを書き出す

`PlotModel::draw` は model を、ある範囲で、ポインタなしに任意の `Draw` へ描きます。`Offscreen` の描画に渡せば、同じ絵を画像にも SVG の文書にも書き出せます。色はその時点のテーマのキットのトークンで、数字は渡したロケールで書かれます。`PlotRange::Auto` はデータに合う範囲、`PlotRange::Manual` は固定の `PlotView` を表示します。

```rust,no_run
use kivact::draw::Offscreen;
use kivact::i18n::Locale;
use kivact::style::ThemeMode;
use kivact_ui_basic::{PlotModel, PlotRange};

fn export(model: &PlotModel) -> kivact::Result<String> {
    let locale = Locale::new("ja-JP");
    let mut renderer = Offscreen::new().theme(kivact_ui_basic::theme(ThemeMode::Light));
    renderer.render_svg(640.0, 360.0, |draw, size| {
        model.draw(draw, size, PlotRange::Auto, &locale);
    })
}
```

同じ `Offscreen` で、この絵の PNG のピクセルを `render` で書き出すこともできます。テーマとフォントは両方で一つを共有するためです。描画の書き出しについては、[キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md)を参照してください。

## 読みやすくする

- すべての軸にラベルを付けてください。タイトルのないプロットは、タイトルの場所を取りません。
- 系列の数は、右上の凡例に収まる程度にしてください。
- 目盛りは横に 80 px、縦に 50 px ごとに 1 本程度です。1 桁未満の対数軸は、その値の線形の目盛りを使います。
- 読み取りカードは、x と y の値を軸のタイトルで示します。ヒートマップのセルの値は `basic.plot.value` のメッセージ（既定は `Value`）で示され、支援技術向けの説明はロケールで書かれます。

## 関連項目

- [データを表示する](data-display.md): 表、グラフ、バッジ。
- [キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md): プロットが使う描画の語彙と、描画の書き出し。
