# オーバーレイ

[English](../../../en/soft/how-to-guide/overlays.md)

ダイアログを表示状態のシグナルに結び付け、内容を表すタイトルを指定します。警告ダイアログは開いたときにキャンセル操作へフォーカスできます。実行する操作を渡し、コールバックで表示状態を更新します。

```rust,no_run
use kivact::prelude::*;
fn delete_confirmation() -> impl IntoView {
    let open = signal(false);
    (
        button("Delete file").on_press(move || open.set(true)),
        kivact_ui_soft::alert_dialog("Delete file?", "This action is permanent.", open)
            .action("Delete", move || open.set(false)),
    )
}
```
