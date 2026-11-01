# 変更を寄せる

当面は作者（Kashiwade）だけが開発し、外部からの pull request は受け付けません。不具合の報告は issue へ、脆弱性は [SECURITY](SECURITY.md) の手順で連絡してください。以下は作者が変更を入れるときの手順です。

## 判断の基準

何を入れ、何を入れないかは [製品戦略](docs/for-kivact-developer/strategy.md) で判断します。現在の API と内部構造は rustdoc とコードが正です。設計の理由は PR の説明に書きます。

公開 API と依存の向きを変える場合は、同じ変更で rustdoc、`dev/xtask/policy/workspace.toml`、利用者向けのドキュメントを更新してください。初期開発版なので、互換 wrapper・alias・旧コマンドは残さず、ドキュメントを新しい使い方に揃えます。

## コード

規則の多くは root の `[workspace.lints]`（全 crate が `[lints] workspace = true` で参加）と `cargo xtask check` が強制します。道具が確かめない規則は次のとおりです。

- コードのコメントと rustdoc は英語で書きます。すべての crate root と module（inline の `mod {}` を含む）の先頭に `//! Role: <英語の 1 文>` を置きます（xtask が図に表示します）。
- `unsafe` は `#[allow(unsafe_code, reason = "…")]` を明示した module だけで使い、ブロックごとに `// SAFETY:` で前提を書きます。
- feature はその crate の振る舞いだけを変えます。他の crate が match する enum の variant や、他の crate が組み立てる struct の field を feature で増減しません。
- library では `unwrap()` を使わず、避けられない前提は `expect("invariant: …")` に書きます。lint の `allow` には必ず `reason` を書きます。
- `xtask` は `vendor/xtask-core` の submodule を使います。clone の後に `git submodule update --init` を実行します。図の生成には Node.js が要ります。

## ドキュメント

| 場所 | 内容 |
|---|---|
| `docs/for-kivact-user/{en,jp}/` | Kivact を初めて使う開発者向け。`getting-started/` はチュートリアル、`how-to-guide/` は目的別のガイド |
| `docs/for-kivact-ui-user/{en,jp}/<kit>/` | 各 UI キットの `getting-started/` と `how-to-guide/` |
| `docs/for-kivact-developer/strategy.md` | 製品戦略（日本語のみ） |

英語と日本語は同じファイル名で対にし、同じ変更で両方を更新します。`docs/` と README の Rust のコードブロックは `dev/tests/docs`（`kivact-docs`）の doctest としてコンパイルされます。新しいページの登録は不要です。

## テスト

テストは実ウィンドウの E2E（`kivact::test`）だけです。公開 API の利用者の契約を確かめるものに絞り、過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は追加しません。

| 場所 | 内容 |
|---|---|
| `dev/tests/kivact-public-api` | 公開 API の E2E（`tests/e2e`）とウィンドウなしの画像書き出し（`tests/offscreen.rs`） |
| `crates/kivact-ui-*/tests` | 各キットの E2E |
| `crates/arvane/tests` | ネイティブホストの E2E |
| `dev/tests/docs` | ドキュメントのコードブロック |
| `dev/tests/gpu-interop-producer` | GPU texture 共有の試験で使う描画エンジン |

ウィンドウを開くコマンド（`cargo test` の大半）は `dev/tools/window-lock.sh -- <command>` を通し、すべての worktree で同時に一つの process だけがウィンドウを開くようにします。同時に開くと focus を奪い合い、試験が止まることがあります。lock は時間の上限（既定 900 秒、`--timeout` で変更）も掛け、時間切れは終了コード 124 です。

## merge 前の検査

```text
cargo fmt --all -- --check
cargo xtask check
git diff --quiet -- architecture
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --no-default-features
dev/tools/check-features.sh
```

`cargo xtask check` はファイルごとの code 行の上限、crate の `visible_to`、module の依存の向きを検査し、`architecture/` の図を生成します。CI は fmt と `cargo xtask check --no-write` だけを実行します。`dev/tools/check-features.sh` は library の非既定 feature を一つずつ有効にしてコンパイルします。

## commit と PR

commit は目的ごとに conventional commits で分けます。PR には、具体的な問題、変更後の挙動、実施した検証、未実施の理由、残る制限を書きます。
