# Kivact Slim-Fast 作業指示書

版 1.1（2026-11-09、`20261109_2107_CODEX_REVIEW.md` の妥当な指摘を反映）。設計は [SLIM_FAST_DESIGN.md](SLIM_FAST_DESIGN.md)（以下「設計書」）。本書は手順だけを書く。設計の判断は設計書が持つ。

---

## 0. 読み方と規則

### 0.1 誰が何をするか

| 役割 | 担当 | すること |
|---|---|---|
| Orchestrator | 本書を受け取った AI（親） | wave を順に進める。各 unit に worktree を作り subagent を起動する。結果を merge する。checkpoint（CP）の gate と計測を実行する。subagent の「停止して報告」に、本書の既定の判断表（0.5 節）で答える |
| Implementer | subagent（最大 6 体を同時に） | 割り当てられた unit を、指定の worktree の中で、本書の手順どおりに実装する。完了条件のコマンドを実行し、結果を報告する |

- 全員が守ること：設計書と本書に書いてない設計の判断をしない。迷ったら 0.4 節の「停止して報告」。
- Implementer は push・merge・PR 作成をしない。main に触れない。
- Orchestrator は push を checkpoint の後にだけ行う（force push 禁止）。PR は作らない（人間が作る）。

### 0.2 禁止事項（全 unit 共通）

1. 互換のためのコードを書かない：旧名の alias、`#[deprecated]`、wrapper、旧と新の二重の経路、「旧 API も受ける」変換。旧名は消し、呼び出し元を全部直す。
2. CI の設定・workflow・CI 用の script を足さない。
3. window を開かない単体試験（`#[test]` で engine や Machine の内部を直接試すもの）を足さない。試験は E2E（`kivact::test`）だけ。
4. 期待値を緩めて試験を通さない（許容差を広げる、assert を消す、`#[ignore]` を付ける）。
5. `#[allow(..)]`・`#[expect(..)]`・`xtask-allow` を、警告や lint の件数を減らす目的で足さない。本書が明示する場合だけ足す。
6. 1 関数 80 code 行・1 ファイル 1000 code 行の上限（xtask）を超えない。超えそうなら意味のある helper に分ける。
7. 実装途中に workspace 全体の `cargo test`、E2E、`dev/bench/run.sh` を実行しない（checkpoint で Orchestrator が実行する）。unit で実行してよいのは 0.6 節のコマンドだけ。
8. 依存の crate を勝手に足さない。本書が名前を挙げたものだけ。
9. `xtask-allow(state-fields)` の marker は、`cargo xtask check --only state-fields-strict`（U12 の後）か `--only state-fields` が実際にその field を報告したときだけ付ける。本書が marker を書くよう指示していても、lint が報告しない field には付けない（使われない marker は xtask の error になる）。
10. `std::cell::OnceCell`、`std::sync::OnceLock`、`LazyCell`、`LazyLock` は `clippy.toml` で禁止されている。遅延して作る cache は `RefCell<Option<T>>`（thread をまたぐなら `Mutex<Option<T>>`）にする。

### 0.3 unit の書式

各 unit は次の項目を持つ。

- **目的**：設計書の節。
- **所有 file**：この unit だけが変更してよい file（glob）。ここに無い file を変える必要が出たら「停止して報告」。ただし、改名・署名の変更に伴う「コンパイルを通すための機械的な追従」は、所有 file の外でも、本書が「追従してよい」と書いた範囲で行ってよい。
- **手順**：番号順に行う。
- **完了条件**：すべて成功させるコマンド。
- **撤退規則**：うまくいかない場合の決まった逃げ道。

### 0.4 停止して報告

次の場合は、その項目の作業を止め、他の項目を続け、最後の報告に `STOP:` で始まる行を書く。

- 本書の手順が、実際のコードと合わない（関数が無い、型が違う、行番号の内容が違う）。
- 手順を行うと、所有 file の外を変える必要がある（「追従してよい」の範囲を除く）。
- 規則（SM-1〜SM-12、M-R1〜M-R4、ARG-1〜ARG-6）のどれにも当てはまらない。
- 同じ compile error を 3 回直しても消えない。
- 既存の E2E の期待値を変えないと通らないと分かった。

報告の書式：`STOP: <unit> <file>:<line> <何が合わないか 1 文> / <試したこと 1 文>`。

### 0.5 Orchestrator の既定の判断表

Orchestrator は設計の判断をしない。subagent の `STOP:` に対し、次の表のどれかに当たればその判断をする。当たらなければ、その項目を **設計者への差戻し**（`dev/bench/results/OPEN.md` に 1 行で追記：unit、file:line、STOP の文、影響する設計書の節）とし、他の作業を続ける。差戻しの項目が残っている限り、設計書 1.6 節の受入れは満たされず、W7 で完了を宣言しない。

| 状況 | 判断 |
|---|---|
| 行番号がずれているが、同じ名前の関数・型が近くにある | その関数・型に対して手順を行うよう指示する |
| 所有 file の外の 1〜3 file を変える必要がある | 同じ wave の他の unit の所有 file でなければ許可する。他の unit の所有なら、その項目を次の wave の同じ crate の所有 unit に回す |
| 付録 A に無い状態の組合せを lint が見つけ、6.3 節の規則の形に当てはまらない | 差戻し（推測で enum にしない、SM-11 にもしない） |
| 性能の変更で既存の E2E が失敗し、2 回直しても通らない | その unit の撤退規則を適用する（撤退規則が無ければ差戻し）。適用したことを `OPEN.md` に記録する |
| 改名で取りこぼしたコンパイル error | 改名表（付録 B）に従って直すよう指示する |

### 0.6 unit で実行してよいコマンド

```bash
cargo fmt -p <crate>                       # 必ず最後に
cargo check -p <crate> --all-targets       # 何度でも
cargo check -p <crate> --release           # 指示がある場合
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --only <rule> --crate <crate>   # 指示された rule だけ
cargo test -p <crate> --test <name> -- <filter>               # 本書が名前を挙げた試験だけ
```

`cargo xtask check` の `--only` に使える rule：`state-fields`、`state-fields-strict`（U12 の後）、`effect-writes`、`api-shape`、`canonical-path`、`name-collision`、`kit-literal-color`、`os-branch`、`structural-options`、`devtools-codegen`。

Linux 専用のコード（`cfg(target_os = "linux")`、`port/x11`、`port/wayland`）を変える unit は、次のどちらかで確かめる。どちらもできない環境なら、Linux 専用の項目を実施せず `STOP:` で報告する。

```bash
wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}" -- bash -lc 'cd <worktree の WSL path> && cargo check -p <crate>'
# または
rustup target add x86_64-unknown-linux-gnu && cargo check -p <crate> --target x86_64-unknown-linux-gnu
```

---

## 1. 体制

### 1.1 branch と worktree

- 統合 branch：`refactor/kashiwade/slim-fast`（既に作成済み。設計書・本書・`dev/bench`・R-0 の修正が入っている）。
- 各 unit：`git worktree add ../kv-wt/<unit> -b wt/<unit> <wave の開始時の統合 branch の HEAD>` の直後に `git -C ../kv-wt/<unit> submodule update --init` を実行する（`dev/xtask` が submodule の `vendor/xtask-core` に依存し、submodule が無いと workspace が読めず、すべての `cargo` が失敗する）。subagent にはその絶対 path を渡す。
- 実 window を開くコマンド（`kv_heap`、`kv_startup`、`kv_stress`、E2E）は、unit の中でも必ず `dev/tools/window-lock.sh --timeout 900 -- <command>` を通す（同じ wave の他の unit と同時に window を開かないため）。
- target dir：unit ごとに分ける（`CARGO_TARGET_DIR=<repo の親>/kv-target/<lane>`、lane は A〜F の 6 つを使い回す）。同じ lane の target を同時に 2 つの unit が使わない。
- 1 wave の unit がすべて終わったら、Orchestrator が 1.3 節の手順で merge する。

### 1.2 subagent への指示の雛形

Orchestrator は各 subagent に次の文をそのまま渡し、`<..>` だけを埋める。

```text
あなたは Kivact の unit <unit ID> の実装者です。
作業場所は <worktree の絶対 path> です。この directory の外の file を変更しないでください。
CARGO_TARGET_DIR=<target の絶対 path> を使ってください。
最初に次を読んでください：
  1. <worktree>/SLIM_FAST_WORK_ORDER.md の 0 章（規則）と「<unit ID>」の節
  2. その節が参照する <worktree>/SLIM_FAST_DESIGN.md の節
手順を番号順に実行し、完了条件のコマンドをすべて実行してください。
設計の判断が必要になったら作業を止め、0.4 節の書式で STOP: 行を書いてください。
最後に `git -C <worktree> add -A && git -C <worktree> commit -m "<unit ID>: <要約>"` で commit してください。
push・merge はしないでください。
報告には次だけを書いてください：変更した file の一覧、完了条件の各コマンドの結果（成功/失敗と最後の数行）、STOP: 行。
```

### 1.3 merge の手順（Orchestrator）

各 wave の終わりに、unit の順に：

```bash
git checkout refactor/kashiwade/slim-fast
git merge --squash wt/<unit> && git commit -m "<unit ID>: <要約>"
```

- conflict が出たら：同じ行を二つの unit が変えた場合は、本書でその file を所有する unit の版を採る。所有が無い file（`Cargo.lock` など）は両方を残す形で手で直し、`cargo check --workspace` が通る版にする。
- wave の全 unit を merge したら `cargo check --workspace --all-targets` を 1 回だけ実行する。失敗しても、本書が「この wave の後は失敗してよい」と書いた crate だけなら次へ進む。それ以外は、error を出した crate を所有した unit の subagent に、error の全文を渡して直させる。直しは、統合 branch の今の HEAD から作った新しい worktree（`wt/<unit>-fix`）で行い、終わったら同じ手順で merge する（merge 済みの元の worktree は消してあり、元の branch には他の unit の変更が無いため）。
- merge の後、worktree と `wt/<unit>` branch を消す：`git worktree remove ../kv-wt/<unit> && git branch -D wt/<unit>`。

### 1.4 wave の一覧

| wave | unit（lane） | 前提 | この wave の後に compile が失敗してよい crate |
|---|---|---|---|
| W0 | U00（Orchestrator 自身） | — | なし |
| W1 | U11（A）、U12（B）、U13（C）、U14（D）、U15（E）、U16（F） | W0 | なし |
| W2 | U21（A）、U22（B）、U23（C）、U24（D）、U25（E） | W1 | なし |
| **CP1** | Orchestrator | W2 | — |
| W3 | U31（A）、U32（B）、U33（C）、U34（D）、U35（E） | CP1 | `kivact`、`kivact-interaction`、`kivact-ui`、5 つの kit、catalog、例、雛形、`kivact-devtools`、`kivact-devtools-mcp`、`dev/tests/*`、`dev/bench` |
| W4 | U41（A）、U42（B）、U43（C）、U44（D）、U45（E） | W3 | 5 つの kit、catalog、例、雛形、`kivact-devtools`、`dev/tests/*` |
| W5 | U51（A）、U52（B）、U53（C）、U54（D）、U55（E）、U56（F） | W4 | なし |
| **CP2** | Orchestrator | W5 | — |
| W6 | U61（A）、U62（B）、U63（C） | CP2 | なし |
| **CP3** | Orchestrator | W6 | — |
| W7 | Orchestrator（W7-1〜W7-3） | CP3 | — |

---

## 2. W0：準備（Orchestrator が自分で行う）

### U00 ベースラインの計測

1. 統合 branch を checkout し、`git submodule update --init` を実行する（xtask が `vendor/xtask-core` を使う）。設計書・本書・`dev/bench`・R-0 の修正がまだ commit されていなければ、`git add SLIM_FAST_DESIGN.md SLIM_FAST_WORK_ORDER.md dev/bench Cargo.toml Cargo.lock dev/xtask/policy/workspace.toml crates/kivact-engine crates/kivact-core && git commit -m "U00: slim-fast design, work order, bench, release build fix"` で commit する。
2. `cargo build --release -p kivact-example-counter` が警告なしで成功することを確かめる（R-0 の修正済み）。失敗したら作業を止め、人間に報告する。
3. 何も描画していない状態で計測する：

```bash
dev/tools/window-lock.sh --timeout 1800 -- dev/bench/run.sh B0
```

4. `dev/bench/results/B0.md` を commit する：`git add dev/bench/results/B0.md && git commit -m "U00: KPI baseline B0"`。
5. `run.sh` が `INVALID:` で終わったら、原因（build の失敗、frame の出ない probe など）を直してから取り直す。INVALID の report を B0 にしない。
6. B0 の値が設計書 1.2 節と大きく違っても（別の機械なので当然）、そのまま進む。倍率はこの B0 に対して判定する。

---

## 3. W1（6 unit 同時）

### U11 build の設定・依存の整理・SVG の font DB の遅延生成（lane A）

- **目的**：設計書 3.4 節、4.1 節（ST-1）、4.8 節。
- **所有 file**：root `Cargo.toml`、`Cargo.lock`、`crates/kivact/Cargo.toml`、`crates/arvane/Cargo.toml`、`crates/arvane-text/Cargo.toml`、`crates/arvane-compose/Cargo.toml`、`crates/arvane-asset/src/**`、`dev/xtask/policy/workspace.toml`（`[visible_to]` の 3 行だけ）、`dev/bench/Cargo.toml`、`templates/basic/Cargo.toml`、手順 9 が挙げる manifest。

手順：

1. root `Cargo.toml` の `[workspace.dependencies]` の `parley` を `parley = { version = "0.11.1" }` にする（`features = ["complex-scripts"]` を消す）。
2. `crates/arvane-text/Cargo.toml` に次を足す：

```toml
[features]
# ICU's word and line dictionaries for scripts without spaces (Thai, Lao,
# Khmer, Myanmar, and Chinese/Japanese words); about 3 MB. The facade
# `kivact` enables it by default.
complex-scripts = ["parley/complex-scripts"]
```

3. `crates/kivact/Cargo.toml`：
   - `[dependencies]` に `arvane-text = { path = "../arvane-text", version = "0.1.0", optional = true }` を、既存の「Enable implementation features; the facade never calls these crates.」の optional 依存の並びに足す。
   - `[features]` の `default = ["gpu"]` を `default = ["gpu", "complex-scripts"]` にし、`complex-scripts = ["dep:arvane-text", "arvane-text/complex-scripts"]` を足す。
4. `crates/arvane/Cargo.toml` の `tracing = "0.1.41"` を `tracing = { workspace = true }` にする。
5. `crates/arvane-compose/Cargo.toml` から `kivact-shader = ...` の行を消す。`cargo check -p arvane-compose --all-features` が通ることを確かめる（source は使っていない）。
6. root `Cargo.toml` の末尾（`[profile.dev.package.kivact-layout]` の後）に、設計書 4.8 節の `[profile.release]` と `[profile.release.package.*]` の block をそのまま写す。
7. `dev/xtask/policy/workspace.toml` の `[visible_to]`：
   - `kivact-shader = ["kivact-style", "arvane-compose"]` を `kivact-shader = ["kivact-style"]` にし、その直前の comment 行（`# ----- Shader validation: naga is confined to kivact-shader, which style (kivact-style) and the compositor (arvane-compose) use.`）を `# ----- Shader validation: naga is confined to kivact-shader, which style (kivact-style) uses.` にする。
   - `arvane-text = ["arvane"]` を `arvane-text = ["arvane", "kivact"]` にする。
8. SVG の font DB（設計書 4.1 節）：
   1. `crates/arvane-asset/src/svg/fonts.rs` を新規に作り、設計書 4.1 節の `SvgFonts` と `has_text`（埋込み SVG の `svg+xml` も見る版）をそのまま書く（`//! Role:` 行も）。`OnceLock` と `static` は使わない。marker は付けない。
   2. `crates/arvane-asset/src/svg.rs` に `pub(crate) mod fonts;` を足し、`pub(crate) use fonts::SvgFonts;` を足す。
   3. `crates/arvane-asset/src/svg/document.rs` の `pub(crate) type FontDb = Arc<usvg::fontdb::Database>;` を `pub(crate) type FontDb = Arc<super::SvgFonts>;` にする。同じ file の `parse` の `fontdb: Arc::clone(fonts),` を `fontdb: fonts.for_document(bytes),` にする。
   4. `crates/arvane-asset/src/store/asset_store.rs` の `with_runner`：`let mut fonts = usvg::fontdb::Database::new(); fonts.load_system_fonts();` の 2 行を消し、`fonts: Arc::new(fonts),` を `fonts: Arc::new(crate::svg::SvgFonts::new()),` にする。
   5. 同じ file の 159 行付近の `Arc::make_mut(&mut self.fonts).load_font_data(bytes.to_vec());` を `self.fonts = Arc::new(self.fonts.with_font(bytes.to_vec()));` にする。
   6. `fonts` の型が `FontDb` のまま他の場所（`job.rs`）で `Arc::clone(&self.fonts)` されていることを確かめる（変更不要）。
9. `complex-scripts` の転送（設計書 4.8 節）。`kivact` に `default-features = false` で依存し `gpu` を転送している次の manifest に、`complex-scripts = ["kivact/complex-scripts"]` を `[features]` に足し、`default` の配列に `"complex-scripts"` を足す：`templates/basic/Cargo.toml`、`examples/kivact-ui-basic/Cargo.toml`、`examples/kivact-ui-pop/Cargo.toml`、`examples/kivact-ui-game/Cargo.toml`、`examples/kivact-ui-creative/Cargo.toml`、`examples/kivact-ui-soft/Cargo.toml`、`dev/tests/docs/Cargo.toml`、`dev/tests/kivact-public-api/Cargo.toml`、`crates/kivact-devtools/Cargo.toml`（`grep -rlE 'kivact = \{[^}]*default-features = false' --include=Cargo.toml . | grep -v '/crates/kivact-ui'` で漏れが無いことを確かめる。kit の library `crates/kivact-ui-*` には足さない）。
10. `dev/bench/Cargo.toml`：依存を `kivact = { path = "../../crates/kivact", default-features = false, features = ["gpu"] }` にし、`[features]` を `default = ["complex-scripts"]`、`complex-scripts = ["kivact/complex-scripts"]`、`latency = ["kivact/test"]` にする（`--no-default-features` の build が K6 の「complex-scripts なし」になる）。
11. 雛形の release profile（設計書 4.8 節）：`templates/basic/Cargo.toml` に root の `Cargo.toml` と同じ `[profile.release]` と `[profile.release.package.*]` の block を写し、その上に `# A standalone App reads profiles only from its own root manifest; these are Kivact's recommended release settings (size and speed; fat LTO lengthens release builds).` の comment を書く。
12. `cargo fmt -p arvane-asset -p arvane-text -p kivact`。

完了条件：

```bash
cargo check -p arvane-asset --all-targets
cargo check -p kivact --all-targets
cargo check -p kivact --no-default-features --features gpu
cargo check -p kivact-template-basic
cargo check -p kivact-bench --no-default-features
cargo clippy -p arvane-asset --all-targets -- -D warnings
cargo build --release -p kivact-example-counter
```

撤退規則：手順 8 で `usvg::Options` に渡す型が合わない場合は、`for_document` が返す `Arc<Database>` を `usvg::Options` の field の型に合わせて渡す（`Arc` のまま）。それでも合わなければ STOP。

### U12 状態の lint の抜け穴を塞ぐ（lane B）

- **目的**：設計書 6.2 節（SL-1〜SL-7）。
- **所有 file**：`dev/xtask/src/state_fields.rs`、`dev/xtask/src/state_fields/**`、`dev/xtask/src/check_cli.rs`、`dev/xtask/src/local_checks.rs`、`dev/xtask/policy/workspace.toml`（`[state_fields]` の table だけ）。

手順：

1. `dev/xtask/src/check_cli.rs` の `RULES` に `"state-fields-strict"` を足す（配列の長さを 10 にする）。
2. `local_checks.rs` の rule の振り分け（`"state-fields" => {..}` の腕）の隣に、`"state-fields-strict" => {..}` の腕を足す。中身は `state-fields` と同じ検査を、次の `Strictness::Strict` で呼ぶ。`state-fields` は `Strictness::Current`（今と同じ）で呼ぶ。`state-fields-strict` は、`--only` で名前を明示したときだけ実行する（`--only` なしの `cargo xtask check` には含めない。CP1 の gate を移行中の件数で失敗させないため）。
3. `state_fields` の検査の入口に引数 `strictness: Strictness`（`enum Strictness { Current, Strict }`）を足し、`Strict` のときだけ次を有効にする：
   - SL-1：`classify` で、型の最後の segment が `OnceCell`・`OnceLock`・`LazyCell`・`LazyLock` のものを stored に数える。
   - SL-2：phase の判定の enum の名前の接尾辞を、`State`・`Phase` から `State`・`Phase`・`Status`・`Stage`・`Connection`・`Lifecycle`・`Run`・`Present` に広げる（workspace の enum に限る。今の判定と同じ方法で workspace の enum かどうかを決める）。
   - SL-3：named helper（policy の `named_helpers`）のうち `Generation`・`Sent`・`Revision`・`Epoch` は、同じ struct（または named-field の variant）に他の stored field が 1 つ以上あるとき stored に数える。
   - SL-4：role の接尾辞による除外（policy の `role_suffixes` 等）を、field の型に `Cell`・`RefCell`・`Signal`・`Var`・`Mutex`・`RwLock` を含む struct には適用しない。
   - SL-5：value module の除外の一覧から `kivact-engine::contract` を除く（`kivact-types` と `kivact-devtools-protocol` は除外のまま）。
   - SL-6：`Vec`・`VecDeque`・`BTreeSet`・`HashSet` の field で、名前が `pending`・`dirty`・`awaiting`・`relayout`・`stale` で始まるものを stored に数える。
   - SL-7：`locals.rs` の local の検出（今は `--report` で note を出すだけ）を、`Strict` では、同じ関数の中に検出対象の local が 2 個以上あるとき error にする。各 local の直前の行に `// xtask-allow(state-fields): <理由>` があれば、その local を数えない。検出対象に `var(..)`、`Rc::new(RefCell::new(None))`、`signal(<f64 の literal>)` を足す。
   - SL-8：marker の理由が `independent:`、`projection:`、`cache:`、`identity:`、`statistic:`、`option:` のどれかで始まらなければ error（設計書 6.2 節）。
4. error の message は今の `state-fields` と同じ書式にし、先頭の rule 名だけ `state-fields-strict` にする。
5. marker の「使われていない」判定（`state_fields/check.rs:297-313`）：`Current` でも、`Strict` の規則がその field を数えるなら、その marker は使われているとみなす（W3〜W5 で strict のために付ける marker が、`--only state-fields` と CP の `cargo xtask check` で unused の error にならないようにする）。
6. 既存の xtask の試験（`dev/xtask/src/**` の `#[cfg(test)]`）を壊さない。新しい試験は足さない（xtask は開発道具なので、既存の試験の書き方に合わせて 1 つだけ足してよい：`Strict` で `OnceCell` が数えられること）。
7. `cargo fmt -p xtask`。

完了条件：

```bash
cargo check -p xtask --all-targets
cargo clippy -p xtask --all-targets -- -D warnings
cargo test -p xtask
cargo xtask check --no-write --only state-fields          # 今と同じく 0 件
cargo xtask check --no-write --only state-fields-strict --report > dev/xtask/strict-report.txt
```

最後のコマンドの出力を `dev/xtask/strict-report.txt` として commit する（W3〜W5 の unit が自分の crate の件数を見るために使う。W6 で消す）。

### U13 layout の O(changed) 化（lane C）

- **目的**：設計書 4.4 節（LAY-1、LAY-2）、4.7 節 MEM-7（`NodeStyle` の上限）。
- **所有 file**：`crates/kivact-layout/src/**`。

手順：

1. `columns.rs` の `NodeLayoutValues` に 3 つの列 `touched: Vec<u32>`、`below: Vec<u32>`、`origin: Vec<(f32, f32)>` と、`scaled: BTreeSet<usize>` を足す。`NodeLayoutColumns` に `stamp: u32` を足す。
   - `Default` で viewport の slot 0 の分を 1 要素ずつ入れる（`touched: vec![0]`、`below: vec![0]`、`origin: vec![(f32::NAN, f32::NAN)]`）。
   - `push()` で `touched.push(0)`、`below.push(0)`、`origin.push((f32::NAN, f32::NAN))`。
   - `reset(slot)` で `touched[slot + 1] = 0`、`below[slot + 1] = 0`、`origin[slot + 1] = (f32::NAN, f32::NAN)`、`scaled.remove(&(slot + 1))`。
2. stamp の進め方：`NodeLayoutColumns` に `fn next_stamp(&mut self) -> u32 { self.stamp = self.stamp.wrapping_add(1).max(1); self.stamp }` を足す。`compute()` の先頭（`self.values.geometry_changed.clear();` の次）で 1 回呼ぶ。`Layout::compute_overlay`（`layout.rs` の公開 method）は `self.columns.compute_overlay(..)` の前に `self.columns.next_stamp()` を呼ぶ。`NodeLayoutColumns::compute_overlay` と `compute_at` と `relayout_parents` は stamp を進めない。
3. 印を付ける関数を `tree.rs` に足す（`LayoutPass` と `NodeLayoutColumns` の両方から使う一つの関数）：

```rust
/// Record that column `index` got a new unrounded layout in pass `stamp`,
/// and that every ancestor (up to the viewport, column 0) has such a node below it.
fn touch(tree: &(impl LayoutTree + ?Sized), touched: &mut [u32], below: &mut [u32], stamp: u32, index: usize) {
    touched[index] = stamp;
    let mut at = Some(index);
    while let Some(i) = at {
        if below[i] == stamp {
            break;
        }
        below[i] = stamp;
        at = if i == 0 { None } else { Some(tree.parent(i - 1).map_or(0, |p| p + 1)) };
    }
}
```

4. `LayoutPass` に `touched: &'a mut [u32]`、`below: &'a mut [u32]`、`origin: &'a mut [(f32, f32)]`、`stamp: u32` の field を足し、`with_pass` で `NodeLayoutValues` の分解に 3 列を足して渡す（`stamp` は `self.stamp`）。
5. `LayoutPass::set_unrounded_layout`（`tree.rs:277`）を次にする：

```rust
fn set_unrounded_layout(&mut self, t: TId, layout: &Layout) {
    let index = usize::from(t);
    // Taffy sets every child again when it lays out the parent: mark only a
    // new value, or a slot that was never rounded (its origin is NaN).
    let changed = self.unrounded[index] != *layout || self.origin[index].0.is_nan();
    self.unrounded[index] = *layout;
    if changed {
        touch(self.tree, self.touched, self.below, self.stamp, index);
    }
}
```

6. `relayout_parents`（`tree.rs:686`）の `self.values.unrounded[at].scrollable_overflow_rect = after.scrollable_overflow_rect;` の直後に `touch(tree, &mut self.values.touched, &mut self.values.below, self.stamp, at);` を足す。
7. `tree.rs` に丸めの関数を足す：
   - `fn rounded(unrounded: Layout, cum: (f32, f32)) -> Layout`：taffy 0.14 の `round_layout_inner`（`~/.cargo/registry/src/*/taffy-0.14.0/src/compute/mod.rs` の 223〜253 行と、`round_scrollable_overflow_rect`）の `layout` の計算を写す。`cumulative_x` は `cum.0 + unrounded.location.x`、`cumulative_y` は `cum.1 + unrounded.location.y`（taffy は関数の先頭で足してから使う。同じ順にする）。`round` は `f32::round`。
   - 設計書 4.4 節の `round_changed` を、`LayoutPass` の method として書く（`self.origin`、`self.below`、`self.touched`、`self.unrounded`、`self.set_final_layout`、`self.child_count`、`self.get_child_id` を使う）。
8. `compute_at` の `taffy::round_layout(pass, root);` を `pass.round_changed(root, (0.0, 0.0));` にする（stamp は `pass.stamp`）。
9. LAY-2：`Layout::set_presence_scale`（`layout.rs`）で、`*current = scale;` の後に `if scale.is_some() { self.columns.values.scaled.insert(slot + 1); } else { self.columns.values.scaled.remove(&(slot + 1)); }` を足す。`scale_item_styles` の引数に `scaled: &BTreeSet<usize>` を足し、`for (index, scale) in presence_scale.iter().enumerate()` を `for &index in scaled { let scale = presence_scale[index]; ..` にする（中身は同じ）。`with_pass` の呼び出しで `scaled` を渡す。
10. `style/node.rs` の `NodeStyle` の定義の直後に `const _: () = assert!(std::mem::size_of::<NodeStyle>() <= 264, "NodeStyle grew: box the new rare field");` を足す。
11. `cargo fmt -p kivact-layout`。

完了条件：

```bash
cargo check -p kivact-layout --all-targets
cargo check -p kivact-engine --all-targets
cargo clippy -p kivact-layout --all-targets -- -D warnings
cargo xtask check --no-write --only state-fields --crate kivact-layout
```

撤退規則：CP1 で layout の E2E が 1 px 単位で失敗し、2 回直しても通らない場合は、手順 8 を `taffy::round_layout` に戻す（手順 1〜6 は残してよい）。LAY-2 は残す。

### U14 CPU 合成の高速経路（lane D）

- **目的**：設計書 4.2 節（CPU-1）。
- **所有 file**：`crates/arvane-compose/src/**`。

手順（CPU-1）：

1. `cpu/draw.rs` の `Shape::coverage`（137 行付近）を読み、整数座標の軸平行の矩形の内側の画素中心で 1.0、外側で 0.0 を返すことを確かめる（`clamp(d + 0.5, 0, 1)` の形）。違えば STOP。
2. `cpu/draw.rs` に関数を足す：

```rust
/// The integer translation `(tx, ty)` of `map` (target px to texel), when it is one.
fn integer_translation(map: &Projective) -> Option<(i64, i64)> {
    let m = &map.m;
    let unit = m[0][0] == 1.0 && m[0][1] == 0.0 && m[1][0] == 0.0 && m[1][1] == 1.0
        && m[2][0] == 0.0 && m[2][1] == 0.0 && m[2][2] == 1.0;
    let (tx, ty) = (m[0][2], m[1][2]);
    (unit && tx.is_finite() && ty.is_finite() && tx.fract() == 0.0 && ty.fract() == 0.0)
        .then(|| (tx as i64, ty as i64))
}

/// The pixel rectangle `[x0, x1) x [y0, y1)` of `points` when they are the
/// four distinct corners of an axis-aligned rectangle on integer coordinates.
fn pixel_rect(points: &[Point]) -> Option<(i64, i64, i64, i64)> {
    if points.len() != 4 || points.iter().any(|p| p.x.fract() != 0.0 || p.y.fract() != 0.0) {
        return None;
    }
    let x0 = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let x1 = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let y0 = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let y1 = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let corner = |x: f64, y: f64| points.iter().any(|p| p.x == x && p.y == y);
    let all = corner(x0, y0) && corner(x1, y0) && corner(x1, y1) && corner(x0, y1);
    (all && x0 < x1 && y0 < y1).then(|| (x0 as i64, x1 as i64, y0 as i64, y1 as i64))
}
```

3. 同じ file に高速経路を足す（`Composite::SourceOver` は `kivact-types/src/render/composition.rs:129` の名前。`PixelRect` の field の型が `u32` でなければ型を合わせる）：

```rust
/// `paint` for a source-over copy of a premultiplied texture moved by whole
/// pixels into a pixel-aligned rectangle (PERF-3): integer src-over per byte.
/// None when the op is not of that kind.
fn paint_fast(dst: &mut Pixmap, op: &DrawOp<'_>, texture: &Texture<'_>, cover: Option<&Pixmap>) -> Option<()> {
    let Texture::Premul(src) = texture else { return None };
    if cover.is_some() || op.opacity != 1.0 || op.blend != BlendMode::Normal || op.composite != Composite::SourceOver {
        return None;
    }
    let (tx, ty) = integer_translation(&op.map)?;
    let (x0, x1, y0, y1) = pixel_rect(&op.polygon.points)?;
    let (w, h) = dst.size();
    let (sw, sh) = src.size();
    let (x0, y0) = (x0.max(0), y0.max(0));
    let (x1, y1) = (x1.min(i64::from(w)), y1.min(i64::from(h)));
    for y in y0..y1 {
        let sy = y + ty;
        if sy < 0 || sy >= i64::from(sh) {
            continue;
        }
        for x in x0..x1 {
            if let Scissor::Rects(rects) = &op.scissor
                && !rects.iter().any(|r| inside(r, x as u32, y as u32))
            {
                continue;
            }
            let sx = x + tx;
            if sx < 0 || sx >= i64::from(sw) {
                continue;
            }
            dst.blend_src_over(x as u32, y as u32, src.byte_texel(sx as u32, sy as u32));
        }
    }
    Some(())
}
```

   （scissor の矩形は重なることがあるので、汎用 loop と同じく画素ごとに `inside` で調べる。矩形ごとに loop すると半透明の画素を二度合成する。）
4. `cpu/surface.rs` の `Pixmap` に 2 つの method を足す：

```rust
/// The premultiplied bytes at (`x`, `y`), which must be inside.
pub(crate) fn byte_texel(&self, x: u32, y: u32) -> [u8; 4] {
    let i = (y as usize * self.width as usize + x as usize) * 4;
    [self.data[i], self.data[i + 1], self.data[i + 2], self.data[i + 3]]
}

/// Source-over of the premultiplied bytes `s` at (`x`, `y`), which must be inside.
pub(crate) fn blend_src_over(&mut self, x: u32, y: u32, s: [u8; 4]) {
    let i = (y as usize * self.width as usize + x as usize) * 4;
    let a = u16::from(s[3]);
    if a == 255 {
        self.data[i..i + 4].copy_from_slice(&s);
        return;
    }
    if a == 0 {
        return;
    }
    for c in 0..4 {
        let d = u16::from(self.data[i + c]);
        self.data[i + c] = (u16::from(s[c]) + (d * (255 - a) + 127) / 255).min(255) as u8;
    }
}
```

5. `paint` の先頭（`let (w, h) = dst.size();` の前）に `if paint_fast(dst, op, texture, cover).is_some() { return Ok(()); }` を足す。
6. `paint_fast` が 80 code 行を超えたら、行の loop を `fn blend_row(..)` に分ける。

GPU-2（pipeline の遅延生成）は行わない（設計書 4.3 節。初回の描画中に compile が走るため）。

7. `cargo fmt -p arvane-compose`。

完了条件：

```bash
cargo check -p arvane-compose --all-targets
cargo check -p arvane-compose --all-targets --features gpu
cargo check -p arvane-compose --all-targets --features gpu-interop
cargo clippy -p arvane-compose --all-targets --all-features -- -D warnings
cargo xtask check --no-write --only state-fields --crate arvane-compose
```

撤退規則：CP1 で画素比較の E2E が高速経路のせいで失敗し、2 回直しても通らない場合は、手順 5 の 1 行を消して高速経路をすべて止める（chunk の種類で不透明を推測して部分的に残すことはしない）。`OPEN.md` に記録する（K1・K4 の必達に影響するので、受入れの前に設計者が判断する）。

### U15 疎な `Style`（lane E）

- **目的**：設計書 4.7 節 MEM-1（8 つの群の疎な struct）、MEM-7（`Style` の上限）。
- **所有 file**：`crates/kivact-style/src/**`。

手順：

1. `sheet.rs` に、設計書 MEM-1 の表の 8 つの struct（`SizeFields`、`SpacingFields`、`FlowFields`、`PaintFields`、`TransformFields`、`TextFields`、`StateFields`、`MotionFields`）を作る。各 struct の field は、表の「入る field」の今の `Style` の field を、名前・型・doc comment・`xtask-allow` の行ごとそのまま移したもの。`#[derive(Clone, Debug, Default, PartialEq)]`。field は `pub(crate)`。
2. 今の `Style` の struct を設計書 MEM-1 の形（8 つの `Option<Box<..>>` と `duplicates: u32`）にする。derive は `Clone, Debug, Default` だけにする。
3. `PartialEq for Style` を手で書く：各群について `self.<群>.as_deref().cloned().unwrap_or_default() == other.<群>.as_deref().cloned().unwrap_or_default()`、かつ `self.duplicates == other.duplicates`（今の dense な derive の比較と同じ結果）。
4. 各群の struct に `pub(crate) fn merge(&mut self, o: &Self)` を書く。本体は今の `Style::merge`（`sheet.rs:1144`）の `take!` の並びと `merge_compound_properties`（`:1222`）のうち、その群の field の行をそのまま移したもの（`self.x` と `o.x` はそのまま）。
5. `Style` に群ごとの `fn <群>_mut(&mut self) -> &mut <群>Fields { self.<群>.get_or_insert_with(Default::default) }` を 8 つ足す。
6. `Style::merge(mut self, o: &Style) -> Style` を書き直す：各群について `if let Some(group) = o.<群>.as_deref() { self.<群>_mut().merge(group); }`。`duplicates` は今の `merge` と同じ扱い（今の本体を読み、同じ式を残す）。
7. setter：
   - 手で書いた setter は、`self.<field>` を、その field の群の `self.<群>_mut().<field>` にする。`self.<field>.is_some()` を見て `self.duplicates += 1` とする行は、`self.<群>.as_deref().is_some_and(|g| g.<field>.is_some())` にする（群を作らずに調べる）。
   - macro（`setter!`、`color_setter!`、`sides!` など）は、引数に群の名前を足し（`setter!(width, size, width, ..)`）、本体を同じ規則で一度だけ書き換える。macro の呼び出しの各行に、設計書の表の群の名前を足す。
8. 読み手（`resolve.rs`、`text.rs`、`internal.rs` など `kivact-style` の中で `Style` の field を読む関数）：`sheet.rs` に設計書 MEM-1 の `group_or_default!` の macro を一度書き、各関数の最初で必要な群を取り出し（`let size = group_or_default!(style.size, SizeFields);` の形）、`s.width` を `size.width` のように読む。外部に公開されている入口（`kivact_style::internal::*`、`pub fn`）の署名は変えない。
9. 入れ子の状態 style（`StateFields` の `hover` など、`Option<Box<Style>>`）は新しい `Style` のまま。
10. `sheet.rs` の `Style` の定義の直後に `const _: () = assert!(std::mem::size_of::<Style>() <= 72, "Style keeps its groups boxed");` を足す。
11. 群の struct に移した field の `xtask-allow(state-fields): passive record - <field 名>` の行は、`// xtask-allow(state-fields): option: one authored property; absent means the inherited or default value` に置き換える（lint が報告する field だけ。0.2 の 9）。
12. `cargo fmt -p kivact-style`。

完了条件：

```bash
cargo check -p kivact-style --all-targets
cargo check -p kivact-core --all-targets
cargo check -p kivact-ui --all-targets
cargo clippy -p kivact-style --all-targets -- -D warnings
cargo xtask check --no-write --only state-fields --crate kivact-style
```

撤退規則：無し（意味は今と同じになる設計なので、compile が通らなければ STOP）。

### U16 runtime：`Binding::update` と runtime の状態機械（lane F）

- **目的**：設計書 5.4 節（`Binding::update`）、6 章（付録 A の A-RT）。
- **所有 file**：`crates/kivact-runtime/src/**`。

手順：

1. `reactive/handles.rs` の `impl<T: crate::PropValue> Binding<T>`（`get`・`set` がある impl）に足す（設計書 5.4 節）：

```rust
/// Change the value and write it back. A binding of a `Signal` changes it
/// in place and notifies even when the value is equal (`Signal::update`);
/// a `binding(get, set)` reads the current value, applies `f` to a clone
/// and calls `set` once (the setter decides whether readers are notified).
#[track_caller]
pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
    match &self.repr {
        BindingRepr::Signal(signal) => signal.update(f),
        _ => {
            let mut value = self.get_untracked();
            let result = f(&mut value);
            self.set(value);
            result
        }
    }
}
```

   `BindingRepr` の variant の名前が違えば、`Signal` を持つ variant に合わせる。`Signal::update` の署名（`handles.rs:148`）が `R` を返さなければ STOP。
2. `reactive/props.rs` の `impl<T: 'static> Reactive<T>` の後に足す（`#[component]` が owner なしで text を保持するため、設計書 5.2 節）：

```rust
impl Reactive<String> {
    /// A text input (as `text(..)` takes it), stored without an owner.
    pub fn from_text(input: impl IntoText) -> Self {
        Reactive(input.into_text())
    }
}
```
3. 付録 A の「A-RT」の表の各行を、表の「直した形」のとおりに直す。表が「SM-11（理由）」と書いた行は、field の直前の marker の理由を表の文言に置き換えるだけにする（marker が無く、lint も報告しない field には付けない）。`AnimInfo` の `running`・`paused` は `crates/kivact-core/src/devtools/motion.rs:37-38`（feature `devtools`）が読むので、その file を追従してよい。
4. `cargo fmt -p kivact-runtime`。

完了条件：

```bash
cargo check -p kivact-runtime --all-targets
cargo check -p kivact-runtime --all-targets --features tokio
cargo check -p kivact-core --all-targets --all-features
cargo clippy -p kivact-runtime --all-targets --all-features -- -D warnings
cargo xtask check --no-write --only state-fields --crate kivact-runtime
```

---

## 4. W2（5 unit 同時）

### U21 paint list の O(changed) 化（lane A）

- **目的**：設計書 4.5 節（PNT-1、PNT-2）、MEM-7（`Look` の上限）。
- **所有 file**：`crates/kivact-engine/src/layer.rs`、`crates/kivact-engine/src/paint/**`、`crates/kivact-engine/src/tree/**`、`crates/kivact-engine/src/model/**`（`model/interact/geometry.rs` を除く。U22 が所有する）、`crates/kivact-engine/src/mutation.rs`。

手順（PNT-2、先に行う）：

1. `tree/node_tree.rs` に設計書 4.5 節の `forget_flat_around`（`&self` と `self.flat.borrow_mut()` を使う形）を足す。
2. 全消去の 3 か所を置き換える：
   - `install`（`node_tree.rs:320`）の `self.flat.get_mut().clear();` を、`if self.nodes.index_of(id).is_some() { self.forget_flat_around(id); }` にする（既存の id の置き換えだけ捨てる。存在を調べる method の名前が違えば、`NodeStore` の同じ意味の method を使う）。この行は、置き換えの前の node が tree にあるうちに実行する（`install` の先頭）。
   - `insert`（`:415`）の `self.flat.get_mut().clear();` を、子をつないだ後に `self.forget_flat_around(<つないだ親の NodeId>)` にする。
   - `detach`（`:446`）の `self.flat.get_mut().clear();` を、外す前に親の id を読んでおき、外した後に `self.forget_flat_around(<元の親の NodeId>)` にする。
   - `remove_with`（`:472`、`detach` の後に subtree を消す）で、消す各 id について `self.flat.borrow_mut().remove(&id);` を呼ぶ（今は `detach` の全消去がこの掃除を兼ねていた。部分的な破棄にすると、消したノードの entry が残り続けるため）。
3. 他に親子をつなぎ変える `NodeTree` の method（`grep -rnE "\.parent = |ChildOrder|child_order" crates/kivact-engine/src/tree` で探す）があれば、変わった親（移動なら両方）について同じく呼ぶ。
4. `forget_flat()`（`node_tree.rs:111`、全消去）とその呼び出し（`model/apply.rs`、透明さが変わるとき）は残す。

手順（PNT-1）：

5. `paint/state.rs`（`PaintState` がある file）に設計書 4.5 節の `StaleFacts` を足し、`PaintState` に `pub(crate) stale_facts: StaleFacts` を足す（既定は `All`）。marker は付けない。
6. `PaintState` に method を足す（`stale_fact` の引数は呼び出し側で「印の正規化」をした id。下の手順 10 の前置きを参照）：

```rust
/// Derive `id`'s subtree facts again before the next list (and its ancestors as they change).
pub(crate) fn stale_fact(&mut self, id: NodeId) {
    if let StaleFacts::Nodes(set) = &mut self.stale_facts {
        set.insert(id);
    }
}
/// Derive every subtree fact again before the next list.
pub(crate) fn stale_all_facts(&mut self) {
    self.stale_facts = StaleFacts::All;
}
```

7. `layer.rs`：
   - `Look` の derive に `PartialEq` を足す。`Look` の定義の直後に `const _: () = assert!(std::mem::size_of::<Look>() <= 72, "Look grew: box the new rare field");` を足す。
   - 設計書 4.5 節の `node_facts` を足す。本体は `subtree_facts` の本体を写し、`for child in view.flat_children(id)` の中の再帰 `subtree_facts(view, child, Some(&look), out)` を `facts.get(&child).copied().unwrap_or_default()` に、最後の `out.insert` を削除（呼び出し側で入れる）にしたもの。
8. `paint/list/walk.rs` の `paint_list` の `list.facts = self.layer_facts();` を、`let stale = std::mem::replace(&mut self.paint.stale_facts, StaleFacts::Nodes(BTreeSet::new())); self.refresh_facts(stale, &mut list.facts);` にする。`refresh_facts` を同じ `walk.rs` の `impl Model` に書く（engine の module の規則で `layer.rs` は `lower` しか使えないので、`layer.rs` には置かない）：

```rust
/// Bring `facts` up to date with the nodes marked stale since the last list.
pub(crate) fn refresh_facts(&self, stale: StaleFacts, facts: &mut HashMap<NodeId, SubtreeFacts>) {
    match stale {
        StaleFacts::All => *facts = self.layer_facts(),
        StaleFacts::Nodes(set) => {
            let tree = self.tree();
            let mut queue: BinaryHeap<(usize, NodeId)> =
                set.into_iter().filter(|id| tree.contains(*id)).map(|id| (tree.depth(id), id)).collect();
            let mut done = BTreeSet::new();
            while let Some((_, id)) = queue.pop() {
                if !done.insert(id) {
                    continue;
                }
                let parent = tree.get(id).and_then(|n| tree.flat_parent(n));
                let parent_look = parent.and_then(|p| self.drawn_look(p));
                let new = crate::layer::node_facts(self, id, parent_look.as_ref(), facts);
                let old = facts.get(&id).copied().unwrap_or_default();
                if new == SubtreeFacts::default() { facts.remove(&id); } else { facts.insert(id, new); }
                if new != old && !self.is_layer_node(id) && let Some(p) = parent {
                    queue.push((tree.depth(p), p));
                }
            }
        }
    }
    #[cfg(debug_assertions)]
    debug_assert_eq!(*facts, self.layer_facts(), "paint: retained subtree facts diverged");
}
```

   - `tree.depth(id)`（root からの祖先の数）が無ければ `NodeTree` に足す（`parent_id` を辿って数える）。
   - `drawn_look`、`is_layer_node` は `LayerView` の実装（`Model`）の method を使う。名前が違えば同じ意味の method を使う。
9. `paint/list/self_check.rs` の self-check（空の `PaintList` から `paint_list()` を呼んで参照の list を作る箇所）で、`paint_list()` を呼ぶ前に `let kept = std::mem::replace(&mut self.paint.stale_facts, StaleFacts::All);` とし、self-check の最後に `self.paint.stale_facts = kept;` で戻す（`paint_list()` を複数回呼ぶなら、各呼び出しの前に `All` にする）。
10. 印を付ける（設計書 4.5 節の一覧）。**印の正規化**：`Model` に `fn stale_fact(&mut self, id: NodeId)` を置き、`id` が透明なノード（`is_transparent()`、`display: contents`）なら最も近い透明でない祖先（`self.tree.flat_parent(n)`）に、そうでなければ `id` に `self.paint.stale_fact(..)` を呼ぶ。以下の印はすべてこの `Model::stale_fact` を通す。子を追加・移動した箇所では、変わった親と、追加・移動した子自身の両方に印を付ける。削除では、`Removed` の id の列（ADR-29）の各 id を `list.facts` から `remove` する（`refresh_facts` の外で、削除を処理する箇所で行う。`facts` 全体の `retain` はしない）。
    1. `model/interact/sync.rs` の `derive(id)`：`self.paint.look[..].replace(look)` の結果 `kept` が `Some(look)` と違うとき（`kept != Some(look)`）、`self.paint.stale_fact(id)` と、`let children: Vec<NodeId> = self.tree.flat_children(id).collect();` の各 `child` に `self.paint.stale_fact(child)` を呼ぶ。
    2. 同じ `sync` の `restyled`（`sync.rs:163-166`、`changes.marked(Dirty::REDRAW)` のうち `states()` を持つノード）の各ノードとその `flat_children` に stale_fact を呼ぶ。
    3. `model/frame/ticks.rs:159` の `restack |= motion.is_active()`：motion の state が動いているノードの一覧を持っていれば各ノードとその `flat_children` に stale_fact、持っていなければ `self.paint.stale_all_facts()`。
    4. 手順 2〜3 で `forget_flat_around(p)` を呼んだ箇所に対応する `Model` 側の呼び出し元で、`p` に stale_fact を呼ぶ（`NodeTree` の中からは `PaintState` に触れないので、`Model` 側で呼ぶ）。
    5. canvas・draw の scene を設定する箇所（`grep -rnE "scenes\[" crates/kivact-engine/src/model`）で、そのノードに stale_fact。
    6. `hidden`・`visibility` を書く箇所、`model/apply.rs` の透明さの変化（`forget_flat()` を呼ぶ箇所）、root・overlay・fixed の集合が変わる箇所で `self.paint.stale_all_facts()`。
    7. 上のどれにも当たらない `self.changes.restack = true` / `changes.restack |= ..` の箇所で、対象のノードが分かれば stale_fact、分からなければ `stale_all_facts`。
11. `cargo fmt -p kivact-engine`。

完了条件：

```bash
cargo check -p kivact-engine --all-targets
cargo check -p kivact-engine --all-targets --features testing
cargo check -p kivact-engine --release
cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings
cargo xtask check --no-write --only state-fields --crate kivact-engine
```

撤退規則：CP1 の E2E で `paint: retained subtree facts diverged` の panic が出たら、その試験が行った操作の種類（scroll、hover、追加、animation…）に当たる手順 10 の箇所を `stale_all_facts` に変える。

### U22 a11y：geometry だけの変化（lane B）

- **目的**：設計書 4.6 節（A11Y-1）。
- **所有 file**：`crates/kivact-engine/src/a11y/**`、`crates/kivact-engine/src/model/interact/geometry.rs`。

手順：

1. `a11y/state.rs` の `Dirt` に設計書 4.6 節の `Geometry` variant を足す。
2. `A11yState` に `moved` を足す（設計書 4.6 節の規則）。
3. `metadata_dirt`（`state.rs:123`）の `match` に `Dirt::Geometry { nodes, subtrees }` の腕を足し、`Dirt::Metadata { nodes, subtrees, removed: BTreeSet::new() }` に昇格してから、今の `Metadata` の腕と同じ扱いをする。publish の受理（accepted の更新）の処理で `LocalText` が「metadata を更新しない」なら、`Geometry` も同じにする。`Dirt` を `match` する他の箇所（`grep -rnE "Dirt::" crates/kivact-engine/src/a11y`）でも、`Geometry` を `LocalText` と同じ側（identity metadata を再利用する側）で扱う。
4. `model/interact/geometry.rs` の 2 か所：`for change in &geometry { self.a11y.subtree(change.id); }` を `self.a11y.moved(change.id)` に、scroll の `self.a11y.subtree(id);`（660 行付近）を `self.a11y.moved(id);` にする。
5. `a11y/build/dirty.rs` の `build_semantics_for` の `if let Dirt::LocalText { nodes } = dirt {..}` の次に、`Dirt::Geometry { nodes, subtrees }` の経路を足す。中身は今の `Metadata` の経路（同じ関数の残り）を写し、次の 3 点だけを変える：
   1. 最初に `let clipped = self.clipped_rows();` とし、`clipped != self.a11y.metadata.clipped` なら `let promoted = Dirt::Metadata { nodes: nodes.clone(), subtrees: subtrees.clone(), removed: BTreeSet::new() }; return self.build_semantics_for(engine, &promoted);`。
   2. `let (metadata, order) = self.build_identity_metadata();` の代わりに `let metadata = Rc::clone(&self.a11y.metadata); let order = self.semantic_order();` を使う（`semantic_order` の名前が違えば、`build_identity_metadata` の中で `order` を作っている関数を使う）。
   3. `metadata.children`・`metadata.parents` を前回と比べて `dirty` に足す loop（今の経路の 57〜71 行付近）と `modal_changed` の計算は行わない（`modal_changed` は false）。`inherited` の loop、`expand_overlay_dirt`、`expand_relation_dirt`、`invalidate_boxless_ancestors`、`build_dirty_nodes(root, false, &dirty, &metadata, engine)`、`semantics_focus(&metadata.clipped)` は今の経路と同じに呼ぶ。最後は `Built { tree, metadata: None }` を返す。
   この経路の関数が 80 code 行を超えるなら、今の経路と共通の後半（`inherited` から `Built` まで）を一つの helper に分け、両方の経路から呼ぶ。
6. `cargo fmt -p kivact-engine`。

完了条件：U21 と同じ 5 つのコマンド。

撤退規則：CP1 の a11y の E2E が失敗し 2 回直しても通らない場合は、手順 4 を元に戻す（`moved` と `Geometry` の経路は残してよい）。

### U23 core のノードごとのメモリ（lane C）

- **目的**：設計書 4.7 節 MEM-4（core）。既定値を保持しない（MEM-2、MEM-3）は engine 側で行うので U34 が行う。
- **所有 file**：`crates/kivact-core/src/view/mount/**`、`crates/kivact-core/src/view/element_data.rs`、`crates/kivact-core/src/view/style_layer.rs`。

手順：

1. `view/mount/resolve.rs` の `LocalPaintSource { value: Rc::new(RefCell::new(None)), .. }` の `value` の型を、`T` が 64 byte を超えるなら `Rc<RefCell<Option<Box<T>>>>` にする（M-R1）。読み書きする箇所（`grep -rnE "local_paint" crates/kivact-core/src`）を `Box` に合わせる。
2. `dev/bench` の `kv_heap` で残りの割当て元を確かめる（release build、CPU。line table を残すため strip を外す）：

```bash
CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release -p kivact-bench --bin kv_heap
dev/tools/window-lock.sh --timeout 900 -- env ARVANE_RENDERER=cpu MODE=box ROWS=3000 <target>/release/kv_heap 2>&1 | grep '^H ' | head -25
dev/tools/window-lock.sh --timeout 900 -- env ARVANE_RENDERER=cpu MODE=button ROWS=3000 <target>/release/kv_heap 2>&1 | grep '^H ' | head -25
```

   （`kv_heap` は window を 6 秒開いて閉じる。この unit では 4 回まで実行してよい。）
3. 出力の割当て元のうち、名前（generic の型引数を含む）に `kivact_core` を含み 300 B/item を超えるものについて、その関数（`alloc::raw_vec::RawVec::grow_one<T>` の行なら、`T` の値を `push` している core の箇所）の `Rc::new`、`Box::new`、`Vec` の確保を見て、M-R1〜M-R4（設計書 4.7 節 MEM-4）を当てはめて直す。当てはまらないもの、特に「mount の後も全ノード分の要素を持ち続ける `Vec`」（M-R5、分析機では button 1 個あたり 14 KB の `Vec<SemanticsPatch>` があった）は、要素を消してよいかを判断せず STOP で、割当て元の名前と B/item を報告する。
4. `cargo fmt -p kivact-core`。

完了条件：

```bash
cargo check -p kivact-core --all-targets
cargo check -p kivact-core --all-targets --all-features
cargo check -p kivact-ui --all-targets
cargo clippy -p kivact-core --all-targets --all-features -- -D warnings
cargo xtask check --no-write --only state-fields,effect-writes --crate kivact-core
```

報告に、手順 2 の最初と最後の `kv_heap` の上位 10 行（box と button）を貼る。

### U24 GPU を必要時に起動する（lane D）

- **目的**：設計書 4.3 節（GPU-1、4.3.1）。
- **所有 file**：`crates/kivact-types/src/gpu.rs`、`crates/kivact-types/src/render/**`、`crates/arvane/src/**`、`crates/arvane/tests/**`、`crates/kivact-core/src/gpu_interop.rs`、`crates/kivact-core/src/gpu_interop/**`、`crates/kivact-core/src/view/elements/viewport.rs`、`crates/kivact-core/src/view/drawable*.rs`、`crates/kivact-core/src/view/drawable/**`、`crates/kivact-engine/src/contract/host.rs`、`crates/kivact-engine/src/host/**`。

手順：

1. `kivact-types/src/gpu.rs` に設計書 4.3 節の `GpuStart` を足し、`GpuPreference` に `pub start: GpuStart` を足す（`Default` は derive のままで `OnDemand` になる）。`GpuPreference` を struct の literal で作っている箇所（`grep -rnE "GpuPreference \{" crates dev docs examples templates`。分析機では `docs/for-kivact-user/en/how-to-guide/gpu-texture-sharing.md:351`、`jp/..:284` にもある）に `start: GpuStart::OnDemand`（または `..Default::default()`）を足す。この追従は所有外の file（文書を含む）でも行ってよい。
2. `kivact-core/src/gpu_interop.rs`：`GpuPreference` を再公開している `pub use` に `GpuStart` を足す。
3. `kivact-types/src/render/` の `Composition` に `pub fn needs_gpu(&self) -> bool` を足す（設計書 4.3.1 節）：user shader を参照する layer の effect（`LayerFilter` の shader の variant と paint shader）があれば true。外部の GPU texture と GPU export は判定しない（手順 10 の需要の伝達で扱う）。
4. `arvane/src/env.rs` の `ARVANE_RENDERER` の解釈を設計書 4.3 節の優先順位の表のとおりにする：`cpu` は GPU を使わない（今と同じ）、`gpu` は `AtLaunch`、`auto` は `OnDemand`、未設定は `GpuPreference.start`。
5. `arvane/src/presenter/gpu.rs`：
   - `enum State` に設計書 4.3 節の `Deferred { slow: u8, waker: ShellWaker }` を足す（`waker` の型は `start_gpu` が受け取る wake の型に合わせる）。
   - `Gpu::start`（346 行）の先頭の `if !matches!(self.state, State::Off) { return; }` を `if !matches!(self.state, State::Off | State::Deferred { .. }) { return; }` にする。
   - `pub(crate) fn defer(&mut self, waker: ..)`（`State::Deferred { slow: 0, waker }` にする）と `pub(crate) fn note_cpu_frame(&mut self, spent: Duration, period: Duration, needs_gpu: bool, notes: &mut Notes)` を足す。`note_cpu_frame` は `State::Deferred { slow, waker }` のときだけ働く：`needs_gpu` か、`spent > period / 2` の frame が 8 回続いたら、`waker` を取り出して `start`（今の `start_gpu` の本体）を呼ぶ。`spent <= period / 2` なら `slow = 0`、超えたら `slow += 1`。
6. `arvane/src/window.rs`：
   - `Driver::start`（416 行付近）の `if self.gpu { presenter.start_gpu(waker.clone()); }` を、`self.gpu` が真で `start == AtLaunch` なら今どおり、`start == OnDemand` なら `presenter.defer_gpu(waker.clone())`（`Gpu::defer` を呼ぶ presenter の method）にする。`start` は `Shell::gpu_preference` で受け取った値の `start`。`ARVANE_RENDERER=gpu` なら `AtLaunch` とみなす。
   - `Windows::open`（224〜226 行付近、二つ目以降の window）の `start_gpu` の呼び出しも同じ規則にする。
7. `arvane/src/presenter/frame_policy.rs` の CPU の frame を present した後（`draw_cpu_native` の後、raster と present の時間を測っている箇所）で、`self.gpu.note_cpu_frame(raster + present, period, composition.needs_gpu(), notes)` を呼ぶ。`period` は presenter が持つ display の周期。composition が届いていない箇所なら、composition を持つ呼び出し元で `needs_gpu()` を計算して引数で渡す。
8. 試験の host で GPU を要求するもの（`GpuMode::Required`、`RenderOptions::gpu`、`kivact-engine/src/host/testing/test_host.rs:203` 付近、`crates/arvane/tests/e2e/support.rs:390` 付近、`gpu_interop` の `.gpu()`）は `start: GpuStart::AtLaunch` を使う（GPU の E2E が CPU で待たないため）。
10. GPU の需要の伝達（設計書 4.3 節）：
    - `kivact-engine/src/contract/host.rs` の `HostRequest` に `DemandGpu { window: WindowId }` を足す。engine の host で `HostRequest` を処理している `match`（`grep -rnE "HostRequest::" crates/kivact-engine/src/host`）に腕を足し、arvane の `Windows::demand_gpu(window)` を呼ぶ。
    - `arvane/src/window.rs` の `Windows` に `pub fn demand_gpu(&mut self, id: WindowId)` を足し、その window の presenter の `demand_gpu`（`Gpu::start` を呼ぶ。`Off` と `Deferred` 以外では何もしない）を呼ぶ。
    - core で、GPU の import registry を作る時（`gpu_interop/import/registry.rs`）、shared image を作る時（`gpu_interop/share/app_side.rs`）、viewport に `gpu_frame` を設定する時（`view/elements/viewport.rs`）、drawable の export を GPU の texture で要求する時（`view/drawable*`）に、その window の `HostRequest::DemandGpu` を一度だけ出す（同じ object からの二度目は出さない。object の field に `demanded: bool` を置かず、作る時に一度出す形にする）。host request を出す既存の関数（`grep -rnE "HostRequest::" crates/kivact-core/src`）と同じ方法で出す。
11. `RendererChange` の note（frame log の `renderer=..`）は今のまま。`Deferred` から始めたときは `RendererChange::Start { gpu: false }` 相当で書く。frame log の書式は変えない。
12. `cargo fmt -p kivact-types -p arvane -p kivact-core -p kivact-engine`。

完了条件：

```bash
cargo check -p kivact-types --all-targets
cargo check -p arvane --all-targets
cargo check -p arvane --all-targets --features gpu
cargo check -p kivact-core --all-targets --all-features
cargo check -p kivact-engine --all-targets --features testing
cargo check -p kivact --all-targets
cargo clippy -p arvane --all-targets --all-features -- -D warnings
cargo xtask check --no-write --only state-fields --crate arvane
```

撤退規則：`Shell` が `GpuPreference` を `Driver` に渡していない場合は、`Shell::gpu_preference` で受け取った値を `Driver` の構築に渡す field を足す（所有内）。

### U25 `#[component]`（lane E）

- **目的**：設計書 3.2 節、5.2 節。
- **所有 file**：`crates/kivact-macros/**`（新規）、`crates/kivact-core/src/view/descriptor/factory.rs` と `IntoViewFn` の sealed の trait がある file、root `Cargo.toml`（`members` と `[workspace.dependencies]` の 3 行だけ）、`crates/kivact/Cargo.toml`、`crates/kivact/src/lib.rs`、`crates/kivact/src/prelude.rs`、`dev/xtask/policy/workspace.toml`（`visible_to` の 1 行）、`dev/tests/kivact-public-api/tests/e2e/component_macro.rs`（新規）、`dev/tests/kivact-public-api/tests/e2e/main.rs`（1 行）。

手順：

1. root `Cargo.toml`：`members` に `"crates/kivact-macros",` を `"crates/kivact",` の次に足す。`[workspace.dependencies]` に `proc-macro2 = "1"`、`quote = "1"`、`syn = { version = "2", features = ["full"] }` を足す。
2. `crates/kivact-macros/Cargo.toml`：

```toml
[package]
name = "kivact-macros"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "The #[component] attribute of Kivact: a props builder from a component function"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = { workspace = true }
quote = { workspace = true }
syn = { workspace = true }

[lints]
workspace = true
```

3. `crates/kivact-macros/src/` の構成（module ごとに file を分け、各 file の先頭に `//! Role:` を書く）：
   - `lib.rs`：`#[proc_macro_attribute] pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream`。`attr` が空でなければ `compile_error!("#[component] takes no arguments")`。`item` を `syn::ItemFn` として parse し、`parse::component(item)` → `expand::component(model)` を呼ぶ。error は `syn::Error::to_compile_error()`。
   - `parse.rs`：`ItemFn` から `Model { vis, name, struct_name, docs, props: Vec<Prop>, body }` を作る。`Prop { name: Ident, ty: Type, kind: PropKind, required: Required }`。`Required` は `Positional`、`Optional`（`#[prop(optional)]`）、`Default(Expr)`（`#[prop(default = expr)]`）。`PropKind` は設計書 5.2 節の表の 8 行（`Text`、`Value(T)`、`Binding(T)`、`Callback0`、`Callback(E)`、`View`、`Children`、`Other(T)`）。`Optional` のときは `Option<X>` の `X` に対して `PropKind` を決める。
     - 型の判定は path の最後の segment の名前で行う（`Value`、`Binding`、`Callback`、`View`、`Vec`、`Option`）。`Value<String>` は `Text`。`Callback<()>` は `Callback0`。`Vec<View>` は `Children`。
     - 診断（`syn::Error::new_spanned(.., "<文言>")`）：型引数・lifetime・where 句（「#[component] functions cannot be generic; take `Value<T>` props instead」）、戻り値が `impl IntoView` でない（「a #[component] function returns `impl IntoView`」）、`self`（「a #[component] is a free function」）、`#[prop(..)]` の未知の key（「#[prop] accepts `optional` or `default = <expr>`」）、`optional` で型が `Option<..>` でない（「an optional prop has type `Option<..>`」）、`optional` と `default` の併用（「use either `optional` or `default`」）。
   - `expand.rs`：設計書 5.2 節の展開の規則 1〜8 の token を出す。名前の path は `::kivact::state::{IntoText, IntoValue, IntoBinding, Callback, HandlerResult, callback}`、`::kivact::view::{IntoView, IntoChildren, View}`。setup の境界は、この unit の時点では `::kivact::view::component`（W3 の U31 が `scope` に置き換える）。
     - struct 名：関数名を `_` で区切り、各部分の先頭を大文字にして連結する。
     - struct の各 field の型は、設計書 5.2 節の表の「保持する型」（`Reactive<T>`、`Binding<T>`、`Box<dyn FnOnce() -> Callback<..>>`、`View`、`Vec<View>`、`ViewFn<()>`、その他の `T`、任意の props は `Option<..>`）。
     - 位置引数の関数（`#[track_caller]` 付き、例）：`pub fn card(title: impl ::kivact::state::IntoText, children: impl ::kivact::view::IntoView) -> Card { Card { title: ::kivact::view::Reactive::from_text(title), children: ::kivact::view::IntoView::into_view(children), on_close: None, gap: ::kivact::view::Reactive::new(12.0) } }`。`'static` を付けるのは callback の引数だけ。
     - `ViewFn` の prop は、生成する関数・setter に型引数 `V: ::kivact::view::IntoView` を足し、`impl ::kivact::view::IntoViewFn<V>` を受け取って `::kivact::view::IntoViewFn::into_view_fn(x)` で保持する。
     - 任意の props の setter（`#[track_caller]` 付き）：`pub fn on_close(mut self, f: impl Into<::kivact::state::Callback<()>> + 'static) -> Self { self.on_close = Some(Box::new(move || f.into())); self }`。`Callback(E)` の setter は `pub fn on_pick<R: ::kivact::state::HandlerResult>(mut self, f: impl Fn(E) -> R + 'static) -> Self { self.on_pick = Some(Box::new(move || ::kivact::state::callback(f))); self }`。`Value` の setter は `Some(Reactive::new(x))`（`default` の props は `Some` なし）。
     - `IntoView` の実装：`let Card { a, b, .. } = self;`（全 field を名前で分解する）の後、`::kivact::view::component(move || { .. })` の中で、`Reactive` の field は `::kivact::state::IntoValue::into_value(a)`、callback の field は `a()`（任意なら `a.map(|f| f())`）にしてから元の関数本体を置く。他の field はそのまま。
     - doc comment（`#[doc = ..]` の attribute）を struct と関数の両方に写す。struct に `#[must_use]`。
4. `crates/kivact-core/src/view/descriptor/factory.rs` に `impl IntoViewFn<View> for ViewFn<()> { fn into_view_fn(self) -> ViewFn<()> { self } }` と、sealed の trait（`super::sealed_view_fn::Sealed<V>`）の `impl Sealed<View> for ViewFn<()> {}` を足す（設計書 5.2 節の規則 8）。`crates/kivact/Cargo.toml` の `[dependencies]` に `kivact-macros = { path = "../kivact-macros", version = "0.1.0" }` を足す。`description` に macro を持たないという文言があれば「and the `#[component]` attribute」を足す。
5. `crates/kivact/src/lib.rs`：`pub use kivact_macros::component;` を、facade の root の再公開の並びに足す（rustdoc：`/// Turn a component function into its props builder (see the guide "Writing components").`）。`crates/kivact/src/prelude.rs` に `pub use crate::component;` を足す（attribute macro。今の関数 `component` と同じ名前だが名前空間が違うので両立する。関数は W3 で `scope` に改名する）。
6. `dev/xtask/policy/workspace.toml` の `[visible_to]` に `kivact-macros = ["kivact"]` を足す。
7. E2E `dev/tests/kivact-public-api/tests/e2e/component_macro.rs`（`main.rs` に `mod component_macro;` を足す）。既存の e2e の file の書き方（`use kivact::prelude::*;`、`kivact::test::render`）に合わせ、次の 3 つの試験を書く：
   - `required_props_are_positional`：`#[component] fn card(title: Value<String>, children: View) -> impl IntoView { column((text(title), children)) }` を `card("Title", text("Body"))` で描き、`get_by_text("Title")` と `get_by_text("Body")` が見つかる。
   - `optional_and_default_props_use_setters`：`#[prop(optional)] on_close: Option<Callback<()>>` と `#[prop(default = 3)] count: Value<i32>`（本体で `text(move || format!("n={}", count.get()))`）を持つ component。既定で `n=3` が出る。`.count(5)` で `n=5`。`.on_close(move || closed.set(true))` を渡したとき、本体の「Close」button（`on_close` が `Some` の時だけ置く）の click で `closed` が true になる（`closed` は試験の root で作った `signal(false)`、表示は `text(move || format!("closed={}", closed.get()))` で確かめる）。
   - `borrowed_text_and_recreated_content`：`#[component] fn panel(title: Value<String>, open: Binding<bool>, content: ViewFn) -> impl IntoView { column((text(title), show(open, content))) }`。試験の root で `let name = String::from("Panel");` を作り `panel(name.as_str(), open, || text("Body"))` で描く（`&str` を渡せる）。`open` を false にして `Body` が消え、true に戻すと `Body` が再び現れる（作り直される）。
   - `component_owner_is_disposed_on_unmount`：`show(visible, move || card("T", text("B")))` で描き、`visible` を false にした後の `screen.stats().live_owners` が、描く前の値に戻る。
   - `payload_callback_infers_the_argument`：`#[prop(optional)] on_pick: Option<Callback<i32>>`。setter に `move |n| picked.set(n)`（型注釈なし）を渡し、本体の button の `on_press` で `if let Some(cb) = on_pick { let _ = cb.call(7); }` を呼ぶと `picked=7` が表示される。
8. `cargo fmt -p kivact-macros -p kivact -p kivact-public-api-tests`。

完了条件：

```bash
cargo check -p kivact-macros --all-targets
cargo clippy -p kivact-macros --all-targets -- -D warnings
cargo check -p kivact --all-targets
cargo check -p kivact-public-api-tests --tests
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --test e2e -- component_macro
```

（最後のコマンドは本書が名前を挙げた試験なので実行してよい。）

## 5. CP1（Orchestrator）

W2 の merge の後に行う。ここで初めて重い試験を実行する。

```bash
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 1800 -- dev/bench/run.sh CP1
```

- 失敗した試験は、その試験の file と失敗の message を、関係する unit（性能の変更なら U13、U14、U21、U22、U24、memory なら U23、U15）の subagent に渡して直させる。撤退規則がある unit は、2 回直して通らなければ撤退規則を適用する。
- `dev/bench/results/CP1.md` と B0 を比べ、K1・K4・K5 が B0 より悪化していないことを確かめる（改善の大きさは問わない）。悪化していたら、悪化した KPI に関係する unit を STOP 扱いにして人間に報告し、先へ進む。
- すべて通ったら `git push origin refactor/kashiwade/slim-fast`。

---

## 6. W3（U31 を先に始め、他は同時）

### U31 改名（lane A）

- **目的**：設計書 5.3 節、付録 B。U31 は機械的な一括置換と、core・test・facade の compile を通すところまで。
- **所有 file**：付録 B の B.2 の `targets` が列挙するすべての file。`arvane*`、`kivact-runtime`、`kivact-layout`、`kivact-editing`、`kivact-types`、`kivact-gpu-share`、`kivact-shader`、`kivact-devtools-agent`、`kivact-style`、`kivact-engine` は含まない（W3 の他の unit が所有する）。

手順：

1. 置換の前に、除外する crate に対象の名前が無いことを確かめる：

```bash
grep -rnE "\bNodeBuilder\b|\bimpl Component\b|view::component\b|[^.:A-Za-z0-9_]component\((move )?\|\|" crates/arvane* crates/kivact-runtime crates/kivact-layout crates/kivact-editing crates/kivact-types crates/kivact-gpu-share crates/kivact-shader crates/kivact-devtools-agent crates/kivact-style crates/kivact-engine
```

   出力があれば、その行を置換せず、file:行を報告する（その crate を W4 で所有する unit が直す）。
2. 付録 B の B.2 の script をそのまま実行する。
3. `crates/kivact-core/src/view/element_data.rs`（分析機で script を試した結果に基づく）：
   - 513 行付近の `pub trait Component: crate::view::descriptor::IntoView + Element {}` と、次の行の `impl<T: crate::view::descriptor::IntoView + Element> Component for T {}` を削除する。
   - その直前の `#[diagnostic::on_unimplemented(message = "a component needs IntoView and Element", ..)]` を、197 行付近の `pub trait Element: Sized {`（旧 `NodeBuilder`）の直前に移し、message を `"`{Self}` is not an element: it needs IntoView and Element"` にする。
   - `pub trait Element: Sized {` を `pub trait Element: crate::view::descriptor::IntoView + Sized {` にする。
   - `Element` の impl の型引数の bound が、同じ型の `IntoView` の impl の bound より緩いと error になる。その場合は `Element` の impl の bound を `IntoView` の impl と同じにする。分析機で確かめた例：`crates/kivact-core/src/view/flow/virtual_list.rs:166` の `impl<T: 'static, K: 'static> Element for VirtualList<T, K>` を `impl<T: kivact_runtime::PropValue, K: kivact_runtime::ItemKey> Element for VirtualList<T, K>` にする。`IntoView` の impl が無い型が error になったら STOP。
   - 分析機では、手順 2〜4 の後に `kivact-core` と `kivact-test` が全 feature・全 target で compile できた。
4. `crates/kivact-core/src/view.rs`：
   - 置換の結果、`Element` が二度 import される（65〜124 行付近の `pub use element_data::Element;` と、`pub use ..::{.., Node, Element, Scroll, ..}` の並び）。並びの中の `Element` を消す。
   - `pub use boundary::{ReadingDirection, component, provide_direction, use_direction};` の `component` を `scope` にする。
   - `crates/kivact-core/src/view/boundary.rs` の `pub fn scope<V: IntoView>(..)`（置換済み）の rustdoc の 1 行目を `/// Run `build` once at mount in a new owner (a scope): its hooks and cleanups belong to it.` にする。
5. `crates/kivact-macros/src/expand.rs` の `::kivact::view::component` を `::kivact::view::scope` にする。
6. `crates/kivact/src/prelude.rs`：
   - `pub use crate::view::{.., canvas, column, component, drawable, ..}` の並びの `component` を `scope` にする。
   - 置換で `pub use crate::view::Component;` が `pub use crate::view::Element;` になり、並びの `Element`（旧 `NodeBuilder`）と重なる。`pub use crate::view::Element;` の行を消す。
   - `pub use crate::component;`（U25 の attribute macro）が残っていることを確かめる。
   - `ButtonVariant` を `pub use crate::ui::{..}` の並びに足す。
7. `PartState` の focus（付録 A の A-PS）：`crates/kivact-core/src/view/part_data.rs` の `PartState` の `focused: bool` と `focus_visible: bool` を `focus: Focus` に置き換え、`pub enum Focus { #[default] None, Focused, FocusedVisible }` を同じ file に足す（`Copy, Clone, Debug, Default, PartialEq, Eq`）。`PartState` に `pub fn is_focused(&self) -> bool { self.focus != Focus::None }` と `pub fn is_focus_visible(&self) -> bool { self.focus == Focus::FocusedVisible }` を足す。core と test の中の利用箇所を直す（ui と kit の利用箇所は W4・W5 で直す）。`Focus` を `kivact::view` から再公開する。
8. 置換した package だけを整形する（`cargo fmt --all` は W3 の他の unit の file も変えるので使わない）：

```bash
cargo fmt -p kivact -p kivact-core -p kivact-test -p kivact-interaction -p kivact-ui -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative -p kivact-ui-soft -p kivact-devtools -p kivact-macros -p kivact-public-api-tests -p kivact-docs -p kivact-bench -p xtask -p kivact-template-basic -p kivact-example-counter -p kivact-example-form -p kivact-example-list -p kivact-example-async -p kivact-example-component -p kivact-ui-basic-catalog -p kivact-ui-pop-catalog -p kivact-ui-game-catalog -p kivact-ui-creative-catalog -p kivact-ui-soft-catalog
```

完了条件：

```bash
cargo check -p kivact-core --all-targets --all-features
cargo check -p kivact-test --all-targets --all-features
cargo check -p kivact-macros --all-targets
cargo clippy -p kivact-core --all-targets --all-features -- -D warnings
```

（`kivact-interaction`、`kivact-ui`、kit などは W4・W5 で直すので、この時点で compile できなくてよい。）

### U32 arvane の状態機械（lane B）

- **目的**：設計書 6 章（付録 A の A-AR）。
- **所有 file**：`crates/arvane/src/**`、`crates/arvane-shell/src/**`、`crates/arvane-native/src/**`、`crates/arvane-virtual/src/**`、`crates/arvane-asset/src/**`、`crates/arvane-compose/src/**`、`crates/arvane-render/src/**`。

手順：

1. 付録 A の「A-AR」の表の各行を直す。Linux 専用の行（`x11`）は 0.6 節の方法で確かめる。
2. 各 crate で `cargo xtask check --no-write --only state-fields-strict --crate <crate>` を実行し、残った error を SM-1〜SM-12 で直す。規則に当てはまらないものは STOP。
3. `cargo fmt` を各 crate に。

完了条件（各 crate について）：

```bash
cargo check -p <crate> --all-targets --all-features
cargo clippy -p <crate> --all-targets --all-features -- -D warnings
cargo xtask check --no-write --only state-fields-strict --crate <crate>    # 0 件
```

`<crate>` は `arvane`、`arvane-shell`、`arvane-native`、`arvane-virtual`、`arvane-asset`、`arvane-compose`、`arvane-render`。

### U33 layout・editing・devtools-agent・i18n・style・shader・types・gpu-share の状態機械（lane C）

- **目的**：付録 A の「A-LE」。
- **所有 file**：`crates/kivact-layout/src/**`、`crates/kivact-editing/src/**`、`crates/kivact-devtools-agent/src/**`、`crates/kivact-i18n/src/**`、`crates/kivact-shader/src/**`、`crates/kivact-gpu-share/src/**`、`crates/kivact-style/src/**`（`resolve.rs` を除く。U34 が所有する）、`crates/kivact-types/src/**`（`gpu.rs` と `render/**` と `paint.rs` を除く）。

手順：U32 と同じ（表は A-LE、crate は上の所有 file の crate）。`kivact-types` の型の形を変えたときの、他の crate の利用箇所の機械的な追従は行ってよい（W3 の他の unit の所有 file を除く）。

完了条件：U32 と同じ形で、上の各 crate。

### U34 engine と types のノードごとのメモリ（lane D）

- **目的**：設計書 4.7 節 MEM-2、MEM-3、MEM-5、MEM-7（`StateStyles`、`HostNode`、`NodeProperties`）、4.9 節 EVT-1。
- **所有 file**：`crates/kivact-types/src/paint.rs`、`crates/kivact-engine/src/**`（W3 では他の unit が engine を所有しない）、`crates/kivact-style/src/resolve.rs`（`StateStyles` の構築の箇所だけ）。

手順：

1. `kivact-types/src/paint.rs` の `StateStyles` の 8 つの `Option<PaintDelta>` を `Option<Box<PaintDelta>>` にする。`deltas()` の中の `[&self.hover, ..].into_iter().flatten()` を `[self.hover.as_deref(), ..].into_iter().flatten()` にする（戻り値の型 `impl Iterator<Item = &PaintDelta>` は変えない）。`StateStyles` の定義の直後に `const _: () = assert!(std::mem::size_of::<StateStyles>() <= 80, "StateStyles keeps its deltas boxed");` を足す。
2. `StateStyles` を作る・読む箇所（`grep -rnE "hover:|pressed:|focus_visible:|group_hover:|group_focus:|\.hover\b|\.pressed\b|\.enter\b|\.exit\b" crates/kivact-style/src/resolve.rs crates/kivact-engine/src`）を `Box` に合わせる（作る側は `Some(Box::new(delta))`、読む側は `.as_deref()`）。`kivact-engine` の中の追従は所有外でも行ってよい（W3 の他の unit は engine を所有しない）。
3. MEM-2・MEM-3（engine 側で既定値を保持しない）：
   - `host/write/typed.rs:166-171` の `set_layout`・`set_paint` の `boxed(Some(v), ..)` を `boxed((v != LayoutProps::default()).then_some(v), ..)`（Paint は `PaintProps::default()`）にする。
   - `model/states.rs:147` の `Some(states)` を `(states != StateStyles::default()).then_some(states)` にする。
   - 読み手が `None` を既定値として扱うことを確かめる：`grep -rnE "\.layout\(\)|\.paint\(\)|\.states\(\)|layout_props\(" crates/kivact-engine/src crates/kivact-layout/src`。`None` を既定値（`unwrap_or_default`、`map_or(default, ..)`、`PaintProps::default()`）か「無い」として扱っていれば変えない。`None` を「まだ書かれていない」として別の処理をする読み手があれば STOP。
4. MEM-5：`measure/inherited_text.rs` の `computed_text`：`let resolved = Rc::new(text_props(tree, id));` を、`let value = text_props(tree, id);` と、親（`tree.parent_id(n)`）の slot の cache（`columns.computed_text(parent_slot)`、**既にある場合だけ**。無ければ計算しない）が `Some(parent_rc)` で `*parent_rc == value` なら `Rc::clone(&parent_rc)`、そうでなければ `Rc::new(value)`、にする。親の `computed_text` を再帰で呼ばない（深い鎖で D² になるため）。
5. EVT-1：`model/events.rs` の `EventSink` の queue を取り出す method（drain・take の類）の後で、`if queue.capacity() > 4 * queue.len().max(1024) { queue.shrink_to(queue.len().max(1024)); }` を行う（M-R4。同じ規模の burst が続いても縮小と拡張を繰り返さない）。`model/drawables/place.rs` の `frames` は一時の `Vec` なので触らない。
6. `tree/node.rs` の `HostNode` の定義の直後に `const _: () = assert!(std::mem::size_of::<HostNode>() <= 464, "HostNode grew: box the new rare field");`、`contract/node.rs` の `NodeProperties` の直後に同じ形で 240 を足す。
7. `cargo fmt -p kivact-types -p kivact-engine -p kivact-style`。

完了条件：

```bash
cargo check -p kivact-types --all-targets
cargo check -p kivact-style --all-targets
cargo check -p kivact-engine --all-targets --features testing
cargo check -p kivact-engine --release
cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings
```

### U35 text の cache の検索で確保しない（lane E）

- **目的**：設計書 4.9 節 TXT-1。
- **所有 file**：`crates/arvane-text/src/**`、`crates/arvane-text/Cargo.toml`、`Cargo.lock`。

手順：

1. `crates/arvane-text/Cargo.toml` に `hashbrown = "0.17"` を足す（parley と fontique が使う版。`Cargo.lock` の `hashbrown 0.17.1`）。
2. `cache.rs` の `TextKey` が `#[derive(Hash, PartialEq, Eq)]` であることを確かめる。手書きの `Hash` なら STOP。
3. `cache.rs` に借用の key を足す：`TextKey` の field を同じ順に並べ、所有の型を借用にした `struct TextKeyRef<'a>`（`String` → `&'a str`、`Vec<T>` → `&'a [T]`、`Box<T>` → `&'a T`、`Option<Box<T>>` → `Option<&'a T>`、`Copy` の field はそのまま）を `#[derive(Hash)]` で作る。`impl hashbrown::Equivalent<TextKey> for TextKeyRef<'_>` で field ごとに比べる。
4. `TextKey` を key にする map（shaped、measured）を `hashbrown::HashMap` にし、検索（`get`）を `TextKeyRef` で行う。miss の時だけ `TextKey` を作って `insert` する。`TextKey::new` 等の今の構築の関数は、`TextKeyRef` を作る関数と、それを所有の `TextKey` にする `to_owned` に分ける。
5. 2 世代の cache の入替えの規則（`cache.rs:300-331`）は変えない。
6. `cargo fmt -p arvane-text`。

完了条件：

```bash
cargo check -p arvane-text --all-targets
cargo check -p arvane --all-targets
cargo clippy -p arvane-text --all-targets -- -D warnings
cargo xtask check --no-write --only state-fields --crate arvane-text
```

撤退規則：`TextKeyRef` の hash が `TextKey` と一致しない（試験で cache が miss し続ける、または型の都合で同じ順の field にできない）場合は、この unit の変更をすべて取り消し、報告する（性能の小さな改善なので撤退してよい）。

---

## 7. W4（5 unit 同時）

各 unit の共通の手順：

- A. W3 の改名と署名の変更（付録 B）に、自分の crate を追従させて compile を通す。
- B. 付録 A の自分の表の各行を直す。
- C. `cargo xtask check --no-write --only state-fields-strict --crate <crate>` の残りを SM-1〜SM-12 で 0 件にする。field 名を繰り返すだけの既存の marker の理由（`passive record - <名前>`、`builder option - <名前>`、`cache key - <名前>`、`mount config - <名前>`）は、6.3 節 SM-11 の形の理由に書き直すか、SM-1〜SM-10 で直す。書き直しの理由は、次の 4 つの雛形のどれかを使う：
  - `option: absent means the component's default`
  - `cache: one component of a cache key; equality of the whole key is what matters`
  - `projection: readonly snapshot of <持ち主の型名>'s state for recipes, written only by <書き手の関数>`
  - `independent: a measured value; any value is valid on its own`
  どの雛形にも当たらない field は SM-1〜SM-10 を当てはめる。当てはまらなければ STOP。
- 移行期の gate：付録 A の A-EC の行（`TurnOutput`、`Routed`、`ImeStateInfo`、`RawKey`）は W6 の U63 が直すので、この wave の strict の 0 件の判定から除く（その field の error だけが残っていればよい。報告に一覧を書く）。
- D. 完了条件：`cargo check -p <crate> --all-targets --all-features`、`cargo clippy -p <crate> --all-targets --all-features -- -D warnings`、`cargo xtask check --no-write --only state-fields-strict,effect-writes,api-shape --crate <crate>`（0 件）。

### U41 interaction（lane A）

- **所有 file**：`crates/kivact-interaction/src/**`。
- 共通の手順 A〜D。表は付録 A の A-IN。
- 追加：`on_*` の setter を ARG-1 に揃える（payload なし `impl Into<Callback<()>>`、payload あり `impl Fn(E) -> R + 'static` で `R: HandlerResult`）。

### U42 ui（lane B）

- **所有 file**：`crates/kivact-ui/src/**`。
- 共通の手順 A〜D。表は付録 A の A-UI。
- 追加：
  1. 設計書 5.4 節の「直す対象」の表の `kivact-ui` の行をすべて直す。
  2. `kivact-ui` の他の公開の部品の setter を ARG-1〜ARG-6 に照らし、合わないものを直す。直したものの一覧を報告に書く（kit の unit が追従に使う）。
  3. `PartState` の focus の変更（U31 手順 7）に、recipe の state の利用箇所を追従させる（`state.focused` → `state.is_focused()`、`state.focus_visible` → `state.is_focus_visible()`）。
  4. MEM-6：`button.rs:241` の `let state = memo(move || ButtonState {..})` の中で読む part の状態が、それぞれ別の `memo` から来ている場合、part の状態を一度だけ読む（`let part = part_state.get();`）形にし、追加の `memo` を作らない。`loading`（`button.rs:389`）は `busy_machine.feed` に渡すので memo のまま残す。
  5. MEM-6（busy）：今は busy を使わない button も、`finish`（`button.rs:265`）で `busy_animation`（signal、memo 2 つ、`frame_loop`、Machine と feed 2 つ）と `busy_indicator` を必ず作る（分析機の計測で、3000 個の button に 3000 個の `frame_loop` があった）。`Button` の `busy: Value<bool>`（既定 `false.into_value()`）を `busy: Option<Value<bool>>`（既定 `None`、field の直前に `// xtask-allow(state-fields): option: absent means the component's default`）にし、`.busy(..)` が `Some` を入れる。`finish` は、`busy` が `None` なら `busy_animation` と `busy_indicator` を呼ばず、`phase` は `0.0.into_value()`、`reveal_pending` は `false.into_value()`、`ButtonState.busy` は `false` にする。`Some` なら今どおり。
  6. 確認：`dev/bench` の `kv_latency` を 1 回だけ実行してよい（`KIVACT_TEST_MIRROR=off cargo run --release -p kivact-bench --bin kv_latency`）。`S stats_n3000` の行の `frame_loops` が 10 以下、`live_reactive` が `analysis-machine.md` の値（168150）の 0.6 倍以下であることを報告する（満たさなくても STOP にはしない。値を報告する）。

### U43 engine（lane C）

- **所有 file**：`crates/kivact-engine/src/**`。
- 共通の手順 A〜D（crate は `kivact-engine`、`--features testing` で check と clippy）。表は付録 A の A-EN。
- 追加：engine の doc comment で、試験の locator を指す `` `Element` `` と `` `Element::.. `` を `` `Locator` `` に直す（`grep -rnE '`Element(::|`)' crates/kivact-engine`）。

### U44 core（lane D）

- **所有 file**：`crates/kivact-core/src/**`、`crates/kivact-test/src/**`。
- 共通の手順 A〜D（crate は `kivact-core`、`kivact-test`）。表は付録 A の A-CO。
- 追加 E2E（`dev/tests/kivact-public-api/tests/e2e/` は U56 が所有するので、ここでは書かない）。

### U45 runtime・style・i18n・types など残りの crate の lint（lane E）

- **所有 file**：`crates/kivact-runtime/src/**`、`crates/kivact-style/src/**`、`crates/kivact-i18n/src/**`、`crates/kivact-types/src/**`、`crates/kivact-editing/src/**`、`crates/kivact-layout/src/**`、`crates/kivact-shader/src/**`、`crates/kivact-gpu-share/src/**`、`crates/kivact-devtools-protocol/src/**`、`crates/kivact-devtools-client/src/**`、`crates/kivact-devtools-mcp/src/**`、`crates/kivact-devtools-agent/src/**`、`crates/arvane-text/src/**`、`crates/kivact-macros/src/**`、`dev/xtask/src/**`。facade の `crates/kivact` は同じ wave の U42 の変更（`PartState.focus` への追従）が無いと compile できないので、この unit の完了条件に `cargo check -p kivact` を含めない（W4 の merge の後に Orchestrator が確かめる）。
- 共通の手順 C と D だけ（A は不要、B は W1・W3 で済み）。
- 追加：U31 の置換の対象外だった crate の doc comment で、試験の locator を指す `` `Element` `` と `` `Element::.. `` を `` `Locator` `` に直す（`grep -rnE '`Element(::|`)' crates/kivact-style crates/kivact-types crates/kivact-runtime` で探す。例：`kivact-style/src/draw.rs:40`、`kivact-style/src/text.rs:48`）。kivact-engine の同じ doc comment は U43 が直す。

## 8. W5（kit と例、6 unit 同時）

各 unit の共通の手順：

- A. W3・W4 の改名と署名の変更（付録 B、U42 の報告の一覧）に追従させて compile を通す。
- B. kit の公開 API を ARG-1〜ARG-6 に揃える。設計書 5.4 節の表にある自分の kit の行は必ず直す。それ以外に合わないものを見つけたら直し、報告に一覧を書く。kit の公開の名前を変えたら、その kit の catalog と README を同じ unit で直す。
- C. 付録 A の自分の表の各行を直す。
- D. `cargo xtask check --no-write --only state-fields-strict,kit-literal-color,name-collision,api-shape --crate <crate>` を 0 件にする（W4 の共通手順 C と同じ雛形）。
- E. 完了条件：自分の crate すべてで `cargo check --all-targets`、`cargo clippy --all-targets -- -D warnings`、手順 D のコマンド。kit の試験（`tests/consumer.rs`）は実行しない（CP2 で行う）。

| unit | lane | 所有 file | 付録 A の表 |
|---|---|---|---|
| U51 | A | `crates/kivact-ui-basic/**`、`examples/kivact-ui-basic/**`、`crates/kivact-devtools/**`（basic kit の利用者。basic の API の変更と同じ unit で追従させる） | A-KB、A-DT |
| U52 | B | `crates/kivact-ui-pop/**`、`examples/kivact-ui-pop/**` | A-KP |
| U53 | C | `crates/kivact-ui-game/**`、`examples/kivact-ui-game/**` | A-KG |
| U54 | D | `crates/kivact-ui-creative/**`、`examples/kivact-ui-creative/**` | A-KC |
| U55 | E | `crates/kivact-ui-soft/**`、`examples/kivact-ui-soft/**` | A-KS |
| U56 | F | `examples/kivact/**`、`templates/**`、`dev/tests/**`、`dev/bench/**` | なし（strict lint の残りは共通手順 D） |

U56 の追加の手順：

1. `examples/kivact/component/src/main.rs` を `#[component]` で書き直す。`card` の children は `content: ViewFn` で宣言し、本体の `show(open, content)` で開閉のたびに作り直す今の契約を保つ。同じ file の試験（Collapse の後の Expand を含む `#[cfg(test)] mod tests`）を変えずに通す。
2. `templates/basic` の部品を `#[component]` で書く（部品が無ければ、`App` の root を一つの `#[component]` にする）。
3. `dev/tests/kivact-public-api/tests/e2e/` に `binding_update.rs` を足し、`main.rs` に登録する：`Binding` を受け取る部品（`switch` など、`Binding<bool>` を受けるもの）を描き、試験の中の button の `on_press` で `binding.update(|v| *v = !*v)` を呼んで、表示と a11y の状態が反転することを確かめる。
4. `dev/tests/kivact-public-api/tests/e2e/` に次を足し、`main.rs` に登録する（設計書 7 章）：
   - `binding_update.rs` に、`binding(get, set)` で作った binding の `update` が `set` を一度呼ぶ試験も足す（`set` の中で数を数える `var`）。
   - `resource_reclaim.rs`：`signal(true)` の `show` の中に、`for_each` で 3000 行（各行 `row((text(..), button(..)))`）の list を描く。描く前の `screen.stats()` を記録し、`show` を false にして `run_until_idle()` した後、`live_reactive`・`live_owners`・`frame_loops`・`tree_nodes` が描く前の値と同じになる。これを 3 回繰り返す。
   - `svg_embedded_text.rs`：内側の SVG に大きな `<text>` があり、外側には `<image href="data:image/svg+xml;base64,...">` だけがある SVG を `image(..)` で描き、内側の text を空にした同じ SVG と比べて `Screenshot::raster_difference` が 0 でない（文字が描かれている）。
   - `gpu_on_demand.rs`（GPU の lane。`kivact::test::gpu_unavailable()` なら skip）：既定の `GpuStart::OnDemand` の試験の host（`RenderOptions` で GPU を要求しない設定）で、(a) shader を使う style の node を描くと GPU の renderer に切り替わる、(b) feature `gpu-interop` で shared image を作ると GPU が始まり、GPU frame が届く、を確かめる。renderer の確認には既存の GPU 試験が使う方法（`kivact-test/src/screen/gpu_tests.rs`）を使う。
5. `dev/tests` の既存の試験を改名（付録 B）と U42〜U55 の API の変更に追従させる（期待値は変えない）。
6. `kivact-public-api-tests`、`kivact-docs`、`kivact-bench`、`gpu-interop-producer`、例と雛形の crate で、共通の手順 D の strict lint を 0 件にする。

U56 の完了条件：

```bash
cargo check --workspace --all-targets --all-features --keep-going
cargo clippy --workspace --all-targets --all-features --keep-going -- -D warnings
```

（W5 の他の unit の変更が merge される前なので、kit の crate と `kivact-devtools` の error は残ってよい。その場合は、error の path がそれらの crate の中だけであることを報告する。）

---

## 9. CP2（Orchestrator）

W5 の merge の後に、CP1 と同じコマンドを実行する（`run.sh` の label は `CP2`）。ただし `kivact-docs` の試験は実行しない（文書の中の Rust は W6 の U62 で新しい API に直すため、CP3 で実行する）。加えて：
feature の組合せ（設計書 7 章。package を一つずつ指定する。workspace の `--no-default-features` は GPU なしの確認にならないので使わない）：

```bash
cargo check -p kivact --no-default-features
cargo check -p kivact --no-default-features --features gpu
cargo check -p kivact --features gpu-interop
cargo tree -p kivact --no-default-features -e features -i arvane-compose | grep -E '"gpu"' && echo "NG: gpu leaked" || echo ok
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-public-api-tests --no-default-features
```


```bash
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
cargo xtask check --no-write --only state-fields-strict      # A-EC の行の field を除いて 0 件
```

- 失敗は CP1 と同じ扱い。kit の試験の失敗は、その kit の unit（U51〜U55）の subagent に直させる。
- `dev/bench/results/CP2.md` を設計書 1.4 節の「必達」と比べ、満たしていない KPI を `OPEN.md` に書く。必達に届かない KPI は、原因の調査と設計の見直しを設計者に差し戻す（実装者が目標を下げたり、測定の条件を変えたりしない）。
- 通ったら push する。

---

## 10. W6（3 unit 同時）

### U61 lint の統合と API の検査（lane A）

- **所有 file**：`dev/xtask/**`。

手順：

1. `state-fields-strict` の検査を `state-fields` に統合する：`Strictness` を消し、`Strict` の検査を常に行う。`RULES` から `state-fields-strict` を消す。`dev/xtask/strict-report.txt` を消す。
2. `api-shape` に ARG-1 の検査を足す：公開の method で名前が `on_` で始まるものの最後の引数の型が、`impl Into<Callback<()>>` か、`impl Fn(..) -> R`（`R` が `HandlerResult` を bound に持つ型引数）でなければ error。例外は `on_mount`・`on_cleanup`（`FnOnce` の lifecycle、architecture.md の規則どおり）と `on_attach`。
3. `--report-audit`（設計書 6.4 節）：`state-fields` の検査が数える field を 2 つ以上持つ struct・named-field の variant・関数（local）ごとに、file:line、field、marker の理由、閉じ方（理由の接頭辞。marker が無く enum・Machine にしたものは `type`）を 1 行ずつ `dev/xtask/state-audit.md` に書く。閉じ方が無い行（marker も型も無いのに数えられた field）は `OPEN` と書き、そのときは非 0 で終わる。
4. `cargo fmt -p xtask`。

完了条件：

```bash
cargo check -p xtask --all-targets
cargo clippy -p xtask --all-targets -- -D warnings
cargo test -p xtask
cargo xtask check --no-write      # workspace 全体で 0 件
```

`cargo xtask check --no-write` に残る error は、error の file を所有した W4・W5 の unit の規則で直す（この unit が所有外の file を直してよい。ただし同じ W6 の U63 の所有 file（`kivact-engine`、`kivact-core`、`kivact-test`、`arvane-shell`、`arvane-native`、`arvane-virtual`、`kivact-devtools`）の error は直さず一覧で報告する。CP3 で U63 の merge の後に再確認する）。

### U62 文書（lane B）

- **所有 file**：`docs/**`、`README.md`、`README_JP.md`、`CONTRIBUTING.md`、`crates/*/README.md`、`examples/README.md`。

手順：

1. `docs/for-kivact-developer/architecture.md` と `decisions.md` には、この unit では触れない（受入れを満たしたときだけ W7 で書き戻す）。ただし、改名した名前（`NodeBuilder`、`Component`、`component(`、`kivact::test::Element`）が rustdoc の link として壊れる箇所だけは新しい名前に直す。
2. `templates/basic/README.md` の profile の説明に、U11 で雛形に書いた `[profile.release]` の理由（サイズと速度、release ビルドが延びること）を足す。
3. `docs/for-kivact-user/en` と `jp`：
   - 「部品を作る」系のガイドを `#[component]` で書き直す（手書きの builder の例は消す）。
   - `NodeBuilder`、`Component`、`component(`、`kivact::test::Element` を新しい名前にする。
   - 設計書 5.4 節の「文書の誤りの修正」をすべて行う。
   - `complex-scripts` を外してサイズを減らす方法を、配布のガイドに足す。
   - `GpuStart` を GPU のガイドに足す（既定は OnDemand、AtLaunch の選び方）。
4. `CONTRIBUTING.md`：`dev/bench` の使い方（手動、CI ではない）を「検証」の節に 3 行で足す。
5. kit の README：U51〜U55 が変えた公開の名前を反映する（各 unit の報告の一覧を使う）。

完了条件：

```bash
KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --all-features -- --test-threads=1
grep -rnE "NodeBuilder|impl Component|kivact::test::Element|view::component\b" docs README.md README_JP.md CONTRIBUTING.md crates/*/README.md | grep -v "docs/for-kivact-developer/decisions.md"   # 出力なし（decisions.md は旧名を記録として残す）
```

（`kivact-docs` の試験は文書の中の Rust を compile する。本書が名前を挙げた試験なので実行してよい。）

---

### U63 crate をまたぐ状態機械と DevTools アプリ（lane C）

- **目的**：付録 A の A-EC。型の変更が W2〜W5 の同じ wave の複数の unit の file にまたがるため、ここでまとめて行う（DevTools アプリ（A-DT）は W5 の U51 が行う）。
- **所有 file**：`crates/kivact-engine/**`、`crates/kivact-core/**`、`crates/kivact-test/**`、`crates/arvane-shell/**`、`crates/arvane-native/**`、`crates/arvane-virtual/**`、`crates/kivact-devtools/**`（追従のみ）、`dev/tests/**`（追従のみ）。

手順：

1. 付録 A の A-EC の各行を直す。型の利用箇所（`grep -rnE "<型名>" crates dev`）をすべて直す。Linux 専用の利用箇所は 0.6 節の方法で確かめる。
2. `cargo fmt -p kivact-engine -p kivact-core -p kivact-test -p arvane-shell -p arvane-native -p arvane-virtual -p kivact-devtools`。

完了条件：

```bash
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask check --no-write --only state-fields-strict --crate kivact-engine
cargo xtask check --no-write --only state-fields-strict --crate kivact-core
```

## 11. CP3（Orchestrator）

```bash
cargo fmt --all -- --check
cargo xtask check                 # --no-write を付けない：architecture/ の図を生成する
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1
cargo xtask check --no-write --only state-fields --report-audit   # dev/xtask/state-audit.md を書く
dev/tools/window-lock.sh --timeout 1800 -- dev/bench/run.sh B1
```

feature の組合せ（設計書 7 章。package を一つずつ指定する。workspace の `--no-default-features` は GPU なしの確認にならないので使わない）：

```bash
cargo check -p kivact --no-default-features
cargo check -p kivact --no-default-features --features gpu
cargo check -p kivact --features gpu-interop
cargo tree -p kivact --no-default-features -e features -i arvane-compose | grep -E '"gpu"' && echo "NG: gpu leaked" || echo ok
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-public-api-tests --no-default-features
```


- WSL があれば、WSL の中（`wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}"`）で、上の clippy と `cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features` を同じ形で実行する（Linux の desktop の環境が要る試験は、WSLg が無ければ skip されてよい）。無ければ報告に「Linux 未確認」と書く。
- `architecture/` の変更を commit する。

---

## 12. W7（Orchestrator）

### W7-1 受入れの判定

1. `dev/bench/results/B1.md` と `B0.md`（どちらも INVALID でないこと）から、設計書 1.4 節の各行の倍率（B0 ÷ B1。K3 のアイドルは B1 の値そのもの）を計算し、`dev/bench/results/RESULT.md` に表で書く：KPI、指標、B0、B1、倍率、必達、判定。必達に届かない行は、同じ条件で 2 回測り直し（`run.sh B1-r2`、`B1-r3`）、3 回中 2 回以上満たせば満たしたとする（設計書 1.6 節 AC-2）。
2. 設計書 1.6 節の AC-1〜AC-4 を確かめ、`RESULT.md` に AC ごとの結果を書く。AC-3 は `dev/xtask/state-audit.md` に `OPEN` が無いことと、`OPEN.md` に状態の項目が無いこと。
3. **AC-1〜AC-4 のどれかを満たさない場合**は、ここで止める：`RESULT.md` と `OPEN.md` を commit して push し、満たさない項目を設計者（人間）に報告する。W7-2 と W7-3 は行わない（設計書・作業指示書を消さず、architecture.md と decisions.md にも書き戻さない）。

### W7-2 書き戻しと後片付け（AC をすべて満たしたときだけ）

1. `docs/for-kivact-developer/architecture.md`：crate の表に `kivact-macros` と `kivact-bench` を足す（設計書 3.2 節の理由）。設計書 2.2 節の PERF-1〜3、MEM-1〜6、DEP-5 を「境界を作る規則」の後に表で足す。DEP の feature の表で `kivact` の default を `gpu, complex-scripts` にし、`arvane-text`（`complex-scripts`）と、`complex-scripts` を転送する consumer を足す。公開 API の規則（API-1・API-3・API-6・API-13）の旧名を新しい名前にし、ARG-1〜ARG-6 を表で足す。「状態と事実の所有」に SM-1〜SM-12、SL-1〜SL-8、6.4 節の監査を要約して足す。
2. `docs/for-kivact-developer/decisions.md`：設計書 8.1 節の置き換え（CONTRACT-01、ADR-34、ADR-47 の「決定」を新しい名前に書き換え、末尾に「2026-11-09 SLIM-05 で改名」と足す）と、8.2 節の SLIM-01〜SLIM-10 を「実装の決定」の表の末尾に足す。
3. `SLIM_FAST_DESIGN.md`、`SLIM_FAST_WORK_ORDER.md`、`20261109_2107_CODEX_REVIEW.md` を削除する（追跡されている file は `git rm`、されていなければ `rm`）。`dev/bench/results/` と `dev/xtask/state-audit.md` は残す。
4. commit し、push する。

### W7-3 PR の説明の下書き（AC をすべて満たしたときだけ）

`dev/bench/results/PR_DRAFT.md` に次を書いて commit し、push する（PR は作らない）：

- 概要（設計書 1.3 節の原因と、直した内容を 10 行以内）
- KPI の表（W7-1 の RESULT.md の表をそのまま）
- 公開 API の変更（付録 B の B.1 の表、ARG-1〜6、`#[component]`、`GpuStart`、`Binding::update`、`complex-scripts`）
- 撤退規則を適用した項目（受入れに影響しないもの）
- 確認の状況（CP3 の各コマンドの結果、Linux の確認の有無）

---

## 付録 A 状態機械の一覧（直した形まで）

各行の「直した形」は、その field を置き換える型。行の「規則」は設計書 6.3 節の番号。直したら、その field に付いていた `xtask-allow(state-fields)` の行も消す。「SM-11（理由）」の行は直さず、marker の理由を書いてある文言に置き換える。

各行は、その field を書くすべての箇所と到達する組合せを設計者が調べて確定したもの（2026-11-09）。実装者は「直した形」を写す。行の前提がコードと合わない（書く箇所が増えている等）場合は STOP（設計者に差し戻す）。

### A-RT（U16、kivact-runtime）

| file:行 | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `motion/state.rs:96-99` | `AnimInfo` | `running: bool`、`paused: bool` | SM-1 | `status: RunStatus`、`enum RunStatus { Idle, Running, Paused }`。`is_running()` は `status == Running`、`is_paused()` は `status == Paused` の投影 |
| `reactive/detail.rs:139` | `DetailBuilder` | `cause: Option<u32>`（None が「まだ無い」と「上限で捨てた」の二つの意味） | SM-4 | `cause: CauseSlot`、`enum CauseSlot { Empty, At(u32), Dropped }` |
| `task/worker_pool.rs:24` | `Pool` | `tx: Mutex<Option<Sender<..>>>`、`handles` | SM-2 | `workers: Mutex<Workers>`、`enum Workers { Unspawned, Running { tx: Sender<..>, handles: Vec<JoinHandle<()>> }, Closed }`（型引数は今の `tx` と `handles` の型） |
| `task/action.rs:112` | `Queue` | `worker_ids: Cell<Generation>` | SM-11 | id の割当て器（`ensure_worker` が Stopped の時だけ進める。今の worker の id は `Worker::Picking/Running{id}` が持つ）。名前を `next_worker_id` にし、理由 `identity: allocates worker ids; the current worker's id lives in Worker` |
| `task/action.rs:249` | `ActionCell` | `last: RefCell<Option<Submitted<I>>>` | SM-12 | 最後に始まった実行の入力（queue では enqueue と最初の poll の間は None、消されない）。書き手は Latest の `make`（`:515`）と worker の pop（`:585`）だけ。型は変えず、理由 `projection: the input of the latest started run, written only when a run starts (Latest make, worker pop)` |
| `motion/state.rs:425` | `LoopCell` | `rate: Cell<Option<Value<..>>>`（最初の呼び出しで埋める） | SM-5 | constructor（`frame_loop` の setup）で rate の `Value` を作って渡す。setup で作れない場合は STOP |
| `motion/state.rs:434` | `LoopRun` | `index`、`last_tick: Option<u64>`（一緒に reset） | SM-2 | `cadence: Cadence`、`enum Cadence { Fresh { start_ns: u64 }, Ticking { start_ns: u64, index: u64, last_tick: u64 } }`（今の `start_ns` の field があればそれを移す。型は今の型） |
| `reactive/data.rs:77` | `Runtime` | `ext: OnceCell<Extensions>` | SM-11 | 理由：`independent: ADR-17: filled by the constructor before it returns; never observed empty` |
| `task/state.rs:95` | `TaskCell` | `task: OnceCell<RawTask>` | SM-11 | 理由：`independent: ADR-21: set before the first poll` |
| `reactive/data.rs:79-81` | `Runtime` | `mount_revision`、`unstamped`、`awaiting` | SM-11 | 理由：`independent: ADR-18: typed queues of unstamped and awaiting mounts` |
| `reactive/data.rs:83-84` | `Runtime` | `state: TurnState`、`exit: ExitState` | SM-11 | 理由：`independent: ADR-18: the turn state and the exit state are separate facts` |
| `reactive/host_types.rs:80` | `TurnReport` | `next_deadline`、`exit` | SM-11 | 理由：`independent: ADR-18: the report carries the deadline and the exit state` |
| `task/async_state.rs:66` | `AsyncCell` | `state`、`latest: Signal<Option<T>>` | SM-12 | ADR-21 の通知の粒度（`latest` の読み手を Pending・Failed で起こさない）を保つため二つの signal のまま。`fn publish(&self, next: AsyncState<T>)` を一つ作り、`start`・`finish` は必ずそれを通す：`next` が `Ready(v)` なら先に `latest` に `Some(v.clone())` を書き、次に `state` を書く。他の箇所は二つの signal を書かない。`latest` の marker の理由は `projection: written only by AsyncCell::publish (Ready(v) implies latest == Some(v))` |
| `reactive/data.rs:362` | `Frame` | `owner`、`pure`、`writer`、`tracking` | SM-11 | 理由：`independent: ADR-20: the frame kind and pure/tracking/owner/writer are separate facts` |
| `reactive/data.rs:171` | `MemoCell` | `value: RefCell<Option<T>>` | SM-11 | 理由：`cache: the memo's last computed value; the graph state is CompState` |

### A-EC（U63、crate をまたぐもの）

| file:行 | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `kivact-engine/src/contract/host.rs:25` | `TurnOutput` | `next_wake_ns: Option<u64>`、`exit: bool` | SM-2 | `pub enum TurnOutput { Continue { next_wake_ns: Option<u64> }, Exit }`。読む側（`host/pump.rs` など）は `match` にする。他の field があれば `Continue` の variant に入れる |
| `kivact-engine/src/contract/routed.rs:7` | `Routed<T>` | `target: Option<NodeId>`、`region: Option<RegionId>`、`captured: bool`、`activation` | SM-3 | `target: Option<RouteTarget>`、`pub struct RouteTarget { pub node: NodeId, pub region: Option<RegionId> }`（region は target がある時だけ）。`captured` は読み手が無いので削除する（構築箇所：`model/pointer/machine.rs:62,702`、`pointer/shell.rs:401,417,439`、`keys.rs:183,415`、`kivact-test` の `host_sync_tests.rs:156,415`）。書換え：`part_pointer.rs:127`、`client/dispatch/key.rs:33`、`host/write/writer.rs:88`。`activation` はそのまま |
| `kivact-engine/src/contract/devtools.rs:123` | `ImeStateInfo` | `node`、`composing: bool`、`preedit: Option<..>`、`cursor_area` | SM-2 | `pub enum ImeStateInfo { Inactive, Focused { node, purpose, cursor_area, composition: Option<String> } }`（`composing` ⇔ `composition.is_some()`）。core の devtools の変換（protocol の型への写し）を直す。protocol の型（`kivact-devtools-protocol`）は変えない |
| `kivact-engine/src/host/pump.rs:57` | `Host` | `close: CloseSignal`、`phase: HostPhase` | SM-11 | 理由：`independent: ADR-09: the thread-safe close inbox; the host phase is drained from it` |
| `arvane-shell/src/pipeline/raw.rs:13` | `RawKey` | `pressed: bool`、`repeat: Option<bool>`、`text` | SM-2 | `phase: RawKeyPhase`、`pub enum RawKeyPhase { Down { repeat: Option<bool>, text: <今の text の型> }, Up }`。`RawKey` を作る・読む箇所（native の port、virtual、`crates/kivact-test/src/raw.rs`、`crates/kivact-core/src/devtools/input.rs`、`crates/kivact-engine/src/contract.rs`、`host/testing.rs`）をすべて直す |

### A-PS（U31、kivact-core）

| file | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `view/part_data.rs:49` | `PartState` | `focused: bool`、`focus_visible: bool` | SM-1 | `focus: Focus`、`pub enum Focus { None, Focused, FocusedVisible }`、`is_focused()`、`is_focus_visible()`（U31 手順 7） |

### A-AR（U32、arvane-*）

| file:行 | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `arvane-virtual/src/window.rs:146-180` | `VirtualWindow` | `ime`、`ime_text`、`occluded`、`failing`、`redraw`、`soft_keyboard` | SM-11 | すべて試験の台本が別々に入れる独立した入力（`ime = None` でも `ime_text` は消えない、遮蔽中は失敗の回数を消費しない `window.rs:474-479`）。型は変えない。各 field の理由は `independent: <この field を書く script の操作> is recorded separately; every combination is a valid script state`。`soft_keyboard: (Cell<bool>, Cell<u32>)` だけは `soft_keyboard_shown: Cell<bool>`（independent）と `soft_keyboard_requests: Cell<u32>`（`statistic: requests counted for tests`）の二つの field に分ける |
| `arvane-native/src/port/x11/notifier.rs:12` | `Notifier` | `thread: Mutex<Option<JoinHandle>>` | SM-2 | `thread: Mutex<UiThread>`、`enum UiThread { Unspawned, Running(JoinHandle<()>), Joined }`（Linux） |
| `arvane-native/src/port/x11/ime.rs:102` | `XimConnection` | `phase: Connection`、`session.context: Context`、`flight`、`pending` | SM-2 | `enum Connection { Handshake { protocol_window: u32 }, Established { protocol_window: u32, server_window: u32 } }`（今の Handshake は server_window が None でしか作られない）。`enum Context { Connecting, Open, Ready { im, ic, keys: Keys }, Disconnected { keys: Keys } }`、`enum Keys { Idle, Waiting { flight: Flight, pending: VecDeque<(KeyPressEvent, bool)> } }`（flight がある時だけ pending が意味を持つ。`degrade` は keys を keyboard に返すので Disconnected も keys を持つ）。型の名前（`im`、`ic`、`Flight`）は今の型。Linux 専用（0.6 節） |
| `arvane-shell/src/pipeline/ime.rs:14` | `ImeDriver` | `allowed: Sent<bool>`、`text: Sent<Option<ShellImeText>>`、`area` | SM-2 | `offer: Sent<ImeOffer>`、`enum ImeOffer { Disallowed, Allowed { area: <今の area の型>, text: Option<ShellImeText> } }` |
| `arvane-asset/src/store/entry.rs:50` | `Entry` | `generation`、`state` | SM-11 | `generation` は entry の一つの世代の識別子で、load・reload・recheck・raster のすべての job に付き、結果の受取りで照合する（`asset_store.rs:360,375,397,413` と `:432,489,504`）。型は変えず、理由 `identity: one incarnation of the entry, stamped on every job and checked on every result` |
| `arvane-asset/src/store/asset_store.rs:49` | `AssetStore` | `frame: Option<NonZeroU64>`（最初の `begin_frame` の前は None） | SM-4 | `frame: FrameClock`、`enum FrameClock { BeforeFirstFrame, Frame(NonZeroU64) }` |
| `arvane-asset/src/store/worker.rs:73` | `Worker` | `stop: Arc<AtomicBool>`、`done` | SM-11 | 理由：`independent: the cross-thread stop request; the thread's lifetime is its handle` |
| `arvane/src/presenter/frame_policy.rs:18` | `Drawn` | `shown: bool`、`raster`、`present` | SM-2 | `enum Drawn { Shown { renderer, raster, present, report }, NoBuffer { renderer, report } }`（`renderer` と `report` は今の field。型は今の型） |
| `arvane-compose/src/cache.rs:96` | `CacheIndex` | `invalidated: bool`、`frame`、記録の map | SM-2 | `struct CacheIndex { frame: u64, records: Records }`、`enum Records { Kept { chunks, layers }, Invalidated }`（`clear` は `Invalidated` にする。`store` は `Kept` に戻す。`plan/damage.rs:217` は `Invalidated` のとき `Damage::Full`）。`frame` は独立のまま |
| `arvane-render/src/derived.rs:52` | `Derived` | `context: Option<(RenderContext, Resources)>` | SM-11 | 理由：`cache: built on the first raster` |

### A-LE（U33）

| file:行 | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `kivact-layout/src/pass_state.rs:29` | `LayoutState` | `changed: bool`、`relayout: Vec<NodeId>` | SM-2 | まず `layout.rs:67` の `if !self.state.take_changed() || tree.root().is_none()` を `if tree.root().is_none() || !self.state.take_changed()` にする（root の無い時に changed を消費しない。今はこの経路で relayout だけが残る）。その後 `pending: Pending`、`enum Pending { Clean, Dirty { relayout: Vec<NodeId> } }` で二つを置き換える（印を付ける関数は `Dirty` にし、push は `Dirty` の relayout に足す。`take_changed` は `Dirty` を取り出す）。`epoch` は独立のまま（理由 `identity: generation of the traversal cursors`） |
| `kivact-layout/src/style/node.rs:69` | `NodeStyle` | `display: taffy::Display`、`grid: Option<Box<TStyle>>` | SM-2 | `display: NodeDisplay`、`enum NodeDisplay { None, Flex, Grid(Option<Box<TStyle>>) }`（grid の style は Grid の時だけ。Grid で container の prop が無ければ `None`）。taffy に渡す `display()` は `NodeDisplay` から作る。書き手：`convert.rs:185-190,302-310`。読み手：`tree.rs:334-338,423-426` |
| `kivact-editing/src/edit_state.rs:54` | `EditState` | `base`、`log`（と `revision`、`damage`） | SM-3 | 「log が空 ⇔ base == revision」を型にする：`log: Option<Log>`、`struct Log { base: u64, entries: Vec<Logged> }`（entries は空にしない）。`base` の field を消し、`recorded_since(r)` は `log.as_ref().map_or(revision, |l| l.base) == r`。`damage: Option<Damage>` は別の consumer（行の cache）の事実なので独立のまま（理由 `independent: line-cache damage is taken by the editor, separately from the edit log`） |
| `kivact-devtools-agent/src/agent.rs:30` | `Agent` | `stop: Sender<..>`、`listener: Option<JoinHandle>` | SM-2 | `listener: Listener`、`enum Listener { Running { stop: Sender<..>, thread: JoinHandle<()> }, Stopped }` |

### A-EN（U43、kivact-engine）

| file:行 | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `tree/node.rs:132-137` | `Place` | `sticky: bool`、`fixed: bool` | SM-1 | `positioning: Positioning`、`enum Positioning { InFlow, Sticky, Fixed }`。読む側の `place.fixed` は `place.positioning == Positioning::Fixed` |
| `tree/child_order.rs:19`、`:184` | `Editing`、`Children` | `first`、`last`（両方 Some か両方 None） | SM-3 | `ends: Option<(NodeIndex, NodeIndex)>` |
| `host/window.rs:61` | `HostWindow` | `last_stats`、`last_present` | SM-11 | 理由：`independent: the stats of the last shown frame stay readable while later presents are skipped` |
| `paint/state.rs:175-189` | `Step` | `own`、`grouped`、`group` | SM-1 | `grouping: Grouping`、`enum Grouping { None, Effects, Opacity }`（今の (group, grouped) の (F,F)、(F,T)、(T,T)。(T,F) は作られない。`walk.rs:551-552` で作る。`grouped` の読み手は `grouping != None`、`group` の読み手は `grouping == Opacity`）。`own` は独立（理由 `independent: whether the node draws itself; any grouping can hold either`） |
| `text_input/records.rs:76` | `Kept` | `run_id: Option<u64>`、`run: Option<(Point, Arc<SemanticsTextRun>)>` | SM-4 | `run: Run`、`enum Run { Unassigned, Id(u64), Published { origin: Point, run: Arc<SemanticsTextRun> } }`（Published の id は `run.id.0`）。書き手：`shaping.rs:477,545,586`、`a11y/text.rs:77` |
| `measure/state.rs:14`、`model/aggregate.rs` の `editors` | `MeasureColumns` | `natural`（Image）、`extents`（Text） | SM-4 | `leaf: Vec<LeafMeasure>`、`enum LeafMeasure { None, Natural(<今の natural の型>), Text(Box<<今の extents の型>>) }`（ノードの種類で排他。種類は作った後に変わらない）。editor を設定する `set_edit`（`inputs.rs:32`）で、その slot の `leaf` を `LeafMeasure::None` にする。`editors` と `computed_text` は独立のまま（理由 `independent: an editor overrides any node kind` と `cache: inherited text style of every node`） |
| `model/pointer/state.rs:93` | `MouseTrack` | `at: Option<Point>`、`over`、`hover` | — | `at` を `Point` にする（一度 Some になると None に戻らず、読み手はすべて `unwrap_or_default`）。`over` と `hover` は独立（`clear_hover` は chain だけを空にする。理由 `independent: the pointer's target and the hover chain are cleared separately`） |
| `lower/assets.rs:49`、`paint/list/walk.rs:85`、`model/events.rs:15` | `ArvaneAssets`、`Walk`、`EventQueue` | `Generation`/`Revision` の counter | SM-11 | 理由：`identity: generation that versions <対象>; readers compare it with their own` |

### A-CO（U44、kivact-core・kivact-test）

| file:行 | 型 | 今の field | 規則 | 直した形 |
|---|---|---|---|---|
| `view/tree.rs:255-262` | `NodeObservation` | `rect`、`layout_rect`（最初の layout の前は両方 None） | SM-3 | `measured: Signal<Option<Measured>>`、`struct Measured { rect: Rect, layout_rect: Rect }`（型は今の型）。`NodeRef` の読み手を追従させる |
| `view/devtools_state.rs:64-66` | `SessionData` | `highlight`、`pick` | SM-4 | `inspector: Inspector`、`enum Inspector { Idle { highlight: Option<Highlight> }, Picking { window: WindowId, hover: Option<Highlight> } }`（`Highlight` は今の highlight の型）。engine 側の写しは変えない |
| `gpu_interop/share/app_side.rs:31` | `SharedImage` | `current`、`pixels` | SM-12 | `current` は engine と共有する `Inbox<LinkState>` の phase の UI 側の投影（thread をまたぐので同じ容器に入れられない）。書き手は `import`（`:208`）と `renew`（`:177`）だけにし、理由 `projection: written only by import and renew from the link phase`。`pixels` は独立（理由 `independent: the last pixels delivered, kept for every phase`） |
| `view/state.rs:151` | `FocusKeys` | `node`、`visible`（`visible` は `node` が Some の時だけ） | SM-2 | `focus: Signal<Option<(FocusTarget, bool)>>`、または `struct KeyFocus { target: FocusTarget, visible: bool }` の Option |
| `view/mount/part_pointer.rs:431` | 関数の引数 | `delivery: (bool, bool)`、`&mut bool` | SM-1 | `enum Delivery { NoRaw, Raw, RawFinishingFallback }`（(deliver_raw, finish_fallback) の (F,F)、(T,F)、(T,T)。(F,T) は作られない。`select_pointer` `:331-335` が作る）。`&mut bool` の代わりに、fallback に届いたかを戻り値 `Result<FinishReached, UiError>`、`struct FinishReached(bool)` で返す |
| `view/state.rs:229` | `ViewState` | `initial_render: Signal<bool>` | SM-11 | 理由：`independent: one-way latch: the first frame was drawn` |

### A-IN（U41、kivact-interaction）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `combobox/handle.rs:368-391, 402-415` | `ComboboxApi` の popup | caller の `open: Binding<bool>` と `ComboMode` の Machine、`:386` の effect が `open.set(false)` | SM-9 | Machine が正本。`open` を `feed` で `Event::OpenRequested(bool)` として入れ、`Event::FocusLeft { transferred: bool }` を足し、Machine の `Effect::WriteOpen(bool)` で binding に書く。`:386` の effect を消す |
| `palette/handle.rs:~330-336` | `CommandPaletteApi` | `effect_on(open)` → `combobox.reset()` | SM-8 | 上の Machine の Close の遷移で reset する。effect を消す |
| `menu/level.rs:259-298`、`menu/state.rs:54` | `watch_item` | `hovered`、`aiming`、`disabled`、`shown`、`pressed_over: Option<bool>` と 3 つの effect | SM-8 | 一つの `Event::Pointer { hovered: bool, press: PressOver }`、`enum PressOver { None, Over, Away }` を一つの `effect` から session に送る |
| `zoom_pan/handle.rs:42-67` | `ZoomPanApi` | space_pan の `armed` と Gesture の Machine | SM-1 | Space を Gesture の Machine に入れる：`Event::Space(bool)`、`State::Idle { space: bool }` |
| `tooltip/machine.rs:11-14`、`api.rs:60-75` | tooltip の hub | `Warmth::Warm { until_ns }` を読む時に比べる | SM-7 | `Step::after(<within の期間>, Event::Cool)`、`Warmth::Warm { excluded }` |
| `list/typeahead.rs:34-48` | `Typeahead` | `Typing { expires_ns }` | SM-7 | list と select の Machine で `.after(<window>, Event::Expire)` |
| `inline_edit.rs:138-150` | `use_inline_edit` | `sent: Var<Sent<Option<K>>>` | SM-8 | `reported` を `InlineEdit` の Machine の状態に入れる |
| `select/handle.rs:784-810` | `track_preview` | `reported: Var<Sent>` と effect と on_cleanup | SM-8 | `State::Open { previewed }` と `Effect::Preview` |
| `calendar/data.rs` | `CalendarDayState` | `selected`、`edge`（edge なら selected） | SM-1 | `selection: DaySelection`、`enum DaySelection { None, Inside, Edge }` |
| `table/data.rs` | `TableHeaderState` | `sort`、`sortable`、`sort_rank` | SM-2 | `sorting: HeaderSort`、`enum HeaderSort { Unsortable, Sortable, Sorted { direction: <今の型>, rank: <今の型> } }` |
| `popup/machine.rs:81-83` | `Machine` の input | `focused: Option<NodeToken>`、`focus_inside: bool` | SM-2 | `focus: HostFocus`、`enum HostFocus { Nowhere, Inside, Elsewhere(NodeToken) }`（`focus_inside` なら focused は Some で、その token は読まない）。`may_restore` は `!matches!(focus, HostFocus::Elsewhere(t) if Some(t) != target)`。書き手は `popup/api.rs:144-149` だけ |
| `text_edit/handle.rs:94, 505` | `TextEditApi` | `next_control: Cell<Generation>` | SM-11 | id の割当て器（古い control を捨てる判定は既に `machine::State::Mounted { id }`）。名前を `next_control_id` にし、理由 `identity: allocates control ids; the mounted id lives in the machine state` |
| `list/state.rs:56,85,92,132` | `ListApi` | `typed`、`pending`、`roving.requested`、anchor | SM-11 | 四つは別の事実（検索の buffer、mount 待ちの活性化、tab stop、選択の anchor）で、それぞれ別の操作で動く（`requested` は Cancel なしで動き、anchor は `requested` と独立に動く）。各 field の理由に、その事実を書く（`independent: ..`）。`pending` の Machine の状態の `focus: Physical | Retained` は独立の mode なので、状態の中の field のままにする |

### A-UI（U42、kivact-ui）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `selection_field.rs:72-80, 148-165, 184-215` | `SelectionField` | `open` の binding、SheetGesture の Machine、surface を untracked で写す memo、`effect_on(open)` | SM-9 | Machine `enum FieldState { Closed { gesture }, Open { surface } }`、`open` は SM-9 の形 |
| `date_input/view.rs:236-250` と `159-185` | `DateInput` | trigger の経路だけ `open = signal(false)` | SM-9 | 両方の経路で `date_controller` を使う |
| `date_input/view.rs:170-185`、`date_input/machine.rs:9-32` | `DateInput` | `DateInputState.draft` と `TypedDate` の二つの Machine、`:185` の effect | SM-8 | 一つの Machine の状態 `struct DateField { typed: TypedDate, picker: Picker }`、`enum Picker { Closed, Open { opened_by } }`。effect を消す |
| `date_range_input.rs:190, 263, 425, 480` | `DateRangeInput` | `open: Signal<bool>` が draft の Machine の外 | SM-9 | `RangeDraft` に `picker: Picker` を足す |
| `data_table/filters.rs:294-312` | text の filter の editor | `sent: Var<Sent>` を Machine の input と effect の両方が使う | SM-8 | `FilterDraft::Valid { text, value, sent }` |
| `drawer_swipe.rs:14-42` | drawer の handle | `pulled: f64`（drag 中だけ 0 でない） | SM-10 | `enum Swipe { Resting, Pulling(f64) }` を drag の Start/Update/End/Cancel で動かす（`toast_swipe` の Machine と同じ形） |
| `alert.rs:355-395` | `BannerMachine` | `BannerState::Shared { visible: Binding<bool> }`、transition が `visible.get()` を読む | SM-9 | binding を input の snapshot に入れ、transition を純粋にする |
| `combobox/machine.rs:5-25` | `ComboboxState` | `Query::Shared(Binding<String>)` | SM-9 | binding を input に移す |
| `data_table/rows.rs:901-917` | row の drag | `DragApi.on_start` で始める domain の Machine | SM-10 | drag の 4 つの event を Machine に送る |
| `component_values.rs:11, 30` | recipe の state | `expanded: Option<bool>`、`selected: Option<bool>` | SM-4 | `expansion: Expansion { NotExpandable, Collapsed, Expanded }`、`selection: Selectable { NotSelectable, Unselected, Selected }` |
| `data_table/types.rs:55-62` | `DataTable` の option | `page`、`page_size`、`has_more` | SM-4 | `paging: Paging`、`enum Paging { Off, Local { page: usize, page_size: usize }, Remote { page: usize, has_more: bool } }`（型は今の型） |
| `values.rs:245-260` | `SwitchDragState`（公開） | `offset`、`touch`、`tracking`、`lifted`、`released` | SM-1 | `pub enum SwitchDragState { TouchPressed, Tracking(TrackInput), TouchReleased(Lift) }`、`pub enum TrackInput { Touch { offset: f64 }, Pointer }`、`pub enum Lift { Raised, Settled }`（到達する 5 通りだけ。それ以外は今どおり `None`）。書き手は `switch_drag/machine.rs:93-126` の `visual()` だけ。読み手：`machine.rs:223`、`kivact-ui-soft/src/recipes_controls_choice.rs:188-283`（U55 が追従） |
| `chat_values.rs:98-103` | `ChatComposerState` | `stopped`、`status` | SM-11 | どちらも caller の値の写し（`chat_composer.rs:353-356`）で、すべての組合せが有効。理由 `independent: copied from the caller's stop_shown and status` |
| `hover_card.rs` | `HoverCard` | `open`、`default_open` | SM-4 | `open: CardOpen`、`enum CardOpen { Controlled(Binding<bool>), Uncontrolled(bool) }` |
| `code_values.rs` | `CodeBlockState` | `expanded`、`copied` | SM-11 | 理由：`independent: the disclosure and the copy feedback are separate parts` |
| `button.rs:241, 389, 395` | button の memo | 設計書 MEM-6 | — | U42 の追加手順 4 |

### A-DT（U63、kivact-devtools）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `panels/accessibility.rs:14-20` | a11y の panel | `tree`、`audit`、`focus`、`tab_order`（4 つの Option）、`tree_selected` | SM-3 | `resource(..)`（runtime の `AsyncState`）で `struct A11yData { tree, audit, focus, tab_order }` を一度に読み込む。`tree_selected` は `A11yData` が Ready の時だけ使う |
| `panels/review.rs:10-12, 40-55` | comment の編集 | `editing: Option<String>`、`draft: String` | SM-2 | `edit: Signal<CommentEdit>`、`enum CommentEdit { Idle, Editing { id: String, draft: String } }` |
| `panels/elements/styles.rs:263-275`、`panels/theme.rs:48-60` | override の行 | `enabled`、`value`、捕まえた `existing_id`、`effect_on(enabled)` | SM-8 | Machine `enum Override { Off { value }, On { id: Option<<型>>, value } }`。effect を消す |
| `panels/scripts.rs:45-51, 80-85, 195, 224, 251` | `Runner` | `state: Signal<ScriptRun>`、`child: Rc<RefCell<Option<Child>>>` | SM-2 | `ScriptRun::Running { child }` に child を持たせる（`Child` が `Clone` でなければ `Rc<RefCell<Child>>` を payload にする） |
| `session.rs:29-31` | `Session` | `panel`、`popped`、`status` | SM-11 | Pop out は `panel` を変えないので、選択中の panel が popped に入る・入らないの両方が有効。`status` は RPC の失敗の報告で独立。各 field に理由を書く |
| `panels/performance.rs:24-27`、`animations.rs:10`、`input.rs:10-11`、`tasks.rs:10`、`theme.rs:11`、`elements/styles.rs:117` | data の panel | `signal(None::<T>)` を「読み込み中」に使う | SM-4 | `resource(..)` の `AsyncState` |
| `panels/elements/view.rs:21-28`、`widgets/virtual_tree.rs:21` | view | effect が highlight を送り直す、`scroll_to` が保留の Option | SM-8 | highlight は表示の状態から導く memo にする。`scroll_to` は SM-11（理由 `independent: one pending scroll request`） |

### A-KB（U51、kivact-ui-basic と catalog）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `crates/kivact-ui-basic/src/chart/view.rs:205-247` | chart の hover | `hit: Option<Hit>`、`at: Point`、`plot: Rc<Cell<Size>>` | SM-2 | `hover: Signal<ChartHover>`、`enum ChartHover { Away, Over { hit: Hit, at: Point } }`。`plot` は SM-11（理由 `independent: measured plot size`） |
| `examples/kivact-ui-basic/src/docs/dialog.rs:122-123`、`alert_dialog.rs:78-79` | catalog | `open`、`deleted` | SM-11 | 削除の後も trigger で開けるので 4 通りすべてが有効。`deleted` は一度だけ true になる記録。各 local に理由を書く |

### A-KP（U52、kivact-ui-pop と catalog）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `examples/kivact-ui-pop/src/pop/shell.rs:81-85`、`patterns.rs:52-56` | 開いている overlay | `confirm`、`form_dialog`、`sheet`、`command` の 4 つの bool | SM-1 | `overlay: Signal<Overlay>`、`enum Overlay { None, Confirm, Form, Sheet, Command }`。各 dialog の `open` の binding は `overlay.project(..)` 相当の `binding(move \|\| overlay.get() == Overlay::X, move \|open\| overlay.set(if open { Overlay::X } else { Overlay::None }))` |
| `examples/kivact-ui-pop/src/pop/components.rs:69, 860-875` | busy・verify・loading | bool と `spawn(sleep)` で戻す | SM-7 | Machine と `Step::after` |
| `examples/kivact-ui-pop/src/pop/messaging.rs:53-100` | 返信 | `replies`、`serial`、Typing の placeholder | SM-6 | `enum Reply { Idle, Waiting { id }, Typing { id } }` の Machine と `Step::after` |

### A-KG（U53、kivact-ui-game と catalog）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `crates/kivact-ui-game/src/surface.rs:73-84` | halo | effect が `visible && !reduced && active` で clock を始める・止める | SM-8 | `kivact-ui` の progress の motion の Machine（`ProgressMotionPhase` の形）と同じ形の Machine |
| `examples/kivact-ui-game/src/game/state.rs:7-14` | dialog の開閉 | `create`、`delete`、`drawer`、`palette` | SM-1 | A-KP の Overlay と同じ形 |
| `examples/kivact-ui-game/src/game/system.rs:49-55, 113-118` | dialog を閉じた時に `title`・`confirmation` を消す effect | SM-8 | `enum Editing { Closed, Open { title } }` |
| `examples/kivact-ui-game/src/game/controls.rs:29-30, 228-250` | busy 等 | bool と sleep | SM-7 | Machine と `Step::after` |
| `examples/kivact-ui-game/src/game/window.rs:15-50` | `running`、`status` の文言 | SM-8 | `enum Run { Idle, Running, Paused }` から文言を導く |

### A-KC（U54、kivact-ui-creative と catalog）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `editors/node/gesture.rs:86-150`、`node/edit.rs:136, 172` | `NodeController` | `ModeEvent::Choose(NodeMode)` と `update(FnOnce)` が setter の代わり、5 つの bool の `Inputs` | SM-1 | 意味の event（`BeginConnect`、`OpenPopup`、`Cancel` など、今 `Choose` で設定している mode ごとに一つ）を足し、`Choose` と `update` を消す |
| `editors/shared/gesture.rs:13-62` | `DecisionMachine` | `State = Option<Decision { payload: Option<Drag> }>` | SM-4 | `enum Press { Idle, Deciding { .. }, Dragging { drag, .. } }` |
| `arrangement/gesture.rs:72`、`graph/gesture.rs:92`、`node/gesture.rs:232`、`time/gesture.rs:56` | editor の State | `active: Signal<Option<Target>>` | SM-8 | `shared/targets.rs:89` の `TargetState` の Machine に入れる |
| `numeric/implementation.rs:377-379, 655-700` | `NumericField` | `typing` と `scrub_phase` と drag の session | SM-10 | `enum NumericMode { Idle, Scrubbing { moved: bool }, Typing }` |
| `dock/data.rs:60`、`chrome.rs:194, 532-544` | dock の tab | `drag: Signal<Option<TabDrag>>` と `rename` | SM-4 | `enum DockGesture { Idle, Dragging(TabDrag), Renaming(<key の型>) }` |
| `dock/data.rs:22-27`、`operations.rs:102-115` | tab の rect | `next_rect: Cell<Generation>` | SM-11 | 理由：`identity: generation of registered tab rects` |
| `outliner/view.rs:219, 466-467, 505-525` | row の registry | `Rc<Cell<Generation>>` | SM-11 | 理由：同上の形（`registered rows`） |
| `outliner/view.rs:228, 363, 381-388, 468` | drop の計画 | `plan: Signal<Option<Plan>>`（drag 中だけ） | SM-10 | drag の payload に入れる |
| `tabs.rs:104-127` | document tab | `consumed: f64` | SM-10 | `enum Reorder { Idle, Dragging { consumed: f64 } }` |
| `panel_group/implementation.rs:38-45, 172-176` | panel の並べ替え | Machine を `on_start` で始める | SM-10 | drag の 4 つの event を Machine に送る |
| `picker.rs:526-543, 570, 609, 623` | `Picker` | `phase: Var<PickerPhase>` と `ColumnMachine` | SM-8 | `PickerPhase` を `ColumnMachine` の状態に入れる |
| `audio/transport.rs:14-21, 104-117` | `TransportState`（公開） | `playing`、`recording`、`cycling` | SM-11 | Stop は再生だけを止め、Play と Record はそれぞれ独立に切り替える（`:104,110,117`）ので 4 通りすべてが有効。型は変えない。各 field の理由 `independent: <Stop/Play/Record のどの操作で変わるか>` |
| `feedback.rs:459-460, 668` | tool window | `collapsed`、`offset`、cleanup の `open=false` | SM-8 | `enum ToolWindow { Closed, Open { collapsed: bool, offset: <型> } }` |
| `editors/shared/model.rs:21-23` | `EditorContext` | `item: Option<u64>`、`lane: Option<u64>` | SM-4 | `enum EditorFocus { Canvas, Lane(u64), Item { lane: u64, item: u64 } }` |
| `dock/items.rs:10-17, 92-106` | `Item::Chrome` | `shown`、`maximized` | SM-1 | `display: ChromeDisplay`、`enum ChromeDisplay { Normal, Maximized, Hidden }`（(shown=F, maximized=T) は作られない）。`Item::shown()` は `display != Hidden` |
| `editors/node/gesture.rs:190-198` | `DragMode::Link` | `rewire: Option<NodeLink>`、`from`、`output`、`released` | SM-4 | `source: LinkSource`、`enum LinkSource { Output { from, output }, Rewire(NodeLink) }`（rewire の時の from/output は link から写した重複）。`released` は独立のまま（理由 `independent: the socket under the pointer at release, for either source`） |
| `examples/kivact-ui-creative/src/creative/arrangement_model.rs:10-35`、`node_model.rs:97`、`animation_model.rs:171`、`editors.rs:33, 556-580` | 編集の transaction | Begin で snapshot、Commit/Cancel で消す | SM-2 | 共通の `enum Txn<T> { Idle, Editing { snapshot: T } }`（catalog の中の一つの module に置く） |
| `examples/kivact-ui-creative/src/creative/jobs.rs:24-70` | job | `running` と clock の effect と完了の確認 | SM-8 | Machine |
| `examples/kivact-ui-creative/src/creative/audio_values.rs:14-18, 57-75` | transport | 上と同じ | SM-11 | 上と同じ |
| `examples/kivact-ui-creative/src/creative/editors.rs:90-91` | `bypass`、`wipe` | SM-1 | `enum Grade { Graded, Bypassed, Wipe }` |
| `examples/kivact-ui-creative/src/creative/shell.rs:815-856` | shell | `compact`、`sidebar_open`、`search`、`category` | SM-11 | 三つの bool は別の操作で切り替わり、すべての組合せが有効（各 local に理由を書く）。`category: Signal<Option<Category>>` は None に一度もならない（`unwrap_or(All)` で読む）ので `Signal<Category>`（初期値 `Category::All`）にする |

### A-KS（U55、kivact-ui-soft と catalog）

| file:行 | 対象 | 今 | 規則 | 直した形 |
|---|---|---|---|---|
| `rich_text.rs:266-300` | `RichTextEditor` | `mode: EditorMode`、`write_composing: bool` | SM-2 | `EditorMode::Write { composing: bool }` |
| `rich_blocks.rs:171, 182-190, 233-245` | block の field | `command: Rc<Cell<Option<Command>>>` | SM-8 | `BlockEvent::Command` を BlockMachine に送る |
| `side_navigation/shell.rs:237-244, 489-510` | sidebar の resize | `collapsed: Option<Binding<bool>>`、`expanded_width` | SM-2 | `enum SidebarWidth { Collapsed { restore: f64 }, Expanded(f64) }` |
| `bottom_sheet.rs:237, 334-360` | `BottomSheet` | `Signal<Sheet { Resting, Dragging }>` を drag の callback で手で遷移 | SM-10 | drag の event で動く Machine |
| `soft_chat_tools.rs:47-125` | `ToolCall`・`ToolStatus`（公開） | `status`、`duration`、`error` | SM-2 | `pub enum ToolStatus { Pending, Running, Complete { duration: Option<Duration> }, Error { message: Option<String> } }`（`Copy` でなくなる）。`ToolCall` の `duration` と `error` の field を消す。`ToolCall::new` は `Complete { duration: None }`。builder の `.duration(..)` と `.error(..)` は消し、`.status(ToolStatus::Complete { duration: Some(d) })`・`.status(ToolStatus::Error { message: Some(m) })` の一つの経路にする。描画：duration は Complete の時（`:291`）、error の文言は Error の時だけ（`:722-732`）。catalog・試験（`chat_keyboard.rs:73,154`、`collections.rs:356-357`）と README を直す |
| `soft_chat_tools.rs:741-760` | tool の詳細 | `expanded`、`rows_visible` | SM-8 | `rows_visible` を `expanded` と表示から導く |
| `chat_dictation.rs:18-29` | `DictationState`（公開） | `supported`、`listening`、`volume`、`bands` | SM-2 | `pub enum DictationState { #[default] Unsupported, Idle, Listening { volume: f64, bands: Vec<f64> } }`（listening は supported を含意し、volume と bands は聞いている時だけ描く `:134-142`）。catalog（`examples/kivact-ui-soft/src/docs/collections.rs:344-347` は `Idle`）と README を直す |
| `lightbox.rs:70-79, 110-160` | `Lightbox`、`LightboxHandle` | builder の `open`、`index`、`view` | SM-12 | `lightbox(open, media)` は caller の binding（ARG-4）で、`LightboxHandle` の経路では `index`・`view` の binding は handle の一つの `Signal<LightboxState>` の投影。型は変えない。各 field の理由 `projection: written only through LightboxHandle's state` か `option: caller binding`。`LightboxHandle::open(self, index)` を `show(index)` に改名する（ARG-6） |
| `mobile_nav.rs:55-60, 110-125` | `AppShellMobile` | `{Desktop, Mobile}` の Machine と外の `open`、effect の補修 | SM-9 | `Mobile { open: bool }` を Machine の状態に入れる |
| `side_navigation/items.rs:429-435` | `ItemBranch::Parent` | `expansion`、`flyout` | SM-11 | `expansion` は item ごとの AlwaysOpen の有無、`flyout` は shell の collapsible の有無で決まり、4 通りすべてが有効。理由を書く |
| `side_navigation/context.rs:8-14` | `SideNavCx` | `collapsed`、`popup` | SM-4 | `surface: NavSurface`、`enum NavSurface { Fixed, Collapsible(Binding<bool>), Flyout { open: Binding<bool> } }`（(Some, Some) は作られない。構築：`shell.rs:240`、`items.rs:763,809`）。`depth` はそのまま |

## 付録 B 改名

### B.1 改名表

| 旧 | 新 | 行う unit |
|---|---|---|
| trait `NodeBuilder` | trait `Element`（supertrait `IntoView`） | U31（一括）、U41〜U56（追従） |
| trait `Component` | 削除。`impl Component` は `impl Element` | 同上 |
| 関数 `kivact::view::component(..)` | `kivact::view::scope(..)` | 同上 |
| `kivact::test::Element<'a>` | `kivact::test::Locator<'a>` | 同上 |
| `PartState.focused`、`PartState.focus_visible` | `PartState.focus: Focus`、`is_focused()`、`is_focus_visible()` | U31（core）、U42・U51〜U55（追従） |
| `select(label, options, ..)` などの引数名 | `items`（名前だけ） | U42 |
| `list(label, items).selection(s)` | `list(label, items, s)` | U42、kit は追従 |
| `alert.visible(b)` | `alert.open(b)` | U42、kit は追従 |
| `item_row.on_press(impl Fn() -> R)` | `impl Into<Callback<()>>` | U42 |
| `breadcrumb.on_activate(impl Fn(K))` | `impl Fn(K) -> R + 'static`（`R: HandlerResult`） | U42 |
| game `icon_menu.on_select` | `on_activate` | U53 |
| creative `SplitButton.on_select`、picker の `on_pick`、outliner の `on_item_activate` | `on_activate` | U54 |
| soft `lightbox.open(self, i)`、`mobile_nav.open(self)` | `show(i)`、`show()` | U55 |
| soft `mobile_nav.width(IntoValue<f64>)` | `width(IntoValue<Length>)` | U55 |
| soft `ToolCall` の `status`・`duration`・`error` | `ToolStatus` の variant（付録 A の A-KS） | U55 |
| soft `DictationState` の struct | enum（付録 A の A-KS） | U55 |

### B.2 一括置換の script（U31 手順 2）

repository の root で、Git Bash か Linux の bash で実行する。`grep -rlE` で対象を選び、対象が無い置換は何もしない。

```bash
set -euo pipefail

# 置換の対象：U32〜U35 の crate、kivact-style、kivact-engine を除いたすべての Rust と Markdown と TOML（除いた crate に名前が無いことは U31 の手順 1 で確かめる）
targets() {
  git ls-files -- \
    'crates/kivact/*' 'crates/kivact-core/*' 'crates/kivact-test/*' 'crates/kivact-interaction/*' \
    'crates/kivact-ui/*' 'crates/kivact-ui-basic/*' 'crates/kivact-ui-pop/*' 'crates/kivact-ui-game/*' \
    'crates/kivact-ui-creative/*' 'crates/kivact-ui-soft/*' 'crates/kivact-devtools/*' \
    'crates/kivact-macros/*' \
    'examples/*' 'templates/*' 'dev/tests/*' 'dev/bench/*' 'dev/xtask/*' 'docs/*' \
    'README.md' 'README_JP.md' 'CONTRIBUTING.md' \
  | grep -E '\.(rs|md|toml)$'
}

# 1. 試験の locator（kivact-test の中は Element がすべて locator）
grep -rlE '\bElement\b' crates/kivact-test/src | xargs -r sed -b -i -E 's/\bElement\b/Locator/g'
# kivact-test の外：path 付き・lifetime 付き・1 行の use の中・doc の `Element::`
targets | grep -v '^crates/kivact-test/' | xargs -r sed -b -i -E \
  -e 's/\b(kivact::test|kivact_test)::Element\b/\1::Locator/g' \
  -e "s/\bElement<'/Locator<'/g" \
  -e 's/(use kivact(_test)?::test::\{[^}]*)\bElement\b/\1Locator/g' \
  -e 's/`Element::/`Locator::/g'

# 2. NodeBuilder -> Element
targets | xargs -r sed -b -i -E 's/\bNodeBuilder\b/Element/g'

# 3. Component trait -> Element（Rust の型の位置だけ）
targets | grep -E '\.rs$' | xargs -r sed -b -i -E \
  -e 's/\bimpl Component\b/impl Element/g' \
  -e 's/(: *|\+ *)Component\b/\1Element/g' \
  -e 's/\bview::Component\b/view::Element/g'
targets | grep -E '\.md$' | xargs -r sed -b -i -E \
  -e 's/\bimpl Component\b/impl Element/g' \
  -e 's/`Component`/`Element`/g'

# 4. 関数 component(..) -> scope(..)（closure を渡す呼び出しと path だけ。`.component(` の method は変えない）
targets | xargs -r sed -b -i -E \
  -e 's/\bview::component\b/view::scope/g' \
  -e 's/(^|[^.:A-Za-z0-9_])component\((move )?\|\|/\1scope(\2||/g'
sed -b -i -E 's/pub fn component<V: IntoView>/pub fn scope<V: IntoView>/' crates/kivact-core/src/view/boundary.rs

# 5. 確認（出力は U31 の報告に貼る）
targets | xargs grep -nE '\bNodeBuilder\b|\bimpl Component\b|view::component\b' || true
```

置換の後、`use` の並びに `Element` が二重に入った箇所（旧 `NodeBuilder` と旧 `Component` の両方を import していた箇所）は、compile error（`the name Element is defined multiple times`）が出るので、一つを消す。
