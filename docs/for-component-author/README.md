# 部品作者ガイド

自社のデザインシステムや標準 UI にない操作部品を、公開 API だけで作るためのガイドです。公式 kit も同じ口を使います。

| 必要なこと | 選ぶ口 |
|---|---|
| 既存の標準部品の色・寸法・外観変更 | `Theme` の token、`ui::UiRecipes`（`Theme::with`）、項目の slot。まず [テーマ](../for-kivact-user/how-to-guide/theming.md)を参照 |
| 自分の部品の色・大きさを theme に載せる | `ThemeExtension` を実装した型を `Theme::with` で載せ、`theme.get::<T>()` で読む |
| icon を theme で差し替える | `Icon::named(..)` と `Theme::icons`（`IconSet`）、表示は `control::icon(..)` |
| 外観を自分で描き、選択・開閉・編集を共通化 | [interaction](interaction.md) の controller と Part |
| 独自計測、canvas の target、native 編集の下位契約 | [primitive](primitive.md) |
| kit の部品と builder の名前 | [kit の命名規則](kit-naming.md) |

`Node` は見た目と子、`Part<S>` は対象の入力・focus・semantics・状態を受け持ちます。部品 builder が root の `Node` を持つ場合は `NodeBuilder` を実装すると通常の modifier を共有できます。部品の `role`・`checked`・`expanded`・`selected`・`value_text` と `part(part)` は `kivact::primitive::NodeExt` の method で、部品が自分の root に付けます。`disabled` と `invalid` は inherent method で上書きせず、`accessible_disabled()`・`accessible_invalid()` で読みます。部品の既定の style は `NodeExt::base_style(..)` で足します。呼ぶ順に関係なく利用者の `.style(..)` より下に積まれるので、利用者の指定が必ず勝ちます。builder の設定が決まる `into_view` の中で呼べるので、設定を後から読むための共有 cell を作りません。

独立した [external-design-system](../../dev/tests/fixtures/external-design-system/src/lib.rs) は button/dialog/select/menu と独自入力の実例です。[external-component](../../dev/tests/fixtures/external-component/src/lib.rs) には SearchBox や measured page dots があります。fixture はアプリと同じ公開契約だけに依存します。

controller と item data、Part が返す state、caller が選ぶ policy は、外部部品のための公開契約です。公開名の数を減らすためだけに内部 module へ隠しません。engine の木や runtime の ID を取り出す必要はありません。公開型・method の全一覧と細かな lifetime の条件は rustdoc を確認してください。

複合部品の主入力には root の `accessible_name()`・`accessible_invalid()` と、`accessible_labelled_by()`・`accessible_described_by()` の NodeRef 列を転送します。参照列は caller と part の順を保ったコピーで、root の source を消費しません。主入力の既存の関係を残して `.labelled_by(reference)`・`.described_by(reference)` で追加すると、field の label・hint・error が実際の editor にも伝わります。参照先の mount と破棄は通常の関係の仕組みが追います。
