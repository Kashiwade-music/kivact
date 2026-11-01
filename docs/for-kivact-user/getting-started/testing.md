# 8. 画面のテスト

`kivact::test` は、Arvane host の上に画面を組み立て、支援技術へ渡す役割と名前で要素を探し、表示を確かめます。入力は host のテスト driver から送り、時計は仮想時計で進めます。OS のマウス・IME・スクリーンリーダーそのものを操作する検査とは別です。`kivact::test` を使うには、テスト用依存の `kivact` に `test` feature が必要です。

テストは `#[cfg(test)] mod tests` の中に `#[test]` 関数として書きます。以下のコードには、そのまま使えるテストが含まれています。

```rust,test_harness
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("回数: {}", count.get())),
        button("増やす").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("テストの例").run(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_increments_the_count() {
        let screen = kivact::test::render(counter);
        screen.get_by_text("回数: 0");
        screen.get_by_role(Role::Button, "増やす").click();
        screen.get_by_text("回数: 1");
        assert!(screen.errors().is_empty());
    }
}
```

`render` は部品を組み立てて `Screen` を返します。`get_by_role` は host が支援技術に出した役割とアクセシビリティ名で操作部品を探すため、利用者が認識する名前を使ったテストになります。`click` は要素の中心をマウスで押すのと同じで、要素が無効・非表示・ほかの要素の下にあるなどで押せないときは、理由を書いて失敗します。入力欄には `get_by_role(Role::TextBox, "名前").type_text("太郎")` のように入力できます（キーを1つずつ打つのと同じです）。

## setup で作った状態を確かめる

signal などを試験へ返すには `render_with_handle(options, setup)` を使います。setup は一度だけ実行され、画面と handle の組を返します。試験ごとに `Rc<RefCell<Option<_>>>` を用意する必要はありません。

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with_handle};

#[test]
fn the_button_updates_its_setup_state() {
    let (screen, count) = render_with_handle(RenderOptions::from_env(), || {
        let count = signal(0);
        (count, button("Increment").on_press(move || count.update(|n| *n += 1)))
    });
    screen.get_by_role(Role::Button, "Increment").click();
    assert_eq!(count.get_untracked(), 1);
    screen.assert_layout();
    assert!(screen.errors().is_empty());
}
```

`Screen::assert_layout()` は文字の overflow などを検出し、失敗時の画面と診断を保存します。意図した切り取りなどに既存の例外がある場合は、その `LayoutCheck` を `screen.assert_layout_with(&check)` に渡します。

## 待ち時間のある画面

テストの時計は仮想時計です。実際に 200 ミリ秒待つ代わりに、テストから時間を進めます。

```rust,test_harness
use std::time::Duration;
use kivact::prelude::*;

fn delayed() -> impl IntoView {
    let message = resource(
        || (),
        |_: ()| async move {
            kivact::task::sleep(Duration::from_millis(200)).await;
            Ok("読み込み完了".to_string())
        },
    );
    column((
        show(move || message.state().is_pending(), || text("読み込み中")),
        show_some(move || message.latest(), |message| text(message)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("待ち時間の例").run(delayed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advancing_the_clock_finishes_the_load() {
        let screen = kivact::test::render(delayed);
        screen.get_by_text("読み込み中");
        screen.advance(Duration::from_millis(200));
        screen.get_by_text("読み込み完了");
    }

    #[test]
    fn find_by_text_waits_for_the_text() {
        let screen = kivact::test::render(delayed);
        screen.find_by_text("読み込み完了");
    }
}
```

すぐ実行できる処理は `screen.run_until_idle()`、タイマーは `screen.advance(...)` で進めます。`Duration` を使うファイルでは `use std::time::Duration;` が必要です。`find_by_text(...)` は仮想時計を進めながら文字の出現を待ちます。待つ必要がない表示には `get_by_text(...)` を使います。

## ウィンドウとスクリーンショット

テストのウィンドウは、フォーカスを奪わずに開き、テストが終わると閉じます。ウィンドウを開けない環境（CI など）では、自動で画面なし（headless）になります。どちらでも描かれる画素は同じです。文字は同梱のフォント（Tuffy）だけで描くので、どの PC でも同じ配置と画素になります（日本語の字形は豆腐になりますが、配置は決まります）。

この repository の検査は実ウィンドウで実行します。`RenderOptions::from_env()` は、`Default` の他の設定を保ち、既定で実ウィンドウを必須にします。環境変数 `KIVACT_TEST_GPU` が正確に `1` のときだけ GPU を使います。`KIVACT_TEST_HEADLESS` が正確に `1` のときだけ画面なしになりますが、これは開発者の手元だけの opt-in です。試験・CI・検証 script では設定しません。環境変数を読まずに選ぶ場合は `RenderOptions::window`・`gpu` を指定します。GPU feature を有効にするだけでは GPU 描画になりません。OS 入力と IME はアプリを起動して別に確認します。

`screen.screenshot()` で最後の画面の画素を取り出せます。`save_png` で PNG に保存できます。

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

fn badge() -> impl IntoView {
    column(text("新着")).style(style().padding(8).background(rgb(0x2050d0)))
}

fn main() -> kivact::Result<()> {
    App::new("スクリーンショットの例").run(badge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_badge_is_drawn_in_blue() {
        let options = RenderOptions {
            width: 200.0,
            height: 100.0,
            ..RenderOptions::default()
        };
        let screen = render_with(options, badge);
        let shot = screen.screenshot();
        assert_eq!((shot.width, shot.height), (200, 100));
        // バッジの左上の角（余白の中）は背景色です。
        assert_eq!(shot.pixel(2, 2), Some(0x2050d0));
    }
}
```

テストが失敗すると、最後の画面（`.png`）と、支援技術に見える木と部品の木（`.txt`）が `target/kivact-test-failures/` にテスト名で残ります。失敗のメッセージにそのファイルの場所が書かれます。

テストのウィンドウを使うテストのバイナリでは `App::run` を呼べません（1つのプロセスでウィンドウのイベントループは1つだけです）。`main` を呼ばない `#[test]` から使う限り問題はありません。

## テストを実行する

```text
cargo test -p kivact-example-counter
cargo test -p kivact-docs --doc
```

前者は入門例のテストを実行します。後者はチュートリアル中の Rust コードブロックをコンパイルし、`#[cfg(test)] mod tests` を含む例はそのテストも実行します。ここまでできたら、[目次](README.md) から気になる章をもう一度試し、[サンプルアプリ](../../../examples/README.md) で実際の組み合わせを読んでみてください。


## 自分で動かして確認する

この章のコード全体を入門 package の `src/main.rs` へ置き、テスト用の `kivact` に `test` feature を設定して `cargo test -p kivact-example-counter` を実行します。期待する数字を意図的に変え、失敗メッセージと出力された画面を確認してから戻してください。エージェントに実行を頼む場合は、事前に実 window の試験として許可してください。
