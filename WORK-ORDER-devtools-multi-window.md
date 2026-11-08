# 作業指示書: 複数ウィンドウ・DevTools・暗黙の状態マシンの解消

何を作るかは [設計書](DESIGN-devtools-multi-window.md)、既存の状態マシンの直し方は [付録](WORK-ORDER-appendix-state-machines.md) に書いてある。この文書は「誰が・どの順で・どのファイルを・どう変え・どう確かめるか」を書く。

---

## 0. 読み方と守ること（指揮役とすべてのサブエージェント）

### 0.1 役割

| 役割 | 人数 | 仕事 |
|---|---|---|
| 指揮役 | 1 | 波（wave）ごとにサブエージェントを起動し、終わったら波の検査をして commit する。自分ではコードを書かない（W0 と commit、検査の失敗の切り分けを除く） |
| サブエージェント | 1 波に最大 6 | 割り当てられた task を 1 つだけ実装する |

### 0.2 絶対の規則

1. 設計をしない。設計書と付録に書いてあるとおりに作る。書いてあることと実際のコードが合わない（行番号がずれている程度なら周辺を読んで合わせてよい。型や関数がそもそも無い、意味が違う）ときは、その task を止めて、合わない点を報告する。推測で作らない。
2. 自分の task の「担当ファイル」以外を編集しない。担当ファイルの外でコンパイルが失敗していても直さない（4. を見る）。
3. 公開 API の名前・形は設計書のとおり。名前を変えない、別名（alias）を作らない、互換の wrapper を残さない。
4. 他の task が編集中の crate のせいで `cargo check` が失敗したとき（エラーの file が自分の担当ではない）は、2 分待ってもう一度実行する。これを 5 回くり返しても直らなければ、そのまま報告して止める。ただし W12 と W14（同じ crate を 6 体で編集する波）では待たない: エラーが他の担当の file だけなら、自分の file を書き終えて「他の file のエラーのため未確認」と報告する。指揮役が全員の報告の後で波の検査をし、残ったエラーを file の担当へ戻す。`cargo xtask check` が、自分の担当ではない file の読み込み（`<path>: No such file ...`）や構文の error で失敗したとき（xtask は `--crate` で絞る前に workspace の全 file を読む）も、同じく 2 分待ってもう一度実行する（5 回まで。W12・W14 も同じく待たずに報告する）。
5. 試験（`cargo test`）を途中で回さない。各 task で走らせてよいのは、その task の「確認」に書いたコマンドだけ。長い試験は指揮役が検査点（C1、C2）で一度だけ回す。
6. `cargo xtask check` の全体は検査点でだけ回す。task の中では `cargo xtask check --no-write --only <rule> --crate <crate> --report` だけを使ってよい（速い）。
7. コードの規則（CONTRIBUTING.md の「コード」）を守る: すべての module と crate root の先頭に `//! Role: <英語 1 文>`、子 module を持つ module は `mod` と `use` だけ、`mod.rs` を作らない、関数は 80 行まで、file は code 1000 行まで、library で `unwrap()` を使わず `expect("invariant: …")`、glob import 禁止（例・雛形・E2E の `kivact::prelude::*` を除く）、コメントと rustdoc は英語、`print!`/`println!`/`eprintln!` を使わない（stdout に書くときは `std::io::stdout().lock()` へ `write!`）。`OnceLock`・`OnceCell`・`LazyLock`・`LazyCell` は使えない（clippy.toml）。
8. 新しい状態は設計書 3.11・4.3・4.5.1 の明示的な型で持つ。bool や Option を 2 つ並べて状態を表さない。
9. 外部 crate を新しく足さない（serde と serde_json は workspace にあるのでよい）。
10. git の操作は指揮役だけ。サブエージェントは commit・checkout・stash・reset をしない。
11. binary の crate（`kivact-devtools`、`kivact-devtools-mcp`）の module の中では `pub` ではなく `pub(crate)` を使う（`unreachable_pub` の警告が clippy で error になる）。`kivact-devtools` の `src/lib.rs` から試験に出す名前だけ `pub` にする。
12. W3 以降に struct・enum・static を作る・変える task は、最後に `cargo xtask check --no-write --only state-fields --crate <その crate> --report` を実行し、自分が作った・変えた型の「combines state fields: <field>, <field>」error を、型ごとに次の順で直す。「struct-level marker」「unused marker」の error と、自分が触っていない型の error は直さない（T15.4 がまとめて直す）。W1・W2 では、xtask を作り直している T1.1・T2.1 の担当と、指揮役の波の検査だけが `cargo xtask` を実行してよい（他の task は実行しない。動いている xtask の実行 file を作り直すと失敗するため）。
    1. その task の手順が、その型の field に書く印の文を指定している → 指定の文を、その field の直前に書く（印の位置は、`///` と `#[..]` の行の上。形は `// xtask-allow(state-fields): <分類> - <その field が何の事実か>`）。
    2. 指定が無く、型（か、下の対応表で元になった型）の行が付録の KEEP 表（各部の 2 章）か 7 章にある → error が並べた field の直前に、その行の分類で印を書く。分類の語は付録 7.4 の一覧の語か、その KEEP 表の行の分類をそのまま（設計書 5.3 の最後の段落）。「何の事実か」は、その field の `///` の最初の文（無ければ field の名前の `_` を空白にしたもの）。
    3. どちらも無い → 基準 commit に無かった型なら、設計書 3.11 の表と違う形で作ってある。その task を止めて、型名と error を報告する（0.2 の 1.）。基準 commit にあった型なら、直さず、報告に「T15.4 へ: <file> <型名> <error>」と書く。
    - 印は理由の文を省かない。分類が 2. と 1. のどれにも当てはまらないと感じても、型を作り変えない・分類を作らない（3. に従う）。

    分けた・移した型の元の行（T15.4 も同じ表を使う）:

    | 基準 commit の型（行のある付録の表） | 今の型 | 印の文の出どころ |
    |---|---|---|
    | `VirtualShell`（arvane-virtual `window.rs`、SM-CORE-06） | `VirtualPlatform`・`VirtualWindow` | T5.1 の 6. |
    | `Runner`（arvane-native `runtime.rs`）の `wake_reasons` | `Runner` | T4.1 の 16. |
    | `WinitWindow`（`runtime.rs:88`、KEEP 表の `WinitWindow.{subclass,monitor}`） | `port/winit/window.rs` の `WinitWindow`（名前は同じ） | KEEP 表の行をそのまま使う |
    | `Host`（engine `host/pump.rs`、KEEP 表の `Host.*`） | `Host`・`HostWindow` | T6.1 の 9. と 18. |
    | `Model`（engine `model/aggregate.rs`、KEEP 表の `Model.{editors,window_request,...}`） | `Model`・`AppModel` | T6.1 の 18. |
    | `ViewState`（core `view/state.rs:195`） | `ViewState`・`WindowRecord` | T7.1 の 5. と 7. |
    | `Window` の option（T7.1 が作る builder） | `Window` | T7.1 の 7. |
    | `Screen`（kivact-test `screen/model.rs`、`Screen.last_contact`・SM-CORE-07・15） | `TestApp`・`Screen` | T8.1 の 7. |

    表にある task の手順に印の文があれば、それが 1. に当たる。印の文が手順に無い field は、上の表の「基準 commit の型」の行の分類で 2. に従う。

### 0.3 サブエージェントを起動するときの指示文（指揮役がこのまま使う）

```
あなたは W:\w\kivact（git ブランチ feat/kashiwade/devtools-multi-window）で、作業指示書 WORK-ORDER-devtools-multi-window.md の task <TASK ID> だけを実装する。
最初に次を読むこと: 作業指示書の 0 章、task <TASK ID> の節、task が参照する設計書 DESIGN-devtools-multi-window.md の節、task が参照する付録 WORK-ORDER-appendix-state-machines.md の entry。
規則: 0.2 の規則をすべて守る。担当ファイル以外を編集しない。設計しない（合わない点は止めて報告）。cargo test は task の「確認」にあるものだけ。git は操作しない。
終わったら、(1) 変えたファイルの一覧、(2)「確認」のコマンドの結果（成功か、失敗ならエラーの要約）、(3) 合わなかった点・できなかった点、を 30 行以内で報告する。
```

### 0.4 波の進め方（指揮役）

1. `git status --short --untracked-files=no` が空であることを確かめる（前の波の commit が済んでいる）。作業の記録（`STOP-<task>.md`、`C1-FAILURES.md`、`STATE-FINDINGS.md`、検査の出力）は `target/plan/` に書く（`target/` は git に入らない）。最初の波（W0 の後、W1 の前）に `mkdir -p target/plan` を一度実行する（shell の `>` は存在しない directory へは書けない）。
2. その波の task を、表のとおり最大 6 体のサブエージェントに同時に渡す（0.3 の指示文）。
3. 全員の報告を待つ。報告に「合わない点」があれば、その task は止めて、内容を記録し、波の検査には進まない。オーナーに報告する（`target/plan/STOP-<task>.md` に書いて止まる）。
4. 波の検査（各波の表の下にあるコマンド）を回す。失敗したら、エラーの出た file の担当 task のサブエージェントを、エラーの全文を付けてもう一度起動する（「この error を直す。担当ファイルの中だけ」）。3 回直らなければ 3. と同じく止まる。
5. 検査が通ったら commit する。message は各波の表の下に書いた conventional commit の形。push はしない。

### 0.5 cargo の使い方

- target directory は共有する（既定の `target/`）。同時に cargo を動かすと「Blocking waiting for file lock」と出て待つが、それは正常。止めない。
- `cargo xtask check` は、`--crate` で絞っても、先に workspace の全 crate の file を読んで構文を調べる。並列の task が書いている途中の file（無い `mod` の file、書きかけの構文）があると、自分の担当の外の error で失敗する。これは規則 4 のとおり待って再実行する。
- 初回の `cargo check` は依存を全部ビルドするので時間がかかる（数分）。2 回目からは差分だけ。
- 実 window を開くコマンド（`cargo test` で App を起動するもの）は必ず `dev/tools/window-lock.sh -- <command>` を通す。task の中では走らせない。
- dev-dependency に `kivact`（facade）を持つ crate（`kivact-runtime`、`kivact-style`、`kivact-i18n`、`kivact-ui`）の `--all-targets` は facade 全体をビルドする。W3 から W8 までは facade が一時的にビルドできないので、これらの crate は W9 より前は `--lib` だけで確かめる（`cargo check -p kivact-runtime --lib`、`cargo clippy -p kivact-runtime --lib -- -D warnings`）。
- Linux 用のコード（`cfg(target_os = "linux")` の X11・Wayland）は Windows では compile されない。WSL に Rust があれば、指揮役は W4 と C1・C2 で `wsl -e bash -lc "cd /mnt/w/w/kivact && CARGO_TARGET_DIR=target/linux cargo check -p arvane-native --features embed --all-targets"`（C1・C2 では `--workspace --all-targets`）も実行する。WSL が無ければ報告に「Linux の compile は未確認」と書く。

---

## 1. 波の計画

依存の段（下ほど先）: kivact-types → arvane-shell・kivact-runtime・kivact-layout ほか → arvane-native・kivact-style ほか → arvane-virtual・arvane → kivact-engine → kivact-core → kivact-interaction・kivact-test → kivact-ui → kivact（facade）→ 5 つの kit・例。同じ波で同時に編集する crate は、互いに依存しないものだけにしてある。

| 波 | 並列の task（最大 6） | 前提 |
|---|---|---|
| W0 | T0.1（指揮役が自分で行う） | — |
| W1 | T1.1 xtask の state-fields、T1.2 protocol と client、T1.3 kit soft の状態、T1.4 kit creative の状態と改名、T1.5 Python の手書き部分、T1.6 複数ウィンドウの文書 | W0 |
| W2 | T2.1 xtask の生成器、T2.2 agent、T2.4 kivact-types、T2.5 DevTools のガイド | W1 |
| W3 | T3.1 arvane-shell、T3.2 kivact-runtime、T3.3 kivact-layout、T2.3 MCP | W2 |
| W4 | T4.1 arvane-native、T4.2 kivact-style、T3.4 新しい crate の state-fields | W3 |
| W5a | T5.1 arvane-virtual | W4 |
| W5b | T5.2 arvane（arvane の試験は arvane-virtual を使う） | W5a |
| W6 | T6.1 kivact-engine（複数ウィンドウ） | W5 |
| W7 | T7.1 kivact-core（複数ウィンドウ）、続けて T7.2 kivact-core（状態と style の層） | W6 |
| W8 | T8.1 kivact-test、T8.2 kivact-interaction | W7 |
| W9 | T9.1 kivact-ui（状態・Variants・recipe 名）→ T9.2 facade と kit の追従 → T9.3 複数ウィンドウの E2E（この順に 1 つずつ） | W8 |
| C1 | 検査点 1（指揮役） | W9 |
| W10 | T10.1 kivact-engine（DevTools の口） | C1 |
| W11 | T11.1 core の devtools module の土台 | W10 |
| W12 | T12.1〜T12.6 core の devtools module の各部 | W11 |
| W13 | T13.1 DevTools アプリの土台 | W12 |
| W14 | T14.1〜T14.6 DevTools アプリの各 panel | W13 |
| W15 | T15.1 DevTools の E2E、T15.2 Python の仕上げ、T15.3 README などの文書、T15.4 xtask の state-fields を有効化 | W14 |
| C2 | 検査点 2 と最終の片付け（指揮役） | W15 |

---

## 2. W0（指揮役）

### T0.1 新しい crate の骨組みと policy

担当ファイル: `Cargo.toml`、`Cargo.lock`、`dev/xtask/policy/workspace.toml`、`.gitignore`、`.gitattributes`、新しく作る 5 crate の `Cargo.toml` と最小の source、`python/kivact-devtools/` の骨組み。

手順:

1. root の `Cargo.toml` の `[workspace] members` に次の 5 行を足す（既存の並びの規則に合わせ、`"crates/kivact-test"` の次の行に置く）:
   `"crates/kivact-devtools-protocol"`、`"crates/kivact-devtools-agent"`、`"crates/kivact-devtools-client"`、`"crates/kivact-devtools"`、`"crates/kivact-devtools-mcp"`。
2. library 3 つの `Cargo.toml`（`crates/kivact-test/Cargo.toml` の `[package]` の書き方を写す。`authors`・`version`・`edition`・`license`・`repository` は `.workspace = true`、`description` は下の文）。root の `[workspace.dependencies]` に `serde`（`derive` 付き）・`serde_json`・`png`・`tracing` があるので、どれも `{ workspace = true }` と書く（新しい外部 crate ではない）:

   ```toml
   # crates/kivact-devtools-protocol/Cargo.toml
   [package]
   name = "kivact-devtools-protocol"
   authors.workspace = true
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   repository.workspace = true
   description = "Wire vocabulary of the Kivact DevTools: JSON-RPC methods, notifications and values."

   [dependencies]
   serde = { workspace = true }
   serde_json = { workspace = true }

   [lints]
   workspace = true
   ```

   `kivact-devtools-agent`: description「Server of the Kivact DevTools inside a target app.」、`[dependencies]` は `kivact-devtools-protocol = { path = "../kivact-devtools-protocol", version = "0.1.0" }`、`serde_json = { workspace = true }`、`png = { workspace = true }`、`tracing = { workspace = true }`。`[dev-dependencies]` に `kivact-devtools-client = { path = "../kivact-devtools-client", version = "0.1.0" }`（agent の試験が使う。policy の `visible_to` は下で足す）。
   `kivact-devtools-client`: description「Discovery and connection to a Kivact DevTools agent.」、依存は protocol（上と同じ書き方）と `serde_json = { workspace = true }`。
3. binary 2 つの `Cargo.toml`（`examples/kivact-ui-basic/Cargo.toml` の形を写す。`publish = false`。`authors`・`repository` は書かない）:

   ```toml
   # crates/kivact-devtools/Cargo.toml
   [package]
   name = "kivact-devtools"
   publish = false
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   description = "The Kivact DevTools app."

   [features]
   default = ["gpu"]
   gpu = ["kivact/gpu"]

   [dependencies]
   kivact = { path = "../kivact", version = "0.1.0", default-features = false }
   kivact-ui-basic = { path = "../kivact-ui-basic", version = "0.1.0" }
   kivact-devtools-client = { path = "../kivact-devtools-client", version = "0.1.0" }
   kivact-devtools-protocol = { path = "../kivact-devtools-protocol", version = "0.1.0" }
   serde_json = { workspace = true }

   [dev-dependencies]
   kivact = { path = "../kivact", version = "0.1.0", default-features = false, features = ["test"] }

   [lints]
   workspace = true
   ```

   `kivact-devtools-mcp`: description「MCP server that exposes a Kivact DevTools agent to AI tools.」、`publish = false`、依存は client・protocol・`serde_json`（上と同じ書き方）。
4. 最小の source（各 crate）:
   - library: `src/lib.rs` に `//! Role: <description と同じ内容の英語 1 文>` だけ。
   - binary: `src/main.rs` に `//! Role: …` と `fn main() {}`。
5. `dev/xtask/policy/workspace.toml`（後の task は、T1.1（`[state_fields]` と `[style] options_structs`）と T3.1（`[module_rules.arvane-shell.uses]`）を除いて policy を変えない。dev-dependency の利用者も入れてある）:
   - `[visible_to]` の「Kivact の内部 crate」の並びの後に次の 3 行を足す。`kivact-core` の行は変えない（visible_to は「誰がその crate を使ってよいか」の表）:

     ```toml
     kivact-devtools-protocol = ["kivact-devtools-agent", "kivact-devtools-client", "kivact-core", "kivact-devtools", "kivact-devtools-mcp", "kivact-public-api-tests"]
     kivact-devtools-agent = ["kivact-core"]
     kivact-devtools-client = ["kivact-devtools", "kivact-devtools-mcp", "kivact-devtools-agent", "kivact-public-api-tests"]
     ```

     「アプリ・例・雛形・ツール: 誰も依存しない」の並び（`kivact-docs = []` の近く）に次の 2 行を足す（`kivact-devtools` は後で `src/lib.rs` を持つので、書かないと誰でも依存できてしまう）:

     ```toml
     kivact-devtools = []
     kivact-devtools-mcp = []
     ```
   - `[analysis]` の `trusted_derives` の上に `trusted_attributes = ["serde"]` を足す（`#[serde(..)]` の attribute を信頼する。key は今無い）。`trusted_derives` に `Deserialize`・`Serialize` が既にあることを確かめる（ある）。
   - `[state_fields] value_modules` の配列の末尾に `"kivact-devtools-protocol"` と `"kivact-types::window"` を足す。
   - `[module_rules.kivact-core.uses]` に次の 1 行を足し、同じ table の `launch` の値の配列の末尾に `"devtools"` と `"geometry"` を、`app` の値の配列の末尾に `"geometry"` を足す（T7.1 が `check_window_sizes` を `geometry.rs` へ移し、`launch/settings.rs` と `app.rs` の試験から呼ぶ。`geometry` は `uses` の key なので、足さないと module の規則の error になる）。`client` と `view` の配列には `"devtools"` を足さない（`devtools` は `client` を参照してよいが、逆は許さない。`client` 側から DevTools へ届く必要がある処理は `view/devtools_hooks.rs` の hook を呼ぶ）:

     ```toml
     devtools = ["task", "motion", "style", "state", "geometry", "sync", "view", "platform", "diagnostics", "client"]
     ```

     `devtools` の module は `launch`・`embed`・`app`・`custom`・`draw`・`gpu_interop` を参照できない（`uses` の key で、この配列に無いため）。`launch::runtime` が `ClientMode` の判断を終えて `agent_slot(title, allowed, can_inject, waker)` に渡す（4.4.1）。
   - 最後の `[[style.exceptions]]`（`glob-imports` の並び。`rule = "glob-imports"` の最後の 1 つの次）に足す:

     ```toml
     [[style.exceptions]]
     rule = "glob-imports"
     path = "crates/kivact-devtools/*"
     reason = "The DevTools app is written as a Kivact user writes an app (dogfooding P-07): it starts from `use kivact::prelude::*`."
     ```

     DevTools アプリの中で protocol の型は `use kivact_devtools_protocol as p;` で `p::Role` のように書く（prelude の `Role`・`Rect`・`Size` とぶつけない）。protocol crate の `lib.rs` の `pub use <子 module>::*;` は「子 module の再公開」なので exception は要らない（xtask-core の `glob-imports` は、子 module を持つ module の中の、直接の子 module の glob 再公開を許す: `vendor/xtask-core/crates/xtask-core/src/style_source.rs` の `child_reexport`）。
6. `.gitignore` に設計書 6 章の行を足す（`__pycache__/`、`*.pyc`、`.venv/`、`*.egg-info/`、`python/kivact-devtools/build/`、`.kivact/review/`、`.kivact/traces/`）。
7. `.gitattributes` の末尾に次の 2 行を足す（生成物の改行を checkout で変えない。Windows の `core.autocrlf` で `cargo xtask devtools-codegen --check` が落ちないように）:

   ```
   crates/kivact-devtools-protocol/schema/*.json text eol=lf
   python/kivact-devtools/src/kivact_devtools/_generated.py text eol=lf
   ```
8. `python/kivact-devtools/` に設計書 4.7.1 の directory を作る（file は空でよい。`pyproject.toml` は次の内容）:

   ```toml
   [build-system]
   requires = ["setuptools>=68"]
   build-backend = "setuptools.build_meta"

   [project]
   name = "kivact-devtools"
   version = "0.1.0"
   description = "Typed Python client for the Kivact DevTools."
   requires-python = ">=3.10"
   dependencies = []

   [tool.setuptools.packages.find]
   where = ["src"]

   [tool.setuptools.package-data]
   kivact_devtools = ["py.typed"]
   ```

9. 確認: `cargo check -p kivact-devtools-protocol -p kivact-devtools-agent -p kivact-devtools-client -p kivact-devtools-mcp`。続けて `cargo check -p kivact-devtools`（kivact 全体をビルドするので時間がかかる）。`Cargo.lock` が更新される。network が要ると言われたら一度だけ network ありで実行する。
10. commit: `chore: scaffold the devtools crates and policy`。設計書・作業指示書・付録の 3 つの file はすでに commit してある（作業ブランチの最初の commit）。この時点で 3 つの file に変更があれば、それも同じ commit に含める。

---

## 3. W1

| task | 担当（サブエージェント） | crate・場所 |
|---|---|---|
| T1.1 | 1 | `dev/xtask` |
| T1.2 | 2 | `crates/kivact-devtools-protocol`、`crates/kivact-devtools-client` |
| T1.3 | 3 | `crates/kivact-ui-soft` |
| T1.4 | 4 | `crates/kivact-ui-creative`、`examples/kivact-ui-creative`、`docs/for-kivact-ui-user/*/creative` |
| T1.5 | 5 | `python/kivact-devtools`（`_generated.py` を除く） |
| T1.6 | 6 | `docs/`（複数ウィンドウの文書） |

波の検査: `cargo check -p xtask -p kivact-devtools-protocol -p kivact-devtools-client -p kivact-ui-soft -p kivact-ui-creative -p kivact-ui-creative-catalog`、`cargo test -p kivact-devtools-protocol`、`cargo test -p xtask`。commit: `feat: add the devtools protocol and client, strengthen state-fields, fix kit state machines`。

### T1.1 xtask: state-fields の強化

担当ファイル: `dev/xtask/src/state_fields.rs`（必要なら `dev/xtask/src/state_fields/` の leaf module に分け、`state_fields.rs` は `mod` と `use` だけにする）、`dev/xtask/src/check_source.rs`、`dev/xtask/src/local_checks.rs`、`dev/xtask/policy/workspace.toml` の `[state_fields]` の節と `[style] options_structs` の 2 行（下の 12.）。

参照: 設計書 5.3 節、付録 7 章。今の実装の説明: `state_fields.rs:11-84`（`check`）、`check_source.rs:42-60`（`Source::allows`）、`check_source.rs:71-87`（`Source::error` が項目の印でも黙らせる）、`local_checks.rs:98-133`（`model_types`）、`local_checks.rs:75-83`（policy の key の許可一覧）。

手順（設計書 5.3 を、上から順にそのまま実装する。設計書の番号と同じ）:

1. 構造体の前の印を無効にする: `state_fields::check` では `source.error(..)`（項目の印を見る）を使わず、`errors.push(..)` に直接積む関数を `check_source.rs` に足す（`Source::error_at(span, rule, detail)` として「項目の印を見ない」版を作る）。構造体の前（構造体の span の直前、doc と attribute の上）に `// xtask-allow(state-fields):` があれば、`"struct-level marker; put a marker before each field instead"` の error を出す。field の印は、field の span の直前（その field の `///` と `#[..]` の行の上）にあるものだけが効く（`Source::allows` の今の走査と同じ）。
2. 使われていない印（設計書 5.3 の 2.）: `Source` に「使われた印の行番号」の集合（`RefCell<BTreeSet<usize>>`。数えない型なので印を付けない）を足し、`allows` の兄弟として「印の行番号を返す」`Source::allow_line(rule, span) -> Option<usize>` を作る。`state_fields::check` は、各 field（tuple の要素・enum の variant の field・static を含む）について、先に 4. の分類をして、印のある field が「数える」（stored か phase）なら、その印の行を集合に入れる。数えない field の前の印は入れない。検査しない型（`NodeBuilder`、`value_modules`）の中の印も入れない。module の検査の最後に、`source.text` の中の `// xtask-allow(state-fields):` の行のうち、構造体の前のもの（1. の error になる）と集合にあるもの以外を `"unused marker"` の error にする。
3. tuple の field: `field.ty` が `syn::Type::Tuple` なら、要素ごとに数える。tuple struct（`struct X(A, B);`）も要素ごと。tuple 型の field の直前の印は、その要素すべてに効く。
4. 型の分類（設計書 5.3 の 4.〜6.）: 一番外側の型の名前は `syn::Type::Path` の最後の segment から取る（`&` と `Box` は見ない）。次の小さい関数を `state_fields/classify.rs`（leaf）に作る: `outer(ty) -> String`、`peel(ty, names) -> Type`（名前が `names` にある一番外側の wrapper を、型引数が 1 つの間くり返して剥がす）、`classify(name: &str, ty: &Type, exempt: bool, enums: &BTreeSet<String>, policy) -> Class`（`Class { None, Phase, Stored }`）。`classify` は設計書 5.3 の 4. の文を上から順に判定する。免除の型（`*Api`・role suffix・`record_types`）は stored のうち (iii)(iv) だけを `Stored` にし、phase を `None` にする。`NodeBuilder` を実装する struct と `value_modules` の module は検査しない（今のとおり）。数えない型の名前は `[state_fields]` の新しい key `named_helpers = ["Generation", "Sent", "ReportOnce", "ReportOnceSet", "Mailbox", "LocalMailbox", "Registry"]` として置き、`local_checks.rs:75-83` の許可一覧にも足す。phase の判定の「名前が `State`・`Phase` で終わる enum」は、今の `surface.resolve_type` と `enums`（`state_fields.rs:50-60` の今の判定）を使う。struct の error は、5. のとおり stored が 2 つ以上、または phase と stored が 1 つ以上のとき。
5. enum の variant（設計書 5.3 の 5.）: `syn::Item::Enum` の、名前付き field を持つ variant ごとに、4. と同じ判定をして同じ error を出す（error の名前は `Enum::Variant`）。tuple variant と unit variant は見ない。免除は enum の名前にも当てる。
6. module の `static` と `thread_local!`: `syn::Item::Static` と、path の最後が `thread_local` の `syn::Item::Macro`（中身は `syn::parse2` で `static` の並びとして読む）を数える。型の分類は 4. の stored の基準で、(v) の名前には static の名前を小文字にしたものを使う。印の無い stored が一つの module に 2 つ以上あれば、その module に `"{path}: static flags/counters combine"` の error（2 つ以上の static の名前を並べる）を出す。
7. 局所変数の報告（error にしない。設計書 5.3 の 7.）: `--report` のときだけ、関数の本体の `let` を `syn::visit::Visit` で辿り、設計書 5.3 の 7. の形を `state-locals: <path>:<line> <name>` として出す。`effect_writes.rs` の scope の辿り方（98-160 行）を参考にする。
8. 単体試験: `state_fields.rs` の既存の試験（86-155 行、`check_source::fixture`）の形で、2.〜7. のそれぞれに「error になる例」と「error にならない例」を 1 つずつ足す。必ず含める例（`Name { a, b }` は fixture の中の型）: `Signal<Vec<bool>>` は数えない・`Signal<Option<X>>` と `Mutex<Option<X>>` と `Arc<AtomicUsize>` は数える・`Option<Callback<()>>` は数えない・`Var<Option<Callback<()>>>` は数える・`Value<bool>` と `Cell<Generation>` は数えない・phase と `Value` だけは error にならない・`MachineHandle` と `Option` は error になる・`*Api` の `Option` は数えず `Signal<Option<X>>` は数える・enum の variant の `Option` 2 つは error・module の `AtomicBool` の static 2 つは error・数えない field の前の印は "unused marker"・全部の stored field に印を書いた型は error にならず "unused marker" も出ない。
9. 確認: `cargo test -p xtask`、`cargo check -p xtask`。`mkdir -p target/plan` の後、`cargo xtask check --no-write --only state-fields --report > target/plan/state-fields-w1.txt` を実行する（この時点では大量の error が出るのが正しい。直すのは後の task）。出力から次を数えて報告に書く: (a)「struct-level marker」の行数、(b)「unused marker」の行数、(c)「combines state fields」の行のうち、その型の直前に struct-level marker が**無い**ものの型名の一覧。(c) を付録 7 章の表（7.1 の型、7.2 の static、7.3 の `ExportRequest::GpuProvided` と `TimelineCell`）と突き合わせ、表に無い型と、表にあるのに出なかった型を、報告の「7 章との差」に書く（この task では直さない。差があれば指揮役が T1.1 を規則の読み違いとして起動し直す）。W1 の他の task（T1.3・T1.4）が同時に直している型は、差に数えてよい（その旨を書く）。
10. この task では既存の印を動かさない・消さない。検査を有効にした状態で W1 を commit してよい（`cargo xtask check` の全体は C1 まで回さない）。
11. error の message に、印の無い stored の field の名前だけを並べる（phase の field と、印のある field は並べない。rule 12 と T15.4 は並んだ field に印を書く）: 今の `"{Name} combines state fields"`（`state_fields.rs:73-79`）を `"{Name} combines state fields: {field}, {field}"` にする（T15.4 が印を付ける field を知るため）。同じ行に複数の field がある variant でも、field の名前を並べる。
12. `dev/xtask/policy/workspace.toml` の `[style] options_structs` から次の 2 行分を直す: `"kivact-ui-creative::WindowOptions"` を `"kivact-ui-creative::ToolWindowOptions"` に変える（T1.4 の改名に合わせる）。`"arvane-shell::ShellOptions"` を消す（T3.1 が `ShellOptions` を消し、main window は `WindowRequest::main(title, size)` で作るため）。
### T1.2 protocol と client

担当ファイル: `crates/kivact-devtools-protocol/**`、`crates/kivact-devtools-client/**`。

参照: 設計書 4.2 節のすべて。

共通の書き方（この task のすべての file）:

- `Mutex` は `lock().expect("invariant: the lock is never poisoned")`。`PoisonError::into_inner` などは使えない（xtask の `poison-access`）。
- `use` は 1 つの module から 1 文（`use crate::app::{A, B};`）。`use a::{b::C, d::E};` のように入れ子の path は書けない。
- protocol の module 同士は `crate::<module>::<名前>` の path で参照する（`crate::Call` のような root の再公開は使わない）。参照の向きは `common` ← 型の module ← `call`・`notify` ← `rpc` の一方向（循環させない）。

手順（protocol）:

1. module の構成。`src/lib.rs` は `mod` と `pub use` だけ（`pub use <module>::*;` は、子 module を持つ `lib.rs` の中の直接の子 module の再公開なので glob の規則に触れない）。`code` だけは `pub mod code;`（`code::NODE_NOT_FOUND` と書けるように。`pub use` はしない）:

   ```rust
   //! Role: Wire vocabulary of the Kivact DevTools: methods, notifications and values.
   mod a11y;
   mod app;
   mod call;
   pub mod code;
   mod common;
   mod config;
   mod console;
   mod discovery;
   mod emulation;
   mod input;
   mod layout;
   mod motion;
   mod node;
   mod notify;
   mod overlay;
   mod perf;
   mod query;
   mod review;
   mod review_markdown;
   mod rpc;
   mod scripts;
   mod style;
   mod tasks;
   mod time;
   mod trace;
   mod tree;
   mod updates;
   pub use a11y::*;
   pub use app::*;
   pub use call::*;
   pub use common::*;
   pub use config::*;
   pub use console::*;
   pub use discovery::*;
   pub use emulation::*;
   pub use input::*;
   pub use layout::*;
   pub use motion::*;
   pub use node::*;
   pub use notify::*;
   pub use overlay::*;
   pub use perf::*;
   pub use query::*;
   pub use review::*;
   pub use review_markdown::*;
   pub use rpc::*;
   pub use scripts::*;
   pub use style::*;
   pub use tasks::*;
   pub use time::*;
   pub use trace::*;
   pub use tree::*;
   pub use updates::*;
   ```

   （`schema` module は T2.1 が足す。）`PROTOCOL_VERSION` は `app.rs` に置く。
2. 設計書 4.2.4 の型を、見出しの module 名の leaf file にそのまま書く（`common.rs`・`app.rs`・`tree.rs`・`node.rs`・`query.rs`・`overlay.rs`・`style.rs`・`updates.rs`・`perf.rs`・`a11y.rs`・`layout.rs`・`input.rs`・`motion.rs`・`tasks.rs`・`console.rs`・`emulation.rs`・`review.rs`・`scripts.rs`。`config.rs` は 8.）。規則は 4.2.3。英語の `///` をすべての型と field に付ける。各 file の先頭は `//! Role: <英語 1 文>` と `use serde::{Deserialize, Serialize};`、他の module の型を使う file はその `use crate::<module>::{..};`。`Role` と `A11yAction` の variant は、書く前に `crates/kivact-types/src/semantics/values.rs` の `pub enum Role`・`pub enum A11yAction` と見比べ、同じ順・同じ名前で写す（設計書 4.2.4 に写してある。違いがあれば止めて報告）。`MouseButton` は `Primary`・`Secondary`・`Middle` の 3 つ（`Modifier` の型は作らない）。
3. `call.rs`:

   ```rust
   //! Role: The DevTools methods, declared once: the `Call` and `Reply` enums, wire names and typed markers.

   use serde::de::DeserializeOwned;
   use serde::{Deserialize, Serialize};
   use crate::a11y::{A11yTree, AuditReport, FocusInfo, TabOrder, WindowParams};
   use crate::app::{AppInfo, InitializeParams, InitializeResult, SubscribeParams, WindowList};
   use crate::common::Empty;
   use crate::config::DevtoolsConfig;
   use crate::console::{ErrorList, ListErrorsParams};
   use crate::emulation::Emulation;
   use crate::input::{ImeState, InjectInputParams, InputLog, ListInputsParams};
   use crate::layout::LayoutLint;
   use crate::motion::{AnimationList, TimeScaleParams};
   use crate::node::{GetNodeParams, NodeDetail};
   use crate::overlay::{HighlightParams, Image, OverlaySettings, ScreenshotParams, StartPickParams};
   use crate::perf::{Counters, DiffSnapshotsParams, ExportTraceParams, ExportedFiles, OwnerTree, SnapshotDiff, SnapshotInfo};
   use crate::query::{FoundNode, FoundNodes, MaybeNode, RoleQuery, TestIdQuery, TextQuery};
   use crate::review::{AddCommentParams, EditCommentParams, ExportedReview, RemoveCommentParams, ReviewDocument, ReviewItem};
   use crate::scripts::ShowTableParams;
   use crate::style::{ClearStyleOverridesParams, ForceStateParams, SetStyleOverrideParams, StyleOverride, StyleOverrideList, ThemeTokenList};
   use crate::tasks::TaskList;
   use crate::tree::{GetTreeParams, TreeSnapshot};
   use crate::updates::{FrameList, ListFramesParams, ListTurnsParams, RecordingSettings, TurnList};

   /// A method of the protocol: its wire name and the types it takes and returns.
   pub trait Method {
       /// The method name on the wire.
       const NAME: &'static str;
       /// The `params` type.
       type Params: Serialize + DeserializeOwned;
       /// The `result` type.
       type Result: Serialize + DeserializeOwned;
   }

   macro_rules! protocol_methods {
       ($( $(#[doc = $doc:literal])* $variant:ident => $wire:literal ($params:ty) -> $result:ty; )*) => {
           /// A request from a client: the JSON-RPC `method` and its `params`.
           #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
           #[serde(tag = "method", content = "params")]
           #[allow(clippy::large_enum_variant, reason = "one short-lived value per call; boxing would only add noise")]
           pub enum Call {
               $( $(#[doc = $doc])* #[serde(rename = $wire)] $variant($params), )*
           }
           /// The result of one call.
           #[derive(Clone, Debug, PartialEq)]
           #[allow(clippy::large_enum_variant, reason = "one short-lived value per call; boxing would only add noise")]
           pub enum Reply {
               $( $(#[doc = $doc])* $variant($result), )*
           }
           impl Call {
               /// The method name on the wire.
               pub fn method(&self) -> &'static str {
                   match self { $( Call::$variant(_) => $wire, )* }
               }
           }
           impl Reply {
               /// The JSON-RPC `result` value.
               pub fn into_value(self) -> serde_json::Value {
                   match self {
                       $( Reply::$variant(value) => serde_json::to_value(value)
                           .expect("invariant: protocol values serialize"), )*
                   }
               }
           }
           /// Every method name, in declaration order.
           pub const METHOD_NAMES: &[&str] = &[$($wire),*];
           $(
               $(#[doc = $doc])*
               pub struct $variant;
               impl Method for $variant {
                   const NAME: &'static str = $wire;
                   type Params = $params;
                   type Result = $result;
               }
           )*
       };
   }
   ```

   marker の struct の名前と `Call` の variant の名前は同じだが、型と variant は名前空間が違うのでぶつからない。marker は `call.rs` の中に置き、`lib.rs` の `pub use call::*;` で出る（client は `kivact_devtools_protocol::GetTree` で呼べる）。続けて、macro の呼び出しを次のとおりに書く（設計書 4.2.5 の全 60 個。順番もこのとおり。variant 名は wire 名から 4.2.3 の 10 の規則で決まっている）:

   ```rust
   protocol_methods! {
       /// Start a session (the first request on a connection).
       Initialize => "initialize" (InitializeParams) -> InitializeResult;
       /// Choose the notification topics of this connection (replaces the earlier choice).
       Subscribe => "subscribe" (SubscribeParams) -> Empty;
       /// The app: name, process, build and workspace root.
       GetAppInfo => "get_app_info" (Empty) -> AppInfo;
       /// The DevTools configuration of the target, with defaults filled in.
       GetConfig => "get_config" (Empty) -> DevtoolsConfig;
       /// Every open window.
       ListWindows => "list_windows" (Empty) -> WindowList;
       /// The element tree of one window.
       GetTree => "get_tree" (GetTreeParams) -> TreeSnapshot;
       /// Everything known about one element.
       GetNode => "get_node" (GetNodeParams) -> NodeDetail;
       /// The one element with this role (and name); fails when there is none or more than one.
       GetByRole => "get_by_role" (RoleQuery) -> FoundNode;
       /// The element with this role (and name), or none; fails when there is more than one.
       QueryByRole => "query_by_role" (RoleQuery) -> MaybeNode;
       /// Like `get_by_role`, but waits up to `timeout_ms` for the element to appear.
       FindByRole => "find_by_role" (RoleQuery) -> FoundNode;
       /// Every element with this role (and name).
       AllByRole => "all_by_role" (RoleQuery) -> FoundNodes;
       /// The one element with this text; fails when there is none or more than one.
       GetByText => "get_by_text" (TextQuery) -> FoundNode;
       /// The element with this text, or none; fails when there is more than one.
       QueryByText => "query_by_text" (TextQuery) -> MaybeNode;
       /// Like `get_by_text`, but waits up to `timeout_ms` for the element to appear.
       FindByText => "find_by_text" (TextQuery) -> FoundNode;
       /// Every element with this text.
       AllByText => "all_by_text" (TextQuery) -> FoundNodes;
       /// The one element with this test id; fails when there is none or more than one.
       GetByTestId => "get_by_test_id" (TestIdQuery) -> FoundNode;
       /// The element with this test id, or none; fails when there is more than one.
       QueryByTestId => "query_by_test_id" (TestIdQuery) -> MaybeNode;
       /// Like `get_by_test_id`, but waits up to `timeout_ms` for the element to appear.
       FindByTestId => "find_by_test_id" (TestIdQuery) -> FoundNode;
       /// Every element with this test id.
       AllByTestId => "all_by_test_id" (TestIdQuery) -> FoundNodes;
       /// Highlight an element on screen (`node: None` clears it).
       Highlight => "highlight" (HighlightParams) -> Empty;
       /// Choose the debugging overlays drawn over every window.
       SetOverlays => "set_overlays" (OverlaySettings) -> Empty;
       /// The debugging overlays that are on now.
       GetOverlays => "get_overlays" (Empty) -> OverlaySettings;
       /// Start pick mode in a window: the next click selects an element instead of reaching the app.
       StartPick => "start_pick" (StartPickParams) -> Empty;
       /// Leave pick mode.
       StopPick => "stop_pick" (Empty) -> Empty;
       /// A PNG of a window or one element.
       Screenshot => "screenshot" (ScreenshotParams) -> Image;
       /// Override one style property of an element, a call site or a theme token.
       SetStyleOverride => "set_style_override" (SetStyleOverrideParams) -> StyleOverride;
       /// Remove one style override, or all of them.
       ClearStyleOverrides => "clear_style_overrides" (ClearStyleOverridesParams) -> Empty;
       /// Every style override that is set.
       ListStyleOverrides => "list_style_overrides" (Empty) -> StyleOverrideList;
       /// Force interactive states (hover, pressed, ...) on an element; an empty list releases them.
       ForceState => "force_state" (ForceStateParams) -> Empty;
       /// The theme tokens of the main window.
       ListThemeTokens => "list_theme_tokens" (Empty) -> ThemeTokenList;
       /// Start recording turns, frames and input (replaces the settings when already recording).
       StartRecording => "start_recording" (RecordingSettings) -> Empty;
       /// Finish the turns that are waiting for their frame, then stop recording.
       StopRecording => "stop_recording" (Empty) -> Empty;
       /// What is being recorded (all false when not recording).
       GetRecording => "get_recording" (Empty) -> RecordingSettings;
       /// Recorded turns with their cause graphs; `wait_ms` waits for a new one.
       ListTurns => "list_turns" (ListTurnsParams) -> TurnList;
       /// Recorded frames.
       ListFrames => "list_frames" (ListFramesParams) -> FrameList;
       /// Counts of live objects, owners, timers and so on.
       GetCounters => "get_counters" (Empty) -> Counters;
       /// Save the list of live owners and objects.
       TakeSnapshot => "take_snapshot" (Empty) -> SnapshotInfo;
       /// What was added and removed between two snapshots.
       DiffSnapshots => "diff_snapshots" (DiffSnapshotsParams) -> SnapshotDiff;
       /// The owner tree of a window.
       GetOwners => "get_owners" (WindowParams) -> OwnerTree;
       /// Write the recorded turns and frames as a Chrome trace file.
       ExportTrace => "export_trace" (ExportTraceParams) -> ExportedFiles;
       /// The accessibility tree of a window.
       GetA11yTree => "get_a11y_tree" (WindowParams) -> A11yTree;
       /// Accessibility problems found in a window.
       AuditA11y => "audit_a11y" (WindowParams) -> AuditReport;
       /// The focused element and the focus history of a window.
       GetFocus => "get_focus" (WindowParams) -> FocusInfo;
       /// The Tab order of a window.
       GetTabOrder => "get_tab_order" (WindowParams) -> TabOrder;
       /// Layout problems found in a window.
       LintLayout => "lint_layout" (WindowParams) -> LayoutLint;
       /// Inject input into a window; answers after the turn that handled it.
       InjectInput => "inject_input" (InjectInputParams) -> Empty;
       /// The recorded input events.
       ListInputs => "list_inputs" (ListInputsParams) -> InputLog;
       /// The text input (IME) state of a window.
       GetImeState => "get_ime_state" (WindowParams) -> ImeState;
       /// The running animations.
       ListAnimations => "list_animations" (Empty) -> AnimationList;
       /// Slow down, pause or restore the animation clock.
       SetTimeScale => "set_time_scale" (TimeScaleParams) -> Empty;
       /// The tasks, resources, actions and timers.
       ListTasks => "list_tasks" (Empty) -> TaskList;
       /// The errors the app reported.
       ListErrors => "list_errors" (ListErrorsParams) -> ErrorList;
       /// The environment emulation that is set.
       GetEmulation => "get_emulation" (Empty) -> Emulation;
       /// Emulate another environment (theme, locale, scale, touch); replaces the whole setting.
       SetEmulation => "set_emulation" (Emulation) -> Emulation;
       /// The review of this session.
       GetReview => "get_review" (Empty) -> ReviewDocument;
       /// Add a comment to an element (or a window) of the review.
       AddComment => "add_comment" (AddCommentParams) -> ReviewItem;
       /// Change the text of a comment.
       EditComment => "edit_comment" (EditCommentParams) -> ReviewItem;
       /// Remove a comment.
       RemoveComment => "remove_comment" (RemoveCommentParams) -> Empty;
       /// Write the review as `review.json` and `review.md` and return their paths.
       ExportReview => "export_review" (Empty) -> ExportedReview;
       /// Show a table in the DevTools app (for scripts).
       ShowTable => "show_table" (ShowTableParams) -> Empty;
   }
   ```
4. `notify.rs`: 同じ形の macro（`$variant:ident => $wire:literal ($params:ty) in $topic:ident;`）とその呼び出し。`use` は `serde::{Deserialize, Serialize}` と、`crate::app::{Topic, WindowList}`、`crate::console::ErrorEntry`、`crate::overlay::PickEvent`、`crate::scripts::ShowTableParams`、`crate::common::Empty`、`crate::tree::TreeChanged`、`crate::updates::{FrameInfo, TurnGraph}`:

   ```rust
   macro_rules! protocol_notifications {
       ($( $(#[doc = $doc:literal])* $variant:ident => $wire:literal ($params:ty) in $topic:ident; )*) => {
           /// A notification from the agent: the JSON-RPC `method` and its `params`.
           #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
           #[serde(tag = "method", content = "params")]
           #[allow(clippy::large_enum_variant, reason = "one short-lived value per notification; boxing would only add noise")]
           pub enum Notification {
               $( $(#[doc = $doc])* #[serde(rename = $wire)] $variant($params), )*
           }
           impl Notification {
               /// The method name on the wire.
               pub fn method(&self) -> &'static str {
                   match self { $( Notification::$variant(_) => $wire, )* }
               }
               /// The topic a connection subscribes to for this notification.
               pub fn topic(&self) -> Topic {
                   match self { $( Notification::$variant(_) => Topic::$topic, )* }
               }
           }
       };
   }

   protocol_notifications! {
       /// The element tree of a window changed.
       TreeChanged => "tree_changed" (TreeChanged) in Tree;
       /// A recorded turn is complete.
       TurnRecorded => "turn_recorded" (TurnGraph) in Turns;
       /// A recorded frame was presented.
       FrameRecorded => "frame_recorded" (FrameInfo) in Frames;
       /// Pick mode hovered, selected or cancelled.
       Pick => "pick" (PickEvent) in Pick;
       /// The app reported a new error.
       ErrorReported => "error_reported" (ErrorEntry) in Errors;
       /// A key asked for DevTools while a DevTools app is already connected.
       FocusRequested => "focus_requested" (Empty) in Devtools;
       /// A script showed a table.
       ScriptTable => "script_table" (ShowTableParams) in Scripts;
       /// A window opened, closed or changed its title.
       WindowsChanged => "windows_changed" (WindowList) in Windows;
   }
   ```
5. `code.rs`（leaf。`pub mod code` を `rpc.rs` の中に作らない）:

   ```rust
   //! Role: JSON-RPC error codes of the DevTools protocol (design 4.2.1).

   pub const PARSE_ERROR: i32 = -32700;
   pub const INVALID_REQUEST: i32 = -32600;
   pub const METHOD_NOT_FOUND: i32 = -32601;
   pub const INVALID_PARAMS: i32 = -32602;
   pub const INTERNAL_ERROR: i32 = -32603;
   pub const BAD_TOKEN: i32 = -32001;
   pub const NOT_INITIALIZED: i32 = -32002;
   pub const VERSION_MISMATCH: i32 = -32003;
   pub const NODE_NOT_FOUND: i32 = -32010;
   pub const WINDOW_NOT_FOUND: i32 = -32011;
   pub const NO_MATCH: i32 = -32012;
   pub const MULTIPLE_MATCHES: i32 = -32013;
   pub const TIMED_OUT: i32 = -32014;
   pub const UNSUPPORTED: i32 = -32015;
   ```

   各行の上に英語の `///` を 1 行付ける（`missing-docs` は検査されないが、生成器が description に使うので付ける）。
6. `rpc.rs`:

   ```rust
   //! Role: JSON-RPC 2.0 envelopes and the reading and writing of one line.

   use serde::{Deserialize, Serialize};
   use serde_json::{Value, json};
   use crate::call::{Call, METHOD_NAMES};
   use crate::code;
   use crate::notify::Notification;

   /// One JSON-RPC request as read from the wire.
   #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
   pub struct RpcRequest {
       pub jsonrpc: String,
       pub id: Value,
       pub method: String,
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub params: Option<Value>,
   }
   /// A JSON-RPC error object.
   #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
   pub struct RpcError {
       pub code: i32,
       pub message: String,
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub data: Option<Value>,
   }
   /// One JSON-RPC response as read from the wire.
   #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
   pub struct RpcResponse {
       pub jsonrpc: String,
       pub id: Value,
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub result: Option<Value>,
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub error: Option<RpcError>,
   }
   /// A line from the agent: a response to a call, or a notification.
   #[derive(Clone, Debug, PartialEq)]
   pub enum Message {
       Response(RpcResponse),
       Notification(Notification),
   }

   impl RpcError {
       /// An error with this code (see `code`) and no `data`.
       pub fn new(code: i32, message: impl Into<String>) -> RpcError {
           RpcError { code, message: message.into(), data: None }
       }
   }

   /// Read a request line. Not JSON → `PARSE_ERROR`; not a JSON-RPC 2.0 request → `INVALID_REQUEST`.
   pub fn parse_request(line: &str) -> Result<RpcRequest, RpcError> {
       let value: Value = serde_json::from_str(line)
           .map_err(|error| RpcError::new(code::PARSE_ERROR, error.to_string()))?;
       let request: RpcRequest = serde_json::from_value(value)
           .map_err(|error| RpcError::new(code::INVALID_REQUEST, error.to_string()))?;
       if request.jsonrpc != "2.0" {
           return Err(RpcError::new(code::INVALID_REQUEST, "jsonrpc must be \"2.0\""));
       }
       Ok(request)
   }

   /// Turn a request into a `Call`. Unknown method → `METHOD_NOT_FOUND`; bad params → `INVALID_PARAMS`.
   /// Missing or `null` params are read as `{}` (so `Empty` methods need no params).
   pub fn parse_call(request: &RpcRequest) -> Result<Call, RpcError> {
       if !METHOD_NAMES.contains(&request.method.as_str()) {
           return Err(RpcError::new(code::METHOD_NOT_FOUND, format!("unknown method: {}", request.method)));
       }
       let envelope = json!({ "method": request.method, "params": params_or_empty(request.params.as_ref()) });
       serde_json::from_value(envelope)
           .map_err(|error| RpcError::new(code::INVALID_PARAMS, error.to_string()))
   }

   /// Read one line from the agent (a response or a notification). A line this
   /// client does not understand is an `Err`; clients ignore it (a newer target).
   pub fn parse_message(line: &str) -> Result<Message, RpcError> {
       let value: Value = serde_json::from_str(line)
           .map_err(|error| RpcError::new(code::PARSE_ERROR, error.to_string()))?;
       if value.get("method").is_some() {
           let envelope = json!({ "method": value["method"], "params": params_or_empty(value.get("params")) });
           return serde_json::from_value(envelope)
               .map(Message::Notification)
               .map_err(|error| RpcError::new(code::INVALID_PARAMS, error.to_string()));
       }
       serde_json::from_value(value)
           .map(Message::Response)
           .map_err(|error| RpcError::new(code::INVALID_REQUEST, error.to_string()))
   }

   /// The response for `id`.
   pub fn response(id: Value, result: Result<Value, RpcError>) -> RpcResponse {
       match result {
           Ok(value) => RpcResponse { jsonrpc: "2.0".to_owned(), id, result: Some(value), error: None },
           Err(error) => RpcResponse { jsonrpc: "2.0".to_owned(), id, result: None, error: Some(error) },
       }
   }

   /// One response as a line (no trailing newline).
   pub fn response_line(response: &RpcResponse) -> String {
       serde_json::to_string(response).expect("invariant: protocol values serialize")
   }

   /// One request as a line (no trailing newline).
   pub fn request_line(id: u64, method: &str, params: Value) -> String {
       json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string()
   }

   /// One notification as a line (no trailing newline): `{"jsonrpc":"2.0","method":..,"params":..}`.
   pub fn notification_line(notification: &Notification) -> String {
       let mut value = serde_json::to_value(notification).expect("invariant: protocol values serialize");
       value["jsonrpc"] = json!("2.0");
       value.to_string()
   }

   fn params_or_empty(params: Option<&Value>) -> Value {
       match params {
           None | Some(Value::Null) => json!({}),
           Some(value) => value.clone(),
       }
   }
   ```

   （`RpcRequest` などの field の `///` と、`Message` の variant の `///` も付ける。）
7. `time.rs`: `pub fn rfc3339_utc(unix_ms: u64) -> String`（`YYYY-MM-DDTHH:MM:SSZ`。秒未満は書かない）。日付は「Unix 日数 → 年月日」の civil-from-days の算法（Howard Hinnant の公開の算法）で計算する:

   ```rust
   //! Role: UTC date-time text from a Unix time without an external crate.

   /// `2026-11-08T05:32:00Z` for the Unix time `unix_ms` (milliseconds).
   pub fn rfc3339_utc(unix_ms: u64) -> String {
       let seconds = unix_ms / 1000;
       let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
       let of_day = seconds % 86_400;
       format!(
           "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
           of_day / 3600,
           of_day % 3600 / 60,
           of_day % 60
       )
   }

   fn civil_from_days(days: i64) -> (i64, u32, u32) {
       let z = days + 719_468;
       let era = z.div_euclid(146_097);
       let doe = z.rem_euclid(146_097);
       let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
       let y = yoe + era * 400;
       let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
       let mp = (5 * doy + 2) / 153;
       let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
       let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
       (if m <= 2 { y + 1 } else { y }, m, d)
   }
   ```
8. `config.rs`（4.2.7。wire の型でもあるので 4.2.3 の形で書く。`Default` も derive してよい）:

   ```rust
   //! Role: The `.kivact/devtools.json` configuration and its defaults.

   use serde::{Deserialize, Serialize};

   /// Keys that open DevTools (`"Ctrl+Shift+I"` style chords). `None` means the default; `[]` disables.
   #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
   pub struct KeyConfig {
       /// Chords that open DevTools.
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub open: Option<Vec<String>>,
       /// Chords that open DevTools in pick mode.
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub pick: Option<Vec<String>>,
   }
   /// The DevTools configuration read from `.kivact/devtools.json`.
   #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
   pub struct DevtoolsConfig {
       /// Key chords.
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub keys: Option<KeyConfig>,
       /// Path of the DevTools app (relative to the workspace root, or absolute).
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub app: Option<String>,
       /// Path of the Python interpreter for scripts.
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub python: Option<String>,
       /// Directory of the scripts (relative to the workspace root, or absolute).
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub scripts_dir: Option<String>,
       /// The command that opens a source location; `{file}`, `{line}` and `{column}` are replaced. Empty means the default.
       #[serde(default, skip_serializing_if = "Vec::is_empty")]
       pub editor: Vec<String>,
       /// The `$schema` key of the file (ignored).
       #[serde(default, rename = "$schema", skip_serializing_if = "Option::is_none")]
       pub schema: Option<String>,
   }
   ```

   続けて `impl DevtoolsConfig`（4 つの method と `with_defaults`。既定は設計書 4.2.7）:

   ```rust
   const OPEN_KEYS: [&str; 2] = ["F12", "Ctrl+Shift+I"];
   const PICK_KEYS: [&str; 1] = ["Ctrl+Shift+C"];
   const SCRIPTS_DIR: &str = ".kivact/devtools/scripts";
   const EDITOR: [&str; 3] = ["code", "--goto", "{file}:{line}:{column}"];

   fn owned(items: &[&str]) -> Vec<String> {
       items.iter().map(|item| (*item).to_owned()).collect()
   }

   impl DevtoolsConfig {
       /// The chords that open DevTools (the default when `keys.open` is not set).
       pub fn open_keys(&self) -> Vec<String> {
           match self.keys.as_ref().and_then(|keys| keys.open.as_deref()) {
               Some(chords) => chords.to_vec(),
               None => owned(&OPEN_KEYS),
           }
       }
       /// The chords that open DevTools in pick mode (the default when `keys.pick` is not set).
       pub fn pick_keys(&self) -> Vec<String> {
           match self.keys.as_ref().and_then(|keys| keys.pick.as_deref()) {
               Some(chords) => chords.to_vec(),
               None => owned(&PICK_KEYS),
           }
       }
       /// The scripts directory text (the default when not set).
       pub fn scripts_dir(&self) -> String {
           self.scripts_dir.clone().unwrap_or_else(|| SCRIPTS_DIR.to_owned())
       }
       /// The editor command (the default when empty).
       pub fn editor_command(&self) -> Vec<String> {
           if self.editor.is_empty() { owned(&EDITOR) } else { self.editor.clone() }
       }
       /// A copy with every default filled in (the answer of `get_config`).
       pub fn with_defaults(&self) -> DevtoolsConfig {
           DevtoolsConfig {
               keys: Some(KeyConfig { open: Some(self.open_keys()), pick: Some(self.pick_keys()) }),
               scripts_dir: Some(self.scripts_dir()),
               editor: self.editor_command(),
               schema: None,
               ..self.clone()
           }
       }
   }
   ```
9. `discovery.rs`（純粋。I/O をしない）:

   ```rust
   //! Role: Where the discovery files of the DevTools agents live (pure: the caller passes the environment).

   use std::ffi::OsString;
   use std::path::PathBuf;

   /// The discovery directory: `%LOCALAPPDATA%\kivact\devtools`, else `$XDG_RUNTIME_DIR/kivact/devtools`,
   /// else `$HOME/.cache/kivact/devtools`, else `<temp_dir>/kivact-devtools`. Empty values count as unset.
   /// Callers pass `|name| std::env::var_os(name)` and `std::env::temp_dir()`.
   pub fn discovery_dir_from(var: impl Fn(&str) -> Option<OsString>, temp_dir: PathBuf) -> PathBuf {
       let set = |name: &str| var(name).filter(|value| !value.is_empty());
       if let Some(base) = set("LOCALAPPDATA") {
           return PathBuf::from(base).join("kivact").join("devtools");
       }
       if let Some(base) = set("XDG_RUNTIME_DIR") {
           return PathBuf::from(base).join("kivact").join("devtools");
       }
       if let Some(home) = set("HOME") {
           return PathBuf::from(home).join(".cache").join("kivact").join("devtools");
       }
       temp_dir.join("kivact-devtools")
   }

   /// `<pid>-<port>.json`.
   pub fn discovery_file_name(pid: u64, port: u32) -> String {
       format!("{pid}-{port}.json")
   }
   ```
10. `trace.rs`（純粋。Chrome Trace Event Format。時刻は µs。turn は `tid` 0、frame は `tid` を window の id にする）:

    ```rust
    //! Role: Chrome Trace Event Format text of recorded turns and frames (pure).

    use serde_json::{Value, json};
    use crate::updates::{FrameInfo, TurnGraph};

    /// `{"traceEvents": [...]}`: a `turn N` event with one child event per stage, and a `frame` event per frame.
    pub fn chrome_trace(turns: &[TurnGraph], frames: &[FrameInfo]) -> String {
        let mut events: Vec<Value> = Vec::new();
        for turn in turns {
            events.push(complete(format!("turn {}", turn.turn), turn.started_ns, turn.duration_ns, 0));
            for stage in &turn.stages {
                events.push(complete(format!("{:?}", stage.stage), stage.start_ns, stage.duration_ns, 0));
            }
        }
        for frame in frames {
            let start = frame.end_ns.saturating_sub(frame.work_ns);
            events.push(complete("frame".to_owned(), start, frame.work_ns, frame.window));
        }
        json!({ "traceEvents": events }).to_string()
    }

    fn complete(name: String, start_ns: u64, duration_ns: u64, tid: u64) -> Value {
        json!({ "name": name, "ph": "X", "ts": micros(start_ns), "dur": micros(duration_ns), "pid": 1, "tid": tid })
    }

    fn micros(ns: u64) -> f64 {
        ns as f64 / 1000.0
    }
    ```
11. `review_markdown.rs`: 設計書 4.8 の形。純粋関数 2 つ。どの関数も 80 行以内にするため、下の小さな関数に分ける。

    - `pub fn to_markdown(document: &ReviewDocument) -> String`: 先頭の 4 行（`# DevTools review: <app.name>`、`- Session: .. / Updated: ..`、`- Build: debug|release / Kivact <kivact_version>`、`- Emulation: <emulation_summary>`）を書き、item ごとに `item_markdown` を足す。改行は `\n`。値が無い行は書かない。
    - `fn emulation_summary(emulation: &Emulation) -> String`: 設定された項目だけを `, ` でつなぐ（`theme dark`、`reduced motion on`、`locale <locale>`、`pseudo-locale accented`、`scale <scale>`、`touch on`。`pseudo_locale` が `Off` なら書かない）。何も無ければ `none`。
    - `fn item_markdown(item: &ReviewItem) -> String`: 見出し（Node は `## <id> <Role の Rust の variant 名> "<name>"  (window "<window_title>")`。role が無ければ `element`、name が無ければ `"<name>"` を書かない。Window は `## <id> window "<window_title>"`）。Node のときだけ、`- Source: <file>:<line>:<column>`（`source` がある場合）、`- Call chain: <app_code が true の frame を先に、残りを後に、`file:line` を ` ← ` でつなぐ>`（frame が無ければ書かない）、`- Query: <locator(item)>`（`None` なら書かない）、`- State: <states を ", " でつなぐ。無ければ none> / Bounds: x <x>, y <y>, <width>×<height>`。Node でも Window でも、`image` があれば `- Image: <image>`、`style_changes` があれば `- Style changes:` と各行 `  - <property>: <before または (unset)> → <after>  (<scope>)`（scope は `node #<id>`／`call site <file>:<line>`／`token <name>`）、最後に `- Comments:` と各行 `  <番号>. <text>  (<created>)`（番号は item の中で 1 から）。
    - `pub fn locator(item: &ReviewItem) -> Option<String>`（設計書 4.2.4 の規則）:

      ```rust
      pub fn locator(item: &ReviewItem) -> Option<String> {
          let ReviewTarget::Node { role, name, test_id, .. } = &item.target else {
              return None;
          };
          let query = match (test_id, role) {
              (Some(id), _) => format!("get_by_test_id({id:?})"),
              (None, Some(role)) => format!("get_by_role(Role::{role:?}, {:?})", name.as_deref().unwrap_or("")),
              (None, None) => return None,
          };
          let screen = if item.window == MAIN_WINDOW {
              "screen.".to_owned()
          } else {
              format!("screen.window({:?}).", item.window_title)
          };
          Some(format!("{screen}{query}"))
      }

      /// The id of the main window (`WindowId::MAIN` is 1).
      const MAIN_WINDOW: u64 = 1;
      ```

      （`{:?}` で文字列を書くと、Rust の文字列 literal としてエスケープされる。`Role` の `Debug` は Rust の variant 名と同じ。）
12. 試験 `crates/kivact-devtools-protocol/tests/protocol.rs`（integration test。`//! Role:` を付ける）:
    - `get_tree` の request の行を `parse_request` → `parse_call` で読み、`Call::GetTree` になる。
    - 知らない method は `METHOD_NOT_FOUND`、params の型違いは `INVALID_PARAMS`、JSON でない行は `PARSE_ERROR`。
    - `Empty` の method（`get_app_info`）は params を省いても `null` でも読める。
    - `Reply::GetAppInfo(..).into_value()` が object になり、`response` → `response_line` → `parse_message` で `Message::Response` に戻る。`notification_line` → `parse_message` で同じ `Notification` に戻る（`Notification::FocusRequested(Empty {})`。`Empty` は `crate::common` の型で、integration test では `kivact_devtools_protocol::Empty`（`lib.rs` の `pub use common::*`）を `use` する。`FocusRequested` という型は作らない。params は `Empty`）。
    - `METHOD_NAMES.len() == 60`、先頭が `"initialize"`、`<GetByTestId as Method>::NAME == "get_by_test_id"`。
    - `rfc3339_utc(0) == "1970-01-01T00:00:00Z"`、`rfc3339_utc(1_762_579_920_000) == "2025-11-08T05:32:00Z"`。
    - `discovery_dir_from` の 4 つの場合（`LOCALAPPDATA` あり／`XDG_RUNTIME_DIR` のみ／`HOME` のみ／何も無い→ `temp_dir` の下。空文字の `LOCALAPPDATA` は無いものとして扱う）。`discovery_file_name(12, 3456) == "12-3456.json"`。
    - `DevtoolsConfig::default().open_keys()` が `["F12", "Ctrl+Shift+I"]`、`keys.open = Some(vec![])` なら空で `pick_keys()` は既定のまま、`{}（すべての field が既定なので `DevtoolsConfig` として読める）と {"$schema": "x", "editor": []}` の JSON が読める。`with_defaults()` の `editor` が空でない。
    - `code::NODE_NOT_FOUND == -32010`、`RpcError::new(code::UNSUPPORTED, "x").code == -32015`。
    - 小さな `ReviewDocument`（Node の item と Window の item を 1 つずつ）を `to_markdown` にかけ、見出し（`## C1 Button "Save"`）と `- Comments:` の行が出る。`locator`（`lib.rs` の `pub use review_markdown::*;` で `kivact_devtools_protocol::locator` として出る。アプリは `p::locator` を呼ぶ）は main window の test id 付きで `screen.get_by_test_id("save")`、別 window の role 付きで `screen.window("Settings").get_by_role(Role::Button, "OK")`、Window の item で `None`。
    - `chrome_trace(&[], &[])` が `{"traceEvents":[]}`。

手順（client）:

1. `src/lib.rs`（`mod` と `pub use` だけ。`pub use <module>::*;`）、leaf module: `discovery.rs`、`connection.rs`、`error.rs`。
2. `error.rs`:

   ```rust
   //! Role: The error of a DevTools client call.

   use std::fmt;
   use kivact_devtools_protocol::RpcError;

   /// Why a call or a connection failed.
   #[derive(Clone, Debug, PartialEq)]
   pub enum ClientError {
       /// The socket failed, or a call took longer than 30 seconds (`Io("timed out")`).
       Io(String),
       /// The connection has ended.
       Closed,
       /// The agent answered with an error.
       Rpc(RpcError),
       /// The agent speaks something this client cannot read (including a different protocol version).
       Protocol(String),
   }

   impl fmt::Display for ClientError {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           match self {
               ClientError::Io(text) => write!(f, "I/O error: {text}"),
               ClientError::Closed => write!(f, "the connection was closed"),
               ClientError::Rpc(error) => write!(f, "{} ({})", error.message, error.code),
               ClientError::Protocol(text) => write!(f, "protocol error: {text}"),
           }
       }
   }

   impl std::error::Error for ClientError {}

   pub(crate) fn io_error(error: std::io::Error) -> ClientError {
       ClientError::Io(error.to_string())
   }
   ```
3. `discovery.rs`: 設計書 4.2.2。`pub fn discovery_dir() -> PathBuf`（`kivact_devtools_protocol::discovery_dir_from(|name| std::env::var_os(name), std::env::temp_dir())`）、`pub fn list_targets() -> Vec<(PathBuf, DiscoveryFile)>`（directory の `*.json` を読み、読めない file は飛ばす。各 file の `port` へ `TcpStream::connect_timeout(&SocketAddr, 100ms)` を試し、`ErrorKind::ConnectionRefused` で失敗した file は消して一覧に入れない。時間切れなど他の失敗は消さずに一覧から外す。`started_unix_ms` の昇順）、`pub fn read_target(path: &Path) -> std::io::Result<DiscoveryFile>`（`serde_json::from_str`。失敗は `io::Error::other`）、`pub fn target_from_env() -> Option<PathBuf>`（環境変数 `KIVACT_DEVTOOLS_TARGET`）。directory の計算は protocol の関数を呼ぶだけで、写しを書かない。
4. `connection.rs`（C8 の形。閉じた状態を明示的に持つ）:

   ```rust
   //! Role: One connection to a DevTools agent: typed calls and notifications.

   use std::collections::HashMap;
   use std::io::{BufRead, BufReader, Write};
   use std::net::{Shutdown, TcpStream};
   use std::sync::atomic::{AtomicU64, Ordering};
   use std::sync::mpsc;
   use std::sync::{Arc, Mutex, MutexGuard};
   use std::time::Duration;
   use kivact_devtools_protocol::{
       ClientInfo, DiscoveryFile, Initialize, InitializeParams, InitializeResult, Message, Method,
       Notification, RpcResponse, PROTOCOL_VERSION, parse_message, request_line,
   };
   use crate::error::{ClientError, io_error};

   const CALL_TIMEOUT: Duration = Duration::from_secs(30);

   type Answer = Result<serde_json::Value, ClientError>;

   /// The calls waiting for an answer, or why the connection ended.
   enum Calls {
       Open(HashMap<u64, mpsc::Sender<Answer>>),
       Closed(ClientError),
   }

   /// A connection to one agent. `Send + Sync`: the DevTools app shares it as `Arc<Connection>`.
   pub struct Connection {
       writer: Mutex<TcpStream>,
       calls: Arc<Mutex<Calls>>,
       next_id: AtomicU64,
       notifications: Mutex<mpsc::Receiver<Notification>>,
   }
   ```

   メソッドと関数（各 80 行以内）:

   - `pub fn open(file: &DiscoveryFile, client: ClientInfo) -> Result<(Connection, InitializeResult), ClientError>`: `u16::try_from(file.port)`（失敗は `ClientError::Protocol("port out of range")`）→ `TcpStream::connect(("127.0.0.1", port))` → `set_nodelay(true)`、`set_write_timeout(Some(CALL_TIMEOUT))` → `try_clone` を reader 用に → `Arc<Mutex<Calls>>`（`Calls::Open(HashMap::new())`）と `mpsc::channel::<Notification>()` を作り、`std::thread::Builder::new().name("kivact-devtools-client".into()).spawn(move || read_loop(reader, calls, sender))` → `Connection { .. next_id: AtomicU64::new(1) .. }` を作り、`connection.call::<Initialize>(InitializeParams { protocol: PROTOCOL_VERSION, token: file.token.clone(), client })`。結果の `protocol` が `PROTOCOL_VERSION` と違えば `ClientError::Protocol(format!("protocol version mismatch: target {}, client {}", result.protocol, PROTOCOL_VERSION))`。
   - `pub fn call<M: Method>(&self, params: M::Params) -> Result<M::Result, ClientError>`: `serde_json::to_value(params)`（失敗は `Protocol`）→ `call_raw(M::NAME, value)` → `serde_json::from_value`（失敗は `Protocol`）。
   - `pub fn call_raw(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, ClientError>`: `let id = self.next_id.fetch_add(1, Ordering::Relaxed);` → `mpsc::channel()` を作り、`lock(&self.calls)` の中で `Calls::Closed(error)` なら `Err(error.clone())` を返し（送らない）、`Calls::Open(map)` なら `map.insert(id, sender)` → 書く（`self.send(&request_line(id, method, params))`。失敗したら `map.remove(&id)` してその error を返す）→ `receiver.recv_timeout(CALL_TIMEOUT)`: `Ok(answer)` はそのまま返す、`Timeout` は `map.remove(&id)` して `ClientError::Io("timed out".into())`、`Disconnected` は `ClientError::Closed`。
   - `fn send(&self, line: &str) -> Result<(), ClientError>`: writer を lock し、`write_all(line.as_bytes())`、`write_all(b"\n")`、`flush()`（失敗は `io_error`）。
   - `pub fn next_notification(&self, timeout: Duration) -> Result<Option<Notification>, ClientError>`: `Ok(None)` は「`timeout` の間に何も来なかった」、`Err` は「接続が終わった」。`notifications` を lock して `recv_timeout(timeout)`: `Ok(n)` → `Ok(Some(n))`、`Timeout` → `Ok(None)`、`Disconnected` → `Err(reason)`（`reason` は `lock(&self.calls)` が `Calls::Closed(error)` ならその `error.clone()`、そうでなければ `ClientError::Closed`）。同時に待つのは 1 か所だけ（DevTools アプリの通知の loop）。（CONTRACT X11 の形に `timeout` を足したのは意図的: 通知の loop が 500ms ごとに戻る。）
   - `impl Drop for Connection`: writer を lock して `shutdown(Shutdown::Both)`（結果は捨てる。reader thread が終わる）。
   - `fn lock(calls: &Mutex<Calls>) -> MutexGuard<'_, Calls>`: `calls.lock().expect("invariant: the calls lock is never poisoned")`。
   - `fn read_loop(stream: TcpStream, calls: Arc<Mutex<Calls>>, notifications: mpsc::Sender<Notification>)`: `BufReader::new(stream)` で `read_line` を繰り返す。`Ok(0)` も `Err(_)` も `ClientError::Closed` を理由に loop を抜ける（接続の終わりは一つの値。Windows で相手の process が落ちると read は `ConnectionReset` の `Err` になるが、MCP（T2.3）・app・Python が選択を捨てる条件は `Closed` だけなので、ここでまとめる）。`Ok(_)` は `deliver(&line, &calls, &notifications)`。loop を抜けたら `close(&calls, reason)`。`notifications` の `Sender` は関数が返ると落ちる（これで `next_notification` が `Disconnected` を知る）。
   - `fn deliver(line: &str, calls: &Mutex<Calls>, notifications: &mpsc::Sender<Notification>)`: `parse_message(line.trim_end())`: `Ok(Message::Notification(n))` → `let _ = notifications.send(n);`、`Ok(Message::Response(r))` → `answer(calls, r)`、`Err(_)` は無視する。
   - `fn answer(calls: &Mutex<Calls>, response: RpcResponse)`: `response.id.as_u64()` が無ければ捨てる。`lock(calls)` の `Calls::Open(map)` から `map.remove(&id)` で `Sender` を取り（lock は取り出したらすぐ放す）、`(response.error, response.result)` が `(Some(error), _)` なら `Err(ClientError::Rpc(error))`、`(None, Some(value))` なら `Ok(value)`、`(None, None)` なら `Ok(Value::Null)` を送る。
   - `fn close(calls: &Mutex<Calls>, reason: ClientError)`: `std::mem::replace(&mut *lock(calls), Calls::Closed(reason.clone()))` で待っていた表を取り出し、待っている全 `Sender` に `Err(reason.clone())` を送る（以後の `call_raw` は送らずに同じ error を返す）。
5. 試験 `crates/kivact-devtools-client/tests/connection.rs`（`//! Role:` 付き）: `std::net::TcpListener` で偽の agent を別 thread に作る（`initialize` の request に `InitializeResult` を返し、続く request に決めた答えを返す。`response`・`response_line` を使う）。確かめること: (1) `Connection::open` が成功し `call::<GetAppInfo>(Empty {})` が答えを返す。(2) 偽の agent が stream を閉じると、`next_notification(Duration::from_secs(5))` が `Err(ClientError::Closed)` になり、その後の `call::<GetAppInfo>` は待たずにすぐ `Err` になる。(3) 偽の agent が `protocol: 99` を返すと `open` が `ClientError::Protocol` になる。(4) 偽の agent が通知の行（`notification_line`）を送ると `next_notification` が `Ok(Some(..))`、何も来ないときは `Ok(None)`。(5) `fn assert_send_sync() { fn f<T: Send + Sync>() {} f::<Connection>(); }`。
6. 確認: `cargo check -p kivact-devtools-protocol -p kivact-devtools-client`、`cargo test -p kivact-devtools-protocol -p kivact-devtools-client`、`cargo clippy -p kivact-devtools-protocol -p kivact-devtools-client --all-targets -- -D warnings`。（`tests/schema.rs` は T2.1 が足す。）

### T1.3 kit soft の状態マシン

担当ファイル: 付録の SM-SOFT-01〜10 が名指す file だけ（すべて `crates/kivact-ui-soft/` の中）と、新しく足す試験の file。

手順:

1. 付録の SM-SOFT-01 から 10 を順に、書いてあるとおりに適用する（置き換えのコード、読み書きの箇所、消す印）。
2. 付録に「試験なし」とある entry は、付録の試験の案に従って `crates/kivact-ui-soft/tests/consumer/` に試験を足す（試験の file を `tests/consumer.rs` に `#[path = ...] mod ...;` で登録する。既存の登録の形を写す）。試験は走らせない。
3. 確認: `cargo check -p kivact-ui-soft --all-targets`（`cargo xtask` は使わない。T1.1 が xtask を作り直している）。付録が「struct の前の印を書き換える／残す」と書いている箇所は、そのとおりにしてよい（T15.4 が field 単位の印に直す）。ただし置き換えのコードに field ごとの印が書いてある型（SM-SOFT-02 の `ItemBranch::Parent`）は、その印もそのまま写す。

### T1.4 kit creative の状態マシンと `window` の改名

担当ファイル: `crates/kivact-ui-creative/**`、`examples/kivact-ui-creative/**`、`docs/for-kivact-ui-user/en/creative/**`、`docs/for-kivact-ui-user/jp/creative/**`（directory 名が違えば `grep -rln "creative" docs/for-kivact-ui-user` で探した creative の文書）。

手順:

1. 付録の creative の entry のうち SM-INT-04、05、06、13 を適用する。SM-INT-14（outliner の paint）は kivact-interaction の変更（W8）を伴うので、ここでは行わない（T9.2 で行う）。SM-INT-13 の新しい試験の案は、付録が名指した既存の file `crates/kivact-ui-creative/tests/consumer/dock.rs` に、その file の `mount_dock` を使って足す（走らせない）。
2. 改名（標準の `window` と名前がぶつかるため。設計書 3.1）: `crates/kivact-ui-creative/src/feedback.rs` の `pub fn window` を `pub fn tool_window`、`pub fn window_button` を `pub fn tool_window_button`、`pub struct WindowOptions` を `pub struct ToolWindowOptions` に変える。`src/lib.rs` の再公開、kit の中の利用、`examples/kivact-ui-creative` と creative の文書（en・jp の両方）のすべての利用を新しい名前にする（`grep -rn "window(\|window_button\|WindowOptions" crates/kivact-ui-creative examples/kivact-ui-creative docs/for-kivact-ui-user` で探す。`ToolWindow` 型はそのまま）。
3. creative の README の部品の一覧も新しい名前にする。
4. 確認: `cargo check -p kivact-ui-creative -p kivact-ui-creative-catalog --all-targets`（`cargo xtask` は使わない）。`dev/xtask/policy/workspace.toml` の `WindowOptions` の行は T1.1 が直す。

### T1.5 Python の手書き部分

担当ファイル: `python/kivact-devtools/` の `README.md`、`src/kivact_devtools/__init__.py`、`py.typed`（空）、`_rpc.py`、`_discovery.py`、`client.py`、`errors.py`、`tests/test_client.py`、`examples/*.py`。`_generated.py` は T2.1 が作るので触らない。

参照: 設計書 4.7 節（4.7.3 の API 一覧が正本。名前・引数・返す型をそのとおりに書く）、4.2.1〜4.2.2 節。

手順:

1. `_generated.py` がまだ無いので、`client.py` が import する生成物の名前を、設計書 4.2.4 の型名と 4.7.2 の規則どおりに使って書く。使う名前: `Methods`、`PROTOCOL_VERSION`、`AppInfo`、`WindowInfo`、`ElementInfo`、`SourceLocation`、`Role`、`ClientInfo`、`ClientKind`、`ErrorCode`、`InitializeParams`、`DiscoveryFile`、`RoleQuery`・`TextQuery`・`TestIdQuery`、`GetTreeParams`・`GetNodeParams`、`HighlightParams`、`ScreenshotParams`、`WindowParams`、`InjectInputParams`、`InputStepClick`・`InputStepText`・`InputStepKey`（union `InputStep` の variant の class）、`OverrideScopeNode`・`OverrideScopeCallSite`、`SetStyleOverrideParams`、`StyleOverride`、`AddCommentParams`、`ReviewItem`、`ShowTableParams`、`RecordingSettings`、`ListTurnsParams`、`TurnGraph`、`FrameInfo`、`TreeSnapshot`、`NodeDetail`、`LayoutLint`、`AuditReport`。T2.1 の後でそのまま動く。**`Methods` は継承しない**: `App` が `self.rpc = Methods(connection.call)` を持つ（設計書 4.7.3）。
2. `errors.py`:

   ```python
   """Errors raised by the DevTools client."""
   from __future__ import annotations
   from ._generated import ErrorCode

   class DevtoolsError(Exception):
       """An error answer from the agent, or a closed connection (`ErrorCode.INTERNAL_ERROR`)."""
       def __init__(self, code: int, message: str) -> None:
           super().__init__(f"{code}: {message}")
           self.code = code
           self.message = message

   class NotFound(DevtoolsError): """`NODE_NOT_FOUND`, `WINDOW_NOT_FOUND` or `NO_MATCH`."""
   class MultipleFound(DevtoolsError): """-32013."""
   class Timeout(DevtoolsError): """`TIMED_OUT`, or no answer within the call timeout."""

   def error_from_rpc(code: int, message: str) -> DevtoolsError:
       if code in (ErrorCode.NODE_NOT_FOUND, ErrorCode.WINDOW_NOT_FOUND, ErrorCode.NO_MATCH):
           return NotFound(code, message)
       if code == ErrorCode.MULTIPLE_MATCHES:
           return MultipleFound(code, message)
       if code == ErrorCode.TIMED_OUT:
           return Timeout(code, message)
       return DevtoolsError(code, message)
   ```

3. `_discovery.py`: 設計書 4.2.2 の directory の規則を、環境を引数に取る純粋な関数にする: `discovery_dir(env: Mapping[str, str], temp_dir: Path) -> Path`（`LOCALAPPDATA` があれば `<値>/kivact/devtools`、無ければ `XDG_RUNTIME_DIR` があれば `<値>/kivact/devtools`、無ければ `HOME` があれば `<値>/.cache/kivact/devtools`、無ければ `temp_dir/kivact-devtools`。空の値は「無い」として扱う）。Rust 側（protocol の `discovery_dir_from`）の写しであり、試験（手順 7）が同じ 4 つの場合を確かめる。ほかに `read_target(path: Path) -> DiscoveryFile`（`DiscoveryFile.from_json(json.loads(path.read_text("utf-8")))`）、`list_targets() -> list[tuple[Path, DiscoveryFile]]`（directory の `*.json` を読み、各 file の `port` へ `socket.create_connection(("127.0.0.1", port), timeout=0.1)` で接続を試し、拒まれた file と読めない file は飛ばす。消さない）、`target_from_env() -> Path | None`（`KIVACT_DEVTOOLS_TARGET`）。
4. `_rpc.py`: `class RpcConnection`。接続の終わりを**一つの値**で持つ（待っている call の表か、閉じた理由のどちらか。閉じた後に呼んだ call が 30 秒待たされないため）。次の形をそのまま書く:

   ```python
   """JSON-RPC over a loopback socket: one request per line, one response per line."""
   from __future__ import annotations
   import json, queue, socket, threading
   from typing import Any
   from ._generated import ErrorCode
   from .errors import DevtoolsError, Timeout, error_from_rpc

   _CLOSED = int(ErrorCode.INTERNAL_ERROR)

   class RpcConnection:
       def __init__(self, port: int, *, timeout: float = 30.0) -> None:
           self._sock = socket.create_connection(("127.0.0.1", port), timeout=5.0)
           self._sock.settimeout(None)
           self._file = self._sock.makefile("rwb")
           self._timeout = timeout
           self._lock = threading.Lock()
           self._next_id = 1
           # The waiting calls, or why the connection closed. Never both.
           self._calls: dict[int, queue.Queue[dict[str, Any]]] | DevtoolsError = {}
           threading.Thread(target=self._read_loop, daemon=True).start()

       def call(self, method: str, params: dict[str, Any]) -> Any:
           replies: queue.Queue[dict[str, Any]] = queue.Queue(maxsize=1)
           with self._lock:
               if isinstance(self._calls, DevtoolsError):
                   raise self._calls
               request_id = self._next_id
               self._next_id += 1
               self._calls[request_id] = replies
               line = json.dumps({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}) + "\n"
               try:
                   self._file.write(line.encode("utf-8"))
                   self._file.flush()
               except OSError as error:
                   self._fail(DevtoolsError(_CLOSED, f"connection closed: {error}"))
           try:
               reply = replies.get(timeout=self._timeout)
           except queue.Empty:
               with self._lock:
                   if isinstance(self._calls, dict):
                       self._calls.pop(request_id, None)
               raise Timeout(int(ErrorCode.TIMED_OUT), f"no answer to {method} in {self._timeout:g} s") from None
           if "error" in reply:
               raise error_from_rpc(int(reply["error"]["code"]), str(reply["error"]["message"]))
           return reply.get("result")

       def close(self) -> None:
           with self._lock:
               self._fail(DevtoolsError(_CLOSED, "connection closed"))
           self._sock.close()

       def _fail(self, error: DevtoolsError) -> None:
           """Close: give every waiting call `error`. The caller holds `self._lock`."""
           if isinstance(self._calls, DevtoolsError):
               return
           waiting, self._calls = self._calls, error
           for replies in waiting.values():
               replies.put({"error": {"code": error.code, "message": error.message}})

       def _read_loop(self) -> None:
           reason = "connection closed"
           try:
               for raw in self._file:
                   message = json.loads(raw)
                   request_id = message.get("id")
                   if request_id is None:
                       continue  # a notification: this client does not subscribe
                   with self._lock:
                       replies = self._calls.pop(request_id, None) if isinstance(self._calls, dict) else None
                   if replies is not None:
                       replies.put(message)
           except (OSError, ValueError) as error:
               reason = f"connection closed: {error}"
           with self._lock:
               self._fail(DevtoolsError(_CLOSED, reason))
   ```

   `socket.create_connection` が失敗したら、`connect` が `DevtoolsError(ErrorCode.INTERNAL_ERROR, "cannot connect: …")` にして上げる（手順 5）。
5. `client.py`: 設計書 4.7.3 の API を、名前・引数・返す型・既定値をそのとおりに書く。`from __future__ import annotations`、すべての公開の関数と属性に型ヒント。

   - `connect(*, target=None, name=None) -> App`（keyword だけ）。`target` と `name` の両方があれば `ValueError`。対象の決め方: `target` → `name`（`list_targets()` から `app_name` が完全一致のもの）→ 環境変数 `KIVACT_DEVTOOLS_TARGET`（`name` が無いときだけ）→ `list_targets()` が 1 つならそれ。0 個は `NotFound(ErrorCode.NO_MATCH, …)`、2 個以上は `MultipleFound(ErrorCode.MULTIPLE_MATCHES, …)`（message に `app_name` と pid の一覧と「`target=` か `name=` を指定する」）。決まったら `read_target`、`RpcConnection(file.port)`、`Methods(connection.call).initialize(InitializeParams(protocol=PROTOCOL_VERSION, token=file.token, client=ClientInfo(name="kivact-devtools-python", version=_package_version(), kind=ClientKind.PYTHON)))`。`initialize` が失敗したら `connection.close()` してから上げる。**版の確認**: `RpcConnection` を開く前に `file.protocol != PROTOCOL_VERSION`、`initialize` の結果の `protocol` が `PROTOCOL_VERSION` と違うときも（接続していれば `connection.close()` してから）`DevtoolsError(ErrorCode.VERSION_MISMATCH, f"protocol version differs (target v{n}, client v{PROTOCOL_VERSION})")` を上げる。`_package_version()` は `importlib.metadata.version("kivact-devtools")`、`PackageNotFoundError` なら `"0.0.0"`（install せずに試験を走らせるため）。
   - `class App`: `__init__(self, connection: RpcConnection, info: AppInfo)`。`self.rpc = Methods(connection.call)`、`self.info = info`。`windows()`（`rpc.list_windows().windows` を `Window` に包む）、`window(title)`（title が完全一致するものを選ぶ。0 個は `NotFound(ErrorCode.WINDOW_NOT_FOUND, …)`、2 個以上は `MultipleFound(ErrorCode.MULTIPLE_MATCHES, …)`）、`record(*, updates=True, frames=True, input=False) -> Recording`、`clear_highlight()`（`HighlightParams(node=None, layout=False)`）、`export_review() -> list[Path]`（`rpc.export_review()` の結果の `[Path(r.json), Path(r.markdown)]`）、`show_table(title, columns, rows)`、`close()`、`__enter__`・`__exit__`。
   - `class _Queries`（`Window` と `Element` の共通の親。12 の query を**一度だけ**書く）: `_app: App` と `def _scope(self) -> tuple[int, int | None]`（window の id と within の node。子が実装する）。`get_by_role(self, role: Role, name: str | None = None) -> Element`: `window, within = self._scope()`、`found = self._app.rpc.get_by_role(RoleQuery(window=window, within=within, role=role, name=name, timeout_ms=None))`、`return Element(self._app, found.node)`。`query_by_role` は `rpc.query_by_role(...)` の `MaybeNode.node` が `None` なら `None`。`find_by_role(self, role, name=None, *, timeout: float = 5.0)` は `timeout_ms=int(timeout * 1000)` で `rpc.find_by_role`。`all_by_role` は `rpc.all_by_role(...).nodes` を `list[Element]` に。`*_by_text(text: str)` は `TextQuery(window=…, within=…, text=text, timeout_ms=…)`、`*_by_test_id(id: str)` は `TestIdQuery(window=…, within=…, id=id, timeout_ms=…)` で、形は同じ。
   - `class Window(_Queries)`: `info: WindowInfo`、`_scope()` は `(self.info.id, None)`。`tree()`（`GetTreeParams(window=self.info.id, root=None, depth=None)`）、`type_text(text)`（`InjectInputParams(window=…, steps=[InputStepText(text=text)])`）、`press_key(key)`（`InputStepKey(key=key)`。chord の解釈は agent が行う）、`screenshot() -> bytes`（`ScreenshotParams(window=…, node=None)` の `png_base64` を `base64.b64decode`）、`add_comment(text, *, screenshot=True) -> ReviewItem`（`AddCommentParams(window=…, node=None, text=text, screenshot=screenshot)`）、`lint_layout()`・`audit_a11y()`（`WindowParams(window=…)`）。
   - `class Element(_Queries)`: `info: ElementInfo`、`source: SourceLocation | None`（`__init__` で `info.source` を入れる）、`_scope()` は `(self.info.window, self.info.id)`。`detail()`（`GetNodeParams(node=self.info.id)`）、`highlight()`（`HighlightParams(node=self.info.id, layout=False)`）、`click()`（`InjectInputParams(window=self.info.window, steps=[InputStepClick(node=self.info.id)])`）、`set_style_override(property, value, *, scope: Literal["node", "call_site"] = "node") -> StyleOverride`（`"node"` は `OverrideScopeNode(node=self.info.id)`、`"call_site"` は `OverrideScopeCallSite(location=self.source)` で、`source` が `None` なら `ValueError`）、`add_comment(text, *, screenshot=True)`（`node=self.info.id`）。
   - `class Recording`: `__init__(self, app, settings: RecordingSettings)`、`turns: list[TurnGraph]`・`frames: list[FrameInfo]`（最初は空）。`__enter__`: `latest = app.rpc.list_turns(ListTurnsParams(since=None, limit=1, wait_ms=None)).turns` の最後の `turn` を `self._since`（無ければ `None`）に覚え、`app.rpc.start_recording(settings)`、`return self`。`__exit__`: `app.rpc.stop_recording()` の後に `self.turns = app.rpc.list_turns(ListTurnsParams(since=self._since, limit=None, wait_ms=None)).turns`、`self.frames = [f for t in self.turns for f in t.frames]`（`stop_recording` が待っている graph を先に完成させるので、`with` の中の操作の turn が入る）。`with` の中で例外が起きても `stop_recording` は呼ぶ。
6. `__init__.py`: `connect`、`App`、`Window`、`Element`、`Recording`、4 つの例外、利用者がよく使う生成物の型（`Role`、`ElementInfo`、`WindowInfo`、`AppInfo`、`NodeDetail`、`TurnGraph`、`FrameInfo`、`SourceLocation`、`ReviewItem`、`StyleOverride`、`TreeSnapshot`、`LayoutLint`、`AuditReport`）を 1 つずつ import して並べ、`__all__` に書く（`import *` は使わない）。
7. `tests/test_client.py`: `unittest`。file の先頭で `sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))` をして、install しなくても `kivact_devtools` を import できるようにする。偽の agent は `socketserver.ThreadingTCPServer`（`allow_reuse_address = True`、`daemon_threads = True`）で、1 行ずつ読んで JSON-RPC として答える。**各 request の `params` を、その method の生成した params class の `from_json` で読み（method 名 → class の表を試験に書く。`Empty` の method は読まない）、失敗（`KeyError`・`TypeError`）したら `-32602` を返して `server.failures` に method 名を足す**。どの試験の最後でも `self.assertEqual(server.failures, [])`（生成した型と手書きの層の食い違いを、実行時でなくここで見つけるため）。答えは method ごとの固定の JSON（必須の field は生成した dataclass の「既定の無い field」に合わせる）: `initialize`→`{"protocol":1,"app":{…}}`、`list_windows`→1 つの window（id 1、title `"Main"`）、`get_by_role`・`query_by_role`・`find_by_role` と text／test_id の同じ形→`{"node": <ElementInfo の JSON: id 2、window 1、kind "box"、role "button"、name "Increment"、hidden false、leaving false、update_count 0>}`、`all_*`→`{"nodes":[…]}`、`get_tree`・`lint_layout`・`audit_a11y`→`{"window":1,…}`、`get_node`→必須 field だけの `NodeDetail`、`screenshot`→1×1 の `png_base64`、`set_style_override`→`StyleOverride`、`add_comment`→`ReviewItem`（`target` は `{"kind":"window"}`）、`list_turns`→`{"turns":[{…turn 3…}]}`、`export_review`→`{"json":"x/review.json","markdown":"x/review.md"}`、`highlight`・`set_overlays`・`start_recording`・`stop_recording`・`inject_input`・`show_table`→`{}`。試験:
   - `connect(target=<discovery file>)` ができ、`app.info.name` が答えのとおり（discovery file は `tempfile` に作る。`port` は偽の agent の port）。
   - `win.get_by_role(Role.BUTTON, "Increment")` が `Element` を返し、`name` を省いた呼び出しの params に `name` の key が無い。
   - 偽の agent が `ErrorCode.NO_MATCH` を返すと `NotFound`、`MULTIPLE_MATCHES` で `MultipleFound`、`TIMED_OUT` で `Timeout`。`initialize` の答えの `protocol` が違うと `connect` が `ErrorCode.VERSION_MISMATCH` の `DevtoolsError`。
   - 接続が閉じる: 偽の agent が接続を閉じた後の `call` が、`RpcConnection(port, timeout=0.5)` で 0.5 秒より前に `DevtoolsError`（code `ErrorCode.INTERNAL_ERROR`）になる（30 秒待たされない）。
   - `with app.record() as rec: button.click()`: 偽の agent が受けた method の順が `list_turns`、`start_recording`、`inject_input`、`stop_recording`、`list_turns` で、最後の `list_turns` の params に `since` = 最初の `list_turns` の答えの turn 番号が入っており、`rec.turns` に turn 3 がある。
   - 一覧: `App`・`Window`・`Element` の公開 method（`windows`、`window`、`record`、`clear_highlight`、`export_review`、`show_table`、12 の query、`tree`、`type_text`、`press_key`、`screenshot`、`add_comment`、`lint_layout`、`audit_a11y`、`detail`、`highlight`、`click`、`set_style_override`、`add_comment`）をそれぞれ 1 回ずつ呼び、例外が出ず `server.failures` が空であること。
   - `discovery_dir` の 4 つの場合（`LOCALAPPDATA` あり、`XDG_RUNTIME_DIR` のみ、`HOME` のみ、どれも無い）。
   この試験は `_generated.py` ができてから（T2.1 の後）走る。
8. `examples/list_buttons.py`（全 window の button を `show_table` で出す。`rows = [[w.info.title, e.info.name or ""] for w in app.windows() for e in w.all_by_role(kd.Role.BUTTON)]`）、`examples/record_click.py`（最初の window の最初の button を `with app.record() as rec:` の中で click し、turn の数と `duration_ns` の合計を print する。button が無ければ `SystemExit("no button")`）。
9. `README.md`: 導入（`pip install "kivact-devtools @ git+https://github.com/Kashiwade-music/kivact#subdirectory=python/kivact-devtools"`。Kivact の repo を編集する人だけ `pip install -e python/kivact-devtools`）、VS Code の設定（interpreter の選び方だけ）、例。
10. 確認: `python -m compileall python/kivact-devtools/src`（構文だけ）。
### T1.6 複数ウィンドウの文書（code を含まない部分）

担当ファイル: `docs/for-kivact-user/en/how-to-guide/windows.md`、`docs/for-kivact-user/jp/how-to-guide/windows.md`（新規）、`docs/for-kivact-user/en/how-to-guide/README.md` と `jp/how-to-guide/README.md` の一覧（無ければ guide の一覧の file）、`docs/for-kivact-user/{en,jp}/how-to-guide/coming-from-react-or-solid.md`、`docs/for-kivact-user/{en,jp}/how-to-guide/testing.md`、`docs/for-kivact-developer/decisions.md`、`docs/for-kivact-developer/strategy.md`。

手順:

1. `windows.md`（en と jp を同じ構成で）: 既存の how-to guide（`testing.md` など）の書き方を写す。節:
   - 「Open a window」（設計書 3.1 の例そのまま。`.mode(WindowMode::Tool)`、`use kivact::prelude::*;` だけで足りる）。
   - 「Close it from the window and from the title bar」（`on_dismiss`。`()` を返すと閉じる、`Handled::PreventDefault`・`Handled::Yes` と失敗では開いたまま、という設計書 3.1 の規則 5）。
   - 「Window modes and placement」。
   - 「Contexts follow the declaration, like a React portal」。
   - 「Bring a window to the front」（`platform::focus_window()`。その window の中から呼ぶこと、最善の努力であること、すでに開いている window の `open` に true を書き直しても何も起きないこと。設計書 3.1 の規則 4 と 3.1.1）。
   - 「Ask about the current window」（設計書 3.1 の規則 8。間違った置き場所と正しい置き場所を 2 つの短い例で示す: main window で作った `callback` を Settings window で押すと、その中の `pick_file` の親は main window になる。window の中の操作で使う service は、その window の content の中で作った handler・callback から呼ぶ）。
   - 「Test windows」（`screen.window("Settings")`、`windows()`、`is_open()`、`request_close()` の例。window の handle は先に変数に入れる: `let settings = screen.window("Settings"); settings.get_by_role(..).click();`。`screen.window(..)` の戻り値から取った `Element` を `let` に入れると E0716 になる、という 1 文を書く。同じ title の window が 2 つあると失敗すること。`RenderOptions::single_window()` で window を開けない環境を試せること）。
   - 「Where windows are not available」（embed と mobile、`Capabilities.multi_window`。embed では `focus_window` も `Unsupported`）。
   Rust の code block は doctest としてコンパイルされる。この段階では API がまだ無いので、code block の言語を `rust` にしたまま書いてよい（C1 の前にコンパイルされる）。
2. `testing.md`（en と jp）: `screen.window(title)` と `screen.windows()` の節を足す（handle は先に `let` に入れる。手順 1 の「Test windows」と同じ内容を短く。詳しくは windows.md へのリンク）。
3. `coming-from-react-or-solid.md`（en と jp）:
   - 対応表（67-101 行付近）に Portal の行を足す: `| Portal | \`createPortal(child, el)\`; \`window.open\` + a portal for a new window | \`<Portal>\` | \`overlay(..)\` in the same window; \`window(title, open)\` for an OS window |`（jp は表の他の行の言語に合わせる）。
   - 「7. A local name can hide a prelude function」の節（511-521 行付近）が挙げる prelude の名前の一覧に `window` を足す（`let window = …` はよくある名前なので、`settings_window` のように呼ぶ、と書く）。
4. `decisions.md`: 「実装の決定」の表に設計書 6 章の decisions の行を足す（日付 2026-11-08、ID は既存の最後の番号の続き）。
5. `strategy.md`: 5 章の 2 行をその場で書き換える（行は増やさない。したがって 200 行の制限は変わらない）:
   - 「window の起動設定と OS サービス」の行（88 行付近）: `サイズ・最小サイズ・clipboard・ファイル選択…` を `サイズ・最小サイズ・複数の window（位置は親に対する相対の意図だけ）・clipboard・ファイル選択…` にする。
   - 「試験と診断」の行（90 行付近）: 末尾の `…検査を持つ` の後（行末の ` |` の前）に `。DevTools（別プロセスの agent と MCP・Python の client）で更新理由・性能・意味情報を調べる` を足す。
6. 確認: 無し（文書だけ）。
## 4. W2

| task | 担当 | crate・場所 |
|---|---|---|
| T2.1 | 1 | `dev/xtask`（生成器）、生成物、protocol crate の `src/schema.rs`・`src/lib.rs` の 1 行・`tests/schema.rs` |
| T2.2 | 2 | `crates/kivact-devtools-agent` |
| T2.4 | 3 | `crates/kivact-types`（と `crates/arvane-compose/src/plan/hash.rs` の 1 行）と、その変更で壊れる箇所 |
| T2.5 | 4 | `docs/for-kivact-user/{en,jp}/how-to-guide/devtools.md` |

T2.3（MCP）は T2.1 が生成する schema の file を使うので W3 で行う。

波の検査: `cargo check --workspace --all-targets`（全体。T2.4 の変更を確かめる。T2.4 自身は書きかけの `xtask` と `kivact-devtools-agent` を除いて確かめるので、全体はここで初めて確かめる）、`cargo test -p xtask -p kivact-devtools-protocol -p kivact-devtools-agent`、`cargo clippy -p kivact-devtools-protocol -p kivact-devtools-agent -p kivact-devtools-client --all-targets -- -D warnings`、`cargo xtask devtools-codegen --check`、`cargo xtask check --no-write --only devtools-codegen`、`python -m unittest discover python/kivact-devtools/tests`。commit: `feat: add window types, the devtools agent and protocol codegen`。
### T2.1 生成器（`cargo xtask devtools-codegen`）

担当ファイル: `dev/xtask/src/devtools_codegen.rs` と `dev/xtask/src/devtools_codegen/*.rs`（新規）、`dev/xtask/src/main.rs`（subcommand の追加）、`dev/xtask/src/check_cli.rs`（`RULES` に `"devtools-codegen"`）、`dev/xtask/src/local_checks.rs`（`check` の loop と `run_rule` の arm）、生成物 3 つ（設計書 4.2.8）、protocol crate の `src/schema.rs`（新規）・`src/lib.rs`（2 行）・`tests/schema.rs`（新規）。

参照: 設計書 4.2.3、4.2.6、4.2.7、4.2.8、4.7.2。

手順:

1. module の構成: `devtools_codegen.rs`（`mod` と `pub(crate) use` だけ。`#[cfg(test)] mod tests;` も書く）。leaf: `model.rs`（読んだ型の中間表現）、`read.rs`（syn で source を読む）、`json.rs`（JSON を key の順のまま書く小さな writer）、`schema.rs`（JSON Schema）、`python.rs`（Python）、`check.rs`（生成して file と比べる）、`cli.rs`（subcommand）、`tests.rs`（試験。先頭に `//! Role:`）。`devtools_codegen.rs` の `pub(crate) use` は `check::{check, generate}` と `cli::run`。xtask には `serde_json` が無い（足さない）ので、JSON は `json.rs` で自分で書く。
2. `model.rs` の中間表現:

   ```rust
   pub(super) enum Ty { Bool, U32, U64, I64, F64, Str, Option(Box<Ty>), Vec(Box<Ty>), Named(String) }
   pub(super) struct Field { pub name: String, pub ty: Ty, pub doc: String }
   pub(super) enum Item {
       Record { name: String, doc: String, fields: Vec<Field> },
       Choice { name: String, doc: String, variants: Vec<(String, String)> },        // (Rust name, doc)
       Union { name: String, doc: String, variants: Vec<(String, String, Vec<Field>)> }, // (Rust name, doc, fields)
       Alias { name: String, doc: String, ty: Ty },
   }
   pub(super) struct MethodDecl { pub variant: String, pub wire: String, pub params: String, pub result: String, pub doc: String }
   pub(super) struct NotificationDecl { pub variant: String, pub wire: String, pub params: String, pub topic: String, pub doc: String }
   pub(super) struct ErrorCode { pub name: String, pub code: i64, pub doc: String }
   #[derive(Default)]
   pub(super) struct Protocol {
       pub version: u32, pub items: Vec<Item>, pub methods: Vec<MethodDecl>,
       pub notifications: Vec<NotificationDecl>, pub errors: Vec<ErrorCode>,
       /// `DevtoolsConfig` has the `#[serde(rename = "$schema")]` field (only the config schema lists it).
       pub config_schema_key: bool,
   }
   ```

   `Ty`・`Item` を読み書きする関数に、`fn snake(name: &str) -> String`（serde の `rename_all = "snake_case"` と同じ: 大文字の前に `_` を入れて小文字にする。先頭は除く。`TestId` → `test_id`、`FocusVisible` → `focus_visible`）と `fn upper_snake(name: &str) -> String`（`snake` を大文字にしたもの。Python の enum の member 名）も置く。
3. `read.rs`: `pub(super) fn read_protocol(root: &Path) -> Result<Protocol, String>` と、試験が使う `pub(super) fn read_source(file: &str, text: &str, out: &mut Protocol) -> Result<(), String>`。
   - `crates/kivact-devtools-protocol/src/` の `.rs` をファイル名の昇順にすべて読む。ただし wire の型ではない次の file は読まない: `lib.rs`、`rpc.rs`、`time.rs`、`review_markdown.rs`、`discovery.rs`、`trace.rs`、`schema.rs`（schema.rs は生成物を `include_str!` するだけで、生成の入力ではない）。`syn::parse_file` で読み、`trait`・inline の `mod`・`fn`・`impl`・`macro_rules!`・`use`・（下の 2 つ以外の）`const`・`static` は無視する。
   - `Item::Struct`（名前付き field のみ）→ `Record`。`Item::Enum` で `#[serde(tag = "kind")]` が付いていれば `Union`、付いていなければ全 variant が unit であることを確かめて `Choice`。`Item::Type` → `Alias`。`Item::Macro` で path の最後が `protocol_methods`／`protocol_notifications` なら中身を読む（下）。`DevtoolsConfig` の `schema` の field（serde の `rename = "$schema"`）は `Record` の field としては出さず、`Protocol` に `config_schema_key: bool` を立てて、`devtools-config.schema.json` の `"$schema": {"type": "string"}` の property としてだけ出す（Python と主 schema には出さない）。
   - `app.rs` の `pub const PROTOCOL_VERSION: u32 = N;` は読み、`version` に入れる。`code.rs` の `pub const NAME: i32 = N;` はすべて読み、`errors` に入れる（doc は `///`）。整数の値は次の関数で読む:

     ```rust
     fn const_int(expr: &syn::Expr) -> Option<i64> {
         match expr {
             syn::Expr::Lit(lit) => match &lit.lit {
                 syn::Lit::Int(value) => value.base10_parse().ok(),
                 _ => None,
             },
             syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
                 const_int(&unary.expr).map(|value| -value)
             }
             _ => None,
         }
     }
     ```
   - 型の対応は次の関数で作る: `syn::Type::Path` の最後の segment が `bool`／`u32`／`u64`／`i64`／`f64`／`String` なら対応する `Ty`、`Option<T>`／`Vec<T>` なら中身を再帰して包む、`Box`・`HashMap`・`BTreeMap`・`HashSet`・`Rc`・`Arc` と tuple・参照・配列・その他の型は error、それ以外の名前は `Ty::Named`。error の文は `"{file}: {type or field}: unsupported type in a protocol type (design 4.2.3)"`。
   - serde の attribute（4.2.3 の 9）は次の関数で調べ、期待と違えば `"{file}: {型}[.{field}]: unexpected serde attribute {entries:?}"` の error にする（attribute が無いのに必要な場合も error）:

     ```rust
     /// The `serde(..)` entries of `attrs` as `name` or `name = "value"`, sorted.
     fn serde_entries(attrs: &[syn::Attribute]) -> Result<Vec<String>, String> {
         let mut entries = Vec::new();
         for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
             attr.parse_nested_meta(|meta| {
                 let name = meta
                     .path
                     .get_ident()
                     .map(ToString::to_string)
                     .ok_or_else(|| meta.error("unsupported serde path"))?;
                 if meta.input.peek(syn::Token![=]) {
                     let value: syn::LitStr = meta.value()?.parse()?;
                     entries.push(format!("{name} = \"{}\"", value.value()));
                 } else {
                     entries.push(name);
                 }
                 Ok(())
             })
             .map_err(|error| error.to_string())?;
         }
         entries.sort();
         Ok(entries)
     }
     ```

     期待する値: unit enum は `["rename_all = \"snake_case\""]`、tag 付き enum は `["rename_all = \"snake_case\"", "tag = \"kind\""]`、`Option<..>` の field は `["default", "skip_serializing_if = \"Option::is_none\""]`、`Vec<..>` の field は `["default", "skip_serializing_if = \"Vec::is_empty\""]`、それ以外の field と struct と variant は `[]`、`DevtoolsConfig.schema` は `["default", "rename = \"$schema\"", "skip_serializing_if = \"Option::is_none\""]`。
   - doc は `#[doc = "..."]` の文字列を改行でつなぎ、各行の先頭の空白 1 つを除く。
   - macro の中身は `syn::parse::Parser::parse2` で読む:

     ```rust
     fn parse_methods(tokens: proc_macro2::TokenStream) -> syn::Result<Vec<MethodDecl>> {
         let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<MethodDecl>> {
             let mut out = Vec::new();
             while !input.is_empty() {
                 let attrs = input.call(syn::Attribute::parse_outer)?;
                 let variant: syn::Ident = input.parse()?;
                 input.parse::<syn::Token![=>]>()?;
                 let wire: syn::LitStr = input.parse()?;
                 let params;
                 syn::parenthesized!(params in input);
                 let params: syn::Type = params.parse()?;
                 input.parse::<syn::Token![->]>()?;
                 let result: syn::Type = input.parse()?;
                 input.parse::<syn::Token![;]>()?;
                 out.push(MethodDecl {
                     variant: variant.to_string(),
                     wire: wire.value(),
                     params: type_name(&params)?,
                     result: type_name(&result)?,
                     doc: doc_text(&attrs),
                 });
             }
             Ok(out)
         };
         syn::parse::Parser::parse2(parser, tokens)
     }
     ```

     `parse_notifications` も同じ形で、`(params)` の後に `input.parse::<syn::Token![in]>()?;` と `let topic: syn::Ident = input.parse()?;` を読む。`type_name` は型の最後の segment の名前（単純な名前でなければ error）。
4. `json.rs`: key の順を保つ JSON の writer。

   ```rust
   pub(super) enum Json {
       Str(String),
       Int(i64),
       Bool(bool),
       Array(Vec<Json>),
       Object(Vec<(String, Json)>),
   }
   ```

   `impl Json { pub(super) fn render(&self) -> String }`（2 空白の indent、最後に改行。空の配列・object は `[]`・`{}`。文字列は `"`・`\`・制御文字（`\n` `\r` `\t` と、それ以外の U+0020 未満は `\u00XX`）をエスケープする。key の順は `Vec` の順のまま）。補助に `fn obj(fields: Vec<(&str, Json)>) -> Json`、`fn s(text: &str) -> Json`。
5. `schema.rs`: 設計書 4.2.8 の型の対応で `kivact-devtools.schema.json` を書く。形:

   ```json
   {
     "$schema": "https://json-schema.org/draft/2020-12/schema",
     "$id": "kivact-devtools.schema.json",
     "title": "Kivact DevTools protocol",
     "x-protocol": 1,
     "x-errors": [ { "name": "PARSE_ERROR", "code": -32700, "description": "..." } ],
     "$defs": { "<Type>": { "description": "...", ... } },
     "x-methods": [ { "name": "get_tree", "params": "GetTreeParams", "result": "TreeSnapshot", "description": "..." } ],
     "x-notifications": [ { "name": "tree_changed", "params": "TreeChanged", "topic": "tree", "description": "..." } ]
   }
   ```

   `x-protocol` は `Protocol.version`（literal の 1 を書かない）。record: `{"description":..,"type":"object","properties":{...},"required":[Option と Vec 以外の field],"additionalProperties":false}`。choice: `{"description":..,"type":"string","enum":[...snake_case...]}`。union: `{"description":..,"oneOf":[{"type":"object","properties":{"kind":{"const":"<snake>"}, ...},"required":["kind", ...],"additionalProperties":false}, ...]}`。alias: 対応する型（`u64` は `{"type":"integer","minimum":0}`）に description。名前の参照: `{"$ref":"#/$defs/<Name>"}`。`Option<T>` は T の schema（required に入れない）、`Vec<T>` は `{"type":"array","items":T}`。field の schema には field の doc を `description` として添える。`topic` は `snake(topic)`。
   `devtools-config.schema.json`: `DevtoolsConfig` を root にした schema（`$schema`・`title`・`type: object`・`properties`（`"$schema": {"type":"string"}` を含む）・`additionalProperties: false`・`$defs` に `KeyConfig`）。主 schema の `$defs` の `DevtoolsConfig` には `"$schema"` の property を入れない。
6. `python.rs`: 設計書 4.7.2 の規則で `_generated.py` を書く。先頭:

   ```python
   # Generated by `cargo xtask devtools-codegen`. Do not edit.
   """Types and methods of the Kivact DevTools protocol."""
   from __future__ import annotations
   from collections.abc import Callable
   from dataclasses import dataclass, field
   from enum import Enum, IntEnum
   from typing import Any, Literal
   PROTOCOL_VERSION = 1
   ```

   （`PROTOCOL_VERSION` の値は `Protocol.version`。）続けて、`Item` を入力の順に出す:
   - record: `@dataclass(frozen=True, slots=True)` の class。docstring、field（`name: 型` の後に既定。`Option(T)` → `T | None = None`、`Vec(T)` → `list[T] = field(default_factory=list)`。既定の無い field → 既定のある field の順に並べ替える）、`@classmethod def from_json(cls, data: dict[str, Any]) -> <Name>:` と `def to_json(self) -> dict[str, Any]:`。field が無い record は `from_json` が `return cls()`、`to_json` が `return {}`。
   - `from_json` の値の読み方: 必須の scalar は `data["f"]`。`Named` が record なら `X.from_json(data["f"])`、choice なら `X(data["f"])`、union なら `<union の snake>_from_json(data["f"])`、alias なら alias 先の型と同じ。`Option(T)` は `<T の読み方> if data.get("f") is not None else None`。`Vec(T)` は `[<T の読み方 (v)> for v in data.get("f", [])]`。
   - `to_json` は `data: dict[str, Any] = {必須の field}` を作り、`Option` の field は `if self.f is not None: data["f"] = ...`、`Vec` の field は `if self.f: data["f"] = [...]`（serde の `skip_serializing_if` と同じ。`None` と空 list は出さない）。値の書き方: record・union は `.to_json()`、choice は `.value`、scalar はそのまま。JSON の key は Rust の field 名のまま。
   - Python の予約語（`False None True and as assert async await break class continue def del elif else except finally for from global if import in is lambda nonlocal not or pass raise return try while with yield`）と同じ field 名は、属性名の末尾に `_` を付け、JSON の key は元の名前にする。
   - choice: `class X(str, Enum):` に `UPPER_SNAKE = "snake"` の member（`Role.BUTTON = "button"`）。
   - union: variant ごとに record と同じ形の class（class 名は `<Union><Variant>`、例 `OverrideScopeNode`。最後の field に `kind: Literal["node"] = "node"`。`to_json` は先頭に `"kind": "node"` を入れる）、`X = A | B | C`、`def x_from_json(data: dict[str, Any]) -> X:`（`data["kind"]` で振り分け、知らない値は `raise ValueError(f"unknown X kind: {kind}")`）。
   - alias: `NodeId = int`。
   - `class ErrorCode(IntEnum):` に `Protocol.errors` の `NAME = code`（docstring 付き）。
   - method: `class Methods:`（docstring「Every protocol method under its wire name.」）に次の形:

     ```python
     class Methods:
         """Every protocol method under its wire name."""

         def __init__(self, call: Callable[[str, dict[str, Any]], Any]) -> None:
             self._call = call

         def get_tree(self, params: GetTreeParams) -> TreeSnapshot:
             """The element tree of one window."""
             return TreeSnapshot.from_json(self._call("get_tree", params.to_json()))

         def get_app_info(self) -> AppInfo:
             """The app: name, process, build and workspace root."""
             return AppInfo.from_json(self._call("get_app_info", {}))

         def stop_recording(self) -> None:
             """Finish the turns that are waiting for their frame, then stop recording."""
             self._call("stop_recording", {})
     ```

     params が `Empty` の method は引数を取らず `{}` を送る。result が `Empty` の method は `-> None`。method の関数名は wire 名（`get_tree`）。docstring は method の doc。
   - docstring は Rust の `///` から。`\` は `\\`、`"""` は `\"\"\"` にエスケープする。
7. `check.rs`: `pub(crate) fn generate(root: &Path) -> Result<Vec<(PathBuf, String)>, String>`（3 つの path（`root` からの相対を `root.join` した絶対 path）と中身。`read_protocol` → `schema`・`python`）と、`pub(crate) fn check(root: &Path) -> Vec<String>`（生成に失敗したらその error 1 件。file が無い・中身が違うものを `"{path}: out of date; run `cargo xtask devtools-codegen`"` の文字列に。比べる前に、file の中身の `\r\n` を `\n` にする）。
8. `cli.rs`: `pub(crate) fn run(root: &Path, arguments: &[String]) -> std::process::ExitCode`。`--check` があれば `check` の結果を stderr に 1 行ずつ書いて、error があれば `ExitCode::FAILURE`、無ければ `SUCCESS`。無ければ `generate` して親 directory を作って書き出す（書いた path を stdout に 1 行ずつ）。書き込みは `std::io::stdout().lock()` に `writeln!`（`println!` は使わない。結果は `let _ =` で捨てない: 失敗したら `ExitCode::FAILURE`）。他の引数は `unknown argument: ...` で `ExitCode::from(2)`。
9. `main.rs`: `mod devtools_codegen;` を足し、`main` の `let arguments = ...;` の次（`--help` の判定の前）に追加:

   ```rust
   if arguments.first().map(String::as_str) == Some("devtools-codegen") {
       return devtools_codegen::run(root, &arguments[1..]);
   }
   ```

   `check_cli.rs`: `RULES` を `[&str; 9]` にして末尾に `"devtools-codegen"` を足す。`local_checks.rs`: `check` の `for rule in RULES {` の中、`if !options.selected(rule) { continue; }` の直後に、`devtools-codegen` が source の loop（module ごと）に入らないよう次を足す:

   ```rust
   if rule == "devtools-codegen" {
       if options.package.as_deref().is_none_or(|package| package == "kivact-devtools-protocol") {
           errors.extend(crate::devtools_codegen::check(root));
       }
       continue;
   }
   ```

   `run_rule` の `match` に `"devtools-codegen" => {}` の arm を足す（`unreachable!` に落ちないため。実際の検査は上の 1 回）。
10. protocol crate: 生成の後（11.）に `crates/kivact-devtools-protocol/src/schema.rs` を作り（先頭に `//! Role: The generated JSON Schema of the protocol, embedded for the MCP server and the tests.`）、`pub const SCHEMA_JSON: &str = include_str!("../schema/kivact-devtools.schema.json");`（英語の `///` 付き）を書き、`lib.rs` に `mod schema;` と `pub use schema::*;` を足す（MCP は `kivact_devtools_protocol::SCHEMA_JSON` を使う。crate をまたぐ `include_str!` の path を書かない）。`crates/kivact-devtools-protocol/tests/schema.rs`（`//! Role:` 付き）:

    ```rust
    //! Role: The generated schema lists the same methods, version and error codes as the Rust protocol.

    use kivact_devtools_protocol::{METHOD_NAMES, PROTOCOL_VERSION, SCHEMA_JSON, code};

    fn schema() -> serde_json::Value {
        serde_json::from_str(SCHEMA_JSON).expect("invariant: the generated schema is JSON")
    }

    #[test]
    fn methods_match_in_order() {
        let schema = schema();
        let names: Vec<&str> = schema["x-methods"]
            .as_array()
            .expect("invariant: x-methods is an array")
            .iter()
            .map(|method| method["name"].as_str().expect("invariant: a method has a name"))
            .collect();
        assert_eq!(names, METHOD_NAMES);
    }

    #[test]
    fn version_and_error_codes_match() {
        let schema = schema();
        assert_eq!(schema["x-protocol"], PROTOCOL_VERSION);
        let errors = schema["x-errors"].as_array().expect("invariant: x-errors is an array");
        let node_not_found = errors.iter().find(|error| error["name"] == "NODE_NOT_FOUND");
        assert_eq!(node_not_found.map(|error| error["code"].clone()), Some(code::NODE_NOT_FOUND.into()));
    }
    ```
11. 試験 `devtools_codegen/tests.rs`: `read_source` に小さな Rust の source（record・Option と Vec の field を持つ record・choice・union・alias・`protocol_methods!`・`protocol_notifications!` の呼び出しを 1 つずつ）を文字列で与え、`schema` と `python` の出力に期待する断片が含まれることを確かめる（例: `"$ref": "#/$defs/Point"`、`class Point:`、`label: str | None = None`、`class Role(str, Enum):`、`BUTTON = "button"`、`class OverrideScopeNode:`、`def get_tree(self, params: GetTreeParams) -> TreeSnapshot:`、`def get_app_info(self) -> AppInfo:`）。拒む場合も確かめる: `#[serde(rename = "x")]` の field、`HashMap<String, u32>` の field、`rename_all` の無い unit enum は、それぞれ型の名前を含む error になる。`snake("TestId") == "test_id"`、`upper_snake("CheckBox") == "CHECK_BOX"`。
12. 生成: `cargo xtask devtools-codegen` を実行し、3 つの file を作る（その後 10. の `schema.rs` を足す）。
13. 確認: `cargo test -p xtask`、`cargo test -p kivact-devtools-protocol`、`cargo xtask devtools-codegen --check`、`cargo xtask check --no-write --only devtools-codegen`、`python -m unittest discover python/kivact-devtools/tests`（T1.5 の試験がここで初めて走る。失敗したら原因が生成物にあるか client にあるかを報告する。client 側の修正は T1.5 の担当なので指揮役が判断する）。

### T2.2 agent

担当ファイル: `crates/kivact-devtools-agent/**`。

参照: 設計書 4.3 節、4.2.1〜4.2.2、4.2.7、4.2.4 の review。

共通の書き方は T1.2 の冒頭と同じ（`Mutex` は `lock().expect("invariant: …")`、`use` は 1 module から 1 文）。`Cargo.toml`（dev-dependency の client も含む）は T0.1 が作ってある。

手順:

1. module: `lib.rs`（`mod` と `pub use`。公開する名前: `Agent`・`AgentConfig`・`Incoming`・`Responder`・`ReviewStore`・`NewComment`・`encode_png`・`base64`）、leaf: `agent.rs`（`Agent`、`AgentConfig`）、`incoming.rs`（`Incoming`、`Responder`）、`session.rs`（`Session`、`Peers`、`Peer`、`ConnectionId`、`Writer`、`write_line`）、`server.rs`（`Shared` と listener thread）、`connection.rs`（1 つの connection の reader thread）、`discovery.rs`（discovery file の書き込みと削除）、`config.rs`（`.kivact/devtools.json` の読み込みと workspace root の探索）、`files.rs`（atomic な書き込みと trace の path）、`review_store.rs`（`ReviewStore`）、`image.rs`（`encode_png`、`base64`）、`launcher.rs`（DevTools アプリの起動）、`source.rs`（`name_hint` と file の cache）。
2. `session.rs`:

   ```rust
   //! Role: What the UI thread and the connection threads share: who is connected and what they follow.

   use std::collections::BTreeMap;
   use std::io::Write;
   use std::net::{Shutdown, TcpStream};
   use std::sync::{Arc, Mutex, MutexGuard};
   use kivact_devtools_protocol::{ClientInfo, ClientKind, Topic};

   pub(crate) type ConnectionId = u64;
   /// The write half of a connection; a writer holds the lock for one whole line.
   pub(crate) type Writer = Arc<Mutex<TcpStream>>;

   /// What a connection thread knows about its own connection.
   pub(crate) enum Session {
       AwaitingInitialize,
       Ready,
   }

   /// An initialized connection.
   pub(crate) struct Peer {
       pub(crate) client: ClientInfo,
       pub(crate) topics: Vec<Topic>,
       pub(crate) writer: Writer,
   }

   /// Every initialized connection.
   #[derive(Default)]
   pub(crate) struct Peers(Mutex<BTreeMap<ConnectionId, Peer>>);
   ```

   `impl Peers` の method（どれも `self.lock()` の中で短く済ませる。`fn lock(&self) -> MutexGuard<'_, BTreeMap<ConnectionId, Peer>>` は `self.0.lock().expect("invariant: the peers lock is never poisoned")`）: `insert(&self, id, peer)`、`remove(&self, id) -> usize`（消した後に残っている数）、`set_topics(&self, id, topics: Vec<Topic>)`、`wants(&self, topic: Topic) -> bool`（どれかの `topics` に含まれる）、`writers_for(&self, topic: Topic) -> Vec<Writer>`（含む peer の writer の clone）、`has_client(&self, kind: ClientKind) -> bool`、`shutdown_all(&self)`（全 writer に `shutdown`）。関数:

   ```rust
   /// Write one line and `\n`. A failed write shuts the stream down, so its reader thread ends and cleans up.
   pub(crate) fn write_line(writer: &Writer, line: &str) {
       let mut stream = writer.lock().expect("invariant: the writer lock is never poisoned");
       let written = stream
           .write_all(line.as_bytes())
           .and_then(|()| stream.write_all(b"\n"))
           .and_then(|()| stream.flush());
       if written.is_err() {
           let _ = stream.shutdown(Shutdown::Both);
       }
   }

   pub(crate) fn shutdown(writer: &Writer) {
       let stream = writer.lock().expect("invariant: the writer lock is never poisoned");
       let _ = stream.shutdown(Shutdown::Both);
   }
   ```
3. `incoming.rs`:

   ```rust
   //! Role: What the agent hands the core: calls to answer and connection endings.

   use serde_json::Value;
   use kivact_devtools_protocol::{Call, Reply, RpcError, response, response_line};
   use crate::session::{Writer, write_line};

   /// Something that arrived since the last `Agent::take_incoming`.
   #[allow(clippy::large_enum_variant, reason = "one short-lived value per call; boxing would only add noise")]
   pub enum Incoming {
       /// A call to answer through its `Responder`.
       Call(Call, Responder),
       /// An initialized connection ended; `remaining` initialized connections are still open.
       Closed { remaining: usize },
   }

   /// Where the reply of one call goes. Answer every call exactly once (`send` consumes it);
   /// a client gives up on an unanswered call after 30 s. There is no `Drop`: a call dropped without an answer is simply never answered.
   pub struct Responder {
       id: Value,
       writer: Writer,
   }

   impl Responder {
       pub(crate) fn new(id: Value, writer: Writer) -> Responder {
           Responder { id, writer }
       }
       /// Write the JSON-RPC response line of this call to its connection.
       pub fn send(self, result: Result<Reply, RpcError>) {
           let answer = response(self.id, result.map(Reply::into_value));
           write_line(&self.writer, &response_line(&answer));
       }
   }
   ```
4. `server.rs`（listener thread。停止の仕組みは `Sender<()>`／`Receiver<()>`）:

   ```rust
   //! Role: The listener thread: accepts connections and starts a reader thread for each.

   use std::net::TcpListener;
   use std::sync::{Arc, mpsc};
   use kivact_devtools_protocol::{AppInfo, DevtoolsConfig};
   use crate::connection;
   use crate::incoming::Incoming;
   use crate::session::{ConnectionId, Peers};

   /// What every connection thread reads and nobody changes after start.
   pub(crate) struct Shared {
       pub(crate) token: String,
       pub(crate) app: AppInfo,
       pub(crate) config: DevtoolsConfig,
       pub(crate) peers: Peers,
       pub(crate) wake: Box<dyn Fn() + Send + Sync>,
   }

   /// Accept until `stop` has a value or is disconnected. `Agent::drop` sends `()` and then connects once, so `accept` returns.
   pub(crate) fn listen(listener: TcpListener, stop: mpsc::Receiver<()>, shared: Arc<Shared>, incoming: mpsc::Sender<Incoming>) {
       let mut next_id: ConnectionId = 0;
       for stream in listener.incoming() {
           if !matches!(stop.try_recv(), Err(mpsc::TryRecvError::Empty)) {
               break;
           }
           let Ok(stream) = stream else {
               continue;
           };
           next_id += 1;
           let id = next_id;
           let (shared, incoming) = (Arc::clone(&shared), incoming.clone());
           let spawned = std::thread::Builder::new()
               .name(format!("kivact-devtools-connection-{id}"))
               .spawn(move || connection::run(stream, id, shared, incoming));
           if let Err(error) = spawned {
               tracing::warn!(target: "kivact::devtools", "cannot start a DevTools connection thread: {error}");
           }
       }
   }
   ```
5. `connection.rs`（reader thread。1 行読んで解釈する。`initialize`・`subscribe`・`get_config` はここで答える）:

   ```rust
   //! Role: One connection: the handshake, the calls it forwards to the UI thread and its cleanup.

   use std::io::{BufRead, BufReader, Read};
   use std::net::TcpStream;
   use std::sync::{Arc, Mutex, mpsc};
   use std::time::Duration;
   use serde_json::Value;
   use kivact_devtools_protocol::{
       Call, Empty, Initialize, InitializeResult, Method, PROTOCOL_VERSION, Reply, RpcError, RpcRequest,
       code, parse_call, parse_request,
   };
   use crate::incoming::{Incoming, Responder};
   use crate::server::Shared;
   use crate::session::{ConnectionId, Peer, Session, Writer, shutdown};

   const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
   const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
   const MAX_LINE: usize = 64 * 1024 * 1024;

   enum Step {
       Continue,
       Initialized,
       Close,
   }

   pub(crate) fn run(stream: TcpStream, id: ConnectionId, shared: Arc<Shared>, incoming: mpsc::Sender<Incoming>) {
       if stream.set_write_timeout(Some(WRITE_TIMEOUT)).is_err()
           || stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT)).is_err()
       {
           return;
       }
       let Ok(clone) = stream.try_clone() else {
           return;
       };
       let writer: Writer = Arc::new(Mutex::new(clone));
       let mut reader = BufReader::new(stream);
       let mut session = Session::AwaitingInitialize;
       loop {
           let line = match read_line(&mut reader) {
               Ok(Some(line)) => line,
               Ok(None) => break,
               Err(error) => {
                   if error.kind() == std::io::ErrorKind::InvalidData {
                       reply(&writer, Value::Null, Err(RpcError::new(code::INVALID_REQUEST, error.to_string())));
                   }
                   break;
               }
           };
           if line.trim().is_empty() {
               continue;
           }
           match handle_line(&line, &session, id, &writer, &shared, &incoming) {
               Step::Continue => {}
               Step::Initialized => {
                   session = Session::Ready;
                   let _ = reader.get_ref().set_read_timeout(None);
               }
               Step::Close => break,
           }
       }
       finish(id, &session, &writer, &shared, &incoming);
   }

   /// One line of at most `MAX_LINE` bytes; `Ok(None)` at the end of the stream; a longer line is `InvalidData`.
   fn read_line(reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<String>> {
       let mut bytes = Vec::new();
       let read = reader.by_ref().take(MAX_LINE as u64 + 1).read_until(b'\n', &mut bytes)?;
       if read == 0 {
           return Ok(None);
       }
       if bytes.len() > MAX_LINE {
           return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "line longer than 64 MiB"));
       }
       String::from_utf8(bytes)
           .map(Some)
           .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
   }

   fn reply(writer: &Writer, id: Value, result: Result<Reply, RpcError>) {
       Responder::new(id, Arc::clone(writer)).send(result);
   }

   fn handle_line(
       line: &str,
       session: &Session,
       id: ConnectionId,
       writer: &Writer,
       shared: &Shared,
       incoming: &mpsc::Sender<Incoming>,
   ) -> Step {
       let request = match parse_request(line) {
           Ok(request) => request,
           Err(error) => {
               reply(writer, Value::Null, Err(error));
               return Step::Continue;
           }
       };
       match session {
           Session::AwaitingInitialize => handshake(&request, id, writer, shared),
           Session::Ready => serve(request, id, writer, shared, incoming),
       }
   }

   /// Before `initialize`: only `initialize` is answered (token first, then the protocol version).
   fn handshake(request: &RpcRequest, id: ConnectionId, writer: &Writer, shared: &Shared) -> Step {
       let params = match parse_call(request) {
           Ok(Call::Initialize(params)) => params,
           Err(error) if request.method == Initialize::NAME => {
               reply(writer, request.id.clone(), Err(error));
               return Step::Continue;
           }
           _ => {
               let error = RpcError::new(code::NOT_INITIALIZED, "call initialize first");
               reply(writer, request.id.clone(), Err(error));
               return Step::Continue;
           }
       };
       if params.token != shared.token {
           reply(writer, request.id.clone(), Err(RpcError::new(code::BAD_TOKEN, "bad token")));
           return Step::Close;
       }
       if params.protocol != PROTOCOL_VERSION {
           let message = format!(
               "protocol version mismatch: target {PROTOCOL_VERSION}, client {}",
               params.protocol
           );
           reply(writer, request.id.clone(), Err(RpcError::new(code::VERSION_MISMATCH, message)));
           return Step::Close;
       }
       shared.peers.insert(id, Peer { client: params.client, topics: Vec::new(), writer: Arc::clone(writer) });
       let result = InitializeResult { protocol: PROTOCOL_VERSION, app: shared.app.clone() };
       reply(writer, request.id.clone(), Ok(Reply::Initialize(result)));
       Step::Initialized
   }

   /// After `initialize`: `subscribe` and `get_config` are answered here; every other call goes to the UI thread.
   fn serve(request: RpcRequest, id: ConnectionId, writer: &Writer, shared: &Shared, incoming: &mpsc::Sender<Incoming>) -> Step {
       match parse_call(&request) {
           Err(error) => reply(writer, request.id, Err(error)),
           Ok(Call::Initialize(_)) => {
               reply(writer, request.id, Err(RpcError::new(code::INVALID_REQUEST, "already initialized")));
           }
           Ok(Call::Subscribe(params)) => {
               shared.peers.set_topics(id, params.topics);
               reply(writer, request.id, Ok(Reply::Subscribe(Empty {})));
           }
           Ok(Call::GetConfig(_)) => {
               reply(writer, request.id, Ok(Reply::GetConfig(shared.config.with_defaults())));
           }
           Ok(call) => {
               let responder = Responder::new(request.id, Arc::clone(writer));
               if incoming.send(Incoming::Call(call, responder)).is_err() {
                   return Step::Close;
               }
               (shared.wake)();
           }
       }
       Step::Continue
   }

   /// End of the connection (EOF, a failed read or write, or `Close`): drop the peer and tell the core.
   fn finish(id: ConnectionId, session: &Session, writer: &Writer, shared: &Shared, incoming: &mpsc::Sender<Incoming>) {
       shutdown(writer);
       if matches!(session, Session::Ready) {
           let remaining = shared.peers.remove(id);
           let _ = incoming.send(Incoming::Closed { remaining });
           (shared.wake)();
       }
   }
   ```

   （`use` の中で使われない名前が出たら外す。`Initialize::NAME` は `Method` trait が要る。）
6. `config.rs`:

   ```rust
   //! Role: The workspace root and the `.kivact/devtools.json` configuration.

   use std::path::{Path, PathBuf};
   use kivact_devtools_protocol::DevtoolsConfig;

   /// From the current directory upward: the first directory with `.kivact`, else the first with `Cargo.lock`.
   /// Neither: the same search from the executable's directory (an editor embedded in a DAW has the DAW's cwd).
   /// Neither: the current directory. Nothing is created here.
   pub(crate) fn find_workspace_root() -> PathBuf {
       let cwd = std::env::current_dir().ok();
       let exe_dir = std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf));
       cwd.as_deref()
           .and_then(search)
           .or_else(|| exe_dir.as_deref().and_then(search))
           .or(cwd)
           .unwrap_or_else(|| PathBuf::from("."))
   }

   pub(crate) fn search(start: &Path) -> Option<PathBuf> {
       let marked = |name: &str| {
           start.ancestors().find(|dir| dir.join(name).exists()).map(Path::to_path_buf)
       };
       marked(".kivact").or_else(|| marked("Cargo.lock"))
   }

   /// `.kivact/devtools.json`; a missing file is the default, an unreadable or invalid one is the default plus a warning.
   pub(crate) fn load_config(root: &Path) -> DevtoolsConfig {
       let path = root.join(".kivact").join("devtools.json");
       let text = match std::fs::read_to_string(&path) {
           Ok(text) => text,
           Err(error) if error.kind() == std::io::ErrorKind::NotFound => return DevtoolsConfig::default(),
           Err(error) => {
               tracing::warn!(target: "kivact::devtools", "cannot read {}: {error}", path.display());
               return DevtoolsConfig::default();
           }
       };
       serde_json::from_str(&text).unwrap_or_else(|error| {
           tracing::warn!(target: "kivact::devtools", "invalid {}: {error}", path.display());
           DevtoolsConfig::default()
       })
   }
   ```
7. `files.rs` と `discovery.rs`:

   ```rust
   // files.rs
   //! Role: Atomic file writes and the default path of a trace file.

   use std::path::{Path, PathBuf};

   /// Create the parent directories, write `<path>.tmp`, then rename it over `path`.
   pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
       if let Some(parent) = path.parent() {
           std::fs::create_dir_all(parent)?;
       }
       let mut temp = path.as_os_str().to_owned();
       temp.push(".tmp");
       let temp = PathBuf::from(temp);
       std::fs::write(&temp, contents)?;
       std::fs::rename(&temp, path)
   }

   /// `requested` is relative to `root` (an absolute path stays as it is); `None` is
   /// `.kivact/traces/<session>-<n>.json` with the first `n` from 1 whose file does not exist yet.
   pub(crate) fn trace_path(root: &Path, session: &str, requested: Option<&str>) -> PathBuf {
       if let Some(path) = requested {
           return root.join(path);
       }
       let mut number = 1_u32;
       loop {
           let path = root.join(".kivact").join("traces").join(format!("{session}-{number}.json"));
           if !path.exists() {
               return path;
           }
           number += 1;
       }
   }
   ```

   `discovery.rs`: `pub(crate) fn write(file: &DiscoveryFile) -> std::io::Result<PathBuf>`（`discovery_dir_from(|name| std::env::var_os(name), std::env::temp_dir()).join(discovery_file_name(file.pid, file.port))` の path に、`serde_json::to_string_pretty(file)` を `write_atomic` で書く。path を返す）と `pub(crate) fn remove(path: &Path)`（`let _ = std::fs::remove_file(path);`）。
8. `image.rs`（外部 crate は workspace の `png` だけ）:

   ```rust
   //! Role: PNG and base64 bytes for screenshots.

   /// An 8-bit RGBA PNG of `rgba` (`width * height * 4` bytes, row by row).
   pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> std::io::Result<Vec<u8>> {
       let mut out = Vec::new();
       {
           let mut encoder = png::Encoder::new(&mut out, width, height);
           encoder.set_color(png::ColorType::Rgba);
           encoder.set_depth(png::BitDepth::Eight);
           let mut writer = encoder.write_header().map_err(std::io::Error::other)?;
           writer.write_image_data(rgba).map_err(std::io::Error::other)?;
           writer.finish().map_err(std::io::Error::other)?;
       }
       Ok(out)
   }

   /// Standard base64 with `=` padding.
   pub fn base64(bytes: &[u8]) -> String {
       const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
       let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
       for chunk in bytes.chunks(3) {
           let byte = |index: usize| u32::from(chunk.get(index).copied().unwrap_or(0));
           let n = (byte(0) << 16) | (byte(1) << 8) | byte(2);
           out.push(char::from(TABLE[(n >> 18) as usize & 63]));
           out.push(char::from(TABLE[(n >> 12) as usize & 63]));
           out.push(if chunk.len() > 1 { char::from(TABLE[(n >> 6) as usize & 63]) } else { '=' });
           out.push(if chunk.len() > 2 { char::from(TABLE[n as usize & 63]) } else { '=' });
       }
       out
   }
   ```

   試験（inline の `#[cfg(test)] mod tests { //! Role: … }`）: `base64(b"Man") == "TWFu"`、`base64(b"Ma") == "TWE="`、`base64(b"M") == "TQ=="`、`base64(b"") == ""`。`encode_png(1, 1, &[255, 0, 0, 255])` の先頭 8 byte が `[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]`。
9. `source.rs`:

   ```rust
   //! Role: Read a source line to find the name a signal was bound to (`let count = signal(0)` → `count`).

   use std::path::{Path, PathBuf};
   use std::rc::Rc;
   use kivact_devtools_protocol::SourceLocation;

   const MAX_FILES: usize = 64;

   /// Source files read so far (a pure cache; the oldest is dropped first).
   pub(crate) struct SourceCache {
       files: Vec<(PathBuf, Rc<str>)>,
   }

   impl SourceCache {
       pub(crate) fn new() -> SourceCache {
           SourceCache { files: Vec::new() }
       }
       /// The identifier after `let` (and `mut`) on the line at `location`; `location.file` is relative to `root`.
       pub(crate) fn name_hint(&mut self, root: &Path, location: &SourceLocation) -> Option<String> {
           let text = self.text(&root.join(&location.file))?;
           let index = usize::try_from(location.line).ok()?.checked_sub(1)?;
           hint_in_line(text.lines().nth(index)?)
       }
       fn text(&mut self, path: &Path) -> Option<Rc<str>> {
           if let Some((_, text)) = self.files.iter().find(|(known, _)| known == path) {
               return Some(Rc::clone(text));
           }
           let text: Rc<str> = std::fs::read_to_string(path).ok()?.into();
           if self.files.len() >= MAX_FILES {
               self.files.remove(0);
           }
           self.files.push((path.to_path_buf(), Rc::clone(&text)));
           Some(text)
       }
   }

   fn hint_in_line(line: &str) -> Option<String> {
       let word_char = |c: char| c.is_alphanumeric() || c == '_';
       let start = line.match_indices("let ").find_map(|(index, _)| {
           let before = line[..index].chars().next_back();
           (!before.is_some_and(word_char)).then_some(index + 4)
       })?;
       let rest = line[start..].trim_start();
       let rest = rest.strip_prefix("mut ").unwrap_or(rest).trim_start();
       let name: String = rest.chars().take_while(|c| word_char(*c)).collect();
       (!name.is_empty()).then_some(name)
   }
   ```

   試験: `hint_in_line("    let mut count = signal(0);") == Some("count")`、`"let (a, b) = pair();"` は `None`、`"outlet x"` は `None`、`"let _ = x"` は `Some("_")`。
10. `review_store.rs`（`Agent` が持つ。設計書 4.2.4 の review）:

    ```rust
    pub struct NewComment {
        pub window: WindowId,
        pub window_title: String,
        pub target: ReviewTarget,
        pub style_changes: Vec<StyleChange>,
        pub image_png: Option<Vec<u8>>,
        pub text: String,
    }
    pub struct ReviewStore { dir: PathBuf, document: ReviewDocument, next_item: u64, next_comment: BTreeMap<String, u64> }
    impl ReviewStore {
        pub(crate) fn new(dir: PathBuf, app: AppInfo, session: String, now_ms: u64) -> ReviewStore;
        pub fn document(&self) -> &ReviewDocument;
        /// Add a comment to the item of the same target in the same window (a new item when none), save, and return the item.
        pub fn add(&mut self, comment: NewComment, now_ms: u64) -> std::io::Result<ReviewItem>;
        pub fn edit(&mut self, comment: &str, text: String, now_ms: u64) -> Result<ReviewItem, String>;
        pub fn remove(&mut self, comment: &str, now_ms: u64) -> Result<(), String>; // an item with no comment left is removed
        pub fn set_environment(&mut self, environment: Emulation); // written by the next save
        /// Save both files and return their paths.
        pub fn export(&self) -> std::io::Result<ExportedReview>;
    }
    ```

    （`add` の引数を 1 つの struct にまとめるのは、`self` を入れて 8 個になると clippy の `too_many_arguments` に当たるため。）
    - `new`: `document = ReviewDocument { schema: "kivact-devtools-review/1", app, session, created: rfc3339_utc(now_ms), updated: 同じ, environment: <次の値>, items: vec![] }`。`Emulation` は `Default` を持たないので、`environment` には `Emulation { theme: None, reduced_motion: None, locale: None, pseudo_locale: PseudoLocale::Off, scale: None, touch: None }` を書く。`next_item: 1`。
    - 同じ target の判定: `Node` は `window` と `node` の値、`Window` は `window` の値。
    - `add` の順: (1) item を探す。無ければ `id = format!("C{}", self.next_item)` で作り `next_item += 1`（消しても再利用しない）。(2) comment の番号は `*self.next_comment.entry(item.id.clone()).or_insert(0) += 1`、`Comment.id = format!("{item_id}.{n}")`、`created = rfc3339_utc(now_ms)`。(3) `style_changes` は渡されたもので置き換える。(4) `image_png` があれば `<dir>/images/<item id>.png` に `write_atomic` で書き（同じ item の古い画像は同じ path なので置き換わる）、`item.image = Some(format!("images/{item_id}.png"))`。(5) `document.updated = rfc3339_utc(now_ms)`。(6) `save()`。
    - `save(&self) -> io::Result<()>`: `<dir>/review.json`（`serde_json::to_string_pretty(&document)`）と `<dir>/review.md`（`to_markdown(&document)`）を `write_atomic` で書く。directory は `write_atomic` が最初に作る。
    - `edit`／`remove` の comment の探し方: `comment` の `.` より前が item の id。無ければ `Err(format!("no such comment: {comment}"))`。`edit`・`remove` も、変えた後に `document.updated` を更新して `save()` する（失敗は `Err(error.to_string())`）。
    - `export`: `save()` して `ExportedReview { json: <dir>/review.json の文字列, markdown: <dir>/review.md の文字列 }`（`to_string_lossy().into_owned()`）。
    - 試験（inline）: 一時 directory（`std::env::temp_dir().join(format!("kivact-review-test-{}", std::process::id()))`）で、2 つの comment を同じ node に足すと item は 1 つ・comment id は `C1.1`・`C1.2`、別の node は `C2`、`C1.1` を消して足すと `C1.3`、item の comment を全部消すと item が消え `C1` は再利用されない、`review.json` と `review.md` ができる、`image_png` を渡すと `images/C1.png` ができる。終わりに directory を消す。
11. `launcher.rs`:

    ```rust
    //! Role: Start the DevTools app for a target.

    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use kivact_devtools_protocol::DevtoolsConfig;

    const NOT_FOUND: &str = "DevTools app not found. Install it with `cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools` or set \"app\" in .kivact/devtools.json";

    /// Start the first DevTools app that can be found, with `--target <discovery path>`. The child is not waited for.
    pub(crate) fn launch(config: &DevtoolsConfig, root: &Path, discovery_path: &Path) -> Result<(), String> {
        let exe_dir = std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf));
        let env_app = std::env::var_os("KIVACT_DEVTOOLS_APP");
        for path in candidates(config, root, env_app, exe_dir.as_deref()) {
            let mut command = std::process::Command::new(&path);
            command.arg("--target").arg(discovery_path);
            match command.spawn() {
                Ok(_) => return Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("cannot start {}: {error}", path.display())),
            }
        }
        Err(NOT_FOUND.to_owned())
    }

    /// In order: the `app` setting, `KIVACT_DEVTOOLS_APP`, next to this executable, `kivact-devtools` on PATH.
    pub(crate) fn candidates(config: &DevtoolsConfig, root: &Path, env_app: Option<OsString>, exe_dir: Option<&Path>) -> Vec<PathBuf> {
        let mut found = Vec::new();
        if let Some(app) = &config.app {
            found.push(root.join(app));
        }
        if let Some(app) = env_app.filter(|app| !app.is_empty()) {
            found.push(root.join(app));
        }
        if let Some(dir) = exe_dir {
            found.push(dir.join(format!("kivact-devtools{}", std::env::consts::EXE_SUFFIX)));
        }
        found.push(PathBuf::from("kivact-devtools"));
        found
    }
    ```

    試験（inline）: `candidates` が 4 つ（設定あり・環境変数あり・exe_dir あり）を決めた順に返す、設定が無ければ 3 つ、最後が `kivact-devtools`。
12. `agent.rs`:

    ```rust
    pub struct AgentConfig { pub app: AppInfo }

    pub struct Agent {
        shared: Arc<Shared>,
        incoming: mpsc::Receiver<Incoming>,
        stop: mpsc::Sender<()>,
        listener: Option<std::thread::JoinHandle<()>>,
        port: u16,
        discovery_path: PathBuf,
        workspace_root: PathBuf,
        review: ReviewStore,
        names: SourceCache,
        session: String,
    }
    ```

    `Agent::start(config, wake)` の順: (1) `let workspace_root = config::find_workspace_root();` `let devtools_config = config::load_config(&workspace_root);` `let mut app = config.app; app.workspace_root = workspace_root.to_string_lossy().into_owned();`。(2) `TcpListener::bind("127.0.0.1:0")?`、`port = listener.local_addr()?.port()`。(3) `token = make_token()`。(4) `started_ms = now_ms()`、`session = format!("{}-{}-{}", compact_utc(started_ms), app.pid, port)`。(5) `discovery::write(&DiscoveryFile { protocol: PROTOCOL_VERSION, pid: app.pid, app_name: app.name.clone(), kivact_version: app.kivact_version.clone(), port: u32::from(port), token: token.clone(), started_unix_ms: started_ms, workspace_root: app.workspace_root.clone() })?`。(6) `Shared { token, app, config: devtools_config, peers: Peers::default(), wake }` を `Arc` に。(7) `mpsc::channel()` を 2 組（incoming、stop）。(8) `std::thread::Builder::new().name("kivact-devtools-listener".into()).spawn(move || server::listen(listener, stop_rx, shared_for_thread, incoming_tx))?`。(9) `ReviewStore::new(workspace_root.join(".kivact").join("review").join(&session), shared.app.clone(), session.clone(), started_ms)` と `SourceCache::new()` で `Agent` を作る。補助の関数:

    ```rust
    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() as u64)
    }

    /// `20261108-053200` from the Unix time (UTC).
    fn compact_utc(unix_ms: u64) -> String {
        rfc3339_utc(unix_ms).replace(['-', ':'], "").replace('T', "-").trim_end_matches('Z').to_owned()
    }

    /// 32 random bytes as hex. No external crate: `RandomState` is seeded by the OS. Loopback only, so this
    /// only has to be hard to guess, not cryptographic.
    fn make_token() -> String {
        use std::hash::{BuildHasher, Hasher};
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        (0..4_u8)
            .map(|round| {
                let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
                hasher.write_u128(nanos);
                hasher.write_u32(std::process::id());
                hasher.write_u8(round);
                format!("{:016x}", hasher.finish())
            })
            .collect()
    }
    ```

    method: `take_incoming`（`self.incoming.try_iter().collect()`）、`notify`（`let writers = self.shared.peers.writers_for(notification.topic());` が空なら返る。`notification_line(&notification)` を 1 回作り、各 writer へ `write_line`）、`wants`（`self.shared.peers.wants(topic)`）、`open_devtools`（`self.shared.peers.has_client(ClientKind::Devtools)` なら `self.notify(Notification::FocusRequested(Empty {}))` して `Ok(())`、そうでなければ `launcher::launch(&self.shared.config, &self.workspace_root, &self.discovery_path)`）、`config`（`&self.shared.config`）、`discovery_path`、`workspace_root`、`app`（`&self.shared.app`）、`review`（`&mut self.review`）、`name_hint`（`self.names.name_hint(&self.workspace_root, location)`）、`write_trace`（`let path = files::trace_path(&self.workspace_root, &self.session, requested); files::write_atomic(&path, contents.as_bytes())?; Ok(path)`）。
    `Drop for Agent`:

    ```rust
    impl Drop for Agent {
        fn drop(&mut self) {
            let _ = self.stop.send(());
            let _ = std::net::TcpStream::connect(("127.0.0.1", self.port));
            if let Some(listener) = self.listener.take() {
                let _ = listener.join();
            }
            self.shared.peers.shutdown_all();
            discovery::remove(&self.discovery_path);
        }
    }
    ```

    reader thread は join しない（client が接続したままでも、target の終了が止まらない）。`Agent` は `Rc` を持つので `Send` ではない。
13. 試験 `tests/agent.rs`（`//! Role:` 付き。client crate は dev-dependency。`Agent` は `Send` ではないので、`Agent` は試験の thread に置き、client を別 thread で動かす）。`Agent::start(AgentConfig { app: sample_app() }, Box::new(|| {}))` で起動し、`kivact_devtools_client::read_target(agent.discovery_path())` で `DiscoveryFile` を読む。確かめること:
    - `get_app_info`: 別 thread で `Connection::open` と `call::<GetAppInfo>(Empty {})`。試験の thread は `take_incoming` を 10ms ごとに回し、`Incoming::Call(Call::GetAppInfo(_), responder)` が来たら `responder.send(Ok(Reply::GetAppInfo(agent.app().clone())))`。client が受け取った `AppInfo.workspace_root` が空でない（agent が埋めた値）。
    - `get_config` は `take_incoming` を回さなくても答える（reader thread が答える）。結果の `keys` が既定で埋まっている。
    - token が違う `DiscoveryFile` では `Connection::open` が `ClientError::Rpc` の `code::BAD_TOKEN`。生の `TcpStream` で `protocol: 99` の `initialize`（`request_line` と `InitializeParams`）を送ると `code::VERSION_MISMATCH` の response が返る。`initialize` の前に `get_app_info` を送ると `code::NOT_INITIALIZED`。
    - `subscribe`: client が `Subscribe` で `Topic::Windows` を選ぶと `agent.wants(Topic::Windows)` が true（client の call が返った後に確かめる）、`agent.notify(Notification::WindowsChanged(..))` が client の `next_notification` に届く。購読していない topic の通知は届かない。
    - client の `Connection` を drop すると、`take_incoming` に `Incoming::Closed { remaining: 0 }` が来る。client が 2 つあって 1 つ閉じると `remaining: 1`。
    - `open_devtools`: kind が `Devtools` の client が接続していれば `Ok(())` で `focus_requested` がその client に届く。
    - `Drop` で discovery file が消える（`agent.discovery_path().to_path_buf()` を先に取っておく）。client が接続したままの `Drop` が止まらない。
14. 確認: `cargo check -p kivact-devtools-agent --all-targets`、`cargo test -p kivact-devtools-agent`、`cargo clippy -p kivact-devtools-agent --all-targets -- -D warnings`。

### T2.3 MCP server（W3 で行う）

担当ファイル: `crates/kivact-devtools-mcp/**`。

参照: 設計書 4.6 節。型は `use kivact_devtools_protocol as p;`、client は `use kivact_devtools_client as client;` で書く。

手順:

1. module: `main.rs`（`fn main` だけ。schema を読んで `server::run(schema)` を呼ぶ）、leaf: `server.rs`（stdin を 1 行ずつ読み、stdout に 1 行ずつ書く loop）、`tools.rs`（tool の一覧と input schema の作成）、`target.rs`（target の選択と接続。下の `Selection`）、`result.rs`（protocol の結果 → MCP の `CallToolResult`）。`target.rs` に置く型:

   ```rust
   /// The target the tools talk to.
   pub(crate) enum Selection { None, Selected { path: PathBuf, connection: client::Connection } }
   /// Why a tool call failed: an answer of the agent, or a text for the AI.
   pub(crate) enum ToolError { Rpc(p::RpcError), Text(String) }
   impl Selection {
       /// `select_target`: open `path` and keep it.
       pub(crate) fn select(&mut self, path: &Path) -> Result<(), ToolError>;
       /// Any other tool: choose a target if none is selected (rule in 4.6), then `call_raw`.
       pub(crate) fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, ToolError>;
   }
   /// `list_targets`: `{ "targets": [ { "target", "app_name", "pid", "port", "kivact_version", "started_unix_ms", "workspace_root" } ] }` (no token).
   pub(crate) fn list_targets() -> serde_json::Value;
   ```

   `call` の決め方（設計書 4.6）: `Selection::None` なら、環境変数 `KIVACT_DEVTOOLS_TARGET` → `client::list_targets()` が 1 つならそれ → それ以外は `ToolError::Text`（0 個は "no running target", 2 個以上は一覧を書いて `select_target` を呼ぶよう伝える）。接続は `client::Connection::open(&file, p::ClientInfo { name: "kivact-devtools-mcp".into(), version: env!("CARGO_PKG_VERSION").into(), kind: p::ClientKind::Mcp })`。`call_raw` が `Err(client::ClientError::Rpc(e))` なら `ToolError::Rpc(e)`、`Err(client::ClientError::Closed)`（T1.2 の読み込み loop は EOF でも I/O の失敗でも `Closed` で終わる。`Io` は見ない）なら `*self = Selection::None` にしてから `ToolError::Text("target closed; call list_targets".into())`、それ以外の `Err(e)` は `ToolError::Text(e.to_string())`。
2. `tools.rs`: schema は protocol crate が公開する `p::SCHEMA_JSON`（`&str`。crate をまたぐ `include_str!` は使わない）を `main.rs` で `serde_json::Value` に読み（`server::run` に渡す。`static`・`OnceLock` は使わない）、`x-methods` から設計書 4.6 の除外（`initialize`・`subscribe`・`show_table` の 3 つだけ）を除いた method を tool にする。`get_config`・`get_owners`・`get_overlays` を含む残りはすべて tool である。`inputSchema` は params の型の schema の `$ref` を `$defs` から再帰的に展開したもの（展開の深さは 16 まで。超えたら panic ではなく `{}` にする）。tool の `description` は method の description。先頭に local の 2 つを足し、`inputSchema` は手で書く:
   - `list_targets`: `{"type":"object","properties":{},"additionalProperties":false}`。
   - `select_target`: `{"type":"object","properties":{"target":{"type":"string","description":"Discovery file path from list_targets."}},"required":["target"],"additionalProperties":false}`。
3. `server.rs`: request の method ごとに: `initialize` → `{"protocolVersion": <受け取った版>, "capabilities": {"tools": {}}, "serverInfo": {"name": "kivact-devtools-mcp", "version": env!("CARGO_PKG_VERSION")}}`、`notifications/initialized` → 何もしない、`ping` → `{}`、`tools/list` → `{"tools": [...]}`、`tools/call` → `name` が `list_targets` なら `target::list_targets()`、`select_target` なら `Selection::select`（結果は `{}`）、それ以外は `arguments` を `Selection::call(name, arguments)` に渡す。それ以外の method → `-32601`。
4. `result.rs`: 成功 → `{"content":[{"type":"text","text":<結果の JSON の文字列>}],"structuredContent":<結果>}`（結果は常に object。protocol の result はすべて object であり、`list_targets` も `{"targets":[…]}`）。`screenshot` → `{"content":[{"type":"image","data":<png_base64>,"mimeType":"image/png"}],"structuredContent":{"width":<n>,"height":<n>}}`。`ToolError::Rpc(e)` → `{"content":[{"type":"text","text":"<code>: <message>"}],"isError":true}`、`ToolError::Text(t)` → 同じ形で text が `t`。
5. log は `std::io::stderr().lock()` に書く。
6. 試験 `crates/kivact-devtools-mcp/tests/mcp.rs`（`//! Role:`。C2 で走る。task の中では走らせない）:
   - 偽の agent: `std::net::TcpListener::bind("127.0.0.1:0")` の thread。1 行ずつ読み、`initialize` には `p::InitializeResult` を、`get_app_info` には `p::AppInfo` を JSON-RPC の response で返す（token は discovery file に書いたものと一致を確かめる）。
   - discovery file を `std::env::temp_dir()` の新しい directory に書く（`p::DiscoveryFile` を `serde_json` で）。
   - `std::process::Command::new(env!("CARGO_BIN_EXE_kivact-devtools-mcp"))` を、stdin・stdout を pipe、環境変数 `KIVACT_DEVTOOLS_TARGET` に discovery file の path で起動し、`initialize`、`tools/list`、`tools/call`（`get_app_info`、`list_targets`）を 1 行ずつ送って response を読む。
   - 確かめること: tool 名の集合が `p::METHOD_NAMES` から `initialize`・`subscribe`・`show_table` を除いたものに `list_targets`・`select_target` を足した集合と等しい。どの `inputSchema` の文字列にも `"$ref"` が含まれない。`get_app_info` の `structuredContent.name` が偽の agent の返した名前と一致する。`list_targets` の結果が object で、`targets[0].target` が discovery file の path で、`token` の key がどこにも無い。偽の agent が接続を閉じた後の `get_app_info` が `isError: true` で、text が "target closed" を含む。終わったら子 process を `kill` して temp directory を消す。
7. 確認: `cargo check -p kivact-devtools-mcp --all-targets`、`cargo clippy -p kivact-devtools-mcp --all-targets -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate kivact-devtools-mcp --report`（0.2 の 12.）。
### T2.4 kivact-types（複数ウィンドウの値と TimeScale）

担当ファイル: `crates/kivact-types/src/window.rs`（新規）、`crates/kivact-types/src/lib.rs`、`crates/kivact-types/src/env.rs`、`crates/kivact-types/src/platform.rs`、`crates/kivact-types/src/motion.rs`、`crates/kivact-types/src/semantics/values.rs`、`crates/kivact-types/src/render/composition.rs`、`crates/arvane-compose/src/plan/hash.rs`、およびこの変更でコンパイルが通らなくなった箇所（下の 5.）。

参照: 設計書 3.1.1、3.2、3.1 の `DismissReason`、4.4.2 の `LayerKey::Devtools`、4.9 の `TimeScale`。

手順:

1. `window.rs` を設計書 3.2 のとおりに作り（`WindowRequest::main` を含む）、`lib.rs` から `pub use window::{WindowId, WindowMode, WindowPlacement, WindowRequest};`。
2. `env.rs` の `Capabilities` に `multi_window`・`window_placement` を足す（doc は設計書 3.2）。`Default` が derive なら false になる。`platform.rs` の `PlatformRequest` に `FocusWindow` を足す（doc: `/// Bring the window to the front and give it the keyboard focus (best effort).`）。
3. `semantics/values.rs` の `DismissReason` に `WindowClose`（doc: `/// The user asked to close the window (its title bar button or a platform shortcut). A window closes itself unless its \`on_dismiss\` keeps it open.`）を足す。
4. `render/composition.rs` の `LayerKey` に `Devtools`（doc: `/// The DevTools overlay of a window, drawn above everything and never hit-tested.`）を足す。`motion.rs` の `MotionClock` の後に `TimeScale` を作る（runtime と engine が使う一つの型。`lib.rs` の `pub use motion::{..}` に足す）:

   ```rust
   /// How fast motion runs for DevTools: since the anchor, motion time passes
   /// `scale` times as fast as host time (0 pauses, 1 is normal).
   #[derive(Clone, Copy, Debug, PartialEq)]
   pub struct TimeScale {
       /// The host time (ns) the current scale began at.
       pub anchor_host_ns: u64,
       /// The motion time (ns) at that moment.
       pub anchor_motion_ns: u64,
       pub scale: f64,
   }

   impl TimeScale {
       /// The motion time (ns) at host time `host_ns` (not before the anchor).
       pub fn motion_ns(&self, host_ns: u64) -> u64 {
           let elapsed = host_ns.saturating_sub(self.anchor_host_ns);
           self.anchor_motion_ns.saturating_add((elapsed as f64 * self.scale) as u64)
       }
       /// The same clock continued from `host_ns` at the new `scale`.
       pub fn rescaled(&self, host_ns: u64, scale: f64) -> TimeScale {
           TimeScale { anchor_host_ns: host_ns, anchor_motion_ns: self.motion_ns(host_ns), scale }
       }
   }

   impl Default for TimeScale {
       fn default() -> Self {
           TimeScale { anchor_host_ns: 0, anchor_motion_ns: 0, scale: 1.0 }
       }
   }
   ```
5. `cargo check --workspace --all-targets --exclude xtask --exclude kivact-devtools-agent` を実行し（W2 で同時に書かれている 2 つを外す。全体は波の検査で指揮役が確かめる）、上の変更で壊れた箇所だけを直す: `Capabilities { .. }` を全 field で組み立てている箇所（native の `capabilities()`、virtual の既定、embed の port など）には `multi_window: false, window_placement: false` を足す（値の正しい設定は後の task。ここでは false）、`DismissReason`・`LayerKey`・`PlatformRequest` を網羅する `match` には新しい arm を足す（`DismissReason::WindowClose` は `Escape` と同じ扱い、`LayerKey::Devtools` は `LayerKey::Root` と同じ扱いの arm を、既存の arm に `|` で足す。`PlatformRequest::FocusWindow` は、native の winit・win32・x11 の `perform` では `PlatformResult::Unsupported` を返す arm にしておく。実際の処理は T4.1）。ただし `arvane-compose/src/plan/hash.rs` の `layer_key` は `LayerKey::Devtools => h.tag(4),` にする（Root と同じ tag にしない。own-parameter の hash が Root と同じになり、変化の検知が鈍るため）。これ以外の変更はしない。
6. 確認: `cargo check --workspace --all-targets --exclude xtask --exclude kivact-devtools-agent`。
### T2.5 DevTools のガイド

担当ファイル: `docs/for-kivact-user/{en,jp}/how-to-guide/devtools.md`（新規）と how-to の一覧。

参照: 設計書 4.2.7（設定）、4.3（agent）、4.5〜4.8、D-16〜D-18（1 章）。

手順: 既存の how-to guide の書き方で、次の節をこの順で書く。en と jp は同じ構成。Rust の code block は Cargo.toml の例だけにし、言語を `toml` にする（doctest にしない）。JSON は `json`、Python は `python`、shell は `sh` の block にする。

1. 「Install the tools」: Kivact の repo の外で使えるようにする。`cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools kivact-devtools-mcp`（2 つとも PATH に入る。agent が DevTools アプリを探す 4 番目の場所が PATH）。Python は `pip install "kivact-devtools @ git+https://github.com/Kashiwade-music/kivact#subdirectory=python/kivact-devtools"`（`pip install -e python/kivact-devtools` は Kivact の repo を編集する人だけ）。
2. 「Enable DevTools」: アプリの Cargo.toml に feature を足し、release のビルドに agent を入れない。

   ```toml
   [dependencies]
   kivact = { git = "https://github.com/Kashiwade-music/kivact" }

   [features]
   devtools = ["kivact/devtools"]
   ```

   `cargo run --features devtools` で起動する。feature を付けた release のビルドは起動時に警告を出す。`.gitignore` に `.kivact/review/` と `.kivact/traces/` を足す。
3. 「Open DevTools」: F12・Ctrl+Shift+I で開く、Ctrl+Shift+C は pick mode から開く（押した window で始まる）。すでに開いていれば前面に出る。`.kivact/devtools.json` の例（`"$schema": "https://raw.githubusercontent.com/Kashiwade-music/kivact/main/crates/kivact-devtools-protocol/schema/devtools-config.schema.json"`、`keys.open`、`keys.pick`、`editor`）と、`keys.open: []` でそのキーを無効にできること、`editor` の既定（`code`。Windows では `code.cmd` を自動で試す）。DevTools アプリが見つからないときの message と対処（`cargo install` か `app`）。`.kivact/` は最初に file を書くときに作られる。
4. 「Inspect elements」: Elements の tree、pick、作成位置から editor を開く、「Copy locator」（test id があればそれ、副 window は `screen.window("…").` 付き）、「Owners」の切り替え、Layout の重ね表示。
5. 「Edit styles」: override の 3 つの範囲、forced states、変更を review に記録。
6. 「Comment and export a review」: コメントの追加。`review.json` と `review.md` は変更のたびに自動で `.kivact/review/<YYYYMMDD-HHMMSS>-<pid>-<port>/` に保存され、「Open folder」で開く。JSON と Markdown の例（Markdown は設計書 4.8 の例）。消えた node の item には「(removed)」が出る。
7. 「See why the screen updated」、「Measure performance」（paint flashing、Perfetto 形式の書き出しは `.kivact/traces/` に番号付きの名前で書かれる）、「Check accessibility」、「Emulate」（theme の emulation は `platform::theme_hint()` に従うアプリにだけ効く）。
8. 「Use DevTools from an AI agent (MCP)」: `claude mcp add kivact-devtools -- kivact-devtools-mcp` と tool の一覧の要約。最初に `list_targets` と `select_target`（`{ "target": "<path>" }`。token は出ない）、target が 1 つなら選ばなくても動く。`inject_input` の後の turn は `list_turns` に `wait_ms` を付けて待つ。
9. 「Script DevTools with Python」: 導入（1.）、VS Code、例（`kd.connect()`、`win.get_by_role(kd.Role.BUTTON, "Increment")`、`win.all_by_role(kd.Role.BUTTON)`、`win.press_key("Ctrl+Enter")`、`with app.record() as rec:`、`app.export_review()`。設計書 4.7.3 の例）、Scripts パネル、`app.rpc` が低水準の method であること。
10. 「Limits」: embed の App では `inject_input` が使えない（`-32015`）。複数の client は同じ状態を共有し、最後の書き込みが勝つ。最後の client が切れると、DevTools が作った状態（pick・重ね表示と強調・style の override・forced states・emulation・時間の倍率・記録）は元に戻り、review だけ残る。layer の境界の表示は無い。`focus_window` が効かない環境では、DevTools アプリは前面に出ない。

確認: 無し。
## 5. W3（依存の段 1）

| task | 担当 | crate |
|---|---|---|
| T3.1 | 1 | `crates/arvane-shell` |
| T3.2 | 2 | `crates/kivact-runtime` |
| T3.3 | 3 | `crates/kivact-layout` |
| T2.3 | 4 | `crates/kivact-devtools-mcp`（節は W2 の中にある） |

この波の後、arvane-shell を使う crate（native、virtual、arvane、engine 以降）は W6 まで一時的にコンパイルが通らない。それが正しい。波の検査: `cargo check -p arvane-shell -p kivact-layout -p kivact-devtools-mcp --all-targets`、`cargo check -p kivact-runtime --lib`、clippy（同じ範囲。runtime は `--lib`）。commit: `feat(shell): multi-window loop SPI; runtime and layout hooks; MCP server`。

### T3.4 W1・W2 で作った crate の state-fields（W4 で行う。MCP が client を使うので、T2.3 と同じ波にしない）

担当ファイル: `crates/kivact-devtools-client/src/**`、`crates/kivact-devtools-agent/src/**`（印の行と、印では済まない struct の形だけ）。

手順: `cargo xtask check --no-write --only state-fields --crate kivact-devtools-client --report` と `--crate kivact-devtools-agent`。確認: 上の 2 つのコマンドの「combines state fields」の error が 0。

この 2 つの crate は、state-fields の error が出ない形に設計してある（出たら設計からずれている）:

- protocol crate は `value_modules` に入れてあるので検査されない（T0.1）。
- client の `Connection` の field は `writer`・`calls`（閉じた状態を持つ `enum Calls { Open(..), Closed(ClientError) }`）・`next_id: AtomicU64`・`notifications`。`Option` も `bool` も無い（閉じたかどうかは `Calls` が持つ）。
- agent の `Agent` の `Option` は `listener: Option<JoinHandle<()>>` だけ（`Drop` で `take()` して join する）。停止は `stop: mpsc::Sender<()>` と、listener thread の `Receiver` で行い、`AtomicBool` の flag は持たない。`Shared`・`Peer`・`Peers`・`Responder`・`ReviewStore`・`SourceCache`・`NewComment` に、`Option`・`bool` が 2 つ以上並ぶ struct は無い（`NewComment` の `Option` は `image_png` だけ）。

error が出た struct があれば、その struct の field を調べ、次の順で直す:

1. 2 つ以上の `Option` や `bool` が同じ状態を表している（例: 「閉じたか」の `bool` と、閉じた理由の `Option`）なら、明示的な enum にする（`Calls` のように、状態ごとの値を variant の payload に入れる）。
2. 状態ではない独立した設定なら、その field の直前に理由付きの印を書く。agent の `listener` を印で済ませるときの文: `// xtask-allow(state-fields): owned resource - the listener thread's join handle, taken once by Drop`。`Connection.next_id` を印で済ませるときの文: `// xtask-allow(state-fields): S-8 ID - the next request id of this connection`。
3. 上のどれでもなければ直さず、報告に「T15.4 へ」と書く。

「struct-level marker」と「unused marker」の error は T15.4 がまとめて直すので、ここでは直さない。
### T3.1 arvane-shell: 複数ウィンドウの SPI と WindowInput

担当ファイル: `crates/arvane-shell/src/window.rs`、`events.rs`、`types.rs`、`lib.rs`、`pipeline/raw.rs`、`input.rs`（新規）、`window_input.rs`（新規）、`dev/xtask/policy/workspace.toml`（`[module_rules.arvane-shell.uses]` の 3 行だけ。手順 7）。

参照: 設計書 3.3、3.1.1。今の定義は `window.rs:31-224`、`events.rs:6-75`、`types.rs:7-128`、`lib.rs:43-64`。

手順:

1. `window.rs`: `EventTarget`、`ShellLoop`（`inject(id, inputs, request: RequestId)` と `waker()` を含む。設計書 3.3。`use kivact_types::{RequestId, WindowId, WindowRequest};`、`use crate::input::InjectedInput;`）を足し（`impl EventTarget { pub fn of(event: &ShellEvent, window: WindowId) -> EventTarget }` も。`SaveState`・`Wake(_)`・`Suspended`・`Resumed`・`MemoryWarning` は `Loop`、それ以外は `Window(window)`。native・virtual・embed が同じ分類を使う）、`ShellHandler`・`Backend`・`SteppedBackend` を設計書 3.3 の形にする。`ShellWindow` には設計書 3.3 の `fn focus(&mut self) -> Result<(), ShellError>;`（doc 付き）を足す。ほかは変えない。`Step`、`StepOutcome` は変えない（doc の「one window」を「any window」に直す）。`events.rs` の `ShellEvent` に `Injected { request: RequestId }` を足す（doc: `/// Everything injected with this request was delivered to the window.`）。
2. `types.rs`: `ShellOptions` の struct と `impl Default` を消す（main window の要求も `kivact_types::WindowRequest` で渡す。設計書 3.2）。`lib.rs` の `pub use types::{..}` から `ShellOptions` を外し、crate の doc（6 行）の「[`ShellOptions`] configures a window」を「[`kivact_types::WindowRequest`] describes a window」に直す。`WakeReasons` には何も足さない（注入した入力は callback が返った直後に配る。設計書 3.3）。
3. `pipeline/raw.rs`: `RawInput`・`RawKey`・`PlatformKey`・`RawIme`・`RawDrop` の derive に `Clone, PartialEq` を足す。
4. `input.rs`: `InjectedInput`（設計書 3.3。`Accessibility` の variant を含む。先頭に `//! Role: The input a tool asks a loop to deliver, before normalization.`）。
5. `window_input.rs`（先頭に `//! Role: One window's input normalization, shared by the native port, the virtual backend and injected input.`）: `WindowInput`（設計書 3.3。`#[derive(Default)]`、`new()` は `Self::default()`。IME の driver は持たない）。中身は既存の部品をつなぐだけ。定数 `const WHEEL_LINE_PX: f64 = 20.0;`（native の `wheel.rs` の 1 行 20 px と同じ）を置く。
   - `platform(event, now_ns, double_click)`: native の `Runner::input`（`crates/arvane-native/src/port/winit/runtime.rs:605-614`）と virtual の `enqueue`（`crates/arvane-virtual/src/script.rs:191-211`）がしていることを一つにする。`Pointer(sample)` は `sample.modifiers = self.modifiers.current()`、`self.mouse.observe(*sample)`、そして `self.clicks.count(&mut event, || now_ns, || double_click)`。`Modifiers(m)` は `self.modifiers.update(m)`（戻り値は捨てる。event はそのまま返す）。それ以外の event はそのまま返す。`Focused(false)` には何もしない（解放は `focus_lost`。呼び出し側が先に `focus_lost()` の event を配る）。
   - `mouse(phase, position, now_ns, double_click)`: `self.mouse.event(phase, position)` が `Some(sample)` なら `Some(self.platform(ShellEvent::Pointer(sample), ..))`、`None` なら `None`。
   - `modifiers_changed(m)`: `self.modifiers.update(m)`（`ModifierState::update`。変わったときだけ `Some(ShellEvent::Modifiers(m))`）。
   - `focus_lost()`: (1) modifiers が既定でなければ `ShellEvent::Modifiers(Modifiers::default())` を作って `self.modifiers.update(Modifiers::default())`、(2) `self.keys.focus_lost()`、(3) `self.contacts.cancel()` の event（`platform` に通す）、(4) `self.mouse.cancel()` が `Some(sample)` なら `platform` に通した `Pointer(sample)`、の順に集めて返す（`script.rs` の `Focused(false)` の処理と `runtime.rs:822-829` を一つにしたもの）。
   - `mouse_position()` は `self.mouse.window_position()`、`last_mouse_position()` は `self.mouse.position()`、`modifiers()` は `self.modifiers.current()`、`dnd_mut`・`contacts_mut`・`keys_mut` は field の `&mut`。
   - `normalize(input, now_ns, double_click, ime)`: 結果の各 event は `platform` を通してある。
     - `MouseMove(p)` → `self.mouse(PointerPhase::Move, Some(p), ..)`。`MouseDown(b)`・`MouseUp(b)` → `self.mouse(PointerPhase::Down(ButtonPress { button: b, clicks: 1 }) / Up(..), None, ..)`。`MouseLeave` → `self.mouse(PointerPhase::Leave, None, ..)`。
     - `Wheel { lines_x, lines_y }` → `self.last_mouse_position()` が `Some(position)` のときだけ `ShellEvent::Wheel(WheelInput { delta: Point { x: lines_x * WHEEL_LINE_PX, y: lines_y * WHEEL_LINE_PX }, position, modifiers: self.modifiers() })`（正の `lines_y` は下へ、正の `lines_x` は右へ進む。位置と modifiers の埋め方は `crates/kivact-test/src/screen/input.rs:72` の `send` と同じ）を `platform` に通す。
     - `Modifiers(m)` → `self.modifiers_changed(m)`。
     - `Raw(RawInput::Key(key))` → `self.keys.key(key, self.modifiers(), self.mouse_position())`。`Raw(RawInput::Ime(ime_input))` → 引数の `ime.input(ime_input)`。`Raw(RawInput::Drop(drop))` → `let mut events = self.dnd.input(drop); events.extend(self.dnd.take_dropped());`（`crates/arvane-virtual/src/script.rs:85-103` の `Script::raw` の分岐を移したもの）。それぞれの event を `platform` に通す。
     - `Accessibility { .. }` → 空の `Vec`（loop が window の a11y tree で検証する。設計書 3.3）。
   - click の数え方は `ClickCounter::count(&mut event, || now_ns, || double_click)`（`clicks.rs:87-91`）。`double_click` は native では `WinitWindow::double_click()`（今は `runtime.rs:1029` の `impl WinitWindow` の中。T4.1 で `window.rs` へ移る）、virtual では `kivact_types::defaults::DOUBLE_CLICK`。
   - 二重に数えない: `mouse`・`normalize`・`focus_lost` が返した event はもう `platform` を通っている。呼び出し側（native の `window_event`、virtual の `Script`）はそれらをそのまま配り、それ以外の event（key、touch、file、IME、Redraw などの window の event）だけを `platform` に一度通す。
   この task では virtual と native の側はまだ変えない（T4.1、T5.1 が `WindowInput` を使うように変える）。
6. `lib.rs`: 新しい名前を再公開する（`EventTarget`、`ShellLoop`、`InjectedInput`、`WindowInput`）。`mod input;`・`mod window_input;` を足す。
7. `dev/xtask/policy/workspace.toml` の `[module_rules.arvane-shell.uses]` に `input = ["pipeline"]` と `window_input = ["events", "types", "ime", "pipeline", "mouse", "clicks", "contacts", "input"]` を足し、`window` の配列に `"input"` を足す。
8. 確認: `cargo check -p arvane-shell --all-targets`、`cargo clippy -p arvane-shell --all-targets -- -D warnings`。
### T3.2 kivact-runtime: 状態の修正と DevTools の口

担当ファイル: `crates/kivact-runtime/src/**`。

参照: 付録 SM-CORE-03、SM-CORE-17、設計書 4.4.3、4.9。`kivact_types::TimeScale` は T2.4 が作る（W2 で済んでいる）。

手順:

1. 付録の SM-CORE-03（write と stabilization の epoch）と SM-CORE-17（`TimelineCell.events` を `Notifier` にする）を適用する。
2. `reactive/identity.rs`: `impl OwnerId { pub fn raw(self) -> u64 }` と `impl NodeId { pub fn raw(self) -> u64 }`（どちらも `(u64::from(self.0.slot) << 32) | u64::from(self.0.generation)`）。protocol の id（`u64`）にするため。`pub(crate) fn key_raw(key: Key) -> u64` も同じ式で足す（object の id 用）。
3. `reactive/detail.rs`（新規の leaf。`//! Role: The detailed turn trace the DevTools reads (hidden from the public API).`。`reactive.rs` に `mod detail;` と、`pub use detail::{CauseEntry, CauseTag, DetailEdge, DetailRun, DetailWrite, StageSpan, TurnDetail};` を足し、`lib.rs` の再公開の並びにも同じ名前を足す。facade（`kivact`）は再公開しない）。設計書 4.9 の型を、そのとおりに書く。すべての `pub` の型と method に `#[doc(hidden)]` と `// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.` を付け、`&'static Location` の field に `// xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.` を付ける。`DetailWrite`・`DetailRun` は `cause: Option<u32>` と `changed: bool` の 2 つが数えられるので、`changed` の直前に `// xtask-allow(state-fields): passive record - one write or run for DevTools; both fields are independent facts of it.` を付ける。続けて、記録の持ち主（`Diag` の field `detail: DetailLog`）の型を書く:

   ```rust
   const CAP: usize = 2000;        // list ごとの上限
   const DONE: usize = 256;        // 取り出されていない終わった turn の上限

   /// Off, or the detail of the turn being recorded (S-3: nothing exists while off).
   #[derive(Default)]
   pub(crate) enum DetailLog {
       #[default]
       Off,
       Recording(Box<DetailBuilder>),
   }
   pub(crate) struct DetailBuilder {
       /// The turn being recorded (what happens between two turns belongs to the next one).
       current: TurnDetail,
       /// Finished turns waiting for `take`.
       done: VecDeque<TurnDetail>,
       // xtask-allow(state-fields): statistics - the index of the last cause recorded in this turn; `None` before the first or after one was dropped at the cap.
       cause: Option<u32>,
       /// The stage being timed and the turn-relative ns it started at.
       open_stage: Option<(Stage, u64)>,
   }
   ```

   `impl DetailLog` の method（本体は短いので 1 つずつ書く。どれも `Off` なら何もしない）:
   - `start(&mut self)`: `Off` なら `Recording(Box::new(DetailBuilder { current: TurnDetail::default(), done: VecDeque::new(), cause: None, open_stage: None }))` にする。`stop(&mut self)`: `Off` にする。`is_recording(&self) -> bool`。`take(&mut self) -> Vec<TurnDetail>`: `done` を空にして返す。
   - `begin(&mut self, turn: u64)`: `current.turn = turn`。
   - `cause(&mut self, tag: CauseTag, label: String)`: `current.causes.len() < CAP` なら push して `self.cause = Some(index)`、そうでなければ `self.cause = None` にして `dropped += 1`。
   - `write(&mut self, node: NodeId, ty, created_at, at, changed: bool)`、`run(&mut self, node, kind, at, changed)`、`edge(&mut self, source: NodeId, target: NodeId)`: それぞれ `current.<list>.len() < CAP` なら push（write と run には `cause: self.cause` を入れる）、そうでなければ `dropped += 1`。
   - `stage(&mut self, stage: Stage, base_ns: u64, elapsed_ns: u64)`: 開いている段（`open_stage`）があれば `StageSpan { stage: prev, start_ns: base_ns + prev_elapsed, duration_ns: elapsed_ns - prev_elapsed }` を `current.stages` に push（`CAP` まで）してから、`open_stage = Some((stage, elapsed_ns))`。`base_ns` はその turn の `rt.now_ns()`（loop の時計。turn の間は変わらない）。
   - `end_turn(&mut self, base_ns: u64, duration_ns: u64)`: 開いている段を `duration_ns` で閉じ、`current.duration_ns = duration_ns`、`current` を `done` の末尾へ（`DONE` を超えたら古いものを捨てる）、`current` を `TurnDetail { turn: previous + 1, ..Default::default() }` にして、`cause = None`、`open_stage = None`。
4. `reactive/data.rs`・`diag.rs`: `Diag` に `detail: DetailLog` と `next_seq: u64`（S-8 の通し番号）を足し、`errors`・`diagnostics` の ring の要素を `(u64, UiError)` にする。`record_error`・`record_diagnostic` は `next_seq += 1` した値を seq にして入れる（`total_errors` はそのまま）。`Runtime::errors()` は今の形（`UiError` の列。seq を外す）のまま、`Runtime::diagnostics()` は作らない。新しい `pub struct Reported { pub seq: u64, pub handled: bool, pub error: UiError }` と `Runtime::reported(&self) -> Vec<Reported>`（`errors` の要素は `handled: false`、`diagnostics` の要素は `handled: true`。2 つの ring を seq の昇順に混ぜる）。`Reported` は `seq` と `handled` が数えられるので、`handled` の `///` の行の上に `// xtask-allow(state-fields): passive record - one reported error for DevTools; seq and handled are independent facts of it.` を付ける。`Diag::begin_turn(events)` は `finish_setup_notes()` の後に、`current.turn` が `Turn::Number(n)` なら `self.detail.begin(n)`。`Diag::end_turn` は引数に `base_ns: u64`（`Runtime::end_turn` が `self.now_ns()` を渡す）を足し、`self.detail.end_turn(base_ns, duration_ns)` を呼ぶ。`Runtime::set_detailed_trace(&self, on: bool)`（`start`／`stop`）と `Runtime::take_turn_details(&self) -> Vec<TurnDetail>` を足す。
5. cause の種類（設計書 4.4.3）: `pub enum CauseTag { Input, Timer, Task, Mount, Devtools }`（`detail.rs`）。`reactive/host.rs` の `TurnGuard::cause(&self, cause: String)` を `cause_with(&self, tag: CauseTag, label: String)` に置き換える（別名を残さない）。`Runtime::record_cause(c)` は `record_cause_with(tag, label)` にして、`current.causes`（文字列。今どおり 64 件まで）に `label.clone()` を入れ、`self.detail.cause(tag, label)` も呼ぶ。`host.rs:116` の `self.cause("timer".into())` は `self.cause_with(CauseTag::Timer, "timer".into())`、`host.rs:123` は `CauseTag::Task`。core の呼び出し（`turn.cause(describe(event))`）は T7.2 が直す。
6. 記録の箇所（`detailed` のときだけ、つまり `self.diag.borrow().detail.is_recording()`。無効なときは今と同じ）:
   - `graph.rs` の `write_node`（`if mutate(&cell)` の中の `record_write` の呼び出し、390 行付近）: 同じ文の中で `diag.detail.write(NodeId(key), ty, created_at, at, true)` も呼ぶ。`mutate` が false だったとき（`else` を足す）は、詳細な記録のときだけ `(ty, created_at)` を graph から引いて `detail.write(NodeId(key), ty, created_at, at, false)`（値を変えなかった書き込みも記録する。edge と notify は今どおり無い）。
   - `graph.rs` の `recompute`（`record_run(label, at)` の呼び出し、542 行付近）: `record_run` は今のまま（run の前）。`let changed = { .. compute.run() .. };` の後に `self.diag.borrow_mut().detail.run(NodeId(key), label, at, changed);`。
   - `graph.rs` の `notify_subscribers`（398-440 行）: 先頭で `let mut diag = self.diag.borrow_mut();` を取り、`stack.extend(n.subs()…)`（書き込んだ source の直接の購読者）の直前に購読者ごとに `diag.detail.edge(NodeId(src), NodeId(*s))`、memo が clean から汚れて `stack.extend(m.subs…)` するところでも購読者ごとに `diag.detail.edge(NodeId(k), NodeId(*s))`。
   - `TurnGuard::stage`（`reactive/host.rs:127`）と `TurnGuard::begin`（`begin` の最後）: `self.rt.record_stage(stage, self.started.elapsed())`。`Runtime::record_stage(&self, stage: Stage, elapsed: Duration)` は `self.diag.borrow_mut().detail.stage(stage, self.now_ns(), u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX))`。
   - 64 の上限は `TurnRecord` のものだけが残る。`TurnDetail` の上限は 2000（`CAP`）で、超えた分は `dropped` に数える。
7. `Runtime::owner_info(&self, id: OwnerId) -> Option<OwnerInfo>`（`self.owners.borrow().get(id.0)` から `OwnerInfo { parent: o.parent.map(OwnerId), created_at: o.created_at }`。`// xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.` を `created_at` の前に付ける。`TraceWrite` と同じ扱い）。
8. `Runtime::live_snapshot(&self) -> LiveSnapshot`: `Arena<T>`（`reactive/arena.rs`）に `pub(crate) fn iter(&self) -> impl Iterator<Item = (Key, &T)>` を足し、owner（`LiveOwner { id: OwnerId(key), parent: o.parent.map(OwnerId), created_at: o.created_at }`）と object（`LiveObject { id: key_raw(key), owner: OwnerId(o.owner), name: o.name, created_at: o.created_at }`）を集める。`Object`（`data.rs:372`）に `name: &'static str` を足し、`insert_object`（`object.rs:28`）の引数に `name: &'static str`（`data` の次）を足す。呼び出し 9 か所: `owner_handle.rs:181`・`196`（`store`・`store_with` に渡された `name`）、`handles.rs:209`・`514`・`603`・`689` と `setup.rs:110` は `"Callback"`、`setup.rs:58` は `"Var"`、`selector.rs:108` は `"Selector"`。型は設計書 4.9。
9. `motion/state.rs`: `MotionState::animations(&self) -> Vec<AnimInfo>`（`Anim` trait に `fn info(&self) -> AnimInfo` を足し、`AnimCell<T>`（`animated.rs`）と `TimelineCell`（`timeline/api.rs`）、frame loop が実装する。型は設計書 4.9。`AnimInfo` は 4 つの数えられる field（`created_at`、`running`、`paused`、`decorative`）を持つので、`created_at`・`paused`・`decorative` の直前に `// xtask-allow(state-fields): passive record - a snapshot of one animation for DevTools; the fields are independent facts read once.` を付け、`created_at` には `OwnerInfo.created_at` と同じ `api-shape` の印も付ける）。`MotionState::set_time_scale(&self, rt: &Runtime, scale: f64)`: `MotionState` に `time_scale: Cell<kivact_types::TimeScale>`（初期値 `TimeScale { anchor_host_ns: 0, anchor_motion_ns: 0, scale: 1.0 }`）を持ち、`set_time_scale` は `self.time_scale.set(self.time_scale.get().rescaled(rt.now_ns(), scale))` の後 `rt.request_turn()`。`MotionState::time_scale(&self) -> f64`（`self.time_scale.get().scale`。DevTools の `list_animations` が読む）も足す。motion の時計 `fn motion_now(rt: &Runtime) -> u64`（`MotionState::of(rt).time_scale.get().motion_ns(rt.now_ns())`）を足し、motion の `rt.now_ns()` の読み 8 か所（`animated.rs:357, 433, 452, 482`、`frame_loop.rs:109`、`state.rs:148, 261`、`timeline/api.rs:527`）を `motion_now(rt)` に置き換える。timer と入力の時間には掛けない。
10. `task/state.rs`・`task/time.rs`: `Scheduler.timers` の値を `Waker` から `Timer { waker: Waker, created_at: &'static Location<'static>, owner: Option<OwnerId> }` に変える（`sleep`・`timeout` は `#[track_caller]` なので `Location::caller()` と、そのときの ambient owner の `id()` を渡す。`take_due` は `waker` を返す）。`TaskState::list(&self) -> Vec<TaskEntry>`（`TaskEntry { created_at, owner, deadline_ns }`。型は設計書 4.9。`created_at` に `api-shape` の印）。task・resource・action は object の名前（`"Task"`、`"Resource"`、`"Action"`）で `live_snapshot` から分かるので、ここでは timer だけでよい。
11. state-fields の印は `// xtask-allow(state-fields): <分類> - <その field が何の事実か>` の形（分類は付録 7.4 の一覧）で、field の `///` の行より**上**に書く。数えられない field（`Sent`・`ReportOnce`・ring・`Diag.detail` など）には印を付けない（付けると「unused marker」の error になる）。
12. 確認: `cargo check -p kivact-runtime --lib`、`cargo clippy -p kivact-runtime --lib -- -D warnings`（`--all-targets` は facade をビルドするので W9 まで使わない。0.5）、`cargo xtask check --no-write --only state-fields --crate kivact-runtime --report`（自分が作った・変えた struct の error が無いこと）。

### T3.3 kivact-layout: 箱の幾何

担当ファイル: `crates/kivact-layout/src/columns.rs`、`tree.rs`、`lib.rs`。

手順:

1. `columns.rs:11-22` の `NodeLayout` に `pub padding: Insets`、`pub border: Insets`、`pub margin: Insets` を足す（`kivact_types::Insets` は `#[derive(Clone, Copy, Debug, Default, PartialEq)]` なので `NodeLayout` の derive のまま使える。`kivact_types::geometry` にある型で、`kivact_types` のルートから使えるか `grep -n "Insets" crates/kivact-types/src/lib.rs` で確かめる）。
2. `tree.rs:476-510` の `NodeLayoutColumns::layout_at`（taffy の `Layout`（丸めた結果）から `NodeLayout` を作る箇所）で、taffy の `Layout` の `padding`・`border`・`margin`（`Rect<f32>`。`top/right/bottom/left`）を `f64::from(..)` で `Insets` に入れる。`content_box` の計算（`l.border`・`l.padding` を使っている）は変えない。
3. `NodeLayout` を組み立てている他の箇所（`Default` を含む）を直す。
4. 確認: `cargo check -p kivact-layout --all-targets`、`cargo clippy -p kivact-layout --all-targets -- -D warnings`。

---

## 6. W4（依存の段 2）

| task | 担当 | crate |
|---|---|---|
| T4.1 | 1 | `crates/arvane-native` |
| T4.2 | 2 | `crates/kivact-style` |
| T3.4 | 3 | `crates/kivact-devtools-client`、`crates/kivact-devtools-agent`（節は W3 の中にある） |

波の検査: `cargo check -p kivact-devtools-client -p kivact-devtools-agent -p kivact-devtools-mcp --all-targets`、`cargo check -p arvane-native --all-targets`、`cargo check -p arvane-native --features embed --all-targets`、`cargo check -p kivact-style --lib`、clippy（native は `--all-targets --features embed -- -D warnings`、style は `--lib -- -D warnings`）、Linux の compile（0.5。WSL があれば）。commit: `feat(native): open several windows; style: forced states and style description`。

### T4.1 arvane-native: 複数ウィンドウ

担当ファイル: `crates/arvane-native/src/port/winit/runtime.rs`、`port/winit/window.rs`・`open.rs`・`windows.rs`・`shell_loop.rs`（新規）、`port/winit.rs`（module の宣言）、`port/linux_settings.rs`（SM-CORE-16）、`native_input/state.rs`（`NativeInput` を `WinitWindow` と一緒に使う箇所の追従だけ。印は T15.4 が後で書く）、`a11y/native.rs`（手順 3 の 1 method だけ）、`display/placement.rs`、`display.rs`、`port/embed/*.rs`、`port/win32/*.rs`、`port/x11/*.rs`、`mirror.rs`、`tests/smoke.rs`。

参照: 設計書 3.4、3.10、3.2 の capability、3.1.1。付録 SM-CORE-05、SM-CORE-16（HOST-REWRITE。この task で一緒に直す）。今の構造: `runtime.rs` の `WinitWindow`（90-112）、`NativeBackend`（499-502）、`Run`（525-538）、`Runner`（553-573）、`resumed`（648-728）、`user_event`（745-786）、`window_event`（798-843）、`map_window_event`（846-924）、`Backend::run`（1046-1093）、`capabilities`（261-310）。

手順:

0. 最初に `runtime.rs` の `WinitWindow`（90-112 行）・`Surface`（78-86 行）・`type WinitSurface`（75 行）と、それらの `impl`（`ShellWindow for WinitWindow` と `impl WinitWindow`（1013-1045 行。`double_click` を含む）を含む）、その impl が使う helper の `cpu_surface`（486 行付近）と `ImeWindow`（1095-1111 行）を、新しい `port/winit/window.rs` へそのまま移す（設計書 3.4。module の参照の循環を避けるため。移した後で `window.rs` が `runtime.rs` の何かを使っていたら、それも移す）。`runtime.rs` はそれを `use` する。`ShellWindow for WinitWindow` の `perform` の `PlatformRequest::FocusWindow` の arm（T2.4 が `Unsupported` にした）を `self.focus().map(|()| PlatformResult::Done)` にし、`fn focus(&mut self) -> Result<(), ShellError> { self.window.focus_window(); Ok(()) }` を実装する（winit の `Window::focus_window()`）。
1. `windows.rs` に `WindowTable` と `NativeWindow`（設計書 3.4。`placement: bool` を含む）を作る。`NativeWindow.injected` は `VecDeque<(Vec<InjectedInput>, RequestId)>`。`WindowTable` の method: `new(placement: bool)`、`insert(id, winit_id, window)`、`get(id)`、`get_mut(id)`、`by_winit(winit_id) -> Option<WindowId>`、`remove(id) -> Option<NativeWindow>`（`ids` の entry も消す）、`ids() -> Vec<WindowId>`、`push_pending_show(id)`、`take_pending_show() -> Vec<WindowId>`、`placement() -> bool`、`next_injected() -> Option<(WindowId, Vec<InjectedInput>, RequestId)>`（id の昇順で最初の空でない `injected` の先頭を取り出す）。
2. `open.rs` に `create_window`（設計書 3.4）。今の `resumed` の 666-714 行の処理をここへ移し、`request` の title・size・min_size・mode・placement と、親の `WinitWindow`（`request.parent` の window。main は無し）を使う。位置: main（`request.parent` が `None`）は今の `initial_window_position` と `center_window`。それ以外は、`placement` が `Auto` か `WindowTable` の `placement()` が false なら位置を指定しない。`CenterOnParent`・`NextToParent` なら親の外側の矩形（`parent.window.outer_position()`・`outer_size()`、物理 pixel）と親がいる monitor の work area から `display/placement.rs` の新しい `place_relative`（設計書 3.4。引数はすべて物理 pixel。`size` は新しい window の外側の大きさ）で位置を求めて `with_position`。親の monitor の work area は、`display/placement.rs` の `target_monitor_area` の「monitor から work area を求める部分」（`let position = monitor.position();` から `Some((monitor, work))` まで）を `fn area_of(monitor: MonitorHandle) -> (MonitorHandle, WorkArea)` に取り出し、`pub(crate) fn work_area_of(monitor: &MonitorHandle) -> Rect` として公開して使う。mode は設計書 3.4 の Windows・X11 の扱い。Windows の HWND は `raw-window-handle` の `window_handle()` から `RawWindowHandle::Win32(handle)` の `handle.hwnd.get()` で取る（今の subclass の install と同じ取り方を写す）。`cfg` の分岐は、既存の `#[cfg(windows)]` が許されている同じ file・同じ形で書く（os-branch の許可は `port/` の下にある）。
3. `shell_loop.rs` に `NativeLoop`（設計書 3.4）を作り、`ShellLoop` を実装する: `window` → table の `WinitWindow` を `&mut dyn ShellWindow` で、`windows` → `ids()`、`open_window` → `create_window` して table に入れ `push_pending_show` する、`close_window` → table から外して drop（Windows の modal なら親の `EnableWindow(親, true)` を先に。積まれていた注入の batch は `NativeWindow` と一緒に捨てる。`Injected` は来ない）、`inject(id, inputs, request)` → `NativeWindow.injected` に `(inputs, request)` を積むだけ（loop を起こす bit は作らない。設計書 3.3）、`now_ns` → `started.elapsed()`、`waker` → `waker.clone()`、`capabilities` → main window の `capabilities()` に `multi_window: true` と `window_placement: table.placement()` を入れたもの。`a11y/native.rs` の `NativeAccessibility` に `pub(crate) fn request_action(&self, node: u64, request: kivact_types::A11yRequest) -> Option<ShellEvent> { self.shared.request_action(node, request) }` を足す（`PlatformTree::request_action`。注入した a11y の入力の検証に使う）。
4. `runtime.rs` の `Runner` の `run: Run` を設計書 3.4 の `phase: LoopPhase`（window の表は `Running(WindowTable)`・`Stopped { table, .. }` の payload）に変える。`mouse`・`modifiers`・`dnd`・`clicks`・`contacts`・`keys` の field を消し、各 window の `NativeWindow.input: WindowInput` を使う。今の `callback` を `dispatch(el, target: EventTarget, event: Option<ShellEvent>)` にする（`NativeLoop` を作り、`None` なら `handler.start(&mut loop, waker)`、`Some` なら `handler.event(target, event, &mut loop)` を呼び、結果の処理は今のまま。返った後で `table.take_pending_show()` の window を `set_visible(true)`）。新しい `callback(el, target, event)` は `dispatch` を呼び、返った直後に `drain_injected(el)` を呼ぶ: `while let Some((id, inputs, request)) = table.next_injected()` で、その window の各入力を取り出し（`InjectedInput::Accessibility { node, request: a }` は `native.window.accessibility.request_action(node, a)` が `Some(event)` のときだけその event を、それ以外は `native.input.normalize(input, now, native.window.double_click(), &mut native.window.input.ime.borrow_mut())`）、出た event を `dispatch(el, EventTarget::Window(id), Some(event))` で配り、batch の最後に `dispatch(el, EventTarget::Window(id), Some(ShellEvent::Injected { request }))`。途中で window が閉じたら（表に無い）その batch の残りは捨てる。`dispatch` の中で handler が `inject` したら、この `while` が続けて配る。
5. `resumed`: `LoopPhase::Starting` なら、Wayland の判定（`display/placement.rs:30-36` と同じ判定）をして `WindowTable::new(!wayland)` を作り、main の `WindowRequest`（`Runner` が `arvane_native::backend(main: WindowRequest)` で受け取ったもの。`NativeBackend.options: ShellOptions` を `main: WindowRequest` に変える。`NativeBackend::new` の寸法検査（正で有限）は `main.size` に対して今のまま）で `create_window` → table に入れる → `dispatch(el, EventTarget::Loop, None)`（= `handler.start`）→ main を見せる → `Running(table)`。失敗は今どおり `Unopened`。それ以外の処理（surface の作り直し、`Resumed` の配達）は全 window に対して行う（`Resumed` は `EventTarget::Loop` で 1 回、`SurfaceCreated` は window ごと）。`pub fn backend(main: WindowRequest)` にする。
6. `window_event`: `table.by_winit(id)` で `WindowId` を引く（無ければ return）。以降の処理（a11y の `process_event`、scale、`preceding_event`、touch、focus を失ったときの解放）はその `NativeWindow` について行い、`EventTarget::Window(id)` で配る。二重に数えないために配り方を次のとおりにする（設計書 3.3）:
   - `CursorMoved`（file の drag 中でないとき）は `input.mouse(PointerPhase::Move, Some(at), now, dc)`、`CursorLeft` は `input.mouse(PointerPhase::Leave, None, now, dc)`、`MouseInput` は `input.mouse(PointerPhase::Down(ButtonPress { button, clicks: 1 }) か Up(..), None, now, dc)`、`ModifiersChanged` は `input.modifiers_changed(m)`。`window_event` の本体がこの 4 つを自分で処理し、返った event（`Some` のとき）をそのまま配る。`map_window_event` からはこの 4 つの arm を消す。
   - `map_window_event` の残り（key、wheel、touch、file、focus、resize など）の結果は、`input.platform(event, now, dc)` に一度通して配る。wheel の位置は `input.last_mouse_position()`、Shift は `input.modifiers().shift`。touch は `input.contacts_mut().deliver(..)` の event を `platform` に通して配る。file の drag は `input.dnd_mut()`。key は `input.keys_mut().key(..)`。
   - `preceding_event` は `&mut Mouse` の代わりに `&mut WindowInput` を受け取り、`input.mouse_position() != Some(at)` のとき `input.mouse(PointerPhase::Move, Some(at), now, dc)` の event を返す（`mouse` の返す event は `platform` を通っているので、そのまま配る）。
   - `Focused(false)` は、`input.focus_lost()` の event（`platform` を通ってある）を先に配り、その後で `Focused(false)` を `platform` に通して配る。
   `now` は `native.window.now_ns()`、`dc` は `native.window.double_click()`。
7. `user_event`: 全 window について、今の 1 window の処理（file drag、reduced motion、環境の変化、a11y action）を行う。`Wake(reasons)` は `EventTarget::Loop` で 1 回。
8. `suspended`、`new_events`、`about_to_wait`（drop の配達）も全 window に広げる。`SaveState`・`Wake`・`Suspended`・`Resumed`・`MemoryWarning` は `EventTarget::Loop`（`EventTarget::of(&event, id)` を使うと、この 5 つが `Loop`、ほかが `Window(id)` になる）。
9. `Backend::run` の最後: `handler.close(&mut loop)` を 1 回呼ぶ（全 window がまだある状態で。`NativeLoop` を作って渡す）。
10. `capabilities`（261-310。`window.rs` へ移った `WinitWindow::capabilities`）: `multi_window: true`、`window_placement` は `ShellWindow::capabilities()` からは分からないので false のままにし、loop の値（3. の `NativeLoop::capabilities`）が正本になる。engine は loop の `capabilities()` を使う。
11. runtime.rs の code 行が 1000 を超えないこと。超えそうなら、window ごとの処理（6〜8 の本体）を `port/winit/window_events.rs`（新規の leaf）に移す。
12. embed（`port/embed`、`port/win32`、`port/x11`）: `port/embed/session.rs` に `EmbedLoop` を一つだけ作り（win32・x11 の両方が使う）、`ShellLoop` を実装する:
    ```rust
    /// The embedded editor's loop: one window, no more.
    pub(in crate::port) struct EmbedLoop<'a> {
        window: &'a mut dyn ShellWindow,
        waker: ShellWaker,
        /// The inject requests answered with `Injected` after the callback returns.
        injected: Vec<RequestId>,
    }
    ```
    `window(id)` は `(id == WindowId::MAIN).then(|| &mut *self.window)`、`windows()` は `vec![WindowId::MAIN]`、`open_window` は `Err(ShellError::unsupported("windows beyond the embedded editor"))`、`close_window` は何もしない（editor の window は host が閉じる）、`inject(id, inputs, request)` は inputs を捨てて `self.injected.push(request)` だけ（embed では DevTools の入力を注入しない。core がこの target の `inject_input` を `UNSUPPORTED` で断る。embed の port を `WindowInput` へ載せ替える作業はこの変更に含めない）、`now_ns` は `self.window.now_ns()`、`waker` は clone、`capabilities` は `Capabilities { multi_window: false, window_placement: false, ..self.window.capabilities() }`。
    `Session` は `waker: ShellWaker` を持つ（`Session::new(shared, waker)`、`pub(in crate::port) fn waker(&self) -> ShellWaker`）。win32 は `State::new`（`window.rs:325-351`）で、今の `State::waker`（374-388 行）の本体を `fn wake_handle(shared: &Arc<Shared>, native: &Arc<Native>) -> ShellWaker` に取り出して `Session::new(shared.clone(), wake_handle(&shared, &native))` とし、`State::waker()` は `self.session.waker()` を返す。x11 は `Window::new`（`port/x11/window.rs:67`）で `Session::new(shared.clone(), waker.clone())`（`waker` は次の行の `Bridge::new(scale, waker)` に渡す前に clone）。`Session::start(make, window)`（waker の引数をやめる。win32 `window.rs:396`、x11 `thread.rs:166` の呼び出しを合わせる）、`deliver(event, window)`、`close(window)` は `EmbedLoop` を作って `handler.start(&mut lp, waker)`・`handler.event(EventTarget::of(&event, WindowId::MAIN), event, &mut lp)`・`handler.close(&mut lp)` を呼び、handler から戻った後（`accept` の前）に `lp.injected` の request を順に `handler.event(EventTarget::Window(WindowId::MAIN), ShellEvent::Injected { request }, &mut lp)` で配る（その中でまた `inject` されたら、空になるまで続ける）。win32・x11 の `ShellWindow` 実装に `fn focus(&mut self) -> Result<(), ShellError> { Err(ShellError::unsupported("focus_window in an embedded editor")) }` を足し、`perform` の `PlatformRequest::FocusWindow` の arm を `self.focus().map(|()| PlatformResult::Done)` にする。
13. `mirror.rs`: 付録 SM-CORE-05 を適用する（今の構造のまま。mirror は test 用の別の event loop なので `ShellLoop` を実装しない）。
14. 付録 SM-CORE-16（Linux settings の起動 flag）を適用する（`port/linux_settings.rs`）。
15. `tests/smoke.rs`: 新しい `ShellHandler` の形に合わせ、1 回の run の中で 2 つ目の window も試す（winit の event loop は 1 process で一度しか作れないので、test 関数を増やさず今の run を広げる）。`start` で `shell.open_window(WindowRequest { id: WindowId::new(2).expect("invariant: 2 is not 0"), title: "smoke 2".into(), size: Size { width: 120.0, height: 90.0 }, min_size: None, mode: WindowMode::Tool, placement: WindowPlacement::CenterOnParent, parent: Some(WindowId::MAIN) })?` を呼び、`shell.windows()` の各 window に `request_redraw()`。`event` は `EventTarget::Window(id)` の `Redraw` で、その window に今の試験と同じ present をして、id ごとに `presented` を記録する。window 2 が present したら `shell.close_window(WindowId::new(2)…)` し、その直後の `shell.windows()` を `windows_after_close` に記録する。main が present 済みで window 2 が閉じ済みになったら `Exit`。`main()` の最後に `assert_eq!(smoke.windows_after_close, [WindowId::MAIN])` と両方の window が present したことを確かめる。main の `backend` は `arvane_native::backend(WindowRequest::main("Arvane native smoke", Size { width: 160.0, height: 120.0 }))`。
16. 確認: `cargo check -p arvane-native --all-targets`、`cargo check -p arvane-native --features embed --all-targets`、`cargo clippy -p arvane-native --all-targets --features embed -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate arvane-native --report`（`Runner` の `phase` と `wake_reasons`（今の field。loop の起こし方の coalescing の事実）が並ぶ error が出たら、`wake_reasons` の `///` の行の上（field の一番上）に `// xtask-allow(state-fields): mailbox - coalesced wake reasons the loop drains` を書く）。
### T4.2 kivact-style: forced states、style の記述、transform の段

担当ファイル: `crates/kivact-style/src/resolve.rs`、`internal.rs`、`transform.rs`、`theme/value.rs`（`install_theme` は変えない。ここでは変えるものは無い。token override は core が `install_theme` に渡す値を包む）。

参照: 設計書 4.4.5、付録 SM-INT-17。今の `Interactive`（`resolve.rs:212-226`）、`compile_style`（257）、`classify_properties!`（397-527）。

手順:

1. `resolve.rs`: forced states の型を書く（形の手本は `crates/arvane-shell/src/types.rs` の `WakeReasons`。`Interactive` が `Clone, Copy, Debug, Default, PartialEq, Eq` を derive しているので、同じ derive が要る）:

   ```rust
   /// Interactive states a tool forces on a node (DevTools), as a set.
   #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
   pub struct ForcedStates(u8);
   impl ForcedStates {
       /// Hovered.
       pub const HOVER: Self = Self(1);
       /// Pressed.
       pub const PRESSED: Self = Self(2);
       /// Focused.
       pub const FOCUS: Self = Self(4);
       /// Focused with a visible focus ring.
       pub const FOCUS_VISIBLE: Self = Self(8);
       /// Disabled.
       pub const DISABLED: Self = Self(16);
       /// Selected.
       pub const SELECTED: Self = Self(32);
       /// Whether every state of `s` is in the set.
       pub fn contains(self, s: Self) -> bool { self.0 & s.0 == s.0 }
       /// Add the states of `s`.
       pub fn insert(&mut self, s: Self) { self.0 |= s.0; }
       /// Whether no state is forced.
       pub fn is_empty(self) -> bool { self.0 == 0 }
   }
   ```

   `Interactive` に `pub forced: ForcedStates`（doc: `/// States a tool forces on the node; empty for an App.`）を足す。`Interactive` を組み立てている全箇所に `forced: ForcedStates::default()` を足す（core の箇所は W7 と W11 で直るので、ここでは kivact-style の中だけ）。
2. `compile_style`（引数 `state: Interactive` を `mut state` にする）:
   - 最初に `if state.forced.contains(ForcedStates::DISABLED) { state.disabled = true; }`、`SELECTED` も同じ（`state.selected = true`）。
   - `if state.disabled && let Some(v) = s.disabled.take() { s = s.merge(&v); }` の後、`let states = ..` の前に、forced の状態の差分を base に merge して `StateStyles` から外す。merge の順は engine の重ね順 focus < focus_visible < hover < pressed と同じ:

     ```rust
     if state.forced.contains(ForcedStates::FOCUS) && let Some(v) = s.focus.take() { s = s.merge(&v); }
     if state.forced.contains(ForcedStates::FOCUS_VISIBLE) && let Some(v) = s.focus_visible.take() { s = s.merge(&v); }
     if state.forced.contains(ForcedStates::HOVER) && let Some(v) = s.hover.take() { s = s.merge(&v); }
     if state.forced.contains(ForcedStates::PRESSED) && let Some(v) = s.pressed.take() { s = s.merge(&v); }
     ```

     （この 4 行は 1 つの関数 `fn merge_forced(s: Style, forced: ForcedStates) -> Style` に出し、`compile_style` からは 1 回呼ぶ。`compile_style` を 80 行に収めるため。）
3. `describe_style`: `classify_properties!` の展開（`@ignored duplicates` の前の macro 本体）に、各 group の各 field の値を返す関数を足す。`fn describe_properties(s: &Style) -> Vec<(&'static str, String)>` は、各 group の `($set, $name, $field)` について `$set` が真なら `($name, format!("{:?}", $field))` を返す（`$set` と `$field` は同じ macro の引数なので、macro の規則を `$field => $set:expr, $name:literal;` のまま、`describe_properties` の展開で `let Style { $($field,)* .. } = s;` を使って field を束縛する）。`state_properties` の group は含めない（hover、group、group_hover、group_focus、pressed、focus、focus_visible、disabled、selected、enter、exit、transition、layout_transition）。
   - `pub fn describe_style(s: &Style) -> Vec<(&'static str, String)>`: `describe_properties(s)` の結果のうち、次の名前だけを辺ごとの名前に分ける。分ける処理は `fn split_sides(name: &'static str, value: String) -> Vec<(&'static str, String)>` に置く（どの関数も 80 行以下）: `padding`・`margin`・`inset` → `padding_top`・`padding_right`・`padding_bottom`・`padding_left`（`Sides` の `Some` の辺だけ）、`border_width`・`border_color`・`border_style` → `border_top_width` のように `border_<辺>_width/_color/_style`（配列は top, right, bottom, left の順で `Some` の要素だけ）、`radius` → `radius_top_left/_top_right/_bottom_right/_bottom_left`（配列の順）、`translate` → `translate_x/_y`。辺ごとの名前は `&'static str` の定数の表（`const PADDING: [&str; 4] = ["padding_top", ..]` など）から引く（`format!` で作らない）。値は各辺・各隅の要素の `Debug`。
   - `pub fn describe_state_styles(s: &Style) -> Vec<(&'static str, Vec<(&'static str, String)>)>`: `"hover"`・`"pressed"`・`"focus"`・`"focus_visible"`・`"disabled"`・`"selected"` ごとに、その state の `Style`（`s.hover` など。`Some` のものだけ）を `describe_style` したもの（core が `State`・`Disabled`・`Selected` の層にする）。
   - `internal.rs` から `describe_style`、`describe_state_styles`、`ForcedStates` を再公開する。
4. `transform.rs`: 付録 SM-INT-17（`Transform.stage` を private enum `Stage` にする）を適用する（`stage: u8` と `STAGE_*` の定数を、順序を持つ enum `Stage { None, RotateZ, RotateY, RotateX, Skew, Scale }`（`#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]`）にする。`in_order(&self, stage: Stage) -> bool { stage >= self.stage }`、`self.stage = STAGE_X` は `self.stage = Stage::X`、初期値は `Stage::None`）。
5. 確認: `cargo check -p kivact-style --lib`、`cargo clippy -p kivact-style --lib -- -D warnings`（0.5）、`cargo xtask check --no-write --only state-fields --crate kivact-style --report`。

---

## 7. W5（依存の段 3）

2 つの小さな波に分ける（arvane の試験は arvane-virtual を dev-dependency に持ち、T5.2 は T5.1 が作る `Script::window`・`Probe::window` を使う）。

| 波 | task | 担当 | crate |
|---|---|---|---|
| W5a | T5.1 | 1 | `crates/arvane-virtual` |
| W5b | T5.2 | 1 | `crates/arvane` |

W5a の検査: `cargo check -p arvane-virtual --all-targets`、`cargo check -p arvane-virtual --features mirror --all-targets`、clippy。commit: `feat(virtual): several windows per loop`。W5b の検査: `cargo check -p arvane --all-targets`、clippy。commit: `feat(arvane): a presenter per window`。

### T5.1 arvane-virtual: 複数ウィンドウ

担当ファイル: `crates/arvane-virtual/src/**`。

参照: 設計書 3.5、3.3。付録 SM-CORE-06（HOST-REWRITE。ここで直す）。今の構造: `backend.rs:20-448`、`window.rs:19-392`、`script.rs:14-211`、`clock.rs`、`surface.rs`、`mirror.rs`（60 行。`open_mirror` は `mirror.rs` の `impl VirtualOptions`（feature `mirror` のときだけ）で、mirror の adapter も `mirror.rs`）。

手順:

1. `window.rs`: `VirtualShell` を `VirtualPlatform`（loop の状態）と `VirtualWindow`（window の状態）に分ける（設計書 3.5 の field の振り分け）。`VirtualWindow` は `platform: Rc<VirtualPlatform>` を持ち、`ShellWindow` を実装する（`perform` の clipboard・dialog などは platform へ、`RequestWindowSize`・`Fullscreen` は自分へ。`PlatformRequest::FocusWindow` は `self.focus().map(|()| PlatformResult::Done)`）。`VirtualWindow` に `focus_requests: Cell<u32>` を足し、`fn focus(&mut self) -> Result<(), ShellError>` は `focus_requests` を 1 増やして `Ok(())`（`WindowProbe::focus_requests()` で読む）。`VirtualOptions` は main window の設定のまま、ただし `title`・`size` の field を `main: WindowRequest` に置き換える（既定は `WindowRequest::main(String::new(), Size { width: 800.0, height: 600.0 })`。`backend.rs` の `options.title`・`options.size` の読みは `options.main.title`・`options.main.size`、`mirror.rs` も同じ）。`capabilities` の既定は `multi_window: true, window_placement: true`。付録 SM-CORE-06（frame と mirror の field）をこの分割の中で適用する。
2. `script.rs`: `Input` を `VirtualInput { queue, clock, saved_state, windows: BTreeMap<WindowId, VirtualWindowInput> }` にする。`struct VirtualWindowInput { input: WindowInput, ime: ImeDriver }`（arvane-shell の `WindowInput` と、virtual の window の IME の driver。設計書 3.3）。`Item` と `WindowItem` は設計書 3.5 のとおり（`Item::Window(WindowId, WindowItem)` と loop の項目）。`Script` の window ごとの method を `WindowScript { script: Script, window: WindowId }` に移し、`Script::window(&self, id) -> WindowScript` を足す。`WindowScript` に置くもの: `input(event)`、`raw`、`mouse`、`contact`、`focus`、`accessibility_action`、`resize`、`set_scale`、`set_frame_period`、`set_reduced_motion`、`set_dark`、`set_locale`、`set_safe_area`、`set_keyboard_inset`、`create_surface`、`destroy_surface`、`set_occluded`、`fail_presents`。`Script` に残すのは loop の method（`saved_state`、`save_state`（`Item::LoopEvent(ShellEvent::SaveState)`）、`respond`、`set_now`、`set_clipboard`、`push_file_dialog`、`push_files_dialog`、`fail_next_effect`、`fail_next_effect_unsupported`、`pointer_settings`、`is_empty`）と、今の振る舞いを保つ `input(event)`: `EventTarget::of(&event, WindowId::MAIN)` が `Loop` なら `Item::LoopEvent(event)`、そうでなければ `Script::window(WindowId::MAIN).input(event)` と同じ。正規化は `WindowInput::normalize`／`mouse`／`platform`／`focus_lost` を使う（今の `raw`・`enqueue` の中身は T3.1 で `WindowInput` に移った）: `WindowScript::mouse(phase, position)` は `input.mouse(phase, position, clock.now(), kivact_types::defaults::DOUBLE_CLICK)` の event を積む、`WindowScript::input(event)` は `platform` を通した event を積む（`Focused(false)` のときは先に `focus_lost()` の event を積む）、`WindowScript::focus(false)` は `focus_lost()` の event の後に `Focused(false)`。`VirtualInput.windows` は `Running` の window の表と同じ window の集合で、`open_window`・`close_window` が一緒に足し・外す。閉じた window の `Item::Window` は step のときに捨てる。
3. `backend.rs`: `VirtualBackend::new` は `platform: Rc<VirtualPlatform>` と空の `windows: Rc<RefCell<BTreeMap<WindowId, VirtualWindow>>>` を作り、`Probe` はこの 2 つの `Rc` を持つ（main の `VirtualWindow` は `new` では作らない）。`SteppedBackend::start` で mirror を開き（`open_mirror(&options.main)`）、表示先（`Display::Mirrored` か `Display::Headless`）を決めて main の `VirtualWindow` を作り、表と `VirtualInput.windows` に入れる（`display` を後から書き換えない）。`Running` は同じ `Rc` を持つ。`ShellLoop` を実装する `VirtualLoop<'a>`（`windows`・`platform`・`input` を借りる）を作り、`start`・`step` で `handler.start(&mut loop, ..)`／`handler.event(target, event, &mut loop)` を呼ぶ。`open_window` は `VirtualWindow` を作り（mirror の mode が Off でなければ `open_mirror(&request)` で mirror も開く）、入力の状態（`VirtualWindowInput`）を作り、`Item::Window(id, WindowItem::Event(ShellEvent::AccessibilityActivated))` を queue の末尾に積む（callback の中から handler は呼べない）。`close_window` は mirror を閉じて取り除き、その window に積まれた注入の項目を捨てる。`inject(id, inputs, request)` は各入力を `InjectedInput::Accessibility { node, request: a }` なら `window.accessibility().request_action(node, a)` が `Some(event)` のときだけその event を、それ以外は `WindowInput::normalize(.., kivact_types::defaults::DOUBLE_CLICK, &mut ime)` の event を、`Item::Window(id, WindowItem::Event(..))` で queue に積み、最後に `Item::Window(id, WindowItem::Event(ShellEvent::Injected { request }))` を積む。`ShellLoop::waker` は `start` で作った waker の clone を返す。`ShellLoop::capabilities()` は main の `VirtualWindow` の `ShellWindow::capabilities()`（`multi_window`・`window_placement` は `VirtualOptions.capabilities` の値）。`Step::Redraw` は redraw の flag が立った全 window に id の昇順で配る。`StepOutcome::Idle.redraw_requested` はどれかの window の flag。
4. `Probe`（backend.rs:362-448）: 今の window ごとの読み取りを `WindowProbe` に移し（`has_window()` は今の `Probe::has_window` と同じ mirror の有無、`focus_requests()` を足す）、`Probe::window(&self, id) -> Option<WindowProbe>`（start の前と閉じた window は `None`）、`Probe::windows(&self) -> Vec<WindowId>` を足す。loop の読み取りは `Probe` に残す（設計書 3.5）。
5. mirror: window ごとに `open_mirror` を呼べるようにする（`mirror.rs` の `VirtualOptions::open_mirror(&self, request: &WindowRequest)` にし、開く window の title と大きさを `request` から読む。`physical(request.size, self.scale)`）。
6. state-fields: `VirtualPlatform`・`VirtualWindow` の field のうち `cargo xtask check --no-write --only state-fields --crate arvane-virtual --report` が error にするものには、field の `///` の行の上に次の印を書く（基準 commit の `VirtualShell` の理由を移したもの）: `redraw`・`ime`・`ime_text`・`soft_keyboard`・`gpu`・`occluded`・`min_size`・`focus_requests` は `// xtask-allow(state-fields): independent fact - a scripted backend fact the test controls (a test input, not a lifecycle)`、`fail_next` は `// xtask-allow(state-fields): mailbox - the service failure the test fills and the window drains`。
7. 確認: `cargo check -p arvane-virtual --all-targets`、`cargo check -p arvane-virtual --features mirror --all-targets`、`cargo clippy -p arvane-virtual --all-targets --features mirror -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate arvane-virtual --report`。

### T5.2 arvane: presenter を window ごとに

担当ファイル: `crates/arvane/src/window.rs`、`embed.rs`、`presentation.rs`、`spi.rs`、`lib.rs`、`presenter/*.rs`（必要な箇所だけ）、`tests/e2e/support.rs` と `tests/e2e/*.rs` の利用箇所（`embed.rs`・`environment.rs`・`external.rs` を含む）。

参照: 設計書 3.6。今の構造: `window.rs:21-350`（`Window`、`Handler`、`Shell`、`ShellSetup`、`SteppedShell`、`driver`、`Driver`）。

手順:

1. `window.rs`: `Handler` と `Windows` を設計書 3.6 の形にする。`Driver<H>` は `presenters: BTreeMap<WindowId, Presenter>`、`gpu: bool`、`env`（`Env::read` の結果）、`observer: Option<Observer>`（`Observer = Rc<RefCell<dyn FnMut(WindowId, &RendererChange)>>`。各 presenter の `Notes` に clone を渡す）。waker は保存しない（`ShellLoop::waker()` から得る）。`ShellHandler for Driver`: `start` → main の presenter を作り（GPU なら `start_gpu`）、`Windows` を作って `handler.start`。`event` → `Wake(GPU_STEP)` なら全 presenter に今の処理、`SurfaceDestroyed` は対象の window の presenter だけ、その後 `handler.event(target, event, &mut windows)`。`close` → `handler.close(&mut windows)` の後、全 presenter を `shutdown`。`Windows::capabilities()` は `Capabilities { gpu: arvane_compose::GPU_BUILT, ..self.shell.capabilities() }`（今の `Window::capabilities` と同じ GPU の上書き）。`Windows::inject(id, inputs, request)` は `ShellLoop::inject` へ渡す。`Driver` の `observer` の `///` の行の上に `// xtask-allow(state-fields): construction option - the renderer observer from ShellSetup, cloned into every presenter's Notes` を付ける（`gpu` と並んで数えられるため）。`Windows` に `gpu` と `observer` を持たせるときは、`Windows.observer` にも同じ印を付ける。
2. `Windows::open`: presenter を作って（GPU なら `start_gpu(shell.waker())`）map に入れ、`shell.open_window(request)`。失敗したら presenter を外して error を返す。`Windows::close`: presenter を `shutdown(window)`（`ShellLoop::window(id)` で window を得て）→ map から外す → `shell.close_window(id)`。
3. `presentation.rs`: `Observer` を `Rc<RefCell<dyn FnMut(WindowId, &RendererChange)>>` に変え（`Notes(Option<Observer>)` はそのまま。clone を持つ）、`Notes` が自分の window の id を持って呼ぶ（`Notes::new(observer, window)`）。`Shell::on_renderer_change` の引数は `impl FnMut(WindowId, &RendererChange) + 'static`。
4. `embed.rs`: `driver(options, make)` を新しい `Handler` に合わせる（main の 1 window）。`lib.rs` の再公開から `ShellOptions` を外し、`EventTarget`・`InjectedInput` を足し（engine の `Handler` の実装が使う）、`pub use window::{Handler, Shell, ShellSetup, SteppedShell, Window, Windows};` に `Windows` を足す。`spi.rs` の再公開に `ShellLoop` を足す（backend の契約）。
5. `tests/e2e/support.rs` の `Harness` と e2e の `Host`（334 行付近）を新しい `Handler` と `Script::window(WindowId::MAIN)`・`Probe::window(WindowId::MAIN)` に合わせる。`options()` は `title`・`size` の代わりに `main: WindowRequest::main(<title>, Size { width, height })`（`environment.rs`・`external.rs` の `..options(..)` はそのまま動く）。observer は `Some(Rc::new(RefCell::new(move |_window, change| observed.borrow_mut().changes.push(change.clone()))))`。`tests/e2e/embed.rs` の `Handler` は新しい形にし、`start` で `windows.get(WindowId::MAIN)` の window に `perform(PlatformRequest::FocusWindow)` が `PlatformResult::Unsupported` であることも確かめる（embed は前面化できない）。試験の内容は変えない。
6. 確認: `cargo check -p arvane --all-targets`、`cargo clippy -p arvane --all-targets -- -D warnings`。

---

## 8. W6（依存の段 4）

| task | 担当 | crate |
|---|---|---|
| T6.1 | 1 | `crates/kivact-engine` |

engine の変更は互いに絡むので 1 体で行う。波の検査: `cargo check -p kivact-engine --all-targets`、`cargo check -p kivact-engine --features testing --all-targets`、`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings`。commit: `feat(engine): one model per window`。

### T6.1 kivact-engine: window ごとの model

担当ファイル: `crates/kivact-engine/src/**`、`crates/kivact-engine/tests/host_locale.rs`。

参照: 設計書 3.7、付録 SM-CORE-13、SM-CORE-15 の engine の部分。今の構造は設計書 3.7 の下の「今の行番号」: `host/pump.rs:49-92`（`Host`）、`host/launch.rs:19-166`、`host/events.rs:54-443`、`host/client.rs`、`host/write/writer.rs`・`reader.rs`・`typed.rs`、`model/aggregate.rs:20-168`、`model/events.rs`、`contract/events.rs:20-246`、`contract/host.rs:9-107`、`contract/routed.rs:7-26`、`embed.rs:53-79`、`host/testing/test_host.rs`。

手順（この順に。各段の後で `cargo check -p kivact-engine --features testing` を通してから次へ進む。途中は失敗してよいが、最後の段の後は通ること）:

1. event の列の共有: `model/events.rs` に `#[derive(Clone, Default)] pub(crate) struct EventSink(Rc<RefCell<EventQueue>>)` と `pub(crate) struct Shared { pub(crate) events: EventSink, pub(crate) assets: Rc<RefCell<ArvaneAssets>> }` を足す（どちらも `model/events.rs`。`model/aggregate.rs` と `model/app_model.rs` の両方が使うので、ここに置くと module の参照が循環しない）。method は `EventQueue` の今の method（`push`、`has_events`、`applied`、`commit`、`drain`）を `&self` で包むもの（中で一瞬だけ `borrow_mut`）。`Model.events`（`aggregate.rs:27`）の型を `EventSink` に変える。`self.events.push(..)`（`aggregate.rs:134`、`presence/transitions.rs:31`）、`has_events`（139）、`applied`（144）、`commit`（`apply.rs:60`）はそのまま動く。`host/pump.rs:349` の `model.events.drain()` は `app.events().drain()` に変わる（手順 4）。
2. asset の共有: `Model.assets` の型を `Rc<RefCell<ArvaneAssets>>` に変える（`.assets.borrow()` の 24 か所はそのまま動く）。
3. `Model` の constructor: `Model::new(viewport: Size, store: AssetStore)`（`aggregate.rs:72`）を `Model::new(window: WindowId, viewport: Size, shared: &Shared)` に変える（`Shared` は手順 1 で `model/events.rs` に置いた）。`Model` に `pub(crate) window: WindowId` を足す（自分の window。HostEvent に window を入れるときに使う）。`Model.writes`（`aggregate.rs:29`）を消す。`model/edit/inputs.rs:71` の `self.writes.created(id)` は、その method に `created: bool` の引数を足し、呼び出し側（`host/write` の中）が `app.writes.created(id)` を渡すように変える。`Model.frame.motion`（`model/frame/state.rs:52`）と `Model::motion_clock()`（`ticks.rs:259`）を消し、`Model::update_env`（`ticks.rs` の `(Clock::Stopped { since }, true)` の arm）の `self.frame.motion.offset_ns = …` と `self.frame.motion.resumed_ns = now;` の 2 文を消す（App の motion の時計は `AppModel` が持つ。手順 13b）。window の `frame.clock` と `shift_state_times` などの時刻ずらしはそのまま。
4. `model/app_model.rs` に `AppModel`、`WindowChange`、`AppClock`（設計書 3.7）と、次の method を作る: `new(shared: Shared, main: Model)`、`model(&self, id) -> Option<&Model>`、`model_mut(&mut self, id) -> Option<&mut Model>`、`window_of(&self, node) -> Option<WindowId>`（設計書 3.7 の式。node の持ち主の `Model` から導く。別の表を持たない）、`with_node<R>(&mut self, node, f: impl FnOnce(&mut Model, &mut Writes) -> R) -> Option<R>`（`window_of` で引き、`windows` と `writes` を同時に借りる）、`open_window(&mut self, request: WindowRequest, viewport: Size)`（`Model::new` して入れ、`WindowChange::Open` を積む）、`close_window(&mut self, id)`（`WindowChange::Close` を積む）、`take_window_changes(&mut self) -> Vec<WindowChange>`、`remove_window(&mut self, id)`（`Model` を外し、`focused_window` が `id` なら `None`）、`windows(&self) -> Vec<WindowId>`、`events(&self) -> &EventSink`、`writes`／`writes_mut`、`motion(&self) -> MotionClock`、`focused_window(&self) -> Option<WindowId>`。`model/frame/state.rs` の `FrameRequests` は `pub(crate) struct FrameRequests(pub(crate) BTreeSet<u64>)` にし、`AppModel` の field はすべて `pub(crate)` にする（`host/write` の `request_frame` が `app.client_frames.0.insert(..)`、`pump` が `app.windows` と `app.client_frames` を同時に借り、`events.rs`・`window_changes.rs` が `focused_window`・`motion`・`clock` を直接書く。`impl AppModel` は `model/` の下にしか置けない）。
5. `Writer`・`Reader`（`host/write/`）: `model: Rc<RefCell<Model>>` を `app: Rc<RefCell<AppModel>>` に変える。node を受け取る method は `app.with_node(id, |model, writes| …)` の中で今の処理をする。`typed.rs` の `props!` が生成する `Writer::record`（411-418 行）も同じ。`Writer::create` に `window: WindowId` を足し、その window の `Model` に作る。`insert` は親と子の `window_of` が同じことを `expect("invariant: a node and its parent live in the same window")` で確かめる。
6. `Writer` の window 向けの method（設計書 3.7）: `open_window(request)`（viewport は `request.size`）、`close_window(id)`、`set_window(window, title, min_size)`（その `Model.window_request` に入れる）、`blur(window)`。`effect(request_id, effect)` は形を変えず、`HostRequest::Platform { window, .. }` の `window` の `Model.effects` へ入れる。node や window を持たない request は設計書 3.7 の行き先のとおり: `Snapshot(node)` は `window_of(node)` の `Model`、`RegisterFont` は共有の asset に 1 回登録して全 window の `Model` の `env.fonts` を更新、`DrawableExport`・`SetDrawableGeometry`・`drawable_request`・`import_external`・`drop_external` はその drawable／image を使う node の window の `Model`、`ReducedMotion` は `Host.reduced_motion_override` に 1 回入れて全 window の env に掛ける、`forget_adopted_pointer`・`reserve_touch_gesture` は全 window の `Model`（`Inject` は T10.1 が足す）。`focus(target)` は node から window を引く。`request_frame(at_ns)` は `app.client_frames.0.insert(at_ns)`（`AppModel.client_frames`。`Model.frame.frames` には入れない。手順 13）。
   `commit()`: 今の `Model::commit_revision`（`model/apply.rs:51-63`）を `Model::prepare_commit(&mut self)`（pack child orders、start entrances、focus の検証と confine、hover の検証、移った focus の reveal、drawable の reconcile。境界は積まない）に変え、`Writer::commit` は (1) 全 window の `prepare_commit`、(2) `let revision = Revision(app.events().applied().0 + 1); app.events().commit(revision);` を 1 回、(3) `app.writes.commit(revision)`、(4) 全 window の `Model::sync()`、の順に行う。
7. `Reader`: `windows()`、`window_of(id)`、`root(window)`、`active_modal(window)`、`dismiss_target(window)`、`focused_window()` を足し、今の `active_modal()`・`dismiss_target()` を消す。`in_modal_scope(id)`・`hit_test(id, point)`・`can_take_focus(target)`・`roving_targets(root)` は node から window を引く。`admit_input(event)` は event の window（下の 9.）の model に問う。
8. `HostEvent`・`HostRequest`・`Routed`（`contract/`）: 設計書 3.7 のとおり field と variant を足す（`InjectDelivered { window, request: RequestId }`、`AppMotion { drawable, frame_period_ns }` を含む）。engine の中でこれらを作る箇所に、その `Model.window`（host が作るものは対象の window の id）を入れる（`grep -rn "HostEvent::FocusChanged\|HostEvent::HoverChanged\|HostEvent::MousePosition\|HostEvent::FrameTick\|HostEvent::EnvChanged\|HostEvent::CloseRequested\|HostEvent::HostFailure\|HostEvent::GpuAdapterChanged\|HostEvent::ModifiersChanged\|Routed {" crates/kivact-engine/src` で全部探す）。
9. `host/window.rs` に `HostWindow`（設計書 3.7 の field）を作り、`Host` の window ごとの field を移す（`pump.rs:59`、`69`、`75`、`78`、`79`、`87`、`91`）。`LastPresent`（`pump.rs:40-47`）、`Surface`・`Unavailable`（`pump.rs:100-108`）、`SoftKeyboard`・`ImeAllowed`（`pump.rs:589-597`）の定義も `host/window.rs` へ移す（`pump.rs` がそれを使う。逆向きの参照を作らない）。`Host` に `windows: BTreeMap<WindowId, HostWindow>` と `app: Rc<RefCell<AppModel>>`（今の `model` を置き換える）。`Host::drawable()` は `Host::drawable(&self, id) -> bool` に変える。`HostWindow.parent` の `///` の行の上に `// xtask-allow(state-fields): independent fact - the parent window fixed when the window opened; modal input routing reads it`、`HostWindow.last_stats` の `///` の行の上に `// xtask-allow(state-fields): delivery record - last presented frame's report; not used for control` を書く（印は常に field の `///` の行の上）。
10. `Host` の入口（`host/events.rs`）: `begin(&mut self, windows: &mut arvane::Windows<'_>, waker)`、`handle(&mut self, target: EventTarget, event, windows: &mut arvane::Windows<'_>)`。`handle` は `EventTarget::Loop` の event（SaveState、Wake、Suspended、Resumed、MemoryWarning）を今の処理で 1 回、`EventTarget::Window(id)` の event は、その window の `HostWindow`・`Model`・`windows.get(id)` を使って今の処理をする（`self.windows` に無い `id` の event は捨てる）。今の処理の中で `window: &mut arvane::Window` を使っていた箇所は、`windows.get(id)` で得た `Window` を使う。入口の順は設計書 3.7 のとおり (1) DevTools の起動キー、(2) pick、(3) modal の親への入力の破棄、(4) 今の処理。この task が書くのは (3) と (4) で、(1)(2) は T10.1 が (3) の前に差し込む。`ShellEvent::CloseRequested` は `HostEvent::CloseRequested { window: id }`。`Focused(true)` は `app.focused_window = Some(id)`、`Focused(false)` は `focused_window` が `id` なら `None` にし、その window の `Model` の `key_down_targets` を消し pointer を取り消す（`events.rs:254-258` を window ごとに）。
11. modal の入力 (3): `handle` の入口で、`target` の window に「`mode == Modal` で `parent == Some(id)` の `HostWindow`」があれば、Pointer・Wheel・Key・Ime・File・Accessibility（`ShellEvent::Accessibility`。DevTools の `InjectedInput::Accessibility` が配る event もここを通る）の event を捨てて `defer` で返す（設計書 3.7）。
12. `pump`（`pump.rs:312-393`）: loop（323-363 行）の各回で、全 window の `poll_assets` と `settle` を id の昇順に呼ぶ。event があるかは共有の `EventSink` で見る。`snapshot_requests` は全 window について。`TurnsCut` は main の `Model` の event として 1 回積む。client の turn（`turn`、279-306 行）の `set_surface`・`frame.set_now` は全 window に行う。関数が 80 行を超えないよう、次の 2 つを新しい leaf `host/window_changes.rs` に `impl Host` で置く（`events.rs` と同じく `use super::pump::Host`。`pump.rs` からはこの module を `use` しない）:
    - `fn apply_window_changes(&mut self, windows: &mut arvane::Windows<'_>)`: client の turn の後で `app.take_window_changes()` を順に実行する。`Open(request)` → `windows.open(request.clone())`、成功なら `HostWindow` を作り（`mode`・`parent` を request から）、`start_window(id, windows)` を呼んで、返った `(environment, surface_active)` で `HostEvent::WindowOpened { window: id, environment, surface_active }` を積む。失敗なら `app.remove_window(id)` と `HostEvent::WindowFailed { window, reason }`。`Close(id)` → `windows.close(id)`、`app.remove_window(id)`、`self.windows.remove(&id)`。`fn start_window(&mut self, id: WindowId, windows: &mut arvane::Windows<'_>) -> (Environment, bool)` は `begin` の window の部分を抜き出したもの（`HostWindow.environment.update(window.environment())`、`refresh_surface`、その window の `Model` に `set_viewport(window.size())`・`frame.set_now(window.now_ns())`・`frame.start_theme(MAIN の Model の frame.theme_hint())`・`start_env(env.clone(), drawable)`・`paint.set_scale(window.scale())`）。`begin` も MAIN についてこの関数を使う。
    - `fn finish_window(&mut self, id: WindowId, windows: &mut arvane::Windows<'_>)`: turn の後の window ごとの処理（title、IME、cursor、redraw の要求、a11y の公開）。`pump` はそれを全 window について呼ぶ。`control` は全 window の期限の最小。
    - `Host::effects`（`pump.rs:147-185`）は window ごとに回す: 各 window の `Model.effects` の `DeferredRequest::Platform(request)` を、その window の `windows.get(id)` の `Window::perform(request)` で実行する（`FocusWindow`・`RequestWindowSize`・`Fullscreen` など。`Fullscreen` の後の `update_env` もその window の `Model` に）。`ReducedMotion` は手順 6 のとおり `Host` で 1 回だけ。
13. frame の tick: `Model::tick(now)`（`model/frame/ticks.rs:134`）を `Model::tick(&mut self, now: u64, client: Option<&mut FrameRequests>) -> Option<FrameStamp>` に変える（`pub(crate) struct FrameStamp { pub(crate) now_ns: u64, pub(crate) estimated_present_ns: u64 }` を `ticks.rs` に足す）。期限は自分の `frame.frames` と `client` の最小で求め（`frame_deadline(&self, client: Option<&FrameRequests>)`）、tick したら `frame.frames` と `client` の `now` 以下の要求を消す。`HostEvent::FrameTick` は `Model::tick` の中で積まない: `client` が `Some` のとき（tick の源の window）だけ `Some(FrameStamp)` を返し、`pump` が `app.events().push(HostEvent::FrameTick { window: id, now_ns, estimated_present_ns, clock: app.motion() })` を積む。`pump` は「描画できる window のうち id が最小のもの」（tick の源。毎回導く。保存しない）にだけ `Some(&mut app.client_frames)` を渡し、ほかの window は `None`。描画できる window が無い間、core の要求は `AppModel.client_frames` に残る。host の `control` の frame の期限は、描画できる window の `Model::frame_deadline(..)`（tick の源の window には `Some(&app.client_frames)`、ほかは `None`）の最小。描画できる window が無ければ frame の期限は無い（今の `control` の `if self.drawable()` と同じ。期限の過ぎた `client_frames` で loop が空回りしないため）。
13a. `ShellEvent::Injected { request }` を受けたら `HostEvent::InjectDelivered { window, request }` を積む（`Host::handle` の arm）。
13b. App の motion の時計: `pump` の各回の最後に `fn finish_motion(&mut self, now: u64)`（`host/window_changes.rs` に置く）を呼ぶ。`let drawable = self.windows.keys().any(|id| self.drawable(*id));` を導き、`app.clock` が `Running` で `!drawable` なら `Stopped { since_ns: now }`、`Stopped { since_ns }` で `drawable` なら `app.motion.offset_ns += now.saturating_sub(since_ns); app.motion.resumed_ns = now; app.clock = Running`。続けて `Host.app_motion: Sent<(bool, u64)>` に `(drawable, frame_period_ns)`（`frame_period_ns` は tick の源の window の `Model` の `frame.period_ns()`。描画できる window が無ければ MAIN の）を `update` し、`Some` が返ったら `HostEvent::AppMotion { drawable, frame_period_ns }` を積む。
14. `present`・`redraw`・`wake`・`resized`（`pump.rs:234-274`、`events.rs:162-242`）は window の id を受け取り、その window について今の処理をする。frame log の行は今の形のまま（window の区別は書かない）。
15. `host/launch.rs`: `Launch` が新しい `arvane::Handler`（`start(windows, waker)`、`event(target, event, windows)`、`close(windows)`）を実装する。`run_native` の `arvane_native::backend(ShellOptions { .. })` は `arvane_native::backend(WindowRequest::main("Kivact", window_size))` にし、`shell.on_renderer_change(move |change| …)` は `move |_window, change| …` にする（`use arvane::{RendererChange, Shell}`、`ShellOptions` の import は消す）。`begin`（`events.rs:54-100`）が作る `HostInfo.capabilities` は、`window.capabilities()` ではなく `windows.capabilities()`（loop の値に GPU を足したもの。`multi_window`・`window_placement` を含む。設計書 3.6）から作る。`start` は `AssetStore`、`Shared`、main の `Model::new(WindowId::MAIN, window_size, &shared)`、`AppModel::new`、`Host`（`windows` に main の `HostWindow`）を作り、`host.begin(windows, ..)`。`close` は何もしない（presenter は arvane が閉じる）。
16. `embed.rs` と `host/testing/test_host.rs`: 新しい `Handler` に合わせる。`TestHostOptions`（`host/testing/test_options.rs`）に `single_window: bool`（doc: `/// The test platform opens no window beyond the first.`、既定 false）を足す。`ArvaneTestHost` の `new` は `VirtualOptions { main: WindowRequest::main(options.title.clone(), Size { width: options.width, height: options.height }), capabilities: Capabilities { multi_window: !options.single_window, window_placement: !options.single_window, ..VirtualOptions::default().capabilities }, .. }`、`ShellSetup` の observer は `Rc<RefCell<..>>` と `(window, change)` の形にする。window ごとの method に `window: WindowId` の第 1 引数を足す: `mouse_position`、`modifiers`、`raw_input`、`send`（`send(window, event)` は `Focused`・`Accessibility` 以外を `self.script.window(window).input(event)` に送る）、`mouse`、`contact`、`frame_stats`、`lose_gpu_device`、`frame_period_ns`、`renderer`、`frame`、`resize`、`set_scale`、`set_theme`、`set_locale`、`set_display_period`、`set_reduced_motion`、`set_safe_area`、`set_surface`、`surface_generation`、`request_close`（`self.send(window, ShellEvent::CloseRequested)`）、`semantics`、`semantics_target`、`semantics_id`、`hit`、`focused`、`hovered`、`modal`、`preorder`、`layout_findings`、`title`、`requested_sizes`、`min_window_size`、`ime_area`、`cursor`、`soft_keyboard`、`ime_purpose`、`ime_text`、`sync_window`、`has_window`（`Probe::window(id).is_some_and(|w| w.has_window())`。「その window が mirror の実 window に映っているか」であって、window が開いているかではない。開いているかは `windows()` で読む）。node を受け取る method（`rect`、`size`、`region_local`、`region_rect`、`window_point`、`kind`、`children`、`properties`、`input_value`、`scenes`、`shown`）は node から window を引くので変えない。loop の method（`wake`、`step`、`redraw`、`due`、`woken`、`exited`、`failure`、`now`、`set_pointer_settings`、`suspend`、`resume`、`memory_warning`、`set_clipboard`、`clipboard`、`push_file_dialog` など）はそのまま。`windows(&self) -> Vec<WindowId>` と `window_title(&self, id) -> Option<String>`、`scale(&self, window: WindowId) -> Option<f64>`（`self.probe.window(window).map(|w| w.environment().scale_factor)`。付録 SM-CORE-15 の engine の部分）、`focus_requests(&self, window) -> u32` を足す。`tests/host_locale.rs` の `host.set_locale(..)` を `host.set_locale(WindowId::MAIN, ..)` に変える（`use kivact_types::WindowId;`。試験の内容は変えない）。
17. 付録 SM-CORE-13 を適用する。
18. 確認: `cargo check -p kivact-engine --all-targets`、`cargo check -p kivact-engine --features testing --all-targets`、`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate kivact-engine --report`（0.2 の 12.）。`AppModel` の `focused_window`（`Option`）が error になったら `///` の行の上に `// xtask-allow(state-fields): independent fact - the window the OS focuses, written from the host's Focused events`、`Model`・`HostWindow` の既存の field は基準 commit の印の理由（struct の前の印のときはその文）を `<分類> - <事実>` の形（分類は付録 7.4 の一覧）にして field の `///` の行の上に移す。xtask の `impl_model` の規則（`impl *Model` は `model`・`paint`・`a11y`・`check` の下だけ）を守る。

---

## 9. W7（依存の段 5）

| task | 担当 | crate |
|---|---|---|
| T7.1 → T7.2 | 1（続けて行う） | `crates/kivact-core` |

波の検査: `cargo check -p kivact-core --all-targets`、`cargo check -p kivact-core --all-targets --features embed,gpu-interop,tokio`、`cargo clippy -p kivact-core --all-targets --all-features -- -D warnings`（`devtools` feature はまだ中身が無いので `--all-features` でも通る）。commit: `feat(core): windows as portals with their own roots`。

### T7.1 kivact-core: window

担当ファイル: `crates/kivact-core/src/**`。

参照: 設計書 3.1、3.8、3.11。今の構造: `view/tree.rs:286-366, 349-388, 669-888`、`view/root.rs:37-89`、`view/descriptor/description.rs:18-110`、`view/boundary.rs:32-53`、`view/flow/branch.rs`、`view/state.rs:64-255`、`launch/runtime.rs:107-187`、`launch/inspect.rs`、`client/dispatch/*.rs`、`platform/env.rs`・`services.rs`、`view/overlays.rs:505-547`、`client/trace.rs:91-130`、`view/mount/style.rs:96-176`。

手順:

1. `Mcx` に `pub window: WindowId` を足す。`Mcx { rt, owner }` を組み立てているすべての箇所（`grep -rn "Mcx {" crates/kivact-core/src`）に window を渡す。`fragment()`・`host(kind)` は `self.window` で node を作る。
2. `ViewNodes`: `root: NodeId` を `roots: BTreeMap<WindowId, NodeId>` に変える（window の root の持ち主は `ViewNodes.roots` だけ。`WindowRecord` に root・owner を持たない）。`root(&self, window) -> Option<NodeId>` と `owner_of(&self, node) -> Owner`（その node の `CoreNode.owner`）を足し、`root_owner` を field にしない（呼び出し元の 3 か所は `rt.root()` を使う）。`CoreNode` に window は足さない（node の window は mount 中は `Mcx.window`、それ以外は `ViewState::tree(rt).reader().window_of(id)`）。`ViewNodes::create(kind, owner, window)` は engine の `Writer::create(window, id, kind)` を呼ぶ。`ViewNodes::new` は main の root（`NodeId(1)`、`WindowId::MAIN`）を作る。`create_root(&mut self, window, owner) -> NodeId`（`NodeType::Root` の node を作り `roots` に入れる）を足す。`root_node`・`remove` などの root を前提にした method を `roots` に合わせる（`tree.rs:368-388`）。
3. `mount_in(rt, owner, window, build)`。flow の `Site`（`view/flow/branch.rs:84-96`）と、list の行・virtual list の行・`dynamic`・`error_boundary`・`provide*`・`component` で `mount_in` を呼ぶ箇所は、親の fragment の window（`cx.window`）を渡す。
4. `view/root.rs`: `mount_root` を `install_app_contexts(rt, owner, settings, at)` と `mount_window_root(rt, window, owner, build, at) -> NodeId` に分ける（設計書 3.8）。`mount_window_root` は root の style（今の 72-81 行）と中身の mount（82-83 行）をする。`launch/runtime.rs:173` は両方を呼ぶ（main は `WindowId::MAIN`）。
5. `view/state.rs`（新しい file を作らない。`EnvState`・`FocusKeys`・`ViewState` と同じ file に置かないと module の参照が循環する）:
   - `WindowRecord`（設計書 3.8 の field: `declared_in`、`env`、`focus`、`hover`、`mouse`、`title`、`min_size`、`dismiss`、`open`、`mode`。`hover`・`mouse` の初期値は `PointerObservation::Plain(None)` と `PointerObservation::Reactive(owner.signal(None, at))`）、`pub(crate) struct WindowScope(pub(crate) WindowId)`（`Clone`）、`pub(crate) fn current_window(owner: Owner) -> WindowId`（context を読み、無ければ `WindowId::MAIN`）。`WindowRecord.min_size` の直前に `// xtask-allow(state-fields): construction option - Window::min_size`（印は field の `///` の行の上。以下同じ）。
   - `ViewState` に `windows: RefCell<BTreeMap<WindowId, WindowRecord>>` と `next_window: Cell<u64>`（ID の採番。field の前に `// xtask-allow(state-fields): S-8 ID - the next window id`）。今の `ViewState.env`・`focus`・`hover`・`mouse`（`state.rs:204, 211, 217, 218`）は main の `WindowRecord` へ移し、`ViewState::env(window)` などの window を受け取る accessor にする（`view::pointer_position()` は呼び出した owner の window の `mouse` を読む。`modifiers` と `pointer_modality` は App に一つのまま `ViewState` に残す）。`ViewState.min_window_size` と `ViewState::min_window_size()` は消す（`ViewState::new` の引数 `min_window_size` も。main の `WindowRecord.min_size` を読む）。`initial_render` の直前に `// xtask-allow(state-fields): independent fact - the App has not drawn yet`（基準 commit の struct の前の印の文を移したもの）。
   - capabilities: `ViewState.capabilities`（定数、206 行）と `capabilities_value`（207 行）をやめ、`host_capabilities: Signal<Capabilities>`（`owner.signal(info.capabilities, at)`。host の event だけが書く。今これを書く event は無い）と `capabilities: Value<Capabilities>`（`ViewState::new` の中の 1 行 `let capabilities = owner.with_setup(|| kivact_runtime::memo(move || host_capabilities.get()));`。T11.1 がこの 1 行を `devtools_hooks::wrap_capabilities(owner, host_capabilities)` に置き換えるので、分けて書かない。何も書かない）にする。`ViewState::capabilities(&self) -> Capabilities` は `self.capabilities.get_untracked()`、`platform::capabilities()`（`platform/env.rs:165`）は `ViewState::of(&rt).capabilities` を返す。T11.1 が memo の中身を `devtools_hooks::wrap_capabilities` に置き換える。
   - motion: `ViewState.app_motion: Signal<AppMotion>`（`pub(crate) struct AppMotion { pub(crate) drawable: bool, pub(crate) frame_period_ns: u64 }`、`Clone, Copy, PartialEq`。初期値は `info.surface_active` と `info.env.frame_period_ns`）。`launch/runtime.rs:128-132` の `MotionEnv` は `surface_active` と `frame_period_ns` をこの signal の投影（`memo`）にする（main の `EnvState` から作らない。`reduced_motion` は main の env のまま）。`app_motion` は `MotionEnv` の組み立てより前に作る。
   main の record は `Launch::start` で作り（`declared_in` は root owner、`open` は常に true の binding、`title` は App の title の値、`min_size` は `App::min_window_size`）、root owner に `WindowScope(WindowId::MAIN)` を `provide` する。`writer.set_window(WindowId::MAIN, title, min_window_size)`（`launch/runtime.rs:152` 付近）にする。
6. `platform/env.rs`・`platform/services.rs`: `state(api, at)` は `current_window(Owner::ambient(..))` の window の `EnvState` を返す。window に向けた service は `HostRequest::Platform { window, request }` に window を入れる。`request_window_size` の最小値は、その window の record の `min_size`。`launch/settings.rs:202` の `check_window_sizes` を `crate::geometry`（`kivact-core/src/geometry.rs`）へ移して `pub(crate)` にし、`launch/settings.rs`（`AppOptions::check_window_sizes` の呼び出しを含む）と `view/window_view.rs` の両方から使う（`view` は module の規則で `launch` を参照できない）。error の message に出す API の名前は引数 `names: (&'static str, &'static str)` で渡す（`App` は `("App::window_size", "App::min_window_size")` で今の文のまま、`Window` は `("Window::size", "Window::min_size")`）。`platform::focus_window()` を `platform/services.rs` に足す（設計書 3.1.1。`set_fullscreen` を写し、`HostRequest::Platform { window, request: PlatformRequest::FocusWindow }`）。rustdoc: window を問う 9 つの関数（`window_size`、`scale_factor`、`is_fullscreen`、`safe_area`、`keyboard_inset`、`theme_hint`、`is_surface_active`、`is_motion_reduced`、`pointer_settings`）の doc に「the window of the calling owner」と書き、`capabilities()` の doc は設計書 3.2 のとおりにする。
7. `view/window_view.rs`（新規）: 設計書 3.1 の `window`・`Window` と `IntoView`。中身:
   - `Window` の field: `title: Value<String>`、`open: Binding<bool>`、`content: Option<ViewFn>`、`size: Size`（既定 640×480）、`min_size: Option<Size>`、`mode: WindowMode`、`placement: WindowPlacement`、`dismiss: Option<Handler<DismissReason>>`、`at`。`min_size` の直前に（`content`・`dismiss` は数えない型なので印を付けない）`// xtask-allow(state-fields): builder option - window() reads it once when the window opens`。
   - `IntoView` の最初の行: `geometry::check_window_sizes(Some(size), min_size, ("Window::size", "Window::min_size"), at)`。失敗したら宣言した owner に `ErrorKind::Message` で報告し、`open` を読む effect を作らずに空の view を返す。
   - 通れば `frag = cx.fragment()`（宣言した window の中の空の Contents）。`slot = WindowSlot { rt: Weak, parent: cx.owner, parent_window: cx.window, at, arm: RefCell<WindowArm>, unsupported: ReportOnce }` を owner に store する。`enum WindowArm { Closed, Open { id: WindowId } }`（root と owner は `ViewNodes` にだけ持つ）。`EffectKind::Internal` の effect で `open` を読み、`slot.switch(open)` を呼ぶ（`Show` の effect の作りを写す。`view/flow/branch.rs:284-318`）。宣言した owner の cleanup で `Open` なら閉じる（下の「cleanup から閉じるとき」）。
   - `switch(true)` で `Closed` のとき: capabilities の memo に `multi_window` が無ければ、`slot.unsupported.note()` が true のときだけ `UiError(ErrorKind::Unsupported, "this platform opens no window beyond the first")` を `parent` に報告して終わる。あれば: id を採番、`owner = parent.child(at)` を先に作り、`EnvState::new(owner, 親の env の今の値, 親の surface_active, at)` と `FocusKeys::new(owner, at)` はその `owner` で作る（記録の寿命を window の owner に合わせ、開閉のたびに signal と memo が残らないようにする）、`Writer::open_window(WindowRequest { id, title, size, min_size, mode, placement, parent: Some(parent_window) })`、`owner.provide(WindowScope(id))`、`root = mount_window_root(rt, id, owner, content, at)`、`WindowRecord { declared_in: parent, title, min_size, .. }` を `ViewState.windows` に入れ、title の Render effect（`Writer::set_window(id, title, min_size)`）を `owner` で作る。`arm = Open { id }`。
   - `switch(false)` で `Open { id }` のとき: `let root = tree.root(id)`、`let owner = tree.owner_of(root)` を取り、`ViewNodes::remove(root)` → `Removed::commit`、`Writer::close_window(id)`、`owner.dispose()`、record を消す、`arm = Closed`。
   - cleanup から閉じるとき（宣言した owner が破棄されるとき）: 上と同じだが `owner.dispose()` は呼ばない（window の owner は宣言した owner の子なので、親の破棄で一緒に破棄される。二重に破棄しない）。
8. dispatch（`client/dispatch/`）: `CloseRequested { window }` → `env.rs` の `close_requested(rt, window)`: main なら `rt.request_exit()`、それ以外は `let close = match &record.dismiss { None => true, Some(h) => run_handler(rt, Some(record.declared_in), || h.call(DismissReason::WindowClose)) == Some(Handled::No) }; if close { record.open.set(false) }`（`Yes`・`PreventDefault`・失敗では閉じない。設計書 3.1 の規則 5）。`EnvChanged { window, .. }` → その window の `EnvState`。`WindowOpened` → その window の `EnvState` を更新。`WindowFailed` → record の `declared_in` に `ErrorKind::Platform` を報告し `open.set(false)`。record の無い window（すでに閉じた）を指す event（`FrameTick` と `AppMotion` 以外の `window` を持つすべての variant と `Routed` の入力）は捨てる。`InjectDelivered` は何もしない arm（T11.1 が使う）。`AppMotion { drawable, frame_period_ns }` は `ViewState.app_motion.set(..)`。`FocusChanged`・`HoverChanged`・`MousePosition` は window の record に対して処理。`FrameTick` は window を無視して今どおり。
9. `view/overlays.rs` の `key_route`（521-536）・`escape_dismisses`（542-547）、`view/tree.rs` の `modal_boundary`（884-888）、`client/dispatch/common.rs` の `admitted`（58）、`client/dispatch/pointer.rs` の `run_requested_activations`（29）は、event か node の window を使う（engine の Reader の新しい method）。`key_route` の既定先は、その window の root（`tree.root(window)`）の最初の子（無ければ root）。
10. `launch/inspect.rs`: `root()` を `roots() -> Vec<(WindowId, NodeId)>`（`ViewNodes.roots` から）に、`app_view()` を削除、`check_tree` は全 root から辿る。`client/trace.rs` の semantics の差分は全 root を辿る。
11. `view/mount/style.rs` の `compile_style` に渡す `Interactive` に `forced: ForcedStates::default()` を足す（T4.2 で増えた field）。
12. `view.rs` から `window`、`Window` を、`kivact-types` から再公開している場所の近くで `WindowMode`、`WindowPlacement` を再公開する（`DismissReason` と同じ扱い）。`platform` の再公開に `focus_window` を足す。
13. 確認: `cargo check -p kivact-core --all-targets`。`cargo xtask check --no-write --only state-fields --crate kivact-core --report` で出る自分が作った・変えた struct の error は、上の印で足りなければ手順 5 と 7 の文を使う。
### T7.2 kivact-core: 状態の修正、is_mounted、style の層、cause の種類

担当ファイル: `crates/kivact-core/src/**`（T7.1 と同じ担当が続けて行う）。

手順:

1. 付録の SM-CORE-01、02、08、09、10、11、12 を適用する。SM-CORE-14（HOST-REWRITE、launch の内部契約）も適用する（T7.1 の変更の上に、付録の置き換えを合わせる）。SM-CORE-01 と SM-CORE-02 の置き換えのコードの field ごとの印（`generation`・`latest`・`rect` など）と、SM-CORE-14 の `ClientMode::Test { timing, name }` の変更をそのまま写す。
2. 付録 SM-INT-09 の core の部分（`kivact::view::is_mounted() -> Value<bool>`）を作り、`view.rs` から再公開する（prelude には入れない）。
3. style の層の記録（設計書 4.4.5 の最初の項目）:
   - `view/element_data.rs:573` の `pub(crate) type StyleLayer = Rc<dyn Fn() -> Style>;` を、設計書 4.4.5 の `#[derive(Clone)] pub(crate) struct StyleLayer { pub(crate) f: Rc<dyn Fn() -> Style>, pub(crate) at: Option<&'static Location<'static>>, pub(crate) origin: LayerOrigin }`、`#[derive(Clone)] pub(crate) enum LayerOrigin { Base, BaseRecipe(RecipeName), Recipe(RecipeName), Caller }`、`pub type RecipeName = Rc<dyn Fn() -> &'static str>`（`#[doc(hidden)]`）に変える。`StyleLayer` は数えられる field が `at` の 1 つだけなので、state-fields の印は要らない。
   - `Common` に `pub(crate) fn base_count(&self) -> usize { self.style.iter().take_while(|l| matches!(l.origin, LayerOrigin::Base | LayerOrigin::BaseRecipe(_))).count() }` を足し、`Common.base_styles: usize`（element_data.rs:518-521）は消す。`prepend_style`（580）は `Base` の層を `self.style.insert(self.base_count(), layer)`、`push_style`（585）は `Caller` の層を末尾へ。`mount/parts.rs:40-44` の
     `self.style.splice(self.base_styles..self.base_styles, later.style.drain(..later.base_styles)); self.base_styles += later.base_styles;` は `let at = self.base_count(); let n = later.base_count(); self.style.splice(at..at, later.style.drain(..n));` にする（`self.base_styles += …` の行は消す）。`mount/resolve.rs:239` の `base_styles: _` の行も消す。
   - `NodeBuilder::style`（element_data.rs:248）、`Node::style`（node_modifiers.rs:55）、`Node::base_style`（292）に `#[track_caller]` を付けて `Location::caller()` を `at` に渡す。
   - kit の recipe の層のために、`NodeBuilder` に `#[doc(hidden)] fn recipe_style(self, name: RecipeName, s: impl IntoValue<Style>) -> Self`（`Recipe(name)` の層を末尾へ）と、`Node` に `#[doc(hidden)] pub fn base_recipe_style(self, name: RecipeName, s: impl IntoValue<Style>) -> Self`（`BaseRecipe(name)` の層を base の群の末尾へ）を足す。どちらも直前に `// xtask-allow(internal-contract): kit recipes tag their style layers with the recipe's name for DevTools`。
   - `merge_layers`（mount/style.rs:19）は `layer.f` を呼ぶ（`layers.iter().fold(Style::default(), |s, layer| s.merge(&(layer.f)()))`）。
   - （`mount/style.rs:123` の `Interactive` の `forced` は T7.1 の 11. が足した。ここでは足さない。T11.1 が `hook.forced()` に置き換える）
4. cause の種類（設計書 4.4.3。T3.2 が `TurnGuard::cause` を `cause_with(tag, label)` に置き換えた）: `launch/runtime.rs` の `turn.cause(describe(event));`（239 行）を `turn.cause_with(kivact_runtime::CauseTag::Input, describe(event));` にする。event が無く、「早く返す」判定（201-205 行）を通り抜けた turn のうち、`boundaries.last()` の revision で `rt.mounts_due(..)` が真のものには、`rt.begin_turn` の直後に `turn.cause_with(kivact_runtime::CauseTag::Mount, "mounts".into())` を足す（`events.is_empty() && boundaries.last().is_some_and(|b| rt.mounts_due(b.revision))` のとき）。DevTools の呼び出しの cause（`CauseTag::Devtools`）は T11.1 が足す。
5. 確認: `cargo check -p kivact-core --all-targets`、`cargo check -p kivact-core --all-targets --features embed,gpu-interop,tokio`、`cargo clippy -p kivact-core --all-targets --all-features -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate kivact-core --report`（0.2 の 12.）。`kivact-test` は W8 まで compile できない（T8.1 が直す）ので、ここでは確かめない。

---

## 10. W8（依存の段 6）

| task | 担当 | crate |
|---|---|---|
| T8.1 | 1 | `crates/kivact-test` |
| T8.2 | 2 | `crates/kivact-interaction` |

波の検査: `cargo check -p kivact-test -p kivact-interaction --all-targets`、`cargo check -p kivact-test --all-targets --features embed,gpu-interop`、clippy。commit: `feat(test, interaction): screens per window; interaction state fixes`。

### T8.1 kivact-test: window ごとの Screen

担当ファイル: `crates/kivact-test/src/**`、`dev/tests/kivact-public-api/tests/e2e/launch_real_window.rs`（`has_window` の改名の追従だけ）。

参照: 設計書 3.9、付録 SM-CORE-07、SM-CORE-14 の kivact-test の部分、SM-CORE-15 の kivact-test の部分（HOST-REWRITE）。今の構造: `screen/model.rs:17-115`、`screen/driver.rs`、`screen/query.rs`、`screen/input.rs`、`screen/controls.rs`、`screen/artifacts.rs`、`screen/state.rs`、`screen/layout.rs`、`facts.rs`、`script.rs`、`render.rs`、`options.rs`。

手順:

1. `screen/model.rs`: 今の `Screen` の field を `pub(crate) struct TestApp`（同じ field）に移し、`#[derive(Clone)] pub struct Screen { pub(super) app: Rc<TestApp>, pub(super) window: WindowId }` にする。`Element<'a>`・`Within<'a>` は `&'a Screen` を持つまま（`screen.window` で window が分かる）。`Drop for Screen`（artifacts.rs:255-282）は `Drop for TestApp` に移す（最後の handle が drop されたとき）。`Contact` に `window: WindowId` を足し（`contact_down` が handle の window を入れる）、`contact_*` はその window へ送る。handle の window と違えば `self.fail("contact belongs to another window")`。`TestApp.scale` は消す（付録 SM-CORE-15。`Screen::scale` は `self.app.host.borrow().scale(self.window).expect("invariant: the window is open")`）。
2. window ごとの処理: `screen/` の中で host を呼ぶ箇所すべてに `self.window` を渡す（T6.1 で `ArvaneTestHost` の window ごとの method に引数が増えた。compile error の箇所を全部直す）。`facts()` は `Facts::new(inspect, self.window)`。`facts.rs` は `inspect.roots()` の中のその window の root から辿る。
3. 新しい method（設計書 3.9）: `window(&self, title: &str) -> Screen`（`find_by_*` の待ち方（`query.rs:152-177`）を写し、`host.windows()` と `host.window_title(id)` で探す。失敗の message は開いている window の title の一覧。同じ title の window が 2 つ以上あれば `get_by_*` と同じく "2 windows are titled …" で失敗する）、`windows(&self) -> Vec<Screen>`、`is_open(&self) -> bool`。閉じた window で window 向けの method を呼んだら、`self.fail(&format!("window {} is closed", self.window.get()))`（title は閉じた後に読めない）。`has_window()`（`controls.rs:261`）は `is_mirrored()` に改名する（`real_window_tests.rs:201` の呼び出しと `launch_real_window.rs:84, 96` も合わせる。意味は「その window が mirror の実 window に映っているか」のまま）。
   `request_close()`（`controls.rs:146-160`）: main の handle では今どおり（App の終了を待ち、終了したかを返す）。main 以外の handle では、その window に `CloseRequested` を送り（`host.request_close(self.window)`）、`run_until_idle()` の後で `!self.is_open()` を返す（App の終了は待たない）。
4. 成果物の名前（設計書 3.9）。
5. `script.rs`: `Script` は `{ platform: arvane_virtual::Script, window: WindowId }` を持つ。window ごとの method（`set_keyboard_inset`、`raw`、`mouse`、`contact`、`focus`、`accessibility_action`、`resize`、`set_scale`、`set_frame_period`、`set_reduced_motion`、`set_dark`、`set_locale`、`set_safe_area`、`create_surface`、`destroy_surface`、`set_occluded`、`fail_presents`）は `self.platform.window(self.window).<同名>(..)` へ渡し、loop の method（`saved_state`、`save_state`、`respond`、`set_now`、`set_clipboard`、`push_file_dialog`、`push_files_dialog`、`fail_next_effect`、`fail_next_effect_unsupported`）は今どおり `self.platform.<同名>(..)`（公開の名前と形は変えない）。`Screen::script()`（`model.rs:102-106`）は handle の window に結び付いた `Script` を返す。
5a. `options.rs` の `RenderOptions` に builder を一つ足す: `pub fn single_window(mut self) -> Self`（doc: `/// The test platform opens no window beyond the first (\`Capabilities.multi_window\` and \`window_placement\` false), like an embedded editor.`）。`RenderOptions` に `single_window: bool`（既定 false）を持たせ、`into_host` が `TestHostOptions.single_window`（T6.1 が足した field）に渡す。`RenderOptions` は名前が `Options` で終わるので structural-options の検査の対象外（policy の変更は要らない）。`capabilities(..)` と `devtools()` は足さない（`devtools()` は feature と一緒に T11.1 が足す）。
6. 付録 SM-CORE-07 を適用する。SM-CORE-14 の kivact-test の部分（`screen/driver.rs:36, 42, 62-63`。`open_trace` の呼び出しを消し、`ClientMode::Test` に test の名前を渡す）も適用する。SM-CORE-15 の engine の部分は T6.1 が済ませた。`Screen::scale` は `self.app.host.borrow().scale(self.window).expect("invariant: the window is open")`（手順 1）。
7. 確認: `cargo check -p kivact-test --all-targets`、`cargo check -p kivact-test --all-targets --features embed,gpu-interop`、`cargo clippy -p kivact-test --all-targets --all-features -- -D warnings`、`cargo xtask check --no-write --only state-fields --crate kivact-test --report`（`TestApp` の field は基準 commit の `Screen` の field 単位の印を `<分類> - <事実>` の形（分類は付録 7.4 の一覧）にして、field の `///` の行の上に写す。例: `last_contact` は `// xtask-allow(state-fields): S-8 ID - the platform id of the last contact`）。

### T8.2 kivact-interaction: 状態の修正と TreeApi

担当ファイル: `crates/kivact-interaction/src/**`、`dev/tests/kivact-public-api/tests/e2e/{lifecycle,interaction_menu,ui_disclosure_surfaces,interaction_list_nav,interaction_edit,interaction_palette}.rs`（付録の E2E の案を足す既存の file）、新しい E2E の file `dev/tests/kivact-public-api/tests/e2e/interaction_state.rs`（`main.rs` への登録は T9.3 が行う）。

手順:

1. 付録の SM-INT-01、02、03、07、08、09（menu の部分）、10、11、12、14（kivact-interaction の部分: `crates/kivact-interaction/src/edit/handle.rs` の `EditSession::replace_initial`）、15、16 を順に適用する。SM-INT-14 の creative の部分は T9.2 が行う。
2. `DismissReason::WindowClose` を、`DismissOn` の対応（`dialog.rs:290-305` の `allowed`）で `Escape` と同じ扱いにしている T2.4 の暫定の arm を確かめる（そのままでよい）。
3. `TreeApi` に `scroll_with` と `page_size` を足す。形は `ListApi` の同名の method（`list/handle.rs:98, 120`）と同じにする（`pub fn scroll_with<R: HandlerResult>(self, f: impl Fn(K) -> R + 'static) -> Self`、`pub fn page_size(self, rows: impl IntoValue<usize>) -> Self`。実際の形は `handle.rs` の定義を写す）。中身は内部の `ListApi` へ渡すだけ（設計書 4.5.4）。
4. 付録の E2E の案は、付録が名指した既存の file に、その file の helper（`real_window`、`menu_button` など。file の中の private な `fn`）を使って足す（上の担当ファイルの 6 file）。新しい file `interaction_state.rs`（`//! Role: Behaviour contracts of interaction state fixes.`）には `TreeApi::scroll_with` の E2E だけを書く（`flows.rs:343-412` の `use_list` の例を写し、tree で End キーが未 mount の行に focus を移すこと）。
5. 確認: `cargo check -p kivact-interaction --all-targets`、`cargo clippy -p kivact-interaction --all-targets -- -D warnings`。

---

## 11. W9（依存の段 7〜9）

| task | 担当 | crate・場所 |
|---|---|---|
| T9.1 | 1 | `crates/kivact-ui` |
| T9.2 | 2 | `crates/kivact`（facade）、5 つの kit、`examples/`、`templates/`、`dev/tests/docs` の追従 |
| T9.3 | 3 | `dev/tests/kivact-public-api`（E2E の追加と登録） |

3 つは依存し合うので、指揮役は T9.1 → T9.2 → T9.3 の順に 1 つずつ起動する（T9.2 は kivact-ui を使う kit を直し、T9.3 の E2E は T9.2 が prelude に足す `window` と kit を使う）。波の検査: `cargo check --workspace --all-targets --all-features`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`。commit: `feat: windows in the facade, kit follow-ups and multi-window E2E`。

### T9.1 kivact-ui

担当ファイル: `crates/kivact-ui/src/**`、`dev/tests/kivact-public-api/tests/e2e/ui_standard.rs`、`crates/kivact-ui-soft/tests/consumer/selector_touch.rs`。

手順:

1. 付録の SM-UI-01〜10 を、付録に書いた順（SM-UI-08 は 01・02 の後）に適用する。
2. recipe の名前（設計書 4.4.5）: `recipes.rs:80` の `Recipe<S>` を `pub struct Recipe<S> { f: Arc<RecipeFn<S>>, name: &'static str }` にする。`From<F>` は `name: ""` で作り、`pub(crate) fn named(mut self, name: &'static str) -> Self` を足し、`pub fn name(&self) -> &'static str` を作る。`Deref`・`same`・`Clone` は `f` を使う。`recipe_table!`（115 行）は、既定の recipe を作る箇所（`Recipe::from(..)`）と、kit が置き換える setter（`Recipe::from(f(self.$field))` の箇所）の両方で `.named(concat!(stringify!($name), ".", stringify!($field)))` を付ける（置き換えても名前が残る）。`default_table()`（今は private）は `pub(crate) fn default_table() -> &'static Self` にする。`layer.rs` の `with_recipe_style`（15 行）は `root.style(..)` の代わりに `let name = pick(UiRecipes::default_table()).name(); root.recipe_style(Rc::new(move || name), move || ..)`、`with_base_recipe_style`（31 行）は `node.base_style(..)` の代わりに `node.base_recipe_style(<同じ名前の関数>, ..)` を呼ぶ（T7.2 で足した hidden の method）。名前は theme を読まずに決まる（kit が recipe を置き換えても名前は変わらないので、既定の table から読めばよい。DevTools が owner の外から呼んでも `use_theme()` を呼ばない）。258 か所の呼び出しは変えない。
3. 付録 SM-UI-08・SM-UI-09 の試験の案を、付録が名指した file（SM-UI-08 は `dev/tests/kivact-public-api/tests/e2e/ui_standard.rs` の `theme_with`・`options`・`pixel_mid`、SM-UI-09 は `crates/kivact-ui-soft/tests/consumer/selector_touch.rs` の `selection_field_reopened_through_the_caller_binding_takes_a_fresh_surface`）に、その file の helper を使って足す（この task では走らせない）。
4. 確認: `cargo check -p kivact-ui --all-targets`、`cargo clippy -p kivact-ui --all-targets -- -D warnings`。

### T9.2 facade と kit の追従

担当ファイル: `crates/kivact/**`、`crates/kivact-ui-{basic,creative,game,pop,soft}/**`、`crates/kivact-core/Cargo.toml`、`crates/kivact-core/src/lib.rs`（`mod devtools;` の 1 行）、`crates/kivact-core/src/devtools.rs`（新規）、`examples/**`、`templates/**`、`dev/tests/docs/**`、`dev/tests/gpu-interop-producer/**`、`dev/tools/check-features.sh`。

手順:

1. facade: `crates/kivact/src/prelude.rs` の `view::{..}` の並びに `window`、`WindowMode`、`WindowPlacement` を足す（`dialog`・`DialogMode`・`Placement` と同じ扱い。設計書 3.1）。`crates/kivact/Cargo.toml` に `devtools = ["kivact-core/devtools"]`。`crates/kivact-core/Cargo.toml` に設計書 4.1 の `devtools` feature と optional 依存（`kivact-devtools-agent`、`kivact-devtools-protocol`）。`crates/kivact-core/src/lib.rs` に `#[cfg(feature = "devtools")] mod devtools;` と、空の `src/devtools.rs`（`//! Role: The DevTools target: answers the agent's calls from the App's internals.` だけ）。
2. `dev/tools/check-features.sh` の `entries` に `kivact/devtools` と `kivact-core/devtools`。
3. creative: 付録 SM-INT-14 の creative の部分（outliner）を適用する。`dev/tests/gpu-interop-producer`: 付録 SM-CORE-04 を適用する。
4. `cargo check --workspace --all-targets --all-features` を実行し、kit・例・雛形・docs の試験の中で、これまでの波の変更（`DismissReason` の新しい variant の match、`kivact::test` の変更など）で壊れた箇所だけを直す。公開 API の使い方を変えずに直せないものがあれば報告して止める。
5. 確認: `cargo check --workspace --all-targets --all-features`。

### T9.3 複数ウィンドウの E2E

担当ファイル: `dev/tests/kivact-public-api/tests/e2e/windows.rs`（新規）、`dev/tests/kivact-public-api/tests/e2e/main.rs`。

手順:

1. `windows.rs`（`//! Role: Contracts of windows opened from views.`）に、設計書 3.12 の 12 の振る舞いを 1 つずつ試験にする。形は既存の試験（例 `tests/e2e/keyboard.rs`）を写し、`support::mount` を使い、最後に `assert!(screen.errors().is_empty(), "{:?}", screen.errors());`。window を指す handle は `let settings = screen.window("Settings");` のように先に変数に入れる（`screen.window(..)` の戻り値から取った `Element` を `let` に入れると E0716 になる）。閉じる操作は `settings.request_close()`。10 番（multi_window が false）は `RenderOptions::default().single_window()` を使う。12 番は `screen.script().destroy_surface()`（main の handle の `script()` は main の window に結び付く）の後に別の window の `animated` が進むことを確かめる。
2. `main.rs` に `mod interaction_state;`（T8.2 が作った）と `mod windows;` を足す（W1〜W8 で `dev/tests/kivact-public-api/tests/e2e/` に新しい file を作る task は T8.2 だけ）。
3. 確認: `cargo check -p kivact-public-api-tests --tests`。

---

## 12. 検査点 C1（指揮役）

複数ウィンドウと状態の修正がそろったところで、一度だけ全体を確かめる。

1. `cargo fmt --all`（差分が出たら commit に含める）。
2. `cargo check --workspace --all-targets --all-features` と `--no-default-features`。
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings`。
4. `cargo xtask check --no-write --only state-fields --report > target/plan/state-fields-after-w9.txt`。残っている error の一覧を保存する（T15.4 で片付ける）。続けて `cargo xtask check --no-write`（全体。module の参照の循環、visible_to、module の規則、style の規則も確かめる）。state-fields 以外の error は、この検査点で直す（6. と同じ手順で、error の file の担当 task へ戻す）。
4a. Linux の compile（0.5。WSL があれば `--workspace --all-targets`）。
5. 試験（時間がかかる。この順に 1 回ずつ）:
   - `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features`
   - `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1`
   - `KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1`
6. 失敗した試験は、試験の名前とエラーを `target/plan/C1-FAILURES.md` にまとめ、失敗した file の crate を担当した task のサブエージェントを、失敗の全文を付けて起動して直させる（担当ファイルの範囲の中で）。直した後は、失敗した試験の binary だけを名前の filter 付きでもう一度回す（全体はもう回さない）。
7. 全部通ったら commit: `test: pass the suite with windows and state fixes`（差分があれば）。

---
## 13. W10: engine の DevTools の口

| task | 担当 | crate |
|---|---|---|
| T10.1 | 1 | `crates/kivact-engine`（と、`check/` を常にコンパイルするための `lib.rs`） |

波の検査: `cargo check -p kivact-engine --all-targets --features testing`、`cargo check --workspace --all-targets --all-features`、clippy（engine）。commit: `feat(engine): devtools overlay, pick, keys, recording and inspection`。

### T10.1 engine の口

担当ファイル: `crates/kivact-engine/src/**`、`crates/kivact-engine/Cargo.toml`（`arvane-shell` の依存を足す。`pipeline::Raw*` の再公開にだけ要る）、`crates/kivact-types/src/render/composition.rs`（`LayerKey::Devtools` は T2.4 が足した。使うだけ）、`crates/kivact-core/src/client/dispatch/delivery.rs`（`dispatch_host` の match に、新しい variant の何もしない arm を足すだけ。core の match は網羅しているので、arm が無いと workspace がコンパイルできない）。

参照: 設計書 4.4.2 の表（1 行ずつ実装する）、4.9。

手順:

1. `crates/kivact-engine/Cargo.toml` の `[dependencies]` に `arvane-shell = { path = "../arvane-shell", version = "0.1.0" }` を足す（policy の `visible_to` が engine を許している）。
2. 型（`contract/devtools.rs`、新規の leaf。`contract.rs` から再公開。`kivact_engine::contract` は state-fields の `value_modules` なので、この file の型は検査されない。印を書くと "unused marker" の error になるので、どの field にも印を書かない）:

   ```rust
   /// What the DevTools draws over one window.
   #[derive(Clone, Debug, Default, PartialEq)]
   pub struct DevtoolsOverlay { pub boxes: Vec<OverlayBox>, pub outlines: Vec<OverlayOutline>, pub fills: Vec<OverlayFill>, pub labels: Vec<OverlayLabel> }
   #[derive(Clone, Debug, PartialEq)]
   pub struct OverlayBox { pub margin: Rect, pub border: Rect, pub padding: Rect, pub content: Rect }
   #[derive(Clone, Debug, PartialEq)]
   pub struct OverlayOutline { pub rect: Rect, pub color: Rgba, pub width: f64 }
   #[derive(Clone, Debug, PartialEq)]
   pub struct OverlayFill { pub rect: Rect, pub color: Rgba }
   #[derive(Clone, Debug, PartialEq)]
   pub struct OverlayLabel { pub at: Point, pub text: String }
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum PickPhase { Hover, Select, Cancel }
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct KeyChord { pub key: Key, pub modifiers: Modifiers }
   /// The keys that open the DevTools and those that open it in pick mode.
   #[derive(Clone, Debug, Default, PartialEq, Eq)]
   pub struct DevtoolsKeys { pub open: Vec<KeyChord>, pub pick: Vec<KeyChord> }
   #[derive(Clone, Debug, PartialEq)]
   pub struct SyncRecord { pub window: WindowId, pub revision: Revision, pub reasons: u8, pub measured: Vec<NodeId>, pub laid_out: Vec<NodeId>, pub redrawn: Vec<NodeId> }
   #[derive(Clone, Debug, PartialEq)]
   pub struct FrameRecord { pub window: WindowId, pub revision: Revision, pub end_ns: u64, pub work_ns: u64, pub scene_ns: u64, pub raster_ns: u64, pub present_ns: u64, pub stats: FrameStats, pub gpu: bool, pub frame_period_ns: u64 }
   /// What the core takes each turn; `dropped` counts the records the ring lost.
   #[derive(Clone, Debug, Default, PartialEq)]
   pub struct EngineRecords { pub syncs: Vec<SyncRecord>, pub frames: Vec<FrameRecord>, pub dropped: u32 }
   #[derive(Clone, Copy, Debug, Default, PartialEq)]
   pub struct EngineEmulation {
       pub theme_hint: Option<ThemeHint>,
       pub scale: Option<f64>,
       pub touch: Option<bool>,
   }
   ```

   `Rgba`・`Rect`・`Point`・`Key`・`Modifiers`・`ThemeHint`・`FrameStats`・`Revision`・`WindowId`・`NodeId` は kivact-types の既存の型。`SyncRecord.reasons` は `LayoutReasons` の bit（`PASS = 1`、`STYLE = 2`、`VIEWPORT = 4`）。`FrameRecord.end_ns` は present の時点の loop の時計（`window.now_ns()`。protocol の時刻と同じ時計。wall clock は `KIVACT_FRAME_LOG` だけが持つ）。`FrameRecord.revision` は、その frame を描いたときに commit 済みだった最新の revision。`BoxModelRects`・`ImeStateInfo`・`DevtoolsChange` は設計書 4.9 のとおりに同じ file に置く。`EngineEmulation.touch` は `PointerSettings.primary` を `Coarse` にする指定で、`Capabilities.touch` は engine が上書きしない（core の `capabilities` の memo が掛ける）。
   同じ file から入力の型を再公開する（core は arvane-shell に依存できない。nested な `{}` の path は xtask の import-granularity で error になるので 2 行に分ける）。`InjectedInput` は T5.2 が arvane の root で再公開しているのでそこから、`pipeline::Raw*` は arvane が再公開していないので arvane-shell から。直前に 1 つの `// xtask-allow(internal-contract): the core builds injected input for the DevTools agent and cannot depend on the shell` を付ける:

   ```rust
   pub use arvane::InjectedInput;
   pub use arvane_shell::pipeline::{PlatformKey, RawDrop, RawIme, RawInput, RawKey};
   ```
3. `HostEvent` に `DevtoolsPick { window: WindowId, node: Option<NodeId>, phase: PickPhase }` と `DevtoolsKey { window: WindowId, pick: bool }`。`HostRequest` に `Inject { window: WindowId, inputs: Vec<InjectedInput> }`（識別は `Writer::effect` に渡す `RequestId`。別の番号を作らない）、`Emulate(EngineEmulation)`、`TimeScale(f64)`。`HostEvent::InjectDelivered { window, request }` は T6.1 が足した（ここでは足さない）。`Writer::effect` の match に 3 つの arm を足す: `Inject` はその window の `Model.effects` に `DeferredRequest::Inject(inputs)`（window が無ければ捨てる）、`Emulate`・`TimeScale` は `ReducedMotion` と同じ列に `DeferredRequest::Emulate(..)`・`DeferredRequest::TimeScale(..)`。core の `client/dispatch/delivery.rs` の `dispatch_host` の match に `HostEvent::DevtoolsPick { .. } | HostEvent::DevtoolsKey { .. } => {}` を足す（`InjectDelivered` の何もしない arm は T7.1 が置いた。T11.1 が 3 つとも `devtools_hooks::host_event` へ渡す arm に置き換える）。`describe`（`launch/runtime.rs`）は `other =>` の arm があるので変えない。
4. 記録の持ち主（`model/events.rs`、`EventSink` と `Shared` の隣）。記録の有無と記録は、全 window の `Model` と `Host` が共有するこの 1 つの型にだけ置く（`Model` ごとの flag・ring、`DevtoolsHost.recording`・`records`、`DevtoolsChange::Recording` は作らない）:

   ```rust
   /// What the DevTools records: one log shared by every window's Model and the Host.
   #[derive(Clone, Default)]
   pub(crate) struct EngineLog(Rc<RefCell<EngineLogState>>);
   #[derive(Default)]
   pub(crate) struct EngineLogState { syncs: Log<SyncRecord>, frames: Log<FrameRecord> }
   /// S-3: the ring exists only while recording.
   #[derive(Default)]
   pub(crate) enum Log<T> { #[default] Off, Recording { ring: VecDeque<T>, dropped: u32 } }
   impl<T> Log<T> {
       fn set(&mut self, on: bool) { /* on && Off → Recording { empty, 0 }; !on → Off */ }
       fn push(&mut self, cap: usize, make: impl FnOnce() -> T) { /* Recording のときだけ。ring.len() == cap なら pop_front して dropped += 1。make() を push_back */ }
       fn drain(&mut self) -> (Vec<T>, u32) { /* Off は (vec![], 0)。Recording は ring を空にして dropped を 0 に戻して返す */ }
   }
   impl EngineLog {
       pub(crate) fn set(&self, syncs: bool, frames: bool);
       pub(crate) fn sync(&self, make: impl FnOnce() -> SyncRecord) { /* cap 4096 */ }
       pub(crate) fn frame(&self, make: impl FnOnce() -> FrameRecord) { /* cap 512 */ }
       pub(crate) fn take(&self) -> EngineRecords;  // 両方を drain、dropped は 2 つの合計
   }
   ```

   `Shared`（T6.1）に `log: EngineLog` を足し、全 `Model` と `AppModel` が同じ clone を持つ。`EventSink` に `applied() -> Revision`（`EventQueue::applied`）を足す（記録の revision に使う。足されていれば何もしない）。`Writer::set_engine_recording(&self, syncs: bool, frames: bool)` は `log.set(..)`、`Reader::take_engine_records()` は `log.take()`。
5. `Host` の状態（`host/devtools.rs`、新規の leaf）: `pub(crate) struct DevtoolsHost { overlays: BTreeMap<WindowId, DevtoolsOverlay>, pick: Option<WindowId>, keys: DevtoolsKeys, emulation: EngineEmulation }`（`Host` の field に 1 つ。数えられる field は `pick` の 1 つだけなので、state-fields の印は要らない）。`Writer` の新しい method（`set_devtools_overlay`、`set_pick`、`set_devtools_keys`）は `AppModel` の `devtools_changes: Vec<DevtoolsChange>` に積み、host が turn の後で `DevtoolsHost` に移す（`window_changes` と同じ流れ。`AppModel` に Option の「次の値」は作らない）。`WindowChange::Close(id)` の処理（T6.1 の 12.）に、`DevtoolsHost.overlays` の `id` の entry を消し、`pick == Some(id)` なら `pick` を `None` にする 1 文を足す。
6. 重ね表示: `Host::present`（window ごと）の `model.composition(..)` の後で、その window の `DevtoolsOverlay` があれば、composition に layer を足す。layer は木の node なので、root の items から参照しないと描かれない（`validate` は全 child が親から 1 回参照されることを要求する）:

   ```rust
   let index = composition.layers.len() as u32;
   composition.layers.push(CompositeLayer {
       key: LayerKey::Devtools,
       transform: Matrix4::IDENTITY,
       bounds: <window の論理の矩形>,
       plane: Plane::Flat,
       backface: Default::default(),
       grid: RasterGrid::Target,
       effects: LayerEffects::default(),
       items: vec![LayerItem::Raster(<重ね表示の RenderScene>)],
   });
   composition.layers[0].items.push(LayerItem::Layer { index, clips: Vec::new() });
   ```

   `RenderScene { commands }` は `RenderCmd::Rect { rect, fill: Paint::Solid(color) }`（塗り）と `RenderCmd::StrokeRect { rect, width, color, radius: 0.0 }`（線）で作る。箱は 4 つの矩形を半透明で塗る（margin `#f6b26b` α0.4、border `#ffe599` α0.4、padding `#93c47d` α0.4、content `#6fa8dc` α0.4）。`fills` は `OverlayFill` の色で塗る。outline は `StrokeRect`、label は黒地に白の文字（文字は `arvane::Engine` の text で shape する。今の `lower` の text の書き方を写す）。重ね表示が変わったら `paint.request_redraw()`。
7. `Host::handle` の入口の gate の順（設計書 4.4.2 の下）: T6.1 が置いた modal の gate（「modal の親への入力の破棄」）の**前**に、次の 2 つを置く:
   - (1) 起動キー: `Key` の Down が `keys.open` か `keys.pick` のどれかと一致したら、event を捨てて `DevtoolsKey { window, pick }`（`pick` は `keys.pick` に一致したとき true）を積む。一致の判定は `modifiers == chord.modifiers` かつ、key が `Key::Char(c)` なら `c.to_lowercase().eq(chord_char.to_lowercase())`、それ以外は `==`。`Key::Unidentified` のときは `KeyInput.code` の `Debug` の文字列が `Key` + 英字 1 文字（`KeyI`）なら、その小文字と chord の文字で比べる（native は Shift を押すと `Char('I')`、X11 は Ctrl で制御文字になるため）。
   - (2) pick: `pick == Some(id)` かつ target がその window のとき、`Pointer` の Move は `Reader::hit_any` と同じ計算で node を求め `DevtoolsPick { phase: Hover }`、左 button の Down で `Select`、Key の Escape の Down で `Cancel` を積み、event を捨てて `defer` で返す。
8. 記録の箇所（`log.sync(|| ..)`・`log.frame(|| ..)` の closure は `Recording` のときだけ走る）:
   - `Model::sync`（`model/interact/sync.rs:96`）: 最初に `let reasons = self.changes.layout.bits()`（`LayoutReasons` に `pub(crate) fn bits(self) -> u8` を足す。`sync()` は `VIEWPORT`・`STYLE` を `take` するので、読むのは最初）。`let changes = self.changes.dispatch();`（153 行）の後で、`redrawn` を `changes.marked(Dirty::REDRAW)` から集め、`self.log.sync(|| SyncRecord { window: self.id, revision: self.events.applied(), reasons, measured: Vec::new(), laid_out: Vec::new(), redrawn })`。`reasons == 0` で `redrawn` が空なら記録しない（`sync()` は 1 turn に何度も走る）。
   - `Model::layout`（`model/interact/geometry.rs:141`、`for id in remeasure` の前後）: `remeasure` の id を `measured`、`self.layout.drain_geometry_changes()`（169 行）の id を `laid_out` に集め、どちらかが空でなければ `SyncRecord { reasons: 0, redrawn: Vec::new(), .. }` を 1 件。同じ revision の `SyncRecord` が 2 件になってよい（core が revision で束ねる）。
   - `Host::present` の `FrameTimes` を作る箇所（`pump.rs:251-272`）: `FrameTimes` を一度だけ作り（`frame_log.enabled()` か frames の記録が `Recording` のとき）、`frame_log.frame(times, stats)` と `log.frame(|| FrameRecord { .. })` の両方へ同じ値を渡す（`FrameLog` の行を別に組み立てない）。`end_ns = window.now_ns()`（`export_presented` に渡している値と同じ）、`gpu = matches!(renderer, arvane::Renderer::Gpu)`（`Presented::Shown { renderer, .. }`）、`frame_period_ns` はその window の `Environment.frame_period_ns`、`revision = app.events().applied()`。
9. Reader の読み取り（設計書 4.4.2 の表の「箱の幾何」〜「IME」）: `box_model(id)`（kivact-layout の `NodeLayout` の padding・border・margin と、`Model::rect` の位置から window 座標の 4 つの矩形）、`effective_paint(id)`（`Model::effective_paint`）、`computed_text(id)`（`Model::text_props` の結果）、`layout_props(id)`、`hit_any(window, point)`（`hit_node` を root と全 layer に、modal を無視して）、`focused(window)`、`tab_order(window)`（`Model::tab_stops`）、`pointer_state(id) -> (bool, bool)`（`Model::is_hovered`・`Model::is_pressed`、`model/pointer/hover.rs:9, 85`）、`layout_findings(window, check)`、`ime_state(window)`（`Edits` の `given`・`target` と preedit）。`windows()`・`window_of`・`root`・`focused_window` は T6.1 が足した。
10. `check/` を常にコンパイルする: `lib.rs:61-62` の `cfg(any(debug_assertions, feature = "testing"))` を外す（`layout_findings` の `cfg(feature = "testing")` も外す）。ただし `check.rs` の `mod lint;` には `#[cfg(debug_assertions)]` を残す（debug の自動の lint は `Model.lint`（`aggregate.rs:57-58`、`debug_assertions` のときだけある field）を使うので、release では compile しない）。`sync.rs:201` の `lint_layout` の呼び出しも今どおり `debug_assertions` のときだけ。
11. a11y action の専用の経路は作らない（`Writer::a11y_action` は無い）。a11y の step は core が `InjectedInput::Accessibility` として `HostRequest::Inject` の列に入れる。順序の保証と受理済みの a11y tree による検証は `ShellLoop::inject`（T4.1・T5.1）の仕事。
12. host が turn の後で実行する deferred request（`Host::effects`。`pump.rs` の `effects`）の arm を足す:
    - `DeferredRequest::Inject(inputs)` → その window が `windows` にあれば `windows.inject(window, inputs, request)`（backend が全部配った後に `ShellEvent::Injected { request }` が来て、T6.1 の 13a. で `HostEvent::InjectDelivered { window, request }` になる）。window が無ければ捨てる。
    - `DeferredRequest::Emulate(e)` → `DevtoolsHost.emulation` に入れ、全 window の環境に掛けて（`theme_hint`、`scale_factor`、`pointer.primary`）`EnvChanged` を積む。`HostEvent::EnvChanged` を作る全箇所で、emulation を掛けた環境を渡す。scale の上書きは、`set_viewport` の論理の大きさを「物理の大きさ ÷ 上書きの scale」にし、`paint.set_scale` も上書きの値にする。
    - `DeferredRequest::TimeScale(s)` → 各 window の `Model` の `frame` に `time_scale: kivact_types::TimeScale`（初期値は倍率 1）を足し、`time_scale = time_scale.rescaled(now_ns, s)` にする。`Model::tick` に渡す時刻を `time_scale.motion_ns(now)` に変換する（`model/frame/state.rs`、`ticks.rs`）。
13. 確認: `cargo check -p kivact-engine --all-targets --features testing`、`cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings`、`cargo check -p kivact-core --all-targets`、`cargo xtask check --no-write --only state-fields --crate kivact-engine --report`。

---

## 14. W11: core の devtools module の土台

| task | 担当 | crate |
|---|---|---|
| T11.1 | 1 | `crates/kivact-core`（と `crates/kivact-test`・`crates/kivact` の feature の配線） |

波の検査: `cargo check -p kivact-core --all-targets --features devtools`、`cargo check -p kivact-core --all-targets`（feature なし）、`cargo clippy -p kivact-core --all-targets --all-features -- -D warnings -A dead_code`（W12 だけが使う関数と field があるので、この波の clippy だけ `dead_code` を許す。W12 の検査から `-D warnings` のみ）、`cargo check -p kivact-test --all-targets --features devtools`、`cargo xtask check --no-write`（module の循環・module の規則・style の規則を見る。`state-fields` 以外の rule の error は、すべて直す。`state-fields` の error は、この波の task が作った・変えた型の分だけを直す（T11.1 が 0.2 の 12. で直す）。それ以外の `state-fields` の error（基準 commit の型の "combines state fields"・"static flags/counters combine"、"struct-level marker"・"unused marker"）は、この検査ではすべて無視する。T15.4 が直す。C1 の 4. と同じ扱い）。commit: `feat(core): devtools target foundation and hooks`。

### T11.1 土台、すべての hook、状態の型

担当ファイル: `crates/kivact-core/src/view/devtools_state.rs`・`devtools_apply.rs`・`devtools_hooks.rs`（新規）、`crates/kivact-core/src/view.rs`（`mod` の 3 行）、`crates/kivact-core/src/devtools.rs`、`crates/kivact-core/src/devtools/*.rs`（この task がすべての leaf file を作る。W12 が中身を書く）、hook の 1 行を入れる core の file（下の 4.）、`crates/kivact-core/src/launch/runtime.rs`・`launch/inspect.rs`・`launch/settings.rs`、`crates/kivact-core/src/view/tree.rs`（`CoreNode.style_layers`）、`crates/kivact-test/Cargo.toml`・`crates/kivact-test/src/options.rs`・`crates/kivact-test/src/screen/state.rs`（`devtools_target` の method だけ）、`crates/kivact/Cargo.toml`（`devtools` feature の転送の 1 行）、`dev/tools/check-features.sh`。

参照: 設計書 4.2.2（起動の条件）、4.3、4.4.1、4.4.5、4.4.6、4.9。

手順:

1. module の構成（設計書 4.4.1。module の参照が循環しないように）:
   - `view.rs` に `#[cfg(feature = "devtools")] pub(crate) mod devtools_state;`、`#[cfg(feature = "devtools")] pub(crate) mod devtools_apply;`、`pub(crate) mod devtools_hooks;`（**feature に関わらず常にコンパイルする**）。向きは `devtools_state` ← `devtools_apply` ← `devtools_hooks`。`devtools_state`・`devtools_apply` は `crate::view` の他の module も `crate::devtools` も `use` しない（必要な値は引数で受け取る）。
   - `devtools.rs`（`lib.rs` に `#[cfg(feature = "devtools")] mod devtools;`。T9.2 が空の file と一緒に足した）は `mod` と `pub(crate) use` だけ。leaf: `turn.rs`、`start.rs`、`target.rs`、`convert.rs`、`tree.rs`、`node.rs`、`query.rs`、`overlay.rs`、`styles.rs`、`updates.rs`、`perf.rs`、`a11y.rs`、`input.rs`、`motion.rs`、`console.rs`、`emulation.rs`、`review.rs`、`scripts.rs`。`devtools` は `view` を参照してよい。`view` は `devtools` を参照しない。`devtools/*` は `crate::launch` を参照しない。protocol の型は `use kivact_devtools_protocol as p;` で `p::Role` のように書く（kivact の型と名前がぶつかるため）。
2. `view/devtools_state.rs`（`//! Role: The DevTools state of the App's runtime, shared by the view hooks and devtools/*.`）。すべての型と field をここで作る（W12 は field を足さない）。signal は `Running` を作るときだけ作る:

   ```rust
   pub(crate) enum DevtoolsState { Off, Running(Box<Inspector>) }
   /// What `devtools::start::agent_slot` returns.
   pub(crate) enum AgentSlot { Off, Running { agent: Agent, can_inject: bool } }
   pub(crate) struct Inspector {
       pub(crate) agent: RefCell<Agent>,
       pub(crate) can_inject: bool,      // false on embed targets: inject_input is refused with UNSUPPORTED
       /// What the view hooks read and write.
       pub(crate) hooks: HookData,
       /// What only devtools/* uses.
       pub(crate) session: SessionData,
   }
   pub(crate) struct HookData {
       pub(crate) recording: RefCell<p::RecordingSettings>,           // read untracked by the hooks
       pub(crate) prop_writes: RefCell<Vec<PropWrite>>,                // this turn's prop writes while recording
       pub(crate) overrides: Signal<Vec<p::StyleOverride>>,
       pub(crate) forced: Signal<BTreeMap<NodeId, ForcedStates>>,
       pub(crate) emulation: Signal<p::Emulation>,
       pub(crate) focus_history: RefCell<BTreeMap<WindowId, VecDeque<p::FocusMove>>>, // at most 100 per window
       pub(crate) programmatic_focus: LocalMailbox<FocusTarget>,      // the target of the last focus request
       pub(crate) input_log: RefCell<VecDeque<p::InputLogEntry>>,     // at most 500
       /// The last `InputLogEntry.seq` given (S-8 id allocator; 1-based, in log order).
       pub(crate) next_input: Cell<u64>,
   }
   pub(crate) struct SessionData {
       pub(crate) events: RefCell<Vec<DevtoolsEvent>>,                 // engine events queued by `devtools_hooks::host_event`, handled by `devtools::run_events`
       pub(crate) pending: RefCell<Vec<Pending>>,                      // find_*, list_turns(wait_ms) and inject_input waiting
       pub(crate) pending_graphs: RefCell<VecDeque<PendingGraph>>,     // graphs waiting for their frame
       pub(crate) turns: RefCell<VecDeque<p::TurnGraph>>,              // completed graphs, at most 1000
       pub(crate) frames: RefCell<BTreeMap<WindowId, VecDeque<p::FrameInfo>>>, // at most 3000 per window
       pub(crate) update_counts: RefCell<HashMap<NodeId, u32>>,        // per-node prop changes since recording began
       // xtask-allow(state-fields): S-8 ID - style override id allocator.
       pub(crate) next_override: Cell<u64>,
       // xtask-allow(state-fields): independent fact - the node the DevTools user highlights (with the layout overlay choice).
       pub(crate) highlight: Cell<Option<Highlight>>,
       // xtask-allow(state-fields): independent fact - the window in pick mode; the core owns it and the engine holds an applied copy (design 4.4.1).
       pub(crate) pick: Cell<Option<WindowId>>,
       pub(crate) overlays: RefCell<p::OverlaySettings>,
       pub(crate) sent_overlays: RefCell<BTreeMap<WindowId, DevtoolsOverlay>>, // the overlay last given to each window (S-9 Sent); `overlay::after_turn` compares with it
       pub(crate) snapshots: RefCell<VecDeque<(u64, LiveSnapshot)>>,   // at most 10
       // xtask-allow(state-fields): S-8 ID - snapshot id allocator.
       pub(crate) next_snapshot: Cell<u64>,
       pub(crate) sent_tree: RefCell<HashMap<NodeId, p::ElementInfo>>, // the tree last sent to Tree subscribers (S-9 Sent)
       pub(crate) sent_windows: RefCell<Sent<Vec<p::WindowInfo>>>,    // the window list last sent to Windows subscribers
       pub(crate) errors_sent: RefCell<Sent<u64>>,                     // the last error seq notified
       pub(crate) open_failed: ReportOnce,                             // "DevTools app not found" reported once
   }
   #[derive(Clone, Copy)]
   pub(crate) struct Highlight { pub(crate) node: NodeId, pub(crate) layout: bool }
   pub(crate) struct PropWrite {
       pub(crate) node: NodeId,
       pub(crate) prop: Prop,
       // xtask-allow(state-fields): passive record - the effect that wrote the prop; known only for effect-driven writes
       pub(crate) effect: Option<EffectId>,
       pub(crate) changed: bool,
   }
   ```

   state-fields の印は `// xtask-allow(state-fields): <分類> - <その field が何の事実か>` の形（分類は付録 7.4 の一覧: `S-8 ID`・`independent fact`・`setting`・`passive record` など）で、field の `///` の行より**上**に書く。数えられない field（`Sent`・`ReportOnce`・ring・`RefCell<Sent<u64>>` の `errors_sent` など）には印を付けない（付けると「unused marker」の error になる）。

   `NodeId` は `kivact_types::NodeId`、`WindowId` は `kivact_types::WindowId`、`EffectId` は `kivact_runtime::NodeId`、`Prop` は `PropValue::prop()` の型。`PropWrite` の印は、数えられる field が 2 つ（`effect`・`changed`）なので `effect` の直前に置く（上の複数行の形。構造体の前に置くと "struct-level marker" の error になる）。`Pending`・`QueryBy`・`PendingGraph`・`DevtoolsEvent` は設計書 4.9 のとおりにこの file に置く（`Pending::Find`・`Inject` の `window` は `kivact_types::WindowId`）。`Pending` に `fn window(&self) -> Option<WindowId>`（`Find`・`Inject` は `Some`、`Turns` は `None`）を足す。`DevtoolsState::of(rt: &Runtime) -> &DevtoolsState`（他の extension の `of` と同じ形: `rt.ext::<DevtoolsState>()`）、`DevtoolsState::inspector(rt) -> &Inspector`（`Running` なら中身、`Off` なら `unreachable!("invariant: a call arrives only while the agent runs")`。`devtools/*` の method の処理が使う）、`DevtoolsState::from_slot(root: Owner, slot: AgentSlot, at: &'static Location<'static>) -> DevtoolsState`（`Off` は `Off`。`Running` は `root.signal(Vec::new(), at)`・`root.signal(BTreeMap::new(), at)`・`root.signal(emulation_off(), at)` を作って `Inspector` を組む）、`Inspector::wants(&self, topic) -> bool`・`Inspector::notify(&self, n: p::Notification)`（どちらも `self.agent.borrow()` を 1 つの文の中で使い、借用を返す）を書く。protocol の型は `Default` を持たない（設計書 4.2.3 の 1.）ので、既定の値を返す関数を同じ file に置く: `pub(crate) fn emulation_off() -> p::Emulation`（`p::Emulation { theme: None, reduced_motion: None, locale: None, pseudo_locale: p::PseudoLocale::Off, scale: None, touch: None }`）、`pub(crate) fn recording_off() -> p::RecordingSettings`（3 つとも false）、`pub(crate) fn overlays_off() -> p::OverlaySettings`（4 つとも false）。`p::Emulation::default()` などは書かない。`HookData.recording` は `RefCell::new(recording_off())`、`SessionData.overlays` は `RefCell::new(overlays_off())` で始める。
3. `start.rs`、launch の変更、設定:
   - `start::agent_slot(title: &str, allowed: bool, can_inject: bool, waker: ThreadWaker) -> AgentSlot`（`devtools/*` は `crate::launch` を参照しない）。`allowed` が false なら `AgentSlot::Off`。条件（設計書 4.2.2）: 環境変数 `KIVACT_DEVTOOLS` が `0` ではなく、実行 file の名前（拡張子を除く）が `kivact-devtools` ではない（`allowed` は launch が `ClientMode::Window`・`Embedded` か、`Test` で `devtools_in_tests` かで求めて渡す）。満たせば `Agent::start(AgentConfig { app }, Box::new(move || waker.wake()))` する。`app` は `p::AppInfo { name: title.to_owned(), pid: u64::from(std::process::id()), kivact_version: env!("CARGO_PKG_VERSION").into(), build: if cfg!(debug_assertions) { p::BuildKind::Debug } else { p::BuildKind::Release }, exe: current_exe の文字列, cwd: current_dir の文字列, workspace_root: String::new() }`（`workspace_root` は agent が求めて埋める）。成功なら `AgentSlot::Running { agent, can_inject }`、失敗は `tracing::warn!` して `Off`。release ビルド（`!cfg!(debug_assertions)`）なら設計書 4.4.1 の警告を出す。
   - `start::engine_keys(config: &DevtoolsConfig) -> DevtoolsKeys`: `config.open_keys()`・`config.pick_keys()`（T1.2。既定は適用済みで、`[]` は無効）の各 chord を `convert::parse_chord` で変換する。読めない chord は `tracing::warn!` を出してその 1 つだけ捨てる。
   - `launch/runtime.rs`（`Launch::start`、113-150 行）: `Runtime::new` の前に `let thread_waker = ThreadWaker::new(move || ready.wake());` を作り、`Runtime::new(.., thread_waker.clone(), ..)` に渡す。feature のとき、`Runtime::new` の前に `let slot = crate::devtools::start::agent_slot(&options.settings.title, self.mode.devtools_allowed(options.settings.devtools_in_tests), !matches!(self.mode, ClientMode::Embedded(_)), thread_waker.clone());` を求め、build closure の `Extensions` に `.with(DevtoolsState::from_slot(root, slot, self.at))` を足す（`#[cfg(feature = "devtools")]` を付けた文で）。`ClientMode` に `fn devtools_allowed(&self, in_tests: bool) -> bool`（`Window | Embedded(_)` は true、`Test { .. }` は `in_tests`。SM-CORE-14 で `ClientMode::Test { timing, name }` の struct variant になっている）を足す。`Launch::start` の最後で、`DevtoolsState::of(&rt)` が `Running` なら `ViewState::tree(&rt).engine().set_devtools_keys(start::engine_keys(i.agent.borrow().config()))`。
   - `launch/runtime.rs`（`Launch::turn`）の呼び出し 4 か所（`cfg(feature = "devtools")` を付ける。ほかの cfg は launch には無い）:
     1. `rt.advance_clock(now_ns);`（197 行）の直後に `#[cfg(feature = "devtools")] let incoming = crate::devtools::take_incoming(&rt);` と、「event が無ければ早く返す」判定（201-205 行）に `#[cfg(feature = "devtools")] let idle = idle && incoming.is_empty();` を足す（判定を `let idle = events.is_empty() && boundaries.last().is_some_and(..);` に出して `if idle { return output(rt.report_now()); }` にする）。
     2. `acknowledge(0);` の後、event を配る `for event in events` の前に `#[cfg(feature = "devtools")] crate::devtools::run_incoming(&rt, &turn, incoming);`。
     3. event を配り終えた後（`acknowledge(events.len());` の直後）に `#[cfg(feature = "devtools")] crate::devtools::run_events(&rt);`（hook が積んだ `DevtoolsEvent` を root owner の handler として処理する）。
     4. `let output = output(turn.finish(writes, requests::has_activations(&rt)));`（254 行）を `let report = turn.finish(writes, requests::has_activations(&rt)); #[cfg(feature = "devtools")] crate::devtools::after_turn(&rt); let output = output(report);` に分ける（`finish` が返った直後に `after_turn`。`rt.turns().last()` と `take_turn_details()` がこの turn の記録になる）。
   - 試験の App で DevTools を有効にする設定（設計書 4.2.2。`devtools` feature と reader が同じ wave にあるので、ここで足す）: `launch/settings.rs` の `AppSettings` に `#[cfg(feature = "devtools")] pub(super) devtools_in_tests: bool,`（field の前に `// xtask-allow(state-fields): setting - an independent opt-in a test sets to start the DevTools agent`。`AppOptions::new` の初期値は false）と、`#[cfg(feature = "devtools")] #[doc(hidden)] pub fn devtools_in_tests(mut self) -> Self`（`AppOptions` の method。直前に `// xtask-allow(internal-contract): kivact::test starts the DevTools agent for a test App`）。`crates/kivact-test/src/options.rs` の `RenderOptions` に `#[cfg(feature = "devtools")] pub fn devtools(mut self) -> Self { self.app = self.app.devtools_in_tests(); self }`（doc: `/// Start the DevTools agent for this test App.`。`gpu_preference` と同じ形）。`feature` の無い crate に `cfg(feature = "devtools")` を書くと `unexpected_cfgs` になるので、この task より前には書かれていない。
4. hook（`view/devtools_hooks.rs`。`//! Role: Hooks the view calls so the DevTools can read and shape what the App does.`）。表の関数は**すべて**この file にあり、feature の有無で同じ signature を持つ。本体は `#[cfg(feature = "devtools")]` の版と `#[cfg(not(feature = "devtools"))]` の版の 2 つ（下の「無し」の列）。呼び出し側に `cfg` を書かない（`launch` の 4 か所と `bind_style` の層の保存を除く。`client/*` は `crate::devtools` を参照せず、hook だけを呼ぶ）。形の例:

   ```rust
   /// Record a prop write while recording.
   #[cfg(feature = "devtools")]
   pub(crate) fn prop_written(id: NodeId, prop: Prop, writer: Option<EffectId>, changed: bool) {
       let Some(rt) = Runtime::current() else { return };
       let DevtoolsState::Running(i) = DevtoolsState::of(&rt) else { return };
       if !i.hooks.recording.borrow().updates { return; }
       i.hooks.prop_writes.borrow_mut().push(PropWrite { node: id, prop, effect: writer, changed });
   }
   #[cfg(not(feature = "devtools"))]
   pub(crate) fn prop_written(_: NodeId, _: Prop, _: Option<EffectId>, _: bool) {}
   ```

   | 場所 | 呼び出し | 本体（あり） | 無し |
   |---|---|---|---|
   | `client/dispatch/delivery.rs` の `dispatch_host` の match | `DevtoolsKey`・`DevtoolsPick`・`InjectDelivered` → `devtools_hooks::host_event(rt, host_event)`（T10.1 と T7.1 の何もしない arm を置き換える。`client` は `crate::devtools` を呼ばない） | event を `DevtoolsEvent`（設計書 4.9）にして `i.session.events` に積む（`DevtoolsKey { window, pick }` → `Key`、`DevtoolsPick { window, node, phase }` → `Pick`、`InjectDelivered { request, .. }` → `InjectDelivered`） | 何もしない |
   | `client/dispatch/delivery.rs` の `dispatch` の `FocusChanged` の処理の後 | `devtools_hooks::focus_changed(rt, window, target: Option<FocusTarget>, visible)` | `focus_history[window]` に `p::FocusMove { at_ns: rt.now_ns(), before: 最後の after, after: target の node, reason }`。`reason`: `programmatic_focus.take()` が `target` と一致すれば `Programmatic`、そうでなく `visible` なら `Keyboard`、それ以外は `Pointer`。記録中（`recording.input`）だけ | 何もしない |
   | 同じ `dispatch` の `FocusResult` の処理の後 | `devtools_hooks::focus_result(rt)` | `programmatic_focus.take()`（中身は使わない。focus が動かない request で値が残らないように。`FocusChanged` は `FocusResult` より先に来る） | 何もしない |
   | `dispatch` の Pointer・Wheel・Key・Ime・File の処理の前 | `devtools_hooks::input_event(rt, event: &HostEvent)` | 記録中（`recording.input`）なら `p::InputLogEntry { seq: 次の `next_input`（1 増やした値。1 始まり）, at_ns: rt.now_ns(), window, kind, detail, source, target, route: vec![], handlers: vec![] }` を `input_log` に積む（500 件まで）。`kind` は `"pointer"`・`"wheel"`・`"key"`・`"ime"`・`"file"`、`detail` は短い説明、`source` は、その window に `Pending::Inject` が残っている間は `Injected`、それ以外は `Platform` | 何もしない |
   | `client/dispatch/key.rs` の bubble の loop の handler の後、`pointer.rs` の activate と pointer の handler の後、`a11y.rs` の handler の後 | `devtools_hooks::handler_ran(rt, node, "<on_key などの event 名>", outcome)` | 記録中なら直前の log の entry の `handlers` に足し、`route` の最後と違う node なら `route` に足す | 何もしない |
   | `view/requests.rs:143` の `flush_focus` で focus を送るとき | `devtools_hooks::focus_requested(rt, target)` | `programmatic_focus.put(target)` | 何もしない |
   | `view/tree.rs` の `ViewNodes::update`（603 行）の最後 | `devtools_hooks::prop_written(id, prop, writer, changed)`（`Runtime::current()` で state を得る） | 上の例 | 何もしない |
   | `view/mount/style.rs` の `bind_style`（`let owner = cx.owner;` の後、effect の前） | `let hook = StyleHook::new(cx.rt, cx.owner, id, at);`。effect の中で `let s = hook.apply(merge_layers(&layers));` と `Interactive { forced: hook.forced(), .. }`（`merge_layers` の直後・122 行、`Interactive` の literal・123 行） | 手順 5 | `StyleHook` は何も持たない型。`apply` は入力をそのまま返し、`forced` は `ForcedStates::default()` |
   | 同じ `bind_style` の `let NodeStyle { layers, .. } = style;` の直後 | `#[cfg(feature = "devtools")] if devtools_hooks::is_running(cx.rt) { ViewState::tree_mut(cx.rt).set_style_layers(id, layers.clone()); }`（この 1 か所だけ call site に `cfg` を書く。`CoreNode` に `#[cfg(feature = "devtools")] pub(crate) style_layers: Vec<StyleLayer>`（既定は空）と `ViewNodes::set_style_layers(&mut self, id, layers: Vec<StyleLayer>)` を足す） | `is_running(rt: &Runtime) -> bool` は `matches!(DevtoolsState::of(rt), DevtoolsState::Running(_))` | `false`（呼び出し自体が `cfg` で消える） |
   | `view/root.rs` の theme の install（68 行）、`view/boundary.rs` の `provide_theme`（105 行） | `let theme = devtools_hooks::wrap_theme(owner, theme);`（`Value<Theme>` を受け取り返す） | `owner.with_setup(\|\| memo(move \|\| apply_token_overrides(theme.get(), &overrides.get())))`（`overrides` は `Running` の signal を `Runtime::current()` から得る） | 入力のまま |
   | `view/root.rs` の root locale（52-60 行） | `let locale = devtools_hooks::wrap_locale(owner, locale);`（`Value<Locale>`） | `owner.with_setup(\|\| memo(move \|\| match emulation.get().locale { Some(tag) => Locale::new(&tag), None => locale.get() }))` | 入力のまま |
   | `view/boundary.rs` の `message`（335 行）・`message_with`（376-387 行）の memo の中、文字列を決めた後 | `let text = devtools_hooks::pseudo(text);`（`String` を受け取り返す） | `emulation` の `pseudo_locale` を tracked で読み、`Off` 以外なら `devtools_apply::pseudo_locale(mode, &text)` | 入力のまま |
   | `view/state.rs` の `ViewState::new` の `capabilities`（T7.1 が `memo(|| host_capabilities.get())` で作った式） | `let capabilities = devtools_hooks::wrap_capabilities(owner, host_capabilities);`（`Signal<Capabilities>` を受けて `Value<Capabilities>` を返す。T7.1 の memo の行をこれに置き換える） | **この hook だけは入口で `DevtoolsState::of` を読まない**（`ViewState::new` は `Runtime::new` の build closure の中、extension を入れる前に呼ばれ、そこで `Runtime::ext` を読むと "extension read while the App is being constructed" で panic する）。`owner.with_setup(\|\| memo(move \|\| { let mut c = host.get(); if let Some(rt) = Runtime::current() && let DevtoolsState::Running(i) = DevtoolsState::of(&rt) && let Some(t) = i.hooks.emulation.get().touch { c.touch = t; } c }))`（memo は最初に読まれたときに評価され、そのときには `DevtoolsState` が入っている。`emulation` は memo の中で読むので購読される） | `owner.with_setup(\|\| memo(move \|\| host.get()))`（T7.1 の式と同じ） |

   `StyleHook` は、feature があるとき `enum StyleHook { Off, On { overrides: Value<Vec<(String, String)>>, forced: Value<ForcedStates> } }`（`Copy`）、無いとき `struct StyleHook;`（`Copy`）。`new(rt: &Rc<Runtime>, owner: Owner, id: NodeId, at: &'static Location<'static>) -> StyleHook`（あり: `Running` でなければ `Off`。`Running` なら `let sites = devtools_apply::call_sites(rt, owner, Some(at));` を一度求め、`owner.with_setup(|| StyleHook::On { overrides: memo(move || devtools_apply::node_overrides(&overrides.get(), id, &sites)), forced: memo(move || forced.with(|f| f.get(&id).copied().unwrap_or_default())) })`）。`apply(&self, s: Style) -> Style`（`On` なら `overrides.get()` の `(property, value)` を順に `devtools_apply::override_style(..)` で `Style` にして `s.merge(&o)`。読めない組は飛ばす）、`forced(&self) -> ForcedStates`。`Value<Vec<(String, String)>>` の memo は `PartialEq` で変化を判定するので、編集した node 以外の style effect は再実行されない。
5. `view/devtools_apply.rs`（`//! Role: Apply DevTools overrides and emulation to styles, themes and texts.`）:
   - `override_style(property: &str, value: &str) -> Result<Style, String>`（設計書 4.4.5 の表。`Style` の同名の setter を使う。値の読み方の関数: `length(value) -> Result<Length, String>`（`auto` は `Length::Auto`、`%` で終われば `Length::Percent`、それ以外は `f64` に読んで `Length::Px`）、`number(value) -> Result<f64, String>`、`color(value) -> Result<Color, String>`（`ColorToken` の名前に一致すればそれ、でなければ `Color::css(value)`）。読めないときの message は `format!("{property}: expected {what}, got {value}")`。80 行を超えないよう、layout の property・paint の property・text の property で `fn override_layout(..)`・`override_paint(..)`・`override_text(..)`（どれも `Option<Result<Style, String>>`）に分け、`override_style` は 3 つを順に試す。どれも合わなければ `Err("unknown property: <property>")`）。
   - `node_overrides(all: &[p::StyleOverride], id: NodeId, sites: &[&'static Location<'static>]) -> Vec<(String, String)>`: scope が `CallSite { location }` で `sites` のどれかと file・line・column が一致するものを先に、`Node { node }` で `node == id` のものを後に、`(property, value)` の列にする（後ろが勝つ）。
   - `call_sites(rt: &Runtime, owner: Owner, at: Option<&'static Location<'static>>) -> Vec<&'static Location<'static>>`: `at` を先頭に、`rt.owner_info(owner.id())` を親へ辿って `created_at` を並べる（同じ位置が続くときは一つにまとめる）。`node::call_chain` もこれを使う（設計書 4.2.4 の `call_chain` と同じ並び）。
   - `apply_token_overrides(theme: Theme, all: &[p::StyleOverride]) -> Theme`: scope が `Token { token }` のものを順に当てる。token の名前と `Theme` の `with_*` の対応: `colors.<ColorToken の snake_case>`（`with_colors`）、`space.xs|sm|md|lg|xl`（`with_space`）、`radius.sm|md|lg`（`with_radius`）、`text.size|size_small|size_large|weight|weight_strong|line_height`（`with_text`）、`focus_ring`、`focus_ring_offset`、`disabled_opacity`、`control_height`、`overlay_margin`（それぞれ `with_<同名>`）。読めない値は飛ばす（検査は `styles::set_style_override` が `-32602` で済ませている）。`token_list(theme: &Theme) -> Vec<p::ThemeToken>`（`list_theme_tokens` が使う、同じ名前の一覧と値）もここに置く。
   - `pseudo_locale(mode: p::PseudoLocale, text: &str) -> String`（設計書 4.2.4 の 3 つの規則。`{name}` の置換位置は変えない: `{` から `}` までは変換しない）。
6. `target.rs` と各 leaf の `route`: `target.rs` に次を書く（`CoreTarget` は作らない）:

   ```rust
   type Route = fn(&Runtime, p::Call, Responder) -> Option<(p::Call, Responder)>;
   const ROUTES: [Route; 14] = [tree::route, node::route, query::route, overlay::route, styles::route,
       updates::route, perf::route, a11y::route, input::route, motion::route, console::route,
       emulation::route, review::route, scripts::route];
   /// Answer one call: the first leaf that owns its method takes it.
   pub(crate) fn dispatch(rt: &Runtime, call: p::Call, reply: Responder) {
       let mut left = Some((call, reply));
       for route in ROUTES {
           let Some((call, reply)) = left.take() else { return };
           left = route(rt, call, reply);
       }
       if let Some((_, reply)) = left {
           reply.send(Err(p::RpcError::new(p::code::INTERNAL_ERROR, "handled by the agent")));
       }
   }
   ```

   各 leaf は T11.1 が `route` を全部書く（W12 は `route` を変えない）。形は 2 つだけ:
   - すぐ答えるもの: `pub(super) fn <method>(rt: &Runtime, params: P) -> Result<R, p::RpcError>`。`route` の arm は `p::Call::GetNode(p) => reply.send(get_node(rt, p).map(p::Reply::GetNode)),`。
   - 後で答えるもの（`find_*`、`screenshot`、`inject_input`、`add_comment`）: `pub(super) fn <method>(rt: &Runtime, params: P, reply: Responder)`。`route` の arm は `p::Call::FindByRole(p) => find_by_role(rt, p, reply),`。
   - `route` の最後の arm は `other => return Some((other, reply)),`、match の後は `None`。`Call`・`Reply` の variant 名は wire 名を `_` で切って各語の先頭だけを大文字にしてつなぐ（`get_a11y_tree` → `GetA11yTree`）。

   method と module の対応（`Initialize`・`Subscribe`・`GetConfig` は agent が答えるので、どの `route` も取らず `dispatch` の最後に残り、`INTERNAL_ERROR`（"handled by the agent"）で答える。他の stub は `UNSUPPORTED`（"not implemented yet"））:

   | module | method |
   |---|---|
   | `tree.rs` | `get_app_info`、`list_windows`、`get_tree` |
   | `node.rs` | `get_node`、`get_owners` |
   | `query.rs` | 12 の query（`get_by_role`・`query_by_role`・`find_by_role`・`all_by_role`、`_by_text`、`_by_test_id`。`find_*` は後で答える） |
   | `overlay.rs` | `highlight`、`set_overlays`、`get_overlays`、`start_pick`、`stop_pick`、`screenshot`（後で答える） |
   | `styles.rs` | `set_style_override`、`clear_style_overrides`、`list_style_overrides`、`force_state`、`list_theme_tokens` |
   | `updates.rs` | `start_recording`、`stop_recording`、`get_recording`、`list_turns`（`wait_ms` があれば後で答える） |
   | `perf.rs` | `list_frames`、`get_counters`、`take_snapshot`、`diff_snapshots`、`export_trace` |
   | `a11y.rs` | `get_a11y_tree`、`audit_a11y`、`get_focus`、`get_tab_order`、`lint_layout` |
   | `input.rs` | `inject_input`（後で答える）、`list_inputs`、`get_ime_state` |
   | `motion.rs` | `list_animations`、`set_time_scale`、`list_tasks` |
   | `console.rs` | `list_errors` |
   | `emulation.rs` | `get_emulation`、`set_emulation` |
   | `review.rs` | `get_review`、`add_comment`（後で答える）、`edit_comment`、`remove_comment`、`export_review` |
   | `scripts.rs` | `show_table` |

   各関数は、この task では引数を使い切って（`let _ = (rt, params);`）`Err(p::RpcError::new(p::code::UNSUPPORTED, "not implemented yet"))` を返す（後で答えるものは `let _ = (rt, params); reply.send(Err(..));`）。ただし `tree.rs` の 3 つはこの task で実装する（下の 8.）。method 以外に、W12 が中身を書く次の関数も、引数を `let _ = (..);` で使い切る空の本体で作る（W12 は自分の file の中の本体だけを書く。signature を変えない）:

   | module | 関数 | 空の本体 |
   |---|---|---|
   | `styles.rs` | `pub(super) fn layers(rt: &Runtime, id: NodeId) -> Vec<p::StyleLayerInfo>`、`pub(super) fn clear_all(rt: &Runtime)`（override と forced を空にする） | `Vec::new()`／何もしない |
   | `overlay.rs` | `pub(super) fn pick_event(rt: &Runtime, window: WindowId, node: Option<NodeId>, phase: PickPhase)`、`pub(super) fn start_pick_in(rt: &Runtime, window: WindowId)`、`pub(super) fn after_turn(rt: &Runtime, records: &EngineRecords)`、`pub(super) fn clear(rt: &Runtime)`（pick・highlight・重ね表示を止める） | 何もしない |
   | `query.rs` | `pub(super) fn after_turn(rt: &Runtime)`（待っている `find_*`） | 何もしない |
   | `input.rs` | `pub(super) fn delivered(rt: &Runtime, request: RequestId)`（`InjectDelivered` を受けたとき） | 何もしない |
   | `updates.rs` | `pub(super) fn after_turn(rt: &Runtime, records: &EngineRecords)`、`pub(super) fn stop(rt: &Runtime)`（残った graph を完成させて記録を止める。`stop_recording` と reset が使う） | 何もしない |
   | `console.rs` | `pub(super) fn after_turn(rt: &Runtime)` | 何もしない |
   | `motion.rs` | `pub(super) fn reset_time_scale(rt: &Runtime)` | 何もしない |
   | `emulation.rs` | `pub(super) fn clear(rt: &Runtime)` | 何もしない |
   | `node.rs` | `pub(super) fn call_chain(rt: &Runtime, id: NodeId) -> Vec<p::CallFrame>`、`pub(super) fn states(rt: &Runtime, id: NodeId) -> Vec<p::InteractiveState>` | 空の Vec |

7. `turn.rs`（T11.1 が全部書く）:
   - `pub(crate) fn take_incoming(rt: &Runtime) -> Vec<Incoming>`: `match DevtoolsState::of(rt) { Off => Vec::new(), Running(i) => i.agent.borrow_mut().take_incoming() }`（借用はこの文で終わる）。
   - `pub(crate) fn run_incoming(rt: &Runtime, turn: &TurnGuard<'_>, incoming: Vec<Incoming>)`（設計書 4.4.1）:

     ```rust
     for item in incoming {
         match item {
             Incoming::Call(call, reply) => {
                 turn.stage(Stage::Dispatch);
                 turn.cause_with(CauseTag::Devtools, format!("devtools {}", call.method()));
                 rt.root().run_handler(|| target::dispatch(rt, call, reply));
             }
             Incoming::Closed { remaining: 0 } => rt.root().run_handler(|| reset(rt)),
             Incoming::Closed { .. } => {}
         }
     }
     turn.stabilize();
     ```

   - `fn reset(rt)`: `overlay::clear(rt)`、`styles::clear_all(rt)`、`emulation::clear(rt)`、`motion::reset_time_scale(rt)`、`updates::stop(rt)` をこの順に呼ぶ（review は残す）。
   - `pub(crate) fn apply_recording(rt: &Runtime)`: `recording` と `overlays` の今の値から、`rt.set_detailed_trace(recording.updates)`、`Writer::set_engine_recording(recording.updates || overlays.paint_flashing, recording.updates || recording.frames)` を呼ぶ（`start_recording`・`stop`・`set_overlays`・reset が呼ぶ。原因グラフの `Frame` の node に `FrameRecord` が要るので、`updates` のときも frames を記録する）。
   - `pub(crate) fn after_turn(rt: &Runtime)`: `Off` なら何もしない。この順に呼ぶ: `sweep_closed_windows`、`tree::notify_windows_changed(rt)`、`tree::notify_tree_changes(rt)`、`let records = ViewState::tree(rt).reader().take_engine_records();`、`query::after_turn(rt)`、`updates::after_turn(rt, &records)`、`overlay::after_turn(rt, &records)`、`console::after_turn(rt)`。
   - `fn sweep_closed_windows(rt: &Runtime, i: &Inspector)`: 開いている window は `ViewState::tree(rt).reader().windows()`。`i.session.pending` を `std::mem::take` して、`Pending::window()` が閉じた window のものを `reply.send(Err(p::RpcError::new(p::code::WINDOW_NOT_FOUND, "the window closed")))` で答え（`Turns` は残す）、残りを戻す。`pick` がその window なら `None` にして `Writer::set_pick(None)`。`frames`・`focus_history`・`sent_overlays`・`sent_tree`（`ElementInfo.window` が閉じた window のもの。`tree_changed` は出さず捨てる。閉じたことは `windows_changed` が伝える）の entry を消す。
   - `pub(crate) fn run_events(rt: &Runtime)`: `Off` なら何もしない。`i.session.events` を `std::mem::take` して、一つずつ `rt.root().run_handler(|| match event { .. })` の中で処理する。`DevtoolsEvent::Key { window, pick }` → `pick` なら先に `overlay::start_pick_in(rt, *window)`、それから `let opened = i.agent.borrow_mut().open_devtools();`（借用はこの文で終わる）、`Err(message)` なら `i.session.open_failed.note()` が true のとき一度だけ `rt.report(Some(rt.root()), UiError::new(ErrorKind::Platform, message, None))`。`DevtoolsEvent::Pick { window, node, phase }` → `overlay::pick_event`。`DevtoolsEvent::InjectDelivered { request }` → `input::delivered(rt, request)`。
   - agent を使う関数は、どれも借用を 1 つの文の中で終える（`i.notify(..)`・`i.wants(..)` を使う）。借用したまま他の関数を呼ばない。
8. `convert.rs` と `tree.rs`（他の module が使う土台）:
   - `convert.rs`: `role(kivact_types::Role) -> p::Role`（全 variant の `match`）、`role_from(p::Role) -> kivact_types::Role`、`location(&'static Location) -> p::SourceLocation`、`rect(kivact_types::Rect) -> p::Rect`、`node_kind(NodeType) -> p::NodeKind`、`window(WindowId) -> u64` と `window_from(u64) -> Option<WindowId>`（`crates/kivact-types/src/window.rs` の `WindowId` の `get`／`new` に合わせる）、`mouse_button(p::MouseButton) -> kivact_types::MouseButton`、`a11y_action(p::A11yAction) -> kivact_types::A11yAction`（全 variant）、`a11y_request(action: p::A11yAction, value: Option<String>) -> Result<A11yRequest, p::RpcError>`（`SetValue` は `value` が無ければ `INVALID_PARAMS`、あれば `A11yRequest::SetValue(value)`。それ以外は `A11yRequest::from_action(a11y_action(action))` の `Some`。`Dismiss` は `from_action` が `Dismiss(DismissReason::Accessibility)` にする）、`forced_state(p::InteractiveState) -> ForcedStates`、`turn_stage(kivact_runtime::Stage) -> p::TurnStage`（`Stage::Setup` は turn の中に出ないが、match を網羅するため `p::TurnStage::Begin` に写す）、`parse_key(&str) -> Option<Key>`（設計書 4.2.7 の規則: `F1`〜`F24` は `Key::F(n)`、1 文字は `Key::Char(c)`、それ以外は `kivact_types::Key`（`crates/kivact-types/src/input.rs`）の unit variant の名前の表。表は unit variant をすべて並べる）、`parse_chord(&str) -> Option<KeyChord>`（`Ctrl`・`Shift`・`Alt`・`Meta` を `+` でつなぎ、最後に key。`Key::Char` は小文字にして持つ）、`app_code(file: &str) -> bool`（設計書 4.2.4 の規則）。
   - `tree.rs`: `pub(crate) fn tree_node(rt: &Runtime, id: NodeId) -> Option<p::ElementInfo>`（他の module も使う。`update_count` は `update_counts` の値、無ければ 0）、`get_app_info`（`i.agent.borrow().app().clone()`。core が `agent_slot` で作った `AppInfo` は `workspace_root` が空なので使わない）、`list_windows`（`ViewState.windows` の record と engine の env から。`bounds` は大きさだけ。`focused` は `Reader::focused_window() == Some(id)`、`mode`・`parent` は record から）、`get_tree`（その window の root から `Reader::children` で辿る。`depth` があればその深さまで）、`notify_tree_changes(rt)`（Tree を購読している connection があるときだけ。`sent_tree` と比べて差分を `tree_changed` で送り、`sent_tree` を更新する。購読者が無い turn では `sent_tree` を空にする: 次の購読者への最初の `tree_changed` は全 node の追加になる）、`notify_windows_changed(rt)`（`Topic::Windows` を購読している connection があり、`list_windows` の結果が `sent_windows` と違えば `windows_changed` を送って `sent_windows.update(..)`）。
9. PNG の符号化・base64・trace の書き出しは agent（`kivact_devtools_agent::{encode_png, base64}`、`Agent::write_trace`）と protocol（`chrome_trace`）にある。core には `png.rs` を作らない（W12 が agent の関数を呼ぶ）。
10. `launch/inspect.rs` に `#[cfg(feature = "devtools")] pub fn devtools_target(&self) -> Option<PathBuf>`（`DevtoolsState::of(self.rt())` が `Running` なら `Some(i.agent.borrow().discovery_path().to_owned())`）。`kivact-test` に feature `devtools = ["kivact-core/devtools"]` と、`#[cfg(feature = "devtools")] pub fn devtools_target(&self) -> Option<std::path::PathBuf>`（`screen/state.rs`。`inspect().devtools_target()`）。facade の `devtools` feature を `["kivact-core/devtools", "kivact-test?/devtools"]` にする。`dev/tools/check-features.sh` の `entries` に `kivact-test/devtools` を足す（この 1 行をこの task の担当に加える）。
11. 確認: `cargo check -p kivact-core --all-targets --features devtools`、`cargo check -p kivact-core --all-targets`、`cargo clippy -p kivact-core --all-targets --all-features -- -D warnings -A dead_code`、`cargo check -p kivact-test --all-targets --features devtools`、`cargo xtask check --no-write --only state-fields --crate kivact-core --report`、`cargo xtask check --no-write`（W11 の波の検査と同じ範囲: `state-fields` 以外の error はすべて直す。`state-fields` はこの task が作った・変えた型の分だけ直し、残りは T15.4 へ）。

---

## 15. W12: core の devtools module の各部

| task | 担当 | 担当ファイル（`crates/kivact-core/src/devtools/` の中だけ） |
|---|---|---|
| T12.1 | 1 | `node.rs`、`query.rs`、`overlay.rs` |
| T12.2 | 2 | `styles.rs` |
| T12.3 | 3 | `updates.rs`、`perf.rs` |
| T12.4 | 4 | `a11y.rs`、`input.rs` |
| T12.5 | 5 | `motion.rs`、`console.rs`、`emulation.rs` |
| T12.6 | 6 | `review.rs`、`scripts.rs` |

hook の本体（`view/devtools_hooks.rs`）、状態の field、`route`、空の関数は T11.1 が全部作った。W12 は自分の file だけを編集する（0.2 の 4. の W12 の例外に従い、他の担当の file のエラーを待たない）。波の検査は指揮役: T11.1 と同じ（`cargo xtask check --no-write` は W11 の波の検査と同じ範囲。clippy は `-D warnings` のみ。`-A dead_code` は付けない。W12 で全部が使われる）。commit: `feat(core): devtools target answers every method`。

すべての task に共通: 関数の引数は `rt: &Runtime`。`let i = DevtoolsState::inspector(rt);` で `Inspector` を得る（`i.hooks.<field>`・`i.session.<field>`）。返す値は protocol の型（`p::`）。engine から読むときは `ViewState::tree(rt).reader()` の method（W6・W10 で足したもの）を使う。node が無ければ `p::RpcError::new(p::code::NODE_NOT_FOUND, ..)`、window が無ければ `WINDOW_NOT_FOUND`。agent は `i.notify(..)`・`i.wants(..)` で短く借りる（T11.1 の 7.）。通知は `i.wants(topic)` が true のときだけ作る。signal の書き込み・`spawn` は、`dispatch` が root owner の handler の中で呼ぶので、そのまま使える。task の中で runtime が要るときは `Runtime::current()`。engine へ request を出すときの `RequestId` は `RequestId(ViewState::of(rt).requests.borrow_mut().request_id())`、request は `ViewState::tree(rt).engine().effect(request, HostRequest::..)`。履歴の method（`list_turns`・`list_frames`・`list_inputs`・`list_errors`）の `limit` は、`since` より後のうち新しい方から `limit` 件を、古い順に並べて返す（設計書 4.2.4 の「履歴の method の共通の規則」。`limit: 1` は最新の 1 件）。

### T12.1 node・query・overlay

1. `node.rs` `get_node`: 設計書 4.2.4 の `NodeDetail`。`call_chain`（`tree.get(id)` の `owner` と `at` から `devtools_apply::call_sites(rt, owner, at)` を求め、`p::CallFrame { location: convert::location(l), app_code: convert::app_code(l.file()) }` にする）と `states`（hover・press は `Reader::pointer_state(id)`、focus は `Reader::focused(window)`、disabled・selected は `Reader::semantics`）を、T11.1 が作った `call_chain`・`states` の関数の本体として書く（T12.6 も使う）。`styles` は `styles::layers(rt, id)`（T12.2 が書く）。`computed` は Reader の `effective_paint`・`computed_text`・`layout_props` の値を `describe` した文字列。`semantics` は `Reader::semantics`、`handlers` は `CoreNode.handlers` のうち設定されているものの名前、`why` は `diagnostics::why::explain` を protocol の型に変換（`name_hint` は agent の `name_hint`）。
   `get_owners`（設計書 4.2.5。params は `WindowParams`、結果は `OwnerTree`。同期で答える）: `rt.live_snapshot().owners` から、その window の root node の owner（`Reader::root(window)` の node の `tree.get(..).owner`。window が無ければ `WINDOW_NOT_FOUND`）の部分木を作る。他の window の root の owner になっている子 owner は、その window のものなので木に入れない。`p::OwnerNode { id: owner.id.raw(), label: 作成位置の "<file の名前>:<行>"（例 `counter.rs:12`）, location: Some(convert::location(owner.created_at)), nodes, children }`。`nodes` は `ViewState::tree(rt).ids()` を走査して、`tree.get(id)` の `owner.id()` がその owner の id のもの（node の id の昇順）。`roots` はその 1 つの owner だけ（`vec![root]`）、`children` は owner の id の昇順。深い木で stack を使い切らないよう、(親 → 子の表) を作ってから明示的な stack で post-order に組み立てる（再帰しない。80 行を超えないよう `owner_nodes`・`build_owner_tree` に分ける）。
2. `query.rs`: 12 の method。探し方は `kivact-test/src/screen/query.rs:60-188` と同じ規則（設計書 4.2.4 の query の注記）。role は `Reader::semantics` の role と name（`RoleQuery.name` が `None` なら name を比べない。`get_*`・`find_*` は 2 件以上で `MULTIPLE_MATCHES`。`Some(s)` は完全一致）、text は Text node の content、test id は `semantics.test_id`。見えている node だけ（`ViewNodes::is_visible`）。window が無ければ `WINDOW_NOT_FOUND`。`find_*`: まず探し、見つかれば答える。見つからなければ `Pending::Find { by, window, deadline_ns: rt.now_ns() + timeout_ms * 1_000_000, reply }` を `i.session.pending` に積み、`kivact_runtime::spawn_in(rt.root(), async move { kivact_runtime::sleep(Duration::from_millis(timeout_ms)).await; }, Location::caller())` で何もしない task を 1 つ起こす（静かな App でも期限で turn が起きる）。`after_turn` のたびに探し直し、見つかるか `rt.now_ns() >= deadline_ns`（`TIMED_OUT`）で答える。その window が閉じたときの答え（`WINDOW_NOT_FOUND`）は `turn.rs` の `sweep_closed_windows` が出す。
3. `overlay.rs`:
   - `highlight`（`HighlightParams { node, layout }`。node が無ければ `NODE_NOT_FOUND`。`i.session.highlight` に `Highlight { node, layout }` を入れる。`node: None` で消す）、`set_overlays`（`i.session.overlays` に入れて `turn::apply_recording(rt)`）、`get_overlays`（`Empty` → `OverlaySettings`。`i.session.overlays.borrow().clone()` を返す）、`start_pick`（`StartPickParams.window` は必須）は `start_pick_in(rt, convert::window_from(..)?)` を呼び、`start_pick_in` が `i.session.pick.set(Some(window))` と `Writer::set_pick(Some(window))`。`stop_pick`、`clear`（pick・highlight を止め、`overlays` を `devtools_state::overlays_off()` にし、全 window の `Writer::set_devtools_overlay(w, DevtoolsOverlay::default())`、`turn::apply_recording(rt)`）。
   - `pick_event`（`Hover` → highlight して `pick` を通知、`Select` → 通知して pick を止める、`Cancel` → 通知して止める。通知は `i.wants(Topic::Pick)` のとき）。
   - `after_turn(rt, records)`: その時点の highlight と設定から各 window の `DevtoolsOverlay` を作り直し、`i.session.sent_overlays` の値と違う window だけ `Writer::set_devtools_overlay` を呼んで `sent_overlays` を更新する: highlight の node は `Reader::box_model` から `OverlayBox` と `role "name" 幅×高さ` の label。`layout` が true なら、その node の children（`Reader::children`）の border の矩形を `OverlayOutline`（色 `#8e7cc3`、幅 1）にし、flex（`layout_props(node).display == Display::Flex`）なら隣り合う 2 つの child の間の隙間を `OverlayFill`（色 `#8e7cc3` α0.35）にする（`direction` が `Row`・`RowReverse` なら x 方向に `prev.x + prev.width < next.x` のとき `Rect { x: prev.x + prev.width, y: content.y, width: next.x - (prev.x + prev.width), height: content.height }`、`Column`・`ColumnReverse` なら y 方向に同様。grid のときは outline だけ）。paint_flashing は `records.syncs` の `redrawn` の node の `Reader::box_model(id).border` を outline にする（engine は damage の矩形を持たない。overlay 自身の redraw は記録に入らない）、update_heatmap は最新の turn の `changed_nodes`（`i.session.turns` の最後）から outline、tab_order は `Reader::tab_order` の順の番号の label、layout_lint は `Reader::layout_findings` の outline。
   - `screenshot`（`NodeRef::snapshot`（`view/node_ref.rs:234`）が使う crate 内部の snapshot の要求を、node（無ければ window の root）について出し、`kivact_runtime::spawn` で動かす local task の中で結果を待ち、`kivact_devtools_agent::{encode_png, base64}` で `p::Image` にして `reply.send`）。

### T12.2 styles

1. `layers(rt, id)`（設計書 4.4.5 の並び）: `tree.get(id)` の `style_layers`（`CoreNode.style_layers`。T7.2 で `origin` と `at` を持つ）を、その node の owner の下で一つずつ評価し（`owner.run_handler(|| untrack(|| pure(|| (layer.f)())))`）、`kivact_style::internal::describe_style` で property にする。recipe の層の名前は `origin` の `RecipeName` を同じ owner の下で呼んで得る。上の層で同じ property が出たら下の層の行に `overridden: true`。override の層（`Devtools`）を一番上に（`i.hooks.overrides` のうちその node に当たるもの）、state の差分（hover など）を `describe_state_styles` から `State` の層として、disabled・selected の差分をそれぞれの層として足す（forced の state には印を付ける）。`ThemeDefault` と `Inherited` は出さない。`clear_all(rt)`: `overrides` と `forced` を空にする。
2. `set_style_override`: `scope` が `Node`・`CallSite` なら `devtools_apply::override_style` で読めることを確かめ（読めなければ `-32602`）、`Token` なら `property` が `token` と同じ名前であること、`value` が `ThemeToken.kind` の書き方であることを確かめる（違えば `-32602`）。`StyleOverride { id: next_override を 1 増やした値, .. }` を `i.hooks.overrides` に足して答える。`clear_style_overrides`、`list_style_overrides`。
3. `force_state`（`i.hooks.forced`。node が無ければ `NODE_NOT_FOUND`。`states: []` はその node の entry を消す。`convert::forced_state` で bit にする）。
4. `list_theme_tokens`（main window の root の theme から、`devtools_apply::token_list`）。

### T12.3 updates と perf

1. `start_recording`: 記録していない状態から呼ばれたら `turns`・`frames`・`pending_graphs`・`update_counts`・`prop_writes` を空にする（記録中に呼んだら設定だけを置き換える）。`i.hooks.recording` に入れて `turn::apply_recording(rt)`。`stop(rt)`: (1) `Reader::take_engine_records()` を取って待っている graph に足し、(2) `pending_graphs` の残りをすべて、その時点の node（`Frame` の node 無し）で完成させて `turns` に移し `turn_recorded` を通知し、(3) `list_turns` の `Pending::Turns` を確かめ、(4) 記録の設定を `devtools_state::recording_off()` にして `turn::apply_recording(rt)`。`stop_recording` は `stop(rt)` を呼ぶ。`get_recording`（`i.hooks.recording` の値）。`list_turns`（`ListTurnsParams { since, limit, wait_ms }`）: `since` より新しい完成した graph を `limit` 件まで返す。無く、`wait_ms` があれば `Pending::Turns { params, deadline_ns, reply }` を積み、`find_*` と同じ何もしない timer の task を起こす（`updates::after_turn` が graph の完成か期限で答える）。
2. `updates::after_turn(rt, records)`（設計書 4.2.4 の「原因グラフの作り方」）: 記録中（`recording.updates`）なら (1) `records`（turn.rs が取った `EngineRecords`）を使う。`pending_graphs` の各 graph について、revision が同じ `SyncRecord`（複数あってよい）から `Layout`／`Paint` の node と edge を足し、revision が同じ `FrameRecord` があれば `Frame` の node を足して完成させる。同じ revision の `FrameRecord` が無くても、より新しい revision の `SyncRecord` か `FrameRecord` があれば完成させる（その revision では frame を描かなかった）。完成した graph は `turns` に移して `turn_recorded` を通知する。frame の記録は `frames` に入れ `frame_recorded` を通知する。(2) `rt.take_turn_details()` の最後の `TurnDetail`（`after_turn` は `turn.finish` の後に呼ばれるので、この turn の記録）と `prop_writes`（取り出して空にする）から、手順の 1〜3・5〜8 で今の turn の graph を作り、今の revision と一緒に `pending_graphs` に積む。cause の node は `CauseEntry.tag` から `CauseKind` を作る（文字列は解釈しない）。`cause: None` の write・run は cause の無い root の node にする。`TurnGraph.started_ns` は `TurnDetail.stages` の最初の `start_ns`（無ければ `rt.now_ns()`）、`duration_ns` は `TurnDetail.duration_ns`。`truncated` は node が 2000 を超えたとき、または `TurnDetail.dropped > 0`。`prop_writes` の `changed: true` 1 件ごとに、その node の `update_counts` を 1 増やす。`Pending::Turns` の確認は記録中かどうかに関わらず毎回行い、`since` より新しい完成した graph があればそれで、`rt.now_ns() >= deadline_ns` なら空の `TurnList` で答える。
3. `perf.rs`: `list_frames`（`ListFramesParams { window, since, limit }`。frame の cursor `since` は `FrameInfo.end_ns`: `end_ns > since` の frame を `limit` 件まで返す。`FrameInfo.end_ns` は `FrameRecord.end_ns` をそのまま写す）、`get_counters`（`Inspect::stats` と同じ式を直接計算する: `live = rt.live_counts()`、`MotionState::of(rt).stats()`、`TaskState::of(rt).stats()`、`ViewState::tree(rt).len()`、`HostState::of(rt).turns()` から `Counters { live_reactive: live.reactive, live_owners: live.owners, live_objects: live.objects, live_constants: live.constants, tree_nodes, timers, frame_loops: motion.loops, animations: motion.animations, turns, outstanding_blocking }`。`launch::inspect` は使わない）、`take_snapshot`（`rt.live_snapshot()`）、`diff_snapshots`（id で集合の差）、`export_trace`（`kivact_devtools_protocol::chrome_trace(&turns, &frames)` で文字列にし、`i.agent.borrow().write_trace(&json, path.as_deref())` で `.kivact/traces/` に書いて path を返す。借用はその 1 つの文の中で終える。時刻は loop の時計の ns を µs にする処理も `chrome_trace` が行う）。

### T12.4 a11y と input

1. `a11y.rs`: `get_a11y_tree`（root から `Reader::semantics` のある node を辿る）、`audit_a11y`（設計書 4.2.4 の規則の表をそのまま実装。コントラスト比は WCAG の相対輝度の式）、`get_focus`（`i.hooks.focus_history`）、`get_tab_order`、`lint_layout`（`LayoutCheck::default()`）。
2. `input.rs`: `inject_input`。`!i.can_inject` なら `UNSUPPORTED`（embed の target）、window が無ければ `WINDOW_NOT_FOUND`。`InputStep` を engine の contract が再公開する `InjectedInput` の列に変える:
   - `Click` は `Reader::box_model` の content の中心へ `MouseMove`・`MouseDown(Primary)`・`MouseUp(Primary)`。
   - `Key { key }` は `convert::parse_chord(&key)`（読めなければ `INVALID_PARAMS`。message に chord の書き方を書く）の modifiers を `InjectedInput::Modifiers(chord.modifiers)` で押し、key の押し下げと離し（`InjectedInput::Raw(RawInput::Key(RawKey { key: PlatformKey::Code { key, code: KeyCode::Unidentified }, pressed, repeat: Some(false), text }))`。`text` は `kivact-test/src/screen/input.rs` の `key_text` と同じ規則: `Key::Char(c)` は `c`、`Space` は `" "`、それ以外と、Ctrl・Alt・Meta を押した chord は `None`）、最後に `InjectedInput::Modifiers(Modifiers::default())`。
   - `Text` は文字ごとに同じ key の押し下げと離し（`kivact-test` の `type_events` と同じ規則: `\n` は `Enter`、`\t` は `Tab`、` ` は `Space`、それ以外は `Key::Char`）。
   - `MouseMove`・`MouseDown`・`MouseUp`・`Wheel` はそのまま `InjectedInput` に（`convert::mouse_button`）。
   - `A11y { node, action, value }` は `InjectedInput::Accessibility { node, request: convert::a11y_request(action, value)? }`（別の経路を作らない）。
   - `RequestId` を採番し、`ViewState::tree(rt).engine().effect(request, HostRequest::Inject { window, inputs })`、`Pending::Inject { window, request, reply }` を `i.session.pending` に積む。`delivered(rt, request)` で、その request の pending に `Empty` で答える。`list_inputs`（`ListInputsParams { since, limit }`。cursor `since` は `InputLogEntry.seq`: `seq > since` を `limit` 件まで。`seq` は `input_event` の hook が振るので、ここでは読むだけ）、`get_ime_state`。

### T12.5 motion・console・emulation

1. `motion.rs`: `list_animations`（`MotionState::animations()`。`id` は並びの番号。`time_scale` は `MotionState::time_scale()`）、`set_time_scale`（`MotionState::set_time_scale` と engine の `HostRequest::TimeScale`。範囲外は `-32602`）、`list_tasks`（`TaskState::list()` の timer と、`rt.live_snapshot()` の object のうち名前が `"Task"`・`"Resource"`・`"Action"` のもの。`phase` は task・resource・action が `Idle`、timer が `Pending`）、`reset_time_scale`（倍率 1 に戻す）。
2. `console.rs`: `list_errors`（`ListErrorsParams { since, limit }`。`rt.reported()` の `Reported`。`seq` はその値、`since` より後だけ。`ErrorEntry.turn` は `None`）。`after_turn`: `rt.reported()` のうち `errors_sent` の値より大きい `seq` を `error_reported` で通知し、最後の seq で `errors_sent.borrow_mut().update(..)`。
3. `emulation.rs`: `get_emulation`・`set_emulation`（`i.hooks.emulation` に入れ、engine へ `HostRequest::Emulate`、reduced motion は `HostRequest::ReducedMotion`。touch の `Capabilities.touch` への反映は `capabilities` の memo（T11.1 の hook）が `emulation` を読んで行うので、ここでは書かない。locale と pseudo も `emulation` を hook が読む）、`clear(rt)`（`devtools_state::emulation_off()` を入れ、`Emulate(EngineEmulation::default())` と `ReducedMotion(None)` を出す）。

### T12.6 review と scripts

1. `review.rs`: `get_review`（agent の review を短く借りて `document().clone()`）、`add_comment`（node が無ければ `NODE_NOT_FOUND`。`tree::tree_node`、`node::call_chain`、`node::states` から `ReviewTarget` を作り、その node に掛かっている override を `StyleChange` にし、`screenshot` が true なら T12.1 の `screenshot` と同じ方法（local task で snapshot を待ち、`kivact_devtools_agent::encode_png`）で PNG の bytes を得てから `i.agent.borrow_mut().review().add(NewComment { window, window_title, target, style_changes, image_png, text }, now_ms)`（`now_ms` は wall clock の ms。借用はその 1 つの文の中で終える）。答え（`ReviewItem`）は local task の中で送る）、`edit_comment`、`remove_comment`、`export_review`（params は `Empty`。`i.agent.borrow_mut().review().export()` が返す `ExportedReview { json, markdown }`（変更のたびに書かれた `review.json` と `review.md` の path）をそのまま返す）。review の環境（`environment`）には `set_environment` で今の emulation を入れる。
2. `scripts.rs`: `show_table` → `script_table` を通知。


---

## 16. W13: DevTools アプリの土台

| task | 担当 | crate |
|---|---|---|
| T13.1 | 1 | `crates/kivact-devtools` |

波の検査: `cargo check -p kivact-devtools --all-targets`、`cargo clippy -p kivact-devtools --all-targets -- -D warnings -A dead_code`（W14 が使うまで未使用の widget・method があるので、この波だけ `dead_code` を許す。それ以外の warning は許さない）、`cargo xtask check --no-write`（全体。`state-fields` 以外の rule の error はすべて直す。`state-fields` はこの波の task が作った・変えた型（`kivact-devtools` の型）の分だけ直し、他の crate の型・基準 commit の型の error と「struct-level marker」「unused marker」は T15.4 が直すので対象外。W11 の波の検査と同じ扱い）。commit: `feat(devtools): app shell, connection and panel scaffolding`。

### T13.1 アプリの土台

担当ファイル: `crates/kivact-devtools/**`。

参照: 設計書 4.5.1、4.5.2。部品の使い方は `examples/kivact-ui-basic/src/shell.rs`（theme の install と panel の配置）、`examples/kivact-ui-basic/src/fonts.rs`（font）、`templates/basic/src/main.rs`（試験の書き方）。この task が **W14 の 6 体が共有するものをすべて先に作る**: 全部の型・全部の `Session` の method・全部の widget の空の実装・全部の試験の file と登録。W14 の担当はこの task が作った file のうち自分の file だけを変える。

書き方の約束:

- すべての file の先頭に `//! Role: <英語 1 文>`。この crate は `src/lib.rs`（試験に出す名前だけ `pub`）と `src/main.rs` の両方を持つので、`lib.rs` が `pub mod` で出す module の外の item はすべて `pub(crate)`（`unreachable_pub` の警告を出さない）。
- protocol の型は `use kivact_devtools_protocol as p;` で `p::Role` のように書き、client は `use kivact_devtools_client as client;` で `client::Connection` と書く。`use kivact::prelude::*;` は使ってよい（T0.1 が `crates/kivact-devtools/*` を glob import の例外にした）。それ以外の glob は使わない。prelude の `Role`・`Size` と protocol の同名の型は、`p::` を付けて区別する。
- 空の本体は、引数を `let _ = (a, b);` で使って `unused_variables` を避ける。`allow` の属性を書かない。
- 状態は `state.rs` の型だけで持つ。`bool` や `Option` を 2 つ並べて状態を表さない。

手順:

1. module の構成: `src/lib.rs`（`mod` と `pub mod` だけ）:

   ```rust
   //! Role: Library of the DevTools app, so that integration tests can build the app.
   pub mod app;
   pub mod args;
   pub mod fonts;
   pub mod source;
   pub mod state;
   mod connection;
   mod editor;
   mod panels;
   mod session;
   mod status_bar;
   mod toolbar;
   mod widgets;
   ```

   `src/main.rs`: `kivact_devtools::args::Args::parse(std::env::args().skip(1))` の `Err(text)` は `Err(kivact::UiError::msg(text))` にして返し、`kivact::App::new("Kivact DevTools").window_size(1280, 800).fonts(kivact_devtools::fonts::fonts().iter().copied()).blocking_threads(6).run(move || kivact_devtools::app::root(args))`（`fn main() -> kivact::Result<()>`）。leaf: `args.rs`、`fonts.rs`、`source.rs`、`state.rs`、`session.rs`、`connection.rs`、`editor.rs`、`app.rs`、`toolbar.rs`、`status_bar.rs`、`panels.rs`（`mod` だけ）と `panels/*.rs`、`widgets.rs`（`mod` だけ）と `widgets/*.rs`。
   - `args.rs`: `pub struct Args { pub target: Option<std::path::PathBuf> }` と `impl Args { pub fn parse(args: impl Iterator<Item = String>) -> Result<Args, String> }`。`--target <path>` だけを受け付ける（値が無ければ `Err("--target needs a path")`、知らない引数は `Err(format!("unknown argument: {arg}"))`）。
   - `fonts.rs`: `pub fn fonts() -> &'static [&'static [u8]]` が `include_bytes!("../../../examples/shared-assets/fonts/inter/Inter[opsz,wght].ttf")` と `…/geistmono/GeistMono[wght].ttf` を返す。
2. `source.rs` — data の出どころ（試験で偽の data を使えるようにする）。この形で書く:

   ```rust
   //! Role: Where the DevTools app gets its data: a live agent or fixed answers for tests.
   use std::cell::RefCell;
   use std::rc::Rc;
   use std::sync::Arc;
   use std::time::Duration;

   use kivact::prelude::*;
   use kivact_devtools_client as client;
   use kivact_devtools_protocol as p;

   /// A live connection, or fixed answers.
   #[derive(Clone)]
   pub enum Source {
       /// A connection to an agent.
       Live(Arc<client::Connection>),
       /// Fixed answers for tests.
       Fixture(Rc<Fixture>),
   }

   impl PartialEq for Source {
       fn eq(&self, other: &Self) -> bool {
           match (self, other) {
               (Source::Live(a), Source::Live(b)) => Arc::ptr_eq(a, b),
               (Source::Fixture(a), Source::Fixture(b)) => Rc::ptr_eq(a, b),
               _ => false,
           }
       }
   }

   /// Fixed answers for tests.
   pub struct Fixture {
       pub app: p::AppInfo,
       pub windows: Vec<p::WindowInfo>,
       pub trees: Vec<p::TreeSnapshot>,
       pub nodes: Vec<p::NodeDetail>,
       pub turns: Vec<p::TurnGraph>,
       pub frames: Vec<p::FrameInfo>,
       /// Answers by method name. They win over the fields above.
       pub answers: Vec<(String, serde_json::Value)>,
       /// Notifications that `Source::listen` delivers once, in order.
       pub notifications: Vec<p::Notification>,
       /// Every call made (method name and params), for assertions.
       pub calls: RefCell<Vec<(String, serde_json::Value)>>,
   }

   impl Source {
       /// Call method `M`. `Live` waits on a blocking thread; `Fixture` answers at once.
       pub(crate) async fn call<M>(&self, params: M::Params) -> Result<M::Result, UiError>
       where
           M: p::Method + 'static,
           M::Params: Send + 'static,
           M::Result: Send + 'static,
       {
           match self {
               Source::Live(connection) => {
                   let connection = connection.clone();
                   blocking(move || connection.call::<M>(params)).await.map_err(UiError::from)
               }
               Source::Fixture(fixture) => fixture.answer::<M>(&params),
           }
       }
   }
   ```

   `Fixture::answer::<M>(&self, params: &M::Params) -> Result<M::Result, UiError>`: params を `serde_json::to_value` で `sent` にし、`(M::NAME, sent)` を `calls` に足し、値を 3 段で決める: (1) `answers` の `M::NAME` の値、(2) `builtin_answer(self, M::NAME, &sent)`、(3) `{}`。その値を `serde_json::from_value::<M::Result>` で読み、失敗なら `UiError::msg(format!("not in the fixture: {}", M::NAME))`（`{}` は `Empty` の result として読めるので、`highlight`・`subscribe` などは答えを書かなくても成功する）。`builtin_answer(f: &Fixture, method: &str, params: &serde_json::Value) -> Option<serde_json::Value>` は `match method` で `"get_app_info"`（`f.app`）、`"list_windows"`（`p::WindowList`）、`"get_tree"`（`f.trees` から `params["window"]` が `window` のもの）、`"get_node"`（`f.nodes` から `params["node"]` が `node.id` のもの）、`"list_turns"`（`p::TurnList`）、`"list_frames"`（`p::FrameList`）を `serde_json::to_value(..).ok()` で返し、それ以外は `None`。

   `Source::listen(&self, on: impl Fn(p::Notification) + 'static, closed: impl FnOnce(String) + 'static)`（`spawn` で 1 本の loop を起こす。state を参照しない）:
   - `Live`: `loop { let c = connection.clone(); match blocking(move || c.next_notification(Duration::from_millis(500))).await { Ok(Some(n)) => on(n), Ok(None) => {} Err(e) => { closed(e.to_string()); break; } } }`。
   - `Fixture`: `for n in fixture.notifications.clone() { on(n); }`（`closed` は呼ばない）。
3. `state.rs` — **このアプリの状態の型をすべてここに置く**（`Connection`・`Target`・`Session`・`Inspected`・`Panel`。型を他の file に置くと module が循環する）。設計書 4.5.1 の定義をそのまま書き、足すもの:
   - `pub const MAIN_WINDOW: p::WindowId = 1;`
   - `Session` の field は設計書 4.5.1 のとおり（`target`・`connection`・`windows`・`inspected`・`turns`・`frames`・`errors`・`tables`・`panel`・`popped`・`status`・`detail`・`recording`・`overrides`・`emulation`・`overlays`・`review`）。`overlays` は `Resource<p::OverlaySettings>`（`get_overlays`）で、resource は field として一度だけ作る（method で作ると呼ぶたびに別の resource ができ、setup 以外では作れない）。`state.rs` に `pub(crate) fn overlays_off() -> p::OverlaySettings`（全部 false の `p::OverlaySettings { paint_flashing: false, layout_lint: false, update_heatmap: false, tab_order: false }`。protocol の型は `Default` を持たないのでここで一度だけ書く）を置く。
   - `#[derive(Clone, Copy, PartialEq, Eq, Hash)] pub(crate) enum Panel { Elements, Updates, Performance, Accessibility, Input, Animations, Async, Theme, Console, Emulation, Review, Scripts }` と `impl Panel { pub(crate) const ALL: [Panel; 12] = [..この順..]; pub(crate) fn title(self) -> &'static str }`（"Elements"、"Updates"、"Performance"、"Accessibility"、"Input"、"Animations"、"Async"、"Theme"、"Console"、"Emulation"、"Review"、"Scripts"）。
   - `Inspected` のメソッド（設計書 4.5.1）を本体まで書く。例:

     ```rust
     impl Inspected {
         pub(crate) fn window(&self) -> p::WindowId {
             match self { Inspected::Loading { window, .. } | Inspected::Loaded { window, .. } => *window }
         }
         pub(crate) fn selected(&self) -> Option<p::NodeId> {
             match self { Inspected::Loaded { selected, .. } => *selected, Inspected::Loading { .. } => None }
         }
         pub(crate) fn select(&mut self, node: Option<p::NodeId>) {
             if let Inspected::Loaded { selected, .. } = self { *selected = node; }
         }
         pub(crate) fn finish_load(&mut self, tree: p::TreeSnapshot) {
             let Inspected::Loading { window, select } = self else { return };
             let (window, select) = (*window, *select);
             if window != tree.window { return; }
             let selected = select.filter(|id| tree.nodes.iter().any(|n| n.id == *id));
             *self = Inspected::Loaded { window, tree, selected };
         }
         pub(crate) fn select_in(&mut self, window: p::WindowId, node: p::NodeId) -> bool {
             if let Inspected::Loaded { window: w, tree, selected } = self
                 && *w == window && tree.nodes.iter().any(|n| n.id == node)
             {
                 *selected = Some(node);
                 return false;
             }
             *self = Inspected::Loading { window, select: Some(node) };
             true
         }
         pub(crate) fn keep_window(&mut self, windows: &[p::WindowInfo]) -> bool {
             if windows.iter().any(|w| w.id == self.window()) { return false; }
             *self = Inspected::Loading { window: MAIN_WINDOW, select: None };
             true
         }
         // apply_diff: 設計書 4.5.1 のとおり（upserted で置き換え・足す、removed を消す、revision、選択の補正）。
     }
     ```

     `push_capped<T>(items: &mut Vec<T>, item: T, max: usize)`（末尾に足し、`max` を超えたら先頭から捨てる）もここに置く。上限の定数: `MAX_TURNS = 1000`、`MAX_FRAMES = 3000`、`MAX_ERRORS = 1000`、`MAX_TABLES = 50`。
   - `Source` の `PartialEq` は手順 2 で書いたので、`Connection`・`Target` は `#[derive(Clone, PartialEq)]`。
   - 同じ file に `#[cfg(test)] mod tests`（`//! Role:` 付き。runtime なしの純粋な試験）: (1) `finish_load` は window が違う答えを捨てる、(2) `finish_load` は `select` が tree にあれば `selected` にし、無ければ `None`、(3) `apply_diff` は upsert・remove をし、選んだ node が消えたら `selected` が `None`、(4) `select_in` は別の window なら `Loading { select: Some(node) }` にして true、(5) `keep_window` は今の window が一覧に無いとき `Loading { window: MAIN_WINDOW, select: None }`。p の値は `serde_json::from_value(serde_json::json!({..}))` で作る。
4. `session.rs` — `impl Session`（`state.rs` の型を一方向に使う）。次の method を、**この名前と形で**書く（W14 の panel はこれを呼ぶ）。`Session` は Copy なので `self` を値で取る:
   - `pub(crate) fn new(target: Target, connection: Signal<Connection>) -> Session`: **setup の中だけ**で呼ぶ（`app.rs` の `Connected` の枝の中）。signal と resource を作り（`detail` は `resource(move || inspected.with(Inspected::selected), move |node: Option<p::NodeId>| { let source = source.clone(); async move { match node { None => Ok(None), Some(node) => source.call::<p::GetNode>(p::GetNodeParams { node }).await.map(Some) } } })`。`recording`・`overrides`・`emulation`・`overlays`・`review` は、引数の無い method を呼ぶ次の小さな関数で作る: `fn read<M>(source: &Source) -> Resource<M::Result> where M: p::Method<Params = p::Empty> + 'static, M::Result: Clone + Send + 'static`（`resource(|| (), move |()| { let source = source.clone(); async move { source.call::<M>(p::Empty {}).await } })`。M は `p::GetRecording`・`p::ListStyleOverrides`・`p::GetEmulation`・`p::GetOverlays`・`p::GetReview`）、最後に `on_mount(move || { session.subscribe(); session.load_window(MAIN_WINDOW, None); session.listen(); })` を登録して `session` を返す（signal の書き込みと task の起動を setup の中でしないため）。初期値: `inspected` は `Loading { window: MAIN_WINDOW, select: None }`、`panel` は `Panel::Elements`。
   - `fn subscribe(self)`: `send::<p::Subscribe>(p::SubscribeParams { topics: vec![p::Topic::Tree, p::Topic::Turns, p::Topic::Frames, p::Topic::Pick, p::Topic::Errors, p::Topic::Devtools, p::Topic::Scripts, p::Topic::Windows] }, |_| ())`。
   - `fn listen(self)`: `self.source().listen(move |n| self.apply(n), move |reason| connection.set(Connection::Lost { path, app_name, reason }))`（`path`・`app_name` は `self.target.with(|t| (t.path.clone().unwrap_or_default(), t.app.name.clone()))`）。**`Lost` にするのはここだけ**。
   - `pub(crate) fn apply(self, notification: p::Notification)`: 設計書 4.5.1 の表のとおりの `match`（全 variant を書く。`_` で受けない）。`FocusRequested` は `spawn(async { let _ = kivact::platform::focus_window().await; })`（前面化は最善の努力なので、失敗を error にしない）。`Pick` と `WindowsChanged` は private の `on_pick(self, &p::PickEvent)`・`on_windows(self, Vec<p::WindowInfo>)` に分ける（1 つの関数は 80 行まで）。
   - `pub(crate) fn load_window(self, window: p::WindowId, select: Option<p::NodeId>)`: `inspected.set(Loading { window, select })`、`send::<p::GetTree>(p::GetTreeParams { window, root: None, depth: None }, move |tree| inspected.update(|i| i.finish_load(tree)))`。
   - `pub(crate) fn send<M>(self, params: M::Params, done: impl FnOnce(M::Result) + 'static) where M: p::Method + 'static, M::Params: Send + 'static, M::Result: Send + 'static`: `spawn(async move { match source.call::<M>(params).await { Ok(r) => done(r), Err(e) => self.status.set(Some(format!("{}: {}", M::NAME, e.message()))) } })`。
   - `pub(crate) fn highlight(self, node: Option<p::NodeId>, layout: bool)`（`send::<p::Highlight>`）、`pub(crate) fn set_overlay(self, edit: impl FnOnce(&mut p::OverlaySettings))`（`let mut o = self.overlays.latest().unwrap_or_else(overlays_off);` で今の値を取り、`edit(&mut o)` で変え、全体を `send::<p::SetOverlays>(o, move |_| self.overlays.refetch())` で送る。値の写しは持たない。panel は `session.set_overlay(|o| o.paint_flashing = on)` のように呼ぶ）、`pub(crate) fn overlays_ready(self) -> bool`（`overlays.latest().is_some()`。toggle を `.disabled(..)` にする条件）、`pub(crate) fn report(self, text: String)`（`status.set(Some(text))`）。
   - `pub(crate) fn open_source(self, location: p::SourceLocation)`: `let config = self.target.with(|t| (*t.config).clone()); let root = self.workspace_root();` を取り、`spawn(async move { if let Err(text) = blocking(move || editor::open_in_editor(&config, &root, &location)).await { self.report(text); } })`（`Rc` の `config` は thread に渡せないので中身を clone する）。
   - 読み取り: `source(self) -> Source`、`config(self) -> Rc<p::DevtoolsConfig>`、`workspace_root(self) -> String`（`target.app.workspace_root`）、`window(self) -> p::WindowId`（`inspected.with(Inspected::window)`）。
5. `connection.rs` — 接続の確立と、`Connection` の enum の各状態の画面（enum 自体は `state.rs`。ここに enum を書かない）。
   - `pub(crate) fn connect(connection: Signal<Connection>, path: PathBuf)`: `connection.set(Connection::Connecting(path.clone()))`、`spawn(async move { let opened = blocking({ let path = path.clone(); move || open_target(&path) }).await; match opened { Ok((conn, app, windows, config)) => connection.set(Connection::Connected(Target { source: Source::Live(Arc::new(conn)), app, windows, config: Rc::new(config), path: Some(path) })), Err(reason) => connection.set(Connection::Lost { path, app_name: String::new(), reason }) } })`。`open_target(path: &Path) -> Result<(client::Connection, p::AppInfo, Vec<p::WindowInfo>, p::DevtoolsConfig), String>` は blocking の中で `client::read_target`、`file.protocol != p::PROTOCOL_VERSION` なら接続せずに `Err(format!("version differs (target v{}, DevTools v{})", file.protocol, p::PROTOCOL_VERSION))`、`client::Connection::open(&file, p::ClientInfo { name: "kivact-devtools".into(), version: env!("CARGO_PKG_VERSION").into(), kind: p::ClientKind::Devtools })`、`conn.call::<p::ListWindows>(p::Empty {})`、`conn.call::<p::GetConfig>(p::Empty {})` を順に呼ぶ（`Rc` は thread をまたげないので、`Target` は UI thread で組む。エラーは `.map_err(|e| e.to_string())`）。
   - `pub(crate) fn discover(connection: Signal<Connection>, prefer: Option<(PathBuf, String)>)`: `connection.set(Connection::Searching)`、`spawn(async move { let list = blocking(client::list_targets).await; match choose(&list, prefer.as_ref()) { Some(path) => connect(connection, path), None => connection.set(Connection::Choosing(list)) } })`。`choose(list, prefer) -> Option<PathBuf>`: `prefer` があれば、path が同じ file、無ければ `app_name` が同じものがちょうど 1 つならそれ、それ以外は `None`。`prefer` が無ければ、一覧がちょうど 1 つならそれ、それ以外は `None`。`choose` は純粋な関数なので `#[cfg(test)] mod tests` で 4 つの場合を確かめる。
   - 画面: `pub(crate) fn searching_view() -> impl IntoView`（"Looking for running apps…" と `ui::spinner()`）、`choosing_view(connection, targets)`（target ごとに `button("<app_name> · pid <pid> · <workspace_root>")` が `connect` を呼ぶ。空なら "No running Kivact app with DevTools was found." と「Refresh」の button が `discover(connection, None)`）、`connecting_view(path)`（spinner と path）、`lost_view(connection, path, app_name, reason)`（reason の text と「Reconnect」の button が `discover(connection, Some((path, app_name)))`、「Choose another target」の button が `discover(connection, None)`）。文言は英語。
6. `editor.rs`: 設計書 4.2.7 の `editor` の command を起動する。**`Command::status()` で待つ**（editor の command は `code` や `subl` のように起動したらすぐ戻るものが前提。`spawn` だけして `Child` を捨てると、clippy の `zombie_processes` に当たるため）。次の形で書く:

   ```rust
   //! Role: Opens a source location in the user's editor.
   use std::io::ErrorKind;
   use std::path::Path;
   use std::process::Command;
   use kivact_devtools_protocol as p;

   const DEFAULT_EDITOR: [&str; 3] = ["code", "--goto", "{file}:{line}:{column}"];

   /// Run the editor command of `config` (blocking). `{file}` is `root` joined with the location's file.
   pub(crate) fn open_in_editor(config: &p::DevtoolsConfig, root: &str, location: &p::SourceLocation) -> Result<(), String> {
       let template: Vec<String> = if config.editor.is_empty() {
           DEFAULT_EDITOR.iter().map(|s| (*s).to_string()).collect()
       } else {
           config.editor.clone()
       };
       let file = Path::new(root).join(&location.file).to_string_lossy().into_owned();
       let args: Vec<String> = template.iter().map(|a| a.replace("{file}", &file)
           .replace("{line}", &location.line.to_string()).replace("{column}", &location.column.to_string())).collect();
       let Some((program, rest)) = args.split_first() else { return Err(hint("the editor command is empty")) };
       run(program, rest).map_err(|e| hint(&format!("could not start `{program}`: {e}")))
   }

   /// Run `program`. If it is not found, try `<program>.cmd` once: on Windows `code` is `code.cmd`,
   /// and `Command` adds only `.exe` by itself.
   fn run(program: &str, rest: &[String]) -> std::io::Result<()> {
       let status = match Command::new(program).args(rest).status() {
           Err(e) if e.kind() == ErrorKind::NotFound => Command::new(format!("{program}.cmd")).args(rest).status()?,
           other => other?,
       };
       if status.success() { Ok(()) } else { Err(std::io::Error::other(format!("exit status {status}"))) }
   }

   fn hint(problem: &str) -> String {
       format!("{problem}. Set \"editor\" in .kivact/devtools.json, for example [\"code\", \"--goto\", \"{{file}}:{{line}}:{{column}}\"].")
   }
   ```

   設定は接続のときに agent に `get_config` で問い合わせる（手順 5 の `open_target`。app 自身は `.kivact/devtools.json` を読まない）。`#[cfg(test)] mod tests` に、テンプレートの置き換え（`{file}`・`{line}`・`{column}`）の試験を 1 つ（置き換えの部分を `fn expand(template, file, line, column) -> Vec<String>` に分けて試験する）。失敗の文は `Session::report` で status bar に出る。
7. `app.rs`・`toolbar.rs`・`status_bar.rs`:
   - `app.rs`: `pub fn root(args: Args) -> impl IntoView`（`let connection = signal(Connection::Searching);` の後 `on_mount(move || match args.target { Some(path) => connection::connect(connection, path), None => connection::discover(connection, None) })`。`shell(connection)` を返す）と、`pub fn view_with(initial: Connection) -> impl IntoView`（試験用。`shell(signal(initial))`）。`fn shell(connection: Signal<Connection>) -> impl IntoView`: `provide_theme(theme, move || dynamic(move || connection.get(), move |c| screen(c, connection)))`。theme は `shell.rs` の例と同じ形（`kivact_ui_basic::theme(mode)` に `Typography { family: Some("Inter, sans-serif".into()), ..theme.text().clone() }` を `with_text`）で、`mode` は `kivact::platform::theme_hint()` が `ThemeHint::Dark` のとき `ThemeMode::Dark`。`fn screen(c: Connection, connection: Signal<Connection>) -> View` は `match` で 5 つの状態を `.into_view()` に: `Connected(target)` だけが `session_view(Session::new(target, connection))`。
   - `session_view(session: Session)`: `column((toolbar::view(session), tabs("Panels", <Panel::ALL から作る Vec<ListItem<Panel>>>, session.panel).content(move |panel| panel_view(panel, session)), status_bar::view(session), popped_windows(session)))`。tab の中身が残りの高さを占める（`grow(1)`、`min_height(0)`）。`panel_view(panel: Panel, session: Session) -> View` は `match` で 12 の `panels::<name>::view(session).into_view()`（`Async` は `panels::tasks`）。`popped_windows(session) -> View` は `for_each(session.popped, |panel| *panel, move |row| { let panel = row.get_untracked(); window(panel.title(), binding(move || true, move |open: bool| if !open { session.popped.update(|v| v.retain(|p| *p != panel)); })).size(900, 600).content(move || panel_view(panel, session)) })`。
   - `toolbar.rs`（`pub(crate) fn view(session: Session) -> impl IntoView`）: 左から target 名（`session.target.with(|t| t.app.name.clone())`）、window の `select`（`options` は `session.windows` から作った `ListItem<p::WindowId>`、選択は `binding(move || Some(session.window()), move |w: Option<p::WindowId>| if let Some(w) = w { session.load_window(w, None) })`。写しを持たない）、「Pick」の button（`send::<p::StartPick>(p::StartPickParams { window: session.window() }, |_| ())`）、「Record」／「Stop」の button（`session.recording.latest()` の `updates`・`frames`・`input` のどれかが true なら「Stop」。押すと `start_recording`（`updates: true, frames: true, input: false`）か `stop_recording` を送り、終わったら `session.recording.refetch()`）、「Review (N)」の button（N は `session.review.latest()` の `items.len()`。押すと `panel` を Review に）、「Pop out」の button（今の panel を `popped` に足す。設計書 4.5.2）。
   - `status_bar.rs`（`pub(crate) fn view(session: Session) -> impl IntoView`）: 接続先（app 名と pid）、記録中なら「Recording」、emulation が何か有効なら「Emulating」（`session.emulation.latest()` が `theme`・`reduced_motion`・`locale`・`scale`・`touch` のどれかが `Some`、または `pseudo_locale` が `Off` でない）、「N overrides」（`session.overrides.latest()` の件数）、`session.status` が `Some` ならその文。
8. `panels/*.rs`: 12 の panel の module を作り、それぞれ `pub(crate) fn view(session: Session) -> impl IntoView` を `let _ = session; text("<Panel> is not built yet")` で作る（W14 が中身を書く）。Elements の Styles の tab のために `panels/elements.rs` を `mod styles; mod view; pub(crate) use view::view;` だけにし、leaf の `panels/elements/view.rs`（`view` の本体）と `panels/elements/styles.rs`（`pub(crate) fn styles_tab(session: Session, detail: Resource<Option<p::NodeDetail>>) -> impl IntoView` の空の実装）を作る。
9. `widgets/*.rs`: 次の関数を、ここで決めた形の空の実装（`let _ = (引数,); node()`）で作る（すべて `pub(crate) fn`。W14 が中身を書く）。`TreeNode` は `kivact::interaction::TreeNode`、`View` は prelude の `View`:
   - `virtual_tree.rs`: `pub(crate) fn virtual_tree<K: ItemKey>(label: &str, nodes: Value<Vec<TreeNode<K>>>, expanded: Signal<Vec<K>>, selected: Binding<Option<K>>, row: impl Fn(&K) -> View + 'static) -> impl IntoView`。**選択は `Binding<Option<K>>`**（呼び出し側が `Inspected` の `selected` を `binding(..)` で渡す。選択の写しを作らない）。
   - `source_link.rs`: `pub(crate) fn source_link(location: p::SourceLocation, session: Session) -> impl IntoView`。
   - `property_row.rs`: `pub(crate) fn property_row(name: String, value: Signal<String>, enabled: Signal<bool>, on_commit: impl Fn(String) + 'static) -> impl IntoView`。
   - `color_input.rs`: `pub(crate) fn color_input(label: &str, value: Signal<String>, on_commit: impl Fn(String) + 'static) -> impl IntoView`。
   - `timeline.rs`: `pub(crate) fn timeline(turns: Signal<Vec<p::TurnGraph>>, frames: Signal<Vec<p::FrameInfo>>, selected: Signal<Option<u64>>) -> impl IntoView`（`selected` は Updates panel の中だけの turn の選択）。
   - `graph.rs`: `pub(crate) fn cause_graph(graph: Value<Option<p::TurnGraph>>, on_select: impl Fn(p::CauseNode) + 'static, on_open: impl Fn(p::SourceLocation) + 'static) -> impl IntoView`。
   - `flame.rs`: `pub(crate) fn flame_chart(turns: Signal<Vec<p::TurnGraph>>, frames: Signal<Vec<p::FrameInfo>>) -> impl IntoView`。
   - `box_model.rs`: `pub(crate) fn box_model(bounds: Value<Option<p::BoxModel>>) -> impl IntoView`。
10. 試験。**`tests/` の直下には `app.rs` だけを置く**（cargo は `tests/*.rs` をそれぞれ別の test binary にするので、`support.rs` を直下に置くと単独で build され `dead_code` で落ちる）。`tests/app.rs` が `#[path]` で他の file を登録し、file は `tests/app/` の下に置く:
    - `tests/app.rs`（`//! Role:`）: 先頭に `#[path = "app/support.rs"] mod support;` と、12 の panel の `#[path = "app/panels/<panel>.rs"] mod <panel>;`（`elements`・`updates`・`performance`・`accessibility`・`input`・`animations`・`tasks`・`theme`・`console`・`emulation`・`review`・`scripts`）を書く。続けて 3 つの試験: (1) `support::open(&fixture, "Elements")` で描き、toolbar の "Pick"・"Record" の button と、12 の tab が `get_by_role(Role::Tab, "<title>")` で見つかる、(2) `Connection::Lost { .. }` を `view_with` で描くと "Reconnect" と "Choose another target" の button がある、(3) `Connection::Choosing(vec![..2 つ..])` を描くと target の行が 2 つ見える。
    - `tests/app/panels/<panel>.rs`: 12 個とも `//! Role: Tests of the <Panel> panel.` の 1 行だけで作る（W14 の担当は自分の file だけを書く）。panel の試験は `crate::support` を使う。
    - `tests/app/support.rs`（`//! Role:`。この task が完成させる。W14 は変えない）:

      ```rust
      use std::rc::Rc;
      use kivact::prelude::*;
      use kivact::test::Screen;
      use kivact_devtools::source::{Fixture, Source};
      use kivact_devtools::state::{Connection, Target};
      use kivact_devtools_protocol as p;

      /// One window "Main" (id 1); a tree of root 1, button "Save" 2 (test id "save", update_count 3), text "Hello" 3;
      /// the detail of node 2; one turn graph; two frames. `answers` には `get_overlays` の答え（全部 false の `OverlaySettings` の JSON）だけを入れる。notifications は無い。
      pub fn fixture() -> Fixture { /* serde_json::from_value(serde_json::json!({ .. })) で各 p の型を作る */ }
      /// `fixture()` with `answers` (method name, result JSON).
      pub fn fixture_with(answers: Vec<(&str, serde_json::Value)>) -> Rc<Fixture> { /* fixture() の answers に足して Rc に */ }
      pub fn connected(fixture: &Rc<Fixture>) -> Connection { /* Target { source: Source::Fixture(fixture.clone()), app, windows, config: Rc::new(serde_json::from_value(serde_json::json!({})).expect("invariant: all config fields default")), path: None } */ }
      /// Render the app on `fixture` and open the tab `panel`.
      pub fn open(fixture: &Rc<Fixture>, panel: &str) -> Screen {
          let connection = connected(fixture);
          let screen = kivact::test::render(move || kivact_devtools::app::view_with(connection));
          screen.get_by_role(Role::Tab, panel).click();
          screen
      }
      ```

      fixture の各値は JSON で書く（`serde_json::from_value` は既定のある field を省略できる）。必須の field は設計書 4.2.4 の型に合わせる。例: window `{"id":1,"title":"Main","bounds":{"x":0,"y":0,"width":800,"height":600},"scale":1.0,"focused":true,"mode":"normal"}`、node 2 `{"id":2,"window":1,"parent":1,"kind":"box","role":"button","name":"Save","test_id":"save","source":{"file":"src/main.rs","line":10,"column":5},"hidden":false,"leaving":false,"update_count":3}`。
    - この試験は走らせない（C2 で走る）。
11. 確認: `cargo check -p kivact-devtools --all-targets`、`cargo clippy -p kivact-devtools --all-targets -- -D warnings -A dead_code`、`cargo xtask check --no-write --only state-fields --crate kivact-devtools --report`（0.2 の 12.）。

---

## 17. W14: DevTools アプリの各 panel

| task | 担当 | 担当ファイル（`crates/kivact-devtools/` の中） |
|---|---|---|
| T14.1 | 1 | `src/panels/elements/view.rs` と、Elements の他の tab のために足す `src/panels/elements/*.rs`（`styles.rs` を除く。新しい leaf を足すときは `src/panels/elements.rs` に `mod` の行を足してよい）、`src/widgets/virtual_tree.rs`、`src/widgets/source_link.rs`、`src/widgets/box_model.rs`、`tests/app/panels/elements.rs` |
| T14.2 | 2 | `src/panels/elements/styles.rs`、`src/widgets/property_row.rs`、`src/widgets/color_input.rs`、`src/panels/theme.rs`、`tests/app/panels/theme.rs` |
| T14.3 | 3 | `src/panels/updates.rs`、`src/widgets/timeline.rs`、`src/widgets/graph.rs`、`tests/app/panels/updates.rs` |
| T14.4 | 4 | `src/panels/performance.rs`、`src/widgets/flame.rs`、`tests/app/panels/performance.rs` |
| T14.5 | 5 | `src/panels/accessibility.rs`、`src/panels/input.rs`、`src/panels/console.rs`、`src/panels/tasks.rs`、`src/panels/animations.rs`、`tests/app/panels/{accessibility,input,console,tasks,animations}.rs` |
| T14.6 | 6 | `src/panels/emulation.rs`、`src/panels/review.rs`、`src/panels/scripts.rs`、`tests/app/panels/{emulation,review,scripts}.rs` |

同じ crate を 6 体で編集する。他の担当の file のエラーで `cargo check` が失敗したら 0.2 の 4. に従う。波の検査: `cargo check -p kivact-devtools --all-targets`、`cargo clippy -p kivact-devtools --all-targets -- -D warnings`（`-A` は付けない。W14 で未使用が残っていたら、使う側の担当に戻す）、`cargo xtask check --no-write`（W13 と同じ範囲。W11 の波の検査と同じ扱い）。commit: `feat(devtools): panels`。

すべての task に共通: 設計書 4.5.3 の表の自分の行を、表の「使う部品」で作る。data は `Session` の signal・resource と `Session::send`（T13.1 が作った method。自分で `Source` を触らない）。状態を足したくなったら、panel の中だけの signal（選択・toggle・入力中の文）にする。`Session` や `state.rs` の型は変えない（足りない物があれば報告して止まる）。文言は英語（DevTools の UI の文字列）。

各 panel には、その panel の振る舞いを確かめる試験を、T13.1 が作った `crates/kivact-devtools/tests/app/panels/<panel>.rs` に 1 つ以上書く（登録は T13.1 が `tests/app.rs` で済ませた。`tests/app.rs` と `tests/app/support.rs` は変えない）。試験の形: `crate::support::fixture_with(vec![("<method>", serde_json::json!({..}))])` に**自分の panel が呼ぶ method の答え**を足し、`support::open(&fixture, "<Panel>")` で panel を開き、role と name（`get_by_role`、`get_by_text`）で要素を探す。操作の結果は、(a) 画面の文字の変化（答えを足した method の結果が表に出る）か、(b) `fixture.calls.borrow()` に期待した method と params が入っていること、で確かめる（signal は試験から見えない）。pick や tree の更新のように通知で動くものは、`Fixture.notifications` に通知を入れる。試験は走らせない。0.2 の 4. の W14 の例外に従う（他の担当の file のエラーを待たない）。波の検査は指揮役が全員の後で行う。

- T14.1: 仮想化した tree は `kivact::interaction::use_tree` + `kivact::view::virtual_list`（`.row_height(RowHeight::Fixed(22.0))`）+ T8.2 で足した `TreeApi::scroll_with`・`page_size`（`dev/tests/kivact-public-api/tests/e2e/flows.rs:343-412` の `use_list` + `virtual_list` の例を写す）。行には role・name・作成位置・`update_count` の badge（`kivact::ui::badge`）。tree は `session.inspected` が `Loaded` のときだけ描く（`Loading` の間は `kivact::ui::spinner`）。`nodes` は `session.inspected` の `Loaded` の tree から `memo` で作る（`TreeNode::new(n.id, n.parent, <role と name の label>)`）。**選択は `binding(move || session.inspected.with(Inspected::selected), move |id: Option<p::NodeId>| { session.inspected.update(|i| i.select(id)); session.highlight(id, layout.get_untracked()); })` を `virtual_tree` に渡す**（`layout` は後述の Layout の toggle の signal）。選んだ node の詳細は `session.detail`。source の link は `Session::open_source`。pick の button は `session.send::<p::StartPick>(p::StartPickParams { window: session.window() }, |_| ())`、Ctrl+Shift+C も同じ（panel の root に `on_key`）。`pick` の通知で選ぶ処理は `Session::apply`（T13.1）が済ませているので、panel は何もしない。「Copy locator」は新しい leaf `panels/elements/locator.rs` に `pub(crate) fn locator_of(session: Session, detail: &p::NodeDetail) -> Option<String>` を作る: `p::ReviewItem { id: String::new(), window_title, window, target: p::ReviewTarget::Node { node, role, name, test_id, source, call_chain, bounds, states }, style_changes: vec![], image: None, comments: vec![] }`（`window_title` は `session.windows` から `window` の id で探した title、`role`・`name`・`test_id`・`source` は `detail.node` から、`bounds` は `detail.bounds.border`、`call_chain`・`states` は `detail` から）を作って `p::locator(&item)` を返す（protocol の `lib.rs` から直接出ている関数。review の Markdown と同じ）。clipboard へは `spawn(kivact::platform::copy_text(text))`。「Owners」の toggle は panel の中だけの `Signal<bool>`。on のとき tree の代わりに `get_owners` の `OwnerTree`（`resource(move || session.window(), ..)` で読む）を `OwnerNode` の `children` をたどって `TreeNode<u64>`（key は `OwnerNode.id`、label は `label`）に平らにして同じ `virtual_tree` で出し、行を選ぶとその owner の `nodes` の先頭を `session.inspected.update(|i| i.select(..))` で選ぶ（owner の選択は panel の中の `Signal<Option<u64>>`）。Layout の tab に「Show layout overlay」の toggle（panel の中の `Signal<bool>`）。変えたら選んだ node で `session.highlight(selected, layout)` を呼び直す。
- T14.2: Styles の tab は `NodeDetail.styles` の層ごとに見出しと `property_row`。行の checkbox を外すと、その property の override を消す（override が無い行の checkbox は「その値で override を作る」）。値の入力の Enter で `set_style_override`。scope の `select`（`This node`／`All nodes from this call site`／`Theme token`）。forced states の `toggle_group`。color の値の行は `color_input`（hex の `text_input`、H・S・L の 3 つの `slider`、見本の node）。override を足す・消すたびに `session.overrides.refetch()` と `session.detail.refetch()` を呼ぶ（override の一覧は agent の事実なので、写しを持たない）。Theme の panel は `list_theme_tokens` の一覧を `property_row` で、編集は token の override。
- T14.3: Updates は設計書 4.5.3 の行のとおり、上に時間軸のレーン（入力・更新・フレームの 3 段。更新の棒はきっかけ（入力・timer・task）で色を変え、フレームの棒は高さが時間で、frame period の点線を引き、超えたものを赤）、下に選んだ turn の見出し（`#<turn> <最初の cause の label>  合計 <ms>`）と原因グラフ、その下に時間の内訳の横帯（Stage と frame の段階を長さの比で）と、件数の 3 つの枠（変わった要素・再計算・同じ値の書き込み）。data は `session.turns`。timeline は canvas で、turn を region（`Node::regions`）にして click で選ぶ。graph は「カードは絶対配置の node（`style().position(Position::Absolute).left(x).top(y)`）、線は canvas」で作る（`crates/kivact-ui-basic/src/chart/tooltip.rs:87-91` の絶対配置を参考）。列はきっかけ・状態の変化・再計算・変わった要素で、最後に layout/paint/frame の帯。`count > 1` のカードは「×N」を出す。カードの click で `session.highlight(node, false)`、double click で `Session::open_source`。「画面で点滅」「選んだ要素だけ」「ヒートマップ」の切り替えは `Session::set_overlay`。toggle の値は `session.overlays.latest()` から表示し、`Session::overlays_ready` が false の間は無効にする。
- T14.4: frame 時間の棒は canvas（予算線と超過の赤。data は `session.frames`）。flame chart は canvas（turn の Stage と frame の段階）。Counters は `basic::chart`（`ChartKind::Line`）。snapshot の差分は `data_table`。「Export trace」は `export_trace`（既定の path に番号付きの名前で書かれ、結果の path を status bar に出す）。paint flashing の toggle は `Session::set_overlay(|o| o.paint_flashing = on)`（値の表示と無効の条件は Updates と同じ）。layer 境界の toggle は無い（作らない）。
- T14.5: 表は `data_table` か、件数の多いもの（Console・Input）は `virtual_list` の行を列のように並べたもの。Console の data は `session.errors`。Accessibility の監査の行を選ぶと `session.highlight(node, false)`、Tab 順の toggle は `Session::set_overlay(|o| o.tab_order = on)`（値の表示と無効の条件は Updates と同じ）。Animations の時間の倍率は `toggle_group`。
- T14.6: Emulation は `select`・`switch`・`text_input`（設計書 4.5.3）。`set_emulation` は全体を置き換えるので、今の値は `session.emulation.latest()` から取って 1 つ変えて送り、`session.emulation.refetch()`。theme の注意の文を表示する。Review は `session.review` の item ごとの `item_row`、コメントの追加は `text_area` と「Add comment」（選んだ node、無ければ window。`screenshot: true`）、編集・削除の後に `session.review.refetch()`。今の tree（`session.inspected` が `Loaded` で同じ window）に node が無い item には「(removed)」を出して強調の操作を無効にする。「Open folder」は `session.send::<p::ExportReview>(p::Empty {}, move |r| { spawn(kivact::platform::reveal_path(r.json)); })`（`ExportedReview.json` の path。書き出しの操作は無い。`review.json` と `review.md` は変更のたびに保存される）。Scripts は設定の `scripts_dir`（`session.config()`、既定の解決は agent の `get_config` の答え）の `.py` を `std::fs::read_dir` で並べ、「Run」で interpreter（設計書 4.7.4）を `std::process::Command` で起動し、環境変数 `KIVACT_DEVTOOLS_TARGET`（`target.path`）を付け、標準出力と標準エラーを `blocking` で行ごとに読んで表示する。`session.tables` の `show_table` の結果を表にする。

---

## 18. W15: 仕上げ

| task | 担当 | 場所 |
|---|---|---|
| T15.1 | 1 | `dev/tests/kivact-public-api`（DevTools の E2E） |
| T15.2 | 2 | `python/kivact-devtools` |
| T15.3 | 3 | README・architecture.md・dev/README・THIRD-PARTY-NOTICES・CONTRIBUTING・testing ガイド・manual-verification・platform-services／diagnostics／embed ガイド |
| T15.4 | 4 | state-fields の残り（印の付け替え） |

波の検査: `cargo check --workspace --all-targets --all-features`、`cargo xtask check --no-write --only state-fields --report`（error が 0 であること。T15.4 が報告した「判断が要るもの」を除く）。commit: `feat: devtools end to end tests, docs and state-fields enforcement`。

### T15.1 DevTools の E2E

担当ファイル: `dev/tests/kivact-public-api/Cargo.toml`（feature `devtools = ["kivact/devtools"]` と dev-dependency `kivact-devtools-client`・`kivact-devtools-protocol`。policy は W0 で設定済み）、`tests/e2e/devtools.rs`（新規）、`tests/e2e/main.rs`（`#[cfg(feature = "devtools")] mod devtools;`）。

手順: 1 つの試験の中で `render_with_handle(RenderOptions::from_env().devtools(), ..)` で counter の App（`button("Increment")` と `text`）を描き、`screen.devtools_target()`（T11.1）の discovery file を読んで、別 thread から `kivact_devtools_client::Connection::open` で接続する。test の thread は実時間の上限で回す（`screen.wait_for` は仮想時間なので使わない）: `let deadline = Instant::now() + Duration::from_secs(60); while !done.load(Ordering::SeqCst) { assert!(Instant::now() < deadline, "the client did not finish"); screen.advance(Duration::from_millis(16)); std::thread::sleep(Duration::from_millis(1)); }`。client の thread が次を順に呼んで結果を channel で返す:

1. `get_by_role` に `p::RoleQuery { window: 1, within: None, role: p::Role::Button, name: Some("Increment".into()), timeout_ms: None }`（main window の id は 1）。
2. `highlight`（`HighlightParams { node: Some(id), layout: false }`）、`set_style_override`（padding 20）。
3. `list_turns` で `limit: Some(1)` を呼んで今の最新の turn 番号 `since` を覚える。`start_recording`、`inject_input`（その button の `Click`）、**`stop_recording`**（待っている graph を先に完成させるので、click の turn が必ず入る）、`list_turns`（`since`。`PropChange` の node を含む graph があること）。繰り返しの待ちは書かない。
4. `add_comment`（`screenshot: true`。返った `ReviewItem.image` が `Some`）、`export_review`（`Empty`。返った `ExportedReview` の `json` と `markdown` の path に file がある）。

test の thread は counter の text が 1 になったことと、button の矩形が padding で大きくなったことを確かめる。export した directory（`ExportedReview.json` の親）は、確かめた後に `std::fs::remove_dir_all` で消す（agent は workspace root、つまり test の cwd から上へ `Cargo.lock` を探した場所の `.kivact/review/` に書く）。確認: `cargo check -p kivact-public-api-tests --tests --features devtools`、`cargo clippy -p kivact-public-api-tests --tests --features devtools -- -D warnings`（試験は走らせない。C2 で走る）。

### T15.2 Python の仕上げ

手順: `python -m unittest discover python/kivact-devtools/tests` を通す（失敗は担当ファイルの中で直す。偽の agent の試験が生成した型と食い違ったら、`client.py` を直す。生成物の側が原因に見えるときは報告して止まる）。pyright があれば `pyright --strict python/kivact-devtools/src` を通す（無ければ報告に「未実施」と書く）。`examples/` の 2 つが生成した型と一致することを `python -m compileall` で確かめる。

### T15.3 文書

担当ファイル: 設計書 6 章の表のうち、T1.6・T2.5 が済ませたもの以外のすべて（README.md、README_JP.md、`docs/for-kivact-developer/architecture.md`、`docs/for-kivact-developer/manual-verification.md`、`dev/README.md`、`THIRD-PARTY-NOTICES.md`、`CONTRIBUTING.md`、`docs/for-kivact-user/{en,jp}/how-to-guide/{testing,platform-services,diagnostics,embed}.md`）。

手順: 設計書 6 章の各行のとおりに書く。package の数は `cargo metadata --offline --format-version 1 --no-deps` の `packages` の数を数えて書く。architecture.md の crate の表には 5 つの crate の行（設計書 4.1 の表の「境界の理由」を 1 文にしたもの）を足す。

`manual-verification.md` に節「複数 window と DevTools」を足す（既存の表の書き方に合わせる。native でしか確かめられない行。C2 の報告では未確認として書く）:

| 環境 | 確かめること |
|---|---|
| Windows | `Tool` の window が親の上に残り taskbar に出ない。`Modal` は親が無効になり、閉じると戻る。`CenterOnParent`・`NextToParent` の位置。window ごとの focus と Tab 順。2 つ目の window の IME。title bar の閉じるが `on_dismiss` に届く。`platform::focus_window()` が window を前面に出す（出せないときは task bar の点滅） |
| X11 | `Utility`・`Dialog` の window type |
| Wayland | 位置が `Auto` に落ちる。`focus_window()` は何も起きないが error にならない |
| DevTools | 実際の native の App で F12・Ctrl+Shift+I・Ctrl+Shift+C が DevTools アプリを起動する／すでに開いていれば前面に出る。Ctrl+Shift+C は押した window で pick を始める。pick と重ね表示が内容の上に描かれる。DevTools アプリの「Open source」が editor を開く（Windows の `code.cmd` を含む） |
| embed（DAW など） | host の中でも F12・Ctrl+Shift+I・Ctrl+Shift+C が DevTools に届く。`inject_input` は `-32015` で断られる |

`how-to-guide` の 3 つの guide（`platform-services`・`diagnostics`・`embed`）は en と jp の両方に、設計書 6 章の行のとおりの短い節を足す（`platform-services`: window ごとの問い合わせと `focus_window`、`diagnostics`: 更新理由は DevTools の guide へ誘導、`embed`: 追加の window と入力の注入は使えない）。`testing.md` の `screen.window(title)`・`screen.windows()` の節は T1.6 が書いた（重ねて足さない）。その節に、`Screen::has_window()` を `is_mirrored()` に改名したこと（公開の変更。意味は同じ）と `RenderOptions::single_window()` の 1 文ずつを足す。
### T15.4 state-fields の残り

担当ファイル: `cargo xtask check --no-write --only state-fields --report` が名指す file の、印の行だけ（コードは変えない）。

参照: 付録の KEEP 表（各部の 2 章）と 7 章の表、作業指示書 0.2 の 12.（印の位置・形・分類の語・分けた型の対応表）。

印の書き方（すべての手順で共通）: 印は、その field（static は static）の直前、`///` と `#[..]` の行の**上**に 1 行で書く。形は `// xtask-allow(state-fields): <分類> - <その field が何の事実か>`。
- 付録 7 章の表の行: 「印の文」の列をそのまま写す（`<分類> - <事実>` が入っている）。
- KEEP 表の行: 分類はその行の分類をそのまま使う（付録 7.4 に同じ意味の語があればその語。無ければ KEEP 表の語のまま。設計書 5.3 の最後の段落）。「何の事実か」は、その field の `///` の最初の文（無ければ field の名前の `_` を空白にしたもの）。
- 同じ行に複数の field があるときは、field ごとに行を分けてから書く。

手順:

1. `mkdir -p target/plan` の後、`cargo xtask check --no-write --only state-fields --report > target/plan/state-fields-w15.txt` で一覧を作る。
2. 「struct-level marker」の error ごとに: struct の前の印の行を消し、もう一度コマンドを実行して、その型の「combines state fields: a, b」の message に並ぶ field それぞれの直前に、印を書く。分類は、KEEP 表のその型の行（`型.*`、または `(struct-level)` の行）。分けた・移した型は、作業指示書 0.2 の 12. の対応表で元の型を引く。SM-CORE-06 のように entry が field ごとの印の文を書いている型は、entry（と、作業指示書の task の手順）の文を使う。message に出なくなった型は、何も書かない。
3. 「unused marker」の error: その印の行を消す。
4. 付録 7 章の表の行（7.1 と 7.2）: 行ごとに、表の「site」の file の、型.field の直前に印を書く（型名と field 名で探す。行番号は基準 commit のもの）。7.3 の型は何もしない（entry が書いた印がすでにある）。
5. `NodeBuilder` を実装する struct と `value_modules` の module の中の印は「unused marker」になるので、3. で消える（付録の「残す」表で builder option とあるもの）。
6. 付録に載っていない型・static の error（2. 〜 4. で消えないもの）: 直さずに `target/plan/STATE-FINDINGS.md` に `file:line 型名 検査の message` を並べる。この変更で作った・変えた型の error は各 task が直しているはずなので、残っていたら、型を作った task を `git log --format=%s -- <file>` と各波の担当ファイルから探して、同じ行に「作った task: Tx.y」を書く。基準 commit にあった型なら「作った task: 無し（基準 commit）」と書く。
7. `state-locals:` の行（局所変数の報告。error ではない）のうち、この branch が作った・変えた file（`git diff --name-only 96e22b6b...HEAD` に載る file）のものを、`target/plan/STATE-FINDINGS.md` の別の節「state-locals」に `file:line 名前 作った task: Tx.y` の形で並べる。他の file の行は並べない。
8. 確認: 手順 1 のコマンドをもう一度実行する。残りが `target/plan/STATE-FINDINGS.md` の行だけであること。残りの行数を（型の error と state-locals に分けて）報告に書く。

---

## 19. 検査点 C2 と最終の片付け（指揮役）

1. `target/plan/STATE-FINDINGS.md` を読む。
   - 「作った task: Tx.y」の型の行: その task のサブエージェントを、行の全文を付けてもう一度起動する（「この型の combines state fields を直す。設計書 3.11 の表の型にする。担当ファイルの中だけ。直したら T15.4 の 1. と同じコマンドで確かめる」）。
   - 「state-locals」の節の行: その task のサブエージェントに、行ごとに次を判定させて報告させる。(a) 呼び出しに渡す Binding・Value の元（例・試験の caller の値）、(b) 独立した一つの状態、(c) 他の局所変数・signal と組み合わさって状態を表している（互いに有効な組合せがある）。(c) のものだけ、設計書 3.11 の表の型（enum か Machine）に直させる。(a)(b) は何もしない。
   - 「作った task: 無し（基準 commit）」の行が残っているのは、付録 7 章の表が T1.1 の規則と食い違っているときだけである（7 章は T1.1 の規則で数えた全件の表）。その場合は、T1.1 のサブエージェントを、T1.1 の 9. の「7 章との差」と STATE-FINDINGS の行を付けて起動し（「規則の読み違いを直す。担当ファイルの中だけ」）、T15.4 を最初からやり直す。2 回やっても基準 commit の行が残るときだけ、止めてオーナーに渡す（`target/plan/STOP-C2.md` に行を書く）。
2. CONTRIBUTING.md の「merge 前の検査」のコマンドを上から順にすべて回す（`cargo fmt --all -- --check`、`cargo xtask check`（Node.js が要る。`architecture/` が再生成される）、`git diff --quiet -- architecture`（再生成したものを commit してから）、clippy、6 つの `cargo test`、`dev/tools/check-features.sh`）。加えて `python -m unittest discover python/kivact-devtools/tests` と、Linux の compile（0.5）。
3. 失敗は C1 の 6. と同じ手順で直す。
4. 恒久的な文書に移すべき内容（設計書 6 章）が移っていることを確かめ、`DESIGN-devtools-multi-window.md`、`WORK-ORDER-devtools-multi-window.md`、`WORK-ORDER-appendix-state-machines.md` を消す（`target/plan/` の記録は git に入っていないので残してよい）。
5. commit: `chore: remove the work order of the devtools and multi-window change`。push はしない。オーナーが PR を作る。
