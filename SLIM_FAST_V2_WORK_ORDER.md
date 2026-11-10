# Kivact Slim-Fast V2 作業指示書

版 2.0（2026-11-10）。設計は [SLIM_FAST_V2_DESIGN.md](SLIM_FAST_V2_DESIGN.md)（以下「V2 設計書」）と付属書 `slim-fast-v2/D1〜D8`（以下「付属書」）。本書は手順だけを書く。設計の判断は V2 設計書と付属書が持つ。旧作業指示書 `SLIM_FAST_WORK_ORDER.md` は使わない。

目標：V2 設計書 1 章の AC-1〜AC-4 をすべて満たす。

---

## 0. 読み方と規則

### 0.1 誰が何をするか

| 役割 | 担当 | すること |
|---|---|---|
| Orchestrator | 本書を受け取った AI（親） | wave を順に進める。lane ごとに worktree を作り、unit ごとに subagent を起動する。結果を merge する。checkpoint（CP）の gate と計測を実行する。subagent の `STOP:` に 0.5 節の表で答える。**設計の判断をしない** |
| Implementer | subagent（**同時に最大 6 体**。モデルは **Haiku 5.5、effort high**） | 割り当てられた unit を、指定の worktree の中で、付属書の手順どおりに実装する。完了条件のコマンドを実行し、結果を報告する |

- Orchestrator は Agent tool を `subagent_type: "general-purpose"`、`model: "haiku"`、`effort: "high"` で呼ぶ。同時に動かすのは最大 6 体（lane A〜F に 1 体ずつ）。subagent に、さらに subagent を作らせない（雛形の文に書いてある）。
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

- **付属書と節**：読む付属書のファイルと節。unit の手順の本体はそこにある。
- **付属書の unit**：付属書の「6 章 unit 分割案」の行の名前。その行の「所有ファイル」「完了時の確認」を使う。
- **本書の追加の指示**：付属書を上書きする指示（V2 設計書 4・5 章の結果）。付属書と食い違ったら本書が正。

### 0.4 停止して報告（STOP）

次の場合は、その項目の作業を止め、他の項目を続け、最後の報告に `STOP:` で始まる行を書く。

- 付属書の手順が実際のコードと合わない（関数・型・field が無い、型が違う、関数名で探しても見つからない）。
- 前の unit（性能の unit を含む）が、付属書が対象とする field・関数を消した・改名した・型を変えた（V2 設計書 5.4 節）。付属書がその場合の扱いを書いていればそれに従う。
- 手順を行うと所有ファイルの外を変える必要がある（0.2 の 12 の追従を除く）。
- 同じ compile error を 3 回直しても消えない。
- 既存の E2E の期待値を変えないと通らないと分かった。
- debug の `self_check`（a11y、hit、paint、layout）の不一致が出た。修正を試みずに、失敗した試験の名前、直前の操作、self_check のメッセージを書く。
- `state-fields` の lint が、付属書に無い field を報告した。

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
  2. その行が指す付属書 <worktree>/slim-fast-v2/<付属書>.md の <節>（unit の行「<付属書の unit>」を含む 6 章も）
  3. <worktree>/SLIM_FAST_V2_DESIGN.md の 4 章（決定）と 5 章（付属書間の調整）のうち、この unit に関係する行
付属書の手順を順に実行してください。行番号はずれています。関数名・型名・field 名で場所を探してください。
本書の「追加の指示」が付属書と食い違うときは本書に従ってください。
設計の判断が必要になったら作業を止め、0.4 節の書式で STOP: 行を書いてください。
完了時の確認のコマンド（付属書の unit の行）をすべて実行してください。続けて、変更した各 crate について cargo xtask check --no-write --crate <crate> を実行し（--report-audit は付けない）、error を直してください。最後に cargo fmt -p <crate> を実行してください。
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

1. **ユーザー確認の記録**：V2 設計書 DEC-01（`gpu` を facade の既定 feature から外す）について、ユーザーの指示がこの作業の依頼文や `dev/bench/results/OPEN.md` に「不承認」と書かれていれば、S6 の D4 の部分（`crates/kivact/Cargo.toml` の default と各 consumer の default の変更、README の表）を行わないと `OPEN.md` に記録する。書かれていなければ DEC-01 のとおり行う。
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
| E | M12 | D3 3.15（M2-15） | U-M12 | 確認は付属書の行の代わりに次を行う（`H=<lane の target>-heap`）：(1) `CARGO_TARGET_DIR=$H CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release -p kivact-bench --bin kv_heap`、(2) `mkdir -p $H/kpi && cp $H/release/kv_heap.exe $H/release/kv_heap.pdb $H/kpi/`、(3) `MODE=button ROWS=10 SETTLE_SECS=1 dev/tools/window-lock.sh --timeout 600 -- $H/kpi/kv_heap.exe`、(4) 出力の最初の `H site` の行が `allocs ` の後に空でない名前（`kivact` か `arvane` で始まる）を持つこと。終わったら `rm -rf $H` |
| F | S1 | D4 3 章「D1」 | P4-U1 | — |
| F | S4 | D4 3 章「D3」 | P4-U4 | — |
| F | S2 | D4 3 章「D2」（Arvane 側） | P4-U2 | — |
| F | G1 | D6 3.1、3.2 | P6-1 | `Windows.gpu`・`Driver.gpu` を消す（AC-3 の OPEN 2 件）。S2 が足した `forget_frame_state` は残す |

W1 の後に compile が失敗してよい crate：なし。

---

## 4. W2

| lane | unit | 付属書と節 | 付属書の unit | 本書の追加の指示 |
|---|---|---|---|---|
| A | AH1 | D2 3.1〜3.5、3.8 | P2-U1a | `build_identity_metadata`（Metadata 経路）の doc comment に PERF-1 の例外の理由を 2 文で書く：構造の変化の正しさを全比較で守るため全走査する、K5 の更新・pointer・wheel の場面には現れない（DEC-06）。F1 が足した索引の書き手を消さない |
| B | S5 | D4 3 章「D3」（登録・SVG） | P4-U5 | 付属書の依存「U3 の後」は順序の目安。本書の wave の順が正（所有ファイルは重ならない） |
| B | F7 | D1 5.2 | P1-U7 | E2E の実行は確認のとおり window-lock を通す |
| C | T4 | D7 3.3（M3-3） | U-T4 | S4 の `FontSource` の後のコードに書く |
| C | S8 | D4 3 章「D7」 | P4-U8 | 付属書の依存「U6」は誤り。S8 が先で、W3 の S6 が後（`templates/basic/Cargo.toml` は S8 が profile 節、S6 が features 節を変える） |
| C | S9 | D4 3 章「D8」 | P4-U9 | W0 の手順 5 が失敗していたら行わない |
| D | M8 | D3 3.8（M2-8）、3.9（M2-9） | U-M8 | M2-8 の手順 6 は行わない（D7 M3-5 が置き換える、V2 設計書 5.1） |
| D | M9 | D3 3.10（M2-10）、3.11 の core の部分 | U-M9 | — |
| D | T1 | D7 3.1（M3-1） | U-T1 | — |
| D | G3 | D6 3.5 | P6-3 | — |
| E | S7 | D4 3 章「D6」 | P4-U7 | — |
| F | G2 | D6 3.3、3.4 | P6-2 | — |

---

## 5. W3

| lane | unit | 付属書と節 | 付属書の unit | 本書の追加の指示 |
|---|---|---|---|---|
| A | M3 | D3 3.4（M2-4）、3.5（M2-5） | U-M3 | `kivact-engine` の外で compile error が出たら直さずに STOP（同じ wave の lane D の T6 と衝突するため） |
| A | T2 | D7 3.0（M3-11） | U-T2 | M3 が作った `model/recent.rs` の `Recent<T>` を `kivact_types::cell` に移す |
| A | T9 | D7 3.8（M3-8） | U-T9 | — |
| A | T3 | D7 3.2（M3-2） | U-T3 | D3 の U-M4（M2-5b）は行わない（取消し） |
| B | AH3 | D2 3.6 | P2-U2 | `HitInk` を model に定義し paint が実装する（DEC-04） |
| B | T7 | D7 3.6（M3-6） | U-T7 | — |
| B | AH2 | D2 3.5 の診断 counter | P2-U1b | `Stats::a11y_enumerated` を公開する（DEC-05） |
| D | T6 | D7 3.5（M3-5） | U-T6 | — |
| D | S6 | D4 3 章「D4」「D5」 | P4-U6 | W0 の手順 1 で DEC-01 が不承認と記録されていれば、D4 の部分を行わず D5（`dev/bench/Cargo.toml`）だけを行う |
| F | G4 | D6 3.6 | P6-4 | — |
| F | S3 | D4 3 章「D2」（host 側）と 5.2 の E2E 3 本 | P4-U3 | — |

---

## 6. W4

| lane | unit | 付属書と節 | 付属書の unit | 本書の追加の指示 |
|---|---|---|---|---|
| A | AH4 | D2 5.3 | P2-U3 | `a11y_enumerated` が増える、または self_check の不一致が出たら STOP（修正を試みない） |
| C | T5 | D7 3.4（M3-4）、3.9（M3-9）、3.10（M3-10） | U-T5 | D3 の U-M5（M2-13）は行わない（取消し） |

---

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

```bash
CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/kpi dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B1-cp1
```

`dev/bench/results/B0-here.md` に対する倍率を、V2 設計書 2.3 節の各行について `dev/bench/results/CP1-V2.md` に表で書く（KPI、指標、B0-here、B1-cp1、倍率、必達、判定）。K3 のアイドルは B1-cp1 の値そのもの。commit して push する。

### 7.3 未達の行がある場合

1. 同じ条件で 2 回測り直す（`run.sh B1-cp1-r2`、`B1-cp1-r3`）。3 回中 2 回満たせば満たしたとする（MS-4）。
2. それでも未達の行は、`OPEN.md` の「V2 差戻し」に、行・3 回の値・関係する付属書の節を書く。目標・測定条件を変えない。
3. 診断の補助として、引継ぎ `20261110_1145_CLAUDE_CODE.md` 6.2 節の計測 patch（prof の span）を別の detached worktree に当てて `kv_stress`／`kv_latency` を測り、span の値を `OPEN.md` に貼ってよい。製品のコードに prof を入れない。
4. W5 以降へ進む（状態と API の作業は KPI と独立）。最終の判定は W8。

---

## 8. W5（3 lane 同時）

| lane | unit | 付属書と節 | 付属書の unit | 本書の追加の指示 |
|---|---|---|---|---|
| A | G5 | D6 9 章 C-1〜C-4 | P6-5 | — |
| B | G7 | D6 9 章 C-12 | P6-7 | — |
| C | G6 | D6 9 章 C-6〜C-11、C-15 | P6-6 | 所有ファイル：`docs/**`（G5 の `custom-components.md` を除く）、`examples/kivact-ui-soft/src/docs/overlays.rs`、`examples/kivact/component/src/main.rs`、`examples/kivact/form/src/main.rs`、`crates/kivact-ui-creative/README.md`、`crates/kivact-ui/src/lib.rs`、`crates/kivact/src/prelude.rs`、C-7 が名前を挙げた file（`crates/kivact-core/src/{app.rs,launch/settings.rs,embed.rs,view/boundary.rs}`、`crates/kivact-style/src/theme/value.rs` の doc と error の文字列）。**C-11 の重複 import**：`docs/for-kivact-user/{en,jp}/**/*.md` の rust の code block のうち、`use kivact::prelude::*;` を含むブロックで、`crates/kivact/src/prelude.rs` が既に export する名前を個別に import している `use` 行からその名前を消す（行が空になれば行ごと、`{X}` が 1 要素になれば `X` に）。prelude に無い名前は残す。**C-15**：prelude から `provide_theme` を外す（DEC-20）。compile error になった呼出し元に `use kivact::style::provide_theme;` を足す。この追従に限り、他の crate（kit・examples・dev）の file も変えてよい。ただし G5・G7 の所有 file に当たったら STOP。prelude の `use` 行の統合（同じ module の複数行を 1 行に）は行わない。|
| C | G8 | D6 9 章 C-5、8 章 1 | P6-8 | DEC-17 のとおり行う（承認済み） |

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

- **unit の内容の決め方**（雛形の `<節>` に書く）：D5 の unit は「D5 6 章の行の括弧内の ID が指す節」と「D5 の本文・付録のうち、その行の所有ファイルを変える全ての指示」。D8 の unit は「D8 6 章の行」と「D8 3 章・付録 A-G*（W 番号）・付録 B（unit 列がその unit の行）」。括弧内の ID の場所：

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

統合 branch から worktree `../kv-wt/W7` を作り、次を順に行う（それぞれ別の subagent。前の unit の commit の後に次を起動する）。

| unit | 付属書と節 | 本書の追加の指示 |
|---|---|---|
| FINAL-1 | D5 6 章 FINAL-1（PartState）、付録 P5-D F-1 | `cargo check --workspace --all-targets` の E0609/E0560/E0063 が示す全ての読み手を直す。kit・examples・dev/tests・docs のコード片も含む（この unit に限り、所有ファイルは compile error が示す全ての file） |
| G9 | D6 9 章 C-13、8 章 2 | 下の「G9 の改名表」を上から順に行う。旧名は残さない。compile error が示す全ての呼出し元（kit・catalog・examples・dev/tests・docs）を直す（この unit に限り、所有ファイルは compile error と `grep` が示す全ての file）。確認：`cargo check --workspace --all-targets --all-features`、`cargo xtask check --no-write --only api-shape`、`dev/tools/window-lock.sh --timeout 1800 -- cargo test -p kivact-public-api-tests --test e2e -- apg_combobox apg_tabs apg_menu interaction_menu ui_toolbar apg_tree` |
| FINAL-2 | D5 3.4（X3〜X5）、6 章 FINAL-2 | 確認の最後に D5 3.7 の「OPEN 0 の確認手順」を実行する。error が出たら付属書の誤りなので STOP。確認の後 `git checkout -- dev/xtask/state-audit.md` で戻してから commit する（audit の commit は CP2 の 11.2） |

**G9 の改名表**（ARG-2：項目の集まりは `items`。ARG-3：0〜n 個を選ぶものは `selected`、必ず一つを選ぶもの（tabs、segmented）は `value`。ARG-6：命令は `show`/`hide`）。引数名だけの変更は呼出し元に影響しない（Rust の位置引数）ので、関数の定義と rustdoc の引数の説明だけを直す。

| 順 | 対象（現在の名前は D6 C-13 の位置で確かめる） | 変更 |
|---|---|---|
| 1 | `MenuApi::show(factory)`、`CommandPaletteApi::show(factory)`（content を mount する method） | `content(factory)` に改名（DEC-18）。2 より先に行う |
| 2 | `MenuApi::open(self)`／`close(self)`、`ComboboxApi::open(self)`／`close(self)` | `show(self)`／`hide(self)` |
| 3 | `InlineEditApi::open(self, key)`（と同じ型の `close` があれば） | `show(self, key)`（`hide`） |
| 4 | `kivact-ui` `combobox(label, options, selection)` | 引数名 `items`、`selected`（型は変えない） |
| 5 | `kivact-ui` `tabs(label, options, binding)` | 引数名 `items`、`value` |
| 6 | `kivact-ui` `toggle_group(.., options, selection)` | 引数名 `items`、`selected` |
| 7 | `kivact-ui` `disclosure_group(selection)` | 引数名 `selected` |
| 8 | `kivact-ui` `tree(label, items).selection(sel)`（setter） | 位置引数 `tree(label, items, selected)` にし、setter `.selection(..)` を削除する。呼出し元は `.selection(x)` を消して第 3 引数に `x` を渡す |
| 9 | `kivact-interaction` の `use_*` hook の引数名 `options`／`selection`／`binding` | それぞれ `items`／`selected`／（必ず一つを選ぶ hook なら `value`、そうでなければ `selected`） |
| 10 | `kivact-ui-soft` `tab_list(binding)` | 引数名 `value` |
| 11 | `kivact-ui-creative` `gradient_editor(selection)` | 引数名 `selected` |

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

feature の組合せ（package を一つずつ。DEC-01 の後は既定の build に GPU が入らない）：

```bash
cargo check -p kivact --no-default-features
cargo check -p kivact
cargo check -p kivact --no-default-features --features gpu
cargo check -p kivact --features gpu-interop
cargo tree -p kivact -e features -i arvane-compose | grep -E '"gpu"' && echo "NG: gpu in the default build" || echo ok
cargo tree -p kivact -e normal | grep -E '^.*wgpu' && echo "NG: wgpu in the default build" || echo ok
KIVACT_TEST_MIRROR=off dev/tools/window-lock.sh --timeout 5400 -- cargo test -p kivact-public-api-tests --no-default-features
```

（DEC-01 が不承認と記録された場合は、`cargo tree` の 2 行を旧作業指示書 11 章のとおり `--no-default-features` に対して行う。）

- WSL があれば、WSL の中（`wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}"`）で上の clippy と `cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features` を同じ形で実行する。無ければ報告に「Linux 未確認」と書く。
- `architecture/` の変更を commit する。

### 11.2 状態の監査（AC-3）

D5 3.7 の「checkpoint（全体、OPEN 0 の確認）」の 4 行をそのまま実行する。`exit=0` かつ `OPEN` 0 件であること。生成された `dev/xtask/state-audit.md` を commit する。

### 11.3 KPI

```bash
CARGO_TARGET_DIR=<空きの多いドライブ>/kv-target/kpi dev/tools/window-lock.sh --timeout 3600 -- dev/bench/run.sh B1-here
```

未達の行は 7.3 の 1 と同じく 2 回測り直す（`B1-here-r2`、`B1-here-r3`）。

---

## 12. W8（Orchestrator）

### 12.1 受入れの判定

1. `dev/bench/results/RESULT-V2.md` に、`B0-here` と `B1-here`（と r2・r3）から、V2 設計書 2.3 節の各行の倍率と判定を表で書く。
2. AC-1〜AC-4 を確かめ、`RESULT-V2.md` に AC ごとの結果を書く：
   - AC-1：CP2 の 11.1 がすべて成功。
   - AC-2：全ての必達行を満たす（MS-4 の 3 回中 2 回を含む）。
   - AC-3：11.2 で OPEN 0 件、`OPEN.md` に状態の差戻しが無い。加えて「人間が audit を確認する」項目が残っていることを書く（DEC-16。Orchestrator は確認済みと書かない）。
   - AC-4：`OPEN.md` の「V2 差戻し」に AC-1〜AC-3 と公開 API に影響する項目が 0 件。U23 と U56 の項目に「V2 の M7・M10、G1〜G4 で解消」と書く。
3. どれかを満たさなければここで止める：`RESULT-V2.md` と `OPEN.md` を commit・push し、満たさない項目を人間に報告する。12.2・12.3 は行わない。

### 12.2 書き戻しと後片付け（AC をすべて満たしたときだけ）

1. `docs/for-kivact-developer/architecture.md`：旧作業指示書 W7-2 の手順 1 の内容に加え、V2 設計書 8 章の規則（PERF-1 の補足、PERF-4、PERF-5、MEM-8、MEM-9）を足す。DEP の feature の表で `kivact` の default を `complex-scripts` にする（DEC-01）。API-3 を DEC-19 のとおり書き直す。
2. `docs/for-kivact-developer/decisions.md`：旧作業指示書 W7-2 の手順 2 の内容（版 1.1 の 8.1・8.2）に加え、V2 設計書 8 章の SLIM-11〜SLIM-16 と、4 章の DEC-01・DEC-14・DEC-15・DEC-17〜DEC-20 を「実装の決定」の表の末尾に足す。
3. `dev/bench/results/OPEN.md` の「後続」節に V2 設計書 7 章の表を写す。
4. `SLIM_FAST_DESIGN.md`、`SLIM_FAST_WORK_ORDER.md`、`20261109_2107_CODEX_REVIEW.md`、`20261110_1145_CLAUDE_CODE.md`、`SLIM_FAST_V2_DESIGN.md`、`SLIM_FAST_V2_WORK_ORDER.md`、`slim-fast-v2/` を `git rm` する。`dev/bench/results/` と `dev/xtask/state-audit.md` は残す。
5. commit し、push する。

### 12.3 PR の説明の下書き（AC をすべて満たしたときだけ）

`dev/bench/results/PR_DRAFT.md` に次を書いて commit し、push する（PR は作らない）：

- 概要（V2 設計書 2.4 と 6 章の原因と、直した内容を 15 行以内）
- KPI の表（`RESULT-V2.md` の表）
- 公開 API の変更：版 1.1 の改名（付録 B）、ARG-1〜6、`#[component]`、`Binding::update`、`GpuStart`、`complex-scripts`、V2 の変更（`gpu` が既定でない、`GpuPreference` 一式が `kivact::launch` に移動、`RenderOptions::gpu_on_demand`・`Screen::renderer`・`Screen::wait_for_gpu`、`Stats::a11y_enumerated`、recipe state の accessor 化と `CheckControl`、G9 の改名と `content(factory)`、prelude から `provide_theme` を削除、D8 の公開 API の変更 3 件（`LayoutInset.own_outer`、`ShellImeText.password` の削除、`GpuAdapter` の `AdapterIds`））
- 確認の状況（CP2 の各コマンドの結果、Linux の確認の有無、AC-3 の人間の確認が残っていること）
