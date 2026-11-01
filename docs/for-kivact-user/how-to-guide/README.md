# 目的別ガイド

各記事は、既に [入門](../getting-started/README.md)の setup と signal の規則を知っている読者向けです。Rust の例は `kivact-docs` の doctest に含めます。

| 作るもの | ガイド |
|---|---|
| 検証と確定操作を持つ入力 | [フォーム](forms.md) |
| 選択、キー移動、表 | [一覧と表](lists-tables.md) |
| dialog、popover、tooltip | [重ねて表示する UI](overlays.md) |
| 図形と操作領域 | [Canvas](canvas.md) |
| 値と見た目の時間変化 | [アニメーション](animation.md) |
| clipboard、dialog、window | [Platform](platform.md) |
| 翻訳と locale | [多言語対応](i18n.md) |
| 色と部品の外観 | [テーマ](theming.md) |
| エラーと更新の理由 | [診断](diagnostics.md) |
| ホストの window とプラグインの editor | [埋め込み](embed.md) |

どの kit を使っても、状態・入力・一覧の契約は共通です。分野固有の部品を選ぶときは [kit 別ガイド](../../for-kivact-ui-user/how-to-guide/README.md) に進んでください。
