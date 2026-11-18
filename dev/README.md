# dev

開発用の検査とテストです。全体の手順は [CONTRIBUTING](../CONTRIBUTING.md)、
境界と規則は [アーキテクチャ](../docs/for-kivact-developer/architecture.md) にあります。
以下の shell コマンドはリポジトリのルートから実行します（Windows では Git Bash）。

| 場所 | 用途 |
|---|---|
| `tests/kivact-public-api/` | 公開 API の E2E（`tests/e2e_core`、`tests/e2e_ui`、`tests/e2e_native`）、共有の helper（`src/`）、offscreen の描画契約 |
| `tests/kivact-kit-tests/` | 5 つの公式 kit が要る E2E（`tests/e2e_kits`） |
| `tests/docs/` | 英語の `docs/` と README の Rust コードブロック（doctest と `no_run` の型検査） |
| `tests/gpu-interop-producer/` | GPU texture 共有の試験で使う独立した描画エンジン |
| `../crates/kivact-test/` | 共通の App 起動経路を使う E2E ハーネス（公開入口は `kivact::test`） |
| `../crates/arvane-virtual/` | 仮想時計・入力台本・共通描画 driver・任意の実 window mirror |
| `tools/window-lock.sh` | 実 window を開くコマンドを worktree 間で一つずつ実行する lock（`--if-mirror`） |
| `tools/check-features.sh` | 公開の意味を持つ feature を一つずつコンパイルする |
| `tools/subset-catalog-font.py` | catalog の Noto Sans JP を使う文字の subset に作り直す（要 fontTools、原本は `--source`） |
| `xtask/` | workspace の規則の検査（`cargo xtask check`）と依存図の生成（`cargo xtask diagrams`、要 Node.js） |
| `bench/` | 性能の指標（K1〜K6）を手動で測る probe と `run.sh`（[README](bench/README.md)、CI では実行しない） |

## 検査の 3 段

すべての cargo のコマンドに `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` を付けると、debug 情報を作らず build が速くなります（debug assertions は有効のまま）。

tier a（変更ごと。数分。lock は E2E だけ）：

```text
cargo fmt -p <crate>
cargo check -p <crate> --all-targets
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --crate <crate>                      # 局所の検査だけ。pending を閉じる unit は --strict を付ける
cargo test -p <crate> <filter>                                     # lock の一覧に無い package
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p <crate> <filter>      # lock の一覧の package
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native --all-features -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- <module>
```

tier b（まとまった変更の後）：

```text
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs
cargo check -p kivact-docs --all-targets --all-features
rg -n "xtask-allow\(state-fields\)" crates dev examples templates     # W1c の後：何も出ない
```

tier c（merge の前、native の経路を変えたとき）は [CONTRIBUTING](../CONTRIBUTING.md) の「検査の 3 段」にあります。

## xtask の検査を絞る

```text
cargo xtask check --no-write --crate kivact-ui-soft
cargo xtask check --no-write --only state --strict --crate kivact-ui-soft --module navigation_session
cargo xtask check --no-write --only effect-writes,state --crate kivact-ui-soft
cargo xtask check --no-write --only local
```

`--crate` は局所の検査だけを行います（xtask-core の行数・`visible_to`・module の向き・style は全体の `cargo xtask check` で行います）。`--strict` は `[rework.pending]` の note を error にします。

## E2E と doctest

`kivact::test` の E2E は `arvane-virtual` の上で App 全体を動かします。
`KIVACT_TEST_MIRROR=auto`（既定）は可能なら実 window に写し、`required` は実 window を必須にし、`off` は仮想 host だけにします。ただし試験が `RenderOptions::mirror(MirrorMode::Required)` を指定すると `off` でも実 window を開くので、E2E は lock を通します。

```text
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- menu
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test consumer
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p gpu-interop-producer --all-features -- --test-threads=1
cargo check -p kivact-docs --all-targets
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc -- readme --test-threads=1
```

`cargo test -p kivact-docs --doc -- readme` は root の `README.md` の例だけを、`-- for_kivact_ui_user_en::basic` は basic の guide だけを検査します。
native 試験が並列実行中に異常終了した場合は、失敗した binary だけ `--test-threads=1` で再検証し、実行条件と症状を issue か PR の説明に記録します。

## feature の確認

```text
cargo check -p kivact --no-default-features --lib
cargo check -p kivact --no-default-features --features test --lib
dev/tools/check-features.sh
```

`kivact` の既定は `gpu` と `complex-scripts`（ICU の分かち書きの辞書）です。任意の `test` は `kivact-test`、`embed` は native 埋込み、
`devtools` は DevTools の agent と protocol、`gpu-interop` は GPU texture 共有、`tokio` は非同期 task の連携を有効にします。
`check-features.sh` は facade の feature と `arvane-render/gpu`・`arvane-native/embed`・`kivact-gpu-share` の feature を一つずつ確認します。
