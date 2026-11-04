# 自前の GPU レンダラーのテクスチャを表示する

[English](../../en/how-to-guide/gpu-texture-sharing.md)

> **実験的。** `gpu-interop` feature は Windows (Direct3D 12 と Vulkan) と Linux (Vulkan) で動作します。既定では無効で、API は予告なく変わることがあり、一部は `unsafe` です。グラフィックス API との契約をあなたが守る必要があります。

3D ビューポート、動画デコーダー、シミュレーションのように、アプリが自前のレンダーエンジンを持ち、それが独自の GPU デバイスで動いている場合に、そのフレームを **CPU へ読み戻さずに** Kivact のウィンドウに表示するためのガイドです。エンジンがテクスチャに描き、ウィンドウはその同じテクスチャを自分の GPU で描きます。両者は OS の共有ハンドルと 2 つのフェンスで結びつきます。

フレームが小さい場合や、エンジンが CPU で動く場合は、この機能は要りません。ピクセルを `Pixels` として公開し、`Draw::pixels` で描いてください ([キャンバスで描く](canvas.md#自前のレンダラーのフレームを表示する))。

## 仕組み

- エンジンは自前の GPU デバイスにテクスチャの**プール** (既定は 3 枚) を作り、各テクスチャと 2 つのタイムラインフェンスを、OS のハンドル (Windows は NT ハンドル、Linux はファイルディスクリプター) として一度だけエクスポートします。
- ウィンドウはそのプールを自分の GPU デバイスに**インポート**して、そこから描きます。
- **フレーム**は、テクスチャのスロットと番号の組です。エンジンはフレームを描き終えると、`ready` フェンスをその番号まで進めます。ウィンドウは読み終えると `release` フェンスを進めるので、エンジンはテクスチャをいつ再び書いてよいかが分かります。
- Kivact はエンジンのスレッドを動かしません。あなたのスレッドが `ShareLink` という小さなハンドルを呼び、呼び出しのたびに UI スレッドが起こされます。

テクスチャの形式は、プリマルチプライドアルファの RGBA8 で、値は `Pixels` や CSS の色と同じく `sRGB` エンコードです。リニアな光で計算するエンジンは、最後の工程で sRGB にエンコードしてください。

## 有効にする

アプリケーション側:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["gpu-interop"] }
```

`gpu-interop` は `gpu` を含みます。エンジン側は `kivact-gpu-share` に依存します。このクレートは UI を知らないので、エンジンのスレッドが `kivact` に依存する必要はありません。

```toml
[dependencies]
kivact-gpu-share = { git = "https://github.com/Kashiwade-music/kivact", features = ["wgpu"] }
```

| `kivact-gpu-share` の feature | 得られるもの |
|---|---|
| なし | 共有される型とエラー。 |
| `export` | 生の Vulkan (`vulkan`) と Direct3D 12 (`d3d12`) でのテクスチャのプールとフェンス。 |
| `wgpu` | `export` に加えて、wgpu で書かれたエンジン向けのヘルパー (`wgpu`)。wgpu のバージョンは Kivact が使う `=29.0.4` に固定されており、二者の型が一致します。 |

エンジンが別のバージョンの wgpu を使うなら、ネイティブのヘルパーを使うか、`kivact-gpu-share` の rustdoc にある「別バージョンの wgpu のエンジン」の節に従ってください。

共有が可能かどうかは、次の 2 つの規則で決まります。

- **同じアダプター。** エンジンのデバイスは、任意の GPU ではなく、ウィンドウが報告するアダプター (`gpu_interop::adapter()`) の上に作ります。
- **同じグラフィックス API。** Direct3D 12 のウィンドウは Direct3D 12 のエンジンと、Vulkan のウィンドウは Vulkan のエンジンと共有します。選ぶには `App::gpu_preference` を設定します ([アダプターと API を選ぶ](#アダプターと-api-を選ぶ))。

## ウィンドウ側

`shared_image(width, height)` が UI 側の作業を引き受けます。setup で作り、その `link()` をエンジンのスレッドに渡し、`frame()` を canvas で描きます。`pixels()` は、インポートできないウィンドウ向けに同じ画像が持つ代替です。

```rust,ignore
use kivact::view::canvas;
use kivact::draw::{ImageOptions};
use kivact::geometry::{Rect};
use kivact::gpu_interop;
use kivact::prelude::*;

fn viewport() -> impl IntoView {
    let image = gpu_interop::shared_image(1280, 720);

    // `ShareLink` は Send + Sync + Clone で、エンジンスレッドが持つハンドルです。
    let link = image.link();
    std::thread::spawn(move || engine_thread(link));

    let status = image.clone();
    let view = image;
    column((
        canvas(move |d, size| {
            let rect = Rect::new(0.0, 0.0, size.width, size.height);
            // 追跡されます。エンジンが新しいフレームを公開するたびに canvas が再描画されます。
            if let Some(frame) = view.frame() {
                d.gpu_frame(&frame, rect, ImageOptions::default());
            } else if let Some(pixels) = view.pixels() {
                d.pixels(&pixels, rect, ImageOptions::default());
            }
        })
        .style(style().grow(1)),
        text(move || format!("{:?}", status.status())),
    ))
}
```

`SharedImage` は UI スレッド用のハンドルです (`Clone` ですが `Send` ではありません)。作成した owner に属し、そのコンポーネントが消えるとリンクが閉じ、エンジンの待機は即座に戻り、ウィンドウは最後のフレームを解放します。

creative kit のビューポートコンポーネントも、同じ 2 つの供給元を受け取ります。

```rust,ignore
use kivact::gpu_interop::shared_image;
use kivact_ui_creative::viewport_surface;

let image = shared_image(1280, 720);
let (frame, pixels, sized) = (image.clone(), image.clone(), image);
viewport_surface(move || pixels.pixels())
    .gpu_frame(move || frame.frame())
    .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
```

kit 自身の `gpu-interop` feature が必要です。このサーフェスはフレームをズームとパンし、ポインターを画像のピクセル座標に写します。

### ビューのサイズに追従する

`image.set_size(width, height)` は、別のサイズの絵 (物理ピクセル) をエンジンに求めます。サイズが変わるたびに**新しいリクエスト**になります。古いインポートは破棄され、エンジンは新しいプールを作って渡します。ウィンドウをドラッグしている間、1 ピクセルごとに呼ばないでください。サイズが落ち着くのを待つか、段階的に丸めてください。

## エンジン側

エンジンのスレッドは、最新のリクエストを受け取り、それに応え、フレームを公開するループを持ちます。リクエストにはモード、サイズ、トークンが入っており、新しいリクエストは古いものを置き換えます。

```rust,ignore
use std::time::Duration;

use kivact_gpu_share::{FrameReady, PoolConfig, wgpu as share};
use kivact::gpu_interop::{ShareError, ShareLink, ShareMode, ShareRequest};

struct Engine {
    request: ShareRequest,
    // drop の順序が重要です。pool は最後のフレームを待つので、最初に drop されます。
    pool: share::Pool,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn engine_thread(link: ShareLink) {
    let mut engine: Option<Engine> = None;
    while !link.is_closed() {
        // 新しいリクエストは pool を置き換えます。エンジンがなければ、リクエストを待ちます。
        let request = match engine {
            Some(_) => link.poll_request(),
            None => link.wait_request(Duration::from_millis(100)),
        };
        if let Some(request) = request {
            drop(engine.take()); // 古い pool を先に解放します
            engine = match &request.mode {
                ShareMode::Gpu(_) => Engine::serve(&link, request).ok(),
                ShareMode::Pixels => {
                    serve_pixels(&link, &request); // CPU で描きます
                    None
                }
            };
        }
        if let Some(running) = &mut engine {
            if running.draw_and_publish(&link).is_err() {
                engine = None; // 次のリクエストを待ちます
            }
        }
    }
}
```

GPU のリクエストに応えるには、報告されたアダプターの上にデバイスを作り、リクエストのサイズのプールを作って、ウィンドウに渡します。

```rust,ignore
impl Engine {
    fn serve(link: &ShareLink, request: ShareRequest) -> Result<Engine, String> {
        let ShareMode::Gpu(adapter) = &request.mode else {
            return Err("a Pixels request".into());
        };

        // Direct3D 12 のウィンドウと併用するときは `InstanceFlags::DEBUG` と
        // `VALIDATION` を付けません。デバイスができたあとにデバッグレイヤーを
        // 有効にすると、そのデバイスが消えます。
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::empty().with_env(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let (_, device, queue) =
            share::request_device(&instance, adapter, &Default::default()).map_err(|e| e.to_string())?;
        let config = PoolConfig::new(request.width, request.height);
        let (pool, textures) = share::Pool::new(&device, adapter, config).map_err(|e| e.to_string())?;

        // SAFETY: pool は `kivact_gpu_share` が作ったもので、`SharedTextures` の契約を
        // 守ります。このエンジンは `wait_ready` のあと、`Target::submit` 経由でのみ
        // テクスチャに描きます。
        unsafe { link.provide(&request, textures) }.map_err(|e| e.to_string())?;
        link.wait_ready(&request, Duration::from_secs(5)).map_err(|e| e.to_string())?;
        Ok(Engine { request, pool, device, queue })
    }

    fn draw_and_publish(&mut self, link: &ShareLink) -> Result<(), ShareError> {
        let target = self.pool.acquire(Duration::from_secs(1)).map_err(|e| ShareError::Failed(e.to_string()))?;
        let view = target.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        draw_scene(&mut encoder, &view); // エンジン自前のレンダーパス
        // SAFETY: `queue` は pool のキューで、このスレッドだけが使います。
        let ready: FrameReady = unsafe { target.submit(&self.queue, [encoder.finish()]) }
            .map_err(|e| ShareError::Failed(e.to_string()))?;
        link.publish(&self.request, ready.slot, ready.value);
        Ok(())
    }
}
```

各ステップが保証することです。

1. **`wait_request` / `poll_request`** は、このハンドルが最後に返したものより新しいリクエストだけを返します。`Gpu` のリクエストはアダプターとグラフィックス API を指定します。`Pixels` のリクエストは、ウィンドウがいまインポートできないことを意味します (後述の状態を参照)。
2. **`request_device`** は、そのアダプター上に適切な拡張つきのデバイスを開きます。ここから得た Direct3D 12 デバイスは、ウィンドウも wgpu を使っている場合は `ID3D12Device` をウィンドウと共有しますが、問題ありません。
3. **`Pool::new`** はテクスチャとフェンスを作り、ハンドルを一度だけ返します。リクエストごとに新しいプールを作ってください。同じプールを 2 回渡してはいけません (`ShareError::Failed`)。
4. **`provide`** はハンドルを UI スレッドに渡し、UI スレッドがそれをインポートします。`unsafe` なのは、エンジンが `kivact::gpu_interop::SharedTextures` の rustdoc にある契約を守ると約束するからです。形式、受け渡しの状態、`wait_ready` のあとにだけ書くこと、解放フェンスがテクスチャの最後のフレームに達してからだけ書くこと、などです。`kivact_gpu_share` のプールが、そのほとんどを代わりに守ります。
5. **`wait_ready`** は、ウィンドウがインポートを終えると `Ok` を返します。それより前に描くと、テクスチャはまだあなたのものではありません。
6. **`acquire` / `submit`** はテクスチャを渡し (`acquire` が返す `Target` は `submit` までプールを借用します)、最後のフレームに対するウィンドウの解放を GPU 側で待ち、あなたのコマンドの前後に受け渡しのバリアを置きます。テクスチャには `slots` フレーム前の絵が残っているので、`LoadOp::Load` でそれを保てます。
7. **`publish(request, slot, value)`** はフレームの準備ができたことを UI に伝え、UI スレッドを起こします。最新のフレームが勝ちます。UI がエンジンより遅ければ古いフレームは飛ばされ、まとめて解放されます。古いリクエスト、準備ができていないリクエスト、閉じたリンクへの publish は無視されます。

生の Vulkan や Direct3D 12 でも同じ手順で、`kivact_gpu_share::vulkan::Pool` と `d3d12::Pool` (`acquire` のあと、その target の `record_acquire`、自前のコマンド、`record_release`、`submit`) を使います。クレートの rustdoc を見てください。

### CPU 上の絵

ウィンドウがインポートできないときも、エンジンは何かを見せなければなりません。`Pixels` のリクエストには、CPU で描くか、自前のテクスチャを読み戻して応え、結果を公開します。

```rust,ignore
use std::sync::Arc;

use kivact::draw::Pixels;

fn serve_pixels(link: &ShareLink, request: &ShareRequest) {
    let rgba: Vec<u8> = render_on_cpu(request.width, request.height); // ストレートアルファの RGBA8
    link.publish_pixels(Pixels::new(request.width, request.height, Arc::from(rgba)));
}
```

`publish_pixels` はどのリクエストでも受け付けられ、最新の絵が勝ちます。UI は `frame()` がないとき `pixels()` を描くので、エンジンが応えられなかった GPU のリクエスト (一致するアダプターがない、`serve` でのエラー) も同じ方法で代替できます。`share::read_back` は、エンジン自前のテクスチャをストレートアルファの RGBA8 として読み戻します。

## 状態と失敗

開発中は `image.status()` を表示してください。シグナルのように追跡されます。

| `ShareStatus` | 意味 | 対処 |
|---|---|---|
| `Waiting` | `Gpu` のリクエストを出したが、エンジンがまだプールを渡していない。 | エンジンのスレッドが動き、リクエストに応えているか確認します。 |
| `Importing` | プールが渡され、ウィンドウがインポート中。 | 待ちます。終わると `wait_ready` が戻ります。 |
| `Ready` | フレームが描かれる。 | フレームを公開します。 |
| `Pixels` | ウィンドウがインポートできない。CPU で描いている、GPU デバイスがまだ準備できていない、またはプラットフォームが Windows でも Linux でもない。 | `publish_pixels` で応えます。 |
| `Failed(error)` | ウィンドウがプールを拒否した。`error.kind()` は `Unsupported` (インポートできる GPU デバイスがない) か `Platform` (テクスチャが、アダプター、API、申告したサイズと合わない、またはデバイスの失敗)。メッセージは Arvane のものです。 | プールを直します。次のリクエストまでこの状態が続きます。 |
| `Closed` | 画像の owner がなくなった (`show` が false になった、行が消えた、App が閉じた)。何も描かれず、リンクは閉じている。 | `is_closed` を見てエンジンのスレッドを止めます。 |

**新しいリクエスト**は、3 つの場合に自動で届きます。ウィンドウのアダプターが変わったとき (デバイスの喪失後の新しいデバイス、または最初のフレームのあとに GPU が使えるようになったとき)、サイズが変わったとき、インポートが失われたときです。いずれもウィンドウ側のインポートを破棄し、エンジンのプールは古くなります。エンジンはそれに気づいて (フレームの合間に `poll_request`)、プールを破棄し、新しいリクエストに応える必要があります。新しいデバイスができると、以前のインポートはすべて失われます。

`ShareLink` の呼び出しは `ShareError` を返すので、扱ってください。

| `ShareError` | 意味 | 対処 |
|---|---|---|
| `Stale` | 新しいリクエストがこれを置き換えた。 | 作ったものを破棄して、新しいリクエストを受け取ります。 |
| `Closed` | 画像の owner がなくなった。 | スレッドを止めます。 |
| `Timeout` | 待機が時間切れになった。 | プールが失敗したものとして扱い、新しいリクエストを待ちます。 |
| `Failed(message)` | そのリクエストには応えられない。ウィンドウがプールを拒否した、リクエストが `Pixels` だった、テクスチャを 2 回渡した。 | ログに残し、プールを破棄します。 |

`kivact_gpu_share::Error` はエンジン自身の失敗を表します。GPU 待ちのタイムアウト (多くはウィンドウが表示を止めた場合)、`NoAdapter`、`Mismatch` (契約に反する引数)、グラフィックス API 呼び出しの失敗 (`Api`) です。待機は必ず上限つきなので、解放が来ない場合はハングではなく、プールが 1 周したあとのエラーになります。プールを drop すると最後のフレームを最大 5 秒待ち、それでも終わらなければ、GPU の下で解放することなく生のオブジェクトをリークさせます。

## アダプターと API を選ぶ

GPU デバイスを先に作るのはウィンドウで、どのアダプターの上にあるかを教えてくれます。`gpu_interop::adapter()` で読みます。デバイスの準備ができるまで、ウィンドウが CPU で描いている間 (`ARVANE_RENDERER=cpu` や GPU なし)、そして他のプラットフォームでは `None` です。追跡されるので、`memo` で追えます。`GpuAdapter` はグラフィックス API、アダプターの LUID または Vulkan の UUID、`generation` を持ち、新しい世代は新しいデバイスを意味します。

エンジンが特定のものを必要とする場合など、ウィンドウにグラフィックス API、アダプターの種類、特定のアダプターを選ばせたいときは、希望を設定します。

```rust,ignore
use kivact::gpu_interop::{GpuBackend, GpuPower, GpuPreference};
use kivact::prelude::*;

fn main() -> kivact::Result<()> {
    App::new("Viewport")
        .gpu_preference(GpuPreference {
            backend: Some(GpuBackend::Vulkan),
            power: GpuPower::HighPerformance,
            adapter: None,
        })
        .run(app)
}
```

これは要求ではなく希望です。応えられなければウィンドウは別のアダプターや API に切り替わり、環境変数 `WGPU_BACKEND` と `WGPU_POWER_PREF` が優先されます。希望がなければ、ウィンドウは省電力のアダプターを好み、Vulkan、次に (Windows では) Direct3D 12 の順で選びます。エンジンは自前の希望ではなく、常に `adapter()` に従ってください。

## 制限

- **ウィンドウは 1 つ。** 画像は、その App のウィンドウでのみ、その GPU で描かれます。CPU で描いているウィンドウは GPU のフレームを何も描きません。その場合は `Pixels` の経路を使います。
- **Windows と Linux のみ。** Windows では Direct3D 12 と Direct3D 12、Vulkan と Vulkan。Linux では Vulkan。ほかの組み合わせはなく、macOS も対象外です。
- **形式。** RGBA8、プリマルチプライドアルファ、sRGB エンコードの値の 1 種類だけです。
- **契約はあなたが守る。** `provide`、`submit`、低レベルのインポートが `unsafe` なのは、エンジンが書いてよい場所にだけ書いているかを Kivact が確認できないためです。契約を破ると、グラフィックス API の未定義動作になり、絵が壊れる、デバイスが失われる、プロセスがクラッシュするといったことが起こりえます。
- **順序どおり、1 枚ずつ。** プールはテクスチャをラウンドロビンで渡します。フレームの target は submit するまでプールを借用します。フレームをやめるときは target を破棄 (drop) します。
- **インポート 1 回につきエクスポート 1 回。** ドライバーによっては (Vulkan の NVIDIA)、以前エクスポートしたハンドルのメモリのインポートを拒否します。リクエストごとに新しいプールを作ってください。
- **プールはウィンドウがインポートしてから使う。** Vulkan では、テクスチャはウィンドウのインポートが置いたレイアウトで始まります。一度もインポートされていないプールは Vulkan の規則の外にあります。`Pixels` には、プールではなくエンジン自前のテクスチャを読み戻してください。
- **実験的な API。** 名前や細部が変わることがあります。

`shared_image` を使わず、アダプターと状態を自分で追うアプリのために、低レベルの `gpu_interop::import` が 1 つのプールをいまインポートし、`ExternalImage::publisher` がエンジンのスレッドから `ExternalImage::latest_frame` へフレームを運びます。`shared_image` はこれの上に作られています。

## 関連項目

- [キャンバスで描く](canvas.md): `Draw::pixels`、`Draw::gpu_frame`、フレーム上の region。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): スレッドと `blocking`。
- [エラーや更新の原因を調べる](diagnostics.md): 拒否されたインポートの `ErrorKind::Unsupported` と `ErrorKind::Platform`。
- `cargo doc -p kivact --features gpu-interop --open`: `kivact::gpu_interop`。`cargo doc -p kivact-gpu-share --features wgpu --open`: エンジン側。
