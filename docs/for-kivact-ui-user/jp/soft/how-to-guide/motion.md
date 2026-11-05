# モーション

[English](../../../en/soft/how-to-guide/motion.md)

独自の描画フィードバックには、有効なテーマのモーショントークンを使います。この例ではポインターが重なったときに背景色を補間します。既定の装飾用トランジションは、動きを減らす設定に従います。

```rust,no_run
use kivact::motion::Transition;
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact_ui_soft as soft;

fn hover_surface() -> impl IntoView {
    soft::text("Build information").style(|| {
        let p = kivact::style::use_tokens::<soft::Tokens>();
        style()
            .padding(p.spacing.s2)
            .radius(p.radius.element)
            .background(p.colors.bg_surface)
            .hover(style().background(p.colors.bg_muted))
            .transition(
                TransitionProp::Paint,
                Transition::tween(p.motion.fast as f64, p.motion.ease_standard),
            )
    })
}
```

描画トランジションでは単色の背景や不透明度などを補間できます。レイアウト変更と文字装飾は、ホバー時の描画レイヤーとは分けて扱います。

従来の Sm、Md、Lg、Xl の glyph サイズには、標準の `kivact::ui::spinner()` と `ControlSize::ExtraSmall`、`Small`、`Medium`、`Large` を使います。色の扱いは `.variant(soft::SpinnerVariant { shade: soft::SpinnerShade::Subtle, ..Default::default() })` で選びます。`OnMedia` と `Inherit` も使え、明示する色は `ink: Some(color)` で渡します。名前と表示ラベルには `.name(label).label_content(text(label))`、独自の内容には `label_content` を使います。Soft は通常の回転周期と、動きを減らす場合の遅い 3000 ms 周期を設定します。
