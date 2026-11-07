# キャンバスで描く

[English](../../en/how-to-guide/canvas.md)

標準のレイアウト要素では表現しにくいもの、たとえばグラフ、図、ゲームの盤面、独自のメーター、画像エディターを表示したいときにキャンバスを使います。図形、パス、テキスト、画像を普通の関数呼び出しで描くと、読んでいるシグナルが変わったときに Kivact が再描画します。このガイドでは、描いた部分をクリック可能にしてスクリーンリーダーにも伝える方法、自前のレンダラーのフレームを表示する方法、ウィンドウなしで画像を書き出す方法も説明します。

## 図形を描く

`canvas` は `Draw` と、そのノードの現在の `Size` を受け取るクロージャを取ります。座標は論理ピクセルで、キャンバス自身の左上を原点にします。

```rust,no_run
use kivact::geometry::Rect;
use kivact::prelude::*;

fn badge() -> impl IntoView {
    canvas(|draw, size| {
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x20242c));
        draw.circle(Point::new(48.0, 48.0), 20, rgb(0x6699ff));
        draw.rounded_rect(Rect::new(96.0, 28.0, 120.0, 40.0), 8.0, ColorToken::Accent);
        draw.line(
            Point::new(12.0, 100.0),
            Point::new(size.width - 12.0, 100.0),
            2,
            rgb(0xffffff),
        );
    })
    .style(style().size(240, 120))
    .name("暗い背景に青い円とアクセント色の帯")
}

fn main() -> kivact::Result<()> {
    App::new("Canvas").run(badge)
}
```

- キャンバスは自分の大きさを持ちません。`.style(style().size(w, h))` で大きさを与えるか、レイアウトに引き伸ばさせ、実際の大きさはクロージャの 2 番目の引数から読みます。最初の描画は大きさ 0 で行われ、レイアウト後にもう一度描かれるので、描画は常に `size` を基準にしてください。
- キャンバスには `.name(..)` を付けます。スクリーンリーダーがその絵として読み上げる文字列になります。
- 色は `rgb(0xrrggbb)`、`rgba(0xrrggbb, alpha)`、または `ColorToken::Accent` のようなテーマトークンで指定します。トークンはアクティブなテーマ（ライト、ダーク、独自テーマ）に従います。`Paint` に変換できるものは、ペイントを受け取る場所ならどこでも使えます。色、トークン、グラデーションのいずれも渡せます。
- 線は幅（`2` は 2 ピクセル幅の実線）か、端・接合・破線を指定できる `Stroke` で指定します。

## パスを描く

`Path` は、移動と線分をつないだものです。`fill_path` は塗りつぶし、`stroke_path` は輪郭線を描きます。

```rust,no_run
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::prelude::*;

fn wave() -> impl IntoView {
    canvas(|draw, size| {
        let mid = size.height / 2.0;
        let wave = Path::new()
            .move_to(Point::new(8.0, mid))
            .cubic_to(
                Point::new(size.width * 0.3, mid - 40.0),
                Point::new(size.width * 0.6, mid + 40.0),
                Point::new(size.width - 8.0, mid),
            );
        let stroke = Stroke::new(4).cap(LineCap::Round).join(LineJoin::Round);
        draw.stroke_path(&wave, &stroke, ColorToken::Accent);

        let triangle = Path::new()
            .move_to(Point::new(20.0, 20.0))
            .line_to(Point::new(60.0, 20.0))
            .line_to(Point::new(40.0, 50.0))
            .close();
        draw.fill_path(&triangle, rgb(0xe5484d));
        draw.stroke_path(&triangle, &Stroke::new(2).dash([4.0, 3.0]), rgb(0xffffff));
    })
    .style(style().size(240, 120).background(rgb(0x20242c)))
    .name("波と三角形")
}
# fn main() -> kivact::Result<()> { App::new("Paths").run(wave) }
```

`Path` には `quad_to`・`arc`・`ellipse`・`svg_arc_to`（SVG の `A`）・`rect`・`rounded_rect` もあります。角度は度で、0 は右向きです。パスは既定では非ゼロ規則で塗られます。穴を空けたいときは `path.with_fill_rule(FillRule::EvenOdd)` を使います。単純な図形なら、パスを組むより `draw.circle` や `draw.fill_shape(rect, &Shape, paint)` のほうが簡潔です。

`arc_to(p1, p2, radius)` は canvas の角を丸め、`Path::parse_svg(data)?` は SVG のパスデータを読み込みます。`contains(point, rule)` は塗り、`stroke_contains(point, &stroke)` は線の内部を判定します。

## グラデーションで塗る

グラデーションには 2 種類あり、色の置き方が違います。

```rust,no_run
use kivact::draw::{CanvasGradient, linear_gradient};
use kivact::geometry::Rect;
use kivact::prelude::*;

fn gradients() -> impl IntoView {
    canvas(|draw, size| {
        // 1. CSS と同じく、塗る図形を基準にします。
        let sky = linear_gradient(180).colors([rgb(0x1e3a8a), rgb(0x93c5fd)]);
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, 60.0), sky);

        // 2. HTML の canvas と同じく、座標で配置します。同じグラデーションを
        //    使った 2 つの図形は、同じ位置で同じ色になります。
        let ramp = CanvasGradient::linear(Point::new(0.0, 0.0), Point::new(size.width, 0.0))
            .stop(0.0, rgb(0xe5484d))
            .stop(1.0, rgb(0x3e63dd));
        draw.fill_rect(Rect::new(0.0, 70.0, size.width, 20.0), ramp.clone());
        draw.circle(Point::new(size.width / 2.0, 110.0), 16, ramp);
    })
    .style(style().size(240, 130))
    .name("グラデーションの見本")
}
# fn main() -> kivact::Result<()> { App::new("Gradients").run(gradients) }
```

`linear_gradient`・`radial_gradient`・`conic_gradient` は、塗る図形に合わせて伸縮します。`CanvasGradient::{linear, radial, conic}` はキャンバス上の点に固定されます。どちらもペイントを受け取る場所（線を含む）で使え、座標で配置するほうは `.extend(GradientExtend::Repeat)` で両端の先まで繰り返せます。

## テキストを描き、測る

`draw.text(text, position, style)` は 1 行を描きます。`TextFormat` の `align` と `baseline` で基準点を変えない限り、位置はテキストの左上です。ラベルの周りの大きさを決めるには、描画と同じ組版で先に測ります。

```rust,no_run
use kivact::draw::{Draw, TextBaseline, TextFormat};
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::style::TextAlign;

/// ラベルの幅に左右 12 px の余白を足した幅のピル。
fn pill(draw: &mut Draw, label: &str, at: Point) {
    let style = TextFormat {
        size: 12.0,
        weight: 600,
        color: rgb(0xffffff),
        ..TextFormat::default()
    };
    let metrics = draw.measure_text(label, &style);
    let rect = Rect::new(at.x, at.y, metrics.size.width + 12.0, metrics.size.height + 8.0);
    draw.rounded_rect(rect, rect.height / 2.0, ColorToken::Accent);
    draw.text(label, Point::new(at.x + 6.0, at.y + 4.0), style);
}

fn labels() -> impl IntoView {
    canvas(|draw, size| {
        pill(draw, "新着", Point::new(12.0, 12.0));
        pill(draw, "進行中", Point::new(12.0, 44.0));

        // 点を中心に置くときは、測らずに基準点を使います。
        let centered = TextFormat {
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            ..TextFormat::default()
        };
        draw.text("中央", Point::new(size.width / 2.0, 100.0), centered);
    })
    .style(style().size(240, 120))
    .name("ラベルの見本")
}
# fn main() -> kivact::Result<()> { App::new("Labels").run(labels) }
```

`TextMetrics` は `size`（`size.width`・`size.height`）と、`baseline` のオフセットを論理ピクセルで返します。折り返すテキストは `draw.layout_paragraph(text, &style, Some(max_width))` で組み、`draw.paragraph(&paragraph, origin, color)` で描きます。`TextFormat` の項目は要素のテキストと同じ語彙（`family`・`weight`・`italic`・`letter_spacing`・`decoration` など）です。

整形済みの文字を描く場合、`ShapedTextSnapshot` と `GlyphRunSnapshot` は読み取り用の描画データです。独自のテキスト編集部品で支援技術へ文字の run を渡す `SemanticsTextRuns` は、所有する `SemanticsTextRun` の `Vec` または iterator から作れます。内部の共有ポインターを組み立てる必要はありません。`iter()`、`get()`、添字は run の参照を返すため、読み取るだけなら run 全体の clone も不要です。

## グループを変形・クリップ・半透明にする

`draw.layer(Layer, |draw| { … })` は、クロージャの中で描いたもの全体に、変形・クリップ・不透明度・合成モード・フィルター・マスク・影を適用します。グループはクロージャが戻ると終わります。レイヤーの変形は、テキスト・画像・クリップにも効きます。

```rust,no_run
# use kivact::view::canvas;
# use kivact::draw::{Layer, Transform};
use kivact::geometry::{Point, Rect};
# use kivact::prelude::*;
fn spinner() -> impl IntoView {
    canvas(|draw, size| {
        let center = Transform::new()
            .translate(size.width / 2.0, size.height / 2.0)
            .rotate(30);
        draw.layer(Layer::new().transform(center).opacity(0.8), |draw| {
            draw.fill_rect(Rect::new(-20.0, -4.0, 40.0, 8.0), ColorToken::Accent);
        });
    })
    .style(style().size(80, 80))
    .name("スピナー")
}
# fn main() -> kivact::Result<()> { App::new("Layer").run(spinner) }
```

## 3D で描き、合成し、枠の画像を描く

レイヤーの transform には、perspective と x・y 軸まわりの回転を含められるので、描いた図形を空間の中で回せます。3D のグループの見え方は、さらに 3 つのレイヤー設定で決まります。

- `transform_style(TransformStyle::Preserve3d)` は、グループの中に描いたレイヤーを、グループ自身の 3D の空間に保ちます（CSS の `transform-style: preserve-3d` と同じ）。
- `backface_visibility(BackfaceVisibility::Hidden)` は、グループが 90 度を超えて回ったときに隠します。表だけが見えます。
- `composite(Composite::…)` は、レイヤーが既に描かれたものとどう合成されるかを選びます（HTML の canvas の `globalCompositeOperation` と同じ）。`SourceIn` は、既存の画素の上にあるレイヤーの部分だけを残し、`DestinationOver` はレイヤーを既にある画素の後ろに描き、`Xor` は重ならない部分を残します。

`draw.border_image(rect, &image)` は九分割の枠を描きます。絵の四隅は変わらず、辺と中央が伸びるか繰り返して `rect` を埋めます。`BorderImage::new(source)` には、画像のバイト列（SVG も可）、ファイル名、グラデーションのいずれかを渡します。`.slice(n)` は絵のどこで切るかを絵の画素で指定し、`.width(..)` は画面での枠の大きさ、`.repeat(BorderImageRepeat::Round)` は辺の部分を整数回に収まるよう繰り返します。

```rust,no_run
use kivact::draw::{BorderImage, Composite, Layer};
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::style::{BackfaceVisibility, TransformStyle};

fn tilted_panel() -> impl IntoView {
    canvas(|draw, size| {
        let centre = Point::new(size.width / 2.0, size.height / 2.0);
        // パネルを縦の軸まわりに回し、400 px 離れた所から見る。
        let tilt = Transform::new()
            .translate(centre.x, centre.y)
            .perspective(400.0)
            .rotate_y(25.0);
        draw.layer(
            Layer::new()
                .transform(tilt)
                .transform_style(TransformStyle::Preserve3d)
                .backface_visibility(BackfaceVisibility::Hidden),
            |draw| {
                draw.fill_rect(Rect::new(-60.0, -40.0, 120.0, 80.0), ColorToken::Accent);
            },
        );

        // パネルが既にある所だけに描く枠。
        let frame = BorderImage::new(br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12"><rect width="12" height="12" fill="#f59e0b"/></svg>"##)
            .slice(4)
            .width(4)
            .repeat(kivact::draw::BorderImageRepeat::Round);
        draw.layer(Layer::new().composite(Composite::SourceIn), |draw| {
            draw.border_image(Rect::new(0.0, 0.0, size.width, size.height), &frame);
        });
    })
    .style(style().size(240, 160))
    .name("傾いたパネル")
}
# fn main() -> kivact::Result<()> { App::new("Panel").run(tilted_panel) }
```

## 文字を塗り、線で縁取り、輪郭を描く

`layout_paragraph` で一度作った段落は 3 通りに描けます。その塗りには任意の paint を使えるので、グラデーションで文字を塗れます。`stroke_paragraph` は `Stroke` でグリフの縁を描き、`paragraph_path` はグリフの輪郭を `Path` として返します。その `Path` は塗っても線を引いてもかまいません。1 行のテキストなら、`draw.text_path(text, 位置, &format)` で輪郭を直接得られます。

```rust,no_run
use kivact::draw::{Stroke, TextFormat, linear_gradient};
use kivact::prelude::*;

fn headline() -> impl IntoView {
    canvas(|draw, _size| {
        let format = TextFormat {
            size: 40.0,
            weight: 800,
            ..TextFormat::default()
        };
        let paragraph = draw.layout_paragraph("SHIFT", &format, None);
        let origin = Point::new(12.0, 8.0);

        // 文字はグラデーションで塗り、その上から縁を描く。
        let fill = linear_gradient(90).colors([rgb(0xffb300), rgb(0xff5a6e)]);
        draw.paragraph(&paragraph, origin, fill);
        draw.stroke_paragraph(&paragraph, origin, &Stroke::new(2.0), rgb(0x1a1a1a));

        // 同じ輪郭を path として、もう一度細く明るく描く。
        let outline = draw.paragraph_path(&paragraph, origin);
        draw.stroke_path(&outline, &Stroke::new(0.5), rgb(0xffffff));
    })
    .style(style().size(240, 64))
    .name("見出し")
}
# fn main() -> kivact::Result<()> { App::new("Headline").run(headline) }
```

要素のテキストにも、スタイルで同じ効果があります。`style().text_stroke(width, color)` はグリフを縁取り、`paint_order` は塗りと線のどちらを先に描くかを決め、`text_shadow(..)` は影を加えます。縁取りをパスやスタイルで表せないグラデーションに沿わせたいときに、キャンバスを使ってください。

## キャンバスの中にドローアブルを表示する

ドローアブルは、キャンバスが画像として描く view の部分木です。そのため、ほかのキャンバスの内容と同じように、回転・クリップ・重ね合わせができます。セットアップで `drawable()` で作り、`node().drawable(d, content)` で内容をノードにマウントし、キャンバスの中で `draw.drawable(d, 位置)` を呼んで描きます。開いたレイヤーや transform の下で描かれます。ヒットテストとアクセシビリティは、描かれた絵に対して自動的に従います。全体の流れは[ドローアブルのガイド](drawables.md)で説明します。自前の GPU レンダラーの絵は、[GPU テクスチャをキャンバスと共有する](gpu-texture-sharing.md)を参照してください。

## 状態が変わったら再描画する

クロージャの中でシグナルから読んだものは、すべて依存関係になります。シグナルが変わると、このキャンバスだけが再描画されます。「汚れた」フラグを立てたり、再描画の関数を呼んだりする必要はありません。

```rust,no_run
use kivact::draw::{LineJoin, Path, Stroke};
use kivact::prelude::*;

fn sparkline(values: Signal<Vec<f64>>) -> impl IntoView {
    canvas(move |draw, size| {
        let values = values.get(); // 読むことでこのキャンバスが購読します
        let step = size.width / values.len().saturating_sub(1).max(1) as f64;
        let path = values.iter().enumerate().fold(Path::new(), |path, (i, v)| {
            let point = Point::new(i as f64 * step, size.height * (1.0 - v.clamp(0.0, 1.0)));
            if i == 0 { path.move_to(point) } else { path.line_to(point) }
        });
        draw.stroke_path(&path, &Stroke::new(2).join(LineJoin::Round), ColorToken::Accent);
    })
    .style(style().size(240, 60))
    .name("推移を示す折れ線")
}

fn app() -> impl IntoView {
    let values = signal(vec![0.5_f64]);
    column((
        sparkline(values),
        button("点を追加").on_press(move || {
            values.update(|v| v.push(((v.len() as f64) * 1.7).sin() * 0.5 + 0.5));
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Sparkline").run(app)
}
```

クロージャは、入力だけで結果が決まる純粋な関数に保ってください。シグナルを読み、`draw` のメソッドを呼ぶだけにします。クロージャの中でシグナルに書き込んではいけません。画像はクロージャの外で一度だけ読み込みます（`let logo = Asset::from(bytes)` のあとで `draw.image(&logo, rect)`）。画像は `Asset` の同一性でキャッシュされるため、描画のたびに新しい `Asset` を作ると、描画のたびにデコードし直すことになります。

キャンバスをアニメーションさせるには、`frame_loop` や `animated` でシグナルを動かし、クロージャでそのシグナルを読みます。[値と要素をアニメーションさせる](animation.md) を参照してください。

## 描いた部分をクリック可能にし、アクセシブルにする

キャンバスは 1 つのノードです。描いたピクセルには、*リージョン*として宣言するまで、ポインター・キーボード・スクリーンリーダーにとっての意味がありません。リージョンは、形（当たり判定）、アクセシブルなロールと名前、ハンドラを持ちます。手順は 2 つです。

1. セットアップで、キャンバスに `.region(id, part)` を付けて振る舞いを宣言します。part は `kivact::custom::Part::new(state, apply)` で作ります。通常の要素と同じように、変換のクロージャーで対象ノードのロール・名前・ハンドラを設定します。
2. 描画のクロージャで、同じ `RegionId` に `draw.region(id, rect)` で形を与えます。

```rust,test_harness
use kivact::draw::{RegionId};
use kivact::geometry::{Rect};
use kivact::prelude::*;
use kivact::custom::Part;

const SWATCHES: [(&str, u32); 3] = [("赤", 0xe5484d), ("緑", 0x30a46c), ("青", 0x3e63dd)];

fn swatches() -> impl IntoView {
    let chosen = signal(0usize);

    let mut board = canvas(move |draw, size| {
        for (i, (_, color)) in SWATCHES.iter().enumerate() {
            let rect = Rect::new(i as f64 * 56.0, 0.0, 48.0, size.height);
            draw.fill_rect(rect, rgb(*color));
            if chosen.get() == i {
                draw.stroke_rect(rect, 3, rgb(0xffffff));
            }
            draw.region(RegionId(i as u64), rect);
        }
    });
    for (i, (name, _)) in SWATCHES.iter().enumerate() {
        board = board.region(
            RegionId(i as u64),
            Part::new(().into_value(), move |host| host
                .role(Role::Button)
                .name(*name)
                .selected(move || chosen.get() == i)
                .on_press(move || chosen.set(i))),
        );
    }

    column((
        board.style(style().size(168, 48)).name("色の見本"),
        text(move || format!("選択中: {}", SWATCHES[chosen.get()].0)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Swatches").run(swatches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_region_is_a_button_for_tests_and_screen_readers() {
        let screen = kivact::test::render(swatches);
        screen.get_by_text("選択中: 赤");
        screen.get_by_role(Role::Button, "緑").click();
        screen.get_by_text("選択中: 緑");
        assert!(screen.get_by_role(Role::Button, "緑").is_selected());
    }
}
```

リージョンは、宣言した順にキーボードフォーカスを受け取り、`on_press` を通じて Enter と Space に反応し、アクセシビリティツリーではボタンとして現れます。そのため `kivact::test` からロールと名前で探してクリックできます。

知っておくこと:

- **宣言してから描きます。** 描いたのに宣言されていない `RegionId`、または同じレイヤーで 2 回描いた `RegionId` は、エラーとして報告され、無視されます。宣言したのに描かれないリージョンは、単に存在しないものとして扱われます。
- **当たり判定は描いたとおりになります。** リージョンは、描いたレイヤーの変形を保持します。回転や拡大縮小をした図形は、見えている位置で当たります。長方形以外の領域には `draw.region_shape(id, rect, &Shape::circle(r))` や `draw.region_path(id, rect, &path)` を使います。
- **重なりは描画順で決まり**、Tab の順は宣言順です。
- 項目数が変わる場合は、`canvas.regions(items, key, make)` でキー付きのコレクションを宣言できます。項目がリストに残っている間、その状態は保たれます。
- キャンバス全体でポインターに反応したい場合（ブラシ、ドラッグ）は `.on_pointer(|event| …)` を使います。`PointerEvent::pos` はキャンバスの座標で、`PointerEvent::region()` は当たったリージョン（あれば）を返します。
- 編集可能なテキストフィールドを、キャンバスのリージョンの中に置くことはできません。

## 自前のレンダラーのフレームを表示する

アプリが自分で絵を作る場合（エミュレーター、3D ビューポート、動画デコーダーなど）は、ワーカースレッドで描画し、各フレームを `Pixels`（RGBA8、ストレートアルファ）として送り、キャンバスで最新のものを描きます。フレームはチャンネルが UI スレッドへ運ぶので、ポーリングは要りません。`Pixels::new(width, height, rgba)` には所有する `Vec<u8>` をそのまま渡せます。バッファ長は `width × height × 4` バイトです。

```rust,no_run
use std::time::Duration;
use kivact::draw::{ImageOptions, Pixels};
use kivact::geometry::{Rect};
use kivact::prelude::*;

fn renderer_preview() -> impl IntoView {
    let image = signal(None::<Pixels>);
    let (tx, rx) = async_channel::bounded(2);

    // フレームの大きさは物理ピクセル: 論理サイズに表示倍率を掛けます。
    let scale = kivact::platform::scale_factor().get();
    let width = (320.0 * scale).ceil().max(1.0) as u32;
    let height = (180.0 * scale).ceil().max(1.0) as u32;

    std::thread::spawn(move || {
        for red in (0..=255u8).cycle() {
            // この単色を、自前のレンダラーの出力に置き換えます。
            let rgba = [red, 64, 128, 255]
                .repeat(width as usize * height as usize);
            if tx.send_blocking(Pixels::new(width, height, rgba)).is_err() {
                break; // ウィンドウが閉じられました
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    });

    spawn(async move {
        while let Ok(mut newest) = rx.recv().await {
            // UI が忙しい間にたまったフレームは読み飛ばします。
            for _ in 0..rx.len() {
                if let Ok(frame) = rx.try_recv() {
                    newest = frame;
                }
            }
            image.set(Some(newest));
        }
    });

    canvas(move |draw, size| {
        if let Some(frame) = image.get() {
            let target = Rect::new(0.0, 0.0, size.width, size.height);
            draw.pixels(&frame, target, ImageOptions::default());
        }
    })
    .style(style().size(320, 180))
    .name("レンダラーのプレビュー")
}

fn main() -> kivact::Result<()> {
    App::new("Renderer").run(renderer_preview)
}
```

- `async-channel` は、自分の `Cargo.toml` に追加する依存です。受信側のタスクを起こすチャンネルなら何でも使えます。
- ウィンドウやコンポーネントがなくなると、受信タスクが破棄されてチャンネルが閉じ、`send_blocking` が `Err` を返します。ワーカーのループはそこで終わらせてください。
- 遅い UI でフレームがたまらないよう、チャンネルは小さくし、上のように古いフレームを捨てます。
- ウィンドウの大きさや表示倍率が変わりうる場合は、新しい大きさをワーカーに伝えて描き直させます。`ImageOptions` では、切り抜き（`source`）、不透明度（`opacity`）、ドット絵向けの最近傍補間（`pixelated`）を指定できます。
- 内容が変わらないフレームは、バッファをコピーし直さず同じ `Pixels` を clone して使い回します。内部の `AssetBytes` は不変バッファの同一性を保ち、clone で画素全体を複製しません。
- GPU からの読み戻しが遅すぎる場合は、[キャンバスで GPU テクスチャを共有する](gpu-texture-sharing.md) を参照してください。

## ウィンドウなしで画像を書き出す

`Offscreen` は、同じ `Draw` の語彙で、指定した大きさの RGBA 画像に描画します。`App` もウィンドウも要らないので、作品の書き出し、サムネイル、連番フレーム、コマンドラインツールで使えます。絵を 1 つの関数として書けば、画面表示と書き出しの両方で使えます。

```rust
use kivact::draw::{Draw, Offscreen};
use kivact::geometry::Rect;
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    draw.circle(center, size.height / 3.0, rgb(0xf2aa4c));
}

fn main() -> kivact::Result<()> {
    let mut renderer = Offscreen::new();
    let image = renderer.render(640, 360, artwork)?;

    // 隙間なく詰めた RGBA で、上の行から順に、ストレート（乗算済みでない）アルファです。
    assert_eq!(image.rgba.len(), 640 * 360 * 4);
    assert_eq!(&image.rgba[..4], &[0x10, 0x18, 0x20, 0xff]);
    // `image.rgba` は、好みの PNG や image クレートで符号化してください。
    Ok(())
}
```

同じアプリで `canvas(artwork)` と書けば、書き出すのとまったく同じ関数を画面に表示できます。

- 画像は透明な状態から始まります。不透明にしたければ背景を塗ってください。
- 座標は、倍率 1 の出力ピクセルです。大きさが 0 の場合や、レンダラーが受け付ける範囲を超える場合はエラーになり、`paint` は呼ばれません。
- 描けないコマンド（有限でない座標、負の半径、バッファの長さが大きさと合わない `Pixels`）や、読み込みに失敗した画像・フォントは、キャンバスなら報告して残りを描きますが、`Offscreen` ではエラーとして返ります。保存する前に `Result` を確かめてください。
- テキストにはシステムのフォントが使われます。どのマシンでも同じ出力にするには、同梱するフォントのファイルを `renderer.font(include_bytes!("…"))` で登録し、`Offscreen::theme(theme)` でアプリのテーマを渡します。
- `Offscreen` は、作ったスレッドから出られません。バックグラウンドで書き出すには、`blocking` の中で作ります。

```rust,no_run
use kivact::draw::{Draw, Offscreen};
use kivact::geometry::Rect;
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
}

fn exporter() -> impl IntoView {
    let export = action(ActionMode::Latest, |(width, height): (u32, u32)| async move {
        let image = blocking(move || Offscreen::new().render(width, height, artwork)).await?;
        // ここで `image` をディスクに書き出します（UI スレッドの外のままです）。
        Ok(image.rgba.len())
    });
    column((
        button("1920 × 1080 で書き出す").on_press(move || export.run((1920, 1080))),
        dynamic(move || export.state(), |state| match state {
            AsyncState::Idle => text(""),
            AsyncState::Pending => text("書き出し中…"),
            AsyncState::Ready(bytes) => text(format!("{bytes} バイトを描画しました")),
            AsyncState::Failed(error) => text(format!("失敗: {error}")),
        }),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Export").run(exporter) }
```

## SVG に書き出す

`Svg` は `Offscreen` と同じ `Draw` の語彙を記録し、1 つの SVG 1.1 文書として
書き出します。`Offscreen` と同じく `App` を必要としないので、1 つの描画関数を
画面、PNG 書き出し、SVG 書き出しで共有できます。

```rust,no_run
use kivact::draw::{Draw, Svg};
use kivact::geometry::{Point, Rect, Size};
use kivact::prelude::*;

fn artwork(draw: &mut Draw, size: Size) {
    draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x101820));
    let center = Point::new(size.width / 2.0, size.height / 2.0);
    draw.circle(center, size.height / 3.0, rgb(0xf2aa4c));
}

fn main() -> kivact::Result<()> {
    let document = Svg::new().render(640.0, 360.0, artwork)?;
    assert!(document.starts_with("<?xml"));
    // `document` を `.svg` ファイルに書き出します。
    Ok(())
}
```

- 大きさは論理 px で、文書の `width`、`height`、`viewBox` はその大きさです。
  文書に背景はないので、不透明にしたい場合は背景を描いてください。
- 矩形、角丸の矩形、線、パス、図形、文字、クリップ、変換、不透明度つきの層、
  線形と放射状のグラデーション、画像は SVG の要素になります。画像は描いた大きさの
  PNG データ URI として埋め込まれます。
- SVG で表せない命令は `ErrorKind::Unsupported` のエラーになり、黙って欠落させません。
  層のブレンドモード、マスク、フィルタ、文字の装飾とフォント機能、整形済みの段落、
  円錐グラデーション、キーワードで大きさを決めた放射状グラデーションがこれに当たります。
  大きさが 0 のときや有限でない値があるときもエラーで、`paint` は呼ばれません。
- SVG はフォントを埋め込みません。文字の計測のために `svg.font(include_bytes!("…"))` で
  使うフォントを登録し、文書を見る環境にも同じフォントを同じファミリ名で入れてください。

## マウント済みの node を画像にする

`NodeRef` を `.node_ref(target)` で結び付け、非同期の action や task から
`target.snapshot()` を await します。背景と子孫を含む現在の scene を、node の
layout の大きさに切り取った、所有された straight-alpha RGBA8 の `Pixels` が返ります。
論理画素一つが出力画素一つになり、小数の大きさは切り上げます。ウィンドウが GPU を
使っていても CPU の rasterizer で描画します。通常の `Pixels` は含まれます。
GPU 共有の frame には CPU の画素の表現がないため、呼び出し位置を持つ
`ErrorKind::Unsupported` のエラーになります。未接続または取り外された target も
エラーです。future は poll された時点の接続先を使い、owner が破棄されていれば失敗します。

## 関連項目

- [値と要素をアニメーションさせる](animation.md): フレームループでキャンバスを動かします。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): チャンネル、`spawn`、`blocking`。
- [キャンバスで GPU テクスチャを共有する](gpu-texture-sharing.md): 読み戻しなしで GPU レンダラーのフレームを表示します。
- [アプリをテストする](testing.md): ロールでリージョンを探し、スクリーンショットを比べます。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::draw` と `kivact::view::canvas` を開いてください。

pointer の handler 内で `event.capture()` を呼ぶと、その pointer が node の外へ動いても後続のイベントを受け取れます。`event.release_capture()` は後続のイベントを通常の当たり判定に戻します。明示の capture はボタンを押さない hover 中にも開始でき、最後のボタンの解放、キャンセル、node の削除で終了します。押している間の暗黙の capture は既定のままです。
