# 4. 入力とイベント

入力欄に書いた文字を画面へ反映するには、`Signal<String>` を `kivact::ui::text_input(名前, signal)` に渡します。入力欄と状態が双方向につながります。

```rust,no_run
use kivact::prelude::*;

fn greeting() -> impl IntoView {
    let name = signal(String::new());
    let saved = signal(String::new());

    column((
        text_input("名前", name)
            .placeholder("名前を入力")
            .on_submit(move || saved.set(name.get())),
        button("決定").on_press(move || saved.set(name.get())),
        text(move || {
            if saved.get().is_empty() {
                "名前を入力してください".to_string()
            } else {
                format!("こんにちは、{}さん", saved.get())
            }
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("入力の例").run(greeting)
}
```

`text_input("名前", name)` の第1引数は入力欄のアクセシビリティ名です。`.placeholder(...)` は未入力時の補助表示です。`.on_submit(...)` は一行入力欄で Enter が押されたとき、`.on_press(...)` はボタンのクリック、Enter、Space などで呼ばれます。

ここでは `name` が編集中の値、`saved` が確定した値です。Rust の `String` は `get()` で複製されます。`Signal<String>` のハンドルは `Copy` なので、入力欄、Enter、ボタンの三箇所へ渡せます。

## 真偽値の入力

真偽値も同じ考え方です。`checkbox(名前, signal)` は `Signal<bool>` を編集するチェックボックスで、クリックと Space で切り替わります。次の例は、確認のチェックを付けるまでボタンを押せない画面と、その動作を確かめるテストです。

```rust,test_harness
use kivact::prelude::*;

fn agreement() -> impl IntoView {
    let agreed = signal(false);
    column((
        checkbox("内容を確認した", agreed),
        button("続ける").disabled(move || !agreed.get()),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("同意の例").run(agreement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checking_enables_the_button() {
        let screen = kivact::test::render(agreement);
        assert!(screen.get_by_role(Role::Button, "続ける").is_disabled());
        screen.get_by_role(Role::CheckBox, "内容を確認した").click();
        assert!(!screen.get_by_role(Role::Button, "続ける").is_disabled());
    }
}
```

`#[cfg(test)] mod tests` の中は `cargo test` のときだけコンパイルされるテストです。`cargo run` は通常のウィンドウ、`cargo test` はフォーカスを奪わない試験のウィンドウで操作します。テストの書き方は [画面のテスト](testing.md) で扱います。

入力に応じて属性を変える場合も、`.disabled(move || ...)` のように状態を読むクロージャを渡します。複数行なら `kivact::ui::text_area("説明", value)` を使います。IME 入力中の未確定文字は binding に書かれず、確定した編集だけが書かれます。変化に合わせて別の処理をするときは、`binding(読む, 書く)` を渡して `書く` の中で行います。

検証と保存は [フォームのガイド](../how-to-guide/forms.md)、独自の外観と native 編集は [部品作者ガイド](../../for-component-author/interaction.md)で扱います。kit の部品は [公式 UI ガイド](../../for-kivact-ui-user/README.md)にあります。

次は [条件分岐と一覧](conditional-lists.md) で、画面の子そのものを増減させます。

## 自分で動かして確認する

`cargo run -p kivact-example-form` でテキスト入力と検証メッセージの表示を確認します。上の greeting のコードを入門 package に置いたら、名前を入力するだけでは確定表示が変わらず、Enter または「決定」で確定することを確認してください。
