# Kivact 利用者ガイド

[入門](getting-started/README.md)では、小さなアプリを作りながら状態、配置、入力、一覧、試験を学びます。作りたい機能が決まっている場合は [目的別ガイド](how-to-guide/README.md) を使ってください。

標準部品は `kivact::ui`、要素と配置は `kivact::control`、見た目を持たない振る舞いは `kivact::interaction` にあります。独自部品の作者向けの口は [部品作者ガイド](../for-component-author/README.md) で説明します。公式 kit のテーマと業務・制作向け部品は [公式 UI ガイド](../for-kivact-ui-user/README.md) を参照してください。

完全な型・method の一覧は `cargo doc -p kivact --no-deps` で生成する rustdoc です。初期開発版のため、API は予告なく破壊的に変わります。
