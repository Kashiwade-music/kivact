# シェーダーを書く

[English](../../en/how-to-guide/shaders.md)

シェーダーを使うのは、絵を形の組み合わせより、ピクセルごとの式として書いたほうが簡単なときです。走査線の模様、ノイズの質感、パラメーターに応じて変わるグロー、背後のものをゆがませるフィルターなどが例です。ピクセルごとの処理を WebGPU のシェーダー言語である WGSL で書き、Kivact が GPU で実行します。このガイドでは、二種類のシェーダー（領域を塗る paint と、要素のピクセルを変えるフィルター）、WGSL が守るべき規則、値がシェーダーに届く仕組み、CPU 描画が代わりに何を見せるかを説明します。

## シェーダーで塗る

paint シェーダーは、ピクセルごとに色を返す WGSL の関数 `paint` です。Kivact はシェーダーを作るときにソースを検査するので、誤りは最初のフレームではなく、セットアップの時点で行と列付きで報告されます。ソースは文字列として `PaintShader::wgsl` に渡します。

```rust,no_run
use kivact::draw::{PaintShader};
use kivact::geometry::Rect;
use kivact::motion::FrameRate;
use kivact::prelude::*;

const SCANLINES: &str = r#"
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
"#;

fn scanline_background() -> impl IntoView {
    // セットアップで一度だけ作ります。検査に通らないシェーダーはここでエラーになります。
    let scanlines = PaintShader::wgsl(SCANLINES).expect("the scanline shader is valid");

    canvas(move |draw, size| {
        let paint = scanlines
            .paint()
            .param("speed", 2.0_f64)
            .param("tint", rgb(0xffb020))
            .animate(FrameRate::Display)
            .fallback(rgb(0x101820));
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
    })
    .style(style().size(240, 96))
    .name("走査線の背景")
}
# fn main() -> kivact::Result<()> { App::new("走査線").run(scanline_background) }
```

- `ShaderPaint` は paint です。paint を受け付けるところならどこでも使えます。キャンバスの塗りや線、または `style().background(Paint::from(..))` による要素の背景です。
- `position` は、塗られる形の左上を原点とする論理ピクセルの位置です。`context.size` は、その形の大きさです。両者を割ると、0 から 1 の座標が得られます。
- シェーダーは、ストレートアルファの sRGB の色を返します。合成は Kivact が行います。
- `fallback` は、CPU で描くときと、GPU がパイプラインを作れないときに代わりに描かれます。指定しなければ、その場合は何も描かれません。

## シェーダーで要素にフィルターを掛ける

フィルターのシェーダーは、要素自身のピクセルを受け取り、新しいピクセルを返します。`effect` という関数で書き、`content(position)` で要素のピクセルを読みます。

```rust,no_run
use kivact::draw::{Filter, FilterShader};
use kivact::prelude::*;

const SCAN_FILTER: &str = r#"
fn effect(position: vec2<f32>, context: Context) -> vec4<f32> {
    let c = content(position);
    let scan = 0.85 + 0.15 * step(0.5, fract(position.y * 0.5));
    return vec4<f32>(c.rgb * scan, c.a);
}
"#;

fn status_panel() -> impl IntoView {
    let scan = FilterShader::wgsl(SCAN_FILTER).expect("the scan filter is valid");

    column(text("状態: オンライン"))
        .style(
            style()
                .padding(16)
                .radius(8)
                .background(ColorToken::Surface)
                .filter(Filter::new().shader(scan.filter())),
        )
}
# fn main() -> kivact::Result<()> { App::new("状態").run(status_panel) }
```

- `content(position)` は要素のピクセルで、ストレートアルファです。要素の外は透明です。シェーダーは要素の箱の外にピクセルを返すこともできます（グローなど）。その場合は `margin(px)` を指定して、余白を確保してください。
- フィルターは、同じリストの他のフィルター関数と順番に組み合わせられます。その出力が、要素のピクセルを置き換えます。

## シェーダーに値を渡す

値は、`Params` という名前の WGSL の構造体のフィールドとして宣言します。Rust からは `param(名前, 値)` で、フィールド名を指定して設定します。設定しなかったフィールドは 0 になります。

| WGSL のフィールドの型 | Rust の値 |
|---|---|
| `f32` | `f64`（例: `2.0_f64`） |
| `vec2<f32>` | `Point` または `(f64, f64)` |
| `vec3<f32>` | `(f64, f64, f64)` |
| `vec4<f32>` | `Color`（例: `rgb(0xffb020)`）または `(f64, f64, f64, f64)` |
| `i32` | `i32` |
| `u32` | `u32` |

- 構造体にない名前や、型の合わない値は捨てられ、一度だけ報告されます。
- `Color` は `vec4` のフィールドに合います。ほかの型には合いません。
- フィールドは入れ子にできず、配列にもできません。

画像はスロット 0 から 3 を通して渡します。`image(スロット, ソース)` でスロットに結び付け、WGSL では `image(スロット, uv)` で読みます。読み出される値はストレートアルファで、双線形補間され、端では値が止まります。

```rust,no_run
use kivact::draw::{PaintShader};
use kivact::geometry::Rect;
use kivact::prelude::*;

static PHOTO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#888"/></svg>"##;

const TINT_PHOTO: &str = r#"
fn paint(position: vec2<f32>, context: Context) -> vec4<f32> {
    let uv = position / context.size;
    let photo = image(0u, uv);
    return vec4<f32>(photo.rgb * vec3<f32>(1.0, 0.8, 0.6), photo.a);
}
"#;

fn tinted_photo() -> impl IntoView {
    let shader = PaintShader::wgsl(TINT_PHOTO).expect("the photo shader is valid");

    canvas(move |draw, size| {
        let paint = shader.paint().image(0, PHOTO);
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
    })
    .style(style().size(160, 80))
    .name("色味を変えた写真")
}
# fn main() -> kivact::Result<()> { App::new("写真").run(tinted_photo) }
```

- 画像は、ほかの `Asset` と同じく一度だけ読み込まれます。`Asset` は paint のクロージャの中で作らず、セットアップで保持してください。
- 何も結び付いていないスロットは、透明な画素を読みます。

## クロックでアニメーションさせる

シェーダーの `context.time` は秒単位です。`animate` と `FrameRate` で動かします。`FrameRate::Display` はディスプレイの更新ごとに、`FrameRate::Hz(30.0)` は一定の頻度で動かします。`animate` を付けないと `time` は 0 のままなので、クロックのないシェーダーは静止画になります。

- 動きを減らす設定の間は、`time` は 0 のままで、絵は動きません。動きに意味がある場合（進行を示すインジケーターなど）は、そのシェーダーに `essential()` を付けると、クロックが動き続けます。
- ウィンドウが表示されていない間、クロックは止まります。
- 一つのノードが複数のシェーダーを持つときは、その中で最も速い頻度で動きます。

## CPU 描画

CPU の描画は WGSL を実行しません。代わりに、シェーダーが指定した絵を描きます。

- paint のシェーダーは、`fallback` を描きます。指定がなければ何も描きません。どの paint シェーダーにも、無理のない `fallback` を付けてください。GPU のないマシンでも、妥当な絵が見えるようになります。
- フィルターのシェーダーは CPU では実行されないため、要素はフィルターなしで描かれます。ユーザーが必ず見る必要のあるものを、フィルターのシェーダーに頼らないでください。
- GPU がシェーダーのパイプラインを作れなかった場合は、CPU と同じように扱われ、その失敗は一度だけ報告されます。

既定では、ウィンドウは初めて GPU を必要とするまで CPU で描きます（`GpuStart::OnDemand`）。シェーダーの最初の数 frame は fallback を表示し、device の準備ができるとシェーダーの絵に替わります。最初の frame からシェーダーを描く必要があるときは、`App::gpu_preference`（`kivact::launch`）で `GpuStart::AtLaunch` を選びます。

シェーダーが影響するのは描画だけです。ヒットテスト、フォーカス、スクリーンリーダーは、シェーダーがなかった場合と同じ要素を見ます。

## WGSL の規則

Kivact は、あなたのソースの前後に独自の定義を足します。そのため、ソースで追加される名前を定義したり、予約された形を使ったりしてはいけません。`wgsl(..)` はソースを検査し、問題の行と列を付けたエラーを返します。

ソースは paint または effect の部分だけです。`Context`、バインディング、ヘルパーは Kivact が用意します。そのため、次は完全な paint シェーダーです。

```wgsl
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
```

- **エントリポイント。** paint シェーダーは `fn paint(position: vec2<f32>, context: Context) -> vec4<f32>` を定義するか、第 3 引数 `params: Params` を加えた形を定義します。フィルターのシェーダーは、同じ形の `fn effect(...)` を定義します。`filter` は WGSL の予約語なので、フィルターの入口は `effect` です。
- **`Params`。** 構造体の名前は `Params` にしてください。フィールドの型は上の表のとおりです。
- **使えるヘルパー。** `image(slot: u32, uv: vec2<f32>)`、`content(position: vec2<f32>)`（フィルターのシェーダーだけ）、`srgb_to_linear(c: vec3<f32>)`、`linear_to_srgb(c: vec3<f32>)`。
- **使えないもの。** `@group` と `@binding` の global、エントリポイントの属性 `@vertex`、`@fragment`、`@compute`、`override`、`enable`、`kivact_` で始まる名前、そして `Context`、`image`、`content`、`srgb_to_linear`、`linear_to_srgb` の再定義。
- **それ以外の WGSL は使えます。** 関数、ループ、関数の中の配列、`var<private>` などです。

`Context` は四つのフィールドを持ちます。`size`（形の大きさ、論理ピクセル）、`scale`（論理ピクセルあたりのデバイスピクセル数）、`time`（秒、`animate` から）、そして `position` で、各ピクセルの位置です。

## GPU で試験する

シェーダーの絵は GPU に依存するので、その試験は GPU のレーンで行います。試験は `RenderOptions::from_env().gpu()` で GPU レーンに参加します。`KIVACT_TEST_GPU=1` を設定すると、環境変数を読むすべての試験でレーンが有効になります。GPU を得られない試験は、理由を伝えて自分をスキップします。ウィンドウの試験をディスプレイなしで動かすには `KIVACT_TEST_MIRROR=off` を設定します。どちらも[テストガイド](testing.md)で説明しています。

## 関連項目

- [キャンバスで描く](canvas.md): 塗りつぶし、線、ほかの paint。
- [エフェクトを使う](effects.md): 組み込みのフィルター、マスク、グラデーション。
- [値と要素をアニメーションさせる](animation.md): `Params` を動かす値。
- [画像を表示し、カスタムフォントを使う](images-and-fonts.md): スロットに結び付ける画像の読み込み。
- [アプリをテストする](testing.md): テストでの GPU レーンと CPU のフォールバック。
