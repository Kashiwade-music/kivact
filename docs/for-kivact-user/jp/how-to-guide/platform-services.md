# プラットフォームのサービスを使う

[English](../../en/how-to-guide/platform-services.md)

このガイドでは、オペレーティングシステムとのやり取り（テキストのコピーと貼り付け、ファイルやフォルダーの選択、ファイルマネージャーでのパスの表示、インストール済みフォントの一覧、アプリの一時停止への対応、ウィンドウの大きさの設定と読み取り）を説明します。すべて `kivact::platform` にあります。

システムに何かを依頼するサービスは、**future** を返す関数です。リクエストは、関数を*呼んだ時点*で送られ、future は答えを返します。使い方は 2 通りです。

- 答えが要らないときは `spawn(platform::copy_text("…"))` と書きます。失敗は、イベントハンドラのエラーと同じく、最も近い `on_error` に届きます。
- 結果が欲しいとき、または失敗を自分で扱いたいときは、`spawn`・`action`・`resource` の中で `.await` します。

呼び出せるのは、セットアップ、イベントハンドラ、タスクの中です。`memo` やレンダーエフェクトの中では呼べません。

## テキストをコピー・貼り付けする

```rust,test_harness
use kivact::platform;
use kivact::prelude::*;

fn clipboard() -> impl IntoView {
    let pasted = signal(String::new());
    column((
        button("コピー").on_press(|| {
            spawn(platform::copy_text("Kivact からこんにちは"));
        }),
        button("貼り付け").on_press(move || {
            spawn(async move {
                let text = platform::read_text().await?;
                pasted.set(text);
                Ok::<(), UiError>(())
            });
        }),
        text(move || format!("貼り付けた内容: {}", pasted.get())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Clipboard").run(clipboard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_and_paste_use_the_scripted_clipboard() {
        let screen = kivact::test::render(clipboard);
        screen.get_by_role(Role::Button, "コピー").click();
        assert_eq!(screen.clipboard().as_deref(), Some("Kivact からこんにちは"));

        screen.set_clipboard("別のアプリから");
        screen.get_by_role(Role::Button, "貼り付け").click();
        screen.get_by_text("貼り付けた内容: 別のアプリから");
    }
}
```

`read_text` は、クリップボードにテキストがなければ空文字列を返します。テキスト入力は、コピー・切り取り・貼り付けを最初から自分で行います。これらの関数は、独自の「リンクをコピー」や「結果をコピー」ボタンに使ってください。

テストが示すのは、もう半分の話です。テストでは、クリップボードはメモリ上の代用品なので、本物のクリップボードに触れずに、アプリがコピーした内容を確かめたり、貼り付けられる内容を用意したりできます。

## ユーザーにファイル・複数のファイル・フォルダーを選んでもらう

`pick_file`・`pick_files`・`pick_folder`・`save_file` は、システム標準のダイアログを開きます。`FileDialog`（タイトル、開始ディレクトリ、提案するファイル名、フィルター）を受け取り、ユーザーがキャンセルすると `Ok(None)` で完了します。キャンセルはエラーではありません。

```rust,no_run
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn open_document() -> impl IntoView {
    let status = signal(String::from("何も開いていません"));
    column((
        button("開く…").on_press(move || {
            spawn(async move {
                let options = FileDialog {
                    title: "文書を開く".into(),
                    // （ラベル, ドットなしの拡張子）
                    filters: vec![("テキスト".into(), vec!["txt".into(), "md".into()])],
                    ..FileDialog::default()
                };
                match platform::pick_file(options).await? {
                    Some(path) => {
                        // blocking のプールで読みます。`?` が io::Error を UiError に変えます。
                        let text = blocking(move || std::fs::read_to_string(&path)).await?;
                        status.set(format!("{} バイト読み込みました", text.len()));
                    }
                    None => status.set("キャンセルしました".into()),
                }
                Ok::<(), UiError>(())
            });
        }),
        text(move || status.get()),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(open_document)
}
```

- `pick_files` は 1 つのダイアログで複数のファイルを選ばせ、システムが返す順で `Ok(Some(Vec<PathBuf>))` を返します。
- `pick_folder` が使うのは `title` と `directory` だけです。`file_name` と `filters` は無視されます。
- `save_file` は保存用の `pick_file` です。`file_name` で名前を提案できます。返るのはユーザーが選んだパスだけで、ファイルは作られません。書き込むのはあなたです。
- 選ばれたファイルの読み書きは普通のブロッキング I/O なので、上のように `blocking` で行います。[コンポーネントから非同期処理を実行する](async-and-tasks.md) を参照してください。

テストでは、ダイアログはスクリプトで答えます。ウィンドウは現れず、ユーザーが「何を選んだか」はテストが決めます。スクリプトのないダイアログはキャンセルされます。

```rust,test_harness
use std::path::PathBuf;
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn export() -> impl IntoView {
    let status = signal(String::new());
    column((
        button("書き出す").on_press(move || {
            spawn(async move {
                let options = FileDialog {
                    title: "書き出し".into(),
                    file_name: Some("report.csv".into()),
                    ..FileDialog::default()
                };
                let message = match platform::save_file(options).await? {
                    Some(path) => format!("保存先: {}", path.display()),
                    None => "キャンセルしました".to_string(),
                };
                status.set(message);
                Ok::<(), UiError>(())
            });
        }),
        text(move || status.get()),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Export").run(export)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_test_decides_what_the_user_picks() {
        let screen = kivact::test::render(export);

        screen.respond_file_dialog(Some(PathBuf::from("out/report.csv")));
        screen.get_by_role(Role::Button, "書き出す").click();
        let expected = format!("保存先: {}", PathBuf::from("out/report.csv").display());
        screen.get_by_text(&expected);

        // 答えを用意していないダイアログはキャンセルされます。
        screen.get_by_role(Role::Button, "書き出す").click();
        screen.get_by_text("キャンセルしました");
    }
}
```

`pick_file`・`save_file`・`pick_folder` には `Screen::respond_file_dialog` を、`pick_files` には `respond_files_dialog` を使います。

## ファイルやフォルダーを開く、場所を表示する

`open_path` は、ファイルをシステムが関連付けたアプリで開き、フォルダーをファイルマネージャーで開きます。`reveal_path` は、ファイルマネージャーで対象を選択した状態で表示します。どちらもパスを受け取り、相対パスはプロセスのカレントディレクトリを基準に解決されます。

```rust,no_run
use kivact::platform;
use kivact::prelude::*;

fn exported() -> impl IntoView {
    row((
        button("画像を開く").on_press(|| {
            spawn(platform::open_path("exports/frame.png"));
        }),
        button("フォルダーで表示").on_press(|| {
            spawn(platform::reveal_path("exports/frame.png"));
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Exports").run(exported)
}
```

- Windows（エクスプローラー）と Linux（`xdg-open`）で使えます。それ以外では `ErrorKind::Unsupported` で失敗します。Linux の `reveal_path` は項目を選択できないため、その項目を含むフォルダーを開きます。
- `Ok(())` は、ハンドラを起動できたことを表します。その後どうなったかは報告されません。存在しないパスや、起動できないハンドラは、パスを含む `ErrorKind::Platform` のエラーになります。
- テストでは何も起動しません。`screen.opened_paths()` と `screen.revealed_paths()` が、アプリが依頼したパスを順に返します。`screen.fail_next_platform_request("理由")` で次のリクエストを失敗させれば、エラーメッセージのテストもできます。

### 失敗を自分で扱う

エラーを `on_error` に任せず、自分でユーザーに伝えたいときは、future を `await` してエラーの種類で `match` します。

```rust,no_run
use kivact::diagnostics::ErrorKind;
use kivact::platform;
use kivact::prelude::*;

fn open_with_message(path: &'static str, message: Signal<String>) {
    spawn(async move {
        match platform::open_path(path).await {
            Ok(()) => {}
            Err(error) if *error.kind() == ErrorKind::Unsupported => {
                message.set("このシステムでは、ここからファイルを開けません".into());
            }
            Err(error) => message.set(format!("開けませんでした: {error}")),
        }
    });
}
```

## インストール済みのフォントを一覧する

`platform::font_families()` は、ウィンドウが使えるフォントのファミリー名を、重複なし・名前順・元の大文字小文字のまま返します。システムのフォントと、`App::font(..)` で登録したフォントが含まれます。呼び出した時点のスナップショットであり、リアクティブな値ではありません。

```rust,no_run
use kivact::prelude::*;

fn font_list() -> impl IntoView {
    let families = kivact::platform::font_families();
    scroll(text(families.join("\n"))).style(style().size(320, 400))
}

fn main() -> kivact::Result<()> {
    App::new("Fonts").run(font_list)
}
```

一覧のファミリー名は `style().font_family("…")` に渡せます。フォントをアプリに同梱するには、最初の画面の前に登録します: `App::new("…").font(include_bytes!("../assets/Inter-Regular.ttf"))`。ファミリー名は、ファイル名ではなく、フォントの中に書かれた名前です。

## アプリの一時停止に対応する

`platform::on_lifecycle` は、セットアップで、アプリのライフサイクルイベント（`Suspended`・`Resumed`・`MemoryWarning`）のコールバックを登録します。コンポーネントが存在する間だけ有効で、そのコンポーネントのコンテキストで実行されます。どのイベントが届くかはプラットフォームによって異なります。メモリー警告が報告されるかどうかは、`platform::capabilities().memory_warnings` で分かります。

```rust,no_run
use kivact::platform::{self, LifecycleEvent};
use kivact::prelude::*;

fn memory_status() -> impl IntoView {
    let warnings = signal(0usize);
    platform::on_lifecycle(move |event| {
        if event == LifecycleEvent::MemoryWarning {
            // 作り直せるキャッシュを捨てます。
            warnings.update(|count| *count += 1);
        }
    });
    text(move || format!("メモリー警告: {}", warnings.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Lifecycle").run(memory_status)
}
```

ウィンドウが今描画できるかどうかは別の問題で、リアクティブな `platform::surface_active()` で読みます。サーフェスの喪失や非表示は、ライフサイクルイベントになりません。メモリー警告では、コールバックの前にウィンドウが使われていないテキストと画像のキャッシュをすでに解放しています。縮めるのは、自分のコードが持つキャッシュだけにしてください。

## ウィンドウの大きさを設定し、読み取る

最初の大きさと、利用者が縮められる最小の大きさは、`App` に設定します。値は論理ピクセルで、既定は 960 × 720 です。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    text("サイズ変更できます")
}

fn main() -> kivact::Result<()> {
    App::new("Editor")
        .window_size(1280, 800)
        .min_window_size(640, 480)
        .run(app)
}
```

大きさが 0・負・有限でない場合や、最小の大きさが開始時の大きさより大きい場合は、`run` がウィンドウを開かずにエラーを返します。ウィンドウの位置を指定する API は、Kivact にはありません。

**現在の大きさを読む**には `platform::window_size()` を使います。追跡付きの読み取りなので、これを読むスタイルやビューは、リサイズのたびに更新されます。CSS ならメディアクエリを使う場面の、レスポンシブなレイアウトに使えます。

```rust,test_harness
use kivact::platform;
use kivact::prelude::*;

fn shell() -> impl IntoView {
    let narrow = memo(move || platform::window_size().width < 600.0);
    column(text(move || {
        if narrow.get() { "狭いレイアウト" } else { "広いレイアウト" }
    }))
}

fn main() -> kivact::Result<()> {
    App::new("Responsive").run(shell)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_follows_the_window() {
        let screen = kivact::test::render(shell); // 既定は 800 x 600
        screen.get_by_text("広いレイアウト");
        screen.resize(400.0, 600.0);
        screen.get_by_text("狭いレイアウト");
    }
}
```

比較は `memo` で包むと、ドラッグ中の 1 ピクセルごとではなく、答えが変わったときだけビューが更新されます。

**システムに大きさの変更を頼む**には `spawn(platform::request_window_size(640, 480))` を使います。大きさは、設定した最小値に切り上げられます。これは依頼なので、システムが断ることもあり、その場合 future は `Platform` エラーで完了します。枠なしの全画面表示には `spawn(platform::set_fullscreen(true))` を呼び、状態は `platform::fullscreen()` で読みます。

環境のそのほかの読み取りも同じ規則です（追跡付きで、セットアップ・ハンドラ・タスク・エフェクトから使えます）。

| 関数 | 返すもの |
|---|---|
| `scale_factor()` | 論理ピクセルあたりの物理ピクセル数。ピクセル単位のレンダラーに渡す大きさに掛けます |
| `reduced_motion()` | システムが動きを減らすよう求めているか |
| `safe_area()` | システムの UI が覆うかもしれない余白（デスクトップではゼロ） |
| `surface_active()` | ウィンドウが今描画できるか |
| `capabilities()` | プラットフォームが対応していること（タッチ、ペン、ソフトウェアキーボード、GPU など）。アプリの実行中は変わりません |

テストでは、`RenderOptions` が開始時の大きさ・倍率・ダークモード・動きを減らす設定を決め、`Screen::resize`・`set_scale`・`set_dark`・`set_reduced_motion`・`set_safe_area` で画面の実行中に変えられます。[アプリをテストする](testing.md#環境を設定する) を参照してください。

## 関連項目

- [コンポーネントから非同期処理を実行する](async-and-tasks.md): future、`spawn`、`blocking`。
- [アプリをテストする](testing.md): テストでのクリップボード、ダイアログ、パスのスクリプト化。
- [値と要素をアニメーションさせる](animation.md): `reduced_motion` の実際の使い方。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::platform` を開いてください。
