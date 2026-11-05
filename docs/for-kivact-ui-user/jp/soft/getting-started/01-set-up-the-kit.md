# Soft をセットアップする

[English](../../../en/soft/getting-started/01-set-up-the-kit.md)

同じ Git リビジョンの `kivact` と `kivact-ui-soft` を追加します。ウィンドウを開く前にアプリのテーマを設定すると、標準コントロールと Soft の部品が同じ配色を使います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_soft as soft;

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(soft::theme(ThemeMode::Light))
        .run(|| {
            column((
                soft::heading(1, "Settings"),
                soft::text("Manage your profile and appearance."),
            ))
            .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
        })
}
```

Figtree を使う場合は、ウィンドウを開く前にフォントを登録します。独自のサーフェスにスタイルを適用する方法は、テーマとトークンのガイドを参照してください。
