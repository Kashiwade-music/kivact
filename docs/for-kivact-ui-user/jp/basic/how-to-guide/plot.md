# ズームできる軸の上にプロットを描く

[English](../../../en/basic/how-to-guide/plot.md)

このガイドでは、`kivact_ui_basic::plot` で数値データを XY プロットとして描く方法を示します。タイトル、軸、系列を記述する model、ユーザーがズームとパンをできる plot、そして報告される範囲を扱います。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

plot は数値を所有しません。`PlotData` は配列を参照で共有するので複製は安価で、plot が描き直されるのは model か範囲が変わったときだけです。

## 折れ線を描く

`PlotModel` を 1 つの `PlotSeries` で作り、ラベルと範囲とともに `plot` へ渡します。範囲はバインディングです。`PlotRange::Auto` はデータに合わせ、`PlotRange::Manual` は固定します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::plot::{
    AxisScale, LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotModel, PlotRange,
    PlotSeries, PlotSeriesKind, TickFormat, plot,
};

fn response() -> impl IntoView {
    let frequencies: Vec<f64> = (1..=100).map(|i| f64::from(i) * 10.0).collect();
    let gain: Vec<f64> = frequencies.iter().map(|f| -20.0 * (f / 500.0).log10().max(0.0)).collect();
    let model = PlotModel {
        title: "Response".into(),
        x: PlotAxis {
            label: "Frequency".into(),
            scale: AxisScale::Log10,
            format: TickFormat::Si,
        },
        y: PlotAxis {
            label: "Gain".into(),
            scale: AxisScale::Linear,
            format: TickFormat::Decibel,
        },
        series: vec![PlotSeries {
            name: "Gain".into(),
            color: PlotColor::Palette(0),
            width: 2.0,
            dash: LineDash::Solid,
            kind: PlotSeriesKind::Line {
                x: PlotData::new(frequencies),
                y: PlotData::new(gain),
            },
        }],
        legend: LegendPosition::TopRight,
    };
    let range = signal(PlotRange::Auto);
    plot("Response", model, range).style(style().width(560))
}
```

- `PlotSeriesKind` には 5 種類あります。`Line`（折れ線）、`Points`（半径つきの円）、`Stems`（下端から各点までの線）、`Steps`（端点と値によるヒストグラム）、`Heatmap`（端点と行優先の `z` によるセルの格子）です。
- 系列の色は、テーマのチャートの色（`PlotColor::Palette(i)`）か、`0xRRGGBB` の固定色（`PlotColor::Rgb`）から選びます。
- 軸は線形か `Log10` です。対数軸では 0 以下の値は描きません。
- 目盛りの書式は `Plain`（有効数字 4 桁）、`Si`（`1.5k`、`10m`）、`Decibel`、`Degrees` から選びます。読み取りも同じ書式です。

## ユーザーがズームとパンをできるようにする

plot はラベルを名前にした canvas で、その region は支援技術向けに `"<ラベル>: <系列数> series"` と名付けられます。

- 主ボタンでドラッグするとパンします。内容はポインタに追従します。
- ホイールはポインタの位置を中心に x 軸をズームします。Ctrl を押すと y 軸、Shift を押すと x 軸を移動します。
- ダブルクリックで範囲を `PlotRange::Auto` に戻します。
- ポインタを重ねると十字線、最も近い点の印、読み取りカードが出ます。折れ線は x が昇順のときだけ、最も近い x の点を読み取ります。

ドラッグは離した時点で確定し、ホイールの一連の操作は最後の操作から 300 ms 後に確定します。確定のたびに新しい `PlotRange::Manual` がバインディングに書き込まれ、`on_range_commit` が呼ばれます。範囲を保存したり、ほかへ送ったりできます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::plot::{PlotModel, PlotRange, plot};

fn watched(model: PlotModel) -> impl IntoView {
    let range = signal(PlotRange::Auto);
    plot("Response", model, range).on_range_commit(move |committed| {
        // `committed` を保存します。たとえばセッションファイルへ。
        let _ = committed;
    })
}
```

操作の途中では plot はライブの表示で追従し、ジェスチャーが終わるまでバインディングには書き込みません。

## ヒートマップを描く

ヒートマップは `PlotSeriesKind::Heatmap` の系列 1 つです。`x_edges` は `cols + 1` 個、`y_edges` は `rows + 1` 個の値を持ちます。値 `z` は行優先で、`z_min`..`z_max` がテーマのビリディス配色に対応します。NaN のセルは透明になり、`z_min`..`z_max` の外の値は近い端の色になります。自動の範囲は端点に合わせ、端点は昇順でも降順でもかまいません。

## プロットを書き出す

`draw_plot` は model を任意の `Draw` へ、ポインタなしで描きます。同じ絵を `Offscreen` の画像や `Svg` の文書へ送れます（[キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md)を参照）。データに合う範囲は `auto_view` で得られます。

## 読みやすくする

- すべての軸にラベルを付けてください。タイトルのないプロットは、タイトルの場所を取りません。
- 系列の数は、右上の凡例に収まる程度にしてください。
- 目盛りは横に 80 px、縦に 50 px ごとに 1 本程度です。1 桁未満の対数軸は、その値の線形の目盛りを使います。

## 関連項目

- [データを表示する](data-display.md): 表、グラフ、バッジ。
- [キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md): プロットが使う描画の語彙。
