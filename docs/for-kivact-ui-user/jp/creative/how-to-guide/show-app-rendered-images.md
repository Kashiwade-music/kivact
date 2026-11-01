# アプリが描いたフレームを表示する

[English](../../../en/creative/how-to-guide/show-app-rendered-images.md)

目的: アプリが作る画像（3D ビューポート、合成結果、ヒストグラム、ノードのプレビューなど）を、パンとズームができる面に表示し、ポインターの位置を画像の座標で受け取り、重い描画を UI スレッドから切り離します。

キットが行うのは描画面の表示とナビゲーションで、シーンの描画ではありません。RGBA のピクセルを渡すと、フィット、ズーム、パン、オーバーレイ、入力の処理はキットが行います。このガイドは、[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。画像の作り方とスレッド間の受け渡しは、コアのガイド [キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md#自前のレンダラーのフレームを表示する)でより一般的に説明しています。

## ズームとパンができる画像を表示する

```rust,no_run
use kivact::draw::{Pixels, Point, Rect, Size};
use kivact::interaction::ViewTransform;
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact_ui_creative as ui;
use std::sync::Arc;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

/// あなたのレンダラー。ここでは、青のチャンネルが `tint` のグラデーションを描く。
fn render(tint: f64) -> Pixels {
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            rgba.push((x * 255 / WIDTH) as u8);
            rgba.push((y * 255 / HEIGHT) as u8);
            rgba.push((tint.clamp(0.0, 1.0) * 255.0) as u8);
            rgba.push(255);
        }
    }
    Pixels::new(WIDTH, HEIGHT, Arc::from(rgba))
}

fn viewer() -> impl IntoView {
    let tint = signal(0.5);
    let transform = signal(ViewTransform::default());
    let surface = signal(Size::new(0.0, 0.0));
    let pointer = signal(Point::new(0.0, 0.0));

    // `tint` が変わるたびに描き直す。`None` なら背景だけを表示する。
    let frame = memo(move || Some(render(tint.get())));

    column((
        row((
            ui::value_slider("色合い", tint).range(0.0, 1.0).step(0.01).precision(2),
            button("全体表示").on_press(move || {
                transform.set(ViewTransform::fit(
                    Size::new(WIDTH as f32, HEIGHT as f32),
                    surface.get_untracked(),
                ));
            }),
            text(move || {
                let p = pointer.get();
                format!("x {:.0}, y {:.0}", p.x, p.y)
            }),
        ))
        .style(style().gap(8)),
        ui::viewport_surface(frame)
            .transform(transform)
            .on_resize(move |size: ui::SurfaceSize| surface.set(size.logical))
            .on_pointer(move |event: ui::SurfacePointer| pointer.set(event.position))
            .overlay(|draw, _transform| {
                draw.stroke_rect(Rect::new(200.0, 100.0, 240.0, 160.0), 1.0, rgb(0xffd25a));
            })
            .name("ビューアー")
            .style(style().height(400)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("ビューアー")
        .theme(ui::theme(ThemeMode::Dark))
        .run(viewer)
}
```

実行してください。最初は、画像が面いっぱいに収まって表示されます。ホイールでポインターを中心にズームし、中ボタン（または `Space` を押しながら主ボタンでドラッグ）でパンして、**全体表示** で元に戻します。黄色の矩形は、オーバーレイが画像の座標で描かれているので、ズームしても絵に張り付いたままです。

### 描画面がしてくれること

- **`.transform(..)` を付けないと、画像は常に面に収まります。** `Binding<ViewTransform>` を渡すと、ズームとパンはあなたのものになります。保存したり、**全体表示** のようにリセットしたり、2 つのビューアーで共有したりできます。
- **座標は画像の座標です。** `on_pointer` は `SurfacePointer`（位置・段階・ボタン・修飾キー）を報告し、`position` はズームに関係なく*画像の*ピクセルで表されます。`overlay(|draw, transform| ..)` も同じ座標で描きます。現在の倍率が必要なとき（たとえばハンドルの画面上の大きさを一定に保つとき）は `transform` を使います。
- **`regions(..)`** は、画像の座標で指定する、アクセシビリティに対応したクリックできる矩形を追加します（`SurfaceRegion::new(id, name, role, bounds)`）。ポインターのイベントの `region` が、ポインターの下にある領域を示します。
- **`pixelated(true)`** は、ネイティブサイズより大きく拡大したときに最近傍のピクセルを使います。ピクセルを確認する用途に向いています。
- **`on_resize`** は `SurfaceSize` を報告します。論理サイズ、物理ピクセルのサイズ、スケール係数です。高密度ディスプレイで鮮明にするには `physical_width` × `physical_height` で描き、変わったら描き直します。
- **再描画は等価性で決まります。** `Pixels` は同一性で比較されるので、同じ `Pixels` をもう一度渡しても、複製しても、コストはかかりません。新しい `Pixels` を渡すと再描画されます。
- **3D・合成・スコープはあなたの仕事です。** 描画面はピクセルの意味を知りません。ヒストグラムや波形モニターは、データから描くもう 1 つの画像で、別の `viewport_surface` に表示します。

タッチスクリーンでは 2 本の指でズームとパンをします。トラックパッド専用のピンチイベントはまだありません。ホイールのイベントとして届くピンチは、ズームになります。

## 別のスレッドで描画する

1 フレームより長くかかる描画で、UI を止めてはいけません。ワーカースレッドで描画し、完成した `Pixels` をチャンネルで送ります。UI のタスクが最新のものをシグナルへ書き込みます。シグナルは UI スレッドのものなので、`Signal` をワーカーへ move してはいけません。送るのは普通のデータだけです。

```rust,no_run
use kivact::draw::Pixels;
use kivact::prelude::*;
use kivact_ui_creative as ui;
use std::sync::Arc;
use std::time::Duration;

fn render(frame: u32) -> Pixels {
    let shade = (frame % 256) as u8;
    Pixels::new(2, 2, Arc::from([shade, 80, 160, 255].repeat(4)))
}

/// レンダラーを起動し、最新のフレームを常に保持するシグナルを返す。
fn start_renderer() -> Signal<Option<Pixels>> {
    let image = signal(None);
    let (sender, receiver) = async_channel::bounded::<Pixels>(2);

    std::thread::spawn(move || {
        for frame in 0.. {
            // ビューがなくなり受信側が破棄されると失敗する: 終了する。
            if sender.send_blocking(render(frame)).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(33));
        }
    });

    spawn(async move {
        while let Ok(mut newest) = receiver.recv().await {
            // UI が間に合わなかったフレームは読み飛ばす。
            while let Ok(later) = receiver.try_recv() {
                newest = later;
            }
            image.set(Some(newest));
        }
    });
    image
}

fn live_view() -> impl IntoView {
    let image = start_renderer();
    ui::viewport_surface(image)
        .pixelated(true)
        .name("ライブビュー")
        .style(style().height(300))
}
```

この例では、`Cargo.toml` に `async-channel` を追加してください。`spawn` で起動したタスクは、`start_renderer` を呼んだコンポーネントに属し、コンポーネントがなくなると止まります。ワーカーに終了を知らせるのは、チャンネルが閉じることです。容量 2 の有限チャンネルにしておくと、UI が遅れても古いフレームが溜まりません。

## GPU のフレームを読み戻さずに表示する（実験的）

レンダラーがすでに GPU で描いている場合、各フレームを `Pixels` として CPU に読み戻すと、フレームの予算を超えることがあります。実験的な方法として、GPU の画像をウィンドウと共有できます。Windows と Linux に限られ、`kivact` と `kivact-ui-creative` の両方の `gpu-interop` feature が必要です。詳しくは、コアのガイド [キャンバスで GPU テクスチャを共有する](../../../../for-kivact-user/jp/how-to-guide/gpu-texture-sharing.md)を参照してください。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["gpu-interop"] }
kivact-ui-creative = { git = "https://github.com/Kashiwade-music/kivact", features = ["gpu-interop"] }
```

有効にすると、描画面に、フレームを受け取る `gpu_frame(..)` が加わります。

```text
let image = kivact::gpu_interop::shared_image(1280, 720);
let (frame, pixels, sized) = (image.clone(), image.clone(), image);
ui::viewport_surface(move || pixels.pixels())
    .gpu_frame(move || frame.frame())
    .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
```

`gpu_frame` がフレームを供給していて、ウィンドウに GPU のデバイスがある間は、描画面はそのフレームを表示します。そうでない間（デバイスの準備がまだ、ウィンドウが CPU で描いている、フレームが失われた）は、`viewport_surface` に渡した `Pixels` を表示するので、両方を用意しておいてください。フィット、ズーム、ポインターの座標、オーバーレイ、領域は、表示されているほうの画像のサイズに従います。この API は予告なく変わることがあります。

## 関連項目

- [キャンバスで描く](../../../../for-kivact-user/jp/how-to-guide/canvas.md#自前のレンダラーのフレームを表示する): `Pixels`、キャンバス、スレッド間のフレームの受け渡し。
- [キャンバスで GPU テクスチャを共有する](../../../../for-kivact-user/jp/how-to-guide/gpu-texture-sharing.md)
- [ノードグラフでノードをつなぐ](node-graph.md): ノードのプレビューも `Pixels` を受け取ります。
- [色・イージング曲線・グラデーションを編集する](color-curves-and-gradients.md)
- API の全体: `cargo doc -p kivact-ui-creative --open`
- カタログのビューポートとグレーディングの章: `cargo run -p kivact-ui-creative-catalog`
