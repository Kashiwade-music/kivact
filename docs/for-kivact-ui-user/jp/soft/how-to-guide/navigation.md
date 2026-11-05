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

サイズ変更できるパネルには、サイズ binding と `ScalarRange` を渡した `kivact::ui::splitter` を使います。
折りたたみは `.collapsible(割合)`、スナップ位置は `.snaps(割合の配列)`、上下の配置は `.orientation(..)`、
逆方向や比率のサイズ変更は `.pointer_scale(..)` で指定します。範囲を reactive にし、コンテナの
`NodeRef::rect()` から割合とピクセルの上下限を計算できます。独自のグリップは `.handle(..)` に渡し、
style で境界線、表示、位置、重ね合わせを指定します。同じサイズ binding を使う layout のパネルには
`LayoutRegion::width(..)` を使います。末尾の resize slot は呼び出し側で逆方向を指定します。
スクロールの端は `NodeRef::scroll_state()` で観測し、キー操作は中核の既定動作を使います。
