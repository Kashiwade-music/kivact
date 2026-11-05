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
- 一度だけ mount する slot は `impl IntoView`、開くたびに作り直す内容は `impl Fn() -> V` で受け取ります。子の mount 時に setup と cleanup owner を作る場合は `component(move || view)` を使います。
- panic は不具合です。`catch_unwind` で隔離せず、毒された lock の中身にも触れません。
- glob import は使いません（例・雛形・利用者側の E2E での `kivact::prelude::*` は除きます）。

規則を外す場合は、Google の指針自体が認める場合（意図した prelude、外部の frame との境界での defense in depth、FRU で組み立てる option の束、変わらない enum など）だけ、`dev/xtask/policy/workspace.toml` の `[[style.exceptions]]` に設計上の理由を書きます。違反の件数を減らすための例外や、規則を逃れるためだけの module 分けはしません（style の規則が選んだ lint は source の `#[allow]` より優先されるので、`#[allow]` では外せません）。

道具が確かめない規則は次のとおりです。

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
| `docs/for-kivact-developer/architecture.md` | crate の境界、状態・API・依存の規則、変更時の確認箇所（日本語） |
| `docs/for-kivact-developer/decisions.md` | 設計の決定と理由（日本語） |
| `docs/for-kivact-developer/strategy.md` | 製品戦略（日本語のみ） |

英語と日本語は同じファイル名で対にし、同じ変更で両方を更新します。`docs/` と README の Rust のコードブロックは `dev/tests/docs`（`kivact-docs`）の doctest としてコンパイルされます。新しいページの登録は不要です。

## テスト

UI の試験は `kivact::test` による App 全体の E2E です。`arvane-virtual` の上で実アプリと同じ描画 driver と仮想時計を使い、手元では実 window に mirror します。公開 API の利用者の契約を確かめるものに絞り、過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は追加しません。

| 場所 | 内容 |
|---|---|
| `dev/tests/kivact-public-api` | 公開 API の E2E（`tests/e2e`）とウィンドウなしの画像書き出し（`tests/offscreen.rs`） |
| `crates/kivact-ui-*/tests` | 各キットの E2E |
| `crates/arvane/tests` | Arvane の公開契約と描画・入力の E2E |
| `crates/kivact-test` | App の共通起動経路を使う `kivact::test` ハーネス |
| `crates/arvane-virtual` | 仮想 backend、入力の台本、描画と任意の native mirror |
| `dev/tests/docs` | ドキュメントのコードブロック |
| `dev/tests/gpu-interop-producer` | GPU texture 共有の試験で使う描画エンジン |

`KIVACT_TEST_MIRROR` は `auto`（既定、可能なら mirror）、`off`（仮想 host のみ）、`required`（mirror が作れなければ失敗）を選びます。`RenderOptions::mirror` でも試験ごとに指定できます。GPU の表示や native host 固有の試験は対応する実環境が必要です。

アプリを起動する試験と実 window を開くコマンドは、`KIVACT_TEST_MIRROR=off` でも `dev/tools/window-lock.sh -- <command>` を通し、すべての worktree で同時に一つの process だけがウィンドウを開くようにします。実 window の focus 競合と backend の同時利用を避けるためです。通常の試験は Cargo の既定の並列数で実行します。`gpu-interop-producer` だけは `--test-threads=1` で直列にします。lock は時間の上限（既定 900 秒、`--timeout` で変更）も掛け、時間切れは終了コード 124 です。

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

`cargo xtask check` はファイルごとの code 行の上限、crate の `visible_to`、module の依存の向き、Google Rust Style Guide の規則（rustfmt と Clippy を含む）を検査し、`architecture/` の図と `architecture/index.html` を生成します。`state-fields`、`effect-writes`、`api-shape`、`canonical-path`、`name-collision`、`kit-literal-color`、`os-branch`、`structural-options` も含み、`--only <検査> --crate <crate> --module <module path>` で範囲を絞れます（`--crate` は policy の `skip` より優先、`--report` は違反を一覧にして成功で終了、例外は対象の field や項目の直前の `// xtask-allow(<検査>): <理由>`）。CI は fmt と `cargo xtask check --no-write` だけを実行します。`dev/tools/check-features.sh` は library の非既定 feature を一つずつ有効にしてコンパイルします。

## commit と PR

commit は目的ごとに conventional commits で分けます。PR には、具体的な問題、変更後の挙動、実施した検証、未実施の理由、残る制限を書きます。
