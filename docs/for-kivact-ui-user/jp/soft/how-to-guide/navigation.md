# ナビゲーション

[English](../../../en/soft/how-to-guide/navigation.md)

現在のビューはアプリの状態に保持します。関連する少数のビューには、選択キーをセグメントコントロールにバインドし、同じキーで表示内容を切り替えます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::ListItem;
use kivact::view::dynamic;
use kivact_ui_soft as soft;

fn project_view() -> impl IntoView {
    let view = signal(Some("summary"));
    column((
        soft::segmented_control(
            "Project view",
            vec![ListItem::new("summary", "Summary"), ListItem::new("activity", "Activity")],
            view,
        ),
        dynamic(view, |view| match view {
            Some("activity") => soft::heading(2, "Activity"),
            _ => soft::heading(2, "Summary"),
        }),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s4))
}
```

表示ラベルが変わっても選択を維持できるよう、`"summary"` などの安定したキーを使います。大きな画面の中で名前付きパネルを切り替える場合は、標準のタブが適しています。
