# Platform のサービスを呼ぶ

clipboard、file dialog、fullscreen、path を開く操作は `kivact::platform` の future を使います。handler から `spawn` すると、失敗はその owner の error の受け口へ届きます。

```rust,no_run
use kivact::prelude::*;
use kivact::platform;

fn copy_button() -> impl IntoView {
    button("結果をコピー").on_press(|| {
        spawn(platform::copy_text("計算結果"));
    })
}
```

要求は関数を呼んだ時点で送られます。future を捨てても「送らなかった」ことにはなりません。結果を区別したい場合は task の中で await し、cancel と error を呼出し側で処理します。`pick_file` などの `Ok(None)` は利用者による取消で、失敗ではありません。サービスの不足は `Unsupported`、提供されたサービスの失敗は platform error として扱います。

`scale_factor`、`window_size`、`safe_area`、`surface_active`、`reduced_motion` と `capabilities` は環境の tracked な値を返します。必要な変更だけを reactive な closure で読みます。window の初期サイズは `App` に設定し、有限・正の値を渡します。描画中の座標へ倍率を手動で掛けないでください。

自動試験の `Screen` は clipboard や dialog の結果を script できるため、OS の clipboard や本物の dialog を操作せず結果を検証できます。実デスクトップでの IME・portal・monitor 移動は [手動確認の一覧](../../for-kivact-developer/manual-verification.md)を使います。

## font family の一覧を読む

`platform::font_families()` は window が使う font database の family 名を同期で返します。system の font と `App::font` で登録した font を含み、元の大文字小文字を保って名前順に並べ、重複を除きます。setup・handler・task・effect の中で呼べます。反応を追跡する値ではなく、呼び出した時点の一覧です。

```rust,no_run
use kivact::prelude::*;

fn font_list() -> impl IntoView {
    column(text(kivact::platform::font_families().join("\n")))
}
```

## フォルダーを選ぶ

`pick_folder` に題名と初期フォルダーを渡し、task で結果を待ちます。取消しは `Ok(None)` です。`FileDialog` の `file_name` と `filters` はフォルダーの選択には使いません。

```rust,no_run
use kivact::prelude::*;
use kivact::platform::{self, FileDialog};

fn folder_picker() -> impl IntoView {
    let folder = signal(String::new());
    column((
        button("作業フォルダーを選ぶ").on_press(move || {
            spawn(async move {
                if let Some(path) = platform::pick_folder(FileDialog {
                    title: "作業フォルダー".into(),
                    ..Default::default()
                }).await? {
                    folder.set(path.display().to_string());
                }
                Ok::<(), kivact::UiError>(())
            });
        }),
        text(move || folder.get()),
    ))
}
```

## ファイルやフォルダーを開く

`open_path` は OS の関連付けられたアプリでファイルを開き、フォルダーならファイルマネージャーを開きます。相対パスの基準は process の作業フォルダーです。成功はアプリの起動までを表し、その後のアプリ内の操作は追跡しません。

```rust,no_run
use kivact::prelude::*;
use kivact::platform;

fn open_export() -> impl IntoView {
    button("書き出した画像を開く").on_press(|| {
        spawn(platform::open_path("exports/frame.png"));
    })
}
```

## ファイルの場所を表示する

`reveal_path` は OS の file manager で対象を示します。選択できない環境では対象を含むフォルダーを開きます。

```rust,no_run
use kivact::prelude::*;
use kivact::platform;

fn reveal_export() -> impl IntoView {
    button("保存先を表示").on_press(|| {
        spawn(platform::reveal_path("exports/frame.png"));
    })
}
```

未対応の OS・サービスでは、いずれも `UiError`（`ErrorKind::Unsupported`）を返します。`open_path`・`reveal_path` の対応 OS は Windows と Linux です。存在しないパスや起動の失敗は `ErrorKind::Platform` になります。task の失敗を画面に出すには [診断とエラー](diagnostics.md)を参照してください。

## ライフサイクルを受け取る

setup で `platform::on_lifecycle` に登録すると、部品が表示されている間だけ通知を受け取ります。初期の resumed 状態は通知せず、実際の `Suspended`・`Resumed` の切替と、毎回の `MemoryWarning` を登録順に届けます。同じ suspend/resume の重複通知はまとめます。

```rust,no_run
use kivact::prelude::*;
use kivact::platform::{self, LifecycleEvent};

fn memory_status() -> impl IntoView {
    let warnings = signal(0usize);
    platform::on_lifecycle(move |event| {
        if event == LifecycleEvent::MemoryWarning {
            warnings.update(|count| *count += 1);
        }
    });
    text(move || format!("メモリー警告: {}", warnings.get()))
}
```

callback は登録した owner の中で untracked に実行され、失敗はその error boundary に届きます。部品の破棄で解除されます。callback が条件分岐の削除を要求した場合、通常の event の終わりに反映され、それ以降の通知を受け取りません。

`surface_active()` は描画できる現在の状態です。surface の喪失・復旧や非表示は lifecycle 通知を合成しません。suspend/resume callback では更新済みの状態が読め、resume しても surface がなければ false のままです。memory warning では host が text と使っていない asset cache を解放してから callback を呼びます。アプリ自身の再生成できる cache は callback で縮めてください。登録した font や現在描画している asset は保持します。

## window のサイズを求める

`spawn(platform::request_window_size(640, 480))` は論理 px で内容のサイズを求めます。有限で正の f32 に収まらない値は `Message`、`App::min_window_size` より小さい値は最小サイズに丸めます。desktop は OS に要求し、埋め込みは `Parent::on_resize_request` に `round(論理サイズ × 倍率)` の物理 px を渡します。callback が true なら App の UI thread の queue にサイズ変更を積み、false は `Platform`、callback がない場合は `Unsupported` です。callback の間にホストが `set_size` を呼んでいればホストのサイズが優先されます。開いている途中の要求は開き終わってから処理します。callback は App の UI thread で走り、ホストの GUI thread の返事を待ってはいけません。[埋め込みの共通契約](embed.md#全-os-で同じ契約)を参照してください。

future は callback の受理・拒否の結果を待ちます。setter の反映完了を待つものではありません。wrapper は callback で使う lock を開いている間保持しないでください。
