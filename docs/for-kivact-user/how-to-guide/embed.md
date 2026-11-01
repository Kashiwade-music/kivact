# ホストの window に埋め込む（プラグインの editor）

feature `embed` を有効にすると、ホストの window の中に、desktop と同じ App・部品・signal・試験で UI を作れます。Kivact は window と UI を担当し、CLAP・VST3・nih-plug・clack との glue、音声、parameter、保存はアプリ（wrapper）が担当します。枠組み固有の adapter は提供しません。

DAW プラグインの editor は[製品戦略](../../for-kivact-developer/strategy/20261009_strategy.md)の主対象です。Windows と Linux X11 は「実験的に動作」、macOS（NSView）は設計対象です。正式対応には主要な DAW（REAPER、Bitwig Studio、Ableton Live、Cubase など）での実機確認と、両 OS の G-03（IME、accessibility）の実機確認が必要です。macOS は実機と保守者がそろったときに実装し、共通 backend 契約と契約試験の通過を完成条件とします。

## UI を書く人へ

画面のコードは desktop と同じです。component、signal、binding、effect と `kivact::test` を使い、親の handle や plugin の規格を画面に持ち込みません。Rust 未経験の React・SolidJS 開発者は[入門](../getting-started/README.md)と同じ手順で部品を組み立てられます。`App::embed` を呼ぶ入口は Rust の wrapper 作者が用意します。

| React・SolidJS の知識 | Kivact の画面 |
|---|---|
| component、state、JSX | 関数、`signal`、view と値を読む closure |
| props の外部 store | component が受け取る `Arc` の model |
| `useSyncExternalStore` | `frame_loop` で読む、または channel と `task::spawn` で通知を受ける |
| gesture の開始と終了 | 標準部品の `on_edit(EditPhase::{Begin, Commit, Cancel})` |

parameter、preset、保存する状態は App の外でプラグインが保持します。editor の選択や hover は App 内で構いません。閉じて再び開くたびに root が作られます。設定の `App` は Clone なので再利用できます。

次は公開 API だけの画面の例です。atomic は外部 parameter の代役で、実際の begin・set・end は wrapper がホストへ送ります。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::EditPhase;
use kivact::motion::{frame_loop, FrameRate};
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

fn editor(parameter: Arc<AtomicU32>) -> impl IntoView {
    let value = signal(f32::from_bits(parameter.load(Ordering::Relaxed)) as f64);
    let incoming = parameter.clone();
    let clock = frame_loop(move |_| {
        value.set(f32::from_bits(incoming.load(Ordering::Relaxed)) as f64);
    });
    clock.start(FrameRate::Display);
    let write = parameter.clone();
    let control = binding(move || value.get(), move |next: f64| {
        // wrapper.set(next): automation と同じ保存先へ書く。
        write.store((next as f32).to_bits(), Ordering::Relaxed);
        value.set(next);
    });
    slider("Gain", control).range(0.0..=1.0).on_edit(move |phase| {
        match phase {
            EditPhase::Begin => { /* wrapper.begin() */ }
            EditPhase::Commit | EditPhase::Cancel => { /* wrapper.end() */ }
        }
    })
}
# fn main() {}
```

順序は Begin → binding の書込み（set）… → Commit です。Escape や pointer の取消しでは、元の値が binding に書き戻された**後**に Cancel が届きます。Cancel では end だけを送り、値を再び書き戻しません。ホストからの自動化更新で gesture を開始しません。
通知型の model なら channel を `task::spawn` で受けて signal を進めます。audio thread を UI の待機や lock で止めない設計は wrapper の担当です。

通知型の model の受信側は次の形です。`async_channel` はアプリが選ぶ channel の説明用で、Kivact の依存として追加するものではありません。受信 task は component の owner とともに取り消されます。

```rust,ignore
fn updates(incoming: async_channel::Receiver<f64>) -> impl IntoView {
    let value = signal(0.0);
    kivact::task::spawn(async move {
        while let Ok(next) = incoming.recv().await {
            value.set(next);
        }
    });
    text(move || format!("Gain: {}", value.get()))
}
```


## wrapper を書く人へ

Rust のフレームワーク開発者が規格の glue と起動の入口を担当します。`createRoot(container).render(...)` に当たる入口が `App::embed(parent, root)`、`root.unmount()` に当たる操作が `Embedded` の drop です。各規格のホスト呼出しの thread 条件を wrapper が守り、必要な配送はホストの run loop へ送ります。App の callback と root からホスト GUI thread の返事を待つと、起動や drop と互いに待つため禁止です。

### 全 OS で同じ契約

1. **thread:** `App::embed` はホストの GUI thread で呼びます。`root` とすべての App の callback（`Parent::on_resize_request` を含む）は App の UI thread で走ります。ホストの GUI thread と同じかどうかは約束しません。root と callback はホストの GUI thread を待ってはいけません。
2. **開く:** root は一度だけ走り、`App::embed` は root が終わってから返ります。root の未処理の panic は元の診断を `Err` で返します。起動失敗も通常は後始末を待ちますが、drop と同じ所有 thread ごとの最大2秒の待機上限があり、上限後は module を保持したまま未完了の後始末が `Err` の返却後も続き得ます。不正な初期サイズ・倍率は `Message`、不正な親・OS 資源の不足・未対応の OS は `Platform` です。
3. **setter:** `Embedded` は `Send + Sync` です。`set_size`・`set_scale` は App の UI thread の queue に変更を積んですぐ返ります。`size`・`scale` は最後に適用した値であり、setter の直後に新しい値を返すとは限りません。共有する場合は `Arc<Embedded>` を使えます。
4. **値の検査:** 0 または backend の上限を超えるサイズ、非有限または 0 以下の倍率を setter に渡すと、`tracing` に警告して無視します。App は動き続けます。上限は下の能力表にあります。
5. **単位:** ホストとの境界（`set_size`・`size`・`on_resize_request`）は物理 px の `u32`、App の `window_size`・`platform::request_window_size` は論理 px です。倍率は物理 px / 論理 px。`set_scale` は物理サイズを保ちます。
6. **サイズの交渉:** `platform::request_window_size` は最小サイズで丸めた論理サイズを物理 px に直し、`on_resize_request` へ渡します。true なら App の UI thread の queue に積みます。callback の間にホストが `set_size` を呼んでいればホストのサイズが勝ちます。false は `Platform`、callback なしは `Unsupported`。開いている途中の要求は開き終わってから処理します。
7. **drop:** ホストの GUI thread で drop することを推奨します。App の callback の外では通常、App と native window の後始末の完了を待ちます。App が所有する thread（UI callback が走る thread を含む）は 1 本につき最大 2 秒待つため、複数なら待機が累積します。上限を超えた場合、未完了の後始末が drop の後も続くことがありますが、Kivact が module を保持するためホストは plugin を unload できます。App の callback の中で drop すると、実行中の turn は最後まで走り、その後に後始末します。以後の turn と入力は配送せず、turn の終わりに残る待機中の action は実行しません。
8. **失敗:** 未処理の panic・backend の失敗・App の exit は App を止めます。起動後の未処理の panic はその turn の終わりで止め、同じ turn の残りの effect は走り、次の turn の event は配送しません。owner の error boundary が回復した panic は停止しません。停止後は `error()` が理由を返し、setter を無視し、getter は最後の値を返します。handle の drop は引き続き必要です。親が先に破棄される場合は安全に止まり、`error()` は `None` です。
9. **unload:** thread が待機上限を超えた場合や、process に残る thread・provider を持つ library を使った場合は Kivact が自分の code の module を process 終了まで保持します。wrapper に module を保持する処理は不要です。ホストは editor を閉じた後に plugin を unload できます。アプリが直接作る thread の停止と join はアプリが管理します。

### 親とホストの呼出し

`Parent` の構築は raw handle の値を保存するだけで、`App::embed` が親を検査します。ホストの既定倍率を自分で決める wrapper は、必ず同じ倍率を `Parent::scale` に渡します。

| ホストの値 | 構築 |
|---|---|
| CLAP `clap_window.win32`、VST3 `"HWND"`、nih-plug `Win32Hwnd` | `Parent::win32(hwnd)` |
| CLAP `clap_window.x11`、VST3 `"X11EmbedWindowID"`、nih-plug `X11Window` | `Parent::x11(id)` |

```rust,no_run
use kivact::{prelude::*, embed::{Embedded, Parent}};
# fn attach(hwnd: *mut std::ffi::c_void) -> kivact::Result<Embedded> {
App::new("Editor").window_size(640, 480)
    .embed(Parent::win32(hwnd).scale(1.25), || text("Editor"))
# }
```

| ホストの呼出し（CLAP / VST3） | wrapper の処理 |
|---|---|
| 開く前の `set_scale` / `setContentScaleFactor` | 倍率を保存 |
| 開く前の `get_size` / `getSize` | `round(論理サイズ × 倍率)` を返す |
| `set_parent` / `attached` | 同じ倍率を Parent に渡し embed、handle を保持 |
| `set_size` / `onSize` | `Embedded::set_size` |
| 開いた後の `get_size` / `getSize` | `Embedded::size` |
| 倍率変更 | `Embedded::set_scale`、必要なら新しい物理サイズをホストへ要求 |
| `adjust_size` / `checkSizeConstraint` | wrapper の最小サイズで丸める |
| `destroy` / `removed` | handle を drop |

title はホストが管理します。`App::window_size` の既定値は 960×720 論理 px です。`App::min_window_size` はプラグインの要求を丸めますが、ホストの `set_size` は制限しません。ホストからの変更は wrapper が自分の最小サイズで丸めます。

### サイズ変更の順序

論理サイズを保って倍率を変える場合は、`set_scale` の後に `round(論理サイズ × 新倍率)` をホストへ要求します。wrapper が自分のサイズを答えるときは、**新しい論理サイズを保存 → ホストへ要求 → 拒否なら元へ戻す**の順です。nih-plug は要求の中で `editor.size()` を読み直します。

```rust,no_run
use kivact::{prelude::*, embed::Parent};
# fn parent(hwnd: *mut std::ffi::c_void) -> Parent {
Parent::win32(hwnd).scale(1.25).on_resize_request(|width, height| {
    // wrapper が物理サイズをホストへ求め、受理の bool を返す。
    let _ = (width, height);
    true
})
# }
# async fn resize() -> kivact::Result<()> {
kivact::platform::request_window_size(640, 480).await?;
# Ok(())
# }
```

受理後のサイズ変更は queue を通ります。交渉中のホストの訂正が優先され、callback から setter を呼んでも App の callback へ再入しません。ホスト GUI thread が必要な要求は run loop へ配送し、同期の応答を待たずに受理を返し、後続のホストのサイズ通知を `set_size` で確定させます。

枠組みの型を使う次の部分は説明用で、依存や完成した adapter を提供する例ではありません。

```rust,ignore
match parent {
    ParentWindowHandle::Win32Hwnd(hwnd) => Parent::win32(hwnd),
    ParentWindowHandle::X11Window(id) => Parent::x11(id),
}
// nih-plug: GuiContext::request_resize(); CLAP: request_resize(w, h)
// VST3: resizeView(view, rect)。要求前に wrapper の size を更新する。
```

### 終了と thread

`blocking` の仕事は `CancelToken` を定期的に見て終了してください。上限を超えた仕事や native library のための module の保持は Kivact が行います。必要な code は process 終了まで残ることがあります。破棄中の panic は二重 panic を避けるため残る資源を漏らして警告します。詳しい順序は[設計の埋め込み](../../for-kivact-developer/design.md#埋め込み)にあります。

## 対応する能力

| 能力 | Windows（実験的に動作） | Linux X11（実験的に動作） |
|---|---|---|
| CPU/GPU、mouse、wheel、key | あり | あり（xkbcommon、DISPLAY が必要。XWayland も対象） |
| IME | Imm32、再変換 | XIM。server がないときは通常の XKB 入力で動作。再変換は未対応 |
| accessibility | UIA | AT-SPI（accesskit_unix）。通常の起動でも常駐 executor のため module を保持 |
| clipboard | あり | arboard（初回利用前に module を保持） |
| file・folder dialog | あり | rfd（初回利用前に module を保持）。portal に子 XID を親として渡す。内部 zenity fallback は親指定なし |
| open / reveal | あり | あり |
| file drop | drop のみ（hover・cancel なし） | XDND v5、drop のみ（hover・cancel なし）。子 window まで XdndAware を探す source が対象で、top-level だけを見る source（GTK など）は未確認 |
| touch / pen / fullscreen | 未対応 / 未対応 / Unsupported | 同左 |
| setter の物理寸法の上限 | `i32::MAX` | `u16::MAX` |
| frame wake | SetTimer の分解能（通常約 64 Hz） | timerfd |

Wayland native の埋め込みは見送りです。未処理キーをホストへ返す仕組みと DAW のキー横取りへの対策は提供しません。実機の DAW・IME・Narrator・Orca・dialog は未実施です。対象と操作、未実施の理由は[手動確認一覧](../../for-kivact-developer/manual-verification.md#埋め込みwindows実験)にあります。自動 E2E は人による確認の代わりに数えません。

## 手動確認用の host

`kivact-embed-host` は公開 API だけで親 window を用意します。`--two` は同じ 800×600 物理 px の親に独立した二つの App を並べます。`TestParent::parent_in` の位置と寸法も物理 px です。`--smoke` は 5 秒後に閉じます。

```bash
dev/tools/window-lock.sh -- cargo run -p kivact-embed-host -- --smoke
dev/tools/window-lock.sh -- cargo run -p kivact-embed-host -- --two --scale 1.25 --smoke
```

実際の IME・支援技術の操作は手動確認一覧の手順に従います。DAW での確認には所有者の wrapper を使います。
