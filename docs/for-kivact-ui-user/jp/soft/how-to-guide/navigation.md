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
        radio_group(
            "Project view",
            vec![ListItem::new("summary", "Summary"), ListItem::new("activity", "Activity")],
            view,
        ).variant(kivact::ui::RadioGroupVariant::Segmented),
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

開閉する節には標準の `disclosure(title, open).content(factory)` と `.animated(true)` を使います。Soft テーマは従来の spring transition を設定します。`disclosure_group(signal(None::<K>))` は単一選択、`Vec<K>` の binding は複数選択です。キー付きの節は `.panel(key, title, factory)` で追加します。区切り線には `.variant(soft::DisclosureVariant { dividers: true, ..Default::default() })` を指定します。`chevron` には `ChevronPosition::Start` または `End`、`density` には `Some(DisclosureDensity::Compact)`、`Balanced`、`Spacious` を指定できます。この装飾は単独の節とグループで使え、文字方向に従います。折りたたまない節には `.always_open()`、独自のトリガーには `.content(factory).attach(trigger)` を使います。初期状態は呼び出し側の `signal(default_open)` で指定します。

ツールバーには標準の `toolbar(label).start(...).center(...).end(...)` を使います。表面の装飾は `.variant(soft::ToolbarVariant(soft::SectionVariant::Muted))`、論理辺の区切り線は `.dividers(&[ToolbarEdge::Bottom])` で指定します。初回フォーカス時のヒントには content factory を `soft::with_key_hints(axis, || view)` で囲み、実際のキー操作対象に `soft::keyboard_hint(button)` を付けます。ヒントの方向はリアクティブな orientation に対応し、矢印操作、フォーカスが離れること、または 3 秒で終了します。segmented control は `radio_group(...).variant(RadioGroupVariant::Segmented)` と、必要に応じて `soft::SegmentedLayout::{Hug,Fill}` を使います。アイコンのみの節では `SegmentedIconOnly` と、アイコンだけを返す `.item(...)` を組み合わせます。標準 radio はアクセシブルな名前を保持します。無効な理由は `tooltip(reason).enabled(disabled)` を付けて説明し、無効化には標準の `.disabled(...)` を使います。
