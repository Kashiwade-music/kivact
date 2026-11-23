# エフェクトを使う

[English](../../en/how-to-guide/effects.md)

このガイドでは、要素と周りのものとの合成の仕方を変える方法を説明します。フィルターで要素をぼかしたり色を変えたりし、マスクで端をフェードさせ、背後のピクセルと混ぜ、画像を九分割して枠にし、テキストの文字をグラデーションで塗ります。これらは CSS のプロパティと同じ名前、同じ結果になります。スタイルに設定し、キャンバスでは `Layer` の引数や、描画器の `border_image` として使います。

## 要素にフィルターを掛ける

`filter` は、要素とその子が描いたものすべてに、描いた後で変更を加えます。関数は CSS と同じです: `brightness`、`contrast`、`saturate`、`grayscale`、`sepia`、`hue_rotate`、`invert`、`opacity`、`blur`、`drop_shadow`。量も CSS と同じで、brightness、contrast、saturate、opacity は `1` で元の絵のまま、grayscale、sepia、invert は `0` で元の絵のままです。角度は度、長さは論理ピクセルです。

```rust,no_run
use kivact::style::Filter;
use kivact::prelude::*;
use kivact::style::rgba;

#[component]
fn lit_badge() -> impl Element {
    column(text("Online"))
        .style(
            style()
                .padding(12)
                .border_radius(8)
                .background(color::ACCENT)
                .filter(
                    Filter::new()
                        .brightness(1.12)
                        .drop_shadow(0.0, 0.0, 4.0, rgba(0x5aa9ff, 0.5)),
                ),
        )
}
# fn main() -> kivact::Result<()> { App::new("Badge").run(lit_badge) }
```

- フィルターの `drop_shadow` は、描かれた形に沿って影を付けます。文字や画像の透明な部分にも従います。箱の `shadow` は箱に沿います。
- `backdrop_filter` は同じ `Filter` を受け付け、要素の背後のものに掛けます。[背後のものを読む](#背後のものを読む)を参照してください。

### フィルターグラフで掛ける

SVG のフィルタープリミティブを使うには `FilterGraph` を組み立てます。メソッドを一つ呼ぶとプリミティブが一つ追加され、その結果を指す `FilterInput`（ハンドル）が返ります。前のプリミティブのハンドルを、後のプリミティブの入力として渡します。`FilterInput::SOURCE` は要素自身のピクセルです。`Filter::graph` はグラフを消費します。

```rust,no_run
use kivact::style::filter::Channel;
use kivact::style::{Filter, FilterGraph, FilterInput};
use kivact::prelude::*;

#[component]
fn glitch_title() -> impl Element {
    // Displace the element's pixels by noise: a glitch.
    let mut graph = FilterGraph::new();
    let noise = graph.turbulence(0.02, 2, 3.0);
    graph.displacement_map(FilterInput::SOURCE, noise, 12.0, Channel::R, Channel::G);

    text("GLITCH")
        .style(style().font_size(32).font_weight(700).filter(Filter::new().graph(graph)))
}
# fn main() -> kivact::Result<()> { App::new("Glitch").run(glitch_title) }
```

- グラフのプリミティブには、`blur`、`offset`、`blend`、`composite`、`color_matrix`、`saturate`、`hue_rotate`、`component_transfer`、`merge`、`erode`、`dilate`、`convolve`、`drop_shadow`、`flood`、`image`、`tile`、`turbulence`、`fractal_noise`、`displacement_map`、そして照明の `diffuse_lighting` と `specular_lighting` があります。
- 色はテーマ変数にできます。`flood` と `drop_shadow` は `Color` を受け取るので、`color::ACCENT` を渡すとグラフは有効なテーマに従います。
- フィルターの領域は、既定では要素の大きさの 10% だけ箱の外へ広がります。`graph.margin(..)` で別の広がりを指定できます。グローを作るときに必要です。
- グラフは SVG と同じく、既定では線形 RGB で計算します。sRGB で計算するには `graph.srgb()` を呼びます。
- 別のグラフのハンドルは誤りです。スタイルを組み立てるときに報告され、その入力は要素自身に置き換えられます。

## マスクでフェードさせる

`mask` は、その層のアルファを保ちます。マスクが透明な部分では、要素とその子は見えなくなります。グラデーションは、よく使われるマスクです。

```rust,no_run
use kivact::style::{MaskComposite, MaskLayer};
use kivact::prelude::*;
use kivact::style::linear_gradient;

#[component]
fn fading_list() -> impl Element {
    let fade = linear_gradient(180).colors([Color::BLACK, Color::TRANSPARENT]);
    column((text("First"), text("Second"), text("Third")))
        .style(style().gap(8).height(160).mask(fade))
}
# fn main() -> kivact::Result<()> { App::new("Fade").run(fading_list) }
```

- より細かく制御するには `kivact::style::MaskLayer` を使います。`MaskLayer::new(paint)` で作り、`.mode(MaskMode::Luminance)` でアルファではなく明るさで掛け、`.composite(MaskComposite::Intersect)` で下の層と組み合わせます。複数の層はタプルで渡します: `style().mask((first, second))`。先頭の層が一番上になります。
- マスクの paint は、既定では要素の border box を基準に配置されます。背景と同じく、`Paint::origin(..)` と `Paint::clip(..)` で変えられます。
- ソースがシェーダーのマスクは描かれず、サポートされていないものとして報告されます。

## 背後のものと混ぜる

`mix_blend_mode` は、要素を背後のピクセルと混ぜます。CSS の `mix-blend-mode` と同じです。`Multiply` は暗くし、`Screen` は明るくします。`Isolation` は、要素がどのピクセルと混ざるかを決めます。

```rust,no_run
use kivact::style::BlendMode;
use kivact::prelude::*;
use kivact::style::Isolation;

#[component]
fn duotone_card() -> impl Element {
    stack((
        column(text("Image"))
            .style(style().size(200, 120).padding(12).background(color::ACCENT)),
        column(text("Tint"))
            .style(
                style()
                    .size(200, 120)
                    .padding(12)
                    .background(rgb(0xffd166))
                    .mix_blend_mode(BlendMode::Multiply),
            ),
    ))
    // Blend only inside this card, not with the rest of the window.
    .style(style().isolation(Isolation::Isolate))
}
# fn main() -> kivact::Result<()> { App::new("Duotone").run(duotone_card) }
```

- 要素は、自分の分離されたグループの中のピクセルと混ざります。`isolation(Isolation::Isolate)` を付けた要素はその一つです。1 未満の `opacity`、`filter`、`mask`、`clip_path` を持つ要素も、CSS と同じく独自のグループになります。
- 背景の層に `Paint::blend(..)` を設定すると、その層を下の背景の色と混ぜられます（CSS の `background-blend-mode`）。

## 背後のものを読む

`backdrop_filter` は、要素の背後にあるピクセルをぼかしたり色を変えたりします。そのため、半透明のパネルがガラスのように見えます。`backdrop_blur(px)` は `backdrop_filter(Filter::new().blur(px))` の短い書き方です。

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgba;

#[component]
fn glass_panel() -> impl Element {
    stack((
        column(text("Behind the glass"))
            .style(style().size(260, 160).padding(16).background(rgb(0x3e63dd))),
        column(text("Glass"))
            .style(
                style()
                    .size(180, 80)
                    .padding(12)
                    .border_radius(12)
                    .background(rgba(0xffffff, 0.2))
                    .backdrop_blur(12.0),
            ),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Glass").run(glass_panel) }
```

- 背景として読まれるのは、要素の背後に描かれたものです。範囲は、グループ（1 未満の `opacity`、filter、mask、backdrop filter のいずれかを持つ最も近い祖先）までです。そのような祖先がなければ、ウィンドウ全体になります。
- 上の例のように、要素自身の背景は半透明にしてください。不透明な背景は、ぼかされた背後を隠してしまいます。

## 画像で箱を縁取る

`border_image` は、画像（またはグラデーション）を九つの部分に切り分けます。四隅、四辺、中央です。隅は大きさを保ち、辺は枠に沿って伸びるか繰り返されます。歪ませずに拡大縮小したい枠や装飾に使います。

```rust,no_run
use kivact::style::{BorderImage, BorderImageRepeat};
use kivact::prelude::*;

#[component]
fn framed_panel() -> impl Element {
    // A 36 px picture with a 12 px frame. The border is 12 px wide.
    let frame = BorderImage::new(br##"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 36 36"><rect x="2" y="2" width="32" height="32" rx="6" fill="none" stroke="#3e63dd" stroke-width="4"/></svg>"##)
        .slice(12)
        .repeat(BorderImageRepeat::Round);

    column(text("Framed"))
        .style(style().padding(16).border(12, color::BORDER).border_image(frame))
}
# fn main() -> kivact::Result<()> { App::new("Frame").run(framed_panel) }
```

- `slice` は、画像の中で切る四つの線の位置を画素で指定します（`slice_pct` は割合で指定します）。隅は、箱の中でもその大きさになります。
- `width(..)` で枠の幅を指定します。既定は境界線の幅です。`outset(..)` は枠を border box の外へ広げます。
- `repeat` には `Stretch`（既定）、`Repeat`、`Round`、`Space` を指定します。辺の部分が、一枚では足りないほど長い辺をどう埋めるかを決めます。
- `fill()` を付けると、内容の背後で中央の部分も描きます。
- 画像の代わりにグラデーションも使えます。例: `BorderImage::new(linear_gradient(90).colors([...]))`。各部分は、画像の部分と同じく箱の大きさで描かれます。

キャンバスでは、`Draw::border_image(rect, &frame)` が同じ九つの部分を長方形の中に描きます。キャンバスには境界線がないので、枠の `width` は自分で指定してください。

```rust,no_run
use kivact::style::{BorderImage, BorderImageRepeat};
use kivact::geometry::Rect;
use kivact::prelude::*;

#[component]
fn framed_canvas() -> impl Element {
    let frame = BorderImage::new(br##"<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 36 36"><rect x="2" y="2" width="32" height="32" rx="6" fill="none" stroke="#3e63dd" stroke-width="4"/></svg>"##)
        .slice(12)
        .width(12)
        .repeat(BorderImageRepeat::Stretch);

    canvas(move |draw, size| {
        draw.border_image(Rect::new(0.0, 0.0, size.width, size.height), &frame);
    })
    .style(style().size(240, 120))
    .name("Framed picture")
}
# fn main() -> kivact::Result<()> { App::new("Framed canvas").run(framed_canvas) }
```

## 背景のクリップと基準の箱を選ぶ

背景には二つの設定があります。`Paint::origin` は、絵やグラデーションを位置と大きさを決める基準の箱を選びます。`Paint::clip` は、それを描く箱を選びます。既定の基準は padding box、既定のクリップは border box で、CSS と同じです。

```rust,no_run
use kivact::style::{BackgroundClip, BackgroundOrigin, Paint};
use kivact::prelude::*;
use kivact::style::rgba;
use kivact::style::linear_gradient;

#[component]
fn padded_tint() -> impl Element {
    let tint = linear_gradient(90).colors([rgb(0x3e63dd), rgb(0x30a46c)]);

    column(text("Tinted content box"))
        .style(
            style()
                .padding(24)
                .border(8, rgba(0xffffff, 0.5))
                .background(
                    Paint::from(tint)
                        .origin(BackgroundOrigin::ContentBox)
                        .clip(BackgroundClip::ContentBox),
                ),
        )
}
# fn main() -> kivact::Result<()> { App::new("Tint").run(padded_tint) }
```

- `BackgroundClip::BorderBox`、`PaddingBox`、`ContentBox` は、それぞれの箱にクリップします。角は境界線の半径に沿って丸められます。
- `BackgroundClip::Text` は、要素自身のテキストのグリフの中だけに描きます。次の節を参照してください。

## テキストをグラデーションで塗る

背景を `BackgroundClip::Text` でクリップし、テキスト自身の色を透明にすると、グラデーションが文字の中に見えます。テキストを持つ要素に付けてください。`BackgroundClip::Text` のコンテナは、自身のテキストを持たないので、何も描かれません。

```rust,no_run
use kivact::style::{BackgroundClip, Paint};
use kivact::prelude::*;
use kivact::style::linear_gradient;

#[component]
fn gradient_title() -> impl Element {
    let fill = linear_gradient(90).colors([rgb(0xe5484d), rgb(0xffb020), rgb(0x30a46c)]);

    text("Gradient text").style(
        style()
            .font_size(40)
            .font_weight(800)
            .color(Color::TRANSPARENT)
            .background(Paint::from(fill).clip(BackgroundClip::Text)),
    )
}
# fn main() -> kivact::Result<()> { App::new("Gradient text").run(gradient_title) }
```

- グラデーションの `hint(..)` は、二つの色の間で、色がちょうど中間になる位置を指定します（CSS と同じ）。例: `linear_gradient(90).colors([...]).hint(30)`。
- グラデーションの `interpolate(..)` は、色の補間を別の色空間で行います。結果は sRGB で表示されます。

## 注意点

- **範囲。** フィルター、マスク、ブレンドは、要素とその中にあるものすべてを一つの絵として扱います。
- **費用。** フィルター、マスク、背景の読み取り、ブレンドは、いずれもグループのピクセルを必要とするので、層として描かれます。多くの要素に同じエフェクトを付けると、費用が積み重なります。子ごとに付けるより、コンテナに一つ付けるほうが安くなります。
- **シェーダーの場合。** シェーダーのフィルターとシェーダーのマスクには、それぞれ独自の規則があります。[シェーダーを書く](shaders.md)を参照してください。
- **アニメーション。** フィルター、不透明度、そして単色の背景は、スタイルのトランジションでアニメーションできます。[値と要素をアニメーションさせる](animation.md)を参照してください。

## 関連項目

- [3D の変換と奥行きを使う](3d-and-depth.md): これらのエフェクトが掛かる変換。
- [シェーダーを書く](shaders.md): WGSL によるフィルターと paint。
- [キャンバスで描く](canvas.md): `Layer` での同じエフェクト。
- [テーマとコンポーネントの見た目を変える](theming.md): テーマ変数からの色とグラデーション。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::style::Filter`、`FilterGraph`、`Mask`、`BorderImage`、`BackgroundClip` を見てください。
