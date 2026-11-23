# 変更を寄せる

当面は作者（Kashiwade）だけが開発し、外部からの pull request は受け付けません。不具合の報告は issue へ、脆弱性は [SECURITY](SECURITY.md) の手順で連絡してください。以下は作者が変更を入れるときの手順です。

## 判断の基準

何を入れ、何を入れないかは [製品戦略](docs/for-kivact-developer/strategy.md) で判断します。境界・状態・API の規則は恒久文書の [アーキテクチャ](docs/for-kivact-developer/architecture.md)、判断の理由は [設計の決定](docs/for-kivact-developer/decisions.md) を参照します。現在の API と内部構造は rustdoc とコードが正です。設計の理由は PR の説明に書きます。

公開 API と依存の向きを変える場合は、同じ変更で rustdoc、`dev/xtask/policy/workspace.toml`、利用者向けのドキュメントを更新してください。初期開発版なので、互換 wrapper・alias・旧コマンドは残さず、ドキュメントを新しい使い方に揃えます。

## コード

[アーキテクチャの規則](docs/for-kivact-developer/architecture.md) に沿って変更します。規則の多くは root の `[workspace.lints]`（全 crate が `[lints] workspace = true` で参加）と `cargo xtask check` が強制します。`cargo xtask check` は [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html) の機械的に確かめられる 20 の規則（`vendor/xtask-core/README.md` の「Style checks」）もすべて検査します。主なものは次のとおりです。

- 子 module を持つ module は、`mod` の宣言と再公開だけを置きます。実装は役割を一つに絞った葉の module に置きます。
- 公開 API がまだ大きく変わるため、次の二つは今は検査しません。
  - 公開 item の rustdoc の有無（`missing-docs`。policy の `disabled_rules` で外しています）。
  - 公開 enum と field を公開した struct への `#[non_exhaustive]`（`non-exhaustive`。policy の `extra_rules` に入れていません）。互換を残さない初期開発版では利点がなく、workspace の中の `match` から網羅性の検査を奪うためです。
- 関数は Clippy の 80 行の上限に収めます。長い処理は責務を持つ helper に分け、lint の無効化や行の詰め込みで回避しません。
- 所有と状態は `Machine`・binding・読み取り専用の投影で表します。公開 API の field に `Arc`・`Rc`・lock を漏らさず、asset は所有する byte 列や `AssetBytes`、保持する描画結果は snapshot の型で渡します。
- 部品は `#[component]` の関数で書きます。一度だけ mount する slot は `View`、開くたびに作り直す内容は `ViewFn` の props で受け取ります。
- panic は不具合です。`catch_unwind` は host から呼ばれる入口の二か所だけに置きます（Win32 の wndproc と、X11 の埋め込み UI thread の `run`。unwind が host へ届くと abort するため）。その外では `catch_unwind` を使いません。毒された lock は `embed/child.rs` の `Shared::lock` と、X11 UI thread の join handle の slot（`x11/thread.rs` の `thread_slot`）だけで回復し、ほかでは毒された lock の中身にも触れません。
- glob import は使いません（例・雛形・利用者側の E2E での `kivact::prelude::*` は除きます）。

規則を外す場合は、Google の指針自体が認める場合（意図した prelude、外部の frame との境界での defense in depth、FRU で組み立てる option の束、変わらない enum など）だけ、`dev/xtask/policy/workspace.toml` の `[[style.exceptions]]` に設計上の理由を書きます。違反の件数を減らすための例外や、規則を逃れるためだけの module 分けはしません（style の規則が選んだ lint は source の `#[allow]` より優先されるので、`#[allow]` では外せません）。

道具が確かめない規則は次のとおりです。

- コードのコメントと rustdoc は英語で書きます。crate root と module（inline の `mod {}` を含む）の先頭に `//! Role: <英語の 1 文>` を置きます（xtask が図に表示します）。例（`examples/`）、雛形（`templates/`）、`dev/tests/docs` は利用者のコードとして書くので Role を置きません。
- `unsafe` は `#[allow(unsafe_code, reason = "…")]` を明示した module だけで使い、ブロックごとに `// SAFETY:` で前提を書きます。
- feature はその crate の振る舞いだけを変えます。他の crate が match する enum の variant や、他の crate が組み立てる struct の field を feature で増減しません。
- library では `unwrap()` を使わず、避けられない前提は `expect("invariant: …")` に書きます。lint の `allow` には必ず `reason` を書きます。
- `xtask` は `vendor/xtask-core` の submodule を使います。clone の後に `git submodule update --init` を実行します。図（`architecture/workspace.xtaskmap` の 1 ファイル、commit しない）は `cargo xtask diagrams` が書き、`vendor/xtask-core` の xtask-viewer で開きます。`cargo xtask check` は図を書きません。

## ドキュメント

| 場所 | 内容 |
|---|---|
| `docs/for-kivact-user/{en,jp}/` | Kivact を初めて使う開発者向け。`getting-started/` はチュートリアル、`how-to-guide/` は目的別のガイド |
| `docs/for-kivact-ui-user/{en,jp}/<kit>/` | 各 UI キットの `getting-started/` と `how-to-guide/` |
| `docs/for-kivact-developer/architecture.md` | crate の境界、状態・API・依存の規則、変更時の確認箇所（日本語） |
| `docs/for-kivact-developer/decisions.md` | 設計の決定と理由（日本語） |
| `docs/for-kivact-developer/strategy.md` | 製品戦略（日本語のみ） |
| `docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md` | 改修前の API からの移行の表（改修の時点で固定） |

英語と日本語は同じファイル名で対にし、同じ変更で両方を更新します。日本語のファイルの Rust のコードブロックは、英語の同じ順番のブロックと token の列を同じにします。文字列の literal の中身だけは訳してかまいません（`cargo xtask check` の `docs-pairs` が、英日のファイルの集合と、ブロックの token の列を確かめます）。`docs/` の英語の木と `README.md` の Rust のコードブロックは `dev/tests/docs`（`kivact-docs`）の doctest としてコンパイルされます。新しいページの登録は不要です。直したガイドだけを確かめるには `KIVACT_DOCS_ONLY=<docs/ からの path をコンマ区切り>` を付けて `cargo test -p kivact-docs --doc` を実行します（root の README は `README.md`）。GPU texture 共有のガイドの例は `kivact-docs` の `gpu-interop` feature（`--all-features` で有効）のときだけコンパイルされます。公開 API を変えたときは、移行ガイド `docs/for-kivact-user/{en,jp}/how-to-guide/migrating-to-rework.md` には足さず、ガイドの本文を新しい使い方に揃えます。

## テスト

UI の試験は `kivact::test` による App 全体の E2E です。`arvane-virtual` の上で実アプリと同じ描画 driver と仮想時計を使い、手元では実 window に mirror します。公開 API の利用者の契約を確かめるものに絞り、過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は追加しません。

| 場所 | 内容 |
|---|---|
| `dev/tests/kivact-public-api` | 公開 API の E2E：`e2e_core`（`kivact` だけ）、`e2e_ui`（ui・interaction・APG）、`e2e_native`（embed・GPU・foreign window・DevTools）。共有の helper は lib（`kivact_public_api_tests::{support, paint}`）。ウィンドウなしの画像書き出し（`tests/offscreen.rs`） |
| `dev/tests/kivact-kit-tests` | 5 つの公式 kit が要る E2E（`e2e_kits`：theme の適合、kit の theme での標準部品） |
| `crates/kivact-ui-*/tests` | 各キットの E2E |
| `crates/arvane/tests` | Arvane の公開契約と描画・入力の E2E |
| `crates/kivact-test` | App の共通起動経路を使う `kivact::test` ハーネス（自身の試験は `tests/`） |
| `crates/arvane-virtual` | 仮想 backend、入力の台本、描画と任意の native mirror |
| `dev/tests/docs` | 英語の guide と README のコードブロック |
| `dev/tests/gpu-interop-producer` | GPU texture 共有の試験で使う描画エンジン |

`KIVACT_TEST_MIRROR` は `auto`（既定、可能なら mirror）、`off`（仮想 host のみ）、`required`（mirror が作れなければ失敗）を選びます。`RenderOptions::mirror` でも試験ごとに指定できます。GPU の表示や native host 固有の試験は対応する実環境が必要です。
GPU lane の試験（shader と GPU export）は `dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_core --all-features -- --test-threads=1` と `dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_native --all-features -- --test-threads=1` のように、window-lock を通して直列に実行します。GPU を使えない環境では、試験は `kivact::test::gpu_unavailable()` の理由を表示して skip します。

`kivact::test` の試験は、`KIVACT_TEST_MIRROR=off` でも `RenderOptions::mirror(MirrorMode::Required)` を指定した試験が実 window を開きます。実 window の focus 競合と backend の同時利用を避けるため、次の package の試験は必ず `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で lock を通し、すべての worktree で同時に一つの process だけがウィンドウを開くようにします：`kivact-public-api-tests`、`kivact-kit-tests`、`kivact-test`、`kivact-docs`、`kivact-devtools`、5 つの kit、5 つの catalog、`catalog-shell`、`kivact-plot`、`kivact-example-*`、`kivact-template-basic`、`arvane`、`arvane-native`、`gpu-interop-producer`。それ以外の package の試験は window を開かないので lock は要りません。`window-lock.sh --if-mirror` は `KIVACT_TEST_MIRROR=off` のとき lock を省きますが、mirror を強制する試験の無い package にだけ使います。`gpu-interop-producer` と `kivact-docs` の doctest は `--test-threads=1` で直列にします（後者は多数の例の同時 compile/link によるメモリ不足を避けるため）。lock は時間の上限（既定 900 秒、`--timeout` で変更）も掛け、時間切れは終了コード 124 です。

native 試験が並列実行中に異常終了した場合は、失敗した binary だけ `--test-threads=1` で再検証し、実行条件と症状を issue か PR の説明に記録します（[設計判断](docs/for-kivact-developer/decisions.md)は設計の決定だけを記録します）。

## 検査の 3 段

tier a（変更ごと。数分。lock は E2E だけ）：

```text
cargo fmt -p <crate>
cargo check -p <crate> --all-targets
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --crate <crate>                                # 局所の検査だけ
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
cargo xtask check
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs
cargo check -p kivact-docs --all-targets --all-features
```

tier c（merge の前、native の経路を変えたとき）：

```text
cargo fmt --all -- --check
cargo xtask check
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
cargo check -p kivact-docs --all-targets --all-features
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --no-default-features
for c in kivact kivact-offscreen arvane-render arvane-compose kivact-style; do cargo check -p "$c" --no-default-features --lib || exit 1; done
tree=$(mktemp) && cargo tree -e normal -p kivact --no-default-features --prefix none > "$tree" && ! grep -Eq '^wgpu(-[a-z-]+)? v' "$tree"
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_core --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_native --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-test --test screen -- gpu:: --test-threads=1
dev/tools/check-features.sh
```

`cargo xtask check` はファイルごとの code 行の上限、crate の `visible_to`（dev-dependency は対象外）、module の依存の向き、Google Rust Style Guide の規則（rustfmt と Clippy を含む）と、局所の検査（`state`、`effect-writes`、`api-shape`、`canonical-path`、`name-collision`、`kit-literal-color`、`os-branch`、`docs-pairs`、`interaction-view`、`ui-machine`、`hand-element`、`unit-machine`、`element-shadow`）を行います。図は書きません（`cargo xtask diagrams`）。`--crate <crate>` を付けると局所の検査だけをその crate に行い（tier a）、`--only <検査,...>` で検査を選び（`--only local` はすべての局所の検査）、`--module <module path>` で範囲を絞ります。`--report` は違反を一覧にして成功で終了します。

状態の規則（`state`）は、`bool`・`Option`・名前の付いた counter・`Cell`・`RefCell`・`Mutex`・`RwLock`・`Atomic*`・`Signal`・`Var` の field を 2 つ以上持ち、作った後に書かれる（`&mut self` の method、内部可変の field、自分の method が書く `Signal`/`Var`）型と、状態の local を 2 つ以上持つ関数を対象にします。組合せは enum で閉じ、閉じない組合せは型（関数）の直前の 1 行 `// xtask-allow(state): <組合せの閉じ方>` に書きます。constructor・`Default`・値渡しの builder だけで書く field は状態ではありません。

CI は fmt と `cargo xtask check` だけを実行します。`dev/tools/check-features.sh` は公開の意味を持つ feature を一つずつ有効にしてコンパイルします（tier c）。DevTools の JSON schema は protocol の型から `cargo test -p kivact-devtools-protocol --test schema_generated` が照合し、`KIVACT_WRITE_SCHEMA=1` を付けると書き直します。

KPI（起動・メモリ・CPU・フレーム時間・更新レイテンシ・バイナリサイズ）は `dev/tools/window-lock.sh --timeout 5400 -- dev/bench/run.sh <label>` で測ります。
実 window を開き release の build を 9 回行うため、時間がかかります。比べる二つの commit を同じ機械で測り、report を比べます（[dev/bench/README.md](dev/bench/README.md)）。
手動の道具であり、CI には組み込みません。結果の Markdown は `dev/bench/results/` に書かれ、commit しません。

## commit と PR

commit は目的ごとに conventional commits で分けます。PR には、具体的な問題、変更後の挙動、実施した検証、未実施の理由、残る制限を書きます。
