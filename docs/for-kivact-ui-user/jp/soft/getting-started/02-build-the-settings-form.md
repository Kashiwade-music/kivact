# 設定フォームを作る

[English](../../../en/soft/getting-started/02-build-the-settings-form.md)

表示名をシグナルに保持します。標準入力欄はその値を読み書きします。保存ボタンは現在の値を取得し、渡された保存コールバックを呼び出します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn settings(save: impl Fn(String) + 'static) -> impl IntoView {
    let display_name = signal(String::new());

    column((
        soft::heading(1, "Profile"),
        text_input("Display name", display_name),
        button("Save")
            .variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || display_name.get().trim().is_empty())
            .on_press(move || save(display_name.get().trim().to_owned())),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
}
```

同じシグナルからボタンの有効状態を求めます。次のページでは、保存結果の表示と表示モードの切り替えを扱います。
