# 公式 Kivact UI キットの作り方（kivact-ui-soft のための基礎資料）

この文書は、新しい公式キット `crates/kivact-ui-soft`（Meta の React デザインシステム「Astryx」の neutral テーマを再現する）を作るエージェントのために、このリポジトリで公式キットがどう作られているかを、コードと rustdoc を根拠にまとめたものです。`crates/kivact-ui-basic` が shadcn/ui（Lyra preset）を再現しているやり方を手本にします。

- 調査対象: `C:\w\kashiwade\kivact`、branch `feat/kashiwade/add-soft-ui-component`（2026-11-03 時点）。
- 表記: `path:line` はその時点の行番号です。**[確認済み]** はコードで確かめた事実、**[推測]** は根拠が間接的なもの、**[未確認]** は調べていない、または調べても分からなかったものです。印のない記述はコードで確かめたものです。
- 正はコードと rustdoc です。README とコメントがコードとずれている箇所は 15 章にまとめました。

---

## 0. いちばん大事な制約（先に読む）

1. **キットは公開 facade の `kivact` だけに依存する。** 内部 crate（`kivact-core`、`kivact-style`、`kivact-ui`、`kivact-interaction`、engine など）に直接依存しない。`kivact::ui`・`kivact::interaction`・`kivact::style`・`kivact::draw`・`kivact::motion`・`kivact::control`・`kivact::primitive` などはすべて `kivact` の再公開です（`crates/kivact/src/lib.rs:37-59`）。
2. **標準部品をコピーしない。** button・checkbox・switch・text_input・select・menu・dialog・tooltip・tabs・radio_group・slider・toast・split・progress・disclosure・list・data_table・command_palette・tag_input・number_input などは `kivact::ui` のものを使い、キットはテーマ（`UiRecipes` の recipe 表）で見た目だけを差し替える（`crates/kivact-ui/src/lib.rs:86-103`）。
3. **キットが持つのは「テーマ」と「分野固有の部品」だけ。** 二つ以上の公式キットが同じ意味の部品を持つなら、それは中核（`kivact::ui` か `kivact::interaction`）へ入れるべきもの（`docs/for-kivact-developer/strategy.md:78`, `:98`）。足りない口があれば中核へ足し、キットで複製しない。
4. **互換性は残さない。** 初期開発版なので alias・互換 wrapper・旧コマンドは作らない（`CONTRIBUTING.md`「判断の基準」）。
5. **state style（hover・pressed・focus・focus_visible・group_hover・group_focus・enter・exit）は paint しか変えられない。** layout と `color` 以外の text の property を state style に書くと `Unsupported` と報告されて無視される（`crates/kivact-style/src/sheet.rs:21-24`, `:763-766`）。`selected` と `disabled` はどの property でも可。
6. **transition で補間されるのは paint の一部だけ。** opacity・単色の background・border の色・transform・最初の二つの shadow・filter の量。**gradient・画像・shape・z-index・text の色・ring は補間されない**（`sheet.rs:150-158`）。
7. **テストは実ウィンドウの E2E（`kivact::test`）だけ。** golden・画素や時間の厳密な比較・過去の不具合の再現だけの試験は追加しない（`CONTRIBUTING.md`「テスト」）。ウィンドウを開くコマンドは `dev/tools/window-lock.sh -- <command>` を通す。

---

## 1. crate の骨格と、五つ目のキットを登録する手順

### 1.1 既存キットの `Cargo.toml`

四つとも同じ形です。

`crates/kivact-ui-basic/Cargo.toml`（全文）:

```toml
[package]
name = "kivact-ui-basic"
authors.workspace = true
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "shadcn/ui (Lyra preset) inspired official Kivact components built solely on the public kivact API"

[dependencies]
kivact = { path = "../kivact", version = "0.1.0", default-features = false }

[dev-dependencies]
kivact = { path = "../kivact", default-features = false, features = ["test"] }

[lints]
workspace = true
```

- `kivact-ui-pop` と `kivact-ui-game` も同じ（description だけ違う）。
- `kivact-ui-creative` だけ `[features] gpu-interop = ["kivact/gpu-interop"]` と、`cfg(any(windows, target_os = "linux"))` の dev-dependency `gpu-interop-producer` を持つ（`crates/kivact-ui-creative/Cargo.toml`）。新キットは feature を持たない想定で良い。
- `default-features = false`: ライブラリは renderer（`gpu` feature）を選ばない。選ぶのはアプリ（`dev/tests/docs/Cargo.toml:11-13` のコメント「Applications choose the renderer; component libraries leave it to their consumer.」）。
- `build.rs` はない。basic の README は「there are no build dependencies」と明記（`crates/kivact-ui-basic/README.md:6-9`）。
- `[lints] workspace = true` は全 member 必須（root `Cargo.toml` のコメント「Every member opts in with `[lints] workspace = true`」）。

root の workspace lint（`Cargo.toml` `[workspace.lints]`）: `unsafe_code = "deny"`, `unreachable_pub = "warn"`, clippy の `mod_module_files = "deny"`（`foo/mod.rs` 禁止、`foo.rs` + `foo/` を使う）, `allow_attributes_without_reason = "deny"`, `unwrap_used = "deny"`, `todo`/`unimplemented`/`dbg_macro`/`print_stdout`/`print_stderr = "deny"`。`clippy.toml` はテストでだけ unwrap・dbg・print を許す（`allow-unwrap-in-tests = true` など）。toolchain は `rust-toolchain.toml` で `1.98.1`、edition 2024。

### 1.2 root `Cargo.toml` の members

`Cargo.toml` `[workspace] members` にキットとカタログが並んでいます。

```text
    "crates/kivact-ui-basic",
    "crates/kivact-ui-pop",
    "crates/kivact-ui-game",
    "crates/kivact-ui-creative",
    ...
    "examples/kivact-ui-basic",
    "examples/kivact-ui-pop",
    "examples/kivact-ui-game",
    "examples/kivact-ui-creative",
```

新キットでは `"crates/kivact-ui-soft"` と `"examples/kivact-ui-soft"` を足します。`Cargo.lock` は cargo が更新します。

### 1.3 `dev/xtask/policy/workspace.toml`

`cargo xtask check` が読む規則です（`dev/xtask/policy/workspace.toml`）。キット関係の項目は次のとおり。

| 行 | 内容 | 新キットで要るか |
|---|---|---|
| `:7` | `max_code_lines = 1000`（1 ファイルの code 行の上限。空行・comment だけの行・文字列 literal の中だけの行は数えない） | 守るだけ |
| `:12-73` `[visible_to]` | crate ごとに「誰が依存してよいか」。**キット crate 自体は列挙されていない**＝誰でも依存できる（`:43-44` のコメント「visible_to は書かない。誰でも依存できる」）。カタログは `kivact-ui-basic-catalog = []`（`:66-69`）＝誰も依存しない | **カタログを `kivact-ui-soft-catalog = []` として足す**。キット crate は書かない |
| `:71` | コメント「Public API tests consume all four official kits for theme conformance.」 | 文言を five に直す |
| `:287-292` `api_packages` | 「facade（kivact）から届く crate と、四つの公式キット」。`"kivact-ui-basic", "kivact-ui-creative", "kivact-ui-game", "kivact-ui-pop"` が並ぶ。将来 `non-exhaustive` などの公開 API 規則の対象 | **`"kivact-ui-soft"` を足す**（コメントも直す） |
| `:294-316` `options_structs` | option の束の struct（`kivact-ui-basic::ChartConfig` など）。`options-default` 規則（FRU で組む option の束）の例外対象 | soft に option 束の struct を作るなら足す |
| `:321-359` `[[style.exceptions]] rule = "glob-imports"` | `examples/*`・`templates/*`・`dev/tests/kivact-public-api/*`・**`crates/kivact-ui-basic/tests/*`**（`:336-339`）・creative・game・pop の `tests/*` で `use kivact::prelude::*` を許す | **`crates/kivact-ui-soft/tests/*` の例外を足す**（`examples/*` は既にワイルドカードで覆われている） |
| `:391-395` | `rule = "non-leaf-implementation"`, `path = "examples/kivact-ui-basic/src/docs.rs"`, `item = "docs::PAGES"`（ページ登録だけの glue） | soft のカタログが同じ `docs.rs` + `PAGES` の形なら、同じ例外を `examples/kivact-ui-soft/src/docs.rs` に足す |

`[style]`（`:270-282`）: `extra_rules = ["glob-imports", "non-leaf-implementation", "rustfmt-skip", "catch-unwind", "poison-access", "options-default"]`、`disabled_rules = ["missing-docs"]`。例外は「Google の指針自体が認める場合」だけ。件数を減らすための例外、規則を逃れるための module 分けは禁止（`CONTRIBUTING.md`「コード」）。

`[module_rules.*]` はキット crate には定義されていません（core・i18n・motion・style・interaction・arvane-shell・engine・types だけ）。キット内部の module の向きは自由ですが、「子 module を持つ module は mod 宣言と再公開だけ」の規則（`non-leaf-implementation`）は全 crate に掛かります。

`vendor/xtask-core` は git submodule です（`.gitmodules`）。この worktree では空ディレクトリでした **[確認済み]**。`cargo xtask check` の前に `git submodule update --init` が要ります。図の生成には Node.js が要ります。

### 1.4 golden・public-names・kit-naming について

- 依頼文にある `dev/tests/workspace-policy` は**存在しません**。`dev/tests/` は `docs`・`gpu-interop-producer`・`kivact-public-api` の三つだけです **[確認済み]**。
- `public-names.golden.md` などの golden、`public-names`・`kit-naming` という名前の文書・policy は**リポジトリに一つもありません**（`git grep -i "golden\|public-names\|kit-naming"` で該当なし。golden という語は arvane の試験コメントと id_hash の定数だけ）**[確認済み]**。
- CONTRIBUTING は golden の追加を禁止しています。したがって「再生成が要る golden」はありません。

### 1.5 `dev/tests/kivact-public-api`（公開 API の E2E）

新キットを足すと**変更が要ります**。

- `dev/tests/kivact-public-api/Cargo.toml:21-26`: `[dev-dependencies]` に `kivact-ui-basic`・`-game`・`-pop`・`-creative` を path で並べている。`kivact-ui-soft = { path = "../../../crates/kivact-ui-soft" }` を足す。
- `dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs`（**全キットのテーマが標準部品の契約を守るかの試験**）:
  - `:156` `const KIT_RECIPES: [(&str, &[&str]); 4] = [...]`: キットごとに「recipe を差し替えている部品名」。配列長が型に入っているので `5` にして `("Soft", &["Save", "Check", "Switch", "Mode", "Choice", "Sections", "Editor", "Field", "Level", "Progress", "Details", "Items"])` を足す。
  - `:262-267` `let themes = [("Basic", kivact_ui_basic::theme as fn(ThemeMode) -> Theme), ("Game", ..), ("Pop", ..), ("Creative", ..)]`: `("Soft", kivact_ui_soft::theme)` を足す。**つまり soft の `theme` は `fn(ThemeMode) -> Theme` の形でなければならない。**
  - この試験が要求すること（キットが満たさないと落ちる）:
    1. `kit_recipes_paint`（`:230-255`）: 12 部品（button "Save"、checkbox、switch、radio_group、select、tabs、text_input、field+text_input、slider、progress、disclosure、list）それぞれについて、その部品の recipe だけ既定に戻した描画と比べて**画素が違う**こと。すなわち soft はこの 12 個すべての recipe を差し替える必要がある。
    2. `focus_pixels`（`:19-58`）: Tab で button に focus したとき、button の上端の外側 `theme.focus_ring()` 幅の位置に `theme.colors().focus`（alpha 合成込み、許容差 8）が描かれ、その外側 2px は変わらないこと。**button の focus ring は「`Theme::focus_ring()` の幅で `ColorToken::Focus`（= `Palette.focus`）の色、border box の外側」でなければならない**。basic は `with_focus` → `theme_ring()` = `style().ring(current_theme().focus_ring(), ColorToken::Focus)` でこれを満たす（`crates/kivact-ui-basic/src/theme_access.rs:17-19`, `:25-27`, `:36-38`）。Astryx の focus ring がオフセット付きでも、少なくとも標準 button はこの契約に合わせる必要がある。
    3. 各部品の role・name・checked・disabled・expanded・selected・value の契約、`screen.assert_layout()`（レイアウト崩れなし）、`screen.errors()` が空。`reduced_motion: true`、Inter フォント、720x960。
- `ui_standard.rs:313-345` は Pop だけを使う試験（高さ 36、Pop の青）。新キットのために足す必要はない。
- `layout_check.rs:18-20` は `examples/kivact-ui-pop/assets/fonts/notosansjp/NotoSansJP[wght].ttf` を include している（フォントの場所に依存するだけで、キット登録とは無関係）。

### 1.6 `dev/tests/docs`（ドキュメントの doctest）

- `dev/tests/docs/Cargo.toml:16-20`: `[dependencies]` に四キットを並べている。soft のドキュメントの Rust ブロックが `kivact_ui_soft` を使うなら `kivact-ui-soft = { path = "../../../crates/kivact-ui-soft" }` を足す（足さないと doctest がコンパイルできない）。
- 仕組みは 11 章。

### 1.7 `architecture/`（生成物）

- `cargo xtask check` が `architecture/crates.svg`、`architecture/index.html`、`architecture/modules/<package>/<target>-<kind>-<profile>.svg` を生成します（`dev/xtask/policy/workspace.toml:3` `output = "architecture"`, `:10` `targets = "lib-bin"`）。
- 既存の例: `architecture/modules/kivact-ui-basic/kivact_ui_basic-lib-normal.svg`・`-lib-test.svg`、`architecture/modules/kivact-ui-basic-catalog/kivact-ui-basic-catalog-bin-normal.svg` など。
- 図の node の説明文は crate root の `//! Role:` の一文です（例: `architecture/crates.svg:739` の「Pop components for calm, compact desktop interfaces.」は `crates/kivact-ui-pop/src/lib.rs:1`）。
- 手で編集しない。新キットを足したら `cargo xtask check` を実行して再生成し、生成物を commit する。merge 前の検査に `git diff --quiet -- architecture` がある（生成し直して差分がないこと）。CI は `cargo xtask check --no-write`（SVG を描かずに構造だけ検査）。

### 1.8 `dev/tools/check-features.sh`

library の非既定 feature を一つずつ有効にしてコンパイルする script。`entries=( ... kivact-ui-creative/gpu-interop )` のように `crate/feature` を列挙（`dev/tools/check-features.sh:20-44`）。**新キットに feature がなければ変更不要**。feature を足すなら entry を足す（「Add an entry here when a crate gains a feature.」）。

### 1.9 CONTRIBUTING.md のコード規則（キット作成に関わるもの）

- 子 module を持つ module は `mod` の宣言と再公開だけ。実装は役割を一つに絞った葉の module に置く（xtask の `non-leaf-implementation`）。basic の例: `src/calendar.rs` は `mod caption; mod panel; mod state; mod view;` と `pub use` だけ、実装は `src/calendar/*.rs`。`foo/mod.rs` は clippy `mod_module_files` で禁止。
- 全 crate root と全 module（inline の `mod {}` を含む、テストファイルも）の先頭に `//! Role: <英語の 1 文>`。xtask が図に出す。例: `crates/kivact-ui-basic/src/theme.rs:1` `//! Role: Install the Lyra Neutral palette and complete standard component recipes.`
- コメントと rustdoc は英語。
- glob import 禁止（例・雛形・利用者側 E2E の `kivact::prelude::*` だけ例外。キットの `src/` では書けない）。
- library で `unwrap()` 禁止。避けられない前提は `expect("invariant: …")`。lint の `allow` には必ず `reason`。
- panic は不具合。`catch_unwind` で隔離しない。毒された lock に触れない。
- `unsafe` はキットでは使わない（`unsafe_code = "deny"`）。
- feature はその crate の振る舞いだけを変える（他 crate が match する enum の variant や組み立てる struct の field を増減しない）。
- 公開 item の rustdoc の有無（`missing-docs`）と `#[non_exhaustive]` は今は検査しない。ただし既存キットは公開 item にほぼ全部 rustdoc を書いている。
- 1 ファイルの code 行は 1000 まで。basic の最大は `src/sidebar/menu.rs` 606 行、`src/recipes_controls.rs` 489 行。recipe を `recipes.rs`・`recipes_popup.rs`・`recipes_controls.rs`… と分けているのはこの上限と役割分割のため。
- 公開 API と依存の向きを変える変更は、同じ変更で rustdoc・`workspace.toml`・利用者向けドキュメントを更新する。
- 英語と日本語のドキュメントは同じファイル名で対にし、同じ変更で両方を更新する。

### 1.10 `kivact-ui-pop` / `kivact_ui_pop` / `kivact-ui-game` の全参照と意味

`git grep -I -E "kivact-ui-pop|kivact_ui_pop"`（`Cargo.lock`、`crates/kivact-ui-pop/`、`examples/kivact-ui-pop/src/` を除く）の結果と、basic・game・creative の参照ファイル一覧を合わせた分類です。**ここに挙がった場所が「キットを足すと触る場所」の全集合です**（`Cargo.lock` と生成物を除く）。

| 場所 | 中身 | soft で必要な対応 |
|---|---|---|
| `Cargo.toml:11-14`, `:28-31` | workspace members | crate とカタログを足す |
| `README.md:47` | 「**Four** official kits restyle the standard components…」 | Five に直す |
| `README.md:89-95` | Crates 表（`kivact-ui-basic` Neutral general-purpose kit…, `kivact-ui-pop` Calm business-application kit. …） | `kivact-ui-soft` の行を足す |
| `README_JP.md:36` | 「4 つの公式キットが…」 | 5 つに直す |
| `README_JP.md:76-82` | Crate 表（日本語） | 行を足す |
| `THIRD-PARTY-NOTICES.md:10-12` | 「Tabler icons in Pop/Game/Creative」「Lucide icons in Basic」「Catalog font families」 | soft の icon の出所とライセンス、カタログのフォントを足す |
| `architecture/crates.svg`, `architecture/index.html`, `architecture/modules/kivact-ui-pop*/…` | 生成物 | `cargo xtask check` で再生成 |
| `dev/tests/docs/Cargo.toml:16-20` | doctest crate の依存 | soft を足す |
| `dev/tests/kivact-public-api/Cargo.toml:21-26` | E2E の dev-dependency | soft を足す |
| `dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs:156`, `:262-267` | 全キットのテーマ適合試験（1.5 参照） | `KIT_RECIPES` の長さと中身、`themes` に足す |
| `dev/tests/kivact-public-api/tests/e2e/ui_standard.rs:320`, `:334` | Pop を使う試験 | 不要 |
| `dev/tests/kivact-public-api/tests/e2e/layout_check.rs:19` | Pop カタログのフォントを include | 不要 |
| `dev/xtask/policy/workspace.toml:66-69` | カタログの `visible_to = []` | `kivact-ui-soft-catalog = []` |
| `dev/xtask/policy/workspace.toml:71` | 「all four official kits」のコメント | 直す |
| `dev/xtask/policy/workspace.toml:287-292` | `api_packages` | soft を足す |
| `dev/xtask/policy/workspace.toml:336-354` | `crates/kivact-ui-*/tests/*` の glob-imports 例外 | soft を足す |
| `dev/xtask/policy/workspace.toml:391-395` | basic カタログの `docs::PAGES` の non-leaf 例外 | soft カタログが同じ形なら足す |
| `dev/tools/check-features.sh:43` | `kivact-ui-creative/gpu-interop` だけ | feature がなければ不要 |
| `docs/for-kivact-ui-user/en/README.md:17-22` | 「Choose a kit」表 | 行を足す |
| `docs/for-kivact-ui-user/en/README.md:27-32` | Tutorial・Guides・catalog コマンドの表 | 行を足す |
| `docs/for-kivact-ui-user/en/README.md:48-71` | 例（`kivact_ui_pop::theme`）と「Replace `kivact_ui_pop` with `kivact_ui_basic`, `kivact_ui_game` or `kivact_ui_creative`…」 | 文に soft を足す |
| `docs/for-kivact-ui-user/en/README.md:78-83` | `cargo doc -p kivact-ui-pop --open`「Use `kivact-ui-basic`, `kivact-ui-game` or `kivact-ui-creative`…」 | soft を足す |
| `docs/for-kivact-ui-user/jp/README.md:14`, `:23`, `:51`, `:56`, `:63` | 上の日本語版 | 同じ |
| `docs/for-kivact-ui-user/{en,jp}/pop/**` | Pop 自身のガイド | 不要（soft 用に `soft/` を新設） |
| `docs/for-kivact-user/en/README.md:59` | 「The **four** official component kits (`kivact-ui-basic`, `kivact-ui-pop`, `kivact-ui-game`, `kivact-ui-creative`)」 | 直す |
| `docs/for-kivact-user/jp/README.md:50` | 「公式の 4 つのコンポーネントキット (…)」 | 直す |
| `docs/for-kivact-user/en/how-to-guide/theming.md:29-31` | 「Every kit exposes `theme(mode: ThemeMode)`: `kivact_ui_basic::theme`, `kivact_ui_pop::theme`, …」 | soft を足す |
| `docs/for-kivact-user/jp/how-to-guide/theming.md:24` | 同じ（日本語） | 同じ |
| `examples/README.md:10` (表 `:7-12`) | カタログ一覧（Basic・Pop・Game・Creative と `cargo run -p kivact-ui-*-catalog`） | 行を足す |
| `examples/kivact-ui-pop/Cargo.toml`, `README.md`, `tests/catalog.rs:11` | Pop カタログ自身 | soft カタログを新設 |
| `crates/kivact-core/src/gpu_interop.rs` | creative だけ（GPU 共有の説明） | 不要 |

`templates/basic` はキットを参照していません（アプリの雛形で、名前の basic はキットと無関係）**[確認済み]**。`.github/workflows/ci.yml` と `.github/CODEOWNERS` もキット名を持ちません。

### 1.11 五つ目のキットを足す手順（チェックリスト）

1. `crates/kivact-ui-soft/` を作る: `Cargo.toml`（1.1 の形）、`README.md`、`src/lib.rs`（`//! Role:` と crate doc、`mod` と `pub use` だけ）、`src/theme.rs`、`src/tokens.rs`、`src/color.rs`、`src/theme_access.rs`、`src/recipes*.rs`、`src/icons.rs`、`assets/icons/*.svg` と icon のライセンスファイル、`tests/consumer.rs` と `tests/consumer/*.rs`。
2. root `Cargo.toml` の members に `crates/kivact-ui-soft` と `examples/kivact-ui-soft` を足す。
3. `examples/kivact-ui-soft/` を作る（package 名 `kivact-ui-soft-catalog`、10 章）。
4. `dev/xtask/policy/workspace.toml`: `kivact-ui-soft-catalog = []`、`api_packages` に `kivact-ui-soft`、`crates/kivact-ui-soft/tests/*` の glob-imports 例外、必要ならカタログの `docs::PAGES` の non-leaf 例外。「four」のコメントを直す。
5. `dev/tests/kivact-public-api/Cargo.toml` と `theme_conformance.rs`（`KIT_RECIPES` 長さ 5、`themes` に Soft）。
6. `dev/tests/docs/Cargo.toml` に依存を足す。
7. `docs/for-kivact-ui-user/{en,jp}/soft/getting-started/` と `how-to-guide/` を作り、`docs/for-kivact-ui-user/{en,jp}/README.md` の二つの表と本文を更新。
8. `README.md`・`README_JP.md`・`docs/for-kivact-user/{en,jp}/README.md`・`docs/for-kivact-user/{en,jp}/how-to-guide/theming.md`・`examples/README.md`・`THIRD-PARTY-NOTICES.md` を更新。
9. `cargo xtask check` で `architecture/` を再生成して commit。
10. 13 章の merge 前の検査を通す。

---

## 2. テーマの仕組み: テーマ・tokens・recipe

### 2.1 `basic::theme(ThemeMode)` の全体

`crates/kivact-ui-basic/src/theme.rs:14-69`（抜粋）:

```rust
pub fn theme(mode: ThemeMode) -> Theme {
    let p = Tokens::new(mode);
    let dark = mode == ThemeMode::Dark;
    let base = Theme::light();
    let colors = Palette {
        surface: p.background,
        surface_alt: p.muted,
        text: p.foreground,
        text_muted: p.muted_foreground,
        accent: p.primary,
        accent_text: p.primary_foreground,
        border: p.border,
        focus: p.ring.with_alpha(0.5),
        danger: p.destructive,
        success: rgb(if dark { 0x4ade80 } else { 0x16a34a }),
        warning: rgb(if dark { 0xfbbf24 } else { 0xd97706 }),
        overlay: Color::BLACK.with_alpha(0.1),
        shadow: Color::BLACK.with_alpha(0.1),
        selection: p.primary.with_alpha(0.15),
        hover: p.muted,
        pressed: p.secondary.mix(p.foreground, 0.05),
        raised: p.popover,
    };
    // Spacing { xs: 4, sm: 8, md: 12, lg: 16, xl: 24 }, Radii { sm: 0, md: 0, lg: 0 },
    // Typography { size: 12, size_small: 12, size_large: 14, weight: 400,
    //              weight_strong: 500, line_height: LineHeight::Scale(16/12), ..base.text().clone() }
    base.with_mode(mode)
        .with_colors(colors)
        .with_space(space)
        .with_radius(radius)
        .with_text(text)
        .with_focus_ring(1.0)
        .with_control_height(32.0)
        .with_icons(crate::icons::set())
        .with(p)
        .with(crate::recipes::recipes())
}
```

テーマは三層です（`crates/kivact-ui/src/lib.rs:40-73`）。

1. **core の token 表**: `Palette`（`kivact::style::theme::Palette`、17 色: `surface, surface_alt, text, text_muted, accent, accent_text, border, focus, danger, success, warning, overlay, shadow, selection, hover, pressed, raised`）、`Spacing`（xs..xl）、`Radii`（sm, md, lg）、`Typography`（`family: Option<String>, size, size_small, size_large, weight, weight_strong, line_height`）、`focus_ring`（px）、`disabled_opacity`、`control_height`、`IconSet`。`ColorToken::Surface` などで style から参照でき、標準部品の既定 recipe もこれを読む。kit は `Hover`・`Pressed`・`Raised` を自分の色で埋め、同じ意味の独自 token を持たない（`crates/kivact-style/src/theme/tokens.rs:6-19`）。
2. **キットの typed tokens**（`ThemeExtension`）: `Theme::with(p)` で入れる。
3. **recipe 表**（`UiRecipes`、これも `ThemeExtension`）: `Theme::with(recipes)` で入れる。

`Theme` の API（`crates/kivact-style/src/theme/value.rs:171-336`）: `with<T: ThemeExtension>(self, T)`, `get<T: ThemeExtension>(&self) -> Arc<T>`（無ければ `T::fallback(self)`）, `mode()`, `colors()`, `space()`, `radius()`, `text()`, `focus_ring()`, `disabled_opacity()`, `control_height()`, `icons()`, `with_mode`, `with_colors`, `with_space`, `with_radius`, `with_text`, `with_focus_ring`, `with_disabled_opacity`, `with_control_height`, `with_icons`, `with_icon(name, image)`, `Theme::light()`, `Theme::dark()`。`current_theme()` は style closure の中で使い、tracked（テーマが変わると style が再評価される）。

`ThemeMode` は `Light`・`Dark` の二値（`value.rs:158-163`）。rustdoc に「Every kit's theme constructor takes it first (`theme(ThemeMode::Dark, ..)`)」とある。

**キットのテーマの決まり**（`crates/kivact-ui/src/lib.rs:86-103` の `# Kits`）:
- キットは標準部品を再実装も再公開もしない。テーマと分野固有の部品を供給する。
- 公式キットのテーマは**すべての標準部品の recipe を持つ**。
- テーマは一つの関数 `theme(mode: ThemeMode, ..)`。
- tokens は `ThemeExtension` を実装する一つの型で、`fallback` はテーマに従う。`Theme::with` で入れ、`theme.get::<Tokens>()` だけで読む。
- 色は field なしの `Color` enum（`ThemeColor` 実装）でも名指せる（`basic::Color::Border`）。
- icon は `Theme::with_icon`（basic は `with_icons(IconSet)`）。
- token 型とテーマ組み立て関数を分け、recipe が tokens を読むときに組み立て関数へ依存し返さない。

**同一性**: basic の recipe 表と icon set は `OnceLock` で一度だけ作って共有し、同じテーマを二回作っても `==` になる（`crates/kivact-ui-basic/src/recipes.rs:18-26`, `icons.rs:275-308`、試験 `tests/consumer/foundation.rs:51-55`）。recipe 表は `Arc::ptr_eq` で比較されるので、毎回新しく作るとテーマ切替のたびに全部が restyle される。

### 2.2 typed `Tokens`

`crates/kivact-ui-basic/src/tokens.rs:12-72`: `pub struct Tokens { pub mode: ThemeMode, pub background, foreground, card, card_foreground, popover, popover_foreground, primary, primary_foreground, secondary, secondary_foreground, muted, muted_foreground, accent, accent_foreground, destructive, border, input, ring: Color, pub chart: [Color; 5], pub sidebar..sidebar_ring, pub code: CodeColors }`。field は shadcn の CSS 変数と一対一（rustdoc に `--background` などを書く）。`#[derive(Clone, Debug, PartialEq)]`。

- `Tokens::new(mode)` が Tailwind v4 Neutral の値を返す（`:120-174`、`pick(light, dark)` の closure）。
- `is_dark()`（`:176-179`）。
- `impl ThemeExtension for Tokens { fn fallback(theme: &Theme) -> Self { Self::new(theme.mode()) } }`（`:182-186`）。他キットのテーマの下でも basic の部品が描けるようにするため。
- アプリは `basic::Tokens` を書き換えて `Theme::with` で入れ直せる（README `:50-54`）。

`ThemeExtension` の定義（`crates/kivact-style/src/theme/value.rs:67-73`）:

```rust
pub trait ThemeExtension: PartialEq + std::fmt::Debug + Send + Sync + 'static {
    fn fallback(theme: &Theme) -> Self;
}
```

### 2.3 キットの `Color` enum（`ThemeColor`）

`crates/kivact-ui-basic/src/color.rs:13-95`: field なしの `pub enum Color { Background, Foreground, Card, …, Chart1..Chart5, CodeKeyword, … }`、`impl ThemeColor for Color { fn resolve(self, theme: &Theme) -> kivact::style::Color { let p = theme.get::<Tokens>(); match self { Color::Background => p.background, … } } }`。

`ThemeColor`（`value.rs:470-481`）は `Copy + PartialEq + Debug + Send + Sync + 'static`、`fn resolve(self, theme: &Theme) -> Color`。**実装は field なしの小さな値型にすること**（「the process keeps one shared copy of each distinct value, so a value that is built from a number (`Gray(f32)`) would grow that table without bound」）。style のすべての色入力（`background`・`border`・`color`・`Shadow::new`・gradient の stop・`Draw` の塗り）が `impl ThemeColor` を取るので、`style().background(basic::Color::Background)` は closure なしでテーマ切替に追従する。

### 2.4 部品が tokens を読む方法（`theme_access.rs`）

`crates/kivact-ui-basic/src/theme_access.rs`（crate 内部の共有部品）:

| 関数 | 行 | 役割 |
|---|---|---|
| `tokens() -> Arc<Tokens>` | `:8-10` | `current_theme().get::<Tokens>()`。style closure の中で呼ぶ |
| `theme_ring() -> Style` | `:17-19` | `style().ring(current_theme().focus_ring(), ColorToken::Focus)` |
| `focus_ring(p) -> Style` | `:25-27` | `theme_ring().border_color(p.ring)` |
| `wide_focus_ring(p)` | `:31-33` | `ring-[3px]`（tabs・toggle・badge） |
| `with_focus(s, p)` | `:36-38` | `s.focus_visible(focus_ring(p))` |
| `disabled()` | `:45-47` | `style().opacity(current_theme().disabled_opacity())` |
| `lit_row(p)`, `list_row(p, lit)`, `menu_shortcut(p, lit)` | `:51-121` | popup の行 |
| `field_fill(p)` | `:56-62` | `dark:bg-input/30` |
| `shadow_md(s)`, `shadow_lg(s)` | `:64-76` | Tailwind の shadow を `Shadow::new(..).spread(..)` 二枚で |
| `popover_surface(p)` | `:79-86` | 浮く面 |
| `Glyph`, `CHECK`, `MINUS`, `CHEVRON_DOWN`, `stroke_glyph`, `stroke_loader` | `:123-172` | Lucide の線を `Draw` で描く（check box の印、busy の spinner） |

キット部品の style は二通り:
- **recipe の中**: `fn button(t: &Theme, state: &Look<ButtonState>) -> Style { let p = t.get::<Tokens>(); … }`（テーマが引数で来る）。
- **キット部品の style closure の中**: `root.base_style(move || badge_style(&tokens(), variant, actionable, &state.get(), padding))`（`crates/kivact-ui-basic/src/badge.rs:207`）。`tokens()` が `current_theme()` を tracked で読むので、テーマ切替で再評価される。

### 2.5 recipe の登録 API

型（`crates/kivact-ui/src/recipes.rs:73`）:

```rust
pub type Recipe<S> = Arc<dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync>;
```

表は macro `recipe_table!`（`recipes.rs:81-164`）で生成され、field は private。part ごとに二つの method があります。

- 置換: `pub fn <part><F>(self, f: impl FnOnce(Recipe<S>) -> F) -> Self where F: Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static`。`f` は**現在の recipe を受け取り、新しい recipe を返す**（包むことも捨てることもできる）。
- 読み出し: `pub fn <part>_recipe(&self) -> &Recipe<S>`。子の表は `<table>(self, f: impl FnOnce(Table) -> Table)` と `<table>_recipes(&self) -> &Table`。
- `PartialEq` は各 recipe の `Arc::ptr_eq`。`Default` は core の既定 look（`look*.rs`）。
- `impl ThemeExtension for UiRecipes { fn fallback(_) -> Self { Self::default() } }`（`:540-544`）。

basic の登録（`crates/kivact-ui-basic/src/recipes.rs:23-59`）:

```rust
pub(crate) fn recipes() -> UiRecipes {
    static RECIPES: std::sync::OnceLock<UiRecipes> = std::sync::OnceLock::new();
    RECIPES.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    UiRecipes::default()
        .field(crate::recipes_field::field)
        .button(|_| button)
        .button_content(|_| button_content)
        .button_busy(|_| button_busy)
        .button_icon(|_| button_icon)
        .toggle(|_| crate::recipes_controls::toggle)
        .checkbox(|_| checkbox)
        .switch(|_| switch)
        .text_input(|_| text_input)
        .text_area(|_| text_area)
        .select(crate::recipes_popup::select)
        .menu(crate::recipes_popup::menu)
        .dialog(crate::recipes_dialog::dialog)
        .tooltip(|tooltip| {
            tooltip
                .surface(|_| crate::recipes_popup::tooltip)
                .arrow(|_| crate::recipes_popup::tooltip_arrow)
        })
        .tabs(crate::recipes_popup::tabs)
        .radio_group(crate::recipes_controls::radio_group)
        .slider(crate::recipes_controls::slider)
        .toast(crate::recipes_controls::toast)
        .splitter(crate::recipes_controls::splitter)
        .list(crate::recipes_collections::list)
        .data_table(crate::recipes_collections::data_table)
        .split(crate::recipes_controls::split)
        .progress(crate::recipes_controls::progress)
        .disclosure(crate::recipes_controls::disclosure)
        .command_palette(crate::recipes_command::command_palette)
}
```

子の表は `fn(SelectRecipes) -> SelectRecipes` の関数で差し替えます（`recipes_popup.rs:16-34`）:

```rust
pub(crate) fn select(r: SelectRecipes) -> SelectRecipes {
    r.trigger(|_| select_trigger)
        .surface(|_| {
            |t: &Theme, _: &kivact::ui::Look<PopupState>| {
                surface_style(&t.get::<Tokens>()).min_width(144).max_height(320)
            }
        })
        .option(|_| |t: &Theme, s: &kivact::ui::Look<ListItemState>| option_row(&t.get::<Tokens>(), s))
        .check(|_| |_: &Theme, _: &kivact::ui::Look<ListItemState>| check_box())
        .indicator(|_| select_indicator)
        .description(|_| |_: &Theme, _: &kivact::ui::Look<ListItemState>| style().color(crate::Color::MutedForeground))
}
```

注意: **basic は `tag_input` と `number_input` の表を差し替えていません**（`recipes.rs:29-58` にない）。pop と game は差し替えています（`crates/kivact-ui-pop/src/recipes.rs:62-63`、`crates/kivact-ui-game/src/recipes.rs:120-121`）。lib.rs の「Every theme of the official kits carries recipes for every standard component」とずれている（15 章）。soft は全部差し替えるのが安全です。

アプリが一部だけ変える例（`recipes.rs:490-500` の rustdoc）:

```rust
let recipes = theme.get::<UiRecipes>();
let square = UiRecipes::clone(&recipes)
    .button(|prev| move |theme, state| prev(theme, state).radius(0))
    .menu(|menu| menu.item(|prev| move |theme, state| prev(theme, state).padding_x(16)));
let theme = theme.with(square);
```

### 2.6 recipe の規則

`crates/kivact-ui/src/recipes.rs:1-46` と `lib.rs:53-73`:

- recipe は **style だけを返す**。view を返さない。item が何を見せるかは部品の slot（`Tabs::item`、`Menu::item`、`Select::option`、`List::item`、`RadioGroup::item`）。
- hover・press・focus・disabled の見た目は、返す style の state variant（`.hover(..)`, `.pressed(..)`, `.focus(..)`, `.focus_visible(..)`, `.disabled(..)`）で書き、`state.input` で分岐しない。**例外は text decoration**（state style で変えられないので `state.input.hovered` を読む。basic の Link の下線、disclosure の `hover:underline`）。
- 局所の描画は recipe の `Style::paint`。それを持つためだけの子 node を足さない。構造上の部品（icon の箱、busy の indicator、menu の印）は自分の node と recipe を持つ。
- root の style の重なり順: 一行の text 設定 → recipe → 呼び出し側の `.style(..)`（呼び出し側が勝つ）。描画順: 呼び出し側の `.draw` → recipe の paint → 子 → 呼び出し側の `.draw_over`。merge は設定していない paint を残す。
- 印（mark）はテーマの `IconSet` の icon で、部品が recipe の箱に入れる。recipe は描かない: `check`（menu の check item、select の選択中）、`dot`（radio）、`menu-radio`、`chevron-right`（submenu）、`chevron-down`（select）、`x`（dialog と toast の close）、`toast-info` など。印が要らない look は箱の大きさを 0 にする。
- **native text field（text_input・text_area・tag/number/palette の入力）は `Style::paint` をサポートしない**（`lib.rs:69-70`）。style property は受け取る。

### 2.7 recipe が受け取る state

`crates/kivact-ui/src/values.rs` と `kivact-interaction`。すべての state は `input: PartState` を持つ（またはそれに準ずる）。

`PartState`（`crates/kivact-core/src/view/part.rs:75-91`）: `pressed`, `hovered`, `focused`, `focus_visible`, `disabled`, `hidden`（すべて `bool`）。

| state 型 | 定義 | 主な field | 使う part |
|---|---|---|---|
| `ButtonState` | `values.rs:82-108` | `input`, `size: ControlSize`, `loading`, `expanded: Option<bool>`（popup の trigger なら `Some(open)`）, `leading_icon`, `trailing_icon`, `icon_only` | `button`, `button_content`, `button_icon`, `button_busy` |
| `ToggleButtonState` | `:130-144` | `input`, `pressed`（on）, `size`, `leading_icon`, `icon_only` | `toggle` |
| `CheckState` | `:209-226` | `input`, `checked: CheckedState`（Unchecked/Checked/Indeterminate）, `size`, `invalid`, `text: Option<Value<String>>`（switch の ON/OFF 文言） | `checkbox`, `switch`, radio の `item`/`indicator`/`description` |
| `TextInputState` | `:193-199` | `input`, `size`, `invalid` | `text_input`, `text_area` |
| `SelectTriggerState` | `:262-273` | `input`, `open`, `placeholder`, `size` | select `trigger` |
| `PopupState`（interaction） | `kivact-interaction/src/popup.rs:51-54` | `input`, `open` | select の `surface`/`indicator`、menu の `trigger`/`surface`/`sub_surface`/`context_surface`、tooltip `surface`、palette `panel` |
| `ListItemState`（interaction） | `list/data.rs:301-305` | `input`, `selected`, `active` | select `option`/`check`/`description`、list `item`/`description`、palette `row`/`icon`/`hint` |
| `MenuItemState` | `menu/state.rs:55-64` | `input`, `check: Option<MenuCheck>`, `expanded`（submenu が開いている）, `destructive` | menu `item`/`shortcut`/`description`/`check`/`submenu` |
| `DialogState` | `values.rs:237-245` | `input`, `open`, `kind: DialogKind`（Modal/AlertModal/Drawer(Side)/Popover）, `media` | dialog の全 part |
| `TooltipArrowState` | `:248-259` | `side: Side`, `offset: f32` | tooltip `arrow` |
| `TabListState` / `TabState` | `tabs.rs:43-57` | `input`, `orientation`（TabState は `selected` も） | tabs `list` / `tab`・`panel` |
| `Orientation` | core | 方向そのもの | radio `group` |
| `AxisState` | `values.rs:277-289` | `input`, `orientation`, `size` | slider `track`/`range`/`thumb`、splitter `handle`、split `grip` |
| `ToastState` | `toaster.rs:361-371` | `input`, `tone: Tone`, `engaged`（hover か focus 中）, `depth`（新しい toast が何枚上にあるか） | toast `toast`/`title`/`body`/`icon`/`close` |
| `ToastGroupState` | `values.rs:294-305` | `input`, `assertive`, `count`, `above` | toast `group` |
| `FieldState` | `:203-206` | `invalid` | field `root`/`label`/`hint`/`error` |
| `ProgressState` | `:148-168` | `value: Option<f64>`, `size`, `shown: Value<f64>`（表示中の割合、400ms で追従）, `phase: Value<f32>`（不定のときの 1.3s 周期の時計）, `sweep()` | progress `track`/`fill` |
| `DisclosureState`（interaction） | `disclosure.rs:21-24` | `input`, `open` | disclosure の全 part |
| `TableHeaderState` | `table/data.rs:75-79` | `input`, `sort: Option<SortOrder>`, `sortable` | data_table `header`/`sort` |
| `DataTableRowState` | `values.rs:310-321` | `interaction: TableRowState{input, selected, active}`, `color`, `height` | data_table `row` |
| `DataTableCellState` | `:326-331` | `interaction: TableCellState`, `numeric` | data_table `cell` |
| `DataTableCheckState` | `:336-342` | `header`, `checked` | data_table `check` |
| `DataTablePageState` | `:347-352` | `index`, `pages` | data_table `page` |
| `PartState` | core | 入力状態だけ | menu `separator`/`label`/`inset`、toast `region`、list `root`、data_table `frame`/`toolbar`/`footer`、palette の多く、tag/number input |

依頼にあった state の対応: hover → `input.hovered`（ただし `.hover(..)` で書く）、pressed → `input.pressed`/`.pressed(..)`、focus-visible → `input.focus_visible`/`.focus_visible(..)`、disabled → `input.disabled`/`.disabled(..)`、checked → `CheckState::checked`/`MenuItemState::check`/`ToggleButtonState::pressed`、open → `PopupState::open`/`DialogState::open`/`SelectTriggerState::open`/`DisclosureState::open`/`ButtonState::expanded`、selected → `ListItemState::selected`/`TabState::selected`/`TableRowState::selected`、invalid → `CheckState::invalid`/`TextInputState::invalid`/`FieldState::invalid`、size → `ControlSize`、variant → `Look::variant::<T>()`。

### 2.8 variant の渡し方（`Look` と `Variant`）

`crates/kivact-ui/src/variants.rs`:

- `pub trait Variant: PartialEq + std::fmt::Debug + 'static {}`（`:29`）。sealed ではなく、キットが自分の型に `impl Variant for X {}` と明示的に実装する。
- すべての標準 builder が `.variant(value: impl Variant)` を持つ。型ごとに一つの値を保持し、同じ型の後の値が前の値を置き換える。値は mount 時に読まれる定数（`crates/kivact-ui/src/disclosure.rs:106-120` の rustdoc）。
- recipe は `Look<S>` を受け取る（`:159-187`）。`Deref<Target = S>` なので `look.size`・`look.input` のように state の field を直接読める。`look.variant::<T>() -> Option<&T>` でキットの variant を読む。知らない型は無視され、plain の look になる。
- `Look::new(state)` と `with_variant(v)`: キットの部品が自分の一部を「テーマの button」として描くとき、`ButtonState::new(input, size)` と組み合わせて recipe を直接呼ぶ（`values.rs:110-127`）。
- `Tone`（interaction）はすべての標準部品の variant（`variants.rs:31-35`）。`Tone::{Neutral, Info, Success, Warning, Danger, Accent}`。

core の意味としての variant:
- `ButtonVariant::{Primary(既定), Secondary, Outline, Ghost, Danger, Link}`（`values.rs:40-57`）。basic は `Primary`=shadcn `default`、`Danger`=`destructive`（README `:38-41`）。
- `ControlSize::{ExtraSmall, Small, Medium(既定), Large}`（`:67-78`）。button・text_input・text_area・select・checkbox・switch・radio_group・slider・toggle・progress が `.size(..)` を取る。basic では xs/sm/default/lg。

キット独自の variant（basic）:
- `ToggleVariant::{Default, Outline}`（`crates/kivact-ui-basic/src/choice.rs:16-25`）: `kivact::ui::toggle(..).variant(basic::ToggleVariant::Outline)`。recipe 側 `look.variant::<ToggleVariant>().copied().unwrap_or_default()`（`recipes_controls.rs:387-398`）。
- `AlertDialogSize`（`alert_dialog.rs:46`）: `dialog(..).kind(DialogKind::AlertModal).variant(basic::AlertDialogSize::Sm)`。
- `TabsVariant`（`line_tabs.rs:39`）: tabs の line 表示。
- 内部用: `Flush`（`recipes_dialog.rs:22`）、`MenuVariant`（`recipes_menu.rs:50`）。
- 他キット: pop の `ButtonTone`・`Slant`・`Segmented`・`Striped`・`Segments`・`Hue`、game の `Segmented`・`ChoiceCard`・`Navigation`・`Boxed`・`ButtonTone`・`Slant`・`Striped`、creative の `Segmented`。

### 2.9 完全な recipe の例（basic の switch）

`crates/kivact-ui-basic/src/recipes.rs:139-185`, `:274-302`（抜粋、全体はファイルを参照）:

```rust
const TRACK: (f32, f32) = (32.0, 18.4);
const THUMB: f32 = 16.0;
const TRACK_SM: (f32, f32) = (24.0, 14.0);
const THUMB_SM: f32 = 12.0;
const LEAD: f32 = 1.0; // room for the 1 px focus ring

fn toggle_row(p: &Tokens, drawn: f32) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(TRACK.1.ceil())
        .padding_left(LEAD + drawn + LABEL_GAP)
        .font_size(12)
        .line_height(1.0)
        .color(p.foreground)
        .focus_visible(style().ring_none()) // the drawn track shows the ring itself
}

fn switch(t: &Theme, state: &kivact::ui::Look<CheckState>) -> Style {
    let (p, state) = (t.get::<Tokens>(), **state);
    let (track, _) = switch_metrics(state.size);
    toggle_row(&p, track.0).paint(move |d, size| draw_switch(&p, &state, d, size))
}

fn draw_switch(p: &Tokens, state: &CheckState, d: &mut Draw, size: Size) {
    let on = state.checked.is_checked();
    let ((width, height), side) = switch_metrics(state.size);
    let track = Rect::new(LEAD, (size.height - height) / 2.0, width, height);
    let radius = height / 2.0;
    let fill = match (on, p.is_dark()) {
        (true, _) => p.primary,
        (false, false) => p.input,
        (false, true) => p.input.with_alpha(0.8),
    };
    d.rounded_rect(track, radius, fill);
    let (border, halo) = toggle_edge(p, &state.input, state.invalid, Color::TRANSPARENT);
    if halo.is_some() {
        edge(d, track, radius, border, halo);
    }
    let x = track.x + 1.0 + if on { side - 2.0 } else { 0.0 };
    let thumb = match (on, p.is_dark()) {
        (_, false) => p.background,
        (true, true) => p.primary_foreground,
        (false, true) => p.foreground,
    };
    d.rounded_rect(Rect::new(x, track.y + (height - side) / 2.0, side, side), side / 2.0, thumb);
}
```

要点:
- switch の root（label の行）は**一つの node**で、track と thumb は `Style::paint` で描く（子 node はない）。paint は state style に書けないので、focus_visible・invalid の枠は paint の中で `state.input.focus_visible` を読んで描いている（`toggle_edge`, `:224-238`）。recipe は state ごとに呼び直されるので、paint は最新の state を受け取る。
- **thumb の移動はアニメーションしない**（paint は transition の対象外）。Astryx の switch の thumb を滑らせるには別の方法が要る（5 章）。
- button の例（`recipes.rs:61-100`）は `trigger_style`（`crates/kivact-ui-basic/src/button.rs:64-147`）で、`.pressed(style().translate_y(1))`、`.hover(style().background(..))`、`with_focus(s, p)` のように state style を使う。popup の trigger（`expanded.is_some()`）は押しても動かない。

---

## 3. 標準部品（`kivact::ui`）、interaction、control

### 3.1 標準部品と builder の主な method と recipe hook

`crates/kivact-ui/src/lib.rs:8-31` の表と各ファイルの `pub fn`。すべての builder は `.variant(impl Variant)` を持つ（省略）。`NodeBuilder` の modifier（`.style`, `.test_id`, `.name`, `.disabled`, `.invalid`, …、3.4）も root に効く。

| 部品 | 作る関数 | 主な method | recipe hook（`UiRecipes`） |
|---|---|---|---|
| button | `button(label: impl IntoText) -> Button`（`button.rs:53`） | `selected`, `expanded`, `size`, `loading`, `icon(impl IntoView)`, `trailing_icon`, `hide_label`, `child`, `on_press`（NodeBuilder） | `button`, `button_content`, `button_busy`, `button_icon`（`ButtonState`） |
| checkbox | `checkbox(label, checked: impl Into<Binding<bool>>)`（`toggle.rs:193`） | `size`, `hide_label`, `indeterminate` | `checkbox`（`CheckState`） |
| switch | `switch(label, on)`（`toggle.rs:213`） | `size`, `hide_label` | `switch`（`CheckState`、`text` に ON/OFF） |
| toggle | `toggle(label, pressed)`（`toggle_button.rs:50`） | `size`, `icon`, `hide_label` | `toggle`（`ToggleButtonState`） |
| text_input / text_area | `text_input(label, value: impl Into<Binding<String>>)`, `text_area(..)`（`text_input.rs:74`, `:78`） | `forward_keys`, `size`, `placeholder`, `password`, `read_only`, `max_length`, `on_submit`, `draft(on_commit)`, `on_edit`, `return_focus`, `on_composition_change` | `text_input`, `text_area`（`TextInputState`）。paint 不可 |
| select | `select(label, options: impl IntoValue<Vec<ListItem<K>>>, selected: impl Into<Binding<Option<K>>>)`（`select.rs:39`） | `placeholder`, `on_preview`, `icon`, `size`, `option(\|item, state\| view)` | `SelectRecipes{trigger, surface, option, check, indicator, description}` |
| menu | `menu(label, entries: impl IntoValue<Vec<MenuEntry<K>>>, choose: impl Fn(K))`（`menu.rs:61`） | `offset`, `placement`, `align`, `at_least_trigger_width`, `behavior`, `item(\|entry, state\| view)`, `attach(trigger) -> View`, `attach_context(target) -> View` | `MenuRecipes{trigger, surface, sub_surface, context_surface, item, separator, label, shortcut, description, check, submenu, inset}` |
| dialog | `dialog(title, open: impl Into<Binding<bool>>)`（`dialog.rs:76`） | `width`, `closable`, `hide_title`, `media(fn)`, `content(fn)`, `footer(fn)`, `kind(DialogKind)`, `placement`, `align`, `description`, `initial_focus`, `dismiss(Dismiss)`, `attach(trigger)` | `DialogRecipes{surface, title, description, header, media, body, footer, scrim, close}`（`DialogState`） |
| tooltip | `tooltip(label)`（`tooltip.rs:30`） | `content(fn)`, `align`, `offset`, `placement`, `attach(trigger)` | `TooltipRecipes{surface, arrow}` |
| tabs | `tabs(label, options, binding: impl Into<Binding<K>>)`（`tabs.rs:47`） | `activation(ListActivation)`, `orientation`, `item(fn)`, `content(\|key\| view)`, `keep_mounted` | `TabsRecipes{list, tab, panel}` |
| radio_group | `radio_group(label, options, value: Binding<Option<K>>)`（`radio_group.rs:65`） | `orientation`, `size`, `on_edit`, `item(fn)` | `RadioGroupRecipes{group, item, indicator, description}` |
| slider / range_slider | `slider(label, value: Binding<f64>)`, `range_slider(label, low, high)`（`slider.rs:58`, `:71`） | `range`, `step`, `orientation`, `size`, `on_edit` | `SliderRecipes{track, range, thumb}`（`AxisState`） |
| splitter | `splitter(name, size: Binding<f64>, range)`（`splitter.rs:54`） | `orientation`, `start(pane)`, `end(pane)`, `on_edit` | `SplitterRecipes{handle}` |
| split | `split(panes: impl IntoChildren, sizes: Binding<Vec<f32>>)`（`split.rs:75`） | `orientation`, `grip(bool)`, `min_size`, `on_edit` | `SplitRecipes{grip}` + splitter の `handle` |
| progress | `progress(value: impl IntoValue<Option<f64>>)`（`progress.rs:56`） | `size`, `moving` | `ProgressRecipes{track, fill}` |
| disclosure | `disclosure(title, open: Binding<bool>, content: impl Fn() -> V)`（`disclosure.rs:72`） | `keep_mounted`, `animated(bool)`（200ms の高さアニメーション） | `DisclosureRecipes{root, trigger, indicator, content}` |
| toast_region | `toast_region(toaster: Toaster)`（`toast.rs:42`） | `hold_all_on_hover` | `ToastRecipes{region, group, toast, title, body, icon, close}` |
| list | `list(label, items, selected: Binding<Option<K>>)`（`list.rs:52`） | `multiple(Binding<Vec<K>>)`, `on_edit`, `item(fn)` | `ListRecipes{root, item, description}` |
| data_table | `data_table(rows, key: impl Fn(&R) -> K, columns: Vec<DataColumn<R>>)`（`data_table/builder.rs:83`） | `selection`, `sort`, `page`, `filter`, `resizable`, `row_label`, `row_actions`, `on_reorder`, `toolbar`, `footer`, `row_details`, `cell_controls`, `focus(TableFocus)`, `row_height`, `row_color` | `DataTableRecipes{frame, toolbar, footer, header, row, cell, check, sort, page}` |
| field | `field(label, control: C)`（`field.rs:59`） | `hint`, `error(impl IntoValue<Option<String>>)` | `FieldRecipes{root, label, hint, error}` |
| command_palette | `command_palette(open, commands, run)`（`command_palette.rs:68`） | `filter`, `close_on`, `placeholder`, `empty_text`, `footer` | `CommandPaletteRecipes{scrim, panel, search, search_icon, field, list, group, row, icon, hint, empty, footer}` |
| tag_input | `tag_input(label, tags, on_add, on_remove)`（`tag_input.rs:61`） | `placeholder` | `TagInputRecipes{root, tag, remove, input}` |
| number_input | `number_input(label, value, range: ScalarRange)`（`number_input.rs:74`） | `on_edit` | `NumberInputRecipes{root, input}` |
| inline edit | `use_inline_edit(..)`（`inline_edit.rs:49`） | `open`, `is_open`, `field` | なし |

データ型の再公開（`lib.rs:167-170`）: `Command, DialogKind, Dismiss, EditPhase, ListActivation, ListItem, MenuEntry, ScalarRange, ScalarRangeError, Side, Toast, Toaster, Tone, TreeNode, toaster`。prelude には `ButtonVariant, DataColumn, ListItem, MenuEntry, button, checkbox, data_table, dialog, field, list, menu, radio_group, select, slider, switch, tabs, text_area, text_input, tooltip`（`crates/kivact/src/prelude.rs`）。

callback は plain closure（`impl Fn(T) -> R`、`R` は `()` か `Result<(), UiError>`）。bool の option は定数か reactive 値（`impl IntoValue<bool>`）（`lib.rs:105-107`）。

### 3.2 `kivact::interaction` の振る舞い

`crates/kivact-interaction/src/lib.rs`。振る舞いは何も描かず、Copy な handle の part を呼び出し側が `.part(..)` で node や canvas region に付けます。命名は react-aria・Ark UI・Radix の anatomy に従い、構築は `use_*`、浮く／畳む本体は `content()`、一覧の行は `item(key)`、容器は `list()`、表は `grid()`/`header`/`row`/`cell`、開閉の問い合わせは `shown()`、命令は `open()`/`close()`/`toggle()`。

| 分類 | API |
|---|---|
| 開閉 | `use_disclosure` → `Disclosure`（`trigger`, `content`, `shown`, `leaving`, `disabled`）、`DisclosureState` |
| popup | `use_popup` → `Popup`（`anchor`, `trigger`, `content`, `shown`, `close`, `initial_focus`, `focus_return`）、`PopupState` |
| dialog | `use_dialog` → `DialogApi`（`trigger`, `content`, `title`, `description`, `close_trigger`, `anchor`, `dismiss`, `initial_focus`, `shown`）、`DialogKind::{Modal, AlertModal, Drawer(Side), Popover}`、`Dismiss { escape: bool, outside: bool }`、`Side` |
| tooltip | `use_tooltip` → `TooltipApi`、`TooltipConfig` |
| 遅延 | `use_delayed` → `Delayed`（hover card の開く遅延など）、`Delays`、`DelayedPhase` |
| focus の戻り | `FocusReturn`, `FocusReturnTarget`, `PopupCloseReason`, `ReasonSet` |
| 一覧 | `use_list` → `List`、`ListPolicy { axis, boundary, activation, mode, typeahead, range }`（`list/data.rs:244-249`、**typeahead は ListPolicy の bool**）、`ListSelection`, `ListMode`, `ListActivation`, `ListItem`, `ListItemState` |
| tabs | `use_tabs` → `TabsApi`, `TabListState`, `TabState` |
| tree | `use_tree` → `Tree`, `TreeNode`, `TreeRow`, `TreeBranch`, `TreeItemState` |
| table・grid | `use_table_model`, `use_table` → `Table`, `ColumnDef`, `SortOrder`, `TableFocus`, `TableHeaderState`, `TableRowState`, `TableCellState`, `GridMove` |
| calendar | `use_calendar_grid` → `CalendarGrid`, `CalendarPolicy`, `CalendarSelection`, `CalendarSpan`, `CalendarDayState` |
| combobox | `use_combobox` → `Combobox` |
| select | `use_select` → `SelectApi` |
| menu | `use_menu` → `MenuApi`, `MenuEntry`, `MenuCheck`, `MenuItemKind`, `MenuItemState`, `MenuOrigin`, `MenuRow`；`use_menubar` → `Menubar` |
| command palette | `use_command_palette`, `Command`, `CommandGroup`, `filter_commands` |
| 値 | `use_scalar` → `Scalar`（`IntoScalarRange`, `ScalarState`）、`use_range_scalar` → `RangeScalar`, `RangeThumb`, `ScalarRange` |
| splitter | `use_splitter` → `SplitterApi`, `SplitterState` |
| drag | `use_drag` → `Drag`（`on_start`, `on_update`, `on_end`, `cancel`, `edit`, `buttons`, …）、`DragPhase`, `DragState` |
| 編集の取消し | `EditPhase`, `EditSession` |
| toggle・button | `use_button`, `use_checkbox`, `use_switch`, `use_toggle_button` → `ButtonApi`, `CheckboxApi`, `Toggle`, `ToggleState` |
| toast | `toaster()` → `Toaster`, `Toast`, `ToastId`, `ToastState` |
| text | `use_text_edit`, `use_text_area`, `TextEdit`, `text_edit_policy` |
| 拡大・pan | `use_zoom_pan` → `ZoomPan`, `ViewTransform`；`use_space_pan` → `SpacePan` |
| tone | `Tone::{Neutral, Info, Success, Warning, Danger, Accent}` |

focus trap: 専用 API はなく、modal の dialog と palette は overlay の focus scope（`Overlay::modal`）を使う（`crates/kivact-core/src/view/part.rs:99` のコメント「A modal requirement uses the attached Overlay's existing focus scope」）。

### 3.3 overlay と layer

`kivact::view::{overlay, overlay_at, Overlay, OverlayAlign, Placement}`（`crates/kivact-core/src/view/overlays.rs`）。`overlay(anchor: NodeRef, placement, child)`、`Placement::{Below, Above, Start, End, Center}`（入らなければ反対側へ flip し window に収める）、`OverlayAlign::{Start, Center, End}`、`Overlay` の method `align`, `offset`, `modal`, `at_least_anchor_width`, `on_dismiss`, `on_dismiss_after`。`overlay_at(point)` は window の点（`PointerEvent::window_pos`）に置く。固定位置は `Position::Fixed`（toast の stack、sidebar の off-canvas）。

### 3.4 `kivact::control` と `kivact::primitive`

`crates/kivact-core/src/control.rs`: host の node は Root・Box・Text・Image・Overlay の五種類だけ。公開: `Node`, `NodeBuilder`, `Span`, `span`, `column`, `row`, `grid`, `stack`, `scroll`（`Scroll`, `ScrollAxes`）, `spacer`, `canvas`, `icon`, `image`, `node`, `rich_text`, `text`, `ImageFit`。

`NodeBuilder`（`crates/kivact-core/src/view/node.rs:178-`）の modifier: `node_mut`（実装者が書く唯一の method）, `test_id`, `node_ref`, `style`, `name`, `description`, `disabled`, `invalid`, `hidden`, `hit_testable`, `focusable`, `on_key`, `on_key_up`, `on_pointer`, `on_wheel`, `on_press`, `on_file`, `draw`, `intrinsic_size`。

`kivact::primitive::NodeExt`（部品作者向け、`crates/kivact-core/src/primitive/node_ext.rs`）: `part`, `role`, `value_text`, **`base_style`**（呼び出し側の `.style` より下に入る既定 style。`into_view` で設定が確定してから呼ぶ）, `selected`, `checked`, `expanded`, `accessible_*`, `has_description`, `at`, `indeterminate`, `value_min`/`max`/`now`, `live_region`, `sort`, `orientation`, `active_descendant`, `level`, `set_size`, `pos_in_set`, `multiselectable`, `described_by`, `controls`, `labelled_by`, `on_key_default`, `on_pointer_default`, `on_wheel_default`, `on_a11y`, `on_a11y_event`, `ime_session`, `on_ime`, `semantics_text`, `draw_over`。ほかに `part`, `PartState`, `Primitive`, `primitive`（計測する新しい primitive、実験的）, `NodeExt`, `NodeRefExt`, `focused_node`, `shape_paragraph` など。

`kivact::view`: `show`, `show_some`, `dynamic`, `for_each`, `for_index`, `virtual_list`, `error_boundary`, `provide`, `provide_theme(theme: impl IntoValue<Theme>, child)`, `provide_direction`, `current_direction`, `provide_locale`, `provide_messages`, `NodeRef`/`node_ref`（`rect()`, `hovered`, `focus_within` を reactive に読める）, 入力イベント型, `Role`, `CheckedState`, `Orientation`, `callback`, `IntoText`, `IntoValue`, `HandlerResult` など。

---

## 4. `kivact::style` の表現力

正は `crates/kivact-style/src/sheet.rs`（`Style` の全 setter）、`effect.rs`（`Shadow`, `Filter`, `Shape`, `BlendMode`）、`paint/`（gradient, `Paint`）、`transform.rs`、`text.rs`、`length.rs`。facade では `kivact::style::*`（`Style`, `style`, `Color`, `ColorToken`, `Length`, `pct`, `auto`, `em`, `rgb`, `rgba`, `hsl`, `Align`, `Justify`, `Direction`, `Position`, `Overflow`, `Cursor`, `TextAlign`, `TextOverflow`, `TextTransform`, `TextDecorationLine`, `FontFeature`, `FontKerning`, `LineHeight`, `TextLength`, `Track`, `GridPlacement`, `GridAutoFlow`, `ScrollbarWidth`, `TransitionProp`, `Theme`…）と `kivact::draw::*`（`Shadow`, `Filter`, `Shape`, `CornerRadius`, `BlendMode`, `Gradient`, `linear_gradient`, `radial_gradient`, `conic_gradient`, `Paint`, `Transform`, `Draw`, `Path`, `Stroke`, `Rect`, `Point`, `Size`, `Layer`, …）。

一般則:
- `Style` は `Option` の field を持つ平らな値。同じ property を二回設定すると後が勝ち、debug では重複を診断に記録。node の style は `.style(..)` の層のリストで、順に merge される。
- `Length`: `Auto`, `Px`, `Percent`（`pct`）, `Calc{percent, px}`（`pct(100) - 20`）, `MinContent`, `MaxContent`, `FitContent`（content size は width・height・basis・grid track だけ）。数値は logical px。**`em`・`rem`・`vw`・`vh` は layout の長さにはない**（`em` は `TextLength` の letter_spacing などだけ）。`platform::window_size()` が tracked なので、メディアクエリと vw は計算で作る。
- state style は paint だけ（0 章の 5）。

### 4.1 Astryx/CSS property → Kivact

| CSS / Astryx | Kivact | 根拠と注意 |
|---|---|---|
| `background-color` | `.background(impl ThemeColor)` | 単色は transition で補間される |
| `linear-gradient(..)` | `.background(linear_gradient(angle).stop(c, at)…)` / `.colors([..])` / `.repeating()` | `paint/gradient.rs:65-138`。角度は CSS と同じ（0=上、90=右、180=下）。**transition で補間されない** |
| `radial-gradient(..)` | `radial_gradient().circle().radius(r).radii(rx, ry).extent(RadialExtent).at(x, y)` | `gradient.rs:74-99` |
| `conic-gradient(..)` | `conic_gradient(from)` | `gradient.rs:100-103` |
| 複数の background layer、`background-size/position/repeat` | `.background((layer1, layer2, color))`（先頭が上）、`Gradient::size(..)`, `.position(..)`, `Paint::image(..)`, `size_cover`, `size_contain`, `no_repeat`, `repeat_x`, `repeat_y` | `paint/background.rs:8-27`, `paint/layer.rs` |
| `border` 全辺 | `.border(width, color)`、`.border_width(w)`、`.border_color(c)`、`.border_style(BorderStyle)` | `sheet.rs:605-660`。`box-sizing: border-box` と同じ。state style の中の width は描かれるが中身は動かない |
| 辺ごとの border | `.border_top/right/bottom/left(width, color)` | `sheet.rs:616-634`。**辺ごとの色だけを変える setter はない**（`border_color` は全辺）。辺ごとに色を変えるには辺ごとの `border_*` を使う |
| `border-style: dashed/dotted` | `BorderStyle::Dashed`（幅の 3 倍の dash）, `Dotted`（`effect.rs:69-75`） | 全辺共通 |
| `border-radius` | `.radius(r)`（`8`, `pct(50)`, `(20, 10)` 楕円） | `sheet.rs:680-685` |
| 角ごとの radius | `.radius_corners(tl, tr, br, bl)`, `.radius_top_left(..)` など | `sheet.rs:687-725` |
| `box-shadow` | `.shadow(Shadow::new(x, y, blur, color).spread(s).inset())`、**複数可**（先頭が上）、**inset 可**、**spread 可**、blur 可、角の radius（楕円も）に従う | `sheet.rs:753-758`, `effect.rs:15-66`。blur は CSS と同じ（標準偏差 blur/2）。**transition で補間されるのは最初の二つだけ** |
| `outline` / focus ring | `.ring(width, color)`（border box の外側）、`.ring_none()`。focus stop は style に focus 系の state がなければ `focus_visible` に `ring(theme.focus_ring(), ColorToken::Focus)` が自動で付く | `sheet.rs:742-751`, `:834-838`, `:38-45` |
| `outline-offset` / Tailwind `ring-offset-*` | **UNSUPPORTED**（`ring` に offset の引数はない） | 代替案 **[推測・未検証]**: `Shadow::new(0,0,0,bg).spread(o)` と `Shadow::new(0,0,0,ring).spread(o+w)` の二枚で描く。ただし theme_conformance の focus_pixels 試験は button の ring が `focus_ring()` の幅で border box に接していることを要求する（1.5） |
| `opacity` | `.opacity(f32)`（部分木を一つの group として） | `sheet.rs:401-405`。補間される |
| `transform: translate` | `.translate(x, y)`, `.translate_x`, `.translate_y`（px か自分の箱の %） | `sheet.rs:549-582`。軸ごとに merge |
| `rotate` / `scale` | `.rotate(deg)`, `.scale(s)`, `.scale_xy(x, y)` | `sheet.rs:439-448`, `:584-591` |
| `transform` 関数列（skew, matrix） | `.transform(Transform::new().skew_x(..).translate_y(..).matrix(..))` | `transform.rs:194-276` |
| `transform-origin` | `.transform_origin(x, y)`（既定は中心） | `sheet.rs:593-603` |
| transform の扱い | layout は無視（兄弟は動かない）、hit test と a11y と `NodeRef::rect` は描かれた位置 | `sheet.rs:429-438` |
| `overflow: hidden` | `.overflow(Overflow::Hidden)`（layout の overflow も変える）、`.clip(true)`（描画だけ切る） | `sheet.rs:55-59`, `:387-390`, `:469-472`。`Overflow::{Visible, Hidden, Scroll}` |
| `clip-path` | `.clip_path(Shape::polygon().point(..)…/inset(..).round(r)/circle(r)/ellipse(rx, ry).at(..)/path(Path))` | `effect.rs:186-305`。hit test も切る |
| `mask` / `mask-image` | `.mask(impl Into<Paint>)`（gradient も可） | `sheet.rs:418-422` |
| `filter` | `.filter(Filter::new().brightness().contrast().saturate().grayscale().sepia().hue_rotate().invert().opacity().blur().drop_shadow(x, y, blur, color))` | `effect.rs:101-184`。量は補間される |
| `backdrop-filter: blur()` | `.backdrop_blur(px)`, `.backdrop_filter(Filter)` | `sheet.rs:410-414`, `:737-740`。**実際の描画品質（GPU/CPU 両方で同じか）は [未確認]** |
| `mix-blend-mode` | `.mix_blend_mode(BlendMode::{Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, …})` | `sheet.rs:415-417` |
| `z-index` | `.z_index(i32)`（容器ごとに isolate） | `sheet.rs:449-456` |
| `cursor` | `.cursor(Cursor::{Auto, Default, Text, Pointer, ColResize, RowResize, EwResize, NsResize, Grab, Grabbing, Crosshair, Move, NotAllowed, None})` | `kivact-types` の Cursor。CSS の `wait`・`progress`・`help`・`zoom-in` などは **UNSUPPORTED** |
| `font-family` | `.font_family("Inter, 'Noto Sans JP', sans-serif")`（comma 区切りの list、generic family） | `sheet.rs:501-514`。未登録の family は system UI font に落ちて一度 `ErrorKind::Asset` |
| `font-weight` | `.weight(u16)`（1..1000） | |
| `font-size` | `.font_size(px)` | |
| `font-style: italic` | `.italic(true)` | |
| `line-height` | `.line_height(1.5)`（倍率）, `LineHeight::px(20)`, `LineHeight::Normal`（font size×1.3） | `sheet.rs:488-498` |
| `letter-spacing` | `.letter_spacing(px)` / `.letter_spacing(em(0.08))` | `sheet.rs:1124-1128` |
| `word-spacing` | `.word_spacing(..)` | |
| `text-align` | `.text_align(TextAlign::…)`（Justify あり） | |
| `font-variant-numeric: tabular-nums` | `.tabular_nums()` | `sheet.rs:1203-1213` |
| `font-feature-settings` | `.font_feature_settings([("tnum", 1), ("ss01", 1)])` | |
| `font-kerning` | `.font_kerning(FontKerning::{Auto, Normal, None})` | |
| `text-decoration` | `.text_decoration(TextDecorationLine::UNDERLINE)`, `.text_decoration_color`, `.text_decoration_thickness`, `.text_underline_offset` | **state style では変えられない**（recipe は `state.input.hovered` で分岐） |
| `text-transform` | `.text_transform(TextTransform::{None, Uppercase, Lowercase, Capitalize})` | |
| `text-shadow` | `.text_shadow(Shadow)`（spread と inset は不可） | |
| `white-space: nowrap` | `.text_wrap(false)` | |
| 一行の省略（truncate） | `.single_line()` = `text_wrap(false) + text_overflow(Ellipsis) + min_width(0) + overflow(Hidden)` | `sheet.rs:527-542`。親の flex row にも `min_width(0)` が要る |
| `text-overflow` | `.text_overflow(TextOverflow::{Clip, Ellipsis})` | |
| `line-clamp` | `.line_clamp(n)` | `sheet.rs:1168-1171` |
| `caret-color`, `::selection`, `::placeholder` | `.caret_color`, `.selection_color`, `.placeholder_color` | text field 用 |
| `word-break`, `hyphens`, `white-space: pre` など | **UNSUPPORTED [推測]**（setter が見当たらない） | |
| `font-stretch`, `font-variant-caps` 等 | **UNSUPPORTED [推測]**（`font_feature_settings` で OpenType feature を直接指定するのが代替） | |
| flex | `node()`/`column`（縦）, `row`（横）, `.direction(Direction::…)`, `.align_items`, `.align_self`, `.justify_content`, `.wrap(bool)`, `.grow`, `.shrink`, `.basis`, `.gap`, `.row_gap`, `.column_gap`, `.align_content` | `sheet.rs:61-79`。`Align::Baseline` は最初の baseline |
| grid | `grid(children)` が唯一の grid 容器。`.grid_template_columns((248, Track::fr(1)))`, `Track::repeat_auto_fill(Track::minmax(..))`, `grid_template_rows`, `grid_auto_columns/rows`, `grid_auto_flow`, `grid_column(GridPlacement::span(4))`, `grid_row`, `justify_items`, `justify_self`, `place_items` | `sheet.rs:1243-1287`。`grid-template-areas` は **UNSUPPORTED [推測]** |
| position | `.position(Position::{Relative, Absolute, Sticky, Fixed})`, `.top/right/bottom/left`, `.inset` | `sheet.rs:119-143`。Fixed は window 基準の独立した layer |
| `aspect-ratio` | `.aspect_ratio(16.0 / 9.0)` | |
| min/max | `.min_width`, `.max_width`, `.min_height`, `.max_height`, `.size(w, h)` | |
| margin / padding | 全辺・`_x`/`_y`・辺ごと | `sheet.rs:85-88`, `:324-350`。`margin: auto` は `Length::Auto` **[推測: Taffy の auto margin に従う]** |
| scroll 容器 | `scroll(child)`（既定は縦）, `.axes(ScrollAxes::{Horizontal, Both})`、`Role::ScrollView` が付く。`Overflow::Scroll` は両軸 | `sheet.rs:55-59` |
| scrollbar | `.scrollbar_width(ScrollbarWidth::{Auto, Thin, None, Px(f32)})`（重なる scrollbar、layout の場所を取らない、既定 6px）, `.scrollbar_color(thumb, track)` | `sheet.rs:457-467`, `:1063-1085`。scrollbar の radius・hover の太り方などの細かな見た目は **UNSUPPORTED [推測]** |
| `:hover` | `.hover(style)`（子の hover は親にも及ぶ、CSS と同じ） | |
| `:active` | `.pressed(style)`（押されていて、かつ hover 中だけ） | |
| `:focus` / `:focus-visible` | `.focus(style)`, `.focus_visible(style)` | |
| `:focus-within` | 自分自身の style には **UNSUPPORTED**。`.group()` を付けた祖先と `.group_focus(style)`（祖先か子孫に focus があるとき）、または `NodeRef::focus_within` を reactive に読む | `sheet.rs:30-37`, `:796-811` |
| `group-hover:` | `.group()` + `.group_hover(style)` | |
| `:disabled` / `[aria-selected]` | `.disabled(style)`, `.selected(style)`（どの property も可） | |
| `@starting-style` / exit | `.enter(style)`, `.exit(style)` | 5 章 |
| CSS 変数 | なし。キットの `Tokens`（`ThemeExtension`）と `Color` enum（`ThemeColor`）で代替 | |
| 局所描画（`::before` 的な装飾） | recipe/テーマは `.paint(\|d: &mut Draw, size\| ..)`、呼び出し側は `.draw(..)`/`.draw_over(..)` | `sheet.rs:916-933`。paint は base style だけ（state style 内は無視）。text・image・text input の leaf には効かない |
| `::before`/`::after` の node | **UNSUPPORTED**（子 node を足すか paint で描く） | |

`Draw`（`kivact::draw::Draw`）でできること: `fill_rect`, `stroke_rect`, `rounded_rect`, `line`, `fill_path`, `stroke_path`, `fill_shape`, `circle`, `polyline`, `text`, `measure_text`, `image`, `image_with`, `pixels`, `layer(Layer::new().area().transform().clip().clip_rect().opacity().blend().filter().mask().shadow(), \|d\| ..)`、`Path::{move_to, line_to, quad_to, cubic_to, close, arc, ellipse, arc_to, rect, rounded_rect}`、`Stroke::new(w).cap(..).join(..).dash([..]).dash_offset(..)`、`CanvasGradient::{linear, radial, conic}`（`crates/kivact-style/src/draw/*.rs`, `path/commands.rs`, `kivact-types/src/shape.rs:54-104`）。

---

## 5. アニメーション

### 5.1 公開 API（`kivact::motion`）

`crates/kivact-core/src/motion.rs`: `Animatable, Animated, Easing, FrameInfo, FrameLoop, FrameRate, Repeat, Spring, Transition, animated, frame_loop`。

- **一つの時計**（host 駆動）。描けない間（surface がない、隠れている）は frame を要求せず止まる。再開時に transition は止まった時点から続く（`crates/kivact-motion/src/lib.rs:6-17`）。
- `Transition`（`crates/kivact-motion/src/transition.rs`）:
  - `Transition::tween(ms, Easing)`
  - `Transition::spring(Spring { stiffness, damping, mass })`（定数 `Spring::GENTLE`=120/14/1, `WOBBLY`=180/12/1, `STIFF`=210/20/1, `DEFAULT`=170/26/1、`kivact-types/src/motion.rs:97-127`）
  - `Transition::keyframes([(offset, value), ..], ms)`（`Animatable` の値の絶対 keyframe）
  - `.delay(ms)`, `.repeat(Repeat::{Times(n), Forever})`, `.alternate()`, `.decorative(bool)`（既定 true。reduced motion で即終了。`false` は spinner のような意味のある動き）
- `Easing`（`kivact-types/src/motion.rs:22-34`）: `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, **`CubicBezier(x1, y1, x2, y2)`**（二分法で解く）, `Steps { count, position }`（`steps_start`, `steps_end`）。**注意: `EaseIn`/`EaseOut`/`EaseInOut` は CSS の `ease-in` 等（cubic-bezier(.42,0,1,1) など）ではなく三次多項式**（`EaseIn = t³`, `EaseOut = 1-(1-t)³`、`:57-65`）。CSS の曲線を再現するなら `CubicBezier` を使う。CSS の `ease`（0.25, 0.1, 0.25, 1）も `CubicBezier` で書く。
- `animated(target: impl IntoValue<T>, transition) -> Animated<T>`: 目標値を追う値（Copy、`Value<T>` が受け付く所ならどこでも使える）。中断されたら現在値（spring は速度も）から続く。`Animated::from(start, target, t)` は初回の mount でも start から動く。`get`, `get_untracked`, `with`, `value()`, `jump_to`, `pause`, `resume`, `on_complete(f)`, `is_running`（`animated.rs:198-401`）。
- `Animatable`: `f32`, `f64`, 2〜4 要素の tuple, `Point`, `Size`, `Rect`, `Color`（`kivact-style/src/theme/value.rs:609`）, `Transform`（`transform.rs:364`）。
- `frame_loop(\|FrameInfo\| ..) -> FrameLoop`: 毎 frame の callback。`start(FrameRate::{Hz(r), Display})`, `stop()`, `rate()`。`FrameInfo` に `now`, 前回からの時間, tick 番号, 表示予定時刻（`values.rs:101-121`）。
- reduced motion: `kivact::platform::reduced_motion()`（tracked）。decorative な transition は自動で即完了。`platform::surface_active()` も読める。試験では `RenderOptions { reduced_motion: true, .. }`、`Screen::set_reduced_motion`。

### 5.2 style の transition（engine 側で補間）

- `.transition(TransitionProp::Paint, Transition)`（`sheet.rs:894-899`）。**`TransitionProp` は `Paint` 一種類だけ**（`:150-158`）。補間対象: opacity、単色の background、border の色、transform（translate・rotate・skew・scale・origin）、最初の二つの shadow（offset・blur・spread・color）、filter の量。**gradient・画像・shape・z-index・filter の順序は即時に変わる**。text の `color`、`ring` は列挙されていない＝**補間されない [コードの doc から推測、engine 側は未読]**。
- state style（hover・pressed・focus…）の切替も、その node に paint transition があればその transition で補間される（`crates/kivact-engine/src/model/states.rs`、`state_motion`、`kivact-motion/src/lib.rs:32-37`「the host's focus, hover and pressed transitions」）**[推測: engine の doc から。直接の試験は読んでいない]**。transition がなければ即時。
- client は frame ごとに paint を書かない。style は目標だけを書き、動く node だけが engine で step される。
- 例（pop の button、`crates/kivact-ui-pop/src/button.rs:58-63`）:

  ```rust
  .shadow(Shadow::new(0, 0, 0, Color::TRANSPARENT))
  .cursor(kivact::style::Cursor::Pointer)
  .transition(TransitionProp::Paint, Transition::tween(150, Easing::EaseOut));
  ```

  （補間のために、hover で現れる shadow の「始点」として透明な shadow を置いている。）

### 5.3 enter / exit（mount・unmount のアニメーション）

`crates/kivact-motion/src/lib.rs:39-56`, `sheet.rs:850-892`, `crates/kivact-engine/src/model/presence.rs`:
- `.enter(style)` = CSS `@starting-style` / Framer Motion の `initial`: 最初に attach されたときの paint。
- `.exit(style)` = 取り除かれた node が消える前に向かう paint。取り除かれた部分木は「leaving」として描かれ続ける（hit・focus・a11y・modal scope からは外れる）が、**layout の場所は消えるまで保つ**（AnimatePresence と同じ）。場所を保たせたくなければ `Position::Absolute`。
- どちらも node の `TransitionProp::Paint` で動く。paint transition がなければ、または reduced motion で decorative なら、即時。
- 標準の toast・dialog・popup・tooltip は recipe が返す style でこれを使う（dialog に独自の entering 値はない）。

例:
- pop の dialog（`crates/kivact-ui-pop/src/recipes_extra.rs:162-168`）: `s.enter(style().scale(0.96)).transition(TransitionProp::Paint, Transition::tween(200, Easing::EaseOut))`。
- game の dialog（`crates/kivact-ui-game/src/recipes.rs:494-529`）: drawer は `.enter(style().opacity(0.).translate(±40, ..))`、modal は `.enter(style().opacity(0.).scale(0.985).translate(0, 10))`、`Easing::CubicBezier(0.2, 0.8, 0.2, 1.)`。
- game の toast（`recipes.rs:829-837`）: `.enter(style().translate_x(40)).exit(style().translate_x(40).opacity(0.)).transition(TransitionProp::Paint, Transition::tween(260, Easing::EaseOut))`。
- **basic の dialog・popover・toast には enter/exit がない**（`recipes_dialog.rs`・`recipes_popup.rs` に `enter`・`transition` の呼び出しがない）**[確認済み]**。

### 5.4 basic・core が今どう動かしているか

| 対象 | 実装 | 場所 |
|---|---|---|
| spinner の回転 | `Animated::from(0.0, 360.0, Transition::tween(1000, Easing::Linear).repeat(Repeat::Forever).decorative(false))` を `style().rotate(turn.get())` に | `crates/kivact-ui-basic/src/primitives.rs:174-187` |
| skeleton の明滅 | `Animated::from(1.0, 0.5, tween(1000, EaseInOut).repeat(Forever).alternate())` を opacity に。reduced motion では 1.0 | `primitives.rs:217-243` |
| toast の stack | 位置・大きさ・opacity を `ToastState.depth` と `engaged` から recipe で計算（後ろの toast は `Position::Absolute`, `translate_y(-GAP*k)`, `scale_xy`, `transform_origin(50%, 100%)`）。**transition は付けていない**ので開閉は即時 [確認済み: recipes_controls.rs に transition なし] | `recipes_controls.rs:177-260` |
| switch の thumb | `Style::paint` で描くので**動かない**（即時） | `recipes.rs:274-302` |
| disclosure（accordion） | chevron は `rotate(if open {180} else {0})`（transition なしなら即時）。高さは core の `disclosure(..).animated(true)` が `animated(.., tween(200, EaseOut))` で content の高さ（`node_ref().rect()` で測った自然な高さ×割合）を style の height に入れ、`overflow(Hidden)`。閉じる間は `leaving` で残す | `recipes_controls.rs:475-481`、`crates/kivact-ui/src/disclosure.rs:99-105`, `:131-200` |
| progress | `shown` は core が `Animated::from(0.0, target, tween(400, EaseOut))`、不定の segment は `frame_loop` を 30Hz で回す `phase` を paint で読む（`ProgressState::sweep`）。reduced motion と surface 非表示で止める | `crates/kivact-ui/src/progress.rs:120-145`、`values.rs:146-190`、basic `recipes_controls.rs:414-435` |
| drawer（vaul 風） | swipe 中は transition なし、離したら `.transition(TransitionProp::Paint, tween(200, EaseOut))` と translate | `crates/kivact-ui-basic/src/drawer.rs:203-221` |
| carousel | track の `translate(pct(..))` に `tween(300, CubicBezier(0.4, 0, 0.2, 1))` | `carousel.rs:168-190` |
| navigation menu | `tween(300, CubicBezier(0.4, 0.0, 0.2, 1.0))` | `navigation_menu.rs:279` |
| game の meter など | `frame_loop` で時計を進め、`reduced_motion()` と `surface_active()` で止める | `crates/kivact-ui-game/src/feedback.rs:276-283`, `surface.rs:122-131` |

### 5.5 animation のパターン → Kivact

| パターン | Kivact でのやり方 | ギャップ |
|---|---|---|
| hover・press の色や影の変化（`transition: background-color, box-shadow 150ms`） | base style に `.transition(TransitionProp::Paint, Transition::tween(150, Easing::CubicBezier(..)))`、`.hover(style().background(..).shadow(..))` | shadow は最初の二つだけ補間。text の色と ring は補間されない。gradient は即時 |
| press で縮む（`active:scale-95`） | `.pressed(style().scale(0.95))` + paint transition | なし |
| focus ring がふわっと出る | `.focus_visible(style().ring(..))` | ring は補間されない [推測]。代替: ring の代わりに shadow を使う（補間される） |
| CSS 曲線（`ease`, `cubic-bezier(..)`） | `Easing::CubicBezier(x1, y1, x2, y2)` | `EaseIn/EaseOut/EaseInOut` は CSS と別の曲線 |
| spring | `Transition::spring(Spring { stiffness, damping, mass })` | なし。CSS の `linear()` easing はない |
| keyframes（`@keyframes`） | `Transition::keyframes([(0.0, v0), (0.5, v1), (1.0, v2)], ms)` を `animated` に、または繰り返しの `Animated::from(..).repeat(..)` | keyframes は `Animated` 用。style transition の keyframes（CSS animation を style に直接）はない。途中の easing を区間ごとに変える手段は **[未確認]** |
| 無限ループ（spinner, shimmer） | `Animated::from(a, b, tween(..).repeat(Repeat::Forever))` を style closure で読む、または `frame_loop` | shimmer の gradient の移動は gradient が補間されないので、`Animated` の値で gradient の位置を毎 frame 作り直すか paint で描く |
| mount 時の登場（fade/scale in） | `.enter(style().opacity(0.).scale(0.96))` + paint transition | 補間される property だけが動く |
| unmount 時の退場 | `.exit(style()..)` + paint transition | 退場中も layout の場所を保つ。外すなら Absolute |
| layout のアニメーション（高さ・幅の変化、FLIP） | `animated` の値を `height`/`width` の style に毎 frame 入れる（disclosure の方式）。`node_ref().rect()` で自然な大きさを測る | **自動の layout animation はない**。transform は layout に影響しない |
| switch の thumb の移動 | 標準の switch では track と thumb は recipe の paint の中だけで描く。paint が `Animated` のような reactive 値を読めば追従する（`ProgressState::shown` を paint で読む progress と同じ形）**[推測]** が、その値を誰が作るかが問題（右欄）。キット独自の switch 風部品なら thumb を子 node にして `translate_x` + paint transition で動かせる | basic は動かしていない。recipe は view を返せないので、子 node を足すことはできない（paint で描くしかない）。`animated` は setup only（`crates/kivact-motion/src/animated.rs:212-214`）なので recipe（`Fn(&Theme, &Look<S>)`）の中では作れず、**標準の switch で thumb を滑らせるには core 側に口（ProgressState::shown のような値）が要る可能性が高い** |
| checkbox の check mark の描画アニメーション | `Path::trim(start, end, offset)` と `Path::length()` は公開されている（`crates/kivact-style/src/path/trim.rs:104`, `:139`）。paint の中で `Animated` の値を読めば描画を進められる | 標準 checkbox の recipe に時間の値が来ないので、同上 |
| accordion の高さ | `kivact::ui::disclosure(..).animated(true)`（200ms EaseOut、固定） | 時間と曲線は変えられない |
| toast の積み重ね・展開 | recipe で位置を計算 + toast の style に paint transition を足す（transform・opacity は補間される） | basic はしていない。pop/game の toast も enter/exit の slide だけ |
| reduced motion | `Transition` は既定で decorative。`kivact::platform::reduced_motion()` を読む | なし |
| 毎 frame の計算 | `frame_loop` | 60fps の上限なし（`FrameRate::Hz(r)`） |

---

## 6. icon

- 描くのは常に core の `kivact::control::icon(name)`（`crates/kivact-core/src/control/node.rs:347-410`）。16px、現在の text 色（monochrome）、名前は描画時にテーマの `IconSet` で引く（テーマ切替で描き直す）。`.name(..)` を付けなければ装飾（a11y に出ない）。16px より小さな箱では縮む。
- **名前がテーマの set にない場合は何も描かず、debug build では呼び出し箇所に一度だけ `ErrorKind::Asset`（「icon "x" is not in the theme's icon set; it draws nothing」）を報告する**（`node.rs:378-397`）。試験の `screen.errors()` に出るので、カタログ試験と consumer 試験で検出される。
- `IconSet`（`crates/kivact-style/src/icon.rs:95-214`）: `IconSet::new()`, `IconSet::standard()`（core の印: `check`, `dot`, `menu-radio`, `minus`, `chevron-down`, `chevron-right`, `x`, `plus`, `toast-info`, `toast-success`, `toast-warning`, `toast-danger`, `toast-accent`）, `with(name, image)`, `Extend<(name, bytes)>`, `get`, `resolve(&Icon)`。`Icon::named("check")` / `Icon::from(Asset)`。
- basic の登録（`crates/kivact-ui-basic/src/icons.rs`）:
  - `pub const ICON_NAMES: &[&str]`（`:5-131`、アルファベット順の Lucide 名 125 個、公開）。
  - `fn bytes(name) -> Option<&'static [u8]>` が `include_bytes!("../assets/icons/<name>.svg")` の match（`:133-263`）。
  - `pub(crate) fn set()`（`:277-308`）: `OnceLock` で一度だけ、`IconSet::standard()` に `ICON_NAMES` を `extend`、さらに Lyra 用に `dot`（8px の円の inline SVG `RADIO_DOT`）、`menu-radio`（check.svg）、`toast-info`/`toast-success`/`toast-warning`/`toast-danger`/`toast-accent`（sonner の icon）を上書き。
  - テーマで `.with_icons(crate::icons::set())`（`theme.rs:66`）。
  - SVG は lucide-static 0.468.0 のファイルそのまま（先頭に `<!-- @license lucide-static v0.468.0 - ISC -->`）。ライセンスは `crates/kivact-ui-basic/assets/LICENSE-lucide.txt`（ISC、Feather の MIT 表記を含む）。`THIRD-PARTY-NOTICES.md:11` に記録。
  - pop/game は `assets/icons/` に Tabler、`assets/LICENSE-tabler-icons.txt`（creative だけ crate 直下 `LICENSE-tabler-icons.txt`）。
- slot の規則（basic lib.rs `:23-29`、README `:77-81`）: 単一部品の icon slot は `impl IntoView`（`alert(..).icon(icon("info"))`）、データの record（`ListItem`, `MenuEntry`, `Command`）と `attachment_action` は名前を取る。
- 名前の重複に注意: standard の印の名前（`check`, `x`, `chevron-down` …）はキットの set で上書きすると標準部品の印も変わる（意図的に使える）。

---

## 7. text と i18n

- `kivact::i18n::message(key: &'static str, default: &'static str) -> Value<String>`（`crates/kivact-i18n/src/context.rs:111-115`）、値の差し込みは `message_with`（`{name}` を置換）。setup のときだけ呼ぶ。recipe の paint で文言が要るときは、部品が setup で作った handle を state で渡す（`CheckState::text`、`values.rs:22-25`）。
- アプリは `App::messages(Messages)` または `provide_messages` で差し替える（`crates/kivact-core/src/app.rs:68-72`）。
- **key の接頭辞はキット名**: basic は `basic.`（`basic.spinner.loading`, `basic.pagination.next` …）、pop は `pop.`、game は `game.`、creative は `creative.`（コードの `message("<prefix>.…")` を数えた結果: pop 33、game 7、creative 97）**[確認済み]**。soft は `soft.` にするのが慣例に沿う。
- 標準部品の key は接頭辞なし（`dialog.close`, `toast.close`, `switch.on`, `switch.off`, `slider.minimum`, `split.handle`, `data_table.*`, `palette.search`）。
- README に key と英語の既定値の表を置く（basic README `:106-182`「Texts and their message keys」: `basic.` の key 表と、キットが mount する標準部品の key 表の二つ）。
- 既定の文言は英語。

---

## 8. フォント

- **フォントはアプリのもの。** キットはフォントを同梱も登録もしない（basic README `:56-59`）。キットの README で推奨 family を書き、アプリが `App::font(bytes)`（`app.rs:99-110`、CSS の `@font-face` 相当、family 名はファイル内の名前）で登録し、`Theme::with_text(Typography { family: Some("Inter, sans-serif".into()), ..theme.text().clone() })` で既定 family を設定する。
- basic は text に Inter、code に `"Geist Mono", monospace` を要求（`code_block` と `inline_code` が `font_family` で指定）。
- カタログ（`examples/kivact-ui-basic/src/fonts.rs:4-10`）:

  ```rust
  pub fn fonts() -> &'static [&'static [u8]] {
      &[
          include_bytes!("../assets/fonts/inter/Inter[opsz,wght].ttf"),
          include_bytes!("../assets/fonts/geistmono/GeistMono[wght].ttf"),
          include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
      ]
  }
  ```

  `main.rs` で `for &font in fonts() { app = app.font(font); }`、試験で `TestFonts::Only(fonts().iter().map(|&b| Arc::from(b)).collect())`。shell は `"Inter, 'Noto Sans JP', sans-serif"` を `Typography.family` に入れる（`shell.rs:11-12`, `:25-36`）。
- `assets/fonts/` に `SOURCE.md`（出所）、各 family の `OFL.txt`、`SHA256SUMS.txt`。`THIRD-PARTY-NOTICES.md:12` に記録。

---

## 9. 試験

### 9.1 キットの consumer 試験の構造

`crates/kivact-ui-basic/tests/consumer.rs`: 一つの integration test binary で、`#[path = "consumer/xxx.rs"] mod xxx;` で分ける（`code_block`, `data`, `display`, `forms`, `foundation`, `navigation`, `navigation_layout`, `overlays`, `support`, `toasts`）。冒頭 `//! Role: External-consumer tests: no internal Kivact or renderer imports are available.`。`kivact` と `kivact_ui_basic` だけを import する（利用者の立場）。`use kivact::prelude::*` は `workspace.toml` の例外で許される。

`tests/consumer/support.rs`:

```rust
pub(crate) fn pixel(screen: &Screen, x: f32, y: f32) -> u32 { .. }      // 0xRRGGBB at a logical point
pub(crate) fn distance(a: u32, b: u32) -> u32 { .. }                    // largest channel difference
pub(crate) fn assert_clean(screen: &Screen) { assert!(screen.errors().is_empty(), ..) }
pub(crate) fn options() -> RenderOptions {
    RenderOptions { theme: basic::theme(ThemeMode::Light), reduced_motion: true, ..RenderOptions::from_env() }
}
```

試験は 33 個（関数名で契約を述べる。例: `theme_switch_restyles_mounted_components_and_keeps_their_state`, `checkbox_and_switch_toggle_with_pointer_and_keyboard`, `dropdown_menu_navigates_by_keyboard_and_skips_labels`, `toast_shows_title_description_and_leaves_after_five_seconds`, `accordion_in_single_mode_keeps_one_section_open`, `popover_opens_from_its_trigger_and_dismisses_with_focus_restored` …）。pop・game も同じ形（`tests/consumer.rs` + `tests/consumer/{controls, interactions, overlays, support, theme, …}.rs`）。

### 9.2 `kivact::test` の API

`crates/kivact-core/src/test.rs`（feature `test`、dev-dependency で有効化）:
- `render(view)`, `render_with(options, view)`, `render_with_handle(options, || (handle, view)) -> (Screen, handle)`, `gpu_unavailable()`。
- `RenderOptions { theme, locale, host_locale, messages, reduced_motion, dark, width(800), height(600), scale(1), fonts: TestFonts, window: WindowMode::Real, gpu: false, frame_period_ns, title, .. }`, `RenderOptions::from_env()`。
- 探す: `get_by_role(Role, name)`, `query_by_role`, `find_by_role`, `all_by_role`, `get_by_text`, `query_by_text`, `get_by_test_id`, `query_by_test_id`, `within(element)`。
- 操作: `click`, `double_click`, `force_click`, `click_at`, `hover`, `press_key(Key)`, `press_key_with`, `press_tab(shift)`, `type_text`, `focus`, `scroll(dx, dy)`, `drag`, `wheel`, `tap`, `compose`（IME）, `select_option`, `toggle`。
- 読む: `rect`, `text`, `name`, `value`, `value_range`, `checked`, `is_focused`, `is_disabled`, `is_expanded`, `is_selected`, `is_visible`, `controls`, `labelled_by`, `described_by`, `level`, `role`, `orientation`, `live_region` など。
- 時間: `advance(Duration)`（仮想時計を進める）, `run_until_idle`, `wait_for`, `frame_ticks`。
- 画像: `screenshot() -> Screenshot`（`pixel(x, y)`, `crop`, `save_png`）。
- 検査: `errors()`（報告された診断）, `assert_layout()`, `assert_layout_with(&LayoutCheck)`, `check_layout`。
- 環境: `set_dark`, `set_reduced_motion`, `set_scale`, `set_size`, `set_host_locale`, `clipboard`, `set_clipboard` など。

**レイアウト崩れの検査**（`crates/kivact-core/src/test/screen/results.rs:169-208`）: `LayoutRule::{OverflowOwnBox, OverflowVisualParent, Wrapped { lines }, OffCenter, ClippedWithoutEllipsis}`。`LayoutCheck::default().ignore_test_id(id)` で意図的な clip を除外（カタログの carousel、`examples/kivact-ui-basic/tests/catalog.rs:34-48`）。

### 9.3 試験の規則

- `CONTRIBUTING.md`「テスト」: 実ウィンドウの E2E だけ。公開 API の利用者の契約を確かめるものに絞る。**過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は追加しない**。
- 実際の basic の試験は画素を見るが、**許容差つき**（`distance(..) <= 2` や `<= 8`）で、色の契約（テーマの token の色が塗られていること）を確かめる。位置は `rect` で確かめる（`toasts.rs:47-54` の 384px・16px の余白は等号比較をしている）。時間は `advance` で既定の 5 秒を越えたかを見る（厳密な frame の比較ではない）。
- `reduced_motion: true` を基本にする（アニメーションの途中を見ない）。

### 9.4 window lock と時間

- `dev/tools/window-lock.sh [--timeout SECONDS] -- <command>`: すべての worktree で共有する lock（`$(git rev-parse --git-common-dir)/kivact-window-tests.lock`）を取り、同時に一つの process だけがウィンドウを開く。既定の上限 900 秒、時間切れは終了コード 124。
- 例: `dev/tools/window-lock.sh -- cargo test -p kivact-ui-basic --test consumer`、`dev/tools/window-lock.sh -- cargo test -p kivact-ui-basic-catalog`。
- **所要時間は測っていません [未確認]**。手がかり: merge 前の workspace 全体の試験に `--timeout 5400`（90 分）を与えている。

---

## 10. カタログ（`examples/kivact-ui-basic`）

- package 名 `kivact-ui-basic-catalog`、`publish = false`、`[features] default = ["gpu"], gpu = ["kivact/gpu"]`、依存は `kivact`（default-features = false）と `kivact-ui-basic`、dev は `kivact` の `test`（`examples/kivact-ui-basic/Cargo.toml`）。`cargo run -p kivact-ui-basic-catalog`、CPU だけなら `--no-default-features`。
- 構造:

| ファイル | 役割 |
|---|---|
| `src/main.rs` | `App::new("Kivact UI · Basic").window_size(1440, 900)`、`fonts()` を登録、`app.run(catalog)` |
| `src/lib.rs` | `mod docs; mod fonts; mod look; mod model; mod page; mod pages; mod shell;`、`pub use fonts::fonts; pub use pages::{ExampleInfo, PageInfo, pages}; pub use shell::{catalog, catalog_with_mode};`（試験が使う公開面） |
| `src/shell.rs` | 上のバー（テーマ切替 button、test id `catalog-theme`）、検索つきの一覧（`use_list` で keyboard 移動、test id `catalog-search`, `catalog-nav-<slug>`, `catalog-nav-scroll`）、ページ領域（`dynamic`）、`toast_region`。`provide_theme` で `basic::theme(mode)` に family を足したテーマを与える |
| `src/page.rs` | 一ページの描画: breadcrumb、`h1`（`catalog-title`）、`lead`、「使い方」の code block（`catalog-usage-code`）、例ごとの見出し・説明・preview（`example-<slug>-<id>-preview`）・code（`example-<slug>-<id>-code`） |
| `src/model.rs` | `Page { slug, name, description(日本語), usage, source: include_str!, align: PreviewAlign, examples }`、`Example::new(id, title, description, fn() -> impl IntoView)`、`Page::snippet(id)`（`// example: <id>` と `// end example` の間を字下げを外して返す） |
| `src/pages.rs` | 試験用の `PageInfo`/`ExampleInfo` と `pages()` |
| `src/docs.rs` | `mod accordion; …`（67 ページ）と `pub(crate) static PAGES: LazyLock<Vec<Page>>`（アルファベット順に `xxx::page()` を並べる）。xtask の non-leaf 例外の対象 |
| `src/docs/<slug>.rs` | 1 部品 1 ファイル。`page()` と、`// example: <id>` … `// end example` で囲んだ `fn xxx() -> impl IntoView` |
| `src/look.rs` | カタログ自身の style のための `tokens()` |
| `src/fonts.rs` | 8 章 |
| `tests/catalog.rs` | 実ウィンドウの試験 |
| `assets/fonts/`, `assets/images/` | フォントと自作の画像 |
| `README.md` | 日本語。ページの足し方、フォント、試験 |

- ページの登録（README の手順）: 既存のページを写して `slug`・`name`・`description`・`usage`・`source` を直す → 例ごとに `Example::new` を足し、marker の間に関数を書く → `docs.rs` の `mod` と `PAGES` に足す。画面のコードは実行しているコードそのもの（`include_str!` で自分のソースを読む）。コードは basic の `code_block` + `rust_lines` で表示。
- `tests/catalog.rs` の試験（1440x900、`reduced_motion: true`、`WindowMode::Real`、`KIVACT_CATALOG_GPU=1` で GPU）:
  - `every_page_opens_from_the_list_and_renders_cleanly`: 全ページを一覧から開き、選択状態・タイトル・page・usage code・全例の preview と code の存在、`errors()` が空、`assert_layout_with`。
  - `theme_toggle_keeps_the_page_and_the_input_state`: テーマ切替でページと入力値が残り、画素が変わる（restyle in place）。
  - `search_filters_the_list_and_the_keyboard_walks_it`。
  - `KIVACT_CATALOG_EVIDENCE=<dir>` でページごとの PNG を保存（通常は file IO なし）。
- 他キットのカタログは形が違う（pop は「original shell, chapters, themes」、`src/{fonts, lib, main, messages, pop, pop.rs}`）。basic の文書型が Astryx の docs に近いので手本に向く **[推測]**。

---

## 11. 利用者向けドキュメントと doctest

- 置き場所: `docs/for-kivact-ui-user/{en,jp}/<kit>/getting-started/`（チュートリアル、`README.md` + `01-…md`, `02-…md`, `03-…md`。一つのアプリを段階的に育てる）と `docs/for-kivact-ui-user/{en,jp}/<kit>/how-to-guide/`（`README.md` の表 + 目的別のページ）。en と jp は同じファイル名で対、同じ変更で更新。
- basic の構成:
  - `getting-started/README.md`（前提、カタログの起動、Steps 表）、`01-set-up-the-kit.md`（Cargo.toml の git 依存、テーマとフォントの hook）、`02-build-the-settings-form.md`、`03-dark-mode-and-feedback.md`（テーマ切替・toast・alert dialog・試験）。
  - `how-to-guide/README.md` と `theme-and-tokens.md`, `overlays.md`, `navigation.md`, `data-display.md`, `forms.md`。
  - 各ページの先頭に他言語へのリンク（`[日本語](../../../jp/basic/…)`）。
- `docs/for-kivact-ui-user/{en,jp}/README.md`: キットの選び方の表（Kit・Crate・Look・Use it for）、Tutorial・Guides・catalog コマンドの表、「How a kit is used」の例、Reference（`cargo doc -p …`）。新キットはここに行を足す（1.10）。
- **doctest**（`dev/tests/docs`、package `kivact-docs`）: `build.rs` が **root の `README.md` と `README_JP.md`**、**`docs/for-kivact-user/` と `docs/for-kivact-ui-user/` の下の全 `.md`** を、それぞれ空の module の `#[doc = include_str!(..)]` にする（`dev/tests/docs/build.rs:13-31`, `const TREES: [&str; 2] = ["for-kivact-user", "for-kivact-ui-user"]`）。新しいページの登録は不要。` ```rust ` ブロックはコンパイルされ、`no_run` でなければ実行される。ウィンドウを開くブロックは `no_run`、Rust でないものは ` ```text `。`#[cfg(test)]` を含むブロックは `test_harness` が必須（`src/tests.rs` の検査）。
- **したがって**:
  - repo root の `.md`: `README.md` と `README_JP.md` だけがコンパイルされる。**それ以外の root の `.md`（`CONTRIBUTING.md` など）はコンパイルされない**。
  - **新しい root のフォルダ（例: `soft-ui-work-order/`）の下の `.md` はコンパイルされない**。
  - `docs/for-kivact-developer/` もコンパイルされない（TREES にない）。
  - crate の `README.md`（`crates/kivact-ui-basic/README.md` など）は kivact-docs ではコンパイルされない。lib.rs が `include_str!("../README.md")` していないので crate の doctest にもならない **[確認済み]**。
  - 一方、**キットの rustdoc の中の ` ```no_run ` 例はその crate の doctest として `cargo test --workspace` でコンパイルされる**（例: `crates/kivact-ui-basic/src/badge.rs:44-54`）。
- `docs/for-kivact-ui-user/{en,jp}/soft/…` を作ったら、その Rust ブロックが `kivact_ui_soft` を使うので `dev/tests/docs/Cargo.toml` に依存を足す。

---

## 12. 命名と API の慣例

kit-naming や public-names の文書・policy は**存在しません**（1.4）。以下はコードと rustdoc から読み取った慣例です。

- crate 名 `kivact-ui-<kit>`、カタログ `kivact-ui-<kit>-catalog`。ドキュメントと例では `use kivact_ui_basic as basic;` と短い alias で使う（basic lib.rs `:38-39`「Docs import the crate as `use kivact_ui_basic as basic;`」、pop は `pop`）。soft なら `use kivact_ui_soft as soft;` が慣例に沿う。
- テーマ: `pub fn theme(mode: ThemeMode) -> Theme`。tokens: `pub struct Tokens`（`ThemeExtension`）。色: `pub enum Color`（`ThemeColor`）。icon の名前一覧: `pub const ICON_NAMES`。
- 部品の構築関数は snake_case の自由関数（`badge`, `card`, `card_header`, `sidebar_menu_button`, `input_group_input`）で、`#[track_caller]` を付ける（診断の位置）。builder 型は PascalCase で関数名に対応（`badge` → `Badge`）。部品の部分は `<component>_<part>`（`breadcrumb_link`, `pagination_next`, `table_cell`, `field_set`）。
- variant と size の enum: `<Component>Variant`（`BadgeVariant`, `AlertVariant`, `ItemVariant`, `SidebarVariant`）、`<Component>Size`（`AvatarSize`, `CardSize`, `CodeBlockSize`）。標準部品に渡す variant は `kivact::ui::Variant` を実装（`ToggleVariant`, `AlertDialogSize`, `TabsVariant`）。
- shadcn の名前との対応: 基本は shadcn の部品名・part 名を snake_case にする（`alert_dialog`, `hover_card`, `input_otp`, `navigation_menu`）。標準部品が意味を持つものは標準の名前を使い（`dropdown-menu` → `kivact::ui::menu`、`sheet` → `dialog` の `Drawer`）、README で対応を説明する。basic の README `:32-48` と各カタログページの description が対応表の役を果たしている。
- builder は `NodeBuilder` と `IntoView` を実装し、自分の既定 style と semantics を先に入れる（`base_style`）。呼び出し側の modifier（`.style`, `.test_id`, `.disabled`, `.name`）が勝つ（basic README `:68-71`）。option のない部分は `Node` を返す（`separator()`, `kbd(..)`, `table_cell(..)`）。
- slot の method（`content`, `header`, `footer`, `action`, `actions`, `media`, …）は closure `impl FnOnce() -> V` を取り、部品が mount するときに一度呼ぶ（`crates/kivact-ui-basic/src/slot.rs`、lib.rs `:18-21`）。容器の子は view（`column((..))`, `field(label, control)`）。開くたびに新しい branch で中身を作る overlay（`drawer`, `hover_card`）は slot を関数で取り、`NodeBuilder` を実装しない。`hover_card(..).attach(trigger)` は trigger と overlay を返し、trigger が modifier を受ける。
- 一つの view を持つ単一部品の icon slot は `impl IntoView`、record（`ListItem`, `MenuEntry`, `Command`）は名前。要素ごとの関数引数は `impl Fn(..) + 'static`。
- 状態は呼び出し側の signal と binding（`impl Into<Binding<T>>`）。collection は読み取り専用の `impl IntoValue<Vec<T>>`。要求（copy の押下など）は callback で報告し、clipboard・永続化・業務規則はアプリ（README `:61-66`）。
- `GroupItems`（`crates/kivact-ui-basic/src/children.rs:18-21`）: 「tuple（最大 16 の異なる builder）か `Vec`」を受ける trait。`button_group` が子ごとに style の層を足すため（2 つ目以降の border を消す）。`#[diagnostic::on_unimplemented]` で分かりやすいエラーを出す。
- sealed trait: `crates/kivact-ui-basic/src/sealed.rs` の `pub trait SelectionValueSeal<K>`（`SelectionValue` を四つの選択の形に限る）。公開境界に現れるが crate の外から実装できない trait にする。
- overlay の取り付けは標準と同じ名前（`attach`, `attach_context`, `content`, `footer`）。`show` は分岐（`view::show`）のために予約、キットの同じ役割の method は同じ名前（`crates/kivact-ui/src/lib.rs:79-84`）。
- 方向（RTL）: `kivact::view::provide_direction` と `current_direction` を読む部品だけが反転する。標準部品は反転しない（README `:64-66`）。

---

## 13. merge 前の検査

`CONTRIBUTING.md`「merge 前の検査」:

```text
cargo fmt --all -- --check
cargo xtask check
git diff --quiet -- architecture
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --no-default-features
dev/tools/check-features.sh
```

- `cargo xtask check`: code 行の上限、`visible_to`、module の依存の向き、Google Rust Style Guide の 20 の規則（rustfmt と clippy を含む）、`architecture/` と `architecture/index.html` の生成。submodule `vendor/xtask-core` と Node.js が要る。`cargo xtask` は `.cargo/config.toml` の alias（`run --package xtask --quiet --`）。
- CI（`.github/workflows/ci.yml`）は `cargo fmt --all -- --check` と `cargo fetch --locked` と `cargo xtask check --no-write` だけ（timeout 30 分）。残りはローカル。
- 所要時間: **測っていません [未確認]**。test の timeout が 5400 秒であることから、workspace 全体の試験は数十分規模と見込まれる **[推測]**。個別には `cargo test -p kivact-ui-soft --test consumer`、`cargo test -p kivact-ui-soft-catalog`、`cargo test -p kivact-public-api-tests --test e2e -- theme_conformance`、`cargo test -p kivact-docs --doc -- for_kivact_ui_user::en::soft` のように絞れる（docs の module 名は `dev/tests/docs/build.rs:5-6` の例）。
- Linux: ツリーの中の記述は、`.gitattributes:1`（「Shell scripts run under Linux (WSL) and must keep LF line endings.」）と `crates/arvane-native/src/lib.rs:32-48`（Linux の E2E は WSLg の Wayland と X11 で動かす、experimental）だけ。作業者のメモには `dev/tools/linux/test-wslg.sh [wayland|x11|all]` と `KIVACT_WSL_DISTRO=Ubuntu-24.04` とあるが、**この branch の `dev/tools/` には `check-features.sh` と `window-lock.sh` しかなく、`linux/` は存在しない [確認済み]**。キットは公開 API だけを使い platform 固有のコードを持たないので、Linux の検査は通常不要 **[推測]**。

---

## 14. kivact-ui-basic は shadcn をどう写したか

原則（`docs/for-kivact-developer/strategy.md:78`, `:98`、`crates/kivact-ui/src/lib.rs:86-103`）: **共通の意味は中核へ。キットはテーマと分野固有の部品だけ。** 標準部品で表せるものは recipe の差し替え（テーマ）で shadcn の見た目にし、標準部品にない意味の部品だけをキットの部品として作る。足りない振る舞いは `kivact::interaction` の部品から組み立てる。

### 14.1 標準部品の restyle になったもの（キットに部品のコピーはない）

| shadcn | Kivact |
|---|---|
| button.tsx（variant: default/secondary/outline/ghost/destructive/link、size: xs/sm/default/lg/icon） | `kivact::ui::button` + `ButtonVariant::{Primary, Secondary, Outline, Ghost, Danger, Link}` + `ControlSize`、icon size は `.icon(icon(..)).hide_label(true)` |
| checkbox.tsx | `kivact::ui::checkbox` |
| switch.tsx | `kivact::ui::switch` |
| input.tsx | `kivact::ui::text_input` |
| textarea.tsx | `kivact::ui::text_area` |
| select.tsx | `kivact::ui::select` |
| dropdown-menu.tsx | `kivact::ui::menu(..).attach(trigger)` |
| context-menu.tsx | `kivact::ui::menu(..).attach_context(target)`（top level が広い） |
| dialog.tsx | `kivact::ui::dialog`（`DialogKind::Modal`） |
| alert-dialog.tsx | `dialog(..).kind(DialogKind::AlertModal)` + `.variant(basic::AlertDialogSize::Sm)`、footer に cancel と action、`.closable(false)` |
| sheet.tsx | `dialog(..).kind(DialogKind::Drawer(side))` |
| popover.tsx | `dialog(..).kind(DialogKind::Popover)` |
| tooltip.tsx（arrow 付き） | `kivact::ui::tooltip`、`TooltipRecipes::arrow` |
| tabs.tsx | `kivact::ui::tabs`（縦は全幅の tab を panel の横に）。line の見た目は `basic::TabsVariant` |
| radio-group.tsx | `kivact::ui::radio_group` |
| slider.tsx | `kivact::ui::slider`, `range_slider` |
| resizable.tsx | `kivact::ui::split`（`.grip(true)` が `withHandle`）、`splitter` |
| sonner（toast の stack） | `kivact::ui::toast_region(toaster()).hold_all_on_hover(true)` + `toaster().push(Toast::new(..).tone(..))` |
| toggle.tsx | `kivact::ui::toggle` + `basic::ToggleVariant` |
| progress.tsx | `kivact::ui::progress` |
| accordion.tsx | `kivact::ui::disclosure` を縦に並べる |
| command.tsx | `kivact::ui::command_palette` |
| data-table | `kivact::ui::data_table` |
| field（label・hint・error の配線） | `kivact::ui::field` + basic の `field_set`, `field_group`, `field_content`, `field_separator` |

### 14.2 キットの部品として作ったもの

`crates/kivact-ui-basic/src/lib.rs:107-184` の公開 API: `alert`, `attachment`（+ `attachment_action`, `attachment_group`）, `avatar`（`avatar_group`, `avatar_group_count`）, `badge`, `breadcrumb*`, `button_group*`, `calendar`, `calendar_range`, `card`, `card_header`, `carousel`, `chart`（`ChartConfig`, `ChartKind`, `ChartRow`, `ChartSeries`）, `bubble`, `bubble_group`, `message`, `message_group`, `code_block`, `code_lines`, `rust_lines`, `collapsible`, `combobox`, `date_picker`, `date_range_picker`, `drawer`（vaul 風、swipe で閉じる）, `empty_state`, `field_*`, `hover_card`, `input_group*`, `input_otp`, `item`, `item_group`, `item_separator`, `kbd`, `kbd_group`, `label`, `separator`, `skeleton`, `spinner`, `aspect_ratio`, `marker`, `menubar`, `message_scroller`, `native_select`, `navigation_menu*`, `pagination*`, `questionnaire`, `scroll_area`, `sidebar*`（`sidebar_provider` ほか）, `table*`, `toggle_group`, `trace*`, typography（`h1`..`h4`, `paragraph`, `lead`, `large`, `small`, `muted`, `blockquote`, `bullet_list`, `inline_code`）。

これらは shadcn の部品のうち標準部品に同じ意味がないもの（calendar, carousel, chart, sidebar, input-otp, …）と、basic 固有の追加（chat の bubble/message、code block、trace、questionnaire、attachment、marker、message scroller）。追加分の出所（shadcn の公式部品か、外部の利用者の要望か）は **[未確認]**。作業者のメモには「キットは外部の dogfooding 利用者の要望で育った。縮めない」とある。

キットの部品も振る舞いは `kivact::interaction`（`use_disclosure`, `use_list`, `use_calendar_grid`, `use_combobox`, `use_menubar`, `use_delayed`, `use_drag`, …）から組み、自分で入力処理を再実装しない。

### 14.3 提供していないもの

- shadcn の `form`（react-hook-form 連携）に当たる部品はない。検証と状態はアプリが signal で持ち、`kivact::ui::field(..).error(..)` で表示する **[推測: カタログに form のページがない]**。
- `direction` はキットの部品ではなく `kivact::view::provide_direction`（カタログに Direction のページはある）。
- カタログに「準備中」のページは残っていない **[確認済み]**。

### 14.4 soft への含意

- Astryx の部品も同じ規則で分ける: 標準部品に意味があるもの（Button, Checkbox, Switch, TextInput, Select, Menu, Dialog, Tooltip, Tabs, Radio, Slider, Toast, Progress, Accordion, …）は **recipe で restyle**。標準にないもの（Avatar, Badge, Card, …）だけキットの部品。二つ以上の公式キットに同じ意味の部品が既にあり、それが標準部品にないなら、本来は中核へ入れるべきもの（ただし既存キットの部品は縮めない）。
- Astryx の見た目のうち Kivact で表せないもの（4 章・5 章の UNSUPPORTED とギャップ）は、勝手に近似する前に、中核へ口を足すべきかを判断する（strategy の「不足する口は中核へ足して kit で複製しない」）。

---

## 15. コードとドキュメントのずれ（注意）

1. `crates/kivact-ui-basic/src/lib.rs:41-46` は「texts … read through `kivact::i18n::message` under **unprefixed** keys that the README lists」と書くが、コードと README は `basic.` 接頭辞（`basic.spinner.loading` など、README `:106-110`）。コードが正。
2. basic README `:64-66` とカタログ `examples/kivact-ui-basic/src/docs/direction.rs:18` は `basic::direction` と書くが、`lib.rs` に `direction` の公開はなく、実際は `kivact::view::provide_direction`（`direction.rs:35-38`）。
3. `crates/kivact-ui/src/lib.rs:91-92`「Every theme of the official kits carries recipes for every standard component」だが、basic は `tag_input` と `number_input` の recipe 表を差し替えていない（`crates/kivact-ui-basic/src/recipes.rs:28-59`）。
4. 依頼にあった `dev/tests/workspace-policy`、public-names の golden、kit-naming の文書は存在しない。
5. 作業者のメモにある `dev/tools/linux/test-wslg.sh` はこの branch に存在しない。
