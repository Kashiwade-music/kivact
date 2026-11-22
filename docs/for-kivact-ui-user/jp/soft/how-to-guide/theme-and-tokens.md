# テーマと変数

[English](../../../en/soft/how-to-guide/theme-and-tokens.md)

キットの色・間隔・角丸は、スタイル内で定数として読めます。独自のサーフェスも最も近いテーマに追従します。背景・境界線・文字には用途別の色を使い、余白と角丸には周囲のコントロールと合わせた定数を使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

#[component]
fn project_summary() -> impl Element {
    column((
        kivact::ui::heading("Project summary", 2),
        soft::styled_text("Three milestones remain.").color(soft::TextColor::Secondary),
    ))
    .style(
        style()
            .padding(soft::SPACE_S4)
            .gap(soft::SPACE_S2)
            .background(soft::BG_SURFACE)
            .border(1.0, kivact::style::color::BORDER)
            .border_radius(soft::RADIUS_CONTAINER),
    )
}
```

キットのそれぞれの色・間隔・角丸はテーマの変数で、`const` が名前を与えます（`soft::BG_SURFACE`、`soft::TEXT_PRIMARY`、`soft::SPACE_S4`、`soft::RADIUS_CONTAINER`）。その定数を使うスタイルは、描画時に有効なテーマの値を読むので、キット自身の部品と同じようにテーマの切り替えに追従します。キットの値は `soft::theme(mode)` が与えます。アプリが選ぶのはモードで、個々の値ではありません。

Soft カタログでは Figtree を登録しています。アプリで Figtree を使う場合は、フォントデータを用意して登録します。未登録の場合は、テーマが指定するシステムの代替書体で表示されます。
