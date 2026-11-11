# アプリをテストする

[English](../../en/how-to-guide/testing.md)

`kivact::test` を使うと、人が使うのと同じやり方で Kivact アプリをテストできます。アプリを描画し、スクリーンリーダーが読み上げる内容で要素を探し、クリックや入力をして、画面に出ているものを確かめます。テストは `cargo test` で App 全体を `arvane-virtual` の上で動かします。画面を保持する仮想ホストが実際のレイアウト・描画・入力を処理し、手元ではその結果を実ウィンドウへ写すこともできます。ブラウザーもドライバーも要らず、実時間で待つこともありません。

このガイドは入門チュートリアルより踏み込み、あらゆる種類の検索と操作、状態とアクセシビリティの検証、時間、スクリーンショット、失敗時の成果物、コンポーネント単体とアプリ全体のテスト、CI での実行を扱います。

## 準備する

`test` feature はテストでだけ有効にします。リリースビルドに含めないためです。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

テストは、コードの隣の `#[cfg(test)] mod tests` か、`tests/` に書きます。`kivact::test::render(root)` はコンポーネントを描画して `Screen` を返します。

```rust,test_harness
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("回数: {}", count.get())),
        button("1 増やす").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_the_button_adds_one() {
        let screen = kivact::test::render(counter);
        screen.get_by_text("回数: 0");
        screen.get_by_role(Role::Button, "1 増やす").click();
        screen.get_by_text("回数: 1");
        assert!(screen.errors().is_empty());
    }
}
```

`render` が実際に行うこと: コンポーネントを、本物のアプリのホスト上で実行します。レイアウト、文字の組版、描画、アクセシビリティもすべて本物で、大きさは論理ピクセルで 800 × 600 です。入力はウィンドウが届けるのと同じ形で届き、時計は仮想です。各操作の後には整定の手順が走るため、次の行が実行されるときにはツリーが最新になっています。設定するモックはありません。

## 要素を探す

要素の探し方は 3 通りあり、それぞれに 4 つの形があります。

| 探し方 | 例 | 一致するもの |
|---|---|---|
| ロールと名前 | `get_by_role(Role::Button, "保存")` | 支援技術に公開される内容。アクセシブルな名前が完全に一致するもの |
| テキスト | `get_by_text("回数: 1")` | 内容がまったく同じ `text(..)` ノード |
| テスト ID | `get_by_test_id("total")` | `.test_id("total")` を付けたノード |

| 形 | 一致するものがないとき | 複数一致したとき | UI を待つか |
|---|---|---|---|
| `get_by_*` | テストを失敗させる | テストを失敗させる（曖昧） | 待たない |
| `query_by_*` | `None` を返す | `None` を返す | 待たない |
| `find_by_*` | 待ってから、仮想時間で 5 秒後に失敗 | 待ってから、5 秒後に失敗 | 待つ |
| `all_by_*` | 空の `Vec` を返す | すべてを返す | 待たない |

**ロールと名前**を優先してください。スクリーンリーダーの利用者が行うのと同じ検索なので、テストが通れば、その要素がアクセシブルであることの証明にもなります。ラベルのないアイコンボタンはこの方法では見つからず、それはテストではなくアプリの不具合です。単なる文字には `get_by_text` を使い、`test_id` は、描画したグラフのように、ロールもテキストもないものにだけ使います。

同じラベルが複数あるときは、`within(element)` で画面の一部の中だけを探します。

```rust,no_run
# use kivact::prelude::*;
# fn demo(screen: &kivact::test::Screen) {
let row = screen.get_by_test_id("row-2");
screen.within(row).get_by_role(Role::Button, "削除").click();
# }
```

存在しないことの確認には `query_by_*` を使います: `assert!(screen.query_by_text("保存しました").is_none())`。

## 利用者のように操作する

`Locator` への操作は、人が行うことをそのまま行います。ポインター操作の前に、人が使える状態の要素かどうかを確かめ、そうでなければ理由を示して失敗します。要素はマウントされていて、見えていて、大きさがあり、有効で、ウィンドウの中にあり、その中心でポインターが実際に当たるのがその要素自身である（ほかの要素やモーダルの背景に覆われていない）必要があります。

| 目的 | 呼び出し |
|---|---|
| クリック | `click()`、`double_click()`、`click_with(MouseButton::Secondary)` |
| タッチ・ペン | `tap()`、`tap_with(PointerKind::Pen)` |
| ホバー | `hover()` |
| 入力欄へ入力 | `type_text("hello")`（先にフォーカスします。`\n` は Enter） |
| キーを押す | `press_key(Key::Enter)`、`press_key_with(Key::Char('a'), Modifiers { ctrl: true, ..Default::default() })` |
| フォーカスを移す | `focus()`、または `screen.press_tab(false)`（Shift+Tab は `true`） |
| チェックボックス・スイッチを切り替える | `toggle()` |
| セレクトの選択肢を選ぶ | `select_option("大")` |
| ドラッグ | `drag(from, to)`（要素自身の座標） |
| スクロール | `scroll(dx, dy)` |
| 支援技術 | `a11y(A11yAction::…)`、`a11y_set_value("text")` |
| IME の変換 | `compose("preedit")`、`commit("text")` |

何も起きないことを確かめるためのクリックや、ポップアップの外側を押す操作では、`force_click()` か、画面レベルの `click_at(x, y)`・`mouse_move`・`mouse_down`・`mouse_up` で検査を省けます。ポインターを先に置いてください（`mouse_move`、またはクリックやホバー）。置く前の `mouse_down`・`mouse_up`・`mouse_leave` は、実際の window と同じく押す位置がないため panic します。`contact_down/move/up` は、複数のタッチを同時に動かします。`Key`・`Modifiers`・`MouseButton`・`PointerKind` は `kivact::view` にあります（`Key` はプレリュードにもあります）。

```rust,test_harness
use kivact::prelude::*;

fn todo() -> impl IntoView {
    let draft = signal(String::new());
    let tasks = signal(Vec::<String>::new());
    column((
        text_input("新しいタスク", draft).on_submit(move || {
            let task = draft.get();
            if !task.trim().is_empty() {
                tasks.update(|tasks| tasks.push(task));
                draft.set(String::new());
            }
        }),
        dynamic(move || tasks.get(), |tasks| {
            column(tasks.into_iter().map(text).collect::<Vec<_>>())
        }),
        text(move || format!("{} 件のタスク", tasks.get().len())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Todo").run(todo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_adds_a_task_and_clears_the_field() {
        let screen = kivact::test::render(todo);
        let input = screen.get_by_role(Role::TextBox, "新しいタスク");
        input.type_text("牛乳を買う");
        input.press_key(Key::Enter);

        screen.get_by_text("牛乳を買う");
        screen.get_by_text("1 件のタスク");
        assert_eq!(input.value().as_deref(), Some(""));
    }
}
```

## 利用者に見えるものを確かめる

`Locator` に尋ねられるのは、スクリーンリーダーが受け取るのと同じ公開された状態であり、内部の変数ではありません。

| 知りたいこと | 呼び出し |
|---|---|
| テキスト、名前、説明 | `text()`、`name()`、`description()` |
| ロール | `role()` |
| 入力値、スライダーの範囲 | `value()`、`value_range()` |
| 状態 | `is_disabled()`、`checked()`（`Option<CheckedState>`: チェックできないものは `None`）、`is_selected()`、`is_expanded()`、`is_focused()`、`invalid()`、`read_only()` |
| 集合の中の位置 | `level()`、`set_size()`、`pos_in_set()` |
| 位置と大きさ | `rect()`、`accessibility_bounds()` |
| 表示 | `exists()`、`is_visible()` |
| 関係 | `labelled_by()`、`described_by()`、`controls()` |

画面全体については、`screen.focused()`、`screen.hovered()`、`screen.cursor()`、`screen.title()` があります。`screen.errors()` は、アプリの既定のエラー出力先に届いたエラー、つまり `on_error` や `error_boundary` が受け取らなかったものを返します。テストの最後にこれが空であることを確かめると、多くの問題に気づけます。

```rust,test_harness
use kivact::prelude::*;

fn terms() -> impl IntoView {
    let accepted = signal(false);
    column((
        checkbox("利用規約に同意する", accepted),
        button("続ける").disabled(move || !accepted.get()),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Terms").run(terms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_needs_the_terms() {
        let screen = kivact::test::render(terms);
        let proceed = screen.get_by_role(Role::Button, "続ける");
        assert!(proceed.is_disabled());

        let terms = screen.get_by_role(Role::CheckBox, "利用規約に同意する");
        assert_eq!(terms.checked(), Some(CheckedState::Unchecked));
        terms.toggle();

        assert_eq!(terms.checked(), Some(CheckedState::Checked));
        assert!(!proceed.is_disabled());
        screen.assert_layout();
    }
}
```

`screen.assert_layout()` はレイアウトも検査します。たとえば、枠からあふれた文字を見つけます。見つけた違反をすべて示して失敗し、そのときのフレームを残します。`check_layout()` は違反を値として返すので、自分で検証することもできます。意図的に切り取っているものは、`assert_layout_with(&LayoutCheck::default().ignore_test_id("clipped"))` で除外できます。

アプリのコード (ハンドラー、effect、memo、setup 関数、タスク、`blocking` のジョブ、Kivact が検出した誤用) でのパニックはバグであり、テストライブラリは捕まえません。パニックはターンを進めた `Screen` の呼び出しから元のメッセージのまま出てきて、失敗時の成果物が書き出され、テストは失敗します。Kivact が誤用を検出することをテストするには、パニックを期待します。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    button("Misuse").on_press(|| {
        // ハンドラーの中の setup 関数はバグです。
        let _late = signal(0);
    })
}

fn main() -> kivact::Result<()> {
    App::new("Misuse").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "SetupOnly")]
    fn a_setup_function_in_a_handler_panics() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Misuse").click();
    }
}
```

## コンポーネント単体とアプリ全体をテストする

コンポーネントは関数なので、必要な引数を渡して、それだけを描画してテストできます。コンポーネントが使うシグナルをテストが読んだり変えたりしたいときは、それをセットアップの中で作り、`render_with_handle` で受け取ります。

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with_handle};

/// 自分の持つ数値を挟む「減らす」「増やす」のペア。
fn stepper(value: Signal<i32>) -> impl IntoView {
    row((
        button("減らす").on_press(move || value.update(|n| *n -= 1)),
        text(move || value.get().to_string()),
        button("増やす").on_press(move || value.update(|n| *n += 1)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Stepper").run(|| stepper(signal(0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_edits_the_signal_it_is_given() {
        let (screen, value) = render_with_handle(RenderOptions::default(), || {
            let value = signal(5);
            (value, stepper(value))
        });
        screen.get_by_role(Role::Button, "増やす").click();
        assert_eq!(value.get_untracked(), 6);

        // 逆向き: テストが書き込むと、画面は整定したあとに追従します。
        value.set(1);
        screen.run_until_idle();
        screen.get_by_text("1");
    }
}
```

アプリ全体をテストするには、`main` と同じように、ルート関数を描画します: `render(app)`。コンポーネントをまたぐ流れ（ログイン、作成、検索、削除）は、アプリ全体でテストします。コンポーネントの状態、キーボードの挙動、端のケースは、準備も読み取りも簡単な単体でテストします。

コンポーネントが、普段はアプリが与えるコンテキスト（テーマ、i18n のメッセージ、ストア）を必要とする場合は、`main` と同じ方法で、`render` に渡すクロージャの中で提供します。あるいは、`RenderOptions` に `theme`・`locale`・`messages` を渡します。

画面を描画するテストバイナリで `App::run` を呼んではいけません。テストのウィンドウはすべて、プロセスにつき 1 つのイベントループを共有するからです。`main` をテストに含めないでください。テストの入口は `render` です。

## ウィンドウをテストする

ウィンドウは、それぞれが独立した `Screen` です。`screen.window("設定")` はそのウィンドウが開くまで待ち（`find_by_*` と同じ時間上限）、ハンドルを返します。`screen.windows()` は開いているすべてのウィンドウを id の昇順で返します。`Screen::has_window()` は `is_mirrored()` に改名しました（公開の変更。この window が本物の window に写っているか、という意味は変わりません）。`RenderOptions::default().single_window()` は、埋め込みの editor のように、追加の window を開けない環境を表します。`Locator` を取る前に、ハンドルを `let` に入れてください。

```rust,no_run
# use kivact::prelude::*;
# fn demo(screen: &kivact::test::Screen) {
let settings = screen.window("設定");
settings.get_by_role(Role::Button, "適用").click();
assert!(settings.is_open());

settings.request_close();
assert!(!settings.is_open());
# }
```

`screen.window(..)` の戻り値は一時値なので、`let apply = screen.window("設定").get_by_role(..);` はコンパイルできません（E0716）。ハンドルがウィンドウを生かします。同じ title のウィンドウが 2 つ開いていると、`window` は曖昧な `get_by_*` と同じように失敗します。詳しくは[複数のウィンドウを開く](windows.md#ウィンドウをテストする)を参照してください。

## 時間と非同期処理

テストの時間は仮想です。タイマーは壁掛け時計ではなく `advance` を待つので、30 秒のタイムアウトも一瞬で済み、結果がマシンの速さに左右されることはありません。

| 呼び出し | 動き |
|---|---|
| `screen.run_until_idle()` | 今すぐ実行できるものをすべて実行し、`blocking` で始めたジョブを（実時間で最大 5 秒）待ちます |
| `screen.advance(duration)` | 仮想時計を進め、途中のタイマーとフレームのたびに止まって実行します |
| `screen.wait_for(timeout, \|s\| condition)` | 条件が成り立つまで 1 フレームずつ進めます。仮想の `timeout` が過ぎると失敗します |
| `screen.find_by_text("…")` | `get_by_text` と同じですが、要素が現れるまで仮想時間で最大 5 秒待ちます |
| `screen.now()` | 画面を開始してからの仮想時間 |

`click` などの操作はすでに整定するので、`run_until_idle` を呼ぶことはあまりありません。タイマーを始めた後には `advance` を呼びます。

```rust,test_harness
use std::time::Duration;
use kivact::prelude::*;
use kivact::task::sleep;

fn saver() -> impl IntoView {
    let shown = signal(false);
    column((
        button("保存").on_press(move || {
            spawn(async move {
                shown.set(true);
                sleep(Duration::from_secs(2)).await;
                shown.set(false);
            });
        }),
        show(shown, || text("保存しました")),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Saver").run(saver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_disappears_after_two_seconds() {
        let screen = kivact::test::render(saver);
        screen.get_by_role(Role::Button, "保存").click();
        screen.get_by_text("保存しました");

        screen.advance(Duration::from_millis(1900));
        assert!(screen.query_by_text("保存しました").is_some());
        screen.advance(Duration::from_millis(200));
        assert!(screen.query_by_text("保存しました").is_none());
    }

    #[test]
    fn waiting_for_a_condition() {
        let screen = kivact::test::render(saver);
        screen.get_by_role(Role::Button, "保存").click();
        let waited = screen.wait_for(Duration::from_secs(5), |s| s.query_by_text("保存しました").is_none());
        assert!(waited >= Duration::from_secs(2));
    }
}
```

`blocking` で始めた処理は実際のスレッドで動くので実時間がかかりますが、テストは自動的にその完了を（最大 5 秒）待ってから先へ進みます。フレームループ、アニメーション、トランジションも同じ仮想時計で動きます。[値と要素をアニメーションさせる](animation.md#アニメーションをテストする) を参照してください。

## 環境を設定する

`render_with(options, root)` は、`RenderOptions` を指定して画面を開始します。すべての項目に既定値があるので、そのテストに関係するものだけを設定します。

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

fn size_label() -> impl IntoView {
    text(|| {
        let size = kivact::platform::window_size().get();
        format!("{} x {}", size.width, size.height)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Size").run(size_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phone_sized_window_at_double_density() {
        let options = RenderOptions::default().window_size(390.0, 844.0).scale(2.0);
        let screen = render_with(options, size_label);
        screen.get_by_text("390 x 844");
        assert_eq!(screen.screenshot().width, 780); // 物理ピクセル

        screen.resize(800.0, 600.0);
        screen.get_by_text("800 x 600");
    }
}
```

`RenderOptions` は builder です。大きさは `.window_size(width, height)`、倍率は `.scale(value)`、テーマは `.theme(theme)`、言語と文言は `.locale(locale)` と `.messages(messages)` で指定します。システムの設定には `.host_locale(tag)`、`.dark()`、`.reduced_motion()`、フォントには `.fonts(...)`、描画には `.gpu()`、写しには `.mirror(MirrorMode)` を使います。`.frame_period_ns(u64)` はディスプレイの更新間隔（既定は 60 Hz）、`.title(...)` は写しのウィンドウ名です。公開 field の struct literal で組み立てる必要はありません。

画面が動いている間は、テストがシステムの役を演じられます: `resize`・`set_scale`・`set_dark`・`set_reduced_motion`・`set_host_locale`・`set_safe_area`・`set_window_focused`・`suspend` と `resume`・`memory_warning`・`lose_surface` と `restore_surface`・`request_close`。プラットフォームのサービスもスクリプト化されています: `clipboard` と `set_clipboard`・`respond_file_dialog`・`respond_files_dialog`・`opened_paths`・`revealed_paths`・`fail_next_platform_request`。[プラットフォームのサービスを使う](platform-services.md) を参照してください。

文字は、既定では**同梱フォント**（Tuffy）で組まれるので、レイアウトと画素はどのマシンでも同じになります。このフォントには日本語などの CJK の字形がありません。そのような文字はどのマシンでも同じ幅を取りますが、空の四角として描かれます。実際の字形でテストするには、同梱するフォントのファイルを使って `.fonts(TestFonts::Only(vec![Asset::from(bytes)]))` を指定するか、`TestFonts::System` でマシンのフォントを使います（その場合、レイアウトはマシンに依存します）。

## スクリーンショット

`screen.screenshot()` は、最後のフレームを `Screenshot` として返します。`width` と `height` は物理ピクセル、`pixels` は `0x00RRGGBB` の値です。`pixel(x, y)` で 1 画素を読み、`crop(x, y, w, h)` で一部を切り出し、`hash()` でフレームを要約し、`save_png(path)` でファイルに書き出します。`element.screenshot()` は、1 つの要素の下にある部分です。

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

fn badge() -> impl IntoView {
    column(text("新着")).style(style().padding(8).background(rgb(0x2050d0)))
}

fn main() -> kivact::Result<()> {
    App::new("Badge").run(badge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_badge_is_drawn_in_blue() {
        let options = RenderOptions::default().window_size(200.0, 100.0);
        let screen = render_with(options, badge);
        let shot = screen.screenshot();
        assert_eq!((shot.width, shot.height), (200, 100));
        // 余白の中の角は背景色です。
        assert_eq!(shot.pixel(2, 2), Some(0x2050d0));
    }
}
```

既定ではフレームを CPU が描くので、マシンが違ってもビット単位で同じになり、画素や `hash()` の比較が安定します。スクリーンショットは描画した内容（キャンバス、色、グラデーション）に使い、ロールやテキストを持つものは検索で確かめてください。ウィンドウ全体のスクリーンショットを保存済みの画像と比べることを主な検証にするのはやめてください。意図した変更のたびに壊れてしまいます。

`RenderOptions` で `.gpu()` を指定するか（`RenderOptions::from_env()` なら環境変数 `KIVACT_TEST_GPU=1`）、GPU で描いて画素を読み戻せます。GPU のテストが始められない理由は `kivact::test::gpu_unavailable()` が返します。始められるなら `None` です。

## テストが失敗したら

検索や操作が失敗すると、探していたもの、見つかった候補すべて、フォーカス、直近の操作、エラー、アクセシビリティツリーと意味のツリーが示されます。失敗したテストは 2 つのファイルも残します。

- `<テスト名>.png`: 最後のフレーム。利用者が見たはずの画面です。
- `<テスト名>.txt`: ウィンドウが公開したアクセシビリティツリー（ロール、名前、値、状態、テスト ID、位置と大きさを、1 ノード 1 行で）、アプリの意味のツリー、直近の操作、仮想時間。

保存先は、Cargo のターゲットディレクトリ内の `kivact-test-failures` フォルダーです（`CARGO_TARGET_DIR` があればそこ、なければテストバイナリがあるターゲットフォルダーで、通常は `target/kivact-test-failures/`）。テスト名の記号は `_` に置き換えられ、同じテスト内の 2 つ目の画面には `-2` が付きます。パニックのメッセージと標準エラー出力にファイル名が出ます。`kivact::test::failure_dir()` がそのフォルダーを返します。

成功するテストで同じ情報を見たいときは、`screen.accessibility_tree()`（スクリーンリーダーに見えるもの）や `screen.debug()`（保持しているツリー）を出力するか、`screen.screenshot().save_png("frame.png")` で保存します。各ステップで要求されたフレームを描きます。`KIVACT_TEST_HOLD_MS=3000` は、テストの終了時にウィンドウを 3 秒間開いたままにします。

## シェーダーを GPU と CPU の両方でテストする

シェーダーは 2 つのレーンで描かれ、テストは両方を対象にできます。**CPU のレーン**（既定）では、シェーダーは実行されず、CPU が `ShaderPaint::fallback` で与えた paint を描きます。そのため絵は代わりの塗りになりますが、テストは実際の画面を描いて要素を探せます。**GPU のレーン**では、ウィンドウが WGSL をグラフィックスカードで実行し、絵はシェーダーの出力になります。GPU のレーンには、グラフィックスアダプターとネイティブのウィンドウが必要です。どちらかが無い環境では、テストは飛ばされます（[CI で実行する](#ci-で実行する)を参照）。

GPU のテストは、`RenderOptions::from_env().gpu()` でレーンを要求し、`.mirror(MirrorMode::Required)` でネイティブのウィンドウを必須にし、`gpu_unavailable()` が理由を返したら早く戻ります。`from_env()` は `KIVACT_TEST_GPU=1` も読むので、`.gpu()` を省いたテストは、CI のジョブがその変数を設定した、グラフィックスカードのあるマシンでだけ GPU のレーンで動きます。飛ばされた GPU のテストは、シェーダーをコンパイルしません。そのため、下の例のように、どのマシンでも WGSL をコンパイルする CPU のテストも一緒に書いてください。

```rust,test_harness
use kivact::draw::PaintShader;
use kivact::geometry::Rect;
use kivact::prelude::*;

const WAVE: &str = r#"
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
"#;

fn wave() -> impl IntoView {
    let shader = PaintShader::wgsl(WAVE).expect("the shader is valid WGSL");
    canvas(move |draw, size| {
        // CPU のレーンでは、シェーダーの代わりに単色の fallback を描く。
        let paint = shader
            .paint()
            .param("speed", 2.0)
            .param("tint", rgb(0x3b82f6))
            .fallback(rgb(0x1e3a8a));
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
    })
    .style(style().size(120, 60))
    .test_id("wave")
}

fn main() -> kivact::Result<()> {
    App::new("Wave").run(wave)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{MirrorMode, RenderOptions, gpu_unavailable, render, render_with};

    #[test]
    fn the_wave_renders_on_the_cpu_lane() {
        let screen = render(wave);
        screen.get_by_test_id("wave");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn the_wave_renders_on_the_gpu_lane() {
        if let Some(reason) = gpu_unavailable() {
            eprintln!("skipped: {reason}");
            return;
        }
        let options = RenderOptions::from_env().gpu().mirror(MirrorMode::Required);
        let screen = render_with(options, wave);
        screen.get_by_test_id("wave");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

両方のレーンで画面は同じなので、構造や文字を確かめるテストはレーンを選びません。画素を比べるテストは、どちらのレーンで動くかを明示してください。2 つのレーンは違う絵を描くためです。

## CI で実行する

テストは通常の `cargo test` で動きます。CPU の検証にはデスクトップが不要です。実ウィンドウへの写しを使うときだけ、ウィンドウを開ける環境を用意します。

- **デスクトップセッションがあれば、実ウィンドウに写せます。** `RenderOptions::default()` と `RenderOptions::from_env()` は `KIVACT_TEST_MIRROR` を読みます。`off` はフレームを保持するだけ、`auto`（既定）は開ける場合に写しを開き、`required` は開けなければ失敗します。`.mirror(MirrorMode::Off)`・`.mirror(MirrorMode::Auto)`・`.mirror(MirrorMode::Required)` で明示的に選べます。`RenderOptions::from_env()` は `KIVACT_TEST_GPU=1` も読みます。GPU の提示には実ウィンドウの写しが必要です。CPU の画素はどのモードでも同じです。
- **ディスプレイのない CI マシン**では、既定のオプションがヘッドレスに切り替わり、ここまでの内容はすべて動きます。そこで実ウィンドウを使いたい場合（ウィンドウ周りの経路全体を試すため）は、仮想ディスプレイの中（たとえば Linux の `xvfb-run`）か、デスクトップセッションのあるランナーでジョブを実行します。
- **同じデスクトップで、複数のプロセスからウィンドウのテストを同時に動かさないでください。** 並列のテストプロセスのウィンドウはフォーカスを奪い合い、固まることがあります。テストバイナリは順番に実行します。
- **プロセスにつきイベントループは 1 つ**: 画面を描画するテストバイナリで `App::run` を呼んではいけません。
- テストが `blocking` のジョブを待つのは、実時間で最大 5 秒です。それ以外はすべて仮想時計で動くので、テストの中で sleep してはいけません。

## 関連項目

- [コンポーネントから非同期処理を実行する](async-and-tasks.md): 仮想時計が駆動するもの。
- [値と要素をアニメーションさせる](animation.md): トランジションと動きを減らす設定のテスト。
- [プラットフォームのサービスを使う](platform-services.md): スクリプト化されたクリップボード・ダイアログ・パス。
- [キャンバスで描く](canvas.md): 描いたリージョンをロールと名前で探します。
- API リファレンス: `cargo doc -p kivact --features test --open` を実行し、`kivact::test` を開いてください。
