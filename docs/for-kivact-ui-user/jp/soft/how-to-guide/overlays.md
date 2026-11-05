# オーバーレイ

[English](../../../en/soft/how-to-guide/overlays.md)

短い補足にはツールチップ、ホバーやフォーカスで開くプレビューにはホバーカード、利用者の判断が必要な場合にはダイアログを使います。削除などの確認には、対象と操作が分かるタイトルとラベルを付けたアラートダイアログを使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn delete_confirmation(delete_file: impl Fn() + 'static) -> impl IntoView {
    let open = signal(false);

    (
        button("Delete file").on_press(move || open.set(true)),
        soft::alert_dialog("Delete file?", "This action is permanent.", open)
            .action("Delete", move || {
                delete_file();
                open.set(false);
            }),
    )
}
```

ダイアログは呼び出し側が渡した開閉用シグナルを読み取ります。確認コールバックでは操作の後にダイアログを閉じます。非同期操作の場合は成功後にシグナルを更新し、エラーは利用者が対処できる場所に表示します。

Soft の lightbox は dialog と zoom/pan の振る舞いを共有します。`use_lightbox(media)` が
選択画像と全体表示・拡大表示の状態を持ち、`.view().zoom(true)` で拡大操作を有効にします。
画像の失敗は画像 node の状態から読み取り、翻訳された代替表示を出します。bottom sheet は
高さ、scrim、purpose、focus、padding、snap point の option を維持し、離したときの速度と
snap の選択に共通の drag の振る舞いを使います。
