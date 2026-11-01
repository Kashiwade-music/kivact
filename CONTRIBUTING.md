# 変更を寄せる

当面は作者（Kashiwade）だけが開発し、外部からの pull request は受け付けません。不具合の報告は issue へ、脆弱性は [SECURITY](SECURITY.md) の手順で連絡してください。以下は作者が変更を入れるときの手順です。

変更前に [設計](docs/for-kivact-developer/design.md) と [コーディング規則](docs/for-kivact-developer/coding-rules.md) を読みます。公開 API と依存の向きを変える場合、同じ変更で設計、`dev/xtask/policy/workspace.toml`、rustdoc、利用者向けの該当手順を更新してください。

利用者の入門は `docs/for-kivact-user/getting-started`、作業別の手順は `docs/for-kivact-user/how-to-guide`、公式部品は `docs/for-kivact-ui-user`、部品作者は `docs/for-component-author`、開発者の説明は `docs/for-kivact-developer` に置きます。API reference は公開 item の rustdoc に書きます。入門で内部 module や engine の統計を前提にしません。

merge の前に次の gate を通します。window を開くものは、下の規則に従って実行します。

```text
cargo fmt --all -- --check
cargo xtask check
git status --porcelain architecture   # 空であること
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
dev/tools/check-features.sh
cargo test -p kivact-workspace-policy-tests
cargo test -p kivact-docs --doc
```

まとめて記録する場合は `bash dev/tools/check-workspace.sh [log-directory]` と `bash dev/tools/check-consumers.sh [log-directory] [--cpu-only]` を使います。既存の出力先は上書きせず終了コード 2 で止まります。後者は独立 consumer をすべて検査し、CPU だけの確認では `--cpu-only` を指定します。各 window のコマンドが個別に lock を取るので、この二つの script 全体を window-lock で包みません。環境変数の renderer・GPU・Cargo 設定は呼出元から渡します。Linux の全体確認は[Linux の検証](docs/for-kivact-developer/tools/linux-verification.md)の Windows 側 outer lock を必ず使います。

公開 API の統合試験は `dev/tests/kivact-public-api`、Markdown の code block を検査する crate は `dev/tests/docs`、検証ツールは `dev/tools` にあります。独立 consumer の確認と Windows/Linux の検証手順は開発者ガイドを参照してください。

window を開く自動試験（`kivact::test`、公開 API の e2e、kit の consumer と catalog、window を開く doctest）は `dev/tools/window-lock.sh -- <command>` を通し、すべての worktree で同時に1つの process だけが window を開くようにします（同時に開くと focus を奪い合い、試験が止まることがあります）。lock は時間の上限（既定 900 秒）も掛け、時間切れは終了コード 124 になります。実際の OS の入力を送る [GUI harness](docs/for-kivact-developer/tools/gui-harness.md) と、IME・スクリーンリーダー・ブラウザの手動確認は、対象アプリ・コマンド・操作を説明してユーザーの明示許可を得てから実行します。未実施の検証は未実施として報告します。今回の対象と操作は [手動確認一覧](docs/for-kivact-developer/manual-verification.md)にあります。

Linux の埋め込みの実入力を含む試験は、同じプロセス内でもfocusを共有するため `--test-threads=1` で実行します。`dev/tools/linux/test-wslg.sh` はこの指定を含みます。FDなどのプロセス資源を比較する試験は専用のintegration binaryに分離します。

commit は目的ごとに conventional commits で分けます。PR は具体的な問題、変更後の挙動、実施した検証、未実施の理由、残る制限を書きます。このプロジェクトは初期開発版なので、互換 wrapper・alias・旧コマンドの維持を追加せず、新しい利用方法へ文書を揃えてください。
