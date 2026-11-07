# モーション

[English](../../../en/soft/how-to-guide/motion.md)

独自の描画フィードバックには、有効なテーマのモーショントークンを使います。この例ではポインターが重なったときに背景色を補間します。既定の装飾用トランジションは、動きを減らす設定に従います。

```rust,no_run
use kivact::motion::Transition;
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact_ui_soft as soft;

fn hover_surface() -> impl IntoView {
    soft::styled_text("Build information").style(|| {
        let p = kivact::style::use_tokens::<soft::Tokens>();
        style()
            .padding(p.spacing.s2)
            .radius(p.radius.element)
            .background(p.colors.bg_surface)
            .hover(style().background(p.colors.bg_muted))
            .transition(
                TransitionProp::All,
                Transition::tween(p.motion.fast as f64, p.motion.ease_standard),
            )
    })
}
```

描画トランジションでは単色の背景や不透明度などを補間できます。レイアウト変更と文字装飾は、ホバー時の描画レイヤーとは分けて扱います。

従来の Sm、Md、Lg、Xl の glyph サイズには、標準の `kivact::ui::spinner()` と `ControlSize::ExtraSmall`、`Small`、`Medium`、`Large` を使います。色の扱いは `.variant(soft::SpinnerVariant { shade: soft::SpinnerShade::Subtle, ..Default::default() })` で選びます。`OnMedia` と `Inherit` も使え、明示する色は `ink: Some(color)` で渡します。名前と表示ラベルには `.name(label).label_content(text(label))`、独自の内容には `label_content` を使います。Soft は通常の回転周期と、動きを減らす場合の遅い 3000 ms 周期を設定します。

従来の角を丸めた placeholder には標準の `skeleton().variant(soft::SkeletonRadius::Rounded)` を使い、時間差には `.index(n)` を指定します。すべての radius はキットの variant として使えます。標準の `.shape(...)` と `.lines(...)` で円形やテキストのグループも指定できます。Soft は 1000 ms と index ごとに 100 ms の遅延後に始まる、400 ms、10 段階の交互の pulse を設定します。動きを減らす場合は低い不透明度で静止します。
