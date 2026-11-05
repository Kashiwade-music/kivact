# テーマとトークン

[English](../../../en/soft/how-to-guide/theme-and-tokens.md)

スタイルクロージャー内で `kivact::style::use_tokens::<soft::Tokens>()` を読み取ると、独自のサーフェスも最も近いテーマに追従します。背景・境界線・文字には用途別の色を使い、余白と角丸には周囲のコントロールと共通のトークンを使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn project_summary() -> impl IntoView {
    column((
        kivact::ui::heading("Project summary", 2),
        soft::styled_text("Three milestones remain.").color(soft::TextColor::Secondary),
    ))
    .style(|| {
        let p = kivact::style::use_tokens::<soft::Tokens>();
        style()
            .padding(p.spacing.s4)
            .gap(p.spacing.s2)
            .background(p.colors.bg_surface)
            .border(p.sizes.border_width, p.colors.border)
            .radius(p.radius.container)
    })
}
```

Soft カタログでは Figtree を登録しています。アプリで Figtree を使う場合は、フォントデータを用意して登録します。未登録の場合は、テーマが指定するシステムの代替書体で表示されます。
