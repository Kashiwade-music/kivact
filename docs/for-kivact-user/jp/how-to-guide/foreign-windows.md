# プラグインのエディタを専用のウィンドウに表示する

[English](../../en/how-to-guide/foreign-windows.md)

DAW はプラグインのエディタを小さな独立ウィンドウに表示するのが一般的です。そのためには `foreign_window(title, open)` を使います。Kivact はアプリの通常のウィンドウを開き、その内容領域全体を 1 つのネイティブ子ウィンドウにします。別のライブラリがその子ウィンドウにプラグインのエディタを描きます。Kivact が描くのはテーマの背景だけなので、2 つが重なって描かれることはありません。

このガイドは、プラグインのインスタンスを所有し、その GUI 関数を呼ぶホストアプリを前提にします。`open_editor`、`set_editor_size`、`close_editor` が何をするかはプラグインのラッパーが決めます（CLAP と VST3 では `vivace-host` が行います）。

## ウィンドウを宣言する

`foreign_window` は `window` と同じく view です。エディタを開いてよい場所、`open` を設定する操作の隣に置きます。

```rust
use std::sync::Arc;

use kivact::platform::NativeWindow;
use kivact::prelude::*;
use kivact::view::ForeignChild;
use kivact::view::SurfaceSize;
use kivact::view::foreign_window;

struct Editor;

impl Editor {
    fn open(&self, _parent: NativeWindow, _size: SurfaceSize) {}
    fn resize(&self, _size: SurfaceSize) {}
    fn close(&self) {}
}

fn plugin_window(editor: Arc<Editor>, open: Signal<bool>) -> impl IntoView {
    let resize_editor = editor.clone();
    let close_editor = editor.clone();
    foreign_window("Plugin editor", open)
        .size(480.0, 320.0)
        .min_size(320.0, 200.0)
        .mode(WindowMode::Tool)
        .on_ready(move |child: ForeignChild| {
            // Give the plugin the child as its parent and its physical size.
            editor.open(child.window(), child.size());
        })
        .on_resize(move |size: SurfaceSize| {
            // The user resized the window, or it moved to another monitor.
            resize_editor.resize(size);
        })
        .on_destroy(move || {
            // Detach the plugin's GUI now: the child still exists here.
            close_editor.close();
        })
}
```

`open` が true の間だけウィンドウが存在します。false を書くと閉じます。タイトルバーの閉じる操作は `on_dismiss` を呼び、`Handled::Yes` / `Handled::PreventDefault` を返さなければ `open` を false にします。宣言した owner の破棄（周囲の `show` の枝が消えた場合など）でも、アプリの終了時でも閉じます。

## 3 つのコールバック

| コールバック | 呼ばれるとき | すること |
| --- | --- | --- |
| `on_ready(child)` | 開くたびに 1 回、子ウィンドウができた後 | プラグインの GUI を `child.window()` を親として開く。`child.size()` はその時点の物理サイズ。 |
| `on_resize(size)` | 子ウィンドウの物理サイズまたはスケールが変わったとき | 新しい物理サイズとスケールをプラグインへ渡す。`size.logical` は論理サイズ、`size.physical_width`/`height` は物理サイズ。 |
| `on_destroy()` | 開くたびに 1 回、子ウィンドウとウィンドウが破棄される前 | プラグインの GUI を閉じ、それが所有するものを drop する。 |

`on_ready` と `on_resize` は通常の Kivact の handler なので、エラーを返せます。返した `UiError` はアプリのエラーハンドラへ届きます。`on_destroy` は引数のない `Fn()` です。

`on_destroy` の保証は通常の閉じ方すべてに及びます。`open` に false を書く、タイトルバーで閉じる、宣言した枝を破棄する、アプリを終了する（`AppHandle::close` を含む）のいずれでも呼ばれます。ウィンドウを開く途中のプラットフォームの失敗や panic では、ウィンドウは `on_destroy` を実行せずに破棄されます。これが唯一の例外です。

## プラグインからのリサイズ

プラグインが新しいサイズを要求したときは `ForeignChild::request_size` を呼びます。プラグインの API が報告する値に合わせて物理ピクセルを受け取り、子ウィンドウがそのサイズになるようにウィンドウをリサイズします。要求は `min_size` にクランプされ、拒否はプラットフォームのエラーになります。ウィンドウが閉じた後の future は `ErrorKind::Closed` で解決します。

```rust
use kivact::prelude::*;
use kivact::view::ForeignChild;
use kivact::view::foreign_window;

fn resizer(child: Signal<Option<ForeignChild>>, wanted: Signal<Option<(u32, u32)>>) {
    effect(move || {
        let Some((width, height)) = wanted.get() else {
            return;
        };
        let Some(child) = child.get_untracked() else {
            return;
        };
        spawn(async move {
            let _ = child.request_size(width, height).await;
        });
    });
}
```

`on_ready` でハンドルを signal に保存します。

```rust,ignore
use kivact::view::ForeignChild;
use kivact::view::foreign_window;

let child = signal(None::<ForeignChild>);
foreign_window("Plugin editor", open).on_ready(move |handle| {
    child.set(Some(handle));
});
```

ユーザーがリサイズできない固定サイズのエディタでは `.resizable(false)` を使います。プラグイン自身が要求するサイズには `request_size` が引き続き使えます。プラグインに合わせて切り替えるなら、`resizable` に signal を渡します。

```rust,ignore
use kivact::view::foreign_window;

let can_resize = signal(true);
foreign_window("Plugin editor", open).resizable(can_resize);
```

## スケールの変更

モニター間の移動でも `on_resize` が呼ばれます。新しい `SurfaceSize` が新しい `scale_factor` を持ちます。物理サイズと一緒にプラグインへ渡してください（多くのラッパーは `set_editor_size` と `set_editor_scale` の組を持っています）。X11 では OS のモニタースケールは通知されないため、ユーザーのリサイズと `request_size` だけが届きます。

## キーボードフォーカス

子ウィンドウは実際の OS の子ウィンドウです。プラグインの操作部品をクリックすると、キーボードはプラグイン自身のウィンドウへ渡ります。Kivact がプラグインの中へキーを配送することはありません。foreign window がアクティブになると、Kivact は子ウィンドウの最初の子へフォーカスを渡します。ほとんどのプラグインのエディタはこれを前提にしています。複数のトップレベル子ウィンドウを持つプラグインでは、キーを受け取るまでに 1 回クリックが必要なことがあります。

## プラットフォームと capability

foreign window は Windows（Win32）と Linux X11 で動きます。Wayland、埋め込みアプリ、capability のないテストホストでは、`foreign_window` は `ErrorKind::Unsupported` を 1 回報告して閉じたままになります。`open` は変更されません。実行中のホストが開けるかどうかは `platform::capabilities().foreign_window` で分かります。

## テストする

テストホストは、テストが capability を要求したときに仮想の foreign window を開きます。

```rust
use kivact::prelude::*;
use kivact::test::RenderOptions;

let options = RenderOptions::default().foreign_windows();
```

`Screen::foreign_child()` はネイティブハンドルを返し、`Screen::is_resizable()` は現在のサイズ固定を報告します。通常の `resize`、`set_scale`、`request_close` もそのウィンドウに対して動きます。`kivact::test::is_native_window_alive` はハンドルがまだ生きているかを返します。`on_destroy` の中で使えば、子ウィンドウがプラグインの GUI より長く生きることを確かめられます。

```rust,ignore
use std::cell::Cell;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::test::RenderOptions;

let alive = Rc::new(Cell::new(None::<bool>));
let seen = alive.clone();
let (screen, ()) = kivact::test::render_with_handle(
    RenderOptions::default().foreign_windows(),
    move || {
        let open = signal(true);
        (
            (),
            column((
                text("host"),
                foreign_window("Plugin editor", open)
                    .on_ready(move |child| {
                        seen.set(Some(kivact::test::is_native_window_alive(child.window())));
                    })
                    .on_destroy(move || {
                        // The window is still alive while this runs.
                    }),
            )),
        )
    },
);
let editor = screen.window("Plugin editor");
assert!(editor.foreign_child().is_some());
```

## 関連項目

- [複数のウィンドウを開く](windows.md): 通常の Kivact のウィンドウ、モード、context、テスト。
- [プラグインのウィンドウの中に Kivact の UI を開く](embed.md): 逆向きの用途で、プラグインの作者向け。
