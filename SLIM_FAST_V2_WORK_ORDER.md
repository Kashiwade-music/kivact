# Kivact Slim-Fast V2 作業指示書

版 2.2（2026-11-10、W1 の後の再監査（V2.1）と、実装予定の AI のレビュー `20261110_1828_CODEX_REVIEW.md` の妥当な指摘（V2.2）を反映。V2 設計書 0.4・0.5 節）。版 2.0：設計は [SLIM_FAST_V2_DESIGN.md](SLIM_FAST_V2_DESIGN.md)（以下「V2 設計書」）と付属書 `slim-fast-v2/D1〜D8`（以下「付属書」）。本書は手順だけを書く。設計の判断は V2 設計書と付属書が持つ。旧作業指示書 `SLIM_FAST_WORK_ORDER.md` は使わない。

目標：V2 設計書 1 章の AC-1〜AC-4 をすべて満たす。

---

## 0. 読み方と規則

### 0.1 誰が何をするか

| 役割 | 担当 | すること |
|---|---|---|
| Orchestrator | 本書を受け取った AI（親） | wave を順に進める。lane ごとに worktree を作り、unit ごとに subagent を起動する。結果を merge する。checkpoint（CP）の gate と計測を実行する。subagent の `STOP:` に 0.5 節の表で答える。**設計の判断をしない** |
| Implementer | subagent（**同時に最大 6 体**） | 割り当てられた unit を、指定の worktree の中で、付属書の手順どおりに実装する。完了条件のコマンドを実行し、結果を報告する |

**実行プロファイル**（unit・lane・wave・STOP の規則は実行環境によらず同じ。変わるのは subagent を起動する道具だけ）：

| 実行環境 | subagent の起動 | モデル |
|---|---|---|
| Claude Code | Agent tool を `subagent_type: "general-purpose"`、`model: "haiku"`、`effort: "high"` で呼ぶ | Haiku 5.5、effort high |
| Codex | その環境の委譲の入口（例：`collaboration.spawn_agent`）に、1.2 節の雛形の文をそのまま渡す。モデルを選べる場合はユーザーが指定したもの（指定が無ければその環境の既定） | ユーザーの指定 |
| その他 | その環境の subagent の起動の道具に、1.2 節の雛形の文をそのまま渡す | ユーザーの指定 |

- どの環境でも：同時に動かすのは最大 6 体（lane A〜F に 1 体ずつ）。subagent に、さらに subagent を作らせない（雛形の文に書いてある）。雛形の「モデルは Haiku です」の 1 文は、使うモデルの名前に読み替えてよい（それ以外の文は変えない）。
- Implementer は push・merge・PR 作成をしない。main に触れない。
- Orchestrator は push を wave の merge の後と checkpoint の後にだけ行う（`git push origin refactor/kashiwade/slim-fast`。force push 禁止）。PR は作らない。main に merge しない（人間が PR で行う）。

### 0.2 禁止事項（全 unit 共通）

1. 互換のためのコードを書かない：旧名の alias、`#[deprecated]`、wrapper、旧と新の二重の経路。旧名は消し、呼出し元を全部直す。
2. CI の設定・workflow・CI 用の script を足さない。
3. 付属書が名前を挙げていない試験を足さない。付属書が挙げた E2E（`dev/tests/kivact-public-api/tests/e2e/` など）と、付属書が明示した `#[cfg(test)]` の試験だけを足す。
4. 期待値を緩めて試験を通さない（許容差を広げる、assert を消す、`#[ignore]` を付ける）。
5. `#[allow(..)]`・`#[expect(..)]`・`xtask-allow` を、警告や lint の件数を減らす目的で足さない。付属書が文言まで指定した marker だけを書く。
6. 1 関数 80 code 行・1 ファイル 1000 code 行の上限（xtask）を超えない。超えそうなら意味のある helper に分ける。
7. 実装途中に workspace 全体の `cargo test`、E2E の全件、`dev/bench/run.sh` を実行しない（checkpoint で Orchestrator が実行する）。unit で実行してよいのは 0.6 節のコマンドと、付属書の unit の行の「完了時の確認」のコマンドだけ。
8. 依存の crate を足さない（付属書が名前を挙げたものを除く）。
9. `std::cell::OnceCell`、`std::sync::OnceLock`、`LazyCell`、`LazyLock` は使わない（`clippy.toml` で禁止）。
10. **行番号を信じない**。付属書の `file:line` は HEAD `73c36c7c` のもの。前の unit がファイルを変えているので、関数名・型名・field 名・周りのコードで場所を探す。
11. 付属書の「8 章 設計者（人間）の判断が要る点」を読んで自分で選ばない。V2 設計書 4 章の決定（DEC-01〜DEC-23）に従う。
12. 所有ファイル（本書の unit の行と付属書の unit の行）の外を変えない。ただし、型・関数の署名を変えた結果の「compile を通すための機械的な追従」は、同じ crate の中なら行ってよい。他の crate なら STOP。

### 0.3 unit の読み方

本書の各 unit の行は次を持つ。

- **修正付属書の節**（W2 以降、V2.1・V2.2）：`slim-fast-v2/D9〜D14` のその unit の節（`## <unit ID>`）と、行の「本書の追加の指示」が名指す `D15-fix-review.md` の節（`## Q<n>：…`。D15 が D9〜D14 より優先）。unit の手順・コード・所有ファイル・確認の本体はまずここにある。修正付属書が「元の付属書のとおり」とした部分だけ、括弧内の元の付属書（D1〜D8）の節を読む。
- **付属書と節／付属書の unit**（W1 の表）：読む付属書のファイルと節、付属書の「6 章 unit 分割案」の行の名前。
- **本書の追加の指示**：付属書・修正付属書に加えて守る指示（V2 設計書 4・5 章の結果）。食い違ったら、本書 → 修正付属書 → 元の付属書 の順に優先する。
- 修正付属書が「期待値を変える」と一字一句書いた試験だけは、その期待値に変えてよい（0.2 の 4 の例外。例：DEC-26 の diagnostics の試験）。

### 0.4 停止して報告（STOP）

次の場合は、その項目の作業を止め、他の項目を続け、最後の報告に `STOP:` で始まる行を書く。

- 付属書の手順が実際のコードと合わない（関数・型・field が無い、型が違う、関数名で探しても見つからない）。
- 前の unit（性能の unit を含む）が、付属書が対象とする field・関数を消した・改名した・型を変えた（V2 設計書 5.4 節）。付属書がその場合の扱いを書いていればそれに従う。
- 手順を行うと所有ファイルの外を変える必要がある（0.2 の 12 の追従を除く）。
- 同じ compile error を 3 回直しても消えない。
- 既存の E2E の期待値を変えないと通らないと分かった。
- debug の `self_check`（a11y、hit、paint、layout）の不一致が出た。修正を試みずに、失敗した試験の名前、直前の操作、self_check のメッセージを書く。
- `state-fields` の lint が、付属書・修正付属書に無い field を報告した。

次は STOP にせず、unit の中で直す（設計の判断ではない）：1 関数 80 行・引数 7 個・1 ファイル 1000 行の上限を超えた（意味のある helper 関数に分ける、引数を struct にまとめる）、clippy の機械的な指摘（`collapsible_if` 等。clippy の提案どおりに直す）、rustfmt の並び（`cargo fmt` に従う）、付属書のコード片の `use` の過不足（compile error が示す `use` を足す・消す）。

書式：`STOP: <unit> <file>:<line> <何が合わないか 1 文> / <試したこと 1 文>`。

### 0.5 Orchestrator の既定の判断表

Orchestrator は設計の判断をしない。subagent の `STOP:` に、次の表のどれかが当たればその判断をする。当たらなければ、その項目を**設計者への差戻し**として `dev/bench/results/OPEN.md` の「V2 差戻し」節に 1 行で書き（unit、file:line、STOP の文、付属書の節）、他の作業を続ける。差戻しが 1 件でも残っていれば W8 で完了を宣言しない。

| 状況 | 判断 |
|---|---|
| 行番号がずれているが、同じ名前の関数・型・field が近くにある | その関数・型・field に対して手順を行うよう指示する（同じ subagent に SendMessage） |
| 所有ファイルの外の 1〜3 ファイルを変える必要がある | 同じ wave の他の lane の所有でなければ許可する。他の lane の所有なら、その項目を次の wave の同じ crate の lane に回す（その wave の unit の手順の最後に足す） |
| 性能の変更で既存の E2E が失敗し、2 回直しても通らない | 差戻し（撤退規則は無い。期待値も緩めない） |
| 状態の unit の対象 field が、前の unit で消えている（付属書が扱いを書いていない） | 差戻し |
| compile error が他の lane の未 merge の変更を前提にしている | その unit を次の wave に回す |
| subagent が完了条件のコマンドを実行していない | 同じ subagent に実行させる |
| 前の unit の残り（dead_code の警告、doc の古さ）で、この unit の確認コマンドが失敗する | その前の unit の lane の fix として 1.3 節の fix の手順で直す（警告の対象を消すか使う。`#[allow]` で黙らせない）。設計の判断が要る場合だけ差戻し |

### 0.6 unit で実行してよいコマンド

```bash
cargo fmt -p <crate>                                         # 必ず最後に
cargo check -p <crate> --all-targets                         # 何度でも
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --only <rule> --crate <crate>   # --report-audit は付けない（全体の audit を上書きする）
cargo test -p <crate> --lib <filter>                         # 付属書が名前を挙げた試験だけ
dev/tools/window-lock.sh --timeout 900 -- cargo test -p <pkg> --test <name> -- <filter...>   # 付属書が名前を挙げた E2E だけ
```

- 実 window を開くコマンド（E2E、`kivact::test`、catalog の試験、`kv_*`）は**必ず** `dev/tools/window-lock.sh --timeout <秒> -- <command>` を通す（lane の間で同時に window を開かないため）。
- すべての cargo のコマンドは環境変数 `CARGO_TARGET_DIR=<1.1 節の target>`、`CARGO_PROFILE_DEV_DEBUG=0`、`CARGO_PROFILE_TEST_DEBUG=0` を付けて実行する（debug assertions は有効のまま。ディスクと PDB の上限のため）。
- `--features` が要る crate（`kivact-engine` の `testing` 等）は付属書の確認のコマンドに書いてあるとおりに付ける。
- Linux 専用のコード（`cfg(target_os = "linux")`、`port/x11`、`port/wayland`）を変える unit は、`wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}" -- bash -lc 'cd <worktree の WSL path> && cargo check -p <crate>'` か `cargo check -p <crate> --target x86_64-unknown-linux-gnu` で確かめる。どちらもできなければ Linux 専用の項目を STOP にする。

---

## 1. 体制

### 1.1 branch・worktree・target

- 統合 branch：`refactor/kashiwade/slim-fast`。
- **lane の worktree は wave ごとに作る**。wave `<W>` の lane `<L>`：

```bash
git worktree add ../kv-wt/W<W>-<L> -b wt/W<W>-<L> refactor/kashiwade/slim-fast
git -C ../kv-wt/W<W>-<L> submodule update --init
```

  その lane の unit はすべて同じ worktree で、表の順に一つずつ行う（unit ごとに commit）。lane の中の unit は同時に動かさない（前の unit の commit を次の unit が使う）。
- target：W0 の手順 4 で決める。空きが 70 GB 以上あるドライブがあれば lane ごと（`<そのドライブ>/kv-target/<L>`）。無ければ 3 つを共有する（A と B が `kv-target/AB`、C と D が `kv-target/CD`、E と F が `kv-target/EF`。cargo の lock で build が順になるが、依存の crate を共有してディスクを節約する）。同じ target を使う 2 つの lane の build は lock で待つ。これは正常。
- 各 wave の始めに空きを確かめ、15 GB 未満なら、その wave で使わない target dir を削除する（`rm -rf`。使っている target は消さない）。

### 1.2 subagent への指示の雛形

Orchestrator は各 unit に次の文をそのまま渡し、`<..>` だけを埋める。

```text
あなたは Kivact の unit <unit ID> の実装者です。モデルは Haiku です。あなた自身は subagent を作らないでください。
作業場所は <worktree の絶対 path> です。この directory の外の file を変更しないでください。
すべての cargo のコマンドに次の環境変数を付けてください：CARGO_TARGET_DIR=<target の絶対 path> CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
最初に次を読んでください：
  1. <worktree>/SLIM_FAST_V2_WORK_ORDER.md の 0 章（規則）と、表の「<unit ID>」の行
  2. その行が指す修正付属書：D15（<worktree>/slim-fast-v2/D15-fix-review.md）の、行が名指す「## Q<n>：…」の節（あれば。D15 が最優先）と、<worktree>/slim-fast-v2/<D9〜D14 のファイル>.md の「## <unit ID>」の節（W2 以降。W1 の unit は無し）
  3. 修正付属書が参照する元の付属書 <worktree>/slim-fast-v2/<D1〜D8 のファイル>.md の <節>（unit の行「<付属書の unit>」を含む 6 章も）
  4. <worktree>/SLIM_FAST_V2_DESIGN.md の 4 章（決定）と 5 章（付属書間の調整）のうち、この unit に関係する行
修正付属書の手順（無ければ付属書の手順）を順に実行してください。行番号はずれています。関数名・型名・field 名で場所を探してください。
本書の「追加の指示」が付属書と食い違うときは本書に従ってください。
設計の判断が必要になったら作業を止め、0.4 節の書式で STOP: 行を書いてください。
完了時の確認のコマンド（修正付属書の節、無ければ付属書の unit の行）をすべて実行してください。続けて、変更した各 crate について次の 3 つを実行し、error と警告を直してください：cargo clippy -p <crate> --all-targets -- -D warnings（feature なし）、cargo clippy -p <crate> --all-targets --all-features -- -D warnings、cargo xtask check --no-write --crate <crate>（--report-audit は付けない）。最後に cargo fmt -p <crate> を実行してください。
最後に `git -C <worktree> add -A && git -C <worktree> commit -m "<unit ID>: <要約>"` で commit してください。push・merge はしないでください。
報告には次だけを書いてください：変更した file の一覧、完了時の確認の各コマンドの結果（成功/失敗と最後の数行）、STOP: 行。
```

### 1.3 merge の手順（Orchestrator、各 wave の終わり）

lane A から F の順に：

```bash
git checkout refactor/kashiwade/slim-fast
git merge --no-ff wt/W<W>-<L> -m "W<W>-<L>: <unit ID の列>"
```

- conflict：
  - `dev/tests/kivact-public-api/tests/e2e/main.rs` の `mod` 行：両方の行を残し、アルファベット順に並べる。
  - `Cargo.lock`：`git checkout --theirs Cargo.lock && cargo metadata --format-version 1 >/dev/null` で作り直す。
  - それ以外：本書の wave の表でそのファイルを所有する lane の版を採る。同じ wave で二つの lane が同じファイルを持つことは無い（V2 で機械的に確かめた）ので、それ以外の conflict は起きないはず。起きたら STOP して人間に報告する。
- 全 lane を merge したら 1 回だけ：

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
```

  失敗したら、error を出した crate を所有した lane の最後の unit の subagent に、error の全文を渡して直させる。直しは統合 branch の今の HEAD から作った worktree `../kv-wt/W<W>-<L>-fix`（branch `wt/W<W>-<L>-fix`）で行い、同じ手順で merge する。
- merge の後、worktree と branch を消す：`git worktree remove ../kv-wt/W<W>-<L> && git branch -D wt/W<W>-<L>`。
- push する。

### 1.4 wave の一覧

| wave | 内容 | lane | 前提 |
|---|---|---|---|
| W0 | 準備・B0-here の計測（Orchestrator 自身） | — | — |
| W1〜W4 | 性能・メモリ・サイズ・GPU OnDemand（AC-2、AC-4） | A〜F | 前の wave |
| **CP1** | gate と KPI（Orchestrator） | — | W4 |
| W5 | 公開 API の修正（G5〜G8） | A〜C | CP1 |
| W6 | 状態機械（AC-3：D5・D8） | L1〜L6 | W5 |
| W7 | FINAL-1 → G9 → FINAL-2（一人で順に） | 1 lane | W6 |
| **CP2** | 全 gate・audit・KPI（Orchestrator） | — | W7 |
| W8 | 受入れの判定・書き戻し・PR 下書き（Orchestrator） | — | CP2 |

---

## 2. W0：準備（Orchestrator が自分で行う）

1. V2 設計書 DEC-01 は「`gpu` を既定 feature から外さない」（ユーザーの決定）。`crates/kivact/Cargo.toml` と各 consumer の `[features] default` は変えない。K6 は MS-5 の行で測る。
2. 統合 branch を最新にする：`git checkout refactor/kashiwade/slim-fast && git pull --ff-only && git submodule update --init`。
3. 計測用の worktree を作り、この機械で B0 を取る：

```bash
git worktree add --detach ../kv-b0 79bbd409749e49dfdc581c7c6497eac1ac52ca92
git -C ../kv-b0 submodule update --init
cd ../kv-b0
CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/b0 dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B0-here
cd -
cp ../kv-b0/dev/bench/results/B0-here.md dev/bench/results/B0-here.md
git add dev/bench/results/B0-here.md && git commit -m "V2-W0: KPI baseline B0-here (79bbd409 on this machine)"
```

   - 何も描画していない状態で、画面を点けたまま測る。`INVALID:` で終わったら原因（build の失敗など）を調べ、`B0-here` の commit を変えずに測り直す。直せなければ人間に報告して止まる。
   - `kv_heap` の行は B0 では site 名が空になる（MS-2 の前）。診断値なので構わない。
   - 続けて、統合 branch の今の HEAD を `B1-start` として測る（V2 の前の値。判定には使わないが、CP1 で変化を見るため）：

```bash
git worktree add --detach ../kv-start HEAD
git -C ../kv-start submodule update --init
cd ../kv-start
CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/b0 dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B1-start
cd -
cp ../kv-start/dev/bench/results/B1-start.md dev/bench/results/B1-start.md
git add dev/bench/results/B1-start.md && git commit -m "V2-W0: KPI before V2 (B1-start)"
git worktree remove ../kv-start
```

   - `../kv-b0` の worktree は W8 まで残す（未達の行の B0 の再計測に使う）。
4. ディスクの空きを確かめ（`df -h`）、1.1 節の target の置き方を決めて、OPEN.md の「V2 環境」節に書く。
5. `python -m pip install --user fonttools` を実行する（S9 が使う。DEC-03）。失敗したら S9 を W2 で行わず、`OPEN.md` に差戻しとして書く。
6. push する。

---

## 3. W1（6 lane 同時）

各行の unit を、lane の中で上から順に行う。「確認」は付属書の unit の行の「完了時の確認」。

| lane | unit | 付属書と節 | 付属書の unit | 本書の追加の指示 |
|---|---|---|---|---|
| A | F1 | D1 3.1（S-1）、5.1 | P1-U1 | — |
| B | F3 | D1 3.4（S-4） | P1-U3 | — |
| B | M2 | D3 3.3（M2-3） | U-M2 | — |
| B | F4 | D1 3.5（S-5） | P1-U4 | ink の不変条件（drawing・geometry・scroll・可視性の変化で自分と祖先の ink を捨てる）を変えない（V2 設計書 5.3、AH3 が前提にする） |
| C | F2 | D1 3.2（S-2）、3.3（S-3） | P1-U2 | — |
| C | F5 | D1 3.6（S-6） | P1-U5 | 所有ファイルは `crates/arvane-compose/src/plan/damage.rs` と `crates/arvane-compose/src/cache.rs` |
| C | F6 | D1 3.7（S-7） | P1-U6 | 所有ファイルに `crates/arvane-shell/src/window.rs`（`ShellWindow::present_with` の doc の 1 文だけ）を足す |
| D | M1 | D3 3.1（M2-1）、3.2（M2-2） | U-M1 | — |
| D | M7 | D3 3.7（M2-7。U23 / M-R5 の確定） | U-M7 | — |
| D | M6 | D3 3.6（M2-6） | U-M6 | — |
| E | M11 | D3 3.14（M2-14） | U-M11 | 所有ファイルは `crates/kivact-runtime/src/reactive/{links.rs（新）,data.rs,graph.rs,frame.rs,selector.rs,arena.rs}` と `crates/kivact-runtime/src/reactive.rs` |
| E | T8 | D7 3.7（M3-7） | U-T8 | — |
| E | M10 | D3 3.12（M2-12）、3.11 の button の部分 | U-M10 | 確認の consumer の試験は `dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-ui-soft --test consumer -- button_busy` で実行する |
| E | M12 | D3 3.15（M2-15） | U-M12 | 付属書に加えて `dev/bench/run.sh` に次を足す（V2 設計書 MS-5）：build の並びに `build kv_startup_nogpu kv_startup -p kivact-bench --no-default-features --features complex-scripts --bin kv_startup` と `build catalog_nogpu kivact-ui-basic-catalog -p kivact-ui-basic-catalog --no-default-features --features complex-scripts`、K6 の節に `log "- K6 counter_nogpu $(wc -c < "$kpi/kv_startup_nogpu$exe")"` と `log "- K6 basic-catalog_nogpu $(wc -c < "$kpi/catalog_nogpu$exe")"`（既存の K6 の行の並びに合わせる）。`dev/bench/README.md` に MS-3 の 1 行（K2 を単独で測るときは build 直後の最初の起動を捨てる）。確認は付属書の行の代わりに次を行う（`H=<lane の target>-heap`）：(1) `CARGO_TARGET_DIR=$H CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release -p kivact-bench --bin kv_heap`、(2) `mkdir -p $H/kpi && cp $H/release/kv_heap.exe $H/release/kv_heap.pdb $H/kpi/`、(3) `MODE=button ROWS=10 SETTLE_SECS=1 dev/tools/window-lock.sh --timeout 600 -- $H/kpi/kv_heap.exe`、(4) 出力の最初の `H site` の行が `allocs ` の後に空でない名前（`kivact` か `arvane` で始まる）を持つこと。終わったら `rm -rf $H` |
| F | S1 | D4 3 章「D1」 | P4-U1 | — |
| F | S4 | D4 3 章「D3」 | P4-U4 | — |
| F | S2 | D4 3 章「D2」（Arvane 側） | P4-U2 | — |
| F | G1 | D6 3.1、3.2 | P6-1 | `Windows.gpu`・`Driver.gpu` を消す（AC-3 の OPEN 2 件）。S2 が足した `forget_frame_state` は残す |

W1 の後に compile が失敗してよい crate：なし。

**W1 は完了（2026-11-10、`2a3b06b8`）**。M6 は W2 lane D へ（D10 で設計し直し）、S4 は W2 lane C の S45 へ（`3c284b0b` を cherry-pick）、G2 は W1 lane F に前倒し済み。W1 の差戻し（F5・M7・T8 の state-fields、F4 の dead_code、diagnostics の E2E）は W2 の F5-fix・M7-fix・T8-fix・F4-fix・T1 が閉じる。

---

## 4. W2

**V2.1**：W2〜W4 の各 unit は、まず修正付属書（D9〜D14）のその unit の節（`## <unit ID>` の見出し）を読み、そこに書かれた手順・コード・所有ファイル・確認をすべて行う。修正付属書の節が「元の付属書の節のとおり」と書く部分だけ、元の付属書（D1〜D8）を読む。下の表の「本書の追加の指示」は修正付属書に加えて守る。

| lane | unit の順 | 修正付属書の節（元の付属書） | 本書の追加の指示 |
|---|---|---|---|
| A | AH1 | D9「AH1」（D2 3.1〜3.5、3.8） | 所有ファイルに `crates/kivact-engine/src/a11y.rs`（doc）と `model/interact/geometry.rs` の `set_scroll` の 1 行を足す（D9）。`build_identity_metadata` の doc comment に PERF-1 の例外の理由を 2 文で書く（DEC-06）。F1 が足した索引の書き手を消さない |
| B | F5-fix → F4-fix → F7 | F5-fix：D9「F5-fix」。F4-fix：本書の下。F7：D9「F7」（D1 5.2） | **F4-fix**（DEC-25）：`crates/kivact-engine/src/paint/state.rs` の `struct Indexed` から field `size` と `content` を削除し、`crates/kivact-engine/src/paint/list/index.rs` の `Indexed::new` の `size: layout.size,` と `content: layout.content_size,` の 2 行を削除する。`crates/kivact-engine/src/paint/list/self_check.rs` の `same_index` の tuple の比較から `before.size,`・`before.content,`・`after.size,`・`after.content,` の 4 行を削除する（この 2 つの field は debug の self_check の比較にだけ使われ、製品の経路は読まない。V2.2 の確認）。所有ファイルはこの 3 つだけ。確認には `cargo check -p kivact-engine --all-targets --features testing` も加える。確認：`cargo clippy -p kivact-engine --all-targets -- -D warnings`（feature なし）と `cargo clippy -p kivact-engine --all-targets --all-features -- -D warnings` |
| C | S45 → T4 → S8 → S9 | S45：D11「S45」（D4 3 章「D3」）。T4：D11 の T4 の順序と D7 3.3。S8：D11「S8」。S9：D11「S9」 | **S45** は `git cherry-pick --no-commit 3c284b0b`（W1 の S4 の変更を commit せずに取り込む。D11 の「`git cherry-pick 3c284b0b`」はこれに読み替える。S4 だけの commit は engine の compile を壊すため作らない）から始め、D11 のとおり S5 の 4 ファイル・試験 2 本・doc を足して一つの commit にする（`S45: FontSource shares font bytes (S4+S5)`）。W0 の手順 5 の `fonttools` が無ければ S9 を行わない **V2.2**：S8 の profile 節の一覧は D15「Q6：3.2」の確定の 20 節（AccessKit・naga・ICU・kivact-i18n・svgtypes を `z` にしない）に置き換える（D11「S8」・D4「D7」の一覧は使わない）。 |
| D | M7-fix → M6 → M8 → T1 → G3 | D10「M7-fix」「M6」「M8」「T1」、D11「G3」 | M6 は D10 の設計（遅延の draw effect）で行い、D3 3.6 の InStyle の経路は使わない（DEC-11 の取消し）。M8 の手順 6 は行わない（D7 M3-5 が T6 で置き換える）。T1 は E2E `diagnostics::why_follows_the_prop_to_the_write_through_a_memo` の期待値を D10 の文に変える（DEC-26） **V2.2**：G3 は D15「Q3：R04」「Q3：3.3」の手順で D11「G3」の registry と viewport の手順を置き換える（core は需要を記録しない。`asked` を作らない。viewport は `has_adapter()`）。所有に D15 Q3 の表のファイル（`docs/for-kivact-developer/architecture.md` の 1 段落、`adapter()` の doc）を足す。 |
| E | T8-fix → S7 | D10「T8-fix」、D11「S7」 | — |
| F | M9 | D10「M9」（D3 3.10、3.11 の core の部分） | — |

---

## 5. W3

| lane | unit の順 | 修正付属書の節（元の付属書） | 本書の追加の指示 |
|---|---|---|---|
| A | M3 → T3 | D10「M3」（D3 3.4・3.5、D7 3.0・3.8 を統合）、D10「T3」（D7 3.2） | **M3 は T2・T9 を含む**（D10）。`Recent<T>` は最初から `kivact_types::cell` に置く。`kivact-engine` の外で compile error が出たら直さずに STOP。AH1 が書いた `host/write/typed.rs` の `mut = ..PAYLOAD_ONLY..` と `model/aggregate.rs` の `repaint`・`a11y_scrolled` を戻さない（D9「全体」の申し送り 2）。D3 の U-M4（M2-5b）は行わない |
| B | AH3 → T7 → AH2 | D9「AH3」「T7」「AH2」 | AH3 の所有に `crates/kivact-engine/src/paint/list/ink.rs`（cache の判定の条件 1 つ）を足す。`HitInk` を model に定義し paint が実装する（DEC-04）。AH2 は `Stats::a11y_enumerated` を公開する（DEC-05） **V2.2**：AH3 は D15「Q1：R01」「Q1：R02」「Q1：追加：culling」の手順を行う（D9「AH3」の手順 3・5・6 を置き換え、8・9 を足す。ink の `Drawn { rect, beyond }`、hit の `HitReach`、参照経路の `HitWalk`、walk・index の culling）。所有に `paint/state.rs`・`paint/list/self_check.rs`・`model/drawables/hit.rs`・`paint/list/walk.rs`・`paint/list/index.rs` を足す。 |
| D | T6 → S6 | D10「T6」（D7 3.5）、D11「S6」 | T6 は `CoreNode.at`・`handlers` の直前にある HEAD の marker 行をそのまま残す（D7 のコード片は省略している。D14「W6 の後に残る error と直し」の X3 の表）。S6 は D4「D5」だけ（DEC-01）で、所有に `docs/for-kivact-user/{en,jp}/README.md` の bench の 1 文を足す（D11） |
| F | G4 → S3 | D11「G4」（D6 3.6）、D9「S3」 | G4 の所有に `dev/tests/kivact-public-api/tests/e2e/gpu_on_demand_shared.rs`（新）を足す（D11）。S3 は `control` の `next` に `release_at` の要素を足す（後の EN-2 が残す） **V2.2**：G4 は D15「Q3：R04」の E2E 2 本（新しいファイル `gpu_on_demand_windows.rs`、`main.rs` に `mod` 1 行）も足す。 |

---

## 6. W4

| lane | unit の順 | 修正付属書の節（元の付属書） | 本書の追加の指示 |
|---|---|---|---|
| A | AH4 | D9「AH4」（D2 5.3） | `a11y_enumerated` が増える、または self_check の不一致が出たら STOP（修正を試みない） **V2.2**：D15「Q1：R01」「Q1：R02」「Q1：追加：culling」の E2E 3 本（`hit_reaches_projected_boxes_outside_their_parents`、`hit_reaches_a_projected_box_inside_a_drawable`、`a_projection_into_the_window_is_drawn_from_a_box_outside_it`）と import の変更も足す。 |
| C | T5 | D10「T5」（D7 3.4・3.9・3.10） | D3 の U-M5（M2-13）は行わない。kivact-layout に単体試験を足さない（DEC-27） |

## 7. CP1（Orchestrator）

### 7.1 gate

```bash
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1
```

- `cargo xtask check --no-write` が state-fields の audit を書き換えた場合は `git checkout -- dev/xtask/state-audit.md` で戻す（AC-3 の確認は CP2）。
- GPU の E2E（`gpu_on_demand.rs` ほか）は、GPU のある機械で次も実行する：`KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e gpu_on_demand`。`ARVANE_RENDERER=cpu` を付けた手動の確認は D6 3.6 の末尾のコマンドのとおり。
- 失敗した試験は、その試験の file を変えた unit（`git log` で探す）の lane の fix として 1.3 節の fix の手順で直す。2 回直しても通らなければ差戻し（0.5）。
- `cargo xtask check` の error（行数の上限、state-fields 等）も同じく、error の file を変えた unit の lane の fix として直す。

### 7.2 KPI

**B0 の測り直し（V2.2、R09）**：W0 は B0 を測り直さず V1 の `B0.md` を `B0-here.md` として使った。CP1 の KPI の前に、同じ機械・同じ条件で B0 を一度だけ測り直す：W0 の手順 3 のとおり `../kv-b0`（commit `79bbd409749e49dfdc581c7c6497eac1ac52ca92`）の worktree を作り、`CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/b0 dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B0-here2` を実行して `dev/bench/results/B0-here2.md` を commit する。**以後、CP1・CP2・W8 の判定の B0 はすべて `B0-here2`**。`B0-here` は参考で、判定に使わない。RESULT には、使った B0 のファイル名・commit・機械・測定の日時を書く。

```bash
CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/kpi dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B1-cp1
```

`dev/bench/results/B0-here2.md` に対する倍率を、V2 設計書 2.3 節の各行について `dev/bench/results/CP1-V2.md` に表で書く（KPI、指標、B0-here、B1-cp1、倍率、必達、判定）。K3 のアイドルは B1-cp1 の値そのもの。commit して push する。

### 7.3 未達の行がある場合

1. 同じ条件で 2 回測り直す（`run.sh B1-cp1-r2`、`B1-cp1-r3`）。3 回中 2 回満たせば満たしたとする（MS-4）。
2. それでも未達の行は、`OPEN.md` の「V2 差戻し」に、行・3 回の値・関係する付属書の節を書く。目標・測定条件を変えない。
3. 診断の補助として、引継ぎ `20261110_1145_CLAUDE_CODE.md` 6.2 節の計測 patch（prof の span）を別の detached worktree に当てて `kv_stress`／`kv_latency` を測り、span の値を `OPEN.md` に貼ってよい。製品のコードに prof を入れない。
4. W5 以降へ進む（状態と API の作業は KPI と独立）。最終の判定は W8。

---

## 8. W5（3 lane 同時）

各 unit は修正付属書 D12 のその unit の節（`## G5` 等）を読み、そこに書かれた手順・コード・所有ファイル・確認をすべて行う（D6 9 章を上書きする）。

| lane | unit の順 | 修正付属書の節（元の付属書） | 本書の追加の指示 |
|---|---|---|---|
| A | G5 | D12「G5」（D6 9 章 C-1〜C-4。C-6 は G6 から G5 に移した） | **V2.2**：D15「Q5：3.1」の回帰試験（`component_macro.rs` の E2E 2 本、`dev/tests/kivact-public-api/src/lib.rs` の doctest `compile_fail` 5 本）も足す。所有に `dev/tests/kivact-public-api/src/lib.rs` を足す。 |
| B | G7 | D12「G7」（D6 9 章 C-12） | **V2.2**：api-shape の判定は D15「Q5：R08」のとおり「`Fn` の入力が 0 個」にし（`impl Fn()`・`impl Fn() -> ()`・`F: Fn()`・where 句を同じに扱う）、fixture の試験を足す。 |
| C | G6 → G8 | D12「G6」（C-6〜C-11・C-15）、D12「G8」（C-5、DEC-17） | G6 の所有ファイルと C-15 の波及の範囲は D12 の表が正（本書の旧版の G6 の行は使わない）。G8 は facade に `crates/kivact/src/launch.rs` を新しく作る（DEC-17） **V2.2**：G8 に D15「Q3：R07」の手順 9（arvane-compose の feature `gpu` に kivact-gpu-share を入れ、adapter の検索の cfg を外し、kivact-types の doc を直す）を足す。所有に `crates/arvane-compose/Cargo.toml`・`crates/arvane-compose/src/gpu/context.rs`・`crates/kivact-types/src/gpu.rs` を足す。 |

---

## 9. W6（状態機械、6 lane 同時）

D5 の 6 章の lane と D8 の 6 章の追加を合わせたもの。lane の中は上から順に行う。D8 の「P5 の <unit> の後」の依存はこの順で満たされる。

| lane | unit の順 |
|---|---|
| L1 | XT-1 → P8-POP → LY → ST・ED → P8-L1 → P8-XT |
| L2 | NA → P8-NA → AS → EN-1 → EN-2 → P8-EN → KT → P8-KT |
| L3 | RT → P8-RT → CO → P8-CO → P8-ARV → P8-GPU |
| L4 | IN-1 → IN-2 → IN-3 → P8-IN |
| L5 | UI-1 → UI-2 → UI-3 → P8-CK → P8-UI5 |
| L6 | UI-4 → SO → CR → EX → DT → P8-UI6 → P8-SOFT → P8-KITS |

- **V2.1**：各 unit は、まず修正付属書のその unit の節を読む：L1〜L3 は D13、L4〜L6 は D14（`## <unit ID>` の見出し）。そこに書かれた手順・コード・marker の文言・所有ファイル・確認をすべて行い、「元の付属書のとおり」とある部分だけ D5・D8 を読む。D14 の先頭の「R6 共通の確定事項」（命名の規則 DEC-28 を含む）は L4〜L6 の全 unit に適用する。所有ファイルの組み直し（IN-3 に `calendar/data.rs`、CR に `region_parts.rs`、UI-1 と P8-UI5 の入替え、UI-3 から外すファイル、EX から外す `jobs.rs`）は D14「1. wave・lane・所有ファイルの組み直し」が正。lane の中の順は下の表のまま。
- **V2.2（D15）**：**IN-3** は D15「Q2：3.4」の手順（新しい `part_slot.rs` の `attach_part_state` で 21 箇所を置き換え、content の Part の状態を owner の終わりで空にする。E2E 3 本）も行い、所有に D15 の表のファイルを足す（D14 IN-1 の「W6 では変えない」は取り消し）。**P8-UI5** は D15「Q2：R03」の手順（DataTable の内部を `SortSetting`・`PageSetting`・`PagingPlan` の enum にする。公開の builder の書き方は変えない。E2E 6 本）も行い、所有に `crates/kivact-ui/src/data_table/builder.rs` と `dev/tests/kivact-public-api/tests/e2e/ui_data_table.rs` を足す（D14 の E-9 の取消しと、DataTable の依存を marker の文言だけで閉じる指示は取り消す）。**EN-2** の `report_app_motion` は `finish_motion` の中で借用の衝突（E0502）になるので、D15「Q4：全体」の申し送りのとおり `Rc::clone(&self.app)` の 2 行で直す。**CO** の `renew` の書換えは clippy の `collapsible_if` に従って let chain で書く。
- **CO** は G3 の後のコードで、`SharedImage.current` への全ての書込み（G3 が足したものを含む）を `replace_current` 経由にする（D14 X4 の条件）。
- **P8-EN** の `Mutation.local_text` の行は、AH1 が改名した `Mutation.payload_only` の直前に置く（文言は D8 のまま。D9「全体」の申し送り 1、D13「P8-EN」）。
- **EN-2** は S3 が `control` に足した `release_at` の要素を残す。
- **unit の内容の決め方**（修正付属書の節に加えて、元の付属書を読む範囲。雛形の `<節>` に書く）：D5 の unit は「D5 6 章の行の括弧内の ID が指す節」と「D5 の本文・付録のうち、その行の所有ファイルを変える全ての指示」。D8 の unit は「D8 6 章の行」と「D8 3 章・付録 A-G*（W 番号）・付録 B（unit 列がその unit の行）」。括弧内の ID の場所：

| ID | 場所 |
|---|---|
| A1〜A10、「A 既存欠陥 D1・D2」 | D5 付録 P5-A（A1〜A10 の各節、3 既存欠陥） |
| B1〜B10、「B D2」 | D5 付録 P5-B（1 事実・3 設計の B 番号、既存欠陥の D2） |
| C1〜C5 | D5 付録 P5-C |
| F-1〜F-9、「D F-x」、「initial_render」「SharedImage」「Queue の doc」 | D5 付録 P5-D（F 番号の節、A-CO・A-RT の表） |
| E-1〜E-11、E-U1〜E-U6 | D5 付録 P5-E（E 番号の節、末尾の unit 表の E-U 行） |
| F-T1〜F-T6、F-U4〜F-U7、「F の …」「F 2 章の …」 | D5 付録 P5-F（F-T の節、2 章、末尾の表の F-U 行と「E-U に追加」の行） |
| X1〜X6 | D5 本文 3.4 |
| 「3.5 …」 | D5 本文 3.5（`projection:` marker の最終一覧と追加の確定事項） |

- **EN-2** の F-7（`Host.next_wake`）は型（付録 P5-D F-7 の `HostPhase` の variant に `next_wake` を持たせる形）で閉じる。「文言だけ」の代案は採らない。
- **UI-3** は付録 P5-F の F-T4（`SwitchFacts`）を行わない（DEC-15、V2 設計書 5.2）。その分は P8-CK が行う。
- **P8-GPU** は G1・G3・G8 の後のコードに書く。G8 で `GpuPreference` 一式が `kivact::launch` に移っているので、import の path は移った先を使う。G1 が消した field の行は不要（D8 の P8-GPU の行のとおり）。
- **P8-SOFT** の `TimestampEntry.copyable` は field を残し、D8 の付録の `option:` の文言にする（DEC-22）。
- 状態の unit の対象が W1〜W4 の性能の unit で変わっていた場合は、V2 設計書 5.4 節と本書 0.4 に従う。
- lane の名前 L1〜L6 は worktree の名前でも使う（`../kv-wt/W6-L1` など）。target は 1.1 節の A〜F を L1〜L6 に読み替える。

---

## 10. W7（1 lane、一人で順に）

統合 branch から worktree `../kv-wt/W7` を作り、次を順に行う（それぞれ別の subagent。前の unit の commit の後に次を起動する）。各 unit は修正付属書のその unit の節を読み、そこに書かれた手順・コード・所有ファイル・確認をすべて行う。

| unit | 修正付属書の節（元の付属書） | 本書の追加の指示 |
|---|---|---|
| FINAL-1 | D12「FINAL-1」（D5 6 章 FINAL-1、付録 P5-D F-1） | 読み手は D12 の全数の表（43 ファイル・92 行と docs の 4 行）と書換え規則が正。`cargo check --workspace --all-targets` は確認に使う。`Availability` の `#[derive(..)]` から `Default` を外し、手書きの `impl Default` だけを残す（D14「3. 担当外の unit への申し送り」2）。marker の文言は D12 のとおり |
| G9 | D12「G9」（D6 9 章 C-13、8 章 2） | D12 の G9 の改名表を上から順に行う（本書の旧版の改名表は使わない）。content を mount する method は `while_open(factory)`（DEC-18）。旧名は残さない。この unit に限り、所有ファイルは compile error と D12 が示す全ての file（docs・doctest を含む） **V2.2**：開閉の命令は D15「Q5：R06」の規約 SH-1〜5 と全数表（C1〜C24、G9-C 表）に従う（D12 G9 の順 2〜5 と「変えないもの」の close の文を置き換える。`show`・`hide` は冪等、反転は `toggle`、`dismiss` も揃える）。`ComboboxApi::toggle` の SetupOnly の panic の修正（D15 のコード A）と E2E `interaction_show_hide.rs` の 4 本も行う。D12 の「`cargo doc` の警告 0」の確認は D15 の置き換えた判定にする。 |
| FINAL-2 | D14「FINAL-2」（D5 3.4・3.7・6 章） | `claims.rs` と試験は D14 の付録 A・B の全文を使う。確認は D14 の FINAL-2 の節の「4. 確認」の表（STOP にしない）。確認の後 `git checkout -- dev/xtask/state-audit.md` で戻してから commit する（audit の commit は CP2 の 11.2） **V2.2**：`claims.rs` と試験は D15「Q4：付録 A」「Q4：付録 B」の全文を使う（D14 付録 A・B は使わない）。D15「Q4：R05」の手順 0（F1〜F6：projection の field の可視性を絞る、`EditorNode::report_session`、facts の `Node` の accessor と `node_type`、`HostWindow.reported_adapter`、previewed の marker）を先に行う。検査器が「未検証」と報告するものは error で、成功扱いにしない。 |

W7 の後、1.3 節の merge の手順で merge する（lane は 1 つ）。

---

## 11. CP2（Orchestrator）

### 11.1 gate

```bash
cargo fmt --all -- --check
cargo xtask check                 # --no-write を付けない：architecture/ の図を生成する
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-docs --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=required dev/tools/window-lock.sh --timeout 900 -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e gpu_on_demand
```

feature の組合せ（package を一つずつ。workspace の `--no-default-features` は GPU なしの確認にならないので使わない）：

```bash
cargo check -p kivact --no-default-features
cargo check -p kivact --no-default-features --features gpu
cargo check -p kivact --features gpu-interop
cargo tree -p kivact --no-default-features -e features -i arvane-compose | grep -E '"gpu"' && echo "NG: gpu leaked" || echo ok
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-public-api-tests --no-default-features
```


- WSL があれば、WSL の中（`wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}"`）で上の clippy と `cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features` を同じ形で実行する。無ければ報告に「Linux 未確認」と書く。
- `architecture/` の変更を commit する。

### 11.2 状態の監査（AC-3）

D5 3.7 の「checkpoint（全体、OPEN 0 の確認）」の 4 行をそのまま実行する。`exit=0` かつ `OPEN` 0 件であること。生成された `dev/xtask/state-audit.md` を commit する。

### 11.3 KPI

B0 は CP1 で測った `B0-here2` を使う（7.2）。CP1 で `B0-here2` を測っていない場合は、ここで 7.2 の手順で測る。

```bash
CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/kpi dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B1-here
```

未達の行は 7.3 の 1 と同じく 2 回測り直す（`B1-here-r2`、`B1-here-r3`）。

---

## 12. W8（Orchestrator）

### 12.1 受入れの判定

1. `dev/bench/results/RESULT-V2.md` に、`B0-here2` と `B1-here`（と r2・r3）から、V2 設計書 2.3 節の各行の倍率と判定を表で書く。
2. AC-1〜AC-4 を確かめ、`RESULT-V2.md` に AC ごとの結果を書く：
   - AC-1：CP2 の 11.1 がすべて成功。
   - AC-2：全ての必達行を満たす（MS-4 の 3 回中 2 回を含む）。
   - AC-3：(a) 機械の検査：11.2 で OPEN 0 件、`OPEN.md` に状態の差戻しが無い。(b) 人間の確認：`dev/bench/results/AC3_REVIEW.md` に人間が確認の結果を書いていること。Orchestrator は (a) が満たされたら `AC3_REVIEW.md` を次の内容で作り（確認の欄は空のまま）、AC-3 を「**人間の確認待ち**」と書く：`state-audit.md` の V2 の前（`B1-start` の時点の commit）からの差分の要約、閉じ方ごとの件数（型・導出・projection・independent・cache 等）、全ての `projection:` の行（field・名指す関数）、全ての `independent:` の行（field・理由）、FINAL-2 の検査器が「未検証」として報告した項目の一覧（無ければ「なし」）、V2 で新しく作った索引・cache・需要の状態の一覧（V2 設計書 6.1 節の表）。Orchestrator は確認の欄を書かない（DEC-16）。
   - AC-4：`OPEN.md` の「V2 差戻し」に AC-1〜AC-3 と公開 API に影響する項目が 0 件。U23 と U56 の項目に「V2 の M7・M10、G1〜G4 で解消」と書く。
3. どれかを満たさない、または AC-3 が「人間の確認待ち」なら、ここで止める：`RESULT-V2.md`・`OPEN.md`・`AC3_REVIEW.md` を commit・push し、満たさない項目と確認を待つ項目を人間に報告する。12.2・12.3 は行わない。人間が `AC3_REVIEW.md` の確認の欄に結果を書いた（または作業の依頼文で確認済みと伝えた）後にだけ、12.1 をやり直して 12.2 に進む。

### 12.2 書き戻しと後片付け（AC をすべて満たしたときだけ）

1. `docs/for-kivact-developer/architecture.md`：旧作業指示書 W7-2 の手順 1 の内容に加え、V2 設計書 8 章の規則（PERF-1 の補足、PERF-4、PERF-5、MEM-8、MEM-9）を足す。DEP の feature の表で `kivact` の default を `gpu, complex-scripts` にする（版 1.1 のまま、DEC-01）。API-3 を DEC-19 のとおり書き直す。
2. （V2.1）`dev/bench/results/OPEN.md` の W1 の付記（F4 の `invalidate` の doc、M2 の EVT-1・M-R4 の文）を直す。DEC-11 は取り消されたので書き戻さない。
3. `docs/for-kivact-developer/decisions.md`：旧作業指示書 W7-2 の手順 2 の内容（版 1.1 の 8.1・8.2）に加え、V2 設計書 8 章の SLIM-11〜SLIM-16 と、4 章の DEC-01（外さない理由）・DEC-14・DEC-15・DEC-17〜DEC-20・DEC-24・DEC-28 を「実装の決定」の表の末尾に足す。
4. `dev/bench/results/OPEN.md` の「後続」節に V2 設計書 7 章の表を写す。
5. `SLIM_FAST_DESIGN.md`、`SLIM_FAST_WORK_ORDER.md`、`20261109_2107_CODEX_REVIEW.md`、`20261110_1145_CLAUDE_CODE.md`、`SLIM_FAST_V2_DESIGN.md`、`SLIM_FAST_V2_WORK_ORDER.md`、`slim-fast-v2/` を `git rm` する。`dev/bench/results/` と `dev/xtask/state-audit.md` は残す。
6. commit し、push する。

### 12.3 PR の説明の下書き（AC をすべて満たしたときだけ）

`dev/bench/results/PR_DRAFT.md` に次を書いて commit し、push する（PR は作らない）：

- 概要（V2 設計書 2.4 と 6 章の原因と、直した内容を 15 行以内）
- KPI の表（`RESULT-V2.md` の表。K6 は MS-5 の定義であることを注記する）
- 公開 API の変更：版 1.1 の改名（付録 B）、ARG-1〜6、`#[component]`、`Binding::update`、`GpuStart`、`complex-scripts`、V2 の変更（`GpuPreference` 一式が `kivact::launch` に移動、`RenderOptions::gpu_on_demand`・`Screen::renderer`・`Screen::wait_for_gpu`、`Stats::a11y_enumerated`、recipe state の型と accessor の統一（D14「2. 作業指示書への反映」の 12.3 の文をそのまま使う）、G9 の改名と `while_open(factory)`、prelude から `provide_theme` を削除、D8 の公開 API の変更 3 件（`LayoutInset.own_outer`、`ShellImeText.password` の削除、`GpuAdapter` の `AdapterIds`））
- 確認の状況（CP2 の各コマンドの結果、Linux の確認の有無、AC-3 の人間の確認が残っていること）
