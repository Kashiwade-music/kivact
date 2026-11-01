# Canvas に図形を描く

`canvas` は `Node` を返します。描画 closure は実際の大きさと、closure が読む reactive な値の変更を追います。各 frame に signal へ同じ値を書き直す必要はありません。

```rust,no_run
use kivact::prelude::*;
use kivact::control::{canvas};
use kivact::draw::{Point, Rect};

fn diagram() -> impl IntoView {
    canvas(|draw, size| {
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), rgb(0x20242c));
        draw.circle(Point::new(48.0, 48.0), 20.0, rgb(0x6699ff));
    }).style(style().size(240, 120)).name("青い円の図")
}
```

図を操作可能にするには、`Part` を `canvas.region(id, part)` で宣言し、描画中に同じ `RegionId` の `Draw::region` で幾何を与えます。描画だけでは focus や role は生まれません。未宣言の ID と同一層の重複は診断になります。詳細は [部品作者の primitive ガイド](../../for-component-author/primitive.md)を参照してください。

操作領域には、表示上の座標と同じ幾何を使います。region の Tab 順序は宣言順です。scroll、transform、clip を独自に二重適用せず、公開の pointer/event と node/region の参照を使います。編集の native leaf を canvas region に付けることはできません。

作品を window なしで画像へ書き出す用途には `draw::Offscreen` があります。通常の canvas と同じ描画語彙を使い、返す RGBA は straight alpha です。使えない資源は error になるので、書出し結果を確認してから保存します。

`draw::Path::flattened(tolerance)` は輪郭ごとの `PathContour` を返します。`points` は直線で近似した点列、`closed` は終点から始点への接続を表し、閉じるための始点は点列の末尾に重複しません。許容誤差は logical pixel で指定し、0 以下や非有限値なら 0.25 を使います。空または不正な path は空の輪郭列です。塗り規則は `path.fill_rule()` で読み、`path.with_fill_rule(FillRule::EvenOdd)` で指定します。

## アプリの renderer の絵を表示する

アプリの renderer は自分の worker スレッドで RGBA8 の `Pixels` を作り、実行時に依存しない async channel で UI の task へ送れます。例では `async-channel = "2.5"` をアプリの依存に追加します。

```rust,no_run
use std::sync::Arc;
use std::time::Duration;
use kivact::prelude::*;
use kivact::control::canvas;
use kivact::draw::{ImageOptions, Pixels, Rect};

fn renderer_preview() -> impl IntoView {
    let image = signal(None::<Pixels>);
    let (tx, rx) = async_channel::bounded(2);
    let scale = kivact::platform::scale_factor();
    let width = (320.0 * scale).ceil().max(1.0) as u32;
    let height = (180.0 * scale).ceil().max(1.0) as u32;
    std::thread::spawn(move || {
        for red in (0..=255).cycle() {
            // Replace this solid frame with the application's renderer.
            let rgba: Arc<[u8]> = [red, 64, 128, 255]
                .repeat(width as usize * height as usize).into();
            if tx.send_blocking(Pixels::new(width, height, rgba)).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(16));
        }
    });
    spawn(async move {
        while let Ok(mut newest) = rx.recv().await {
            // Skip only the frames already queued at this point.
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
            draw.pixels(&frame, Rect::new(0., 0., size.width, size.height),
                ImageOptions::default());
        }
    }).style(style().size(320, 180)).name("Renderer preview")
}
```

描画先の論理サイズに `scale_factor()` を掛けて物理画素数を求めます。上の例は初期サイズを使います。サイズや DPI が変わる画面では、変更した大きさを worker に渡して次の frame を描き直します。各 frame に新しい `Arc` を使い、変わらない frame は同じ `Arc` を再利用します。

`spawn` の task は表示の owner に属します。表示や window を破棄すると受信 task が取り消され、worker の送信は `Err` になるためループを終了できます。停止可能な renderer は同じ終了条件に従います。channel の上限を小さくし、`try_recv` で古い frame を読み飛ばすと、転送待ちによる遅れを抑えられます。

## GPU で描いた画像を読み戻さずに表示する（実験）

**実験 feature `gpu-interop`（既定で off）。** API と挙動は変わることがあります。

アプリの renderer が GPU で絵を描いていて、`Pixels` への読み戻しと再 upload が frame の予算を超えるときに使います。RTX 3060 Ti で 4K の frame を表示する費用は、`Pixels` を経由する経路が約 50〜66 ms、この経路が約 2 ms でした。読み戻しが予算に収まるなら、上の `Pixels` の経路を使ってください。

### 有効にする

`kivact` の feature `gpu-interop` を有効にします（`viewport_surface` の `gpu_frame` を使うなら、`kivact-ui-creative` の同名の feature も）。engine 側は crate `gpu-share` の feature `export` を有効にして、Vulkan・Direct3D 12 の pool を使います。Kivact と同じ版の wgpu で書くなら、`export` を含む feature `wgpu` を有効にします。対象は Windows（Direct3D 12 と Vulkan）と Linux（Vulkan）です。それ以外の OS では build は通りますが、`kivact::gpu_interop::adapter()` は常に `None` で、`shared_image` は `Pixels` の経路になり、`import` は失敗します。

`App::gpu_preference(GpuPreference { .. })` で、使いたい GPU の種類（`power`）、graphics API（`backend`）、adapter（`adapter`）を希望します。希望であり、保証ではありません。希望の adapter や API がなければ、別のもので device が作られます。環境変数 `WGPU_BACKEND` と `WGPU_POWER_PREF` が優先されます。

window の GPU device が用意できるまで、`gpu_interop::adapter()` は `None` です。CPU で描く window では、ずっと `None` です。engine の device は、その adapter の LUID または UUID と backend（`GpuAdapter::backend`）に合わせて作ります（Direct3D 12 の window には Direct3D 12 の device、Vulkan の window には Vulkan の device）。`shared_image` を使えば、この待機と adapter の engine への受け渡しはライブラリが行います。

### 推奨の組み方

UI 側は `shared_image(width, height)` を setup で作ります。`SharedImage` は、adapter・大きさ・取り込みの変化のたびに、engine への要求（`ShareRequest`）を出します。engine が渡した texture の pool を取り込み、engine の frame と、CPU で描いた絵（`Pixels`）を受け取ります。UI の handle は `SharedImage`（`!Send`）、engine の thread へは `link()` の `ShareLink`（`Send + Sync + Clone`）を渡します。

```rust,ignore
// `kivact` と `kivact-ui-creative` の feature `gpu-interop` が要る。
use kivact::gpu_interop;
use kivact::prelude::*;
use kivact_ui_creative::viewport_surface;

fn viewport() -> impl IntoView {
    let image = gpu_interop::shared_image(1280, 720);
    let link = image.link();
    std::thread::spawn(move || engine_thread(link)); // 下の engine
    let (frame, pixels, sized) = (image.clone(), image.clone(), image);
    viewport_surface(move || pixels.pixels())
        .gpu_frame(move || frame.frame())
        .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
}
```

- `frame()` は、取り込みが `Ready` の間の最新の frame です（それ以外は `None`）。`pixels()` は、engine が CPU で描いて `publish_pixels` した最新の絵です。どちらも追跡され、新しい値で surface が描き直されます。
- surface は、`gpu_frame` が `Some` で `adapter()` も `Some` のときだけ GPU の frame を表示し、それ以外は `pixels` を表示します（自動の退避）。fit・zoom・pan・pointer の座標・overlay・region は、表示している方の画像の寸法を基準にします。`viewport_surface` の代わりに自分で `canvas` を使うなら、同じ値を `draw.gpu_frame(&frame, rect, ImageOptions::default())` と `draw.pixels(..)` で描きます。
- 大きさは `set_size` で要求します。surface の `on_resize` が渡す物理画素数（`physical_width`、`physical_height`）を使います。`SharedImage` は新しい要求を出し、engine は新しい大きさの pool を作り直します。同じ大きさの `set_size` は要求を増やしません。
- 要求は、作った時点、adapter の変化（device の作り直し、CPU への退避）、`set_size`、取り込みの `Lost` のたびに出ます。新しい要求を出した時点で、UI は古い取り込みを外します。engine は古い pool を捨てて、新しい要求に答えます。
- `SharedImage` を作った owner を破棄すると link が閉じ、取り込みが外れて、window は最後の frame まで release を signal します。

#### engine の thread

engine の thread は、`ShareLink` に対して次の順に動きます。

1. `wait_request(timeout)` で最新の要求（`ShareRequest`）を受けます。要求の `width` と `height` が pool の大きさ、`mode` が答え方です。
2. `ShareMode::Gpu(adapter)` なら、その adapter とその graphics API で自分の device を作り、要求の大きさの pool を作ります。
   - 例の wgpu 版では、`gpu_share::wgpu::request_device` と `Pool::new` です。
   - `unsafe { link.provide(&request, textures) }` で pool を渡し、`wait_ready` が `Ok` を返すまで texture へ書きません。
   - frame ごとに `acquire` → 描く → `unsafe { pool.submit(..) }` → `link.publish(&request, slot, value)` と進めます。
3. `ShareMode::Pixels` なら、window が取り込めません（CPU で描く window、GPU の起動前）。自分の texture に描いて `read_back` し、straight alpha の RGBA8 を `link.publish_pixels(Pixels::new(..))` で渡します。共有の pool は要りません。
4. frame の合間に `poll_request` を呼びます。新しい要求が来たら、古い pool を捨てて（古い要求の呼出しは `ShareError::Stale` になります）、新しい要求に答え直します。`publish_pixels` は要求の種類によらず受け付けられます。GPU の経路で描くあいだも CPU で描いた絵を渡しておくと、GPU の起動前や取り込みの失敗の間も surface は絵を表示できます。
5. `is_closed` が真になったら終わります。`wait_request` と `wait_ready` は、閉じると待たずにすぐ返ります。

次は、Kivact と同じ版の wgpu で書いた engine の例です。frame の絵は、`clear` が色を塗るだけです。ここへ engine 自身の描画を置きます。

```rust,ignore
// `kivact`、`wgpu`（Kivact と同じ版）、`pollster`、`gpu-share`（feature `wgpu`）が要る。
use std::sync::Arc;
use std::time::Duration;

use kivact::draw::Pixels;
use kivact::gpu_interop::{ShareLink, ShareMode, ShareRequest};
use gpu_share::{PoolConfig, wgpu as share};

const WAIT: Duration = Duration::from_secs(5);

/// What the engine serves one request with.
enum Served {
    /// Frames drawn into the shared pool.
    Gpu { device: wgpu::Device, queue: wgpu::Queue, pool: share::Pool },
    /// Frames drawn into a texture of the engine's own and read back.
    Pixels { device: wgpu::Device, queue: wgpu::Queue, texture: wgpu::Texture },
}

fn engine_thread(link: ShareLink) {
    // No `InstanceFlags::DEBUG` or `VALIDATION`: see "制限".
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        flags: wgpu::InstanceFlags::empty().with_env(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let mut served: Option<(ShareRequest, Served)> = None;
    let mut number = 0u32;
    while !link.is_closed() {
        // 1. The newest request; wait for one only while nothing is served.
        let request = match served {
            Some(_) => link.poll_request(),
            None => link.wait_request(Duration::from_millis(100)),
        };
        if let Some(request) = request {
            // 4. A newer request replaces the pool: drop the old one first.
            drop(served.take());
            served = serve(&link, &instance, request);
        }
        let Some((request, engine)) = &mut served else { continue };
        number = number.wrapping_add(1);
        let done = match engine {
            Served::Gpu { device, queue, pool } => {
                frame_on_gpu(&link, request, device, queue, pool, number)
            }
            Served::Pixels { device, queue, texture } => {
                frame_on_cpu(&link, request, device, queue, texture, number)
            }
        };
        if done.is_none() {
            // The pool is unusable (the window's device is lost, say):
            // wait for the request that replaces it.
            served = None;
        }
    }
}

fn serve(
    link: &ShareLink,
    instance: &wgpu::Instance,
    request: ShareRequest,
) -> Option<(ShareRequest, Served)> {
    let served = match &request.mode {
        // 2. The device on the window's adapter, and a pool of the asked size.
        ShareMode::Gpu(adapter) => {
            let (_, device, queue) =
                share::request_device(instance, adapter, &Default::default()).ok()?;
            let config = PoolConfig::new(request.width, request.height);
            let (pool, textures) = share::Pool::new(&device, adapter, config).ok()?;
            // SAFETY: the pool is `gpu-share`'s: it makes
            // the textures and fences as `SharedTextures` says, and this
            // engine draws only between `acquire` and `submit`.
            unsafe { link.provide(&request, textures) }.ok()?;
            // Draw only after the window imported the pool.
            link.wait_ready(&request, WAIT).ok()?;
            Served::Gpu { device, queue, pool }
        }
        // 3. The window cannot import: draw on a texture of the engine's own.
        ShareMode::Pixels => {
            let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
            let (device, queue) =
                pollster::block_on(adapter.request_device(&Default::default())).ok()?;
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: request.width,
                    height: request.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: share::FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            Served::Pixels { device, queue, texture }
        }
    };
    Some((request, served))
}

fn frame_on_gpu(
    link: &ShareLink,
    request: &ShareRequest,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pool: &mut share::Pool,
    number: u32,
) -> Option<()> {
    let target = pool.acquire(WAIT).ok()?;
    let mut encoder = device.create_command_encoder(&Default::default());
    clear(&mut encoder, &target.create_view(&Default::default()), number);
    // SAFETY: this thread alone uses `queue`.
    let ready = unsafe { pool.submit(queue, target, [encoder.finish()]) }.ok()?;
    link.publish(request, ready.slot, ready.value);
    Some(())
}

fn frame_on_cpu(
    link: &ShareLink,
    request: &ShareRequest,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    number: u32,
) -> Option<()> {
    let mut encoder = device.create_command_encoder(&Default::default());
    clear(&mut encoder, &texture.create_view(&Default::default()), number);
    queue.submit([encoder.finish()]);
    let rgba = share::read_back(device, queue, texture, WAIT).ok()?;
    link.publish_pixels(Pixels::new(request.width, request.height, Arc::from(rgba)));
    std::thread::sleep(Duration::from_millis(16));
    Some(())
}

/// A render pass that fills the view; the engine's own drawing goes here.
/// The values are written as they are: they are sRGB-encoded.
fn clear(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, number: u32) {
    let red = f64::from(number % 256) / 255.0;
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color { r: red, g: 0.25, b: 0.5, a: 1.0 }),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
}
```

この例は、frame を描けるだけ描きます。実際の engine は、場面が変わったときに描きます。`acquire` が時間切れになったとき（window が release を返さない）は、pool を捨てて次の要求を待ちます。

`wgpu::Pool::submit` は、wgpu の queue に生の submit を出すので `unsafe fn` です。その間、同じ queue を別の thread から使いません。`acquire` が返す texture には、その slot の前の frame が残っているので、`LoadOp::Load` で読めます（各 slot の最初の frame の中身は未定義です）。

#### 生の Vulkan（ash）と Direct3D 12 の engine

`gpu-share` の feature `export` で使える `gpu_share::vulkan::Pool` と `gpu_share::d3d12::Pool` は、pool と 2 本の timeline fence を作って export します。`Pool::new` が `SharedTextures` を返し、それを `link.provide` に渡します。Vulkan では `find_physical_device` で window の adapter の physical device を見つけ、`vulkan::DEVICE_EXTENSIONS` を有効にして device を作ります。Direct3D 12 では `find_adapter` で見つけた adapter の device を使います。frame ごとに、次の順に呼びます。

1. `acquire(timeout)`：次の texture を `Target` として得ます。その texture の前の frame が `ready` になるまで、CPU で時間切れ付きで待ちます。
2. `record_acquire`：window から texture を受け取る barrier を、engine の command の最初に記録します（Vulkan は `QUEUE_FAMILY_EXTERNAL` から、Direct3D 12 は `COMMON` から、engine が使う layout と状態へ）。
3. engine 自身の描画を記録します。
4. `record_release`：texture を window へ返す barrier を、command の最後に記録します。
5. `submit`：command を queue に出します。window の `release` の GPU 側の待ち、command、`ready` の signal を、この順に行います。返る `FrameReady { slot, value }` を `link.publish` に渡します。
6. engine が自分で submit するなら、`submit` の代わりに `finish` を呼びます。返る `Submission` の「待つ fence と値」を submission の先頭で待ち、「signal する fence と値」を最後に signal します。

`record_acquire`、`record_release`、`submit` は `unsafe fn` です。全体の流れは [crate doc](../../../crates/gpu-share/src/lib.rs) にあります。

#### 違う版の wgpu の engine

Kivact と違う版の wgpu で書いた engine は、`as_hal` で生の handle（Vulkan は ash の instance・physical device・device・queue family、Direct3D 12 は `ID3D12Device` と queue）を取り出し、上の `vulkan::Pool` か `d3d12::Pool` を使います。ash と `windows` の版は、`gpu-share` と同じ（0.38 と 0.62）にします。wgpu の device は `vulkan::DEVICE_EXTENSIONS` を有効にしないので、engine は device を作るときにそれを足します（wgpu の `open_with_callback`、または ash で作った device を `create_device_from_hal` で渡します）。pool の生の submit は wgpu の queue に出るので、wgpu の `submit` と同じ thread で、その間に呼びます。

### engine の約束

engine は texture の pool と 2 本の timeline fence を `gpu_interop::SharedTextures` として渡します。作り方と順序の正確な規則は [`SharedTextures` の rustdoc](../../../crates/ui-types/src/gpu_interop.rs) にあります。`ShareLink::provide` と `import` は `unsafe fn` で、その約束を守ることを呼出元に求めます。`gpu-share` は、pool と fence の作成と export、値の計数、受け渡しの barrier、時間切れ付きの待機を肩代わりします。engine が自分で守るのは、描画を `acquire` から `submit` の間に置くことと、色の符号化、`Ready` の前に書かないことです。要約は次のとおりです。

- texture の作成条件：2D、RGBA8 unorm（premultiplied）、`width` × `height`、mip・layer・sample は 1。graphics API ごとに、heap、usage、共有の flag、export の型が決まっています。
- usage は render target を常に含みます（Direct3D 12 は `ALLOW_RENDER_TARGET`、Vulkan は `TRANSFER_SRC | TRANSFER_DST | SAMPLED | COLOR_ATTACHMENT`）。storage への書込みは選択式で、使うなら `SharedTextures::storage` を真にして、Direct3D 12 は `ALLOW_UNORDERED_ACCESS`、Vulkan は `STORAGE` を足します（`PoolConfig::storage`）。Vulkan の adapter は storage の image を export できないことがあり、そのとき `Pool::new` は `Err` を返します。
- 色は sRGB で符号化された値です（premultiplied alpha）。`Pixels` や CSS の色と同じ符号化として合成されます。線形の値で描く engine は、最後に sRGB へ符号化してから渡します。sRGB の view で標本化しないでください。
- export は、texture と fence のそれぞれを、取り込みごとに 1 回だけ行います（NVIDIA の Vulkan driver は、一度 export して閉じた memory の再 export を断ります）。
- 受け渡しの状態：engine は、まだ書いていない texture を渡します。受け渡しの間、texture は Direct3D 12 では `COMMON` の状態、Vulkan では `SHADER_READ_ONLY_OPTIMAL` の layout で `QUEUE_FAMILY_EXTERNAL` が持ちます。engine は、最初の書込みの前に取得し、最後に同じ layout のまま返します。
- ready と release の値：engine は frame を描いたら `ready` を、frame ごとに増える値 `v` に signal し、`(slot, v)` を frame として渡します。`release` が `v` 以上なら、window は `v` 以下の frame を二度と読みません。engine は、texture へ書き直す前に、そこへ最後に描いた frame の値が `release` 以下であることを確かめます。
- Ready の前に書きません。`ShareLink::wait_ready` が `Ok` になる（低水準では `import` が `Ready` になる）まで、texture へ書きません。
- release を待つときは時間切れを付けます。device が失われると、`release` が来ないことがあります。
- 各 slot の最初の frame の中身は未定義です。前の frame を読む（`LoadOp::Load` など）のは、その slot に一度書いた後にします。

約束を破ると、graphics API（Vulkan と Direct3D 12）の未定義動作になります。壊れた画、device の喪失、process の異常終了、resource の範囲外の GPU memory への access がありえます。Kivact と Arvane は、その大部分を確かめられません。だから `ShareLink::provide` と `import` は `unsafe fn` です。

### 状態と失敗

`SharedImage::status()`（追跡されます）が、`ShareStatus` で今の段階を返します。

- `Waiting`：`Gpu` の要求を出して、engine の pool を待っています。
- `Importing`：engine が pool を渡し、window が取り込んでいます。
- `Ready`：取り込めました。engine は frame を描き始められます（`wait_ready` が `Ok`）。
- `Pixels`：window が取り込めません（CPU で描く window、GPU の起動前、対象外の OS）。engine は CPU で描いた絵を `publish_pixels` で渡します。
- `Failed(UiError)`：window が pool を取り込めませんでした。次の要求（`set_size`、adapter の変化）まで、この状態です。`error.kind()` で分けます。
  - `Unsupported`：その window に取り込める GPU device がありません。
  - `Platform`：engine の約束との食い違いです。adapter や graphics API が違う、texture が宣言と合わない、などで、message に原因が出ます。

device が作り直されて（`adapter()` の `generation` が変わったか、`None` になった）取り込みが `Lost` になると、`SharedImage` は新しい要求を出します。engine は古い pool を捨て、新しい adapter の device で pool を作り直します。handle は一度しか渡せないので、同じ pool は取り込み直せません。

engine の `ShareLink` の呼出しは、失敗を `ShareError` で返します。

- `Stale`：要求が新しい要求に置き換わりました。作ったものを捨て、`poll_request` で新しい要求を受けます。
- `Closed`：`SharedImage` の owner が破棄されました。engine の thread を終えます。
- `Timeout`：`wait_ready` が時間内に終わりませんでした。
- `Failed(message)`：window が pool を断りました（message に原因）。`Pixels` の要求へ pool を渡した場合と、同じ要求へ pool を二度渡した場合もこれです。次の要求を待ちます。

### 低水準 API

adapter と取り込みの状態を自分で追いたいときは、`shared_image` を使わずに、次を直接使えます。

- `unsafe { gpu_interop::import(textures) }`：pool を取り込み、`ExternalImage` を返します。window が答えるのは turn の後で、`ExternalImage::status()` が `ImportStatus` を追跡して報せます。取り込みは、`import` を呼んだ owner（`show` の中など）に属し、owner を破棄すると window が取り込みを外します。
  - `Pending`：答えを待っています。
  - `Ready`：engine は frame を描き始められます。
  - `Failed`：取り込めませんでした。error は `import` を呼んだ場所に紐づきます。種類は `ShareStatus::Failed` と同じ `Unsupported`（`adapter()` が `Some` になってから取り込みます）と `Platform` です。
  - `Lost`：`Ready` だった取り込みの device が作り直されました。新しい adapter で texture を作り、取り込み直します。自動では取り込み直しません。
- `ExternalImage::publisher()`：engine の thread から frame を出す `FramePublisher`（`Send + Sync`）です。`publish(slot, value)` した frame が `latest_frame()`（追跡されます）に届きます。
- `ExternalImage::latest_frame()` と `ExternalImage::frame(slot, value)`：canvas に渡す `GpuFrame` です。`GpuFrame::width()` と `height()` は取り込んだ texture の画素数で、`ImageOptions::source` はこの画素座標です。
- `Draw::gpu_frame(&frame, rect, ImageOptions::default())`：`Draw::pixels` と同じ rect・source・opacity・pixelated で frame を描きます。

frame を自分の channel で運ぶなら、engine の thread が `(slot, value)` を channel へ送り、`spawn` の task が受け取って `image.frame(slot, value)` を signal に書き、canvas の描画 closure がその signal を読んで `draw.gpu_frame` で描きます。全体の組立ては [`kivact::gpu_interop` の module doc](../../../crates/kivact-core/src/gpu_interop.rs) にあります。

### 制限

- 取り込みは、その window の device にだけ属します。ほかの window では描かれません。
- 取り込んだ texture には画像のような端の複製がないので、拡大縮小した bilinear では端の 1 texel が `Pixels` と違います。等倍なら一致します。
- ready が 100 ms 来ない frame は描かず、同じ値を二度は待ちません。
- Vulkan では、取り込んだ memory の実際の寸法と、image の usage を確かめられません。宣言と違う寸法や usage の texture は約束違反で、未定義動作になります（Direct3D 12 は寸法も flag も resource の記述で確かめ、`Platform` の失敗になります）。
- Direct3D 12 の debug layer を、window の device ができた後に同じ process で有効にすると、window の device が失われます。engine の `wgpu::Instance` は `InstanceFlags::DEBUG` と `VALIDATION` を付けずに作ります（`InstanceFlags::empty().with_env()`。`InstanceDescriptor` の既定値は debug build でそれらを有効にします）。
- 実験 feature です。Linux の実機、Vulkan の validation layer、別 process の engine は未検証で、API は変わることがあります。

## Creative の画像 surface

`kivact_ui_creative::viewport_surface(image)` は `Option<Pixels>` を表示します。画像が `None` の間は背景を描き、transform を渡さなければ全画像を中央へ fit します。`transform(Binding<ViewTransform>)` を渡すと中核の `zoom_pan` が pan・wheel zoom・`0` の fit を担当します。

`overlay(|draw, transform| ...)` の `Draw` は画像座標に変換済みです。`regions(Vec<SurfaceRegion>)` の bounds も同じ座標で、描画と支援技術の領域を共有します。`on_pointer` の `SurfacePointer.position` は画像座標、`region` はアプリの region ID です。`on_resize` は logical size と physical width/height、scale factor を変更時だけ通知します。画像や signal への書込みは描画 closure の外で行います。

CPU gradient・histogram と、上の bounded worker channel を surface に接続する実例は [creative catalog の画像モデル](../../../examples/kivact-ui-creative/src/creative/editor_images.rs) と [editor composition](../../../examples/kivact-ui-creative/src/creative/editors.rs) にあります。
