# Kivact 抜本改修（Rework）作業指示書

版 1.0（2026-11-18）。設計の正本は [REWORK_DESIGN.md](REWORK_DESIGN.md)（以下「設計書」）、unit の手順は付属書 `rework/<X>-*.md`（以下「付属書」）。本書は**進め方だけ**を書く。設計の判断は設計書と付属書が持つ。

目標：設計書 1 章の AC-1〜AC-5 をすべて満たす。

---

## 0. 読み方と規則

### 0.1 誰が何をするか

| 役割 | 担当 | すること |
|---|---|---|
| Orchestrator | 本書を受け取った AI（親） | wave を順に進める。lane ごとに worktree を作り、unit ごとに subagent を起動する。結果を merge する。gate と KPI の計測を行う。subagent の `STOP:` に 0.5 節の表で答える。**設計の判断をしない** |
| Implementer | subagent（**同時に最大 6 体**） | 割り当てられた unit を、指定の worktree の中で、付属書の手順どおりに実装する。確認のコマンドを実行し、結果を報告する |

subagent の起動（実行環境ごと。unit・lane・wave・STOP の規則は同じ）：

| 実行環境 | 起動の方法 | モデル |
|---|---|---|
| Claude Code | Agent tool を `subagent_type: "general-purpose"`、`model: "haiku"`、`effort: "high"` で呼ぶ | Haiku 5.5、effort high |
| その他 | その環境の subagent の起動の道具に、1.2 節の雛形をそのまま渡す | ユーザーの指定 |

- 同時に動かすのは最大 6 体（lane ごとに 1 体）。subagent に subagent を作らせない。
- Implementer は push・merge・PR 作成をしない。`main` に触れない。
- Orchestrator は wave の merge の後と checkpoint の後にだけ `git push origin refactor/kashiwade/fundamental-rework` する（force push 禁止）。PR は作らない。`main` に merge しない（人間が PR で行う）。

### 0.2 禁止事項（全 unit 共通）

1. 互換のためのコードを書かない：旧名の alias、`#[deprecated]`、wrapper、旧と新の二重の経路。旧名は消し、呼出し元を全部直す。
2. CI の設定・workflow を足さない。
3. 付属書が名前を挙げていない試験を足さない。期待値を緩めない（許容差を広げる、assert を消す、`#[ignore]` を付ける）。
4. `#[allow(..)]`・`#[expect(..)]`・`xtask-allow` を、警告や検査の件数を減らす目的で足さない。付属書が文面まで指定した印だけを書く。
5. 1 関数 80 code 行・1 ファイル 1000 code 行・引数 7 個の上限を超えない。超えそうなら意味のある helper に分ける。
6. 実装の途中で workspace 全体の `cargo test`、E2E の全件、`dev/bench/run.sh` を実行しない（checkpoint で Orchestrator が実行する）。unit で実行してよいのは 0.6 節と、付属書の unit の「確認（tier a）」のコマンドだけ。
7. 依存の crate を足さない（設計書か付属書が名前を挙げたものを除く）。
8. `std::cell::OnceCell`、`std::sync::OnceLock`、`LazyCell`、`LazyLock` を使わない。遅延初期化は `kivact_types::cell::{Deferred, SharedDeferred}` だけ（設計書 DEC-13）。
9. **行番号を信じない**。付属書と監査報告の `file:line` は基点 `2b8df5ac` のもの。関数名・型名・field 名・周りのコードで場所を探す。
10. 設計書の 5 章（公開 API の契約）と違うコードを書かない。付属書の手順が契約と食い違うなら STOP。
11. 所有ファイル（付属書の unit の「所有」）の外を変えない。ただし、署名を変えた結果の「compile を通すための機械的な追従」は、同じ crate の中なら行ってよい。他の crate なら STOP。
12. 生成物（`architecture/`、`target/`、`dev/bench/results/`）を commit しない。

### 0.3 unit の読み方

付属書の各 unit は次の欄を持つ：wave / lane / 前提、所有、目的、手順、消すもの、状態の閉じ方、試験、確認（tier a）、完了条件。

- **手順**は番号の順に行う。コード片は書いてあるとおりに書く（`use` の過不足は compile error に従って直してよい）。
- **状態の閉じ方**に `// xtask-allow(state): ...` の文面があれば、その文面を一字一句書く。
- 本書の wave の表の「追加の指示」は付属書より優先する。

### 0.4 停止して報告（STOP）

次の場合は、その項目の作業を止め、他の項目を続け、最後の報告に `STOP:` で始まる行を書く。

- 付属書の手順が実際のコードと合わない（関数・型・field が無い、型が違う、名前で探しても見つからない）。
- 前の unit が、付属書が対象とする関数・型・field を消した・改名した・型を変えた（付属書がその場合の扱いを書いていればそれに従う）。
- 手順を行うと所有ファイルの外を変える必要がある（0.2 の 11 の追従を除く）。
- 同じ compile error を 3 回直しても消えない。
- 既存の E2E の期待値を変えないと通らないと分かった。
- debug の `self_check`（a11y、hit、paint、layout）の不一致が出た。修正を試みず、試験の名前・直前の操作・メッセージを書く。
- xtask の `state` 検査が、付属書に無い型を報告した。ただし W1b の印の変換の unit（付属書 T の手順 M を使う unit）はこの STOP の対象外で、手順 M に従って印を書く。
- 設計書 5 章の契約と付属書の手順が食い違う。

STOP にせず unit の中で直すもの（設計の判断ではない）：関数・ファイル・引数の上限を超えた（意味のある helper に分ける、引数を struct にまとめる）、clippy の機械的な指摘（clippy の提案どおり）、rustfmt、`use` の過不足。

書式：`STOP: <unit> <file>:<関数名> <何が合わないか 1 文> / <試したこと 1 文>`。

### 0.5 Orchestrator の既定の判断表

Orchestrator は設計の判断をしない。`STOP:` に次の表が当たればその判断をする。当たらなければ、その項目を**設計者への差戻し**として `rework/OPEN.md`（無ければ作る）の「差戻し」節に 1 行で書き（unit、場所、STOP の文、付属書の節）、他の作業を続ける。差戻しが 1 件でも残っていれば、最終 checkpoint で完了を宣言しない。

| 状況 | 判断 |
|---|---|
| 行番号がずれているが、同じ名前の関数・型・field が近くにある | その名前に対して手順を行うよう、同じ subagent に指示する（SendMessage） |
| 所有ファイルの外の 1〜3 ファイルを変える必要がある | 同じ wave の他の lane の所有でなければ許可する。他の lane の所有なら、その項目を次の wave の同じ crate の lane に回す（その unit の手順の最後に足す） |
| compile error が、他の lane の未 merge の変更を前提にしている | その unit を次の wave に回す |
| subagent が確認のコマンドを実行していない | 同じ subagent に実行させる |
| 前の unit の残り（dead_code の警告、古い doc）で、この unit の確認が失敗する | 前の unit の lane の fix として 1.3 節の手順で直す（警告の対象を消すか使う。`#[allow]` で黙らせない）。設計の判断が要るときだけ差戻し |
| 性能の変更で既存の E2E が失敗し、2 回直しても通らない | 差戻し（期待値は緩めない） |
| 状態の unit の対象が、前の unit で消えている | 付属書が扱いを書いていなければ差戻し |
| compile の対象外（2.1 節の `XD`・`XK`・`XT`）の package だけが壊れる | 想定どおり。何もしない |

### 0.6 unit で実行してよいコマンド

（gate の 3 段の正確なコマンドは 2 章。ここは unit の中で使ってよい範囲。lock の一覧は 2.1 節）

```bash
cargo fmt -p <crate>                                         # 必ず最後に
cargo check -p <crate> --all-targets                         # 何度でも
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --crate <crate>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p <crate> <filter>    # 付属書が名前を挙げた試験だけ
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test <e2e_core|e2e_ui|e2e_native> -- <filter>   # 付属書が名前を挙げた E2E だけ
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- <filter>   # W1b の T15 以降。W6a〜W10 は compile の対象外（K44 が W11a で戻す）
```

- すべての cargo のコマンドに環境変数 `CARGO_TARGET_DIR=<1.1 節の target>`、`CARGO_PROFILE_DEV_DEBUG=0`、`CARGO_PROFILE_TEST_DEBUG=0` を付ける（debug assertions は有効のまま）。
- 2.1 節の lock の一覧の package の `cargo test`（と `kv_*` の probe）は、`KIVACT_TEST_MIRROR=off` であっても**必ず** `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off ...` を通す（`MirrorMode::Required` で実 window を開く E2E があるため）。
- Linux 専用のコード（`cfg(target_os = "linux")`、`port/x11`、`port/wayland`）を変える unit は、`cargo check -p <crate> --target x86_64-unknown-linux-gnu`、それができなければ `wsl -d "${KIVACT_WSL_DISTRO:-Ubuntu-24.04}" -- bash -lc 'cd <worktree の WSL path> && cargo check -p <crate>'` で確かめる。どちらもできなければ Linux 専用の項目を STOP にする。

---

## 1. 体制

### 1.1 branch・worktree・target

- 統合 branch：`refactor/kashiwade/fundamental-rework`。
- **lane の worktree は wave ごとに作る**。wave `<W>` の lane `<L>`：

```bash
git worktree add ../kv-wt/rw-W<W>-<L> -b wt/rw-W<W>-<L> refactor/kashiwade/fundamental-rework
git -C ../kv-wt/rw-W<W>-<L> submodule update --init
```

  その lane の unit はすべて同じ worktree で、表の順に一つずつ行う（unit ごとに commit）。lane の中の unit は同時に動かさない。
- target：W0 の手順 3 で決める。空きが 200 GB 以上のドライブがあれば lane ごと（`<drive>/kv-target/rw-<L>`）。無ければ 3 つを共有する（lane 1・2 が `kv-target/rw-12`、3・4 が `rw-34`、5・6 が `rw-56`）。同じ target を使う 2 つの lane の build は cargo の lock で順になる。これは正常。
- 各 wave の始めに空きを確かめ、15 GB 未満なら、その wave で使わない target dir を削除する（使っている target は消さない）。

### 1.2 subagent への指示の雛形

Orchestrator は各 unit に次の文をそのまま渡し、`<..>` だけを埋める。

```text
あなたは Kivact の改修の unit <unit ID> の実装者です。モデルは Haiku です。あなた自身は subagent を作らないでください。
作業場所は <worktree の絶対 path> です。この directory の外の file を変更しないでください。
すべての cargo のコマンドに次の環境変数を付けてください：CARGO_TARGET_DIR=<target の絶対 path> CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
最初に次を読んでください：
  1. <worktree>/REWORK_WORK_ORDER.md の 0 章（規則）と、wave の表の「<unit ID>」の行（追加の指示を含む）
  2. <worktree>/rework/<付属書のファイル> の 1 章、4 章と「## <unit ID>：」の節
  3. <worktree>/REWORK_DESIGN.md の 5 章と 6 章のうち、その節が参照する箇所
付属書の手順を番号の順に実行してください。行番号はずれています。関数名・型名・field 名で場所を探してください。
本書の「追加の指示」が付属書と食い違うときは本書に従ってください。設計書 5 章の契約と付属書が食い違うときは STOP してください。
設計の判断が必要になったら作業を止め、0.4 節の書式で STOP: 行を書いてください。
付属書の「確認（tier a）」のコマンドをすべて実行し、error と警告を直してください。最後に cargo fmt -p <変更した各 crate> を実行してください。
最後に `git -C <worktree> add -A && git -C <worktree> commit -m "<unit ID>: <要約>"` で commit してください。push・merge はしないでください。
報告には次だけを書いてください：変更した file の一覧、確認の各コマンドの結果（成功/失敗と最後の数行）、完了条件の各項目の結果、STOP: 行。
```

### 1.3 merge の手順（Orchestrator、各 wave の終わり）

lane の表の順に：

```bash
git checkout refactor/kashiwade/fundamental-rework
git merge --no-ff wt/rw-W<W>-<L> -m "rework W<W>-<L>: <unit ID の列>"
```

- conflict：
  - `dev/tests/kivact-public-api/tests/<binary>/main.rs` の `mod` 行、`crates/*/src/lib.rs` の `mod`・`pub use` 行、`Cargo.toml` の `members`：両方の行を残し、元の並びの規則（アルファベット順など）に揃える。
  - `Cargo.lock`：`git checkout --theirs Cargo.lock && cargo metadata --format-version 1 >/dev/null` で作り直す。
  - それ以外：本書の wave の表でそのファイルを所有する lane の版を採る。同じ wave で二つの lane が同じファイルを持たないように表を作ってある。それでも起きたら作業を止め、人間に報告する。
- 全 lane を merge したら、2 章の **tier b** を 1 回実行する。失敗したら、error を出した crate を所有した lane の最後の unit の subagent に、error の全文を渡して直させる。直しは統合 branch の今の HEAD から作った worktree `../kv-wt/rw-W<W>-<L>-fix`（branch `wt/rw-W<W>-<L>-fix`）で行い、同じ手順で merge する。
- merge の後、worktree と branch を消す：`git worktree remove ../kv-wt/rw-W<W>-<L> && git branch -D wt/rw-W<W>-<L>`。
- push する。
- `rework/PROGRESS.md`（無ければ作る）に 1 行を足す：`W<W> merged <commit>: <unit の列>、tier b 成功`。

---

## 2. gate の 3 段

付属書 T の 4.1・4.2 節を基にした正本。食い違ったら本章が正しい。

### 2.1 共通

- **すべての cargo のコマンドに次の環境変数を付ける**（debug assertions は有効のまま）：`CARGO_TARGET_DIR=<1.1 節の target> CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`。
- **window-lock**：`kivact::test` の E2E は `KIVACT_TEST_MIRROR=off` でも `MirrorMode::Required` の試験が実 window を開く。次の package（**lock の一覧**）の `cargo test` は必ず `dev/tools/window-lock.sh` を通す：`kivact-public-api-tests`、`kivact-kit-tests`、`kivact-test`、`kivact-docs`、`kivact-devtools`、5 つの kit、5 つの catalog、`catalog-shell`、`kivact-plot`、`kivact-example-*`、`kivact-template-basic`、`arvane`、`arvane-native`、`gpu-interop-producer`。書き方は `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p <crate> <filter>`（gate の workspace の試験だけ `--timeout 5400`）。それ以外の package の `cargo test`、および `cargo fmt`・`check`・`clippy`・`xtask` は lock なし。
- **compile の対象外**（DEC-14）にする仕組みは gate の `--exclude` だけ。root の `Cargo.toml` の `members`・`exclude` は W1 の後どの wave でも変えない。付属書の手順が「`members` から消す／戻す」と書いていたら何もしない。
  - `XD`（W5a〜W10 の tier b、CP2）：`--exclude kivact-docs`
  - `XK`（W6a〜W9 の tier b、CP2）：`--exclude kivact-ui-basic --exclude kivact-ui-pop --exclude kivact-ui-game --exclude kivact-ui-creative --exclude kivact-ui-soft --exclude kivact-ui-basic-catalog --exclude kivact-ui-pop-catalog --exclude kivact-ui-game-catalog --exclude kivact-ui-creative-catalog --exclude kivact-ui-soft-catalog --exclude kivact-kit-tests`
  - `XT`（W10 の tier b）：`--exclude kivact-kit-tests`
  - xtask：W1a〜W4b と W11a 以降は `cargo xtask check --no-write`（全体）。**W5a〜W10 は `cargo xtask check --no-write --only local`**（xtask-core の style の Clippy は workspace 全体を compile するので、対象外の crate があると必ず失敗する）。

### 2.2 tier a（unit ごと）

付属書の各 unit の「確認（tier a）」のコマンドを実行する。雛形：

```bash
cargo fmt -p <crate>
cargo check -p <crate> --all-targets
cargo clippy -p <crate> --all-targets -- -D warnings
cargo xtask check --no-write --crate <crate>                      # pending を閉じる unit は --strict を付ける（付属書が書く）
cargo test -p <crate> <filter>                                     # lock の一覧に無い package
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p <crate> <filter>      # lock の一覧の package
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native --all-features -- <module>
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- <module>
```

W1b の T15 より前は E2E の binary が一つ（`--test e2e`）である。

### 2.3 tier b（wave の merge の後。Orchestrator）

W1a〜W4b と W11a 以降の形：

```bash
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs
cargo check -p kivact-docs --all-targets --all-features
rg -n "xtask-allow\(state-fields\)" crates dev examples templates     # W1c の後：何も出ない
```

- W5a・W5b：xtask を `--only local` にし、clippy と test に `XD` を足し、`cargo check -p kivact-docs` を行わない。
- W6a〜W9：W5a の形に加えて clippy と test に `XK` を足す。
- W10：W5a の形に加えて clippy と test に `XT` を足す（kit と catalog は含める）。
- W11a 以降：上の形のまま（何も除外しない）。

### 2.4 tier c（checkpoint：CP1・CP2・CP3）

CP1 と CP3 の形：

```bash
cargo fmt --all -- --check
cargo xtask check --no-write
cargo clippy --workspace --all-targets --all-features -- -D warnings
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features
dev/tools/window-lock.sh --timeout 5400 -- cargo test -p gpu-interop-producer --all-features -- --test-threads=1
cargo check -p kivact-docs --all-targets --all-features
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-docs --doc --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --no-default-features
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-public-api-tests --test e2e_core --test e2e_native --all-features -- --test-threads=1
dev/tools/window-lock.sh --timeout 5400 -- env KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required cargo test -p kivact-test --test gpu -- --test-threads=1
dev/tools/check-features.sh
```

- CP1 の時点（W4b の後）は `kivact-docs` の doctest がまだ英語の木だけになっている（W1b の T17）。
- CP2（W8 の後）：xtask を `--only local` にし、clippy と 2 つの workspace の test に `XD` と `XK` を足し、`kivact-docs` の 2 行を行わない。GPU lane の 2 行と `check-features.sh` はそのまま。加えて次の 2 つを実行し、結果（error の行の数と検査の名前）を `rework/PROGRESS.md` に書く。**失敗しても先に進む**。
  ```bash
  cargo xtask check --no-write --only local --strict --crate kivact-ui
  cargo xtask check --no-write --only local --strict --crate kivact-interaction
  ```
- CP3（W11c の後）：上の形のまま。G15 が `[rework.pending]` と `--strict` を消した後なので、xtask のすべての検査が error になる（AC-3）。加えて 2.5 節の AC-4 の検査を行う。
- `gpu-interop-producer` と `kivact-docs` の `--no-default-features` は行わない。GPU lane は GPU の無い環境では `kivact::test::gpu_unavailable()` の理由を出して skip する。

### 2.5 AC-4 の検査（設計書 5.12 の削除名。正本）

CP3 で実行し、**どれも何も出さない**ことを確かめる（出力の扱いは下の規則）。

```bash
EXCL=(--glob '!docs/for-kivact-user/*/how-to-guide/migrating-to-rework.md' --glob '!docs/for-kivact-developer/decisions.md' --glob '!REWORK_*.md' --glob '!rework/**' --glob '!target/**')
rg -n -w "${EXCL[@]}" --glob '!dev/xtask/src/public_surface.rs' \
  -e Binding -e IntoBinding -e IntoReactive -e Reactive -e Notifier -e notifier \
  -e ActionOutcome -e ActionMode -e Retry -e AsyncSnapshot \
  -e ThemeColor -e ColorToken -e ColorName -e Palette -e Spacing -e Radii -e UiRecipes -e UiTokens -e use_tokens -e ThemeHint \
  -e TextLen -e GridFlow -e Insets -e TextExtras -e DecorationStyleValue -e prefer_gpu \
  -e use_menu_with_open -e on_open_change -e on_step_press -e on_draft_commit -e selection_field -e input_group_with -e while_open \
  -e BadgeHue -e ItemHue -e StatHue -e ProgressHue -e WeekendHue -e CodeTone -e CodeColors \
  -e GameInputGroupVariant -e GameCellVariant -e PopBubble \
  crates examples templates docs dev
rg -n -P "${EXCL[@]}" '(?<!taffy::)\bDimension\b' crates examples templates docs dev
rg -n -w "${EXCL[@]}" --glob '!crates/kivact-devtools-protocol/**' --glob '!crates/kivact-devtools/**' --glob '!crates/kivact-core/src/devtools/**' TokenKind crates examples templates docs dev
rg -n -w "${EXCL[@]}" ButtonTone crates/kivact-ui-game examples/kivact-ui-game docs/for-kivact-ui-user/en/game docs/for-kivact-ui-user/jp/game
rg -n -w "${EXCL[@]}" SpanStyle crates/kivact-style crates/kivact-core crates/kivact crates/kivact-ui crates/kivact-ui-basic crates/kivact-ui-pop crates/kivact-ui-game crates/kivact-ui-creative crates/kivact-ui-soft examples templates docs
rg -n "${EXCL[@]}" -e 'kivact::launch' -e 'launch::AppOptions' -e 'interaction::ViewTransform' -e 'DialogMode::Popover' crates examples templates docs dev
rg -n -P "${EXCL[@]}" '\b(toaster|announcer)\(' crates examples templates docs dev
rg -n -F "${EXCL[@]}" '.composite(' crates examples templates docs dev
rg -n -P 'pub fn (weight|italic|radius\w*|ring\w*|basis|grow|shrink|wrap|direction|pressed)\b' crates/kivact-style/src
rg -n -w constant_value crates examples templates docs dev      # 各行が付属書 R の 4.7 節の「許す場所」にあること
```

- `Binding` は xtask 内部の型（`public_surface.rs`）を除く。`TokenKind` は DevTools の wire の型を除く（protocol の `TokenKind` は残す）。`ButtonTone` は game だけを見る（pop の `ButtonTone` は残す）。`Dimension` は taffy の型を除く。
- 旧 style setter（`.weight(` など）は style 以外の API と名前が重なるので、最後から 2 行目（`kivact-style` に定義が無い）と tier c の compile で確かめる。

**出力の扱い**（付属書 G の G15 から写した。CP3 では Orchestrator が、W11c では G15 の実装者がこの規則で読む）：

- (i) 付属書 R の 4.7 節が `constant_value` を許すと列挙した場所（`#[doc(hidden)]` の定義と内部の利用）の行は残す。
- (ii) `AppOptions` が `pub` でない型・field の名前として crate の内部にある行は残す（5.12 は公開の path だけを禁じる）。
- (iii) `Composite` は `Mix::Composite(..)` と `Composite::<variant>` の中の行を対象にしない（`.composite(` だけを見る）。filter graph の primitive と `MaskLayer` の `.composite(` は残す（`Layer` の method ではない）。
- (iv) `.wrap(`・`.direction(`・`.radius(`・`.grow(` などは、受け手が `Style`（`style()` の鎖、`Style` の値）のものだけが旧名。受け手が部品（`text_area(..).wrap(true)`、code block の `.wrap(true)`）、`Draw`、`Shadow`、`Transform` などの値のものは残す。
- (v) 文字列の literal の中の英語の語（button の label の `"Retry"` など）と、Kivact の API を指さない英語の本文の語（文頭の「Reactive」、DevTools の「Palette」の UI 名など）は残す。
- それ以外の行は旧名の残りである。文書・comment・doc comment・文字列の中なら、移行の置換表（`docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md`）で新しい名前に直す。コードの識別子として残っていたら、その名前を所有する付属書の削除漏れとして `rework/OPEN.md` の「差戻し」に書く。
- 残した行（(i)〜(v)）は、ファイル・行・理由を `rework/PROGRESS.md` に一覧にする。

---

## 3. W0：準備（Orchestrator が自分で行う）

1. `git status` が clean で、`refactor/kashiwade/fundamental-rework` にいることを確かめる。
2. 基点の試験の名前を記録する：`KIVACT_TEST_MIRROR=off cargo test --workspace --exclude gpu-interop-producer --exclude kivact-docs --all-features -- --list 2>/dev/null | grep ': test$' | sort > rework/baseline-tests.txt`（commit しない。`rework/.gitignore` に `baseline-tests.txt`、`OPEN.md` 以外の生成物を書く）。
3. target の場所を決める（1.1 節）。`df -h` で空きを見て、決めた path を `rework/PROGRESS.md` の先頭に書く。
4. **B0 を測る**：PC を他の作業に使わない時間に、`dev/tools/window-lock.sh --timeout 5400 -- dev/bench/run.sh B0`（基点 `2b8df5ac` のまま、W1 の前に測る）。`dev/bench/results/B0.md` を `rework/kpi/B0.md` に写す（`rework/kpi/` は commit する：比較の正本として残す）。report の末尾に `INVALID:` があれば原因を直さずにもう一度測る。2 回とも INVALID なら作業を止めて人間に報告する。
5. 基点の full gate（2 章 tier c）を一度実行し、失敗する試験があれば `rework/OPEN.md` の「基点の失敗」節に名前を書く（改修の失敗と区別するため。直さない）。

---

## 4. wave の表

全 wave 共通の追加の指示（各 lane のすべての unit に効く）：

1. 試験を実行する `cargo test`（付属書の「確認」の行を含む）は、2.1 節の lock の一覧の package なら、付属書が書いていなくても `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で実行する。
2. `e2e_kits` は package `kivact-kit-tests` の binary である。付属書が `-p kivact-public-api-tests --test e2e_kits` と書いていたら `-p kivact-kit-tests --test e2e_kits` と読む。`dev/tests/kivact-public-api/tests/e2e_kits/` は `dev/tests/kivact-kit-tests/tests/e2e_kits/` と読む（INC-10）。
3. 付属書の所有・手順が `dev/tests/kivact-public-api/tests/e2e/<topic>.rs` と書いていたら、W1b の後は `ls dev/tests/kivact-public-api/tests/*/<topic>.rs dev/tests/kivact-kit-tests/tests/*/<topic>.rs` で見つかる file と読む（INC-16）。
4. kit・catalog・`kivact-kit-tests`・`kivact-docs` の compile の除外は gate の `--exclude` だけで行う。root `Cargo.toml` の `members`/`exclude` は変えない。付属書 K・G の「`exclude` から消して `members` に戻す」手順は何もしない（INC-09）。
5. tier b の xtask は W5a〜W10 で `cargo xtask check --no-write --only local`、それ以外は `cargo xtask check --no-write`（2 章）。
6. R06：E31（W4b）が `launch/runtime.rs` の turn の event の loop に足した `crate::view::state::ViewState::tree(&rt).engine().deliver(event);` の行も、turn の関数と一緒に `launch/turn.rs` へ移す。C07：`edits.deliver(event);` を消すとき、その次の `...engine().deliver(event);` の行は残す（付属書 E の 4.2 節 E-REQ-1）。
7. 本章の各 wave の「追加の指示」の列は付属書より優先する。`INC-xx` は付属書の間の不整合の検査（すでに付属書へ反映済み）の項目の番号で、背景を示すだけである。

---

### W1a：xtask の state 規則と DevTools の整理、`Deferred`

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| T0 | T01 → T02 → T03 → T04 → T23 | T | `dev/xtask/src/*`、`dev/xtask/policy/workspace.toml`、`python/`、`crates/kivact-devtools-protocol`、`kivact-devtools`・`-client` の試験・`-mcp`、core の `devtools.rs`・`devtools/{scripts,target}.rs` | — |
| S | S01 | S | `crates/kivact-types/src/cell.rs` | — |

前提の確認：なし（基点 `2b8df5ac` と W0）。
tier b の compile の対象外：なし（tier b の基本の形）。

### W1b：印の変換と e2e の分割、docs・scripts（crate の群ごと）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| T1 | T05 → T06 | T | `crates/kivact-ui/src/**`、`crates/kivact-interaction/src/**` | — |
| T2 | T07 → T08 | T | core・facade・macros・engine・layout・editing の `src/**` | — |
| T3 | T09 → T10 | T | runtime・i18n・style・types（`cell.rs` を除く）・shader・svg・gpu-share・`arvane*`・bench・producer の `src/**` | — |
| T4 | T11 → T12 | T | 5 kit の `src/**`、`examples/**`（shared-assets を除く）、`templates/**` | — |
| T5 | T13 → T14 → T15 | T | DevTools の crate、root `Cargo.toml` の `members`、`Cargo.lock`、`dev/tests/kivact-public-api/**`、新 `dev/tests/kivact-kit-tests/`、kit の README、workspace.toml の style 例外 1 件 | T15 は `kivact-kit-tests` を作る（54 package）。`apg_grid_kit`・`presence_kit` の名前は T15 のまま（INC-11） |
| T6 | T16 → T17 → T18 → T19 → T20 → T21 | T | `crates/kivact-test/**`、`dev/tests/docs/**`、docs の 4 file、README、strategy.md、`dev/tools/*.sh`、`architecture/`、`.gitignore`、CONTRIBUTING・dev/README・manual-verification | — |

前提の確認：W1a（T23 の後。T 1.3）。
tier b の compile の対象外：なし。

### W1c：pending の `state`・`docs-pairs` を外す

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| T0 | T22 | T | `workspace.toml`、`[workspace.lints.rust]`、`redundant_imports` の警告の file（全体） | — |

前提の確認：W1b の全 lane。
tier b の compile の対象外：なし。`rg -n "xtask-allow\(state-fields\)"` が空。

### W2a：protocol の新設と各 crate の内部（公開 API は変えない）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| A | S02 → S03 → S04 → S05 → S06 | S | `crates/kivact-types/`、新 `arvane-protocol/`、`kivact-engine/`、`arvane*/`、`kivact-svg/`、root `Cargo.toml`（`members`）、`workspace.toml`、core の `view/drawable/events.rs`・`view/drawable_share.rs`・`launch/runtime.rs` と S04 手順 10 の `match`、editing `geometry.rs`、style `draw.rs`・`draw/layer.rs`・`resolve/compile.rs`、test `screen/state.rs`・`lib.rs` | S04 手順 10 の `match` の直しは `crates/kivact-core/src/devtools/console.rs`（R04）と `crates/kivact-devtools/`（K01）に及ばないこと。及ぶなら STOP（INC-23） |
| S | S07 → S08 → S09 | S | style `Cargo.toml`・`shader/*`・`effect/*`・`icon.rs`・`theme*`・`lib.rs`・`facade.rs`・`color/value.rs`、`kivact-shader` の 3 file、core・kivact の `Cargo.toml`（`[features]`）、e2e `shaders.rs` | — |
| R | R01 → R02 → R03 → R04 | R | `crates/kivact-i18n/`、runtime の `reactive/*`・`task/*`・`machine.rs`、core `devtools/console.rs` | — |
| U | U01 → U02 → U03 | U | `crates/kivact-interaction/`（`Cargo.toml`、doctest、collection・list・select・tabs・combobox・menu・palette） | U02 の `e2e_kits` の行は全体指示 2 のとおり。W2 では kit を compile する |
| K | K01 → K02 → K03 → K04 → K05 | K | `crates/kivact-devtools/`（KIT-18）、`Cargo.lock`、soft `mobile_nav.rs`・`timestamp.rs`、creative `editors/`・`picker.rs`・`audio/meter.rs`、game `surface.rs` | — |

前提の確認：W1a（S01 の `Deferred`）、W1b・W1c（T の印・e2e の分割）。
tier b の compile の対象外：なし。

### W2b：core の DevTools の hook と `kivact-offscreen` の crate（W2a の lane A・S・R とファイルが重なるため分けた）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| C | C01 → C02 | C | core の `view/devtools_*`・`inspect_mode.rs`・`devtools/`・`devtools.rs`・`view.rs`・`view/{tree,state,boundary,root,mount/style}.rs`・`launch/{runtime,inspect}.rs`、新 `crates/kivact-offscreen/`、root `Cargo.toml`、core・kivact の `Cargo.toml`、core `draw.rs`・`draw/offscreen.rs`（削除）、facade `lib.rs`・`draw.rs`、`workspace.toml`（`visible_to`） | C01 は S04 が `launch/runtime.rs` に足した `HostEvent::StatesChanged` の腕を残す |

前提の確認：W2a の lane A（S04 の `launch/runtime.rs`）、lane S（S07 の `Cargo.toml` の `[features]`）、lane R（R04 の `devtools/console.rs`、`Runtime::reported_since`）。
tier b の compile の対象外：なし。

### W3a：style・svg・runtime の motion・core の内部・interaction・Arvane の描画と文字

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| S | S10 → S11 → S12 | S | types `type_map.rs`・`lib.rs`、style `theme/value.rs`・`draw/*`・`internal.rs`、`kivact-svg/src/`、e2e `offscreen_svg.rs`、core `view/mount/draw.rs`、`workspace.toml` の 1 行 | S12 の所有の `crates/kivact-core/src/draw/offscreen.rs` は C02 の後の `crates/kivact-offscreen/src/lib.rs` と読む（INC-08） |
| R | R07 → R10 | R | runtime `motion/*`、core `view/flow/virtual_list.rs`・`virtual_list/heights.rs`（新） | — |
| C | C03 → C04 | C | core `view/{state,requests,snapshots,foreign_window,mount/resolve,node_ref,elements/builders,mount/style,mount/create,boundary,root,tree,node_modifiers}.rs`、`client/dispatch/{platform,delivery}.rs`、`platform/{app_handle,services,env}.rs`、`gpu_interop*`、`launch/runtime.rs`、`diagnostics/why.rs`、`devtools/node.rs` | — |
| U | U04 → U05 → U06 → U07 → U08 | U | interaction の table・tree・combobox・palette・popup・dialog・list・select・menu・scalar・delayed・toaster、ui の calendar と date の view | — |
| A1 | A01 | A | `arvane-render/src/*`（列挙の file）、`arvane-compose/src/cpu/compositor.rs` | — |
| A2 | A02 | A | `arvane-text/`、`arvane-asset/src/{font,lib}.rs`、`arvane/src/{engine,asset}.rs` | — |

前提の確認：W2a（S02〜S06 の protocol：A01・A02、S09：S10）、W2b（C01：C03・C04、C02：S12）、W2a の R04（R07）。
tier b の compile の対象外：なし。

### W3b：engine の時計・motion・slot・祖先（kivact-types の `lib.rs` を S10 と分けた）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| E1 | E01 → E02 | E | `kivact-types/src/{clock,lib}.rs`（1 行）、engine `model/frame*`・`app_model.rs`・`model.rs`・motion の directory・`host/{pump,events,window_changes}.rs`・`model/motion_run.rs` | E01 は S10 の後の `lib.rs` に `pub mod clock;` を足す（E 4.1 の A-1 は不要になる） |
| E2 | E03 → E04 | E | engine `model/{aggregate,lifecycle}.rs` と `editors`・`measure`・`paint.ink/look` を読む file、`tree/*`、`kivact-layout/src/*`（4 file と試験） | — |

前提の確認：W2a（S02〜S06 の protocol と prop_interp の移動）、W3a の S10（types `lib.rs`）。
tier b の compile の対象外：なし。

### W4：core・ui の配置の移動・engine と Arvane の性能

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| C | C05 | C | core `view/{state,window_view,foreign,foreign_window,root,hover,tree}.rs`、`launch/{runtime,close}.rs`、`client/dispatch/env.rs`、`platform/env.rs`、`custom/node.rs` | — |
| U | U09 → U10 → U11 → U12 → U13 → U14 | U | ui の carousel・async_view・button・sparkline・combobox と部品ごとの directory への移動（ui 全体）、interaction `menu/level.rs`・`drag/`・`combobox/handle.rs`・`tooltip/api.rs` | — |
| E1 | E05 → E06 → E07 → E08 → E09 → **E30（新設が必要）** | E | engine `model/edit/*`・`focus/navigation.rs`・`keys.rs`・`host/events.rs`・`host/write/{reader,writer,typed}.rs`・`measure/*`・`apply.rs`・`paint/state.rs`・`tree/node_tree.rs`・`contract*`・`model/{events,lifecycle}.rs`・`text_input*`、`kivact-editing/src/*`；E30：`Routed` の定義の file と literal の 7 か所、`kivact-test/src/host_sync_tests.rs` | E30 は INC-01・INC-02 の E-REQ-6（`InputOrigin`）と `TreeChanges` の window 付きの形。E30 が付属書 E に足されるまで C06 を始めない |
| E2 | E10 → E11 | E | engine `model/changes.rs`・`model/interact/*`・`model/aggregate.rs`・`host/pump.rs`・`layer.rs`・`paint/list/{look,walk}.rs` | — |
| A1 | A03 | A | `arvane-render/src/{gpu.rs,gpu/*,lib.rs}`、`arvane-compose/src/`（`plan/` を除く）・`Cargo.toml` | — |
| A2 | A04 | A | `arvane-asset/`、`Cargo.lock`、`workspace.toml`（`visible_to`）、`arvane/tests/e2e/assets.rs` | `visible_to` の `arvane-protocol = [..]` にも `"arvane-asset"` を足し、`arvane-asset/Cargo.toml` に `arvane-protocol` の依存を足す（INC-33） |

前提の確認：W3a（C03・C04：C05、U08：U09、A01・A02：A03・A04）、W3b（E01〜E04：E05〜E11）。
tier b の compile の対象外：なし。

### W4b：DevTools の残り（C06）と engine の C 向けの API（E31）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| C | C06 | C | core `devtools/`（全体）・`view/devtools_hooks.rs`・`view/mount/style.rs`（1 か所）・`launch/inspect.rs`、`kivact-devtools-agent/src/{source,agent}.rs`、`kivact-test/src/screen/state.rs` | E-REQ-5 は E08 の `Reader::record_tree_changes`・`Reader::take_tree_changes`（E30 で window 付き）、E-REQ-6 は E30。C06 の手順の `Writer::record_tree_changes` は `Reader::` と読む（INC-02） |
| E | **E31（新設が必要）** | E | engine `host/write/{reader,writer,typed}.rs`、`contract.rs`・`contract/events.rs`、flags と layout の報告の model の file、`HostEvent` の新しい variant を受ける core の `match` の 1 腕（C06 の所有の外の file） | E-REQ-1〜4（`Writer::set` の InputValue の抑止、`LayoutReport`、`NodeFlags`・`watch_flags`・`NodeFlagsChanged`、`font_generation`）。INC-01 |

前提の確認：W4 の E08・E30（C06）、W2a の R04（`reported_since`）、W2b の C01、W4 の E1（E31 は E30 の後の `host/write/*`）。
tier b の compile の対象外：なし。

### CP1（W4b の後）

tier c（基本の形）と KPI（`run.sh CP1`）。

---

### W5a：公開 API の掃引 1（状態・handler・context・macro・App・editing）と engine の性能

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| 鎖 | R05 → R06 → R08 → R09 → R20 → R21 → R22 → R23 → R24 → R25 → R26 → R27 → R28 → R29 → R30 → R31 → C07 → C08 → C09 → C10 → C11 → C12 → C13 → C14 → C15 → C16 → C17 → C18 → E17 → E18 → **E20** | R・C・E | runtime・i18n・style・core・macros・facade・interaction・ui・5 kit・catalog・devtools・test・`dev/tests/`（`kivact-kit-tests` を含む）・bench・examples・templates の全体、C11/C13 の engine `embed.rs`、C13 の types `embed.rs`・`lib.rs`・arvane の embed の file（`arvane/src/embed.rs`、`arvane-shell/src/child.rs`、`arvane-native/src/{wait.rs,port/embed/*,port/win32/{create,window}.rs,port/x11/*}`）、E17/E18/E20 の editing・engine の file（`kivact-editing/src/*`、`model/{keys,apply}.rs`、`model/edit/*`、`model/pointer/{machine,state,touch,shell}.rs`、`measure/editor.rs`、`text_input.rs`、`text_input/{shaping,word}.rs`、`contract/{node.rs,contract.rs}`、`host/write/typed.rs`、`tree/node.rs`） | (1) R21 は `IntoHandler` を `E = ()` だけに実装する（INC-06）。(2) C07 の E-REQ-1〜4 は E31。(3) C10 は `kivact::view` から `Variant`・`Variants` を `#[doc(hidden)]` にする（INC-22）。(4) `prefer_gpu` と `GpuStart::Never` は C ではなく A05（INC-07）。(5) C の `W5_CHECK` の `W5_PACKAGES` に `kivact-kit-tests` を足す（INC-17）。(6) E20 を E1 の W6 から移した：E20 の前提 E19 は無視してよい（E20 の所有に E19 の file は無い）。(7) R30 と C の掃引は kit・catalog・`kivact-kit-tests` も直す（W5 は kit を compile する） |
| E1 | E15 → E16 | E | engine `host/*`（`write/{reader,writer}.rs`、`testing/{test_host,self_check}.rs`、`launch.rs`、`devtools.rs`、`foreign.rs` を含み `write/typed.rs` を除く）、`host.rs`、`model.rs`、`model/app_model.rs`、`model/window.rs`（新）、`a11y/build/{dirty,identity,node}.rs` | `src/embed.rs` を編集しない（C の E-REQ-7） |
| E2 | E12 → E13 | E | engine `paint/*`、`paint/list/{invalidation,walk}.rs`、`lower/scene_out.rs`、`model/{aggregate,states,scrollbar,transform,changes}.rs`、`model/interact/*`、`model/pointer/hover.rs`、`model/frame/ticks.rs`、`model/state_motion/*`、`model/layout_motion/steps.rs` | E14 はここに置かない（A09 の後。W6b） |

前提の確認：CP1、W2a の R04、W3a の R07・R10、W4 の C05、W4b の E31（C07 の E-REQ-1〜4）、W2a の S09（C14 の `Theme::with_fonts`）、W3a〜W4 の E の lane（E12〜E16 の前の E unit）。
tier b の compile の対象外：`kivact-docs`（XD）。

### W5b：Arvane の鎖（facade の削除・shell の縮小・`content_version`・premultiplied）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| 鎖 | A05 → A06 → A07 → A08 → A09 → A10 | A | `arvane*` の全体、`kivact-engine/` の全体（A06 の `use` の掃引）、types `gpu.rs`・`embed.rs`・`platform.rs`・`pixel.rs`・`color.rs`、core `embed/open.rs`・`platform/services.rs`・`view/foreign_window.rs`・doc、`kivact-test/src/`、`kivact-svg/src/encode.rs`、`kivact-devtools-agent/src/image.rs`、`kivact-offscreen/src/lib.rs`、`kivact-gpu-share/`、ui-basic `plot/heatmap.rs`、`dev/tests/`・producer の試験、root `Cargo.toml`（profile 1 行）、`templates/basic/README.md`、`workspace.toml`、`check-features.sh` | A05 が `prefer_gpu` を消し `GpuStart::Never` を足す（C には手順が無い）。A09 の `fresh_content_version()` は W6b の E14 が置き換える |

前提の確認：W5a の鎖の全 unit（C11・C13 の後の名前、E17・E18）、W5a の E1・E2（engine の host と paint の変更の後）、W4 の A03・A04。
tier b の compile の対象外：`kivact-docs`（XD）。

### W6a：style の鎖の前半（型の統合と改名。engine・arvane・layout・editing・gpu-share を所有する）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| 鎖 | S13 → S14 | S | W6 の掃引の範囲（types・protocol・runtime・i18n・style・shader・svg・core・interaction・ui・test・devtools・facade・offscreen・`dev/tests/kivact-public-api/tests`・bench・`examples/kivact`・`templates/basic`）と `crates/kivact-engine/`・`crates/arvane*/`・`crates/kivact-layout/`・`crates/kivact-editing/`・`crates/kivact-gpu-share/` の全体 | 鎖の共通確認の `--exclude` に `kivact-kit-tests` を足す（INC-09）。S13 が DEC-14 の除外の始まり：kit・catalog・`crates/kivact-ui-*/tests`・`kivact-kit-tests` を直さない |

前提の確認：W5a・W5b の鎖の全 unit（S 1.2 の「W5 の後の名前」）。
tier b の compile の対象外：`kivact-docs`、5 kit、5 catalog、`kivact-kit-tests`（XD＋XK）。

### W6b：style の鎖の後半と engine・Arvane の性能

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| 鎖 | S15 → S16 → S17 → C19 → S18 → S19 → S20 → S21 → S22 → S23 | S・C | W6 の掃引の範囲（上と同じ）、`kivact-layout/src/style/convert.rs`（S15）、engine `lower/conversion.rs`・`measure/leaf.rs`・`text_input/shaping.rs`・`host/write/typed.rs`・`tree/node.rs`（S22）、`arvane-text` の 2 file（S22）、core `devtools/{apply,styles,node,hooks}.rs`（C19） | C19 は S17 が `kivact_style::internal::{StyleProperty, style_property}` を作った後（INC-03。S17 に手順が足されるまで C19 は STOP する）。中核の変数は `color::TEXT_MUTED`（INC-18） |
| E1 | E19 → E21 | E | engine `model/{aggregate,app_model}.rs`、`model.rs`、`host/write/writer.rs`、`host/{pump,launch}.rs`、`embed.rs`、`frame_log.rs`、`lib.rs`、`model/pointer/*`、`model/pointer.rs` | E20 は W5a の鎖で済んでいる。E22・E23 は W7a へ移した（S22・S15 と file が重なる） |
| E2 | E14 → E24 | E | engine `lower/scene_out.rs`、`paint/*`、`paint/list/{walk,hit_reach,facts,hit}.rs`、`paint/list.rs`、`paint.rs`、`layer.rs`、`model/interact.rs`・`interact/{hit,geometry}.rs`、`model/drawables*`、`lower/shape.rs`（doc）、`lower/content_versions.rs`（A09 の新設） | E14：前提は A09（W5b）。A09 の `fresh_content_version()` を layer の版の stamp に置き換え、DevTools の overlay だけ毎 frame 新しい値のままにする（INC-04） |
| A1 | A11 | A | `arvane-compose/src/{gpu.rs,gpu/*,gpu_unbuilt.rs,lib.rs}`、`arvane/src/{presenter/gpu.rs,presenter/frame_policy.rs,test_hooks.rs}` | — |
| A2 | A12 | A | `arvane-native/`（列挙の file と `Cargo.toml`）、`arvane-shell/src/{types,window,a11y/shared}.rs`、`arvane-virtual/src/window.rs` | — |

前提の確認：W6a（S13・S14）、W5b（A09：E14、A08：A12、A10：A11）、W5a（E13：E14、E20：E21）、W4（A03：A11）。
tier b の compile の対象外：XD＋XK。

### W7a：ui の共有の土台（1 lane）と engine・Arvane の残り

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| U1 | U20 → U21 → U25 → U22 → U23 → U24 | U | `crates/kivact-ui/` の全体、interaction の toaster・announcer・combobox・table の一部、`kivact-macros`、core の `view`・`custom`、呼出し元（facade、`kivact-devtools`、`dev/tests/kivact-public-api`、`examples/kivact`、`templates`、`dev/bench`） | U24 の最後で `cargo xtask check --no-write --strict --only ui-machine,element-shadow,hand-element --crate kivact-ui` を報告する（INC-14） |
| E1 | E22 → E23 → E25 → E26 | E | engine `mutation.rs`・`host/write*`・`a11y/*`・`model/*`（列挙の file）・`lib.rs`・`tree/{node,node_tree}.rs`・`lower/assets.rs`・`paint/*`（列挙）・`Cargo.toml`、`kivact-layout/{Cargo.toml,src/*}` | E22・E23 は W6 から移した |
| A1 | A13 | A | `arvane/src/{window,presenter,presenter/*,gpu_driver,test_hooks,lib}.rs`、`arvane/tests/e2e/presenter.rs`、compose `gpu/{export,device,state}.rs`・`gpu.rs`・`lib.rs`・`gpu_unbuilt.rs` | — |
| A2 | A14 | A | `arvane-native/src/port/embed/*`・`port/win32/*`（列挙）・`port/x11/{window,thread}.rs`、`arvane-shell/src/child.rs` | Linux の file：0.6 節の WSL の確認 |

前提の確認：W6a・W6b（S の鎖の全 unit、theme 変数）、W5a（C09・C10、R の API）、W6b（E21：E22、A11：A13、A12：A14）。
tier b の compile の対象外：XD＋XK。

### W7b：interaction の破壊的な改修（4 lane）と engine・Arvane の残り

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| U1 | U30a → U30c → U30b → U31 → U33 | U | interaction の disclosure・popup・dialog・hover_card・navigation_menu・inline_edit・tooltip・menu・select・combobox・palette・part の slot・`sheet_tap.rs`、ui の対応する部品（U 2.2 の表） | U 2.2 の表が各 unit の所有より優先 |
| U2 | U32 | U | interaction `switch_drag.rs`・`swipe_dismiss.rs`・`announcer.rs`・`toaster/data.rs`、ui の toggle・toast・`dialog/drawer_swipe.rs`・empty_state・`combobox/notices.rs` | — |
| U3 | U34 → U35 | U | interaction `date_segments.rs`・`draft.rs`・`history.rs`・`month_swipe.rs`、ui の date の 4 部品・pagination・number_input・chat_composer・calendar・`date_parts/sheet.rs` | — |
| U4 | U36 | U | interaction `text_edit/policy.rs`、試験 `text_editing.rs` | `EditState` の API は E17・E20（W5a）の名前：`goal_x()`、`move_vertically(to, extend, goal_x)`、`word_at`、`line_at`。`set_goal_x` は無い（INC-05） |
| E1 | E27 → E28 | E | engine `model/changes.rs`・`interact/{geometry,sync}.rs`・`states.rs`・`apply.rs`・`focus/navigation.rs`・`state_motion/steps.rs`・`pointer/machine.rs`・`drawables/request.rs`・`check/layout_reference.rs`、layout `{columns,tree,compare}.rs` | — |
| A1 | A15 | A | compose `cpu/{compositor,filters,draw}.rs`、`arvane/src/presenter/frame_policy.rs` | — |

前提の確認：W7a（U20〜U25、E26、A13）。
tier b の compile の対象外：XD＋XK。

### W8：ui の部品を `#[component]` に（4 lane）と engine・Arvane の最後

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| U1 | U42 → U40 → U44 → U45 → U46 | U | interaction `scalar/`・`range_scalar.rs`・`splitter/`、ui の操作部品と入力（U 2.3 の U40・U44〜U46 の行） | 規則 C7 の `cargo test` は window-lock を通す |
| U2 | U41 → U47 → U48 | U | interaction の policy 3 file・`hover_card.rs`・`menu/press_state.rs`・`switch_drag.rs` の 1 行、ui の重ねて開く部品 | 同上 |
| U3 | U43 → U49 → U50 → U51 | U | interaction `reorder_drag.rs`、ui の collection とナビゲーション | 同上 |
| U4 | U52 → U53 → U54 | U | ui の表示と分野の部品 | 同上。U54 の最後で `--strict --only interaction-view,ui-machine,unit-machine,hand-element,element-shadow` を `kivact-ui`・`kivact-interaction` について報告（INC-14） |
| E1 | E29 | E | engine `Cargo.toml`・`lib.rs`・`check.rs`・`host/{devtools,pump,window_changes,events}.rs`・`host/write/{reader,writer}.rs`、core `Cargo.toml`（`devtools` の行） | — |
| A2 | A16 | A | compose `plan/{hash,builder,layers}.rs`・`Cargo.toml`・`gpu/device.rs`（1 行）、`kivact-gpu-share/`、producer `src/{vulkan,d3d12}.rs` | — |

前提の確認：W7b（U30a・U30c〜U36、A15 は A16 と別 file）、W6b（E14：A16 の版）、W5b（A09）。
tier b の compile の対象外：XD＋XK。

### CP2（W8 の後）

tier c の CP2 の形（xtask `--only local`、XD＋XK、`kivact-docs` の 2 行なし）と KPI（basic catalog の行は測らない）。

---

### W9：kit の移行 1（5 kit）と中核の追加（K6）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| K1 | K15 → K16 → K17 → K18 → K19 | K | `crates/kivact-ui-basic/`（`plot*`・`nice.rs`・`sidebar*`・`tests/consumer/plot.rs` を除く） | 手順 1 の `members` の操作はしない（全体指示 4） |
| K2 | K20 → K21 | K | `crates/kivact-ui-pop/` | 同上。`ButtonTone` は pop に残す |
| K3 | K22 → K23 → K24 → K25 | K | `crates/kivact-ui-creative/` | 同上 |
| K4 | K26 → K27 | K | `crates/kivact-ui-game/` | 同上。`DISPLAY_FAMILY_VAR` の fallback は INC-18 のとおり |
| K5 | K28 → K29 → K30 → K31 → K32 → K33 | K | `crates/kivact-ui-soft/` | 同上 |
| K6 | K06 → K07 → K08 → K09 → K10 → K11 → K12 → K13 → K14 | K | root `Cargo.toml`（`members` の 2 行）、新 `crates/kivact-plot/`・`examples/catalog-shell/`、`workspace.toml`、`Cargo.lock`、style `icon*.rs`・`lib.rs`・`facade.rs`、ui の variant の印・`syntax*.rs`・`sidebar*`・tabs・`lib.rs`・`theme.rs`、core `view/node_ref_visibility.rs`・`view.rs`、basic の `plot*`・`nice.rs`・`sidebar*`、e2e の `node_ref.rs`・`ui_sidebar.rs`・`ui_tabs_overflow.rs` と `main.rs` | K09 は `task::clock` を作らない（R23 が W5a で作った。INC-20）。K09 の所有から `crates/kivact-core/src/task.rs`・`task/clock.rs`・ViewState の file・e2e `task.rs` を外す |

前提の確認：CP2（W8 の完了）、W5a（R23 の `clock`、C09 の `#[derive(Prop)]`）、W6b（S09 の `cached_theme`、S15 の変数）、W7a〜W8（U の表）。
tier b の compile の対象外：XD＋XK（kit は W10 の終わりまで外す）。

### W10：kit の移行 2 と catalog

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| K1 | K34 → K35 | K | `crates/kivact-ui-basic/`、`examples/kivact-ui-basic/` | — |
| K2 | K36 → K37 | K | `crates/kivact-ui-pop/`、`examples/kivact-ui-pop/` | — |
| K3 | K38 | K | `crates/kivact-ui-creative/`、`examples/kivact-ui-creative/` | — |
| K4 | K39 → K40 | K | `crates/kivact-ui-game/`、`examples/kivact-ui-game/` | — |
| K5 | K41 → K42 → K43 | K | `crates/kivact-ui-soft/`、`examples/kivact-ui-soft/` | K42 の時計は `kivact::task::clock`（R23） |

前提の確認：W9 の K6（K07・K08・K09・K10・K11・K12・K13・K14）と各 kit の W9 の unit。
tier b の compile の対象外：`kivact-docs`、`kivact-kit-tests`（K44 が W11a で戻す。INC-12）。

---

### W11a：`kivact-docs` の再開、開発者の文書、kit の試験の統合

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| G1 | G01 | G | `dev/tests/docs/**` | root `Cargo.toml` は変えない（全体指示 4） |
| G3 | G13 | G | `docs/for-kivact-developer/architecture.md` | — |
| G4 | G14 | G | `docs/for-kivact-developer/{decisions,strategy,manual-verification}.md` | — |
| K | K44 | K | `dev/tests/kivact-kit-tests/`（`Cargo.toml`、`tests/e2e_kits/`）、`crates/kivact-ui-*/tests/consumer/` の「移す」file | 所有の `dev/tests/kivact-public-api/{Cargo.toml,tests/e2e_kits/}` は `dev/tests/kivact-kit-tests/` と読む。module は `apg_grid_kit`・`presence_kit`（INC-10・INC-11） |

前提の確認：W10 の全 kit の unit（K35・K37・K38・K40・K43）、W1〜W10 の完了（G01）。
tier b の compile の対象外：なし（基本の形に戻る）。

### W11b：利用者の文書（G01 の後に始める unit）

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| G1 | G02 → G09 → G12 | G | getting-started、`migrating-to-rework.md`・README の索引、README・templates・examples の文書・`crates/*/README.md` | 中核の変数は `color::TEXT_MUTED`（INC-18） |
| G2 | G03 → G06 → G08 | G | how-to の coming-from・custom-components・theming など 9 file・custom-controls | 同上 |
| G3 | G04 → G05 | G | how-to の forms〜images-and-fonts、collections〜slides-and-files | — |
| G4 | G07 → G10 → G11 | G | how-to の async〜gpu-texture-sharing、`docs/for-kivact-ui-user/**`、新 `plot.md` | `AvatarStatusTheme` は無い（INC-19） |

前提の確認：W11a（G01：G03・G04・G07、G13：G04、G14：G07）。
tier b の compile の対象外：なし。

### W11c：最終の整合

| lane | unit（順に） | 付属書 | 所有の要約 | 追加の指示 |
|---|---|---|---|---|
| G1 | G15 | G | `CONTRIBUTING.md`、`dev/README.md`、`workspace.toml` の `[rework.pending]`、`dev/xtask/src/{pending,local_checks,check_cli,main,policy}.rs` の pending の部分、AC-4 の残り | AC-4 は作業指示書 2 章の一つのコマンド（INC-13）。`kivact-docs` の `--no-default-features` は実行しない（INC-31）。pending のコードの削除は T 4.4 のとおり（INC-14） |

前提の確認：W11a・W11b の全 unit、W8 の U の `--strict` の報告、W10 の K の `--strict` の報告。
tier b の compile の対象外：なし。

### CP3（W11c の後）

tier c（除外なし）、AC-4 のコマンド、`cargo xtask check --no-write --strict`、KPI。

---

### unit → wave / lane（全 unit の索引）

| unit | wave | lane |
|---|---|---|
| T01, T02, T03, T04, T23 | W1a | T0 |
| S01 | W1a | S |
| T05, T06 | W1b | T1 |
| T07, T08 | W1b | T2 |
| T09, T10 | W1b | T3 |
| T11, T12 | W1b | T4 |
| T13, T14, T15 | W1b | T5 |
| T16, T17, T18, T19, T20, T21 | W1b | T6 |
| T22 | W1c | T0 |
| S02, S03, S04, S05, S06 | W2a | A |
| S07, S08, S09 | W2a | S |
| R01, R02, R03, R04 | W2a | R |
| U01, U02, U03 | W2a | U |
| K01, K02, K03, K04, K05 | W2a | K |
| C01, C02 | W2b | C |
| S10, S11, S12 | W3a | S |
| R07, R10 | W3a | R |
| C03, C04 | W3a | C |
| U04, U05, U06, U07, U08 | W3a | U |
| A01 | W3a | A1 |
| A02 | W3a | A2 |
| E01, E02 | W3b | E1 |
| E03, E04 | W3b | E2 |
| C05 | W4 | C |
| U09, U10, U11, U12, U13, U14 | W4 | U |
| E05, E06, E07, E08, E09, （E30 新設） | W4 | E1 |
| E10, E11 | W4 | E2 |
| A03 | W4 | A1 |
| A04 | W4 | A2 |
| C06 | W4b | C |
| （E31 新設） | W4b | E |
| R05, R06, R08, R09, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31 | W5a | 鎖 |
| C07, C08, C09, C10, C11, C12, C13, C14, C15, C16, C17, C18 | W5a | 鎖 |
| E17, E18, E20 | W5a | 鎖 |
| E15, E16 | W5a | E1 |
| E12, E13 | W5a | E2 |
| A05, A06, A07, A08, A09, A10 | W5b | 鎖 |
| S13, S14 | W6a | 鎖 |
| S15, S16, S17, C19, S18, S19, S20, S21, S22, S23 | W6b | 鎖 |
| E19, E21 | W6b | E1 |
| E14, E24 | W6b | E2 |
| A11 | W6b | A1 |
| A12 | W6b | A2 |
| U20, U21, U25, U22, U23, U24 | W7a | U1 |
| E22, E23, E25, E26 | W7a | E1 |
| A13 | W7a | A1 |
| A14 | W7a | A2 |
| U30a, U30c, U30b, U31, U33 | W7b | U1 |
| U32 | W7b | U2 |
| U34, U35 | W7b | U3 |
| U36 | W7b | U4 |
| E27, E28 | W7b | E1 |
| A15 | W7b | A1 |
| U42, U40, U44, U45, U46 | W8 | U1 |
| U41, U47, U48 | W8 | U2 |
| U43, U49, U50, U51 | W8 | U3 |
| U52, U53, U54 | W8 | U4 |
| E29 | W8 | E1 |
| A16 | W8 | A2 |
| K15, K16, K17, K18, K19 | W9 | K1 |
| K20, K21 | W9 | K2 |
| K22, K23, K24, K25 | W9 | K3 |
| K26, K27 | W9 | K4 |
| K28, K29, K30, K31, K32, K33 | W9 | K5 |
| K06, K07, K08, K09, K10, K11, K12, K13, K14 | W9 | K6 |
| K34, K35 | W10 | K1 |
| K36, K37 | W10 | K2 |
| K38 | W10 | K3 |
| K39, K40 | W10 | K4 |
| K41, K42, K43 | W10 | K5 |
| G01 | W11a | G1 |
| G13 | W11a | G3 |
| G14 | W11a | G4 |
| K44 | W11a | K |
| G02, G09, G12 | W11b | G1 |
| G03, G06, G08 | W11b | G2 |
| G04, G05 | W11b | G3 |
| G07, G10, G11 | W11b | G4 |
| G15 | W11c | G1 |

数：T 23、S 23、R 22、C 19、E 29（＋提案の E30・E31）、A 16、U 43、K 44、G 15。合計 234（＋2）。wave（sub-wave を一つずつ数え、checkpoint を除く）は 21：W1a・W1b・W1c・W2a・W2b・W3a・W3b・W4・W4b・W5a・W5b・W6a・W6b・W7a・W7b・W8・W9・W10・W11a・W11b・W11c。最大の lane の数は 6（W1b、W3a、W4、W7b、W8、W9）。

### 変更した配置（付属書の最初の表との違い）

| unit | 付属書の配置 | この表 | 理由 |
|---|---|---|---|
| C01、C02 | W2 / C | W2b / C | C01 と S04 が `crates/kivact-core/src/launch/runtime.rs`、C01 の `devtools/` と R04 の `devtools/console.rs`、C02 と S02 が root `Cargo.toml`・`workspace.toml`、C02 と S07 が core・kivact の `Cargo.toml` を同じ wave に持つ（INC-23） |
| S10〜S12、R07、R10、C03、C04、U04〜U08、A01、A02 | W3 | W3a | W3 の lane が 8 本（S・R・C・U・E1・E2・A1・A2）で 6 を超える（INC-25） |
| E01〜E04 | W3 / E1・E2 | W3b | 6 lane の上限と、E01 と S10 の `crates/kivact-types/src/lib.rs`（INC-24） |
| C06 | W4 / C | W4b / C | E-REQ-5 の E08 が W4 の E1 にあり、同じ wave の別 lane に頼れない（INC-26）。E-REQ-6 は付属書 E に無く E30 が要る（INC-01） |
| E30、E31（新設の提案） | 無い | W4 / E1、W4b / E | C06・C07 が求める E-REQ-1〜4・6 を提供する unit が付属書 E に無い（INC-01） |
| R05〜R31、C07〜C18、E17、E18 | W5 / 鎖 | W5a / 鎖 | 親の順のまま。A05〜A10 だけを W5b に分けた |
| E20 | W6 / E1 | W5a / 鎖（E18 の後） | W6b では S22 と S の掃引が `text_editing.rs` を持ち、W7a は U22、W7b は U36 が試験を持つ。U36 は E20 の `word_at` を「W5 の名前」として使う（INC-05・INC-28） |
| A05〜A10 | W5 / 鎖 | W5b / 鎖 | A05・A07・A08・A09・A06 が E15（`host/*`）・E12・E14（`lower/scene_out.rs`）と engine の file を共有する（INC-27） |
| E14 | W5 / E2 | W6b / E2（E24 の前） | A09 が `content_version` を足した後に版を安定させる（親の決定、INC-04） |
| S13、S14 | W6 / 鎖 | W6a / 鎖 | S14 が engine・arvane・layout・editing・gpu-share の全体を所有するので、E・A の lane と同じ wave に置けない（INC-28） |
| S15〜S23、C19 | W6 / 鎖 | W6b / 鎖 | C19 は S17（宣言の表）の直後（C 4.1） |
| E19、E21、E24、A11、A12 | W6 | W6b | S13・S14 の後。S15〜S23 と file が重ならない |
| E22、E23 | W6 / E1 | W7a / E1 | E22 は `host/write/typed.rs` を S22 と、E23 は `kivact-layout` を S15（`style/convert.rs`）と共有する（INC-28） |
| E25、E26 | W7 / E1 | W7a / E1 | — |
| E27、E28 | W7 / E1 | W7b / E1 | W7a の lane を短くする（同じ lane の順は変えない） |
| A13、A14 | W7 / A1・A2 | W7a / A1・A2 | W7b は U の 4 lane で、残りは 2 lane（INC-29） |
| A15 | W8 / A1 | W7b / A1 | W7b を 6 lane にし、性能の unit を CP2 の前に早く終える |
| A16 | W8 / A2 | W8 / A2 | 変更なし |
| K44 | W11 / K | W11a / K | W10 の tier b は `kivact-kit-tests` を外す（INC-12） |
| G01、G13、G14 | W11a | W11a | 変更なし（G01 を単独の段にした） |
| G02〜G12（G13・G14 を除く） | W11a | W11b | G03・G04・G07 が同じ wave の別 lane の G01 を前提にする（INC-30） |
| G15 | W11b | W11c | 上の分割に合わせた |

満たせなかった制約：なし（ただし C06・C07・C19 は INC-01〜03 の付属書の修正が前提。修正が入らなければこの 3 unit は STOP になる）。


---

## 5. checkpoint

### 5.1 CP1（W4b の後）・CP2（W8 の後）

1. 2 章の **tier c** を実行する（compile の対象外は 2 章の表のとおり）。失敗した試験は、`rework/baseline-tests.txt` と `rework/OPEN.md` の「基点の失敗」と照らし、改修による失敗だけを、その試験の領域の付属書の最後の unit の lane に fix として回す（1.3 節の fix の手順）。
2. KPI を測る：`dev/tools/window-lock.sh --timeout 5400 -- dev/bench/run.sh CP<n>`。`rework/kpi/CP<n>.md` に写し、設計書 2 章の表の各行の比（CP<n> / B0）を `rework/kpi/CP<n>-ratio.md` に表で書く。CP2 では、kit の catalog の行（basic catalog）は compile の対象外なので測らない（`run.sh` がその行を INVALID にしたら、その行だけ除いて記録する）。
3. 必達（設計書 2 章）を満たさない行は、同じ条件で 2 回測り直し、3 回中 2 回で判定する。それでも満たさない行は `rework/OPEN.md` の「KPI の後退」節に書き、その行の「主な手段」の所見の unit を担当した付属書の領域に差戻しとして記録する（Orchestrator は直さない）。
4. push する。

### 5.2 CP3（最終、W11c の後）

1. 2 章の tier c を、compile の対象外なしで実行する。
2. 設計書 5.12 節の削除名の検査（2 章の「AC-4 の検査」のコマンド）を実行し、0 件であることを確かめる。
3. xtask の `state` 検査が全 workspace で 0 件（`cargo xtask check --no-write`）であることを確かめる（AC-3）。
4. KPI を測る（`run.sh CP3`）。全行の比を `rework/kpi/CP3-ratio.md` に書く。必達を満たさない行は 5.1 節の 3 と同じに扱う。
5. 受入れの判定：AC-1〜AC-5 をすべて満たし、`rework/OPEN.md` の「差戻し」と「KPI の後退」が空なら完了。そうでなければ、残りを書いた `rework/OPEN.md` を commit し、push して人間に報告する。
6. 完了したら PR の説明の下書きを `rework/PR_DRAFT.md` に書く：目的、変更の要約（領域ごと 3 行）、公開 API の破壊的変更の一覧（移行ガイドへの link）、KPI の比の表、実施した検証、未実施の検証とその理由、残る制限（設計書 11 章）。PR は作らない。

`REWORK_DESIGN.md`、`REWORK_WORK_ORDER.md`、`rework/` は PR が merge されるまで残す（PR の review の材料）。merge の後に人間が消す。
