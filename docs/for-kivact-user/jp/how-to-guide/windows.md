# 複数のウィンドウを開く

[English](../../en/how-to-guide/windows.md)

アプリは最初、main window という 1 つの OS ウィンドウで始まります。`window(title, open)` はもう 1 つ追加します。`open` が true の間だけ content が mount されて専用の OS ウィンドウに表示され、`open` が false になると content は unmount されてウィンドウが閉じます。content は、宣言した場所の context を引き継ぎます。React の Portal と同じです。

## ウィンドウを開く

`window` は view です。content に引き継がせたい context の中、開く操作の隣に置きます。

```rust
use kivact::prelude::*;

fn editor() -> impl IntoView {
    let settings_open = signal(false);
    column((
        button("設定").on_press(move || settings_open.set(true)),
        window("設定", settings_open)
            .size(480, 360)
            .mode(WindowMode::Tool)
            .placement(WindowPlacement::CenterOnParent)
            .content(|| text("設定はここに表示されます")),
    ))
}
```

`title` は reactive です。closure か signal を渡すと、OS のタイトルがそれに従います。`open` は入力部品が取るのと同じ普通の `bool` の binding です。`true` を書くと content が新しいウィンドウに mount され、`false` を書くと unmount されてウィンドウが閉じます。宣言した owner が破棄されたときも閉じます。すでに開いているウィンドウの `open` に `true` を書き直しても何も起きません。前面には出ません（[ウィンドウを前面に出す](#ウィンドウを前面に出す)を参照）。

`content` はファクトリーです。ウィンドウを開くたびに、新しい owner の下で実行されます。その中で作った signal はウィンドウが閉じると破棄されるので、閉じても残す必要があるものはファクトリーの外で作ってハンドルを capture してください。dialog の content と同じ規則です。

## ウィンドウの閉じる操作とタイトルバー

タイトルバーの閉じるボタンと Alt+F4 は、ウィンドウに閉じる要求をします。`on_dismiss` があれば `DismissReason::WindowClose` で呼ばれます。`()` か `Handled::No` を返すと閉じます。`open` が false になり、content が unmount されます。`Handled::Yes` と `Handled::PreventDefault` は開いたままにし、エラーを返したハンドラーも開いたままにします（エラーは報告されます）。ウィンドウは既定で自分で閉じます。dialog と同じで、overlay とは違います。

```rust,test_harness
use kivact::prelude::*;
use kivact::view::DismissReason;

fn app() -> impl IntoView {
    let open = signal(false);
    let dirty = signal(true);
    column((
        button("設定").on_press(move || open.set(true)),
        window("設定", open)
            .content(|| text("設定はここに表示されます"))
            .on_dismiss(move |reason| match reason {
                DismissReason::WindowClose if dirty.get() => Handled::PreventDefault,
                _ => Handled::No,
            }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Preferences").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prevented_dismiss_keeps_the_window_open() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "設定").click();

        let settings = screen.window("設定");
        settings.request_close();
        assert!(settings.is_open());
    }
}
```

`open` を自分で false にしたときは `on_dismiss` は呼ばれません。呼ばれるのは利用者の閉じる要求です。main window の閉じる操作は今までどおりアプリを終了し、この変更ではそれを取り消す方法を追加しません。

## ウィンドウのモードと配置

`.mode(..)` と `.placement(..)` は構造の option です。view を組み立てるときに決まる定数で、後から変わる binding ではありません。

| `WindowMode` | 何か |
| --- | --- |
| `Normal`（既定） | タスクバーに自分の項目を持つ普通のウィンドウ |
| `Tool` | 親ウィンドウの上に留まり、タスクバーに項目を持たないユーティリティウィンドウ |
| `Modal` | 開いている間、親ウィンドウへのポインター、キーボード、支援技術の入力を受け付けないウィンドウ |

| `WindowPlacement` | どこに開くか |
| --- | --- |
| `Auto`（既定） | プラットフォームが決める場所。main window はプライマリーモニターの作業領域の中央 |
| `CenterOnParent` | 宣言したウィンドウの中央 |
| `NextToParent` | 親ウィンドウの右。右に余地がなければ左、どちらにも余地がなければ `Auto` |

`.size(width, height)` はウィンドウを開くときの論理サイズ（既定 640 × 480）で、`.min_size(..)` は利用者が縮められる最小サイズです。どちらも開く瞬間の値で、開いた後は利用者が大きさを変えます。view を mount するときに `App::window_size` と同じ規則（論理ピクセルで 1 以上、有限）で一度だけ確かめます。不正なサイズは、宣言した owner に `ErrorKind::Message` として報告され、ウィンドウは開きません。`open` は変更されません。

Wayland では `platform::capabilities().get().window_placement` が false です。プラットフォームは `Auto` 以外の配置を無視します。ウィンドウは開きますが、指定した場所には置かれません。

## context は宣言した場所に従う（React の Portal と同じ）

content のファクトリーは、ウィンドウを宣言した owner の下で動きます。そのため、宣言した場所で使える context はすべてウィンドウの中でも使えます。`provide` の値、テーマ、ロケール、文言、文字の向き、エラーハンドラーです。React の `createPortal` と同じで、子は別の場所に描かれますが、書いた場所の context を見ます。

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Draft {
    dirty: Signal<bool>,
}

fn form() -> impl IntoView {
    let draft = use_context::<Draft>();
    text(move || if draft.get().dirty.get() { "未保存の変更があります" } else { "保存済み" })
}

fn app() -> impl IntoView {
    let settings_open = signal(false);
    let dirty = signal(true);
    provide(Draft { dirty }, || {
        column((
            button("設定").on_press(move || settings_open.set(true)),
            window("設定", settings_open).content(|| form()),
        ))
    })
}

fn main() -> kivact::Result<()> {
    App::new("Editor").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_reads_the_context_of_its_declaration() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "設定").click();
        screen.window("設定").get_by_text("未保存の変更があります");
    }
}
```

`form` はウィンドウのファクトリーの中で組み立てられるので、`window(..)` がツリーの別の場所にあっても `Draft` を読めます。ファクトリーの中で作った signal は、開くたびに最初からやり直しになります。外で作って capture した signal は、閉じても残ります。

## ウィンドウを前面に出す

`platform::focus_window()` は、呼び出した owner のウィンドウを前面に出し、キーボードのフォーカスを与えます。前面に出したいウィンドウの中から呼びます。

```rust,no_run
use kivact::platform;
use kivact::prelude::*;

fn settings() -> impl IntoView {
    button("このウィンドウを前面に出す")
        .on_press(|| {
            spawn(platform::focus_window());
        })
}
```

- 最善の努力です。Windows はタスクバーの項目を点滅させるだけで、ウィンドウを前面に出さないことがあります。Wayland にはウィンドウを前面に出す方法がありません。capability は要りません。埋め込みのエディターでは `ErrorKind::Unsupported` で失敗します。
- すでに開いているウィンドウの `open` に `true` を書き直しても何も起きません。前面には出ません。別のウィンドウの操作から前面に出すには、signal でそのウィンドウに伝え、その content の中で作ったハンドラーか callback から `focus_window` を呼びます（次の節を参照）。

## 現在のウィンドウを問い合わせる

`platform` の問い合わせとサービスは、呼び出した owner のウィンドウを対象にします。`window_size`、`scale_factor`、`is_fullscreen`、`safe_area`、`keyboard_inset`、`theme_hint`、`is_surface_active`、`is_motion_reduced`、`pointer_settings` と、`request_window_size`、`set_fullscreen`、`focus_window`、ファイルダイアログ（ダイアログの親）です。ハンドラーと callback は、実行した場所ではなく作った場所のウィンドウに属します。ウィンドウの中で使う callback は、そのウィンドウの content の中で作ってください。

間違い: callback を main window で作っているので、それが開くダイアログの親は main window になります。

```rust,no_run
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn app() -> impl IntoView {
    let open = signal(false);
    let choose: Callback<()> = (|| {
        spawn(async { let _ = platform::pick_file(FileDialog::default()).await; });
    })
    .into();
    column((
        button("設定").on_press(move || open.set(true)),
        window("設定", open)
            .content(move || button("ファイルを選ぶ…").on_press(choose)),
    ))
}
```

正しい: callback を設定の content の中で作っているので、ダイアログの親は設定ウィンドウになります。

```rust,no_run
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn app() -> impl IntoView {
    let open = signal(false);
    column((
        button("設定").on_press(move || open.set(true)),
        window("設定", open).content(|| {
            button("ファイルを選ぶ…").on_press(|| {
                spawn(async { let _ = platform::pick_file(FileDialog::default()).await; });
            })
        }),
    ))
}
```

`request_window_size` と `set_fullscreen` も同じです。大きさやフルスクリーンを変えたいウィンドウの中で呼んでください。フォーカスもウィンドウごとにあります。`NodeRef::focus()` は、その node のウィンドウの中でフォーカスを移します。

## ウィンドウをテストする

テストでは、ウィンドウはそれぞれ独立した `Screen` です。`screen.window("設定")` はそのウィンドウが開くまで待ち（`find_by_*` と同じ時間上限）、ハンドルを返します。`screen.windows()` は開いているすべてのウィンドウを id の昇順で返します。main window が最初です。

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};
use kivact::ErrorKind;

fn app() -> impl IntoView {
    let settings_open = signal(false);
    column((
        button("設定").on_press(move || settings_open.set(true)),
        window("設定", settings_open).content(|| text("設定はここに表示されます")),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Editor").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_is_a_screen_of_its_own() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "設定").click();

        let settings = screen.window("設定");
        settings.get_by_text("設定はここに表示されます");
        assert!(settings.is_open());

        settings.request_close();
        assert!(!settings.is_open());
    }

    #[test]
    fn windows_lists_the_open_windows() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "設定").click();

        let titles: Vec<String> = screen.windows().iter().map(|window| window.title()).collect();
        assert_eq!(titles, ["Editor", "設定"]);
    }

    #[test]
    fn an_environment_with_one_window_reports_unsupported() {
        let screen = render_with(RenderOptions::single_window(), app);
        screen.get_by_role(Role::Button, "設定").click();

        assert!(screen.query_by_text("設定はここに表示されます").is_none());
        assert!(screen.errors().iter().any(|error| *error.kind() == ErrorKind::Unsupported));
    }
}
```

規則:

- `screen.window(..)` の戻り値は一時値です。`Element` を取る前にハンドルを `let` に入れてください。`let save = screen.window("設定").get_by_role(..);` はコンパイルできません（E0716）。ハンドルがウィンドウを生かします。
- `is_open()` はウィンドウがまだ開いているかを返します。`request_close()` はタイトルバーと同じ閉じる要求なので、`on_dismiss` が実行されます。閉じたハンドルでウィンドウ向けの method を呼ぶと、「window <id> is closed」で失敗します。
- 同じ title のウィンドウが 2 つ開いていると、`window(title)` は曖昧な `get_by_*` と同じように失敗します。
- `RenderOptions::single_window()` は、追加のウィンドウを開けない環境を表します。どの `window` も `ErrorKind::Unsupported` を報告して開きません。ウィンドウを開けない経路をテストするときに使います。

## ウィンドウが使えない環境

`platform::capabilities().get().multi_window` は、最初のウィンドウより多くを開ける環境かを表します。埋め込みのエディター（embed）とモバイルでは false です。false の環境で `open` が true になると、Kivact は宣言した owner に `ErrorKind::Unsupported` のエラーを報告し（宣言ごとに一度）、ウィンドウを開きません。binding は変更しません。

埋め込みのエディターでは、`platform::focus_window()` も `ErrorKind::Unsupported` で失敗します。

## 関連項目

- [ダイアログ、メニュー、ツールチップ、トーストを表示する](overlays.md): 1 つのウィンドウの中の overlay と、ウィンドウが dialog と共有する閉じる規則。
- [アプリをテストする](testing.md): 検索、操作、時間、失敗時の成果物。
- [プラットフォームのサービスを使う](platform-services.md): 上で挙げた問い合わせとサービス。
- API リファレンス: `cargo doc -p kivact --open` で `kivact::view` と `kivact::platform`。
