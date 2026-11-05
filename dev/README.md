# dev

開発用の検査とテストです。全体の手順は [CONTRIBUTING](../CONTRIBUTING.md)、
境界と規則は [アーキテクチャ](../docs/for-kivact-developer/architecture.md) にあります。
以下の shell コマンドはリポジトリのルートから実行します（Windows では Git Bash）。

| 場所 | 用途 |
|---|---|
| `tests/kivact-public-api/` | 公開 API を使う App 全体の E2E（`tests/e2e/`）と offscreen の描画契約 |
| `tests/docs/` | `docs/`・README・README_JP の Rust コードブロックの doctest |
| `tests/gpu-interop-producer/` | GPU texture 共有の試験で使う独立した描画エンジン |
| `../crates/kivact-test/` | 共通の App 起動経路を使う E2E ハーネス（公開入口は `kivact::test`） |
| `../crates/arvane-virtual/` | 仮想時計・入力台本・共通描画 driver・任意の実 window mirror |
| `tools/window-lock.sh` | App の E2E と window を開くコマンドを worktree 間で一つずつ実行する lock |
| `tools/check-features.sh` | library の非既定 feature を一つずつコンパイルする |
| `xtask/` | workspace の規則の検査と `architecture/` の依存図の生成 |

## 静的な検査

```text
git submodule update --init vendor/xtask-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask check --no-write
```

`cargo xtask check` は `architecture/` の図を更新します。確認だけなら `--no-write` を使います。
ファイルの行数、module の依存の向き、公開 API の形、単一の公開 path、kit の色、OS 分岐、
構造の option、状態の組合せ、effect 内の状態書き込みを検査します。関数の 80 行の上限と
禁止する所有・lock 型は Clippy でも検査します。対象を絞る例は次のとおりです。

```text
cargo xtask check --no-write --only state-fields --crate kivact-ui-soft --module navigation_session
cargo xtask check --no-write --only effect-writes --crate kivact-ui-soft
```

`--only` は一度に一つです。ほかに `api-shape`・`canonical-path`・`name-collision`・
`kit-literal-color`・`os-branch`・`structural-options` を選べます。

## E2E と doctest

通常の試験は既定の並列数のまま実行します。GPU producer の試験だけを直列にします。
アプリを起動する試験は mirror を無効にしても共通 lock を通します。

```text
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --test e2e
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft --test consumer
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
cargo test -p kivact-docs --doc
```

`kivact::test` の E2E は `arvane-virtual` の上で App 全体を動かします。
`KIVACT_TEST_MIRROR=auto`（既定）は可能なら実 window に写し、`required` は実 window を必須にします。
GPU の表示や native host 固有の試験は実環境が必要です。`cargo test -p kivact-docs --doc -- guides::readme` は
二つの root README にある例だけを検査します。実行する Rust の例が App を起動する場合は、
doctest も `dev/tools/window-lock.sh` を通します。

## feature の確認

```text
cargo check -p kivact --no-default-features --lib
cargo check -p kivact --no-default-features --features test --lib
dev/tools/check-features.sh
```

`kivact` の既定は `gpu` です。任意の `test` は `kivact-test`、`embed` は native 埋込み、
`gpu-interop` は GPU texture 共有、`tokio` は非同期 task の連携を有効にします。
workspace 全体の build は dev-dependency 経由で feature が統合されるため、
`check-features.sh` が `kivact-core`・`kivact-test`・`kivact-engine`・Arvane adapters・
`kivact-gpu-share` の非既定 feature も個別に確認します。
