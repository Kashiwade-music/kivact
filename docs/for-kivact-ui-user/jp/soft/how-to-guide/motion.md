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
