# 画面をレイアウトする

[English](../../en/how-to-guide/layout.md)

子を並べるには `row` と `column`、大きさや余白を決めるには `style()`、ウィンドウより長くなりうる内容には `scroll` を使います。このガイドでは、サイズと単位、伸び縮みする余白、配置、折り返し、スクロール、重ね合わせ、グリッドを扱い、最後に 2 ペインの画面を作ります。

どの例も `use kivact::prelude::*;` を前提にしています。prelude にない長さやレイアウト用の型は、使う場所で import します。

## 最短で動くレイアウト

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    column((
        text("設定"),
        row((button("キャンセル"), button("保存"))).style(style().gap(8)),
    ))
    .style(style().padding(16).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Layout").run(app)
}
```

`column` は子を上から下へ、`row` は左から右へ並べます。どちらも、1 つの view、view のタプル(最大 16 個、それぞれ別の型でかまいません)、または同じ型の view の `Vec` を受け取ります。`gap` は子の間隔、`padding` は箱の内側の余白です。数値は論理ピクセルなので、高密度ディスプレイでも同じ見た目になります。

## コンテナを選ぶ

| やりたいこと | 使うもの |
| --- | --- |
| 子を縦に並べる | `column((a, b))` |
| 子を横に並べる | `row((a, b))` |
| 行と列のセルに並べる | `grid((a, b, c))` と `grid_template_columns` |
| 子を重ねる | `stack((a, b))` |
| スクロールさせる | `scroll(child)` |
| 伸びる空白 | `spacer()` |
| 自分でスタイルする素の箱 | `node().child(a)` |

`node()` はこれらすべての土台になる箱です。`node()` も縦方向の flex box なので、`column(x)` は `node().child(x)` に縦方向の指定を加えたものです。

## サイズと単位

数値は論理ピクセルの長さです。それ以外は長さのヘルパーを使います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Length, auto};

fn app() -> impl IntoView {
    column((
        // 固定サイズ。
        node().style(style().width(120).height(32)),
        // 親の幅の半分。
        node().style(style().width(pct(50)).height(32)),
        // 親の幅から 40 px を引いた幅(`calc(100% - 40px)`)。
        node().style(style().width(pct(100) - 40).height(32)),
        // 「内容に必要なだけ」に戻す。
        node().style(style().width(auto()).height(32)),
        // 内容の幅。ただし使える幅を超えない。
        node().style(style().width(Length::FitContent).height(32)),
        // 下限と上限: 最小 100 px、最大 400 px。
        node().style(style().min_width(100).max_width(400).height(32)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Sizes").run(app)
}
```

- `pct(50)` は親に対する割合です。`pct(100) - 40` や `pct(50) + 8` は、割合とピクセルを組み合わせた calc の長さです。
- `width`、`height`、`min_*`、`max_*`、`padding`、`margin`、`gap`、`basis` はすべて長さを受け取ります。`padding` と `margin` には `padding_x`、`padding_y` と、`padding_top` のような 1 辺だけの版があります。
- `Length::MinContent`、`MaxContent`、`FitContent` は内容に合わせて大きさを決めます。使えるのは `width`、`height`、`basis` とグリッドのトラックだけです。
- `aspect_ratio(16.0 / 9.0)` は一方の辺からもう一方を決めます。
- `style()` の各メソッドは新しい値を返すので、1 つの式でつなげて書きます。同じプロパティを 2 回指定すると最後の値が残ります(デバッグビルドでは重複が報告されます)。

## 固定サイズと可変サイズ

既定では、子は内容に必要な大きさになり、親が小さすぎれば縮みます。これを変えるプロパティが 3 つあります。

- `grow(n)` は主軸方向の余りを分け与えます。`grow(1)` の子は余りをすべて取り、`grow(1)` と `grow(3)` の 2 つの子は 1 対 3 で分け合います。
- `shrink(0)` は、子が指定サイズより小さくならないようにします。
- `spacer()` は `grow(1)` を持つ空の箱で、後ろに続くものを端へ押しやります。

```rust,no_run
use kivact::prelude::*;

fn toolbar() -> impl IntoView {
    row((
        text("無題"),
        spacer(),
        button("元に戻す"),
        button("やり直す"),
    ))
    .style(style().gap(8).padding(8))
}

fn main() -> kivact::Result<()> {
    App::new("Toolbar").run(toolbar)
}
```

`text("無題")` は左端に残り、2 つのボタンは右端に寄り、間のすべてを spacer が占めます。

## 配置

箱の中で子を配置するプロパティは 2 つです。

- `justify_content(..)` は子を**主軸**方向に配分します。column では上から下、row では左から右です。
- `align_items(..)` は**交差軸**方向にそろえます。column では左右、row では上下です。

`align_items` の既定は `Align::Stretch` なので、column の子は column と同じ幅に、row の子は row と同じ高さに伸びます。column の中のボタンが、何も指定しないと全幅に広がるのはこのためです。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    column((
        // 高さ 240 px の箱の中で、縦横ともに中央。
        column((text("こんにちは"),))
            .style(style().height(240).align_items(Align::Center).justify_content(Justify::Center)),
        // 1 つ目のボタンは本来の幅のまま。2 つ目は伸びる。
        column((
            button("本来の幅").style(style().align_self(Align::Start)),
            button("全幅"),
        ))
        .style(style().gap(8)),
        // 均等に広げる。
        row((text("A"), text("B"), text("C"))).style(style().justify_content(Justify::SpaceBetween)),
    ))
    .style(style().gap(16).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Alignment").run(app)
}
```

`Align` には `Start`、`Center`、`End`、`Stretch`、`Baseline` があります(row では文字の最初のベースラインをそろえます)。`Justify` には `Start`、`Center`、`End`、`SpaceBetween`、`SpaceAround`、`SpaceEvenly`、`Stretch` があります。子に `align_self` を付けると、その子だけ親の `align_items` を上書きします。

## 折り返し

row が狭すぎると、ふつうは子がはみ出します。`wrap(true)` を指定すると、子は次の行へ回ります。`gap` は縦横の両方に効きます(`row_gap` と `column_gap` で別々に指定できます)。

```rust,no_run
use kivact::prelude::*;

fn tags() -> impl IntoView {
    row(vec![
        button("Rust"), button("GUI"), button("シグナル"),
        button("ネイティブ"), button("Windows"), button("Linux"),
    ])
    .style(style().wrap(true).gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Wrap").min_window_size(200, 200).run(tags)
}
```

ウィンドウを狭くすると、ボタンはより多くの行に分かれて並びます。

文字は、与えられた幅の中で自動的に折り返します。1 行に収め、狭いときは「…」で終わらせたいラベルには `style().single_line()` を使います。row の中では、その行の子にも `min_width(0)` が必要です。flex の子は、そうしないと内容より小さくなりません。

## スクロール

`scroll(child)` は、子がスクロール領域より高いとき、縦にスクロールさせます。スクロール領域には**大きさの上限**が必要です。高さを与えるか、高さが決まっている親に伸ばしてもらいます。

```rust,no_run
use kivact::prelude::*;

fn log() -> impl IntoView {
    let lines: Vec<_> = (1..=100).map(|n| text(format!("{n} 行目"))).collect();
    scroll(column(lines)).style(style().height(200))
}

fn strip() -> impl IntoView {
    scroll(row(vec![button("一"), button("二"), button("三")]))
        .axes(ScrollAxes::Horizontal)
        .style(style().width(160))
}

fn table() -> impl IntoView {
    scroll(column(vec![text("横にも縦にも大きい表")]))
        .axes(ScrollAxes::Both)
        .style(style().size(300, 200))
}

fn main() -> kivact::Result<()> {
    App::new("Scroll").run(|| column((log(), strip(), table())).style(style().gap(12)))
}
```

- 既定は縦スクロールです。`.axes(ScrollAxes::Horizontal)` は内容を row に並べて横にスクロールし、`.axes(ScrollAxes::Both)` は縦横の両方にスクロールします。
- スクロール領域のアクセシビリティ上の向きは軸に従います（縦、横、両方なら指定なし）。そこに付けた `.orientation(..)` が軸と食い違うときは、報告されて無視されます。
- `scroll` は `Scroll` ビルダーを返します。`.style(..)`、`.name(..)` などの node 用メソッドは、スクロール領域そのものに効きます。
- ホイールとタッチのドラッグでスクロールします。端まで来ると、ホイールは親へ引き継がれます。
- 何百行もある場合は、見えている行だけをマウントする `virtual_list` を使います([一覧と表](lists-and-tables.md)を参照)。
- flex の column の中で、スクロール領域に残りの空間を埋めさせたいときは、`grow(1)` と `min_height(0)` が必要です。最小値を指定しないと、内容の高さが領域の縮小を妨げます。

## 重ね合わせ

`stack` は子を重ねます。最初の子が大きさを決め、後の子は同じ箱を埋めます。

```rust,no_run
use kivact::prelude::*;

fn badge() -> impl IntoView {
    stack((
        node().style(style().size(64, 64).background(ColorToken::SurfaceAlt)),
        text("3").style(style().color(ColorToken::Danger).padding(4)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Stack").run(badge)
}
```

画面の上に浮かぶべきメニューやポップアップには、overlay 系のコンポーネントを使います([オーバーレイ](overlays.md)を参照)。stack は 1 つの箱の中の層のためのものです。

## グリッド

`grid` は、定義したトラックのセルに子を配置します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::Track;

fn form_grid() -> impl IntoView {
    grid((
        text("名前"), text_input("名前", signal(String::new())),
        text("メール"), text_input("メール", signal(String::new())),
    ))
    .style(
        style()
            .grid_template_columns((96, Track::fr(1)))
            .gap(8)
            .padding(16),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Grid").run(form_grid)
}
```

`(96, Track::fr(1))` は、幅 96 px の列と、残りを取る列です。子は順にセルを埋めます。`Track::repeat_auto_fill(Track::minmax(160, Track::fr(1)))` は、幅 160 px 以上の列を入るだけ作ります。セルをまたぎたい子には、`kivact::style::GridPlacement` を使って `style().grid_column(GridPlacement::span(2))` と書きます。

## 2 ペインの画面

幅が固定のサイドバーと、残りの幅を取ってスクロールするメイン領域です。

```rust,test_harness
use kivact::prelude::*;

fn sidebar() -> impl IntoView {
    column((text("受信箱"), text("送信済み"), text("アーカイブ")))
        .style(style().width(200).padding(12).gap(8).background(ColorToken::SurfaceAlt))
        .test_id("sidebar")
}

fn content() -> impl IntoView {
    let items: Vec<_> = (1..=60).map(|n| text(format!("メッセージ {n}"))).collect();
    scroll(column(items).style(style().padding(12).gap(8)))
        .style(style().grow(1).min_width(0))
        .test_id("content")
}

fn app() -> impl IntoView {
    row((sidebar(), content())).style(style().height(pct(100)))
}

fn main() -> kivact::Result<()> {
    App::new("Mail").window_size(900, 600).run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidebar_is_fixed_and_the_content_takes_the_rest() {
        let screen = kivact::test::render(app);
        let sidebar = screen.get_by_test_id("sidebar").rect().expect("laid out");
        let content = screen.get_by_test_id("content").rect().expect("laid out");
        assert_eq!(sidebar.width, 200.0);
        assert_eq!(content.x, 200.0);
        // 既定のテストウィンドウの幅は 800 px。
        assert_eq!(content.width, 600.0);
        assert_eq!(content.height, sidebar.height);
    }
}
```

サイドバーの `width(200)` は固定です。既定の `Align::Stretch` により、サイドバーは row の高さまで伸びます。メイン領域は `grow(1)` で残りの幅を取り、`min_width(0)` により狭いウィンドウでも縮められます。row 全体を `pct(100)` の高さにしているのは、スクロール領域がスクロールするための上限の高さを持てるようにするためです。

手で開かずにレイアウトを確かめたいときは、テストで `Screen::assert_layout` を使います。文字が箱からはみ出す、ボタンの中で折り返す、中央からずれる、省略記号なしで切れる、といった問題を報告します。[アプリをテストする](testing.md)を参照してください。

## 関連項目

- [一覧と表](lists-and-tables.md): 長い、または選択できるコレクション。
- [オーバーレイ](overlays.md): レイアウトの上に浮かぶダイアログ、メニュー、ツールチップ。
- [テーマ](theming.md): 色、余白、hover や focus の見た目。
- リファレンス: `cargo doc -p kivact-style --open` で `Style` のすべての設定メソッドを確認できます。

タッチスクロールは指を離した後も platform の設定に従って減速します。`.scroll_snap(ScrollSnap::Start)`、`Center`、`End` を指定すると、停止時に直下の通常フローの子の位置へ揃えます。`Display::Contents` の子は展開されます。`ScrollSnap::None` は snap を無効にします。`ScrollSnap` は `kivact::style` から import します。プログラムからのスクロールは `ScrollBehavior::Smooth(Easing::EaseOut)` などの easing と `ScrollBehavior::Instant` を受けます。reduced motion では目標位置へ即座に移動します。
