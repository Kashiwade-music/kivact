# コーディング規則

この workspace のすべての crate（Kivact・engine・Arvane・外部部品・アプリ・例・xtask）に掛かる規則です。道具が自動で確かめるものと、欠かせないものだけを書きます。設計は [設計](design.md) にあります。人間もエージェントも、`dev/xtask/`・`clippy.toml`・policy・この文書を必要に応じて直してかまいません。

## 自動で確かめるもの

| 検査 | 内容 |
|---|---|
| `cargo fmt --all -- --check` | rustfmt の既定 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | root の `[workspace.lints]`。全 crate が `[lints] workspace = true` で参加する |
| `cargo xtask check` | 1ファイルの code 行の上限、`visible_to`、全 module の `Role:` 宣言、module の参照の循環（下の節）。全 feature を有効にして解析し、図を `architecture/` に書き出す |
| `cargo test -p kivact-workspace-policy-tests` | 公式 kit・例・雛形の直接依存の許可リストと、公開名の一覧（golden） |
| `cargo test --workspace --all-features` | 試験 |
| `cargo test -p kivact-docs --doc` | README と `docs/` の rust の code block |
| `dev/tools/check-features.sh` | library crate の非既定 feature を一つずつ有効にして facade を build する（workspace 全体の build では dev-dependency で feature が揃い、単独の feature の誤りが見えないため） |
| `dev/tools/linux/test-wslg.sh` | Linux の検査（WSLg の上で clippy と、Wayland と X11 のそれぞれで Arvane の契約の試験と workspace の試験。`kivact-music-player` を除く）。Windows の Git Bash から WSL の中で動かす |


workspace の lint の主なもの:

- `unsafe_code` は deny。使うときは、その module に `#[allow(unsafe_code, reason = "…")]` を明示し、`unsafe` ブロックごとに `// SAFETY:` で前提を書き、1ブロックに unsafe 操作を1つだけ入れる（clippy の `undocumented_unsafe_blocks`・`multiple_unsafe_ops_per_block`）。`unsafe fn` の中でも `unsafe {}` で囲む（`unsafe_op_in_unsafe_fn`）。今の workspace の `unsafe` は `arvane-native` の `module.rs`（thread・provider が残る間の OS loader による code module の保持）、`embed/windows`（system STATIC の subclass・元の wndproc の復元、UIA・Imm32・描画と入力）、`embed/x11`（所有する XCB 接続の raw handle）と、同 crate の `display/windows.rs`（初回 window を配置するための monitor の作業領域の読取り）と `native_input/windows.rs`（Windows の window の subclass、IME の問い合わせの buffer、system の設定の読み取り、touch と pen の pointer の情報の読み取り。winit が扱わないため）、`gpu-share` の `import/dx12.rs`・`import/vulkan.rs`（GPU texture 共有の取り込みと、wgpu の queue に直接出す同期）、共通の `d3d12_contract.rs`・`vulkan_raw.rs`・`wgpu_device.rs`（借用した resource の transition、Vulkan の barrier・timeline・adapter identity、共有 extension を足した device の作成）、feature `export` の `d3d12.rs`・`d3d12/raw.rs`・`d3d12/read_back.rs`・`vulkan.rs`・`vulkan/read_back.rs`（engine の device の上の pool・barrier・同期・読み戻し。生の handle を受け取る公開関数は `unsafe fn` で、`# Safety` に条件を書く）、feature `wgpu` の `wgpu.rs`・`wgpu/vulkan.rs`・`wgpu/d3d12.rs`（wgpu の hal の入口と、wgpu の queue に出す生の submit。queue の外部同期を求める `submit` と `read_back` は `unsafe fn`）と、試験用の engine `dev/tests/gpu-interop-producer` の `d3d12.rs`・`vulkan.rs`・`wgpu_engine.rs` だけ。ほかに、`kivact-core` の `view/gpu_interop.rs` が公開の `unsafe fn import`、`view/gpu_share/link.rs` が `unsafe fn ShareLink::provide`（どちらも engine が約束を守ることを呼出元に求める宣言。`unsafe` のブロックはない）を宣言し、公開 API の試験 `dev/tests/kivact-public-api/tests/e2e/gpu_interop.rs` と `kivact-ui-creative` の consumer 試験 `tests/consumer/gpu_interop.rs` がそれらを呼ぶ。
- feature はその crate の振る舞いだけを変える。他の crate が match する enum の variant や、他の crate が組み立てる struct の field を feature で増減しない（上位の crate は下位の crate の feature を見られない）。
- 子を持つ module は `<name>.rs` と `<name>/` で書き、`mod.rs` は使わない（`mod_module_files`）。
- 公開しない item は `pub(crate)`・`pub(super)` にする（`unreachable_pub`）。`pub` は公開 API を意味する。
- library で `unwrap()`・`todo!`・`unimplemented!`・`dbg!`・`print!`・`eprint!` を使わない。避けられない前提は `expect("invariant: …")` に書く。試験の中は `clippy.toml` で許している。出力が役目のツールや例では、理由を書いた `allow` を置く。
- lint を `allow` するときは必ず `reason = "…"` を書く（`allow_attributes_without_reason`）。
- `examples/`・`dev/apps/`・`templates/` の crate は、各 target の root（`lib.rs`・`main.rs`・`bin/*.rs`・`tests/*.rs`）に `#![warn(redundant_imports)]` を置く。prelude で届く名前の再 import を防ぐためで、`[lints] workspace = true` とは別に root の attribute で指定する。

## `cargo xtask check`

`cargo xtask check` は [xtask-core](https://github.com/Kashiwade-music/xtask-core)（`vendor/xtask-core` の submodule）を動かします。規則は [`dev/xtask/policy/workspace.toml`](../../dev/xtask/policy/workspace.toml) にあります。clone の後は `git submodule update --init` を実行します。図を書くには Node.js が要ります（`--no-write` なら不要）。

- **code 行の上限。** workspace の全 crate の `src/`・`tests/`・`benches/`・`examples/` の `.rs` と `build.rs` は、1ファイル 1000 code 行まで。空行、comment だけの行、文字列 literal の中だけの行は数えない。超えたら責任の単位でファイルを分ける。
- **visible_to。** crate ごとに、依存してよい crate を `visible_to` に書く（書かなければ誰でも依存できる。末尾の `*` は前方一致、`[]` は誰も依存できない）。normal・dev・build・optional・target 別のすべての依存を照合する。crate を足す・依存の向きを変えるときは、同じ変更で policy を直す。意図は [設計](design.md) の「依存の向きと visible_to」。
- **Role。** すべての crate root と module（inline の `mod {}` を含む）の先頭に、1行の内側 doc `//! Role: <英語の1文、80文字まで>` を書く。xtask-core はこれを図に表示する。
- **module の循環。** 全 crate の module 間の循環と未解決の local path を禁止する。共有する値と受動的な状態を操作より下へ置き、定義元から明示的に import する。`dev/xtask` は xtask-core の検査を呼び、検出した問題を警告へ振り分けない。
- **crate 内の module の向き。** `[module_rules.<crate>.uses]` に、最上位の module（子孫を含む）が参照してよい同じ crate の最上位の module を書く。`uses` が名前を挙げた module への、許していない参照は error になる（`uses` に無い module は検査しない）。上向きの例外は `[module_rules.<crate>] except = ["from -> to"]`（`crate::` を省く。from はその module だけ、to はその module と子孫）に書き、どの参照にも使われない例外も error にする。xtask-core は知らない table を拒むので、`dev/xtask` は `[module_rules]` を除いた policy を `target/xtask/workspace.toml` に書いて xtask-core に渡す。今の規則は `kivact-core`（最上位の全 module）と `kivact-interaction`（部品ごと）の向きで、意図は [設計](design.md) の「依存の向きと visible_to」。macro の呼出しの path は参照に数えないので、macro は定義元の path で呼ぶ（`kivact_reactive::constant_value!`。上の層の再公開を経由しない）。
- **macro と derive。** 解析は macro を展開しない。workspace の `macro_rules!` と、それを呼ぶ path（`crate::view::constant_value` など）は、`dev/xtask` が repository の `.rs` を token に分けて見つけ、`[analysis] trusted_macros` に足す（`target/`・`vendor/`・隠しディレクトリは除く）。policy に手で書くのは標準と外部 crate の macro だけ。
- **図。** `cargo xtask check` は `architecture/crates.svg` と `architecture/modules/**` を書き換える。生成物なので手で編集せず、`docs(architecture): regenerate diagrams` の独立した commit にする。`--no-write` は図との差分を確かめないため、gate では `cargo xtask check` の後に `git status --porcelain architecture` が空であることを確かめる。

公開名と依存の許可リストは `dev/tests/workspace-policy` の試験が確かめます。

- **公開名。** `kivact` の root と全公開 module（`prelude`・`ui`・`interaction`・`primitive`・`view`・`style`・`draw`・`platform`・`control`・`task`・`motion`・`i18n`・`diagnostics` と feature の `test`・`gpu_interop`、その公開の子 module）の直接公開 item と再公開の名前を `public-names.golden.md` と照合する。feature 条件の item も含み、method と From 実装は数えない。数は目安（上限ではない）。名前を足す・消すときは `KIVACT_NAMES_BLESS=1 cargo test -p kivact-workspace-policy-tests --test public_names` で golden を更新し、PR で理由を説明する。prelude には、入門の例が名指しする名前だけを置く。
- **依存の許可リスト。** 公式 kit・例・雛形・catalog の直接依存（外部 crate、normal・dev・build、target 別、rename を含む）の完全な一覧を `tests/dependencies.rs` に持つ。公式 kit は `kivact` だけに依存し、外部の開発者と同じ公開 facade で部品を作る。

コードのコメントと doc は英語、`docs/` は日本語で書きます。設計上の不変条件（外部 crate の型を公開 API に出さない、など）は [設計](design.md) にあります。
一つの module の中で閉じる振る舞いと不変条件は、その module の `//!` doc（`Role:` の行の後に説明と `Invariants:`）か公開 item の rustdoc に書き、設計には crate をまたぐ規則と全体の契約だけを書きます。

- **kit の共通署名。** `kit_signatures` は [kit の命名規則](../for-component-author/kit-naming.md) の署名表を読み、明示的な公開 export を解決して七つの共通構成関数の入力と制約を照合します。引数名・空白・既知の facade の修飾だけを正規化し、別の kit の独自型は同一視しません。標準のみの `field` は公開口の所属だけを確かめます。kit の root は export を1つずつ名指しします（`pub use …::*` と `pub mod` は試験が拒みます）。統一した綴り `empty`・`breadcrumbs` は export できません。

## 試験

- 試験は実 window の上の E2E だけで書きます。`kivact::test` は Arvane の実 window（仮想時計）の上で動き、driverの正規化入力で操作し、engine が公開した accessibility の木・frame の pixel・`errors()` で確かめます。内部 module の単体試験、mock host、host の適合 suite は作りません。OS の入力を通す確認は [GUI harness](tools/gui-harness.md)（SendInput と UI Automation）で行います。
- E2E は公開 API の試験（`dev/tests/kivact-public-api/tests/e2e/`・公式3kitの `tests/`・アプリの `tests/`）に、確かめる振る舞いのファイルへ足します。`dev/tests/kivact-public-api/tests/e2e/` は1つの試験 binary（`main.rs` と振る舞いごとの module。共有の `mount` などは `support`）で、新しい振る舞いは module を足して `main.rs` に並べます（`tests/` 直下に試験のファイルを足すと、kivact・Arvane・wgpu を link する binary が1つ増えます）。1つの試験で関連する確かめを続けて書いてかまいません。公開 API で作れない場面（`Screen` の内部や Arvane の試験 driver が要るもの）だけ、`kivact-core` の lib の `#[cfg(test)]` の `Screen` の試験（`crates/kivact-core/src/test/screen/*_tests.rs`）に置きます。そこでは公式kitを使えません。
- Arvane の上面（`arvane` facade）の契約の試験は `crates/arvane/tests/e2e/`（1つの試験 binary）の、確かめる部分の module に足します。Kivact の crate を使わず、facade と `arvane::test_display` の試験の window だけで書きます（Kivact の試験を通してだけ Arvane を確かめない）。試験の window で起こせない platform の振る舞いが要るときは、native の window と同じ規則を通る口を試験の window に足します。
- ほかに書いてよい試験は、trybuild のコンパイル試験（公開 API の使い勝手と誤用）、docs と公開 item の rustdoc の例、release の性能の試験（`#[ignore]`）、xtask の試験です。
- 例外として、window を開かない `kivact::draw::Offscreen` の試験だけは `dev/tests/kivact-public-api/tests/offscreen.rs` の専用の試験 binary に置きます（canvas の window と比べる試験は e2e に置きます）。
- 試験の window を使う試験 binary（`kivact::test` を使うもの）では `App::run` を呼びません（1 process に winit の event loop は1つ）。
- 押せないはずの要素への press は `force_click` で明示します。
- 試験は Windows と Linux の両方で通します。その platform にしかない振る舞い（Windows の IME の再変換、`WM_` の message など）や、platform が持つとは限らない資源（color emoji の font など）に依る試験だけを `#[cfg(...)]` で限り、理由を doc comment に書いて [設計](design.md) の10章の表に載せます。`#[ignore]` で黙って外しません。
- engine の振る舞い（[設計](design.md) の6章の契約と9章の資源の規則）は、engine の本番の経路を通る E2E で確かめます。既知の失敗は `#[ignore = "理由"]` に書き、暗黙にしません。
- engine と core の差分の計算（layout、paint の一覧、編集の行、accessibility）は、debug build で「全部作り直した結果」と比べる自己検査を持ってよい。自己検査は試験の window の settle で走り、release には入らない。
- バグ修正には、修正前に落ちる回帰試験を付けます。

## commit と PR

- 1つの commit には目的を1つだけ入れる（リファクタと振る舞いの変更、fmt のような機械的な変更を分ける）。message は conventional commits（`feat:`・`fix:`・`refactor:`・`docs:`・`chore:` ..）。
- 「なぜ」は commit message と PR の説明に書く。設計の判断と退けた案は PR の説明に残し、設計が変わったら [設計](design.md) を同じ PR で直す。
- merge の前に上の表の検査をすべて通す。
