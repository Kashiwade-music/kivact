# 設定フォームを作る

[English](../../../en/soft/getting-started/02-build-the-settings-form.md)

表示名をシグナルに保持します。標準入力欄が値を読み書きし、Soft が見た目を設定します。

```rust,no_run
use kivact::prelude::*;
fn settings() -> impl IntoView {
    column((
        text_input("Display name", signal(String::new())),
        button("Save").variant(ButtonVariant::Primary),
    )).style(style().gap(12))
}
```
