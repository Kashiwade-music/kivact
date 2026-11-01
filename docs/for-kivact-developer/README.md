# Kivact 開発者向け資料

Kivact 本体・Arvane・公式 UI・内部検査を保守するための入口です。アプリ作者は [利用者ガイド](../for-kivact-user/README.md)、外部部品の作者は [interaction と primitive のガイド](../for-component-author/README.md)を参照してください。

- [現行設計と公開 API の責務](design.md)（crate をまたぐ規則と全体の契約。module ごとの振る舞いは各 module の rustdoc）
- [コーディング規則と検査](coding-rules.md)
- [製品戦略 v2](strategy/20261009_strategy.md)、[設計レビュー](strategy/REVIEW_TEMPLATE.md)、[見送りと実アプリ要求の台帳](strategy/ledger.md)
- [GUI harness・IME・スクリーンリーダーなどの手動確認一覧](manual-verification.md)
- アーキテクチャ図: [crate の依存](../../architecture/crates.svg) と [module の図](../../architecture/modules/)（`cargo xtask check` が生成する）

戦略は目標と判断基準、設計は現行契約、バックログは未解決事項です。過去の検査成功・当時の操作許可は現在の検査成功や入力許可を意味しません。旧作業計画と Web 試作は重複する実行手順を除去し、必要な要件と固定版への参照へ統合しています。

- 検査ツール: [Windows GUI harness](tools/gui-harness.md)・[Linux / WSLg](tools/linux-verification.md)
