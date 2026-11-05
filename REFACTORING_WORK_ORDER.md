# 作業手順書: Kivact 基盤再設計（Foundation v3）

- 発行: 2026-11-04（Claude Code が作成）。2026-11-05 改訂（W2 以降を最終の形に書き直した）
- 実装担当: ChatGPT Codex（主エージェント = **オーケストレータ**）と、オーケストレータが起動する**サブエージェント**
- 設計: [`REFACTORING_DESIGN.md`](REFACTORING_DESIGN.md)（以下「設計書」）、付録 [`refactoring/A-state-machines.md`](refactoring/A-state-machines.md)（付録 A）、[`refactoring/B-public-api.md`](refactoring/B-public-api.md)（付録 B）、[`refactoring/C-kits.md`](refactoring/C-kits.md)（付録 C）
- 起点: `main` @ `a3536ef`

---

本書は **順序・作業の単位（unit）・各 unit の手順・検査・完了条件** を定める。**何をどの形にするか** は設計書が決める。本書と設計書が矛盾したら設計書に従い、本書を直す。

**作業の速度を優先する**（作者の方針）。サブエージェントは同時に 6 つまで使う。integration branch は最後に作者が `main` への PR を立てて squash merge するので、途中の commit と途中の integration branch で build が壊れていてよい。

> **2026-11-05 の改訂（W0・W1 の実績から）**
>
> W0・W1（P・V）に 5 時間かかり、W2 に入れなかった。原因と対策:
>
> | 原因 | 対策（節） |
> |---|---|
> | CP1 の trace の厳密な比較（f32→f64 で全数値が変わるのに一致を求め、2 時間以上） | trace の比較を廃止する。合否は E2E と静的検査の終了コードだけ（0.6 節） |
> | 毎回の重い gate（約 30 分）を修正のたびに全部回し直した | 軽い gate と重い gate に分ける。失敗した試験だけを回し直す。重い部分は裏で回し、次の wave を待たせない（0.6 節） |
> | 記録（台帳、証拠の bundle、review の文書、比較の script）に時間を使った | 記録は表の一行ずつ。証拠の file を作らない（0.5 節） |
> | 次の wave の読むだけの調査と、独立の review のサブエージェント | どちらも行わない（0.11 節） |
> | 設計が現状の code に対して細かさが足りず、調査と判断で止まった | 設計書 19 章で W2 以降の不足を決め、unit の節を最終の形に書き直した（決め方の既定は 0.11 節） |
> | disk の空きが少なく、build の並列を 2〜4 に絞った | target dir を三つに限る（0.3 節）。作者が disk の空きを確保する |
>
> **再開の手順:**
>
> 1. CP1 は **通過** とする（`97689c4` で fmt・Clippy・xtask・feature の検査・全 feature と no-default の E2E が全部 exit 0。trace の比較は廃止）。`PROGRESS.md` はこの改訂で書き直してある。
> 2. `target/foundation-v3/`（trace と log の保存）を消す。`target/kivact-trace/` があれば消す。
> 3. 0.3 節の target dir と環境変数を設定する。
> 4. 1 章の表の W2 から進める。

---

## 目次

0. [進め方の規則](#0-進め方の規則)
1. [全体の順序](#1-全体の順序wave)
2. [Phase P: 準備（完了）](#2-phase-p-準備完了)
3. [Phase V: 語彙（完了）](#3-phase-v-語彙完了)
4. [Phase A: Arvane](#4-phase-a-arvane)
5. [Phase E: エンジン](#5-phase-e-エンジン)
6. [Phase R: ランタイム](#6-phase-r-ランタイム)
7. [Phase C: 中核](#7-phase-c-中核)
8. [Phase N: 公開 API](#8-phase-n-公開-api)
9. [Phase I・U: 振る舞いと標準部品](#9-phase-iu-振る舞いと標準部品)
9.5 [Phase T・G・KL: 拡張口・不足・kit の中の状態](#95-phase-tgkl-拡張口不足kit-の中の状態)
10. [Phase K: 公式 kit と検査の有効化](#10-phase-k-公式-kit)
10.5 [Phase X: 横断の整理](#105-phase-x-横断の整理)
11. [Phase F: 仕上げ](#11-phase-f-仕上げ)
12. [完了条件](#12-完了条件)
13. [unit の一覧](#13-unit-の一覧)

---

## 0. 進め方の規則

### 0.1 役割

| 役割 | する | しない |
|---|---|---|
| オーケストレータ（Codex 主エージェント） | 1 章の順に unit を進める。unit ごとに worktree を作り、サブエージェントに 0.4 節の形で指示する。差分を点検し（`git diff --stat` と要所の読み）、integration branch へ merge し、gate を回し、push する。設計書と本書にない判断を 0.11 節の既定で決める。`refactoring/PROGRESS.md` を更新する | 設計書の決定を変える（変える必要が分かったら 0.11 節）。次の wave の先回りの調査。独立の review のサブエージェント。PR を作る・`main` へ merge する・`main` へ push する（作者が行う） |
| サブエージェント | 指示された unit の手順を、指示された worktree の中だけで実行する。0.6 節の「unit の確認」だけを回して報告する | 公開の名前・公開の型と署名・振る舞いを指示と違えて決める（食い違ったら止めて報告する）。指示の範囲の外の file を変える。互換の wrapper・alias・旧 API を残す。push する |

作者は判断をオーケストレータに任せている（2026-11-04）。作者には聞かず、0.11 節で決めて進む。

### 0.2 守る規則

- `CONTRIBUTING.md` の全部（module の `//! Role:` 行、`expect("invariant: ..")`、lint の `reason`、glob import の禁止、panic を隔離しない、など）。
- **互換を残さない。** 旧い名前・旧い経路・二重の mode・「呼び出し元が壊れないように」の既定の実装を作らない。移行のための一時的な二重の経路も作らない（途中で build が壊れてよい）。workspace の中の呼び出し元は、同じ unit か 0.10 節の追従の段で新しい形に移し、その wave の checkpoint までに旧い形を残さない。
- **試験は実 window の E2E だけ**（`kivact::test`）。過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden を足さない。例外は `dev/xtask` の検査の自分の試験だけ。
- **状態機械の unit は振る舞いを変えない**（設計書 16.2 節）。振る舞いを意図して変える unit（13 章の表の「振る舞いの変更」の列）だけが E2E を足す。
- **既存の試験の期待値:** 既存の試験が失敗したら、まず code を直す。f64 化による数値の端数だけが原因で（差が 0.001 論理 px 未満、または相対 1e-9 未満）、code を安定した値にする簡単な直し方がないときに限り、期待値を新しい値に直し、`DECISIONS.md` に一行書く。振る舞いの assertion を弱めない。
- window を開くコマンド（mirror を有効にした `cargo test`）は必ず `bash dev/tools/window-lock.sh [--timeout 秒] -- <command>` を通す。`KIVACT_TEST_MIRROR=off` の `cargo test`（A-1b の後）は window を開かないので lock を通さない。終了コード 124 は時間切れ。出力が空でも終了コードを確かめる。
- **native 試験の lock の例外**: `MIRROR=off` でも `TestParent::open` を使う embed と native smoke は実窓を開く。これらを含む public-api binary と全 workspace の試験も window-lock を通す。
- GUI の OS 入力の harness、実機の IME・screen reader・browser の手動の確認は作者が行う。エージェントは行わない。
- commit は conventional commits で目的ごとに分ける。公開 API を壊す commit は `!` を付ける（例: `refactor(core)!: ...`）。commit が build できる状態である必要はない。
- `vendor/xtask-core` は変えない（検査は `dev/xtask` に足す）。
- CI（`.github/workflows/`）は既定で無効にしてあり、変えない。CI に job・step・設定を足さない。

### 0.3 branch・worktree・build の資源

- integration branch: `refactor/kashiwade/foundation-v3`。checkout は `C:/w/kashiwade/kivact`（以下「integration の checkout」）。全 unit の merge 先。checkpoint の軽い gate を通したら origin へ push する（force push はしない）。全部が終わったら、F-3 で PR の説明の下書きを書き、push して作者に知らせ、止まる。**`main` への PR と squash merge は作者が行う。**
- unit ごとに worktree を作る:
  ```text
  git worktree add .claude/worktrees/<unit-id> -b wt/<unit-id> refactor/kashiwade/foundation-v3
  ```
  サブエージェントにはその絶対 path を渡す。`wt/*` は push しない。merge したら worktree と branch を消す。
- **同時に動かすサブエージェントは最大 6 つ。** 1 章の表のとおりに並べる。一つの unit が終わったら、その枠にすぐ次の unit を入れる（`{..}` の組の全部の終わりを待たない。待つのは `→` の前の unit の merge だけ）。
- merge は `git merge wt/<unit-id>`（fast-forward でよい）。unit が終わったらすぐ merge する。
- **生成物を unit で作らない:** サブエージェントは `architecture/` を書き換えない（`cargo xtask check` は `--no-write` で回す）。`architecture/` は重い gate でオーケストレータが作り直す。
- **target dir は三つだけ**（`/target-*/` は `.gitignore` 済み）:

  | 使う所 | target dir |
  |---|---|
  | integration の checkout（merge の後の `cargo check`・clippy、軽い gate） | `C:/w/kashiwade/kivact/target` |
  | 全部の unit の worktree（共有） | `CARGO_TARGET_DIR=C:/w/kashiwade/kivact/target-wt` |
  | gate の worktree（0.6 節の裏で回す試験） | `CARGO_TARGET_DIR=C:/w/kashiwade/kivact/target-gate` |

  unit の worktree が target dir を共有すると、同時の `cargo` は lock で順番待ちになる。サブエージェントの `cargo check` は短いので、これを受け入れる（disk と依存の build を節約する方が速い）。
- **環境変数**（全部の cargo に）: `CARGO_PROFILE_DEV_DEBUG=0`、`CARGO_PROFILE_TEST_DEBUG=0`（debug 情報を作らない。link と disk が軽くなる）。`CARGO_INCREMENTAL` は既定（有効）。`CARGO_BUILD_JOBS` は既定（全 core）。`XTASK_NODE` は CP0 で決めた bundled の node。`RUSTC_WRAPPER=sccache` は入っていれば使う。
- **disk:** 作業の前に C: の空きが 100 GB 以上あることを確かめる（作者が確保する）。checkpoint ごとに空きを見て、40 GB を切ったら `target-wt` を `cargo clean` する。
- サブエージェントは `cargo build` を使わず `cargo check` を使う（試験を回す unit だけ `cargo test`）。

### 0.4 サブエージェントへの指示の型

次の形で書く（unit の節をほぼそのまま写す）。unit の節の「決定済みの形」は指示に直接写す（サブエージェントが設計書を通して読まなくてよいように）。

```text
あなたは Kivact の実装担当です。公開の名前・公開の型と署名・振る舞いは指示のとおりにし、指示と code が食い違ったら作業を止めて報告してください。private な名前・file の分け方・内部の補助の型は、0.11 節の既定で自分で決めて報告に書いてください。

- worktree: <絶対 path>（この中だけを変更する。branch は wt/<unit-id>）
- unit: <unit-id> <名前>
- 環境: CARGO_TARGET_DIR=C:/w/kashiwade/kivact/target-wt、CARGO_PROFILE_DEV_DEBUG=0、CARGO_PROFILE_TEST_DEBUG=0
- 読む: REFACTORING_WORK_ORDER.md の <unit の節>（必要なら設計書の <節>）
- 変更する crate・file: <列挙>
- 変更しない: <同時に動くほかの unit の file>
- 決定済みの形: <unit の節から写す>
- 手順: <unit の節から写す>
- 状態の点検表: 0.7 節（状態機械の unit のとき）
- 試験: <足す E2E の名前と確かめる内容。足さないなら「足さない」>
- 規則: CONTRIBUTING.md、互換を残さない、push しない、`architecture/` を書き換えない、記録の file（PROGRESS・DECISIONS・付録）を書き換えない
- build の期待: <「build 可」か「build 不可」>
- 完了の前に回す確認: 0.6 節の「unit の確認」（<crate の列>、<試験の名前>）
- 報告: 0.8 節の形
```

### 0.5 記録

記録は少なく、表の一行ずつにする。オーケストレータだけが書く。

- `refactoring/PROGRESS.md`: (1) unit の表（ID、状態 `todo`/`doing`/`merged`、merge の commit、一言の残件）、(2) checkpoint の表（CP、commit、各コマンドの終了コード、試験の件数、時間）。一つの checkpoint に添える文は 3 行まで。
- `refactoring/DECISIONS.md`: 0.11 節で決めたことのうち、後の unit が知るべきものだけを、表の一行（日付、unit、問い、決定と理由）で書く。決定が設計書・本書・付録の記述を変えるときは、その記述を直接直し、ここには一行だけ書く（後の読み手が DECISIONS を読まなくてよいように）。
- **作らないもの:** 台帳、証拠の bundle、review の文書、比較や集計の script、log の要約、hash の manifest、次の wave の調査の packet。log は `target/` に置いて commit しない。
- 付録 A にない暗黙の状態機械を見つけたら、設計書 13.2 節の「解消の型」で解消し、付録 A に一行足す。

全 unit が終わったら、長く使う規則と根拠を F-1 で `docs/for-kivact-developer/architecture.md` と `CONTRIBUTING.md` に移してから、`refactoring/` と設計書・本書を消す（F-3）。

### 0.6 確認と gate

合否は **コマンドの終了コードだけ** で決める。trace の比較、試験の件数の照合、数値の同値性の review は行わない。

**unit の確認**（サブエージェントが自分の worktree で、報告の前に回す）:

```text
cargo fmt -p <crate> [-p <crate> ...]                                   # 書き換える
cargo check -p <crate> [-p <crate> ...] --all-targets --all-features   # 指示が「build 可」のときだけ
KIVACT_TEST_MIRROR=off cargo test -p <crate> <試験の名前>               # 試験を足した・回すと指示された unit だけ（A-1b の後）
cargo xtask check --no-write --only state-fields --crate <crate> --module <module path>   # 状態機械の unit だけ
```

- 「build 不可」の unit（fan-out の並列の一つ、定義の段など）は `cargo fmt` だけを回す。
- A-1b より前に試験を足した unit は、試験を書くだけにして回さない。
- clippy は unit では回さない。

**merge の後**（オーケストレータが integration の checkout で、`→` の区切りごとに）: `cargo check --workspace --all-targets --all-features`。error は crate の組（0.10 節）ごとにまとめて fix の unit（`fix/<組>-<n>`）に配り、並べて直させる。次の `→` の unit は、自分の触る crate が check を通っていれば始めてよい。

**軽い gate**（各 wave の終わり。1 章の CPn）:

```text
cargo fmt --all                                                  # 書き換えて commit
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

これが通ったら push し、次の wave を始める。同時に、gate の worktree で全 E2E を裏で回す:

```text
git -C .claude/worktrees/gate checkout --detach <CP の commit>      # 初回は git worktree add --detach .claude/worktrees/gate <commit>
# gate の worktree で、CARGO_TARGET_DIR=C:/w/kashiwade/kivact/target-gate
KIVACT_TEST_MIRROR=off cargo test --workspace --all-features --no-fail-fast --exclude <GPU の crate>   # A-1b より前は window-lock.sh を通す
KIVACT_TEST_MIRROR=off cargo test -p <GPU の crate> --all-features -- --test-threads=1
```

- `<GPU の crate>` は `gpu-interop-producer`（`dev/tests/gpu-interop-producer/tests/read_back.rs` が並列だと heap の破損で落ちる。`DECISIONS.md` の CP0 の行）。それ以外は並列（既定の thread 数）で回す。並列で別の試験が落ちたら、その crate も `--test-threads=1` の側へ移し、`DECISIONS.md` に一行書く。
- **失敗したとき:** 失敗を crate の組ごとにまとめて fix の unit に配る。fix の unit は次の wave の枠を優先して使う。直したら、**落ちた試験の binary だけ** を回し直す（`cargo test -p <crate> --test <binary> <名前>`）。全体の回し直しは次の checkpoint の E2E に任せる。前の checkpoint の E2E の失敗が全部直るまで、次の checkpoint を通過にしない。
- 原因が 15 分で分からない試験は、`KIVACT_TRACE=stage,semantics,host_events` でその試験だけを今と前の checkpoint の commit で回し、差を目で見てよい（調べるための道具。比較の file を残さない）。

**重い gate**（CP2・CP4 と F-3）: 軽い gate と裏の E2E に加えて、gate の worktree で順に回す（次の wave は待たない）:

```text
KIVACT_TEST_MIRROR=off cargo test --workspace --no-default-features --no-fail-fast
bash dev/tools/check-features.sh
cargo xtask check                      # architecture/ を作り直す。integration の checkout で回して commit する
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact/.claude/worktrees/gate && CARGO_TARGET_DIR=$HOME/kivact-target cargo clippy --workspace --all-targets --all-features -- -D warnings'
```

F-3 では、さらに mirror を有効にした試験（`bash dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --all-features`）と、WSL の試験（Wayland と X11 を分けて記録）を回す:

```text
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target WAYLAND_DISPLAY=wayland-0 cargo test --workspace --all-features'
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target WAYLAND_DISPLAY= DISPLAY=:0 cargo test --workspace --all-features'
```

（WSL の distro を聞かれたら `KIVACT_WSL_DISTRO=Ubuntu-24.04`。）

本書の unit の節の「unit の確認」は上の「unit の確認」、「checkpoint」はその unit を含む wave の終わりの gate を指す。**G-final** は 12 章（F-3 で回す）。

### 0.7 状態の点検表（状態機械を扱う unit の完了の前に、サブエージェントが確かめて報告に書く）

変えた型・module のそれぞれについて:

1. 状態の field に `bool`・`Cell<bool>`・`AtomicBool`・`Signal<bool>`・`Var<bool>` が残っていないか（option の束と、単独の事実の bool を除く。設計書 18.14 節）。
2. `Some`/`None` が別の field に依存する `Option` が残っていないか（S-3）。
3. 同じ事実を二つの場所に持っていないか。派生できる値を保存していないか（S-4）。
4. effect が別の signal を後から直していないか（S-5）。
5. 「X の前に Y」の呼び出し順の要求が doc に残っていないか（S-6）。
6. 後から一度だけ埋める field が残っていないか（S-7。設計書 19 章が認めた構築の中の `OnceCell` を除く）。
7. counter を event として使っていないか（S-8）。
8. 例外として残したものは S-9 の名前の付いた型か（`ReportOnce`、`Sent<T>`、`Generation`、`Mailbox<T>`、cache、統計）。
9. 付録 A のその unit の項目の「解消後」の型になっているか。違う形にしたなら、その理由（code の事実）。
10. 付録 A にない暗黙の状態機械を見つけたか（見つけたら場所と、解消したかどうか）。

### 0.8 サブエージェントの報告の形

短く書く（変更の一覧は `git diff` で分かるので書かない）。

```text
unit: <id>
結果: 完了 / 止めた（理由）
確認: <回したコマンドと終了コード、試験の件数>
状態の点検表: <状態機械の unit だけ。0.7 節の 1〜10 を「OK」か内容で一行ずつ>
自分で決めたこと・見つけたこと: <なし / 一覧（0.11 節で決めた private なこと、付録 A にない状態機械、ほかの unit に影響すること）>
```

### 0.9 crate の作成・統合・削除のときに一緒に直すもの

crate を作る・統合する・消す unit は、次を全部直す（大きな unit は 0.10 節の追従の段に分けてよい。その wave の checkpoint までに揃える）。

- root の `Cargo.toml` の `members` と、`[profile.dev.package.<name>]`（engine 側の crate は opt-level 3 を付け直す。`kivact-runtime` は付けない: debug のまま）
- `dev/xtask/policy/workspace.toml` の `visible_to`、`module_rules`、`api_packages`、`options_structs`、`[[style.exceptions]]` の path
- `dev/tools/check-features.sh` の feature の一覧
- 各 crate の `Cargo.toml` の依存と feature の伝播（DEP-1: feature の名前を全層で揃える）
- `THIRD-PARTY-NOTICES.md`（依存が移る場合）
- README・README_JP・`docs/` の crate 名の言及
- `architecture/` は unit では直さない（重い gate で作り直す）

### 0.10 速く進めるための分け方

**fan-out（定義の段と追従の段）:** 多くの crate にまたがる変更（型の移動、改名、公開 API の形の変更）は、一つの unit を二段に分ける。

1. **定義の段**（サブエージェント一つ、または定義が別々の file にあれば複数）: 型・trait・関数の定義を変える。呼び出し元は直さない（build 不可でよい）。
2. **追従の段**（下の crate の組ごとに一つ、最大 6 つを並べる）: それぞれの組の中の呼び出し元を、新しい定義に合わせて直す。並べている間は互いの結果を待たない（build 不可でよい）。全部を merge したら、オーケストレータが `cargo check --workspace --all-targets --all-features` を回し、残った error を組ごとの fix の unit に配る。

**crate の組**（追従の段と fix の分け方）:

| 組 | crate |
|---|---|
| G1 engine | `kivact-engine`、`kivact-layout`、`kivact-lower`（`kivact-text` は統合まで） |
| G2 arvane | `arvane`、`arvane-shell`、`arvane-native`、`arvane-render`、`arvane-text`、`arvane-asset`、`arvane-virtual`、`kivact-gpu-share`、`gpu-interop-producer` |
| G3 core | `kivact-types`、`kivact-runtime`（統合までは reactive・task・motion）、`kivact-style`、`kivact-i18n`、`kivact-editing`、`kivact-core`、`kivact-test` |
| G4 ui | `kivact-interaction`、`kivact-ui`、`kivact`（facade） |
| G5 kit | 5 つの kit（kit ごとに分けてよい: 一つの kit を一つのサブエージェントに） |
| G6 外側 | `examples/`、catalog、`templates/`、`dev/tests/`（public-api、docs）、`docs/`、README |

**公開の署名を固定して並べる:** 同じ crate の中の別々の module を並べるとき（E-2〜E-7、I-1〜I-6 など）、各 unit は、ほかの unit が使う公開の型・メソッドの署名を変えない。署名を変える必要が分かったら、作業を止めて報告する。

**file の持ち主:** 並べる unit の指示には「変更しない」の欄に、同時に動くほかの unit の file を書く。共有の file（各 crate の `lib.rs` の `mod`・`pub use`、`dev/xtask/policy/workspace.toml`、`Cargo.toml` の `members`）は足すだけにし、merge の衝突はオーケストレータが手で合わせる。

### 0.11 判断の既定（止まらないために）

設計書・本書・付録で決まっていないことは、オーケストレータ（private なことはサブエージェント）が次の順の既定で決める。一つの問いに使う調査は 15 分まで。決めたら、後の unit が知るべきものだけを `DECISIONS.md` に一行書いて進む。全体を止めない。

1. 現在の振る舞いを保つ形にする。
2. 変更の最も小さい形にする。
3. 新しい公開の名前・型を足さない（足すしかないときは付録 B.10 と設計書 3.4 節の API の規則に合う名前）。
4. 設計書の規則（S・API・VOC・DEP・KIT）と衝突するときは、衝突を最小にする形を選び、設計書の該当の節に一行の例外を書く。
5. 設計書の形が code の事実と合わないとき（型が作れない、依存が循環する、振る舞いが変わる）は、設計書の目的（節の最初の文）を満たす最小の形に直し、設計書の記述を直接直す。

**行わないこと:** 次の wave の先回りの調査、独立の review のサブエージェント、証拠の file、一つの unit の中での同じ gate の全体の回し直し。

---

## 1. 全体の順序（wave）

同時に動かすサブエージェントは最大 6 つ。`←` の右は merge を待つ相手（書いていない相手は待たない）。一つの unit が終わったら、待つ相手が merge 済みの unit をすぐ次の枠に入れる。fix の unit（0.6 節）は空いた枠を優先して使う。

| wave | 状態 | 終わりの gate |
|---|---|---|
| W0 準備（P-0、P-1、P-2） | 完了（CP0 通過） | — |
| W1 語彙（V-1a〜V-6） | 完了（CP1 通過。2026-11-05 の改訂の基準） | — |
| W2 下層と runtime（A、E、R、X） | 次 | CP2（重い） |
| W3 中核（C、L-1、KL） | | CP3（軽い） |
| W4 公開 API（N） | | CP4（重い） |
| W5 振る舞いと拡張口（I、U、T、G） | | CP5（軽い） |
| W6 kit と検査の有効化（K、H） | | CP6（軽い） |
| W7 仕上げ（F） | | F-3（G-final） |

### W2

```text
R-1
R-2  ← R-1
R-3  ← R-1
R-6  ← R-1
R-4, R-5 ← R-2, R-3
A-0
A-1a ← A-0
A-1b ← A-1a, R-2
A-2  ← A-1b
A-3  ← A-2
A-4  ← A-0
A-5  ← A-1a
E-1
X-1  ← E-1, A-1b, A-2, A-3, A-4, A-5
A-6a ← X-1
A-6b ← A-6a
E-2  ← X-1, R-2
E-3  ← X-1, R-2
E-4, E-5a, E-5b, E-7 ← X-1
E-6  ← X-1, A-3
E-8  ← E-3, E-5a, E-6, E-7
E-9a ← E-5b, E-8
E-9b ← E-5b, R-3
```

枠の例: `{R-1, A-0, E-1}` → `{R-2, R-3, R-6, A-1a, A-4}` → `{A-1b, A-5, R-4, R-5}` → A-2 → A-3 → X-1 → `{E-2, E-3, E-4, E-5a, E-5b, A-6a}` → `{E-6, E-7, E-9b, A-6b}` → E-8 → E-9a → CP2。

- R の unit は A・E の file に触らない。例外は R-2 の core の `client/` と `test/screen/driver.rs`（上の `←` で順を決めた）。
- A-2 と R-5 が core の `platform/services.rs`・`view/requests.rs` を両方触るときは、merge で手で合わせる。

### W3

```text
C-1
C-2, L-1, C-3a, C-3b ← C-1
C-4  ← C-3a
C-5  ← C-4
KL-basic, KL-game, KL-soft   （R-6 の後、W3 の空いた枠。KL-creative は I-3 merge 後の W5。KL-pop は項目がない）
```

- C-5 が W3 の終わりに間に合わなければ、CP4 の後（W5 の空いた枠）に回してよい（Phase N は C-5 に依らない）。W4 の間は動かさない（N-F1 と同じ core の file を触る）。
- KL の unit は kit の file だけを触る。C の unit は kit を触らない。

### W4

```text
N-D1, N-D2                                  （定義の段。build 不可でよい）
N-F1, N-F2, N-F3, N-F4, N-F5, N-F6 ← N-D1, N-D2   （fan-out A。build 不可でよい）
（オーケストレータの cargo check と組ごとの fix）
N-G1, N-G2, N-G3, N-G4, N-G5 ← N-F1〜N-F6 と fix
N-8  ← N-G1〜N-G5
```

### W5

```text
I-0
I-1, I-2a, I-2b, I-3, I-4a, I-4b, I-5a, I-5b, I-6 ← I-0   （公開の署名を固定して並べる）
U-1  ← I-2a
U-2  ← I-3, I-5a
I-7  ← I-1, I-4a, I-6, U-1
T-1
T-2  ← T-1
G-1a, G-1b, G-2, G-4a, G-4b, G-4c, G-5
G-3  ← I-3・U-2
```

- T・G は I・U と crate が重ならないので、I・U の空いた枠に入れる（各 unit の前提は unit の節）。

### W6

```text
K-0
K-1a, K-1b, K-2a, K-2b, K-3a, K-3b, K-4, K-5, K-6a, K-6b ← K-0
K-8  ← K-0（G-1b・G-2・I-5b は W5 で済み）
K-9a, K-9b ← K-2a
K-7a〜K-7e ← その kit の K の unit が全部
H-G1, H-G2                 （W6 の始めから）
H-G3 ← K-0, K-8
H-G4 ← K の中核の段が全部
H-G5 ← その kit の K-7a〜K-7e
H-G6 ← H-G5
```

### W7

```text
F-1（六つに分ける） → F-2 → F-3
```

---

## 2. Phase P: 準備（完了）

P-0（検査の道具）、P-1（状態の全数調査。付録 A は 203 件）、P-2（`KIVACT_TRACE` の trace。今は調べる道具としてだけ使う）は完了した（CP0 通過）。結果は `PROGRESS.md` と `DECISIONS.md`。

---

## 3. Phase V: 語彙（完了）

V-1a〜V-6（`kivact-wire` の解体、`kivact-types` への語彙の一本化、f64 への統一）は完了した（CP1 通過）。W1 で決めたことは `DECISIONS.md` の「W1 definition reconciliation」にある。

---

## 4. Phase A: Arvane

### A-0 `Backend` の注入

- **目的**: 設計書 9.1 節と 19（Backend）。arvane を arvane-native から外す準備（外し終わるのは A-1a）。
- **決定済みの形**:
  - `arvane_shell::ShellEventLoop` を `Backend` と改名する。署名は変えない: `pub trait Backend { fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError>; }`。`capabilities`・`open_child`・`shutdown` は作らない。
  - `arvane_native::backend(options: ShellOptions) -> Result<Box<dyn Backend>, ShellError>`。今の `arvane_native::Shell` は crate の中の `NativeBackend` にする（公開しない）。
  - `arvane::Shell::new(backend: Box<dyn Backend>) -> Shell`（失敗しない。窓の大きさの検査は `arvane_native::backend` が今のまま行う）。`prefer_gpu`・`gpu_preference`・`on_renderer_change`・`run` は変えない。
  - `arvane::spi` を足し、`arvane_shell` の `Backend`・`ShellWindow`・`ShellHandler`・`ShellError`・`ShellEvent`・`ShellChild`・`MakeHandler`・`ChildHooks` を再公開する。
  - embed は native の自由関数のまま: `arvane_native::embed::open(parent, options, hooks, make: MakeHandler)`（今の署名）。`arvane::embed::open` を消し、`arvane::embed::driver<H: Handler + 'static>(options: &EmbedOptions, make: impl FnOnce() -> Embedder<H> + Send + 'static) -> MakeHandler` を足す（今の `open` の中の closure を返す）。`arvane::embed::Embedded::new(child: Box<dyn ShellChild>) -> Embedded` を足す。
  - module の保持: `arvane/src/shutdown.rs`（`RetainOnDrop`）を消す。arvane-native の embed の `Session` が、handler（arvane の `Driver`。presenter を含む）を drop した直後に `if DetachedThreads::any() { retain_module() }` を呼ぶ（今の `Driver._retention` と同じ順。起動の失敗・unwind の経路も同じ）。`arvane::retain_module` の再公開を消し、使う所は `arvane_native::retain_module` を直接呼ぶ。
  - arvane-native の中の `pub(super) trait Backend`（`embed/child.rs`）を `ChildPort` と改名する（`Backend as ChildBackend` などの別名の import も直す）。
- **手順**:
  1. `arvane-shell/src/window.rs`: 改名。`lib.rs` の再公開。
  2. `arvane-native`: `NativeBackend` と `backend()`、`ChildPort` の改名、embed の `Session` に module の保持を移す。
  3. `arvane`: `Shell::new`、`spi`、`embed::driver`・`Embedded::new`、`shutdown.rs` の削除、`lib.rs` の `retain_module` の削除。`window.rs` の `driver()` の `embedded: bool` 引数を消す。
  4. `kivact-engine`: `host/launch.rs` は `Shell::new(arvane_native::backend(ShellOptions { .. })?)`。`embed.rs` は `arvane_native::embed::open(parent, options, hooks, arvane::embed::driver(&options, || ..)).map(Embedded::new)`。`Cargo.toml` に `arvane-native` を足す。`kivact_engine::embed::retain_module` は `arvane_native::retain_module` を呼ぶ。
  5. policy: `visible_to` の `arvane-native` に `kivact-engine` を、`arvane-shell` に `kivact-engine`・`arvane-virtual` を足す。
  6. core は変えない（`run_native` のまま）。arvane の `test_display` はこの unit では arvane-native を使ったまま残す（A-1a で消す）。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p arvane-shell -p arvane-native -p arvane -p kivact-engine -p kivact-core --all-targets --all-features`。
- **規模**: S。**並列**: R-1・E-1 と同時に動いてよい。共有の file は `kivact-engine/Cargo.toml` と `dev/xtask/policy/workspace.toml`（足すだけ。merge で手で合わせる）。

### A-1a `arvane-virtual` と一つの `Window`

- **目的**: 設計書 9.2・9.3 節と 19（arvane-virtual）。付録 A の SM-A15・SM-A16。試験が本番と同じ `Driver`・`Presenter` を通るようにし、`arvane::Window` trait を消す。この unit は arvane 側（shell・virtual・arvane・arvane の e2e）だけを行い、engine と core は A-1b が追従する（この unit の後、engine の試験 host は build 不可でよい）。
- **決定済みの形**:
  - `arvane-shell` に足す:
    ```rust
    pub trait SteppedBackend {
        /// Create the window and call `handler.start` before returning.
        fn start(self: Box<Self>, handler: Box<dyn ShellHandler>) -> Result<Box<dyn SteppedLoop>, ShellError>;
    }
    pub trait SteppedLoop {
        fn step(&mut self, step: Step) -> StepOutcome;
    }
    pub enum Step {
        /// Deliver every queued script item, each as one handler event, in order.
        Input,
        /// Deliver one `ShellEvent::Wake`.
        Wake,
        /// Deliver `ShellEvent::Redraw` if a redraw was requested; else nothing.
        Redraw,
    }
    pub enum StepOutcome {
        Idle { next_deadline: Option<u64>, redraw_requested: bool, woken: bool },
        Exited,
        Failed(ShellError),
    }
    ```
    `ShellWindow` に `fn keeps_frames(&self) -> bool;` と `fn keep_frame(&mut self, frame: CpuFrame);` を足す（native は `false` と `unreachable!("invariant: ..")`）。GPU で出した frame は、`keeps_frames()` のときだけ presenter が read back して `keep_frame` に渡す（今の `present_test_window` の read back と同じ時点）。CPU の frame は `present_with` の buffer を virtual の窓が持つ。
  - `arvane::Shell::start_stepped(backend: Box<dyn SteppedBackend>, setup: ShellSetup, handler: impl Handler + 'static) -> Result<SteppedShell, ShellError>`。`ShellSetup { prefer_gpu: bool, gpu_preference: GpuPreference, observer: Option<Observer> }`（`Shell` の builder の三つと同じ値）。`SteppedShell::step(&mut self, step: Step) -> StepOutcome`。`run` と同じ `driver()` で handler を包む。
  - `arvane::Window` trait を消し、`NativeWindow<'a>` を `arvane::Window<'a>`（struct）と改名する。`arvane::Handler` の引数も `&mut Window<'_>`。
  - `Presenter` は native の経路一つにする: `presenter/test_frames.rs`（`present_test`、`refused`、`enable_gpu`）、`presenter/gpu.rs` の `State::Foreground`・`enable`・`render`（offscreen の read back）、`defers_accessibility` を消す。read back は `read_back` を残し、上の `keep_frame` から使う。
  - 新しい crate `crates/arvane-virtual`（`//! Role: Virtual Arvane backend with scripted input, a virtual clock and optional mirror windows.`）:
    ```rust
    pub struct VirtualOptions {
        pub title: String, pub size: Size, pub scale: f64, pub frame_period_ns: u64,
        pub reduced_motion: bool, pub dark: bool, pub locale: Option<String>,
        pub gpu: GpuMode, pub mirror: MirrorMode,
    }
    pub enum GpuMode { Off, Required }
    pub enum MirrorMode { Off, Auto, Required }
    pub struct VirtualBackend { .. }
    impl VirtualBackend {
        pub fn new(options: VirtualOptions) -> Self;
        pub fn script(&self) -> Script;   // Clone。入力・時刻・環境・サービスの前提を入れる
        pub fn probe(&self) -> Probe;     // Clone。読むだけ
    }
    impl SteppedBackend for VirtualBackend { .. }  // Backend は実装しない
    ```
    `Script` のメソッドは、今の `arvane::test_display::VirtualWindow` の書き換えのメソッド（`input`、`mouse`、`contact`、`focus`、`set_now`、`resize`、`set_scale`、`set_dark`、`set_locale`、`set_frame_period`、`set_reduced_motion`、`set_safe_area`、`create_surface`、`destroy_surface`、`set_occluded`、`fail_presents`、`accessibility_action`、`set_clipboard`、`push_file_dialog`、`push_files_dialog`、`fail_next_effect`、`fail_next_effect_unsupported` など）を **同じ名前と同じ規則** で持つ。`input`・`mouse`・`contact` は今の `VirtualWindow` と同じ正規化（`ClickCounter`・`Mouse`・`Contacts`）をして台本に積み、`Step::Input` で届く。`Probe` は今の `VirtualWindow` の読み出しのメソッド（`now_ns`、`mouse_position`、`modifiers`、`redraw_requested`、`frame`、`accessibility`、`clipboard`、`opened_paths`、`revealed_paths`、`title`、`requested_sizes`、`min_size`、`ime_area`、`cursor`、`soft_keyboard`、`ime_text`、`surface_generation`、`has_window`、`environment`、`size`、`scale`、`capabilities`）を同じ名前で持つ。借用を返していたものは所有の値を返す（`String`、`Vec<PathBuf>`）。`Probe::sync_mirror()` は今の `sync`。
  - mirror の実 window と process に一つの UI スレッドは `arvane-native` に残す（feature `testing`。A-1b で `test` と改名）: `arvane-native/src/test_display.rs` を `mirror.rs` と改名し、`arvane_native::mirror::{MirrorWindow, MirrorStats}` を公開する。`arvane-virtual` の feature `mirror = ["dep:arvane-native", "arvane-native/testing"]` が写しの adapter（`mirror` module、`enum MirrorPending { Idle, Scheduled(MirrorBatch) }`）を持つ。OS の入力は無視する（今と同じ）。
  - `arvane::test_display` を消す: `test_font()` と `gpu_unavailable()` は `arvane` の root へ移す（`gpu_unavailable` は `GpuContext` を一度作れるかで判定する。`Presenter::enable_gpu` は消える）。`display_stats()` は `arvane_virtual::mirror_stats() -> Option<MirrorStats>`。`TestParent` の再公開は A-1b で engine へ移す（この unit では arvane からの再公開を消すだけ）。
  - arvane の feature `testing` を消す（code が残らない）。arvane は arvane-native に依存しなくなる。`arvane/src/env.rs` の `ARVANE_TEST_HEADLESS` の読み出しを消す（mirror の有無は `VirtualOptions.mirror` が決める）。
  - `GpuMode::Required` は mirror の窓を要する（`MirrorMode::Off` と組むと `start` が `ShellError::Unsupported`）。GPU の準備は本番と同じ背景の起動で進む。
- **手順**:
  1. `arvane-shell`: `SteppedBackend`・`SteppedLoop`・`Step`・`StepOutcome`、`ShellWindow::keeps_frames`・`keep_frame`。
  2. `crates/arvane-virtual` を作る。`arvane/src/test_display/{shell.rs, virtual_window.rs, environment.rs}` の code を移して `ShellWindow`・`SteppedLoop` の実装に直す（`git mv` してから直す）。module は設計書 4.4 節の `[module_rules.arvane-virtual.uses]` のとおり（`clock`・`script`・`surface`・`window`・`backend`・`mirror`）。
  3. `arvane-native`: `test_display.rs` → `mirror.rs`。`ShellWindow` の native の実装に `keeps_frames`・`keep_frame`。
  4. `arvane`: `Shell::start_stepped`・`ShellSetup`・`SteppedShell`、`Window` の具体型化、Presenter の一本化、`test_display` の削除、`Cargo.toml` から arvane-native と feature `testing` を消す。
  5. `crates/arvane/tests/e2e`（33 試験）: `support.rs` に `Harness` を作る（`Harness::open(options) -> Harness`、`with<R: 'static>(&mut self, f: impl FnOnce(&mut arvane::Window<'_>) -> R + 'static) -> R` は closure を handler の次の `Wake` で走らせて結果を返す、`script()`、`probe()`）。試験は窓の書き換え（`set_title`、`present`、`request_redraw` など App 側）を `with` に、platform 側（`contact`、`set_occluded`、`fail_presents` など）を `script()` に、読み出しを `probe()` に置き換える。`arvane-virtual` を dev-dependency にする。
  6. 0.9 節の一覧（`arvane-virtual` を足す）。policy: `arvane-virtual` の `visible_to = ["kivact-engine", "arvane"]`（`kivact-test` は C-1 で作る際に足す）。`arvane-native` の `visible_to` は `["kivact-engine", "arvane-virtual", "arvane"]`（`arvane` は既存 native/embed E2E の dev-dependency のみ。通常依存は消す）。`arvane-shell` の `visible_to` に `arvane-virtual` があること。
- **試験**: 足さない（arvane の 33 試験を移す）。
- **unit の確認**: build 可（arvane 側だけ）。`cargo check -p arvane-shell -p arvane-virtual -p arvane-native -p arvane --all-targets --all-features`。engine・core は check しない。
- **規模**: L。手順 1〜4 と 5 で commit を分ける。**並列**: R-2・R-3・E-1 と同時に動いてよい。A-4（arvane-native/src/embed だけ）と同時でよい。

### A-1b engine の試験 host・core の driver・名前

- **目的**: 設計書 9.3 節と 19（arvane-virtual）。付録 A の SM-A15（`woken`）。A-1a の上に engine の試験 host と core の driver を作り直し、試験の名前と環境変数を最終の形にする。
- **決定済みの形**:
  - engine の試験 host（`kivact-engine/src/host/testing/test_host.rs` の `ArvaneTestHost`）は `SteppedShell` と `VirtualBackend` の上に作る。handler は `SharedHost(Rc<RefCell<Launch>>)`（`Launch` は E-2 で作る。この unit の時点では `Rc<RefCell<Host>>`）で、試験 host は同じ `Rc` を持ち、node の問い合わせ（`rect`、`hit`、`focused`、`semantics_target`、`properties`、`scenes` など）はそこから読む。`inject(HostEvent)` は試験専用の engine の口として残す（`Rc` 経由で `Host::inject`）。
  - 入力は `Script` を通す。`send(event)` = `script.input(event)` の後 `step(Step::Input)`。`wake()` = `step(Step::Wake)`。`redraw() -> bool` = `step(Step::Redraw)` で `Redraw` を届けたか。`set_now(ns)` = `script.set_now(ns)`。`due()`・`next_deadline()`・`woken()` は直前の `StepOutcome::Idle` の値。`failure()` は `StepOutcome::Failed` を保存したもの。`exited()` は `Exited`。窓の読み出しは `probe()`。描いた renderer は `ShellSetup.observer` で受けて保存する。`woken: AtomicBool` は消す（`Idle.woken`）。
  - GPU（`GpuMode::Required`）の試験 host は、作った後に `step(Step::Wake)` を、renderer の通知が GPU か失敗になるまで繰り返す（間は `std::thread::park_timeout(5 ms)`、合計 5 秒まで）。
  - core の driver（`kivact-core/src/test/screen/driver.rs`）の `pump` は、turn を回し切った後、`redraw_requested` のあいだ `Step::Redraw` を回す（本番と同じく、支援技術の公開は frame の後）。`present` は `pump` に吸収する。`KIVACT_TEST_LIVE` は意味がなくなるので消す。
  - 試験の窓の公開の名前: `kivact::test::WindowMode` を `MirrorMode { Off, Auto, Required }`、`RenderOptions.window` を `RenderOptions.mirror` にする（`Real` → `Required`、`Headless` → `Off`）。
    `MirrorMode` は engine の再公開を通して `arvane-virtual` の正準型を再公開し、同じ enum を core に定義し直さない。
  - 環境変数は三つ: `KIVACT_TEST_MIRROR`（`off`/`auto`/`required`、既定 `auto`）、`KIVACT_TEST_GPU`（`1` で GPU）、`KIVACT_TEST_HOLD_MS`（今のまま）。`ARVANE_TEST_HEADLESS`・`KIVACT_TEST_HEADLESS`・`KIVACT_TEST_LIVE` を消す。`RenderOptions::default()` が `KIVACT_TEST_MIRROR` を読み、`from_env()` は `KIVACT_TEST_GPU` も読む。
  - feature の名前: `arvane-native` の `testing` を `test` と改名する。`kivact-engine` の feature は `testing` のまま（名前を変えない）で、中身を `testing = ["dep:arvane-virtual", "arvane-virtual/mirror", "dep:png"]` にする。`arvane-virtual` の `mirror = ["dep:arvane-native", "arvane-native/test"]`。`kivact-core` の `test = ["kivact-engine/testing"]`（facade・core の `test` は C-1 が扱う）。
  - `TestParent` は `kivact_engine::testing::TestParent`（`arvane_native::embed::TestParent` の再公開。cfg は今の `all(feature = "embed", any(windows, target_os = "linux"))` のまま）。core の `test/parent.rs` はそれを使う。
  - native の smoke の試験 `crates/arvane-native/tests/smoke.rs` を足す: `ShellHandler` を直接実装し、`start` で `request_redraw`、`Redraw` で `present_with` に一色を書いて `ShellControl::Exit`。`arvane_native::backend` が窓を開けないとき（display がない）は理由を `eprintln!` して return する。window-lock を通して回す試験。
- **手順**:
  1. engine: `host/testing/*` を上のとおり作り直す。`host/pump.rs`・`host/events.rs` の `W: arvane::Window` の generic を `&mut arvane::Window<'_>` にする。`testing.rs` の再公開（`ShellEvent`、`TestParent`、`MirrorMode`、`GpuMode`）。
  2. core: `test/screen/driver.rs`・`input.rs`・`controls.rs`・`ime.rs`・`artifacts.rs` などを新しい試験 host の API に合わせる。`test/options.rs` の `MirrorMode` と環境変数。
  3. `arvane-native` の feature の改名（`Cargo.toml`、`#[cfg(feature = "testing")]`、`required-features`）と、`kivact-engine` の `testing` の中身。`dev/tools/check-features.sh` の一覧。
  4. 文書: `docs/for-kivact-user/{en,jp}/how-to-guide/testing.md`、`crates/kivact-ui-creative/README.md`、`docs/for-kivact-ui-user/{en,jp}/creative/getting-started/03-dock-and-outliner.md` の環境変数と `MirrorMode`。
  5. smoke の試験。0.9 節の一覧。
- **試験**: `crates/arvane-native/tests/smoke.rs` の `native_window_opens_presents_and_closes`。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p kivact-core -p kivact -p arvane-native -p arvane-virtual --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e`（全 E2E が新しい経路で回ること。失敗は 16.2 節のとおり本番の振る舞いを正として試験を直し、直した試験の名前を報告する）。`bash dev/tools/window-lock.sh -- cargo test -p arvane-native --features test --test smoke`。
- **規模**: L。手順 1 と 2〜5 で commit を分ける。**並列**: R-2 の merge を待つ（core の `test/screen/driver.rs` が R-2 の消す `turn_requested` を読んでいる）。A-4・A-5・R-3〜R-6 と同時でよい。

### A-2 サービスと capability

- **目的**: 設計書 9.1 節の後半と 19（perform）。付録 A の SM-A09。
- **決定済みの形**:
  - `ShellWindow` から `copy_text`・`read_text`・`pick_file`・`pick_files`・`save_file`・`pick_folder`・`open_path`・`reveal_path`・`request_size`・`set_fullscreen` を消し、`fn perform(&mut self, request: PlatformRequest) -> PlatformResult;` を一つ足す（UI スレッドで同期に答える。未対応は `PlatformResult::Unsupported`、失敗は `PlatformResult::Failed(Failure)`）。`fullscreen()` の getter は残す。
  - `kivact_types::PlatformRequest` から `RegisterFont`・`ReducedMotion` を消す。engine の契約に `pub enum HostRequest { Platform(PlatformRequest), RegisterFont(Asset), ReducedMotion(Option<bool>) }`（`kivact_engine::contract`）を足し、`Writer::effect(request: RequestId, effect: HostRequest)` にする。`RegisterFont` は今の Writer の時点で答える。`ReducedMotion` は今の pump の処理のまま（Host の `reduced_motion_override` 一か所）。`PlatformResult::Fonts` は残す。
  - engine の pump の効果の処理は `HostRequest::Platform(r)` を `window.perform(r)` へ渡す一本にする。`Fullscreen` の結果の後は今のとおり `window.fullscreen()` を環境へ写してから `EffectResult` を積む。key の copy・paste（`host/events.rs`）も `perform(CopyText)`・`perform(ReadText)` にする。
  - `ShellWindow::cpu_format(&self) -> CpuFormat`、`pub enum CpuFormat { Xrgb8888 }`（`arvane-shell`）。renderer の CPU の書き出しはこの形式を見て書く（今は一つ）。
  - `Capabilities`（`kivact-types/src/env.rs`）に `clipboard_text`、`file_dialogs`、`open_path`、`reveal_path`、`fullscreen`、`min_size`（bool）、`command_key: CommandKey`、`context_menu_on: ContextMenuTrigger` を足す。`pub enum CommandKey { #[default] Ctrl, Command }`、`pub enum ContextMenuTrigger { Press, #[default] Release }`（`Default`・`Copy`・`Eq` を derive）。native は `cfg!(target_os = "macos")` で `Command`・`Press`、ほかは `Ctrl`・`Release`。embed の子 window は X11 の `min_size`・`soft_keyboard`・`fullscreen` と IME の text、Win32 の `min_size`・`soft_keyboard`・`fullscreen` を `false` にする（今 no-op のもの）。virtual は全部 `true`。core の `missing_capabilities` の一覧は変えない。
  - native の clipboard（`arvane-native/src/services.rs`）: `enum Clipboard { Unopened, Open(arboard::Clipboard), Failed(Failure) }`。`Failed` の後は再試行せず `PlatformResult::Failed` を返す。
  - virtual の `perform`: 今の `VirtualShell` の service の振る舞い（memory の clipboard、dialog の queue、open・reveal の記録、`fail_next_effect`）を既定のまま持つ。`Script::respond(f: impl FnMut(&PlatformRequest) -> Option<PlatformResult> + 'static)` を足し、登録した responder を新しい順に試して `Some` ならそれを答える（`None` なら既定へ）。既定にない request は `Unsupported`。`Screen` の台本の API は変えない。
- **手順**: `arvane-shell`（trait、`CpuFormat`）→ `arvane-native`（`runtime.rs` の native の窓、`embed/windows/window.rs`・`embed/x11/window.rs`、`services.rs`）→ `arvane-virtual` → `arvane`（`Window` の転送、CPU の書き出し）→ `kivact-types`（`PlatformRequest`、`Capabilities`）→ engine（`contract`、`host/write/writer.rs`、`host/pump.rs`、`host/events.rs`）→ core（`platform/services.rs`、`view/requests.rs` が `HostRequest` を作る）→ `kivact-interaction/src/menu/handle.rs` の `cfg!(target_os = "macos")` を `capabilities().context_menu_on == ContextMenuTrigger::Press` にする。
- **試験**: 足さない（platform services の E2E が回帰の網）。
- **unit の確認**: build 可。`cargo check -p arvane-shell -p arvane-native -p arvane-virtual -p arvane -p kivact-types -p kivact-engine -p kivact-core -p kivact-interaction --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- platform`。
- **規模**: M。**並列**: A-4・A-5 と同時でよい（`arvane-native/src/embed/{windows,x11}/window.rs` を A-4 と共有: merge で手で合わせる）。

### A-3 native の runner と shell

- **目的**: 付録 A の SM-A07・A08・A10・A13・A901。設計書 7.5 節（modifier と mouse の位置の持ち主）。
- **決定済みの形**:
  - `Runner.cursor`（physical pixel の位置）を消し、shell の `Mouse` の論理の位置で「前と同じ move か」を比べる。file drag の位置の fallback も `Mouse` の位置を使う。
  - shell に `pub struct ModifierState(Modifiers)`: `fn update(&mut self, m: Modifiers) -> Option<ShellEvent>`（変わったときだけ `ShellEvent::Modifiers`）、`fn current(&self) -> Modifiers`。winit の `ModifiersChanged`、X11 の xkb の mask（`keyboard.rs` の `sync_modifiers`）、Win32 の `GetKeyState`（`input.rs` の `modifiers()`）はここへ入れる。
  - `NativeInput`（`native_input/state.rs`）: mailbox の `Cell<Option<T>>` を `kivact_types::cell::LocalMailbox<T>`、`env_changed` を `LocalMailbox<()>`、`subclass_installed` を `enum Subclass { NotInstalled, Installed }`。`NativeInput::new(environment: Environment, wake: InputWake)`、`enum InputWake { Direct, EventLoop(ShellWaker) }`（native の runner は `EventLoop`、embed は `Direct`）。後から環境・waker を埋める処理を消す。
  - `ShellEvent::Wake(WakeReasons)`。`pub struct WakeReasons(u8)` と定数 `CLIENT`・`ASSETS`・`A11Y_ACTIONS`・`ENVIRONMENT`・`GPU_STEP`、`contains`・`insert`・`is_empty`。`ShellWaker::wake(&self, reasons: WakeReasons)`。native は届くまでの理由を `AtomicU8` に OR で溜め、一つの `Wake` で届ける。engine は理由を見ずに今の `Wake` と同じ処理をする。arvane の `Driver` は `GPU_STEP` を含むときだけ GPU の準備を進める（GPU の worker は `GPU_STEP` で起こす）。
  - 支援技術（SM-A13）: shell の `AcceptedTree` は残す（支援技術の起動時の木と action の検査に要る）。`Shared::update` は、受理済みの木との差分の計算をやめ、update が運ぶ node の列をそのまま変更の集合として適用・検証する。`AcceptedTree.focus` は保存せず update の値を使う。遅延 activation と snapshot に要る最新 focus は `Shared` だけに保持し、`PlatformUpdate::focus()` で渡す。`Unsent::Changed.merged: bool` は `enum Merge { Separate, Merged }`。
  - `ShellEvent::AccessibilityActivated`・`AccessibilityDeactivated` を足し、native（AccessKit の activation と deactivation の handler）と virtual（start の直後に `Activated`）が出す。engine はこの unit では受けて何もしない（使うのは E-6）。
- **手順**: `arvane-shell`（`ModifierState`、`WakeReasons`、event、a11y）→ `arvane-native`（`runtime.rs`、`native_input/*`、`embed/windows/input.rs`・`embed/x11/keyboard.rs`、AccessKit の adapter）→ `arvane-virtual` → `arvane`（`Driver`、presenter の GPU の waker）→ engine（`ShellEvent::Wake(_)` の match、新しい event の arm）。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p arvane-shell -p arvane-native -p arvane-virtual -p arvane -p kivact-engine --all-targets --all-features`。状態の点検表（0.7 節）。
- **規模**: M。**並列**: A-2 の merge を待つ（`runtime.rs` の窓の実装を共有）。A-4・A-5 と同時でよい（`embed/windows/input.rs`・`embed/x11/keyboard.rs` を A-4 と共有: 手で合わせる）。

### A-4 embed の lifecycle

- **目的**: 設計書 9.4 節と 19（A-4 lifecycle）。付録 A の SM-A01〜A06。
- **決定済みの形**:
  - thread をまたぐ `Shared` の中の `ChildState` は `ChildStatus { Running, Stopped { cause: Stop, window: NativeWindow } }` と改名し、そのまま `Mutex` に置く（host の thread が `failure()` で読む）。
  - UI スレッドだけの状態を `embed/session.rs` の一つの enum にする: `enum ChildLifecycle { Opening(HandlerSlot), Live(HandlerSlot), Disposed, Leaked }`、`enum HandlerSlot { Unstarted, Running { handler: Box<dyn ShellHandler>, control: ShellControl } }`。今の `HandlerState`、Win32 の `Phase`（`state.rs`）、X11 の `opening: bool`（`x11/window.rs`）を置き換える。panic の後の `mem::forget` は `Leaked` への遷移で行う。
  - X11 の `Connection.exists: AtomicBool`（`x11/conn.rs`）は thread の間の guard なので `struct ConnectionAlive(AtomicBool)`（`is_alive(&self) -> bool`、`end(&self) -> bool`: 最初の一回だけ true）に包む。
  - Win32 の再入の guard は二つが重なる（drain の中で callback が走る）ので、一つの enum にしない: `in_callback`・`draining` を `struct ReentryGuard(Cell<Held>)`・`struct DrainGuard(Cell<Held>)`、`enum Held { Free, Held }` にし、今の `Hold` を各 guard の `enter(&self) -> Option<Hold<'_>>` にする。
  - Win32: `pending_surrogate` を `struct Utf16Decoder`（`push(&mut self, unit: u16) -> Option<char>`）、`uia: RefCell<Option<Adapter>>` を `enum Uia { Inactive, Active(Adapter) }`、`wake_pending: AtomicBool` を `struct WakeCoalescer`（`Mailbox<()>` と同じ意味）、`Opening.armed: bool` を `enum OpeningGuard { Armed, Disarmed }`。
  - X11: `redraw: Cell<bool>` を `enum ChildRedraw { Clean, Needed }`、`Input.focused: bool` を `enum WindowFocus { Blurred, Focused }`、XIM の `App.allowed` を `enum ImeAllowance { Disallowed, Allowed }`、focus は `WindowFocus`、`Connection` の handshake の間だけの field を `enum Connection { Handshake { protocol_window: u32, server_window: Option<u32> }, Established { .. } }`。
  - engine の `closing` は触らない（E-2）。
- **手順**: `arvane-native/src/embed/` の中だけ（`child.rs`、`session.rs`、`windows/*`、`x11/*`）。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p arvane-native -p arvane -p kivact-engine -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- embed`（A-1b の前なら window-lock を通す）。状態の点検表。
- **規模**: M。**並列**: A-0 の merge の後なら、A-1a・A-1b・A-2・A-3・A-5 と同時でよい（A-2・A-3 と `embed/*/window.rs`・`embed/windows/input.rs`・`embed/x11/keyboard.rs` を共有: 手で合わせる）。

### A-5 GPU の状態

- **目的**: 付録 A の SM-A11・A12・A14 と 19（A-5 FrameHealth）。
- **決定済みの形**:
  - surface の失敗の数え方を `FrameHealth`（`arvane/src/presenter/health.rs`）一つにする: `fn lost(&mut self, renderer: Renderer) -> LossAction`、`enum LossAction { Recover, FallbackToCpu, Fail }`。続けての失敗の数 `losses` が、GPU で 2 なら `FallbackToCpu`、どの renderer でも 3 以上なら `Fail`、それ以外は `Recover`。`shown` で `Healthy`、`skipped` で `Skipping`（今と同じ）。`Device.failures` と `SURFACE_FAILURES` を消す。GPU の surface の失敗は `gpu.failed` が `Fallback::Surface` を返し、frame の方針（`frame_policy.rs` の `settle`）が `lost(Renderer::Gpu)` を呼んで、`FallbackToCpu` なら今の `cpu_session` と同じ処理（同じ frame を CPU で描く）を行う。今との違いは「GPU の失敗の間の skip が数を戻す」だけで、それは受け入れる（DECISIONS に一行）。
  - `Device.surface: Option<Box<GpuSurface>>` を `enum DeviceSurface { Pending, Live(Box<GpuSurface>) }`、`Worker.run: Option<Running<T>>` を `enum WorkerState<T> { Running(Running<T>), Joined }`、`Step::Configuring { stale: bool }` を `Step::Configuring { .., surface: ConfigureTarget }`、`enum ConfigureTarget { Current, Destroyed }`。
  - `arvane-render/src/gpu/state.rs`: device lost の latch を `DeviceLost`（`Mailbox<String>` の包み）、`offscreen: Option<Texture>` を `enum Offscreen { None, Allocated { texture, size } }`、`keep: bool` を `enum Retain { Drop, Keep }`、`Configured.skipped: bool` を `enum LastConfigure { Applied, Skipped }`。
  - `kivact-gpu-share/src/pool.rs`: `drawn: Vec<Option<NonZeroU64>>` を `Vec<SlotState>`、`enum SlotState { Fresh, Drawn(NonZeroU64) }`。`wgpu/pool.rs` の `failed: bool` を `enum PoolHealth { Ok, Broken }`。`arvane-render/src/gpu/external.rs` の `with_external` を型状態の `ExternalPass<'_>`（`submit(self) -> Submitted`）にし、`frame: Option<Frame>` と `Frame.submitted: bool` を消す。
- **手順**: `arvane/src/presenter/{gpu.rs, health.rs, frame_policy.rs}` → `arvane-render/src/gpu/*` → `kivact-gpu-share`。
- **試験**: 足さない。GPU のない環境の skip の扱いは変えない。
- **unit の確認**: build 可。`cargo check -p arvane -p arvane-render -p kivact-gpu-share -p gpu-interop-producer -p kivact-public-api-tests --all-targets --all-features`。状態の点検表。
- **規模**: M。**並列**: A-1a の merge を待つ（`State::Foreground` の削除）。A-1b・A-2・A-3・A-4 と同時でよい。

### A-6a arvane-native の port と key・cursor・提示

- **前段からの申し送り**: A-4 に残した native child の `frame: Option<CpuFrame>` は、この unit の softbuffer/CpuSurface 提示統合で解消する。

- **目的**: 設計書 17.2 節と 19（pipeline の置き場所）。D-14 の前半。
- **決定済みの形**:
  - OS を知らない正規化は `arvane-shell/src/pipeline/` に置く: `raw.rs`（`RawInput`）、`keys.rs`、`cursor.rs`（A-6a）、`ime.rs`、`dnd.rs`（A-6b）。`arvane-native/src` は `port/{winit, win32, x11}`（OS の呼び出しと winit の型からの写像だけ）、`present.rs`（softbuffer）、`a11y/`（AccessKit の conversion。今のまま）、`services.rs`、`display/`、`mirror.rs` にする。今の `embed/windows/*` は `port/win32/`、`embed/x11/*` は `port/x11/`、`runtime.rs`・`key_code.rs`・`wheel.rs`・`contacts.rs` は `port/winit/` へ（`git mv`）。`embed/{child.rs, session.rs, geometry.rs}` は `port/embed/` へ。
  - ```rust
    // arvane_shell::pipeline
    pub enum RawInput { Key(RawKey), Ime(RawIme), Drop(RawDrop) }
    pub struct RawKey { pub key: PlatformKey, pub pressed: bool, pub repeat: Option<bool>, pub text: Option<String> }
    pub enum PlatformKey {
        /// Win32: virtual key, the character ToUnicodeEx gave, and the physical code the port mapped.
        Vk { vk: u16, chr: Option<char>, code: KeyCode },
        /// X11: the layout's keysym, its character, and the X keycode.
        Keysym { keysym: u32, chr: Option<char>, keycode: u8 },
        /// Already mapped by the port (winit).
        Code { key: Key, code: KeyCode },
    }
    pub struct KeyPipeline { /* pressed keys */ }
    impl KeyPipeline {
        pub fn key(&mut self, raw: RawKey, modifiers: Modifiers, pointer: Option<Point>) -> ShellEvent;
        pub fn focus_lost(&mut self);
    }
    pub fn key_from_vk(vk: u16, chr: Option<char>) -> Key;
    pub fn key_from_keysym(keysym: u32, chr: Option<char>) -> Key;
    pub fn code_from_x11_keycode(keycode: u8) -> KeyCode;
    pub fn cursor_to_vk_cursor(..); pub fn cursor_to_x11_name(cursor: Cursor) -> &'static CStr;  // 写像表だけ
    ```
    `repeat` が `None` なら pipeline が押下中の集合から決める（X11 の今の規則）。`Some` は port の値（Win32 の lparam の bit 30、winit の `repeat`）。text は制御文字を含めば捨て、離しでは持たない（今の三つの経路の規則）。winit の `KeyCode`・`Key` の写像（`key_code.rs`）と Win32 の scancode → `KeyCode`（winit の表を使う）は `port/winit` に残す。
  - 提示: Win32 の子 window の CPU の提示を softbuffer にする（`present.rs` を三つの port が使う）。`embed/windows/cpu.rs`（GDI）を消す。
  - `Script::raw(input: RawInput)`（`arvane-virtual`）を足す。virtual の窓は同じ `KeyPipeline` を通す。
  - policy: `[module_rules.arvane-shell.uses]` に `pipeline = ["events", "types"]` を足す（今の行は保つ）。`[module_rules.arvane-native.uses]` は `port = ["present", "a11y", "services", "display"]`、`present = []`、`a11y = []`、`services = []`、`display = []`、`mirror = ["port", "present", "a11y"]`。
- **手順**: (1) `git mv` で port へ並べ替え（中身は変えない）。(2) key の表を shell の `pipeline/keys.rs` へ移し、三つの port が `RawKey` を作って `KeyPipeline::key` を呼ぶ。(3) cursor の表を `pipeline/cursor.rs` へ。(4) softbuffer の提示。(5) `Script::raw`。
- **試験**: `dev/tests/kivact-public-api/tests/e2e/input_platform.rs` に `raw_key_mapping_reaches_handlers`（`Script::raw` で `PlatformKey::Vk { vk: 0x41, chr: Some('a'), code: KeyCode::KeyA }` と `Keysym { keysym: 0xff0d, .. }` を入れ、`on_key_down` が `Key::Char('a')`・`Key::Enter` を受けること）。`Script` を公開の試験から使う口は `Screen::raw_input(RawInput)`（`kivact::test` の試験専用のメソッド。`RawInput` 一式は `kivact::test::raw` に再公開する）。
- **unit の確認**: build 可。`cargo check -p arvane-shell -p arvane-native -p arvane-virtual -p arvane -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- input_platform`。Windows の embed の E2E（`--test e2e -- embed`）。
- **規模**: L。手順 (1) と (2)〜(5) で commit を分ける。**並列**: X-1 の merge を待つ。E-2〜E-7 と同時でよい（engine を触らない）。E-6 が `ShellWindow::accessibility_active` の実装を消すので、移した file の上で手で合わせる。

### A-6b IME と DnD の pipeline

- **目的**: 設計書 17.2 節と 19（pipeline の置き場所）の後半。
- **決定済みの形**:
  - ```rust
    // arvane_shell::pipeline
    pub enum RawIme { Preedit { text: String, caret: Option<(usize, usize)> }, Commit(String), End, Reconvert { selection: (usize, usize) } }
    pub trait ImePort {
        fn allow(&mut self, allowed: bool);
        fn cursor_area(&mut self, area: Rect);
        fn surrounding(&mut self, text: Option<&ShellImeText>);
    }
    pub struct ImeDriver { composition: Composition, allowed: Sent<bool>, area: Sent<Rect>, text: Sent<Option<ShellImeText>> }
    enum Composition { Idle, Composing { preedit: String, caret: Option<(usize, usize)> } }
    impl ImeDriver {
        pub fn input(&mut self, raw: RawIme) -> Vec<ShellEvent>;
        pub fn set_allowed(&mut self, port: &mut dyn ImePort, allowed: bool);
        pub fn set_cursor_area(&mut self, port: &mut dyn ImePort, area: Rect);
        pub fn set_text(&mut self, port: &mut dyn ImePort, text: Option<ShellImeText>);
    }
    pub enum RawDrop { Hover { files: Vec<PathBuf>, at: Option<Point> }, Move(Point), Drop, Leave }
    pub enum DropSession { Idle, Hovering { files: Vec<PathBuf>, at: Option<Point> }, Dropped { files: Vec<PathBuf>, at: Option<Point> } }
    impl DropSession { pub fn input(&mut self, raw: RawDrop) -> Vec<ShellEvent>; pub fn take_dropped(&mut self) -> Option<ShellEvent>; }
    ```
    `DropSession` は今の winit の `Dnd`（`runtime.rs`）の規則（winit の一束の終わりで drop を届ける）を持つ。Win32 の `WM_DROPFILES` と X11 の Xdnd は `RawDrop` を作るだけにする。
  - `ImePort` の実装は winit＋Windows の subclass、IMM32、XIM の三つ。XIM の handshake（A-4 の `Connection`）とその protocol は port に残す。`Sent` で重複を除くのは `ImeDriver` だけ（port の中の同じ重複除去を消す）。
  - `Script::raw` は `RawInput::Ime`・`RawInput::Drop` も通す。
- **手順**: `pipeline/ime.rs` と三つの `ImePort` → `pipeline/dnd.rs` と三つの port → virtual。
- **試験**: `input_platform.rs` に `raw_ime_composition_commits`（`RawIme::Preedit` の後 `Commit("日本")` で text input の値が `日本` になり、preedit の間は値が変わらないこと）と `raw_file_drop_reaches_dropzone`（`RawDrop::Hover` → `Drop` で `on_file` の handler が一度だけ file の列を受けること）。
- **unit の確認**: build 可。上と同じ crate。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- input_platform embed`。
- **規模**: M。**並列**: A-6a の merge を待つ。E-2〜E-9b と同時でよい。

## 5. Phase E: エンジン

### E-1 `kivact-text` と `kivact-lower` を engine に入れる

- **目的**: 設計書 4.3・8.1 節と 19（lower を engine へ）。付録 A の SM-E33（latch の部分）。
- **決定済みの形**:
  - `crates/kivact-text/src` を `crates/kivact-engine/src/text_input/`（module の root は `text_input.rs`）へ、`crates/kivact-lower/src` を `crates/kivact-engine/src/lower/`（root は `lower.rs`）へ `git mv` する。二つの crate を消す。lower の中の `pub` は `pub(crate)` にする（engine の外に出ない）。`kivact_lower::` は `crate::lower::`、`kivact_text::` は `crate::text_input::` にする。
  - lower の `Context` の latch: `dropped`・`failures` を外から読む getter を消し、`Context::finish(self) -> LowerReport` を足す。`pub(crate) struct LowerReport { pub(crate) dropped: u32, pub(crate) failures: Vec<Failure> }`。Context の中の `Cell`・`RefCell`（一回の lowering の範囲の値）は残す。lowering の呼び出し元は、終わりに `finish` を呼んで報告を受け取る。`ArvaneAssets`（`take_pending -> Vec<AssetEvent>`）・`AssetEvent`・`BoxShape`・`shape` は lower の中にそのまま残す（engine の中なので移さない）。
  - module の向き（`[module_rules.kivact-engine.uses]`）: `lower = []`、`text_input = []` を足し、`tree`・`measure`・`model`・`text_measure`・`paint`・`check`・`offscreen` に `"lower"` を、`measure`・`model`・`host` に `"text_input"` を足す。`tree` の生の子の扱いは E-8。
  - kivact-text の `Cargo.toml` の依存のうち engine にないもの（`unicode-segmentation`）を engine に足す。kivact-text の kivact-lower への依存（使っていない）は消える。
- **手順**:
  1. `git mv` と path の置き換え（file の中身は変えない）。
  2. `pub` → `pub(crate)`、`//! Role:` の行を module の root に。
  3. `finish` と `LowerReport`。
  4. 0.9 節の一覧（二つの crate を消す。`Cargo.toml` の `members`、policy の `visible_to`・各検査の `skip` から `kivact-lower`・`kivact-text` を消す、`arvane` の `visible_to` から `kivact-lower`・`kivact-text` を消す）。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p kivact-core --all-targets --all-features`。`cargo xtask check --no-write --only module-rules --crate kivact-engine`（その名の検査がなければ `cargo xtask check --no-write`）。
- **規模**: M。**並列**: A-0〜A-5・R-1〜R-6 と同時でよい（engine の `host/` を触らない）。共有の file は `Cargo.toml` の `members`、policy、`kivact-engine/Cargo.toml`（手で合わせる）。

### E-2 host の状態機械

- **目的**: 設計書 8.2 節と 19（CloseSignal）。付録 A の SM-E01〜E07、E18（E18 は A-2 で済み）、SM-E33（store の差し替え）。
- **決定済みの形**:
  - `Host` の状態:
    ```rust
    enum HostPhase { Starting, Running, Exiting, Closed }
    enum AppPresence { Foreground, Suspended }
    enum Surface { Drawable { generation: u64 }, Unavailable(Unavailable) }
    enum Unavailable { NoSurface, ZeroSize }
    enum LastPresent { Settled, Skipped { retry_at_ns: u64 } }   // 今の LastFrame の改名
    ```
    `Host` は `phase: HostPhase`、`presence: AppPresence`、`surface: Surface`、`last_present: LastPresent` を並べて持つ（`presence`・`surface` はどの phase でも事実なので phase の中に入れない）。`suspended: bool` を消す。`fn drawable(&self) -> bool` = `!matches!(self.phase, HostPhase::Closed) && self.presence == Foreground && matches!(self.surface, Surface::Drawable { .. })`。model の `Clock` と core への `surface_active` はここから turn ごとに渡し、保存しない（`model/frame/state.rs` の `surface_active` の field と `ticks.rs` の `debug_assert_eq!` を消す）。
  - `SurfaceLost`（`Presented::SurfaceLost`。窓は既に surface を作り直している）は、`HostFailure` を一度積んで paint を無効にするだけにする（`set_surface(false)` → `set_surface(true)` の往復を消す。状態は持たない）。
  - 閉じる要求: `pub struct CloseSignal(Arc<AtomicBool>)`（`kivact_engine::contract`。`Clone`、`new()`、`close(&self)`、`is_set(&self) -> bool`）。`EmbeddedHost::drop` が `close`。`Host` は `close: CloseSignal` を持ち（native も持つが立たない）、`fn observe_close(&mut self) -> bool` が立っていれば `phase = Closed` にして true を返す。呼ぶ所は「host の入口（`begin`・`handle`・`inject`）」と「client の呼び出し（start・turn）の直後」と「効果を一つ処理した直後」だけ（今の 14 か所の `closing()` をこの関数に置き換え、場所の数は今の再入の点だけに減らす）。`Closed` では turn も効果も行わず `ShellControl::Wait` を返す（今の振る舞い。exit の turn は走らせない）。core の `AppClient.embed_closing: Option<Arc<AtomicBool>>` は `Option<CloseSignal>` にし、event ごとの中断（今の `embed_closing()`）と drop の時の module の保持はそのまま。`HostEvent::CloseRequested` は作らない。
  - 起動: `enum Launch { Pending(Box<LaunchArgs>), Starting, Running(Box<Host>) }`（`host/launch.rs`）が `arvane::Handler` を実装する。`LaunchArgs { client: Client, window_size: Size, engine: arvane::Engine, frame_log: Rc<RefCell<FrameLog>>, theme: ThemeHint, assets: AssetMode, close: CloseSignal }`、`enum AssetMode { Worker, Inline }`。`start` は waker から `AssetStore::new`（`Inline` なら `AssetStore::inline()`）を作り、`Model::new(size, store)` で model を作って `Host` を組み、`begin` を呼んで `Running(host)` になる。引数の移動から構築中は `Starting` とし、dummy Client/Engine は作らない。`Pending` のまま event が来るのは不変条件の違反（`expect("invariant: ..")`）。`ArvaneAssets::replace_store` と `Model::set_store` を消す。試験 host の `SharedHost` は `Rc<RefCell<Launch>>` を持ち、問い合わせは `Running` の `Host` を読む（`TestHostOptions` の theme と `inline_assets` は `LaunchArgs` へ）。Offscreen は今のまま inline の store。
  - frame の状態: `frames: Vec<u64>` を `struct FrameRequests(BTreeSet<u64>)`、`SmoothScroll.began: Option<u64>` を `enum SmoothScroll { Scheduled { start: Point, target }, Running { start: Point, target, began_ns: u64 } }`（補間の基準 start は両状態で保持）、model の `last_frame: Option<u64>` を `enum TickHistory { NeverTicked, Ticked { last_ns: u64 } }`。
  - `environment` を `Sent<Environment>`、`adapter` を `Sent<GpuAdapter>`、`accessibility_noted` を `ReportOnce`。`Sent::get(&self) -> Option<&T>` で前回値を読み、theme override と ReducedMotion(None) の挙動を保つ。`reduced_motion_override: Option<bool>` は設定の値として残す。`soft_keyboard` は E-4。
  - font の世代（V-3 の続き）: font の collection の持ち主（`lower::assets::ArvaneAssets::register_font`）が登録の成功で `Environment.fonts: Generation` を `next()` し、更新後の環境を `EnvChanged` で知らせる。native の環境の読み直しで世代を初期値へ戻さない（読み直しの値に今の世代を入れる）。
- **手順**: `host/pump.rs`・`host/events.rs`・`host/launch.rs`・`host/testing/test_host.rs`・`model/frame/*`・`embed.rs` → core の `client/app_client.rs`・`embed.rs`（型の差し替えだけ）。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- lifecycle environment embed`。状態の点検表。
- **規模**: M。**並列**: R-2・X-1 の merge を待つ。E-3〜E-7・A-6a と同時でよい（`host/events.rs` を E-3・E-5a と、`host/pump.rs` を E-4・E-6 と共有: 手で合わせる）。

### E-3 pointer と key

- **目的**: 設計書 8.5 節。付録 A の SM-E10〜E14、E16、E17（SM-E15 は行わない: 下）。
- **決定済みの形**:
  - ```rust
    enum Track { Mouse(MouseTrack), Touch(TouchTrack), Pen(PenTrack) }
    struct Hover { chain: Vec<NodeId> }
    struct MouseTrack { source: PointerSource, at: Option<Point>, over: Option<NodeId>, hover: Hover, gesture: MouseGesture }
    struct TouchTrack { source: PointerSource, at: Point, over: Option<NodeId>, hover: Option<Hover>, gesture: TouchGesture }
    struct PenTrack { source: PointerSource, at: Point, over: Option<NodeId>, hover: Option<Hover>, route: Route }
    enum MouseGesture { Route(Route), Scrollbar(ScrollDrag), SelectText { editor: NodeId, route: Route } }
    enum TouchGesture { Route(Route), Pending { route: Route, candidate: TouchCandidate }, Pan(TouchPan) }
    struct Route { capture: Option<NodeId>, press: Option<Press> }   // 二つは独立の事実なので struct のまま
    ```
    touch・pen の `hover` は primary のときだけ `Some`（今の「primary でなければ空」の不変条件）。`over` は各 pointer の Enter/Leave/Move の配送先として Track に一つだけ持ち、secondary の既存配送も保つ。`PointerState.mouse` を消し、`MouseTrack.at` を読む（`None` になる規則は今の `PointerState.mouse` と同じ）。`SelectText` は press の時点で、今の推定（focus のある editor、primary の押下、押下の chain が focus を含む）が成り立つときに入る。button のない move の `Cancel` の合成は今のまま。
  - SM-E15 は行わない: `Admission`（`model/modal/state.rs`）は client の dispatch の順で問われる表で、`Track` は入力の時点の状態なので、混ぜると振る舞いが変わる。付録 A の SM-E15 の「解消後」を「`Admission` のまま（dispatch の順の表）」と直す。
  - `struct KeyDownTargets(HashMap<KeyCode, FocusTarget>)`。`key_down` を受けた target を記録し、`key_up` はその target へ送る。記録がない・`KeyCode::Unidentified`・target の node がもうないときは、今と同じく focus のある node へ送る。focus を失ったとき（window の blur）に表を空にする。
  - modifier: `Model.modifiers` を `Sent<Modifiers>` にし、core への `Modifiers` の event は変わったときだけ送る（shell の `ModifierState` が正）。
  - `HostEvent::HoverChanged { node: Option<NodeId> }` を `HoverChanged { target: Option<FocusTarget> }` にする（canvas の region の hover を運ぶ）。core の `client/dispatch.rs` と `view/hover.rs` は `target.map(|t| t.node)` で今の振る舞いのまま受ける（region の使い道は C-4）。
- **手順**: `model/pointer/*`、`model/keys.rs`、`model/aggregate.rs`、`host/events.rs` の modifier、`contract/events.rs` → core の `client/dispatch.rs`・`view/hover.rs`。
- **試験**: `dev/tests/kivact-public-api/tests/e2e/keyboard.rs`（なければ作り、`main.rs` に `mod` を足す）に `key_up_goes_to_the_node_that_received_key_down`（node A で key を押し、押している間に focus を B へ移し、離すと A の `on_key_up` が呼ばれ B のは呼ばれない）。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- keyboard pointer hover`。状態の点検表。
- **規模**: M。**並列**: R-2・X-1 の merge を待つ。E-2・E-4〜E-7 と同時でよい。

### E-4 IME と編集

- **目的**: 設計書 8.5 節。付録 A の SM-E19〜E22、E32。
- **決定済みの形**:
  - `EditorNode.pending_controlled: Option<Rc<str>>` と `controlled_at: u64` を `enum ControlledValue { InSync { at: u64 }, HeldDuringComposition { value: Rc<str> } }`。composition が終わる遷移で `InSync` に戻し、保留した値を適用する（今の規則）。
  - `ImeGiven::Input { source, start: Option<usize> }` の `start` を `enum InputStart { At(usize), NoText }`。`ime_select` は `NoText` を明示の分岐で扱う（今は黙って無視: 同じ振る舞い）。
  - `enum ImeTarget { None, Native(EditorSlot), Session(NodeId) }` を focus が変わったときに一度だけ決め、IME の全部の入口がこれを読む（`focused_session` と二つの経路の判定を置き換える）。
  - host の `set_ime_allowed` の毎 pump の呼び出しを `Sent<ImeAllowed>`（`enum ImeAllowed { No, Yes }`）、`soft_keyboard: bool` を `Sent<SoftKeyboard>`（`enum SoftKeyboard { Hidden, Shown }`）。
  - `kivact-editing`: `damage: Cell<Option<Damage>>` を `damage: Option<Damage>`、`take_damage(&mut self)`。編集完了時に mutable editor から既存 EditLines cache へ damage を移譲し、paint/a11y/measure の共有読取りを維持する（二重保持や新 journal は作らない）。`max_length` は `EditState::set_max_length(&mut self, ..)` の明示の呼び出しだけで変える。engine の editor の索引は `editors` だけにし、`Edits.inputs: BTreeSet` を消す。`EditLines` の借用の失敗（今は黙って `Ok(None)`）は `expect("invariant: ..")`。
- **手順**: `measure/editor.rs`・`measure/input.rs`、`model/edit/*`、`model/aggregate.rs`、`host/pump.rs`（IME と soft keyboard）、`kivact-editing/src/{edit_state.rs, editor.rs}`。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-editing -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- text_editing`。状態の点検表。
- **規模**: M。**並列**: X-1 の merge を待つ。E-2・E-3・E-5a〜E-7 と同時でよい。

### E-5a layout の持ち主と settle

- **目的**: 設計書 8.4 節と 19（Layout）。付録 A の SM-E27〜E30。
- **決定済みの形**:
  - `kivact-layout` の `NodeLayoutColumns` と `LayoutState` を一つの持ち主 `pub struct Layout`（`Clone`、`Default`）にまとめる。二つの型は crate の外に出さない。
    ```rust
    impl Layout {
        pub fn push(&mut self);
        pub fn reset(&mut self, slot: usize);
        pub fn rebuild_style<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId);   // 今の署名の引数のまま
        pub fn rebuild_all<T: LayoutTree + ?Sized>(&mut self, tree: &T);
        pub fn topology_changed(&mut self);
        pub fn invalidate_chain<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId);
        /// Computes when anything changed (the former take_changed); whether it did.
        pub fn compute<T: LayoutTree + ?Sized>(&mut self, tree: &T, available: Size, measure: &mut dyn LeafMeasurer) -> bool;
        pub fn compute_overlay<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId, available: Size, limit: Option<Size>, measure: &mut dyn LeafMeasurer);
        pub fn drain_geometry_changes(&mut self) -> GeometryChanges<'_>;
        pub fn computed_layout<T: LayoutTree + ?Sized>(&self, tree: &T, id: NodeId) -> Option<..>;   // 今の戻り値
    }
    ```
    `take_changed` は `compute` の中へ入り公開しない。型状態の pass（`Edit`・`Pending`・`Computed`）は作らない。engine が要する既存 geometry/query/diagnostic は Layout に移し、`Settled` は private の Kept/Updated とする。`lib.rs` の呼び出し順の doc は「slot を再利用する前に `reset`」の一行だけにする。`columns.rs` の `Record.incomplete: bool` を `enum RecordState { Complete, Incomplete }`。
  - engine の `Model` は `layout: kivact_layout::Layout` 一つを持つ（`layout_columns`・`layout_state` を消す）。
  - engine の `Changes` の `needs_layout`・`layout_changed`・`viewport_chain` を `layout: LayoutReasons`（bit の集合: `PASS`、`STYLE`、`VIEWPORT`）にする。node ごとの `Dirty::MEASURE` の bit は残す（cache の無効化）。`settled()` は `PASS` だけを消す（今の規則）。
  - `Model::settle(&mut self, engine: &mut arvane::Engine) -> Settled` を一つの関数にし、今の `relayout`（sync → layout → lint → sync）の順を持つ。`sync` の中の二度の `pack_child_orders` は一度にする。host の `layout()` の呼び出し（hit test の前、file drop、redraw、wheel）は `settle()` の呼び出しに置き換える（何もなければすぐ返る）。呼び出しの場所は減らさない（入力の時点の幾何が要るため）。
  - `tree/node.rs` の `sent_layout: Option<(Rect, Size, bool)>`・`sent_scroll` を `Sent<LayoutReport>`・`Sent<ScrollReport>`（`LayoutReport` の bool は `enum Clipping { No, Yes }`）、`Place.stacks: bool` を `enum Stacking { Flat, Stacks }`。
- **手順**: `kivact-layout/src/{lib.rs, columns.rs, pass_state.rs, tree.rs}` → engine の `model/{aggregate.rs, changes.rs, apply.rs, lifecycle.rs, interact/geometry.rs, interact/sync.rs}`、`check/layout_reference.rs`、`measure/leaf.rs`、`tree/node.rs`、`host/events.rs`・`host/pump.rs` の `layout()`。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-layout -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- layout`。状態の点検表。
- **規模**: M。**並列**: X-1 の merge を待つ。E-2〜E-4・E-5b〜E-7 と同時でよい（`tree/node.rs` を E-5b と共有: 手で合わせる）。

### E-5b presence と状態の動き

- **目的**: 設計書 8.5 節。付録 A の SM-E31。
- **決定済みの形**:
  - `tree/node.rs` の `Presence { Mounted, Leaving }` と `model/presence/state.rs` の `Presences { arriving, leaving, notify_exit }` を、node ごとの `enum NodePresence { Arriving, Mounted, Leaving(LeavingRole) }`、`enum LeavingRole { Root { notify: Notify }, Descendant }`、`enum Notify { Silent, OnComplete }` にする。`arriving`・`leaving` の集合は、遷移の関数（`model/presence` の中の一つ）だけが更新する索引として残す。
  - `model/state_motion/state.rs` の `colors_text`・`colors_base_text` の bool を `ColorChannels`（bit の集合: `TEXT`、`BASE_TEXT`）、`AnimPhase` を `StateAnimPhase` と改名する。
- **手順**: `tree/node.rs`、`model/presence/*`、`model/state_motion/*` と使う所。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- motion`。状態の点検表。
- **規模**: S。**並列**: X-1 の merge を待つ。E-2〜E-5a・E-6・E-7 と同時でよい。

### E-6 支援技術

- **目的**: 設計書 8.5 節。付録 A の SM-E23〜E26。
- **決定済みの形**:
  - `Changes` の `a11y_structure`・`a11y_subtrees`・`a11y_removed`・`a11y_focus` と `Dirty::A11Y_LOCAL_TEXT`・`A11Y_METADATA` を消し、変更は `A11yState.dirt: Dirt` に直接積む（`model/interact/sync.rs` の畳み込みを消す）。
  - `root: Option<NodeId>`・`empty_root` を `enum A11yPublish { Inactive, AwaitingFirstTree, Published { root: SemanticsId } }`。`ShellEvent::AccessibilityActivated` で `Inactive` → `AwaitingFirstTree`（次の公開で木の全部を送り `Published`）、`AccessibilityDeactivated` で `Inactive`。毎 pump の `window.accessibility_active()` を消し、`ShellWindow::accessibility_active` を trait と全部の実装（native の窓、embed の二つ、virtual）から消す。
  - `next_semantics_id`（`u64::MAX` から減る）を `struct SemanticsIdAllocator`（`next(&mut self) -> SemanticsId`）。hash は `Semantics<R>` の `derive(Hash)` を使う。
  - `Focused { target, visible: bool, reveal_at_commit: bool }` を `Focused { target, visibility: FocusVisibility, reveal: Reveal }`、`enum FocusVisibility { Hidden, Visible }`、`enum Reveal { No, AtCommit }`。`A11yState.focus` を消し、公開のときに focus の状態から読む。
- **手順**: `a11y/*`、`model/changes.rs`、`model/interact/sync.rs`、`model/focus/*`、`host/pump.rs`・`host/events.rs`（新しい event）、`arvane-shell` の `ShellWindow`、`arvane-native` と `arvane-virtual` の実装。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p arvane-shell -p arvane-native -p arvane-virtual -p arvane -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- relations landmark_roles image_semantics`。状態の点検表。
- **規模**: M。**並列**: A-3・X-1 の merge を待つ。E-2〜E-5b・E-7・A-6a と同時でよい（A-6a が arvane-native の file を移すので、`accessibility_active` の実装の削除は移した先で手で合わせる）。

### E-7 paint

- **目的**: 付録 A の SM-E08・E09。
- **決定済みの形**: `needs_redraw: bool`（初期値 true）を `enum Redraw { Clean, Needed }`（初期値 `Needed`）、`paint_due()` = `Redraw::Needed && drawable()`。`PaintList` の `stale: bool` と `built_for: Option<(Size, f64)>` を `enum ListState { Stale, Built { size: Size, scale: f64 } }`（初期値 `Stale`）。`pass: u64`（layout の pass の数）と `ink_gen: u64` を `Generation`。
- **手順**: `paint/state.rs`、`paint/list/*`、`host/events.rs`・`host/pump.rs` の redraw の判定。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check -p kivact-engine --all-targets --all-features`。状態の点検表。
- **規模**: S。**並列**: X-1 の merge を待つ。E-2〜E-6 と同時でよい（`drawable()` は E-2 が作る。先に merge されていなければ今の判定のまま書き、merge で合わせる）。

### E-8 engine の fragment と `Display::Contents`

- **目的**: 設計書 17.1 節の engine 側、18.21 節（fragment の透明性）、19（Display）。C-5 が core の木を消すための engine の準備。
- **決定済みの形**:
  - 設計 19.29 を優先し `NodeKind::Fragment` は足さない。fragment は `Display::Contents` の箱で表し、透明の判定を一つにする（core の flow の移行は C-5）。
  - `NodeTree::flat_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_`: fragment と `Display::Contents` の box を降りて、その子を順に並べる。親ごとの cache を持ち、topology の変化（insert、move、remove）と display の変化で無効にする。`tree` の中の生の子の列は `tree` module の private にし、外は `flat_children` と下の `Reader` の二つだけを使う。layout の adapter（`tree/layout_adapter.rs`）、paint の walker（`paint/list/walk.rs`）、a11y の build（`a11y/build/*`）、hit test、focus の移動はこれだけを使う。`Display::Contents` の box は支援技術に自分の semantics を出さず、子を親へ上げる。
  - `Reader::parent(&self, id: NodeId) -> Option<NodeId>`（fragment を含む論理の親）、`Reader::children(&self, id: NodeId) -> Vec<NodeId>`（論理の子。退場中の node を除く）。
  - `Writer::remove_with_exit(&self, id: NodeId, pop_layout: bool, notify: bool) -> Vec<NodeId>`: 論理の木から外した subtree の id を preorder で返す（退場で残る node も含む）。`Writer::remove` は戻り値を捨てる。
  - 退場中の子孫を持つ fragment は、その子孫の退場が全部終わるまで残す（最後の一つが消えたときに消える）。
  - `kivact_types::layout::Display { Flex, Grid, Contents }`（既定 `Flex`）を足す。`kivact-style` の `grid_box: Option<bool>` を `display: Option<Display>` にし、`Style::display_grid()` を `Style::display(Display)` に置き換える（呼び出し元 4 か所: `kivact-core/src/control/container.rs`、`kivact-ui-soft/src/{aspect_ratio.rs, form_layout.rs, toolbar.rs}`）。解決済みの `LayoutProps` に `display: Display` を足す（`Grid` のときだけ grid の container の tracks が意味を持つ。今の `grid_box == Some(true)` の解決は `Display::Grid` で同じに行う）。`kivact-layout` の taffy の display は `hidden` なら `None`、ほかは `display` から（`Contents` は adapter が flat にするので taffy に届かない）。`Display` は prelude に入れない。
- **手順**: `contract/node.rs` → `tree/*` → layout の adapter・paint の walker・a11y の build・hit test・focus → `host/write/{reader.rs, writer.rs}` → `kivact-types/src/layout.rs`・`kivact-style/src/{sheet.rs, resolve.rs}`・`kivact-layout/src/style/convert.rs` → 呼び出し元 4 か所。core の flow は変えない（C-5）。
- **試験**: `dev/tests/kivact-public-api/tests/e2e/layout.rs`（なければ作り、`main.rs` に `mod` を足す）に `display_contents_lays_out_children_in_the_parent`（`Display::Contents` の box の子が親の flex の項目として並び、支援技術の木でも親の子になること）。
- **unit の確認**: build 可。`cargo check -p kivact-types -p kivact-style -p kivact-layout -p kivact-engine -p kivact-core -p kivact-ui-soft --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- layout`。
- **規模**: M。**並列**: E-3・E-5a・E-6・E-7 の merge を待つ（walker・hit test・a11y の build・adapter を共有）。E-9b と同時でよい。

### E-9a 出現と退場の大きさ

- **目的**: 設計書 17.10 節の engine 側と 19（presence の大きさ）。
- **決定済みの形**:
  - 軸は既存の `Orientation`（`Horizontal`・`Vertical`）を使う（`Axis` は作らない）。`Style::enter_size(self, axis: Orientation) -> Self`、`Style::exit_size(self, axis: Orientation) -> Self`。`TransitionProp::Size` を足す（大きさの動きの時間と easing）。
  - 規則: node の自然な大きさは通常の layout が決める（二度目の layout はしない）。動きの間、その軸の外形の大きさ = 自然な大きさ × 進み具合（出現は 0→1、退場は 1→0）で、親の layout はこの大きさを使う。子は自然な大きさで layout し、node は動きの間その軸を clip する。内容が途中で変わったら、次の layout の自然な大きさに同じ進み具合を掛ける。進み具合は `TransitionProp::Size` の transition で、`Style::enter`・`exit` の paint の動きと同じ時計（frame の要求、reduced motion の扱い）で進む。`TransitionProp::Size` がない node は大きさの動きをしない（即時）。`ExitLayout::Pop` の node では `exit_size` を無視する。
  - 実装: `kivact_layout::Layout::set_presence_scale(&mut self, slot: usize, scale: Option<(Orientation, f64)>)`。`kivact-layout/src/tree.rs` の `compute_cached_layout` の hook の中で、その slot の出力の大きさの軸だけに倍率を掛ける（子の配置は自然な大きさのまま。倍率を cache の依存に含め、変更時の無効化でもよい）。engine の presence の遷移（E-5b の `NodePresence`）が `Arriving`・`Leaving` の間、毎 frame 倍率を更新する。clip は paint の walker がその軸の overflow を clip する。
- **手順**: `kivact-types/src/layout.rs` → `kivact-style/src/sheet.rs`（`enter_size`・`exit_size`・`TransitionProp::Size`）と resolve → `kivact-layout`（倍率）→ engine の `model/presence`・`model/state_motion`・paint の clip。
- **試験**: `dev/tests/kivact-public-api/tests/e2e/presence.rs`（なければ作り、`main.rs` に `mod` を足す）に `exit_size_collapses_the_node_before_it_is_removed`（`exit_size(Orientation::Vertical)` と `TransitionProp::Size` の node を閉じると、途中の frame で高さが 0 と元の間にあり、退場の完了で node が消えること。`Screen::advance` で時刻を進める）。
- **layout 境界**: 親が直接読む item style の size/min/max/basis だけ private adapter で倍率を反映し、子の natural style を維持する。倍率変更時は依存 cache を無効化してよい。E2E は親 auto 箱の縮小と Size だけの Entering→Present の寿命も確かめる。
- **unit の確認**: build 可。`cargo check -p kivact-types -p kivact-style -p kivact-layout -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- presence`。
- **規模**: M。**並列**: E-5b と E-8 の merge を待つ（layout の adapter と presence を共有）。E-9b と同時でよい。

### E-9b presence の値

- **目的**: 設計書 17.10 節の公開の部分。
- **決定済みの形**: `kivact_types::Presence { Entering, Present, Leaving }`（公開の正準パスは `kivact::view::Presence`。facade の module は N-D1 が作るので、この unit では `kivact_core::view` から再公開する）。engine は `NodePresence` の遷移（`Arriving` に入る、`Mounted` に入る、`Leaving` に入る）で `HostEvent::PresenceChanged { node: NodeId, presence: Presence }` を積む。core の `NodeRef::presence(&self) -> Value<Presence>` は、最初の呼び出しで node ごとの signal を作り（それまで event は捨てる。作る時点の値は engine の `Reader::presence(id) -> Presence` から読む）、event で更新する。node の破棄で signal を消す。
- **手順**: `contract/events.rs`・`host/write/reader.rs`・`model/presence` → core の `client/dispatch.rs`・`view/node_ref.rs`。
- **出現の寿命**: `NodePresence::Arriving(ArrivalPhase::{Pending, Animating})` とし、未開始と動作中を node が明示的に所有する。arriving 集合は索引一つだけ。start は Pending だけを始め、完了で Mounted にする。公開値は両方 Entering。
- **試験**: `presence.rs` に `presence_reports_entering_present_leaving`（出現の動きの node の `presence()` が `Entering` → `Present`、閉じると `Leaving` になること）。
- **unit の確認**: build 可。`cargo check -p kivact-engine -p kivact-core --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features --test e2e -- presence`。
- **規模**: S。**並列**: E-5b と R-3 の merge を待つ。E-8・E-9a と同時でよい（`presence.rs` を E-9a と共有: 手で合わせる）。

---

## 6. Phase R: ランタイム

Phase R は `kivact-reactive`・`kivact-task`・`kivact-motion` を `kivact-runtime` 一つにし（R-1）、host との境界と turn の状態（R-2）、reactive の内部（R-3）、時間と動き（R-4）、非同期（R-5）、`Machine`（R-6）を直す。設計は設計書 19 章の runtime の節（4.4・6.1〜6.7・17.5・17.11・18.13・18.21 を置き換えたもの）に従う。turn の順序は core の `AppClient::turn` の一つの関数が持ち、runtime は部品を出す（`TurnClient` は作らない）。

並べ方: R-1 → `{R-2, R-3, R-6}` → `{R-4, R-5}`。R-6 は R-1 の後ならいつでもよい（既存の `spawn_in`・`sleep`・`effect_on`・`Owned` だけを使う）。

### R-1 `kivact-runtime` を作る

- **目的**: 設計書 4.1 節、19 章の module の向き。移すだけで、振る舞いも内部の形も変えない。
- **決定済みの形**:
  - `crates/kivact-runtime/src/{reactive,task,motion}/`（三つの crate の `src` を `git mv`）。三つは非公開の module。crate の root が旧三つの `lib.rs` の公開の集合をそのまま明示的に再公開する（`pub use reactive::{..}`、`pub use task::{..}`、`pub use motion::{..}`、`pub use reactive::keyed` と同じ形で `keyed`）。glob の再公開はしない。
  - `constant_value!` は `#[macro_export]` で crate の root に出る。中の `$crate::{IntoValue, Prop}` は root の再公開で解ける。macro のために module を公開しない。core の macro の入口は `kivact_runtime::constant_value` を再公開する。
  - `Transition` は V-6 の後の形（`kivact-types` の単一型の再公開）のまま移す。`as_wire` などの古い wrapper を作らない。
  - crate の属性: `#![forbid(unsafe_code)]`、`#![warn(missing_docs)]`、`clippy::type_complexity` の allow（reason は旧 reactive のもの）。
- **手順**:
  1. `crates/kivact-runtime` を作り、`crates/kivact-reactive/src/*` を `src/reactive/`、`crates/kivact-task/src/*` を `src/task/`、`crates/kivact-motion/src/*` を `src/motion/` へ `git mv` する。各 `lib.rs` は `reactive/mod.rs` などになり、`//! Role:` を一行にまとめ、crate の root の `lib.rs` に三つの Role の要約を書く。
  2. 中の path を直す: reactive の中の `crate::x` は `crate::reactive::x`（または `super::x`）、task・motion の中の `kivact_reactive::X` は `crate::X`（root の再公開）、motion の中の `kivact_task::X` があれば `crate::X`。
  3. `Cargo.toml`（workspace の `members`）から三つを消し `crates/kivact-runtime` を足す。`kivact-style`・`kivact-i18n`・`kivact-core` の依存を `kivact-runtime` にし、`use kivact_reactive::`・`kivact_task::`・`kivact_motion::` を `kivact_runtime::` に置き換える（core の中の約 100 file。機械的な置き換え）。facade の `kivact::task`・`kivact::motion` などの公開の path は変えない（core の再公開の module の中身だけ直す）。
  4. `dev/xtask/policy/workspace.toml`: `[visible_to]` の三行を `kivact-runtime = ["kivact-style", "kivact-i18n", "kivact-core"]` 一行にする。`[module_rules.kivact-motion.uses]` を消し、`[module_rules.kivact-runtime.uses]` に `reactive = []`、`task = ["reactive"]`、`motion = ["reactive"]` を書く（R-2 が `host`、R-6 が `machine` を足す）。skip の一覧と crate の一覧の三つの名前を `kivact-runtime` 一つにする。依存の図の comment（`kivact-reactive ◀ ..`）を直す。
  5. 0.9 節の一覧（三つの crate を消し `kivact-runtime` を足す）。`architecture/` は書き換えない。
- **試験**: 足さない。
- **unit の確認**: `cargo check -p kivact-runtime -p kivact-style -p kivact-i18n -p kivact-core -p kivact --all-features`。
- **規模**: M（移動は機械的だが file が多い）。**並列**: A-0 と同時に動いてよい。`Cargo.toml` の `members` と `workspace.toml` は手で合わせる。

### R-2 host との境界と turn の状態

- **目的**: 設計書 19 章の「host との境界と構築」「turn の順序は core の一つの関数が持つ」。付録 A の SM-R04、R05、R06、R09、R10、R19、SM-C02、C05、C06。
- **決定済みの形**:
  ```rust
  // reactive に定義し root から再公開
  pub trait Host: 'static { fn wake(&self); }               // Quiet → WakeSent の時だけ呼ぶ
  #[derive(Clone)] pub struct ThreadWaker(Arc<dyn Fn() + Send + Sync>);
  impl ThreadWaker { pub fn new(f: impl Fn() + Send + Sync + 'static) -> Self; pub fn wake(&self); }
  pub enum Stage { Setup, Begin, Reports, Dispatch, Async, Commit, Mounts, UserEffects, Output }
  enum TurnState { Quiet, WakeSent, Running(Stage) }      // 非公開
  pub enum ExitState { Running, Draining { deadline: u64 }, Exited }
  pub struct TurnReport { pub next_deadline: Option<u64>, pub exit: ExitState }
  #[must_use] pub struct TurnGuard<'a> { /* &Runtime と記録中の turn（旧 TurnRecorder を吸収） */ }

  // host（新しい module）に定義し root から再公開
  pub struct RuntimeConfig {
      pub start_ns: u64,
      pub blocking_threads: Option<usize>,
      pub at: &'static Location<'static>,
  }
  impl Runtime {
      pub fn new(host: Rc<dyn Host>, waker: ThreadWaker, config: RuntimeConfig,
                 build: impl FnOnce(&Rc<Runtime>, Owner) -> Extensions) -> Rc<Runtime>;
      pub fn root(&self) -> Owner;
      pub fn now_ns(&self) -> u64;
      pub fn advance_clock(&self, ns: u64);
      pub fn request_exit(&self);
      pub fn begin_setup(&self) -> TurnGuard<'_>;
      pub fn begin_turn(&self, events: usize) -> TurnGuard<'_>;
      pub fn report_now(&self) -> TurnReport;
      pub fn mounts_due(&self, acked: Revision) -> bool;
      pub fn pending(&self) -> Pending;          // turn_requested = (TurnState == WakeSent)
  }
  impl TurnGuard<'_> {
      pub fn stage(&self, stage: Stage);
      pub fn cause(&self, cause: String);
      pub fn stabilize(&self);
      pub fn deliver_reports(&self);
      pub fn run_mounts(&self, acked: Revision);
      pub fn stamp_mounts(&self, revision: Revision);
      pub fn check_draining(&self);
      pub fn settle_async(&self);
      pub fn plan_frame(&self) -> Option<u64>;
      pub fn run_user_effects(&self) -> bool;
      pub fn abandon_user_effects(&self) -> Vec<NodeId>;
      pub fn finish(self, ops: usize, client_pending: bool) -> TurnReport;
      pub fn finish_setup(self);
  }
  ```
  - 規則:
    - `Runtime::new` の順: registry に登録（`Running(Stage::Setup)`、内部の `ext: OnceCell<Extensions>` は空）→ root owner を `config.at` で作る（`Key` で持ち、`Option` にしない）→ root の setup の frame で `build(&rt, root)` → 戻り値に `TaskState::new(blocking_threads, waker)` と `MotionState` を足して `ext` に入れる → `Quiet` にして返す。`build` が panic したら登録を外す（guard の `Drop`）。`ext()` が空の `OnceCell` を読んだら misuse の panic（`ErrorKind::SetupOnly` ではなく、新しい kind を足さずに located な invariant の message: 「extension read while the App is being constructed」）。
    - 時計は runtime の `Cell<u64>` 一つ。`start_ns` で始まり、`advance_clock` は `max` で進める。`clock_now()` はこの値。
    - `request_turn`（書き込みなどが呼ぶ）: `Quiet` なら `WakeSent` にして `Host::wake`。`WakeSent`・`Running(_)` では何もしない。
    - `begin_turn`・`begin_setup` は `Quiet`・`WakeSent` から `Running` にする。`Running` の中で呼んだら misuse（turn は入れ子にならない）。
    - 段階は runtime のメソッドが自分で記す（`deliver_reports` → `Reports`、`run_mounts` → `Mounts`、`check_draining`・`settle_async` → `Async`、`run_user_effects` → `UserEffects`、`finish`・`report_now` → `Output`）。core は `stage(Dispatch)`・`stage(Commit)` を記す。`stabilize` は段階を変えない。
    - misuse はこの三つだけ: turn の入れ子、`Stage::Commit` の外の `stamp_mounts`、構築中の `ext()`。段階ごとの signal の書き込みの禁止は作らない（commit の中の focus の失敗が `on_error` を走らせるため）。
    - `on_mount` の待ち: `unstamped: Vec<Mount>` と `awaiting: VecDeque<(Revision, Mount)>`。`stamp_mounts(rev)` は unstamped を全部 `rev` で awaiting へ移す（rev は前より小さくない。invariant）。`run_mounts(acked)` は `rev <= acked` のものを先に取り出してから、queue の順に、生きている owner のものだけを handler の frame で実行する。実行中に積まれたものは今回の snapshot に入らない（現在の `mounts.rs` と同じ）。
    - `deliver_reports` は始めに snapshot を取り、報告の handler が積んだ報告は次の turn へ（現在の `report.rs` と同じ）。
    - `settle_async` は現在の `client/settle.rs` の本体（1000 round、round ごとに締切の来た timer を起こし、task を poll し、cause を記し、stabilize。上限で残ったら `Cycle` を既定の sink へ）。
    - `ExitState`: `request_exit` は `Running` のときだけ `Draining { deadline: now + DEFAULT_DRAIN_NS }`（saturating）。`finish`・`report_now` の `Output`: `Draining` で queue が空なら `Exited`、締切を過ぎたら全 queue を捨て `ErrorKind::Closed` を報告して `Exited`。`next_deadline` は `Draining` なら task の締切と drain の締切の小さい方、ほかは task の締切。
    - `finish`: `Output` の遷移 → turn の記録（`ops`）→ user effect・ready の task・後の報告・`client_pending` のどれかが残れば `WakeSent` にして `Host::wake`、なければ `Quiet`。`finish_setup`: setup の記録を閉じ（現在の `finish_setup_notes`）、user effect か ready の task が残れば `WakeSent` と wake、なければ `Quiet`。
- **手順**:
  1. runtime（commit を分ける）: `reactive/` に `Host`・`ThreadWaker`・`Stage`・`TurnState`・`ExitState`・`TurnReport`・`TurnGuard` を置く（file の名前は自由）。`Runtime` の field から `driver`・`root_owner`・`turn_requested`・`mounts: RefCell<Vec<PendingMount>>` を消し、`host: Rc<dyn Host>`・`state: Cell<TurnState>`・`exit: Cell<ExitState>`・`clock: Cell<u64>`・`root: Key`・`ext: OnceCell<Extensions>`・二つの mount の queue を足す。`Driver` trait、`reactive/driver.rs`（`clock_now` は残す）、`Runtime::create_root`・`clear_turn_request`・`Mounts`・`TurnRecorder`・`Runtime::turn(events)`・旧 `Runtime::new(extensions, driver)` を消す。`src/host.rs`（新しい module）に `RuntimeConfig`・`Runtime::new`・`request_exit`・`report_now` と、task・motion を使う `TurnGuard` のメソッド（`check_draining`・`settle_async`・`plan_frame`・`finish`）を `impl` で書く。`task/executor.rs` の `waker: Arc<Mutex<Option<ReadyWaker>>>` と `set_waker` を消し、`Executor::new(waker: ThreadWaker)`・`TaskState::new(blocking_threads, waker)` にする。`workspace.toml` に `host = ["reactive", "task", "motion"]`。
  2. core（commit を分ける）:
     - `client/link.rs`・`client/phases.rs`・`client/settle.rs` を消す。`ClientHost(HostWaker)` を `client/` に置き、`kivact_runtime::Host` を実装する（`wake` は `HostWaker::wake`）。
     - `AppClient` の `rt` を `Launch::Started(Started { rt })` に移す。`Launch::Waiting` は `AppConfig` と blocking の thread 数を持つ（`AppConfig` の形は C-1 まで保つ）。
     - `start(info, waker, writer)`: `Runtime::new(Rc::new(ClientHost(waker.clone())), ThreadWaker::new(move || waker.wake()), RuntimeConfig { start_ns: info.now_ns, blocking_threads, at }, |rt, root| extensions)`。`extensions` は今までの core の entry（`ViewState::new()`、gpu-interop の `State`、debug の `Trace`、`LifecycleState`、`HostState`）。`TaskState`・`MotionState` は runtime が足すので core は作らない。その後 `let setup = rt.begin_setup();`、今までの setup の順（`view::root::start` に `rt.root()` を渡す、title と最小の大きさ、font、`mount_root`、能力の通知、`setup.stabilize()`、`setup.stage(Stage::Commit)` の commit、trace）、`setup.finish_setup()`。
     - `turn`: 設計書 19 章の「core の `AppClient::turn` の並び」の 1〜6 のとおりに書き直す（phases と settle の中身をこの関数の中と小さな private な関数に入れる。commit は `fn commit(turn: &TurnGuard, rt, writer) -> usize` で、`plan_frame` → `writer.request_frame`、`requests::flush_focus`、削除と asset の失敗の掃除、`pending_writes` と `Writer::commit`（0 なら現在の revision）、`turn.stamp_mounts(revision)`）。`output()` は `TurnReport` を `TurnOutput { next_wake_ns: report.next_deadline, exit: matches!(report.exit, ExitState::Exited) }` に写す一つの関数にする。
     - `client/state.rs`: `ExitPhase`・`link`・`request_close`・`advance_clock` を消す（`HostState` は `turns`・`stale_events`・`asset_failures` だけ）。`client/dispatch/env.rs`: `frame_tick` は `rt.advance_clock(now_ns)` の後に motion、`close_requested` は `rt.request_exit()`。
     - `Drop for AppClient`: `Launch::Started` のときだけ今の処理（`rt.root().dispose()` など）。
     - `test/screen/driver.rs` と `scene_cache_tests.rs`・`window_tests.rs`: runtime は host の start の後にだけ読める。driver は host を作った後に runtime を取る（`AppClient` を共有する `Rc<RefCell<..>>` から読む口を一つ作る。名前は自由）。debug の trace の名前は start の前に `AppClient` に渡し、`build` の中で `Trace` を開く。`pending.turn_requested` の読み方はそのまま（意味は `WakeSent`）。
     - debug の trace の stage の label は `Stage` の名前に合わせてよい（trace の比較はもうない）。
- **試験**: 足さない。全 E2E が回帰の網。
- **unit の確認**: `cargo check -p kivact-runtime -p kivact-core -p kivact --all-features`、`cargo test -p kivact-public-api-tests --test e2e -- runtime:: setup_functions:: task:: motion:: lifecycle:: embed:: disposal::`（A-1b の前は `bash dev/tools/window-lock.sh --` を通す）。
- **規模**: M。サブエージェントは一つ。**並列**: R-3・R-6 と同時。R-3 とは `reactive/data.rs`（R-2 は `Runtime` の field、R-3 は `Computation`・`Ctx`・`OwnerEntry` と `writer`・`stab_epoch`）を手で合わせる。

### R-3 reactive の内部の状態

- **目的**: 設計書 6.4 節と 19 章の「reactive の frame と Watcher」、19 章の「内部の handler の型の置き場所」。付録 A の SM-R01〜R03、R07、R08、R11、R12。
- **決定済みの形**:
  ```rust
  enum CompState { Clean, Stale(Staleness), Queued(Staleness), Running }
  enum Staleness { Check, Dirty }
  struct PassRuns { epoch: u64, runs: u16 }    // fn note_run(&mut self, epoch: u64) -> Result<(), CycleDetected>
  enum OwnerPhase { Live, Disposing { round: u8 } }   // Gone = arena にない
  struct Frame { owner: Option<Key>, kind: FrameKind, tracking: Option<CollectorSlot>, pure: bool, writer: Option<Key> }
  enum FrameKind { Handler, Setup(Key), Computation { node: Key, effect: bool } }
  struct SelectorCache<K> { current: Option<K>, nodes: HashMap<K, Key>, prune: PruneSchedule }

  // reactive/handler.rs（root から再公開）
  pub enum Handled { Yes, No, PreventDefault }   // core の view/input.rs から移す（`and` も）
  #[derive(Clone)] pub struct Handler<E>(Rc<dyn Fn(E) -> Result<Handled, UiError>>);
  impl<E: 'static> Handler<E> {
      pub fn from_fn<R: HandlerResult>(f: impl Fn(E) -> R + 'static) -> Self; // Ok(()) は Handled::No
      pub fn call(&self, event: E) -> Result<Handled, UiError>;
  }
  ```
  - queue に key がある ⇔ `Queued`。push と pop は `CompState` の遷移のメソッドだけが行う。`abandon_render_effects`・`abandon_user_queue` は `Queued → Clean`。
  - `pure_section`・`untracked_frame`・`callback_frame` は親の frame の `kind` を写し、`pure`・`tracking`・`owner` だけを変える（現在の `..self.context()` と同じ意味）。`setup_frame` は `Setup(owner)`、`tracking_frame` は `Computation`、`handler_frame` は `Handler`。`writer` は frame が持ち、`Runtime.writer: Cell` を消す。`running_effect()` は一番上の frame の `writer`。
  - 破棄中の親の下での owner の作成は、一度も Live にならない handle を返す（現在の tombstone と同じ振る舞い）。
  - `Watcher`: graph の node の種類。自分の queue（`EffectKind::User` か `Internal`）の最初の実行で source を pure・tracking で評価して依存を取るだけで `f` を呼ばない。以後の再実行のたびに source を評価し、`f` を untracked で呼ぶ。値の比較はしない。crate の中の関数（例 `pub(crate) fn watch<T>(source, kind, f)`）で作る。`effect_on` は `kind = User` の Watcher にする（`first: Cell<bool>` を消す）。
  - pure の中の書き込みの misuse（`ErrorKind::WriteInPureContext`）は既にある。変えない。
- **手順**: `reactive/data.rs`（`Computation`、`Ctx`→`Frame`、`OwnerEntry.disposing`→`phase`、`stab_epoch`/`stab_runs`→`PassRuns`、`writer`）、`reactive/turn.rs`、`reactive/graph.rs`、`reactive/frame.rs`、`reactive/owner.rs`、`reactive/selector.rs`、`reactive/setup.rs`（`effect_on`）。`reactive/handler.rs` を作り、`Handled` を core の `view/input.rs` から移す（`HandlerResult` は `error.rs` のまま。移動と `KeyHandlerResult` との統合は C-3a）（core は `kivact_runtime::Handled` を import する。`view::Handled` の公開の path は core の再公開で保つ）。
- **試験**: 足さない。
- **unit の確認**: `cargo check -p kivact-runtime -p kivact-core --all-features`、`cargo test -p kivact-public-api-tests --test e2e -- runtime:: setup_functions:: flows:: disposal:: components::`。
- **規模**: M。**並列**: R-2・R-6 と同時（`reactive/data.rs` を R-2 と手で合わせる）。

### R-4 時間と動きの状態

- **目的**: 設計書 19 章の「frame と animation の時間」。付録 A の SM-R16、R22〜R25、SM-C23（motion の Weak の表）。
- **決定済みの形**:
  ```rust
  // task（timer）
  enum SleepState { Unregistered { deadline: u64 }, Registered { id: TimerId }, Fired }
  struct Scheduler { timers: BTreeMap<(u64, TimerId), Waker>, next_id: u64, cancel: CancelQueue }
  // motion
  struct FrameClock { phase: FramePhase, offset: ClockOffset, ticks: u64 }
  enum FramePhase { Idle, Requested { at: u64, last_tick: Option<u64> }, Ticked { last: u64 } }
  enum LoopState { Stopped, Running(LoopRun) }   // LoopRun は今の struct（PartialEq を足す）
  struct LoopCell { owner: Owner, f: Handler<FrameInfo>, state: Signal<LoopState>, frames: Cell<u64> }
  pub(crate) enum AnimPhase { Idle, Running { start_ns: u64 }, Paused { start_ns: u64, at_ns: u64 }, Completed }
  #[derive(Clone, Copy)]
  pub struct MotionEnv { pub reduced_motion: Value<bool>, pub surface_active: Value<bool>, pub frame_period_ns: Value<u64> }
  ```
  - `Sleep` の `Drop` は `CancelQueue`（`RefCell` を借りない `kivact_types` の `LocalMailbox`）に id を積む。`take_due` と `next_deadline` は heap を読む前に必ず `CancelQueue` を空にする。
  - frame loop は `FrameTick` の event の中で同期に走る（今の `on_frame_tick` の順: clock の適用 → 期限の来た loop → animation の step）。`next_frame()` と task の frame loop は作らない。`FrameLoop::rate()` は `state` の memo。`LoopCell.run`・`rate` を消す。
  - `MotionEnv` は core が `Runtime::new` の `build` の中で `Extensions` に入れる。motion は `rt.ext::<MotionEnv>()` で読む。`MotionState.env: Cell<Option<MotionEnv>>`・`attach_env` を消す。`RuntimeConfig.reduced_motion`・`Runtime::set_reduced_motion` は作らない。`reduce_motion()` の呼び出し（`env_changed`）は今のまま。
  - `Animated`: `phase: Signal<AnimPhase>` 一つ（`running: Signal<bool>`・`phase: Cell<Phase>`・`completed: Signal<u64>` を消す）。自然に終わった run は `Completed`、それ以外の終わり（`jump_to`、同じ値への target）は `Idle`。`is_running()` は `Running` の memo。`on_complete(f)` は `phase` を source にした `Internal` の Watcher で、見た値が `Completed` のときに owner の handler の frame で `f` を呼ぶ（`seen: Cell` を消す）。初回の skip は Watcher（`explicit_start` のときは普通の Internal effect）。
  - frame loop と animation の Weak の表は `kivact_types::Registry`。
- **手順**:
  1. `task/time.rs`・`task/state.rs`（`TimeState`→`Scheduler`）。
  2. `motion/state.rs`（`Frames`→`FrameClock` と二つの `Registry`、`applied_offset_ns`→`ClockOffset`、`MotionEnv` の読み方）、`motion/frame_loop.rs`、`motion/animated.rs`。frame loop の関数を `Handler<FrameInfo>` にする（R-3 の型）。
  3. core: `view/root.rs` の `mount_root` の `attach_env` を消す。`client/app_client.rs` の `build` の中で `EnvState` を作り（今の `view::root::start` から移す）、`MotionEnv { reduced_motion: env.field(..), surface_active: env.surface_active.into_value(), frame_period_ns: env.field(..) }` を `Extensions` に入れ、`EnvState` を `build` の外へ返して `view::root::start` に渡す（`ViewState` を完成形にするのは C-1）。
- **試験**: 足さない。animation・frame loop・timer の E2E が回帰の網。
- **unit の確認**: `cargo check -p kivact-runtime -p kivact-core -p kivact-interaction -p kivact-ui --all-features`、`cargo test -p kivact-public-api-tests --test e2e -- motion:: task:: interaction_delayed:: interaction_toast:: interaction_tooltip:: environment::`。
- **規模**: M。**並列**: R-5 と同時。`task/state.rs`（R-4 は timer、R-5 は pool と drain）と `kivact-runtime/src/lib.rs` を手で合わせる。

### R-5 非同期の状態

- **目的**: 設計書 19 章の「非同期の状態」。付録 A の SM-R13〜R15、R18、R21（SM-R17 は行わない）。
- **決定済みの形**:
  ```rust
  struct AsyncCell<T> { state: Signal<AsyncState<T>>, latest: Signal<Option<T>> }
  impl<T: Clone> AsyncCell<T> {
      fn start(&self);                                // Pending
      fn finish(&self, result: Result<T, UiError>);   // Ok は Ready と latest、Err は Failed（latest は保つ）
      fn reset(&self);                                // Idle
  }
  enum Latest { Idle, Running { task: Owned<TaskCell>, generation: Generation } }
  enum ActionPhase<I> { Open { worker: Worker, waiting: VecDeque<Submitted<I>> }, Closed { worker: Worker, waiting: VecDeque<Submitted<I>> }, Detached(Drain) }
  enum PoolState { NotStarted, Running(Arc<Pool>), ShutDown }
  ```
  - `Resource` と `Action` の状態は `AsyncCell` を通してだけ書く。公開の `state()`・`latest()` はそれぞれの signal を読む（memo にしない）。
  - `Drain` は `Runner::Queue`・`Runner::All` からだけ作れる（`Runner::Latest` は `Detached` にならない）。`AsyncState::Pending` ⇔ worker が `Running|Picking` か `waiting` が空でない、を遷移の関数の中で保つ。`worker_ids` は `Generation`。
  - `TaskState.queues` は `Registry<dyn Drain>`。`draining` は `Detached` への遷移で移される型付きの queue。
  - `PoolState::ShutDown` の後の `blocking` は invariant の panic（App の code は shutdown の後に走らない）。`ErrorKind` は足さない。
  - `TaskCell.task: OnceCell` は残す（future が cell を保存した後の key を持つため）。
  - task の `DoneFn` を `Handler<Result<T, UiError>>` にする（R-3 の型）。`on_done` の公開の署名は変えず、`Handler::from_fn(move |r: Result<T, UiError>| f(&r))` で包み、handler ごとに結果を clone して渡す。
- **手順**: `task/async_state.rs`（`AsyncCell` を足す）、`task/resource.rs`、`task/action.rs`、`task/latest.rs`、`task/state.rs`（pool・drain）、`task/blocking.rs`、`task/worker_pool.rs`。
- **試験**: 足さない。
- **unit の確認**: `cargo check -p kivact-runtime -p kivact-core --all-features`、`cargo test -p kivact-public-api-tests --test e2e -- async_tests:: task:: disposal::`、`cargo check -p kivact-example-async`。
- **規模**: M。**並列**: R-4 と同時（`task/state.rs` を手で合わせる）。

### R-6 `Machine`

- **目的**: 設計書 18.13 節と 19 章の「`Machine` の実装の規則」。
- **決定済みの形**:
  ```rust
  // kivact_runtime::machine（root から再公開。facade から出すのは N-D1）
  pub trait Machine: Sized + 'static {     // Self は遷移が読む入力の snapshot
      type State: Clone + PartialEq + 'static;
      type Event: 'static;
      type Effect: 'static;
      fn transition(&self, state: &Self::State, event: Self::Event) -> Step<Self::State, Self::Event, Self::Effect>;
  }
  pub struct Step<S, Ev, Ef> { /* 非公開: next: Option<S>, effects: Vec<Ef>, timers: Vec<TimerOp<Ev>> */ }
  impl<S, Ev, Ef> Step<S, Ev, Ef> {
      pub fn stay() -> Self;
      pub fn to(next: S) -> Self;
      pub fn with(self, effect: Ef) -> Self;
      /// Send `event` after `delay`, replacing a pending timer of the same event variant.
      pub fn after(self, delay: Duration, event: Ev) -> Self;
      /// Cancel the pending timer of `event`'s variant (the value is only used for its variant).
      pub fn cancel(self, event: Ev) -> Self;
  }
  /// Setup only. `input` builds the machine value (its input snapshot) for each event, untracked.
  #[track_caller]
  pub fn machine<M: Machine>(initial: M::State, input: impl Fn() -> M + 'static,
                             run: impl Fn(MachineCx<M>, M::Effect) + 'static) -> MachineHandle<M>;
  pub struct MachineHandle<M: Machine> { /* Owned<MachineCell<M>> */ }
  impl<M: Machine> Clone for MachineHandle<M> { .. }   // 手で書く
  impl<M: Machine> Copy for MachineHandle<M> {}
  impl<M: Machine> MachineHandle<M> {
      pub fn send(&self, event: M::Event);
      pub fn state(&self) -> Value<M::State>;
      pub fn get(&self) -> M::State;                    // tracked
      /// Whenever `source` changes (not for its initial value), send `to_event(new value)`. Setup only.
      pub fn feed<T: Clone + PartialEq + 'static>(&self, source: impl IntoValue<T>, to_event: impl Fn(T) -> M::Event + 'static);
  }
  pub struct MachineCx<M: Machine> { /* MachineHandle<M> */ }
  impl<M: Machine> MachineCx<M> { pub fn send(&self, event: M::Event); }
  ```
  - `send`: その machine の遷移の実行中なら queue に積む。そうでなければその場で `input()` を untracked で作る → `transition` → `next` があれば state の `Signal` に書く → timer の操作 → effect を順に `run` で実行する（machine の owner の handler の frame、untracked）→ 積まれた event を同じ順で処理する。実行中かどうかと queue は `enum Dispatch { Idle, Running(VecDeque<Ev>) }` 一つ。
  - timer: event の variant（`std::mem::discriminant`）ごとに一つの `Task`。`after` は同じ variant の task を `cancel` してから `let wait = sleep(delay);` を先に作り、`spawn_in(owner, async move { wait.await; handle.send(event) }, at)` で作る。`cancel` は同じ variant の task を `cancel` する。owner の破棄で task は消える。
  - `feed` は `effect_on(source, move |v| handle.send(to_event(v.clone())))`。
  - 状態の値は arena の `Signal<State>` 一つ。破棄された machine への `send` は破棄された `Signal` への書き込みと同じ misuse（`Disposed`）。
  - `Trigger` と `trigger` は既に `kivact-runtime` の公開にある。facade の `kivact::state` から出すのは N-D1。
- **手順**: `crates/kivact-runtime/src/machine.rs`（または `machine/`）を作り、root から再公開する。`workspace.toml` に `machine = ["reactive", "task"]`。rustdoc に、`useReducer` に effect と timer を足したもの、と例を書く（doctest は `ignore` にしない。`kivact_runtime` だけで書けない部分は `no_run`）。
- **試験**: この unit では足さない（public-api の E2E は facade だけに依存するので、N-G5 が `dev/tests/kivact-public-api/tests/e2e/machine.rs` に次を足す: `a_machine_renders_its_state_and_runs_effects_after_the_transition`、`a_machine_timer_fires_once_and_is_replaced_by_the_same_event_variant`、`feed_turns_an_input_change_into_one_event`、`a_machine_timer_is_canceled_with_its_owner`、`send_inside_a_transition_effect_is_queued_not_reentrant`）。
- **unit の確認**: `cargo check -p kivact-runtime --all-features`、`cargo test -p kivact-runtime --doc`。
- **規模**: S。**並列**: R-1 の後、R-2〜R-5 のどれとも同時（`kivact-runtime/src/lib.rs` と `workspace.toml` を手で合わせる）。

---

## 7. Phase C: 中核

W3 の順: C-1 → `{C-2, L-1, C-3a, C-3b}` → C-4 → C-5。C-5 は CP4 の後へ遅らせてよい（Phase N は C-5 に依らない。C-5 までは `Inspect::check_trees` が残る）。

### C-1 起動の経路と `kivact-test`

- **前提**: R-2（runtime を `Launch::Started` で作る）、E-2（embed の `CloseSignal` を engine が持つ）、E-8（`Reader::parent`・`Reader::children`）、A-1b（`MirrorMode`、`arvane-virtual` の台本）。
- **目的**: 設計書 7.1〜7.3 節と 19 章の「`AppOptions` と `Inspect`」。付録 A の SM-C01、C03、C04、C08、C28。
- **決定済みの形**:
  ```rust
  // kivact_core::launch（公開の module。kivact-test だけが使い、facade は再公開しない）
  pub struct AppOptions { /* 全部 private */
      title: String, window_size: Option<Size>, min_window_size: Option<Size>,
      theme: Theme, locale: Option<Locale>, messages: Messages,
      fonts: Vec<(Asset, &'static Location<'static>)>,
      blocking_threads: Option<usize>,        // None: min(4, CPU の数)
      gpu_preference: GpuPreference,          // feature gpu-interop がないときは読まない
  }
  impl AppOptions {
      pub fn new(title: impl Into<String>) -> Self;
      pub fn title(self, title: impl Into<String>) -> Self;
      pub fn window_size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self;
      pub fn min_window_size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self;
      pub fn theme(self, theme: Theme) -> Self;
      pub fn locale(self, locale: Locale) -> Self;
      pub fn messages(self, messages: Messages) -> Self;
      #[track_caller] pub fn font(self, font: impl Into<Asset>) -> Self;
      pub fn blocking_threads(self, n: usize) -> Self;
      #[cfg(feature = "gpu-interop")] pub fn gpu_preference(self, p: GpuPreference) -> Self;
      pub fn check_window_sizes(&self) -> Result<()>;   // 現在 App::run が行う検査
      pub fn host_options(&self) -> kivact_engine::api::HostOptions;
      pub fn window_title(&self) -> &str;
  }
  pub enum ClientMode { Window, Embedded(CloseSignal), Test(TestTiming) }
  pub struct TestTiming(pub Rc<dyn Fn(u64)>);           // 現在 into_client(Some(timing)) に渡している値
  pub struct Launch { /* enum Waiting { options, mode, root } | Started(Started) */ }
  impl Launch {
      pub fn new(options: AppOptions, mode: ClientMode, root: impl FnOnce() -> View + 'static) -> Self;
      pub fn into_engine_client(self) -> (kivact_engine::api::Client, LaunchHandle);
  }
  pub struct LaunchHandle { /* Rc の共有 */ }
  impl LaunchHandle { pub fn inspect(&self) -> Option<Inspect<'_>>; }   // Started のときだけ Some
  pub struct Inspect<'a> { /* Started を借りる */ }
  impl Inspect<'_> {
      pub fn reader(&self) -> kivact_engine::Reader;                   // engine の事実（semantics、layout、編集の値、scene、stats）は全部これで読む
      pub fn root(&self) -> NodeId;
      pub fn app_view(&self) -> Option<NodeId>;
      pub fn located_at(&self, node: NodeId) -> Option<&'static Location<'static>>;
      pub fn contains(&self, node: NodeId) -> bool;
      pub fn why(&self, node: NodeId) -> Why;
      pub fn stats(&self) -> Stats;                                    // core_ns は 0。kivact-test が自分の計測で埋める
      pub fn turns(&self) -> Vec<TurnRecord>;
      pub fn errors(&self) -> Vec<UiError>;
      pub fn pending(&self) -> bool;
      pub fn outstanding_blocking(&self) -> usize;
      pub fn motion_ticks(&self) -> u64;
      #[cfg(debug_assertions)] pub fn open_trace(&self, name: &str);
      #[cfg(debug_assertions)] pub fn check_trees(&self) -> Result<(), String>;   // C-5 で消す
  }
  ```
  `App` は `AppOptions` を一つ持ち、builder のメソッドはそのまま `AppOptions` の同名のメソッドを呼ぶ。`kivact::test::RenderOptions` は `AppOptions` を包む builder（field は private）: `AppOptions` と同名のメソッド（`window_size`、`min_window_size`、`theme`、`locale`、`messages`、`font`、`blocking_threads`、`gpu_preference`）と、試験だけのメソッド `host_locale(tag)`、`reduced_motion()`、`dark()`、`scale(f64)`、`fonts(TestFonts)`、`mirror(MirrorMode)`、`gpu()`、`frame_period_ns(u64)`、`title(..)`、`from_env()`、`with_theme(theme)`（`default().theme(theme)`）。名前が付録 B.1 の `kivact::test` と違うときは付録 B.1 に従う。`render_with_handle` は残す。engine の feature の名前は `testing` のまま。
- **手順**:
  1. `client/app_client.rs` の `AppConfig` と `view/root.rs` の `AppSettings` を消し、`AppOptions` にする。`App` の field を `AppOptions` 一つにする。窓の大きさの検査を `AppOptions::check_window_sizes` にする。
  2. 現在の private な `enum Launch`（`app_client.rs`）を上の `Launch` にする。`App::run` は `Launch::new(options, ClientMode::Window, root).into_engine_client()` を engine の `run` に渡す。`App::embed` は `ClientMode::Embedded` で同じことをする。`ClientMode::Embedded(CloseSignal)` は engine の `CloseSignal`（E-2）を持ち、今の event ごとの中断を保つ（設計書 19.8 節。`HostEvent::CloseRequested` と `Runtime::request_exit` の呼び出しは作らない）。`AppClient.embed_closing`・`with_embed_closing`・`embed_closing()` はこの `CloseSignal` に置き換えて消す（SM-C03）。
  3. `embed.rs` の `Embedded`: `host: Option<EmbeddedHost>` を `host: EmbeddedHost` にし、その後ろに宣言する guard の field（`Drop` で `DetachedThreads::any()` なら `retain_module()`、panic していなければ panic を resume）で drop の順を表す（field は宣言の順に drop される。doc comment に書く。`Embedded` の `Drop` の impl と `Option` を消す。SM-C04）。panic の扱いを `enum EmbeddedState { Running, Panicked(PanicPayload), Closed }` にする: 各メソッドは host を呼んだ後に一度だけ slot を見て、payload があれば `Closed` にして resume する。`Closed` の後は resume しない（SM-C28）。
  4. `ViewState` を `Launch::Started` の構築のときに完成した値として作る: `Started` の中身を `ViewState` の field にし、`started: OnceCell<Started>`・`ViewState::start`・`started()` の panic を消す（SM-C08）。`Extensions::new().with(ViewState::new())` を `Started` の構築へ移す。
  5. `crates/kivact-test` を作る。`kivact-core/src/test/` の公開 API（`render`、`render_with`、`render_with_handle`、`Screen`、`Element`、問い合わせ、`LayoutCheck`、`TestParent`、`TestFonts`）を移す。依存は `kivact-core`、`kivact-engine`（feature `testing`）、`kivact-runtime`、`kivact-types`、`kivact-style`、`kivact-i18n`。feature `gpu-interop`・`embed` を core へ伝える（engine 内部の経路は常時 compile、engine の feature は `testing` だけ）。
  6. `kivact-test` の中の core の内部の読み出し（`ViewState::tree(..)`、`HostState::of`、`TaskState::of`、`MotionState::of`、`rt.live_counts()`、`rt.turns()`、`why::explain`、`Trace::of`）を全部 `Inspect` のメソッドに置き換える。semantics の snapshot は `Inspect::reader()` の host の木（`Reader::children`）を辿り、位置は `located_at`（なければ一番近い祖先の位置）で付ける。`Screen` は `Rc<Runtime>` を持たず、`LaunchHandle` を持つ。`render` は `RenderOptions` の `AppOptions` から `Launch::new(.., ClientMode::Test(timing), ..)` を通る（現在の `Screen::start` の `AppClient::new(config, Some(2), at)` と、窓の大きさ・最小の大きさ・blocking の数の無視をなくす）。入力の注入は `Screen::script() -> kivact::test::Script`（A-1b の台本を包む型）を通す。
  7. `kivact-core/src/test/screen/*_tests.rs`（10 file、37 試験）を扱う: 公開 API で書けるものは `dev/tests/kivact-public-api/tests/e2e/` の該当の module へ移す。core と engine の木の一致などの内部の不変条件は `kivact-test` の debug の自己検査（pump の後に `Inspect::check_trees`）へ入れる。`real_window_tests.rs` の `#[ignore]` の `cost` は消す。消した試験と移した試験の名前を報告に書く。embed の E2E（`TestParent`）は `kivact::test::native_unavailable() -> Option<&'static str>` が `Some` のとき理由を出して skip する。
  8. facade: `kivact` の feature `test = ["dep:kivact-test"]`、`kivact::test` を `kivact_test` の再公開にする。`kivact-core` の feature `test` と `#[cfg(feature = "test")]`（`client/app_client.rs`、`client/state.rs`、`client.rs`、`diagnostics/stats.rs`、`lib.rs`、`view/tree.rs`）を消す。`Tree` の試験だけの読み出し（`input_value`、`placeholder`、`value`、`selected`、`checked`、`expanded`、`len`）は消す（`Inspect::reader()` で読む）。0.9 節の一覧。
- **状態の点検表**: 0.7 節。
- **既存試験の境界（C-1 限定）**: App 公開 API で書けるものは public-api E2E へ移す。core/tree/Owner の内部不変条件は debug 自己検査へ。rootless Client、worker/cache、renderer 選択・pool・期限・最小化等の既存 backend 契約は、新公開の raw 観測 API を増やさず `kivact-test` 内の private E2E として既存名/assert を保持してよい。この移管分だけの例外で、新たな backend-only 試験は足さない。engine 所有の統計は `Reader::stats() -> WriteStats` で読む。
- **試験**: 足さない（手順 7 で移す）。
- **unit の確認**: `cargo check -p kivact-core -p kivact-test -p kivact -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- <手順 7 で移した試験の名前>`。`grep -rn 'feature = "test"' crates/kivact-core` が 0 件。
- **規模**: L。サブエージェントは一つ。手順 1〜4 と 5〜8 で commit を分ける。
- **並列**: なし（W3 の最初）。

### C-2 i18n と style の純粋化

- **前提**: C-1。
- **目的**: 設計書 7.6 節と 19 章の「style の内部の口」。書式の関数は L-1。
- **決定済みの形**:
  - `kivact-i18n` は値と計算だけを持つ。`kivact-runtime` への依存は `constant_value!(Locale, Weekday)`・`constant_value!(Messages)` のためだけ。
  - 文脈の設置と読み出しは `kivact_core::view::boundary`: `install_locale`、`install_messages`、`install_layered_messages`、`LocaleCtx`、`MessagesCtx`、`locale()`（N で `use_locale`）、`message`、`message_with`。`kivact_core::i18n` が `message`・`message_with`・`locale` を core から、値の型と `interpolate` を `kivact_i18n` から再公開する。
  - `kivact_style::internal`（`#[doc(hidden)] pub mod`。core だけが使う。facade は再公開しない）の自由関数:
    ```rust
    pub fn install_theme(owner: Owner, theme: Value<Theme>);
    pub fn theme_of(owner: Owner) -> Option<Value<Theme>>;
    pub fn compile_style(style: &Style, theme: &Theme, state: Interactive) -> CompiledStyle;
    pub fn compile_span_style(style: &SpanStyle, theme: &Theme) -> CompiledSpanStyle;
    pub fn record(size: Size, theme: &Theme, f: impl FnOnce(&mut Draw)) -> Recording;
    pub fn with_scroll_axes(style: Style, axes: ScrollAxes) -> Style;
    ```
    同名の固有メソッド（`Theme::install`、`Theme::of`、`Style::compile`、span の `compile`、`Draw::record`、`Style::with_scroll_axes`）は消す。
  - `Style::merge` は公開のまま（kit が使う）。`Style::display_grid` は `Style::display(self, display: Display) -> Style`（`Display::{Flex, Grid, Contents}`。`Contents` は E-8 が足す）に置き換える。
  - `static THEMED`（`kivact-style/src/theme/value.rs`）は残す（19 章: `Color::of` は theme なしで Copy の `Color` を作るので、表を `Theme` に入れられない）。rustdoc に「`ThemeColor` の値ごとに一つの、縮まない interning の表」と書き、`#[expect]` の理由に同じことを書く。
- **手順**:
  1. `kivact-i18n/src/context.rs` の中身を `kivact-core/src/view/boundary.rs`（または `view/boundary/` の下の file）へ移し、`context.rs` を消す。`kivact-i18n` の `lib.rs` の再公開を直す。
  2. `kivact-style` に `internal` を作り、上の関数へ移す。core の呼び出し元（`view/mount/style.rs`、`view/span.rs`、`style.rs`、`view/boundary.rs`、`view/descriptor.rs`、`view/root.rs`、`draw/offscreen.rs`、`view/mount/draw.rs`、`diagnostics.rs`、`control/container.rs`）を直す。`kivact_style::Theme::install` などの署名から `Owner` が消えたことを確かめる（`internal` の外の公開の署名に `kivact_runtime::Owner` が出ない）。
  3. `Style::display` を足し、`display_grid` の呼び出し元（core の `control/container.rs`、soft の `aspect_ratio.rs`、`form_layout.rs`、`toolbar.rs`）を `.display(Display::Grid)` にする。
  4. `THEMED` の rustdoc と `#[expect]` を書く。
- **試験**: 足さない。
- **unit の確認**: `cargo check -p kivact-i18n -p kivact-style -p kivact-core -p kivact-ui -p kivact-ui-soft --all-targets --all-features`。
- **規模**: M。
- **並列**: L-1、C-3a、C-3b と同時。共有の file: `kivact-i18n/src/lib.rs`（L-1 と。足すだけ）、`kivact-core/src/view/boundary.rs`（C-3a と）。

### L-1 ICU4X による地域書式

- **前提**: C-1。
- **目的**: 設計書 17.14 節（旧 C-2 の書式の手順を含む）。
- **決定済みの形**（`kivact_i18n`、`kivact::i18n` から出る。ICU4X の型は公開しない）:
  ```rust
  pub struct TimeOfDay { pub hour: u8, pub minute: u8, pub second: u8 }
  pub struct DateTime { pub date: CalendarDate, pub time: TimeOfDay }
  pub enum HourCycle { H12, H23 }
  pub struct NumberStyle { pub fraction_digits: FractionDigits, pub grouping: Grouping }
  pub enum FractionDigits { Exact(u8), Range { min: u8, max: u8 } }
  pub enum Grouping { None, Locale }
  pub enum DateStyle { Short, Medium, Long, Iso }
  pub struct TimeStyle { pub seconds: bool }
  pub enum NameWidth { Narrow, Short, Long }
  pub enum PluralCategory { Zero, One, Two, Few, Many, Other }
  pub struct PluralForms { pub zero: Option<String>, pub one: Option<String>, pub two: Option<String>,
                           pub few: Option<String>, pub many: Option<String>, pub other: String }
  impl Locale {
      pub fn format_number(&self, value: f64, style: NumberStyle) -> String;
      pub fn parse_number(&self, text: &str) -> Option<f64>;
      pub fn format_date(&self, date: CalendarDate, style: DateStyle) -> String;
      pub fn parse_date(&self, text: &str) -> Option<CalendarDate>;   // ISO 8601 と、locale の短い日付の数字の順
      pub fn format_time(&self, time: TimeOfDay, style: TimeStyle) -> String;
      pub fn month_name(&self, month: u8, width: NameWidth) -> String;
      pub fn weekday_name(&self, day: Weekday, width: NameWidth) -> String;
      pub fn hour_cycle(&self) -> HourCycle;
      pub fn first_day_of_week(&self) -> Weekday;                       // 既存。ICU4X で
      pub fn plural_category(&self, n: f64) -> PluralCategory;
      pub fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering;
  }
  impl Messages {
      pub fn with_plural(self, key: &'static str, forms: PluralForms) -> Self;
      pub fn get_plural(&self, key: &str, category: PluralCategory) -> Option<&str>;
  }
  ```
  新しい値の型には `constant_value!` を付ける。`message_with` は引数に `("count", n)` があり key が plural のとき `plural_category(n)` で形を選ぶ。`kivact-i18n/src/date.rs` の `days_from_civil`・`civil_from_days` を公開の暦の計算（`CalendarDate::from_days(i64)`、`CalendarDate::to_days(self) -> i64`）にする。
- **手順**:
  1. `kivact-i18n` に ICU4X 2 系（作業の時点の最新の 2.x の `icu_decimal`、`icu_datetime`、`icu_calendar`、`icu_locale_core`、`icu_plurals`、`icu_collator`、compiled data、全 locale）を足し、上の API を実装する。`locale.rs` の `SUNDAY_START`・`SATURDAY_START` の手書きの表を消す。`parse_number` は ICU4X の locale の記号で kivact-i18n が解析する。
  2. `kivact-ui/src/number_input.rs` の `format_value`（step の精度の書式）を `locale.format_number(value, NumberStyle { fraction_digits: FractionDigits::Range { min: 0, max: places }, grouping: Grouping::None })` に、入力の解析を `parse_number` にする（locale は `use_locale()` の値）。
  3. `THIRD-PARTY-NOTICES.md` に ICU4X（Unicode License）を足す。L-1 の前後で `examples/kivact/counter` の release build の実行ファイルの大きさと build の時間を測り、`refactoring/PROGRESS.md` に書く。
- **試験**: 足す。`dev/tests/kivact-public-api/tests/e2e/i18n.rs` に `en_us_formats_numbers_dates_and_names`、`ja_jp_and_de_de_format_numbers_and_dates`、`plural_forms_follow_the_locale`、`collation_sorts_by_locale`（どれも書式の結果を text に出して確かめる）。
- **unit の確認**: `cargo check -p kivact-i18n -p kivact-core -p kivact-ui -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e i18n::`。
- **規模**: M。
- **並列**: C-2、C-3a、C-3b と同時。共有の file: `kivact-i18n/src/lib.rs`（C-2 と）。

### C-3a view・flow・木の状態と `HandlerResult`

- **前提**: C-1、R-4（`kivact_runtime::handler::Handler<E>`）。
- **目的**: 付録 A の SM-C07、C09、C11、C12、C13、C14、C18〜C22、C24、C25。設計書 17.5 節の core の slot。
- **決定済みの形**:
  - `kivact_runtime::handler` に `Handled` と `HandlerResult` を移して一つにする（`KeyHandlerResult` を消す。core の中だけが使っている）:
    ```rust
    pub enum Handled { Yes, No, PreventDefault } // and による既存の伝播・既定動作の合成も保つ
    pub trait HandlerResult: Sealed { #[doc(hidden)] fn into_outcome(self) -> Result<Handled, UiError>; }
    impl HandlerResult for () {}                   // Ok(Handled::No)
    impl HandlerResult for Handled {}
    impl HandlerResult for Result<(), UiError> {}  // Ok(()) は Ok(Handled::No)
    impl HandlerResult for Result<Handled, UiError> {}
    ```
    `#[diagnostic::on_unimplemented(message = "a handler returns `()`, `Handled`, `Result<(), UiError>` or `Result<Handled, UiError>`", note = "use `?` on other errors; they convert to `UiError`")]`。core の `view` が再公開する（正準パスは N-D1 で `kivact::view`）。
  - core の内部の callback の slot（`view/tree.rs` の `Handlers` の全 field、`PartActions`、`view/handler_layers.rs` の `EventHandlers<E>`、`view/editing_data.rs` の `SessionHandler`、`TNode.exit_complete`）は全部 `Handler<E>`。`Rc<dyn Fn ..>` を field に持たない。
  - 削除の結果（C-5 も同じ署名を使う）:
    ```rust
    #[must_use] pub(crate) struct Removed { pub(crate) ids: Vec<NodeId>, records: Vec<CoreNode> }  // ids は削除した部分木の全 node（前順）
    pub(crate) fn remove(&mut self, id: NodeId) -> Removed;
    ```
    commit の段階で `Removed.ids` を `asset_failures` から消す（`removes: bool`・`take_removes` を消す。SM-C07）。同じ ids で `NodeRef` を `Detached` にする（SM-C14）。
  - SM-C12: `leaving: IdMap<Leaving>`、`struct Leaving { on_exit: Option<Handler<()>> }`。
  - SM-C11: `TextEditsInFlight` は単一の共有 counter store（`Rc<RefCell<IdMap<u32>>>`）、`TextEditsAhead` は独立した所有 guard（begin/drop）。dispatch 中に Tree の可変借用を保持せず、store の二重 begin を拒否する。
  - `IdMap<V>` は `NodeId` の key だけ（`kivact-types/src/ids.rs`）。ほかの key は `HashMap`。
  - そのほかは付録 A の各項目の「解消後」の形（`Modality`、`DynamicArm<T>`、`Held<K>` と `Trigger`、`HookSlot`、`ModalScope`、`ReportOnce`/`ReportOnceSet`、`HostStats`、`is_initial_render() = matches!(turn, Turn::Setup)`、revision の trigger は `Tree::new` で作る）。
- **手順**:
  1. `Handled`・`HandlerResult` を runtime へ移し、`KeyHandlerResult` と `sealed::KeyHandlerResultSeal` を消し、core の bound を `HandlerResult` にする。
  2. core の slot を `Handler<E>` にする。
  3. 付録 A の上の項目を「解消後」のとおりにする。`ReportOnce`・`ReportOnceSet`・`Sent`・`Generation`・`Registry` は `kivact-types::cell`。
- **状態の点検表**: 0.7 節。
- **試験**: 足さない。
- **unit の確認**: `cargo check -p kivact-runtime -p kivact-core -p kivact-interaction -p kivact-ui -p kivact-public-api-tests --all-targets --all-features`。`cargo xtask check --no-write --only state-fields --crate kivact-core --module view`。
- **規模**: L。
- **並列**: C-2、L-1、C-3b と同時。共有の file: `kivact-core/src/view/state.rs`（C-3b と。C-3a は `pointer_modality`・`noted_missing` だけ、C-3b は focus の要求と `adapter` だけを変える）、`view/boundary.rs`（C-2 と）。

### C-3b focus の要求・lifecycle・gpu-interop の状態

- **前提**: C-1。
- **目的**: 付録 A の SM-C10、C23、C26、C27。設計書 17.11 節の `adapter`。
- **決定済みの形**:
  ```rust
  struct FocusRequests { requests: HashMap<RequestId, FocusRequest>, order: Vec<RequestId> }
  struct FocusRequest { target: FocusTarget, reply: FocusReply, stage: RequestStage, visible: bool }
  enum RequestStage { Queued, Sent }
  struct FocusCompletion { owner: Owner, at: &'static Location<'static> } // private
  enum FocusReply { Report(FocusCompletion), Restore, RestoreAndReport(FocusCompletion) }
  ```
  flush の restore 取消条件は既存のまま `Queued → 消える` とする。取消は内部の `Canceled` で、既存どおり callback・UiError を発生させない。host の拒否だけを保存した owner/location に報告する。`FocusCompletion` は runtime の future 用 Completion とは別。`platform/lifecycle.rs` の `LifecycleState` は `Registry<T>`、登録の id は `Generation`。SM-C26 の `enum Issue { NotIssued, Issued { mode: ShareMode, size: Size, generation: Generation } }`。SM-C27 の `Open.pixels` は `Mailbox<Pixels>`。`ViewState` の `adapter: Signal<Option<GpuAdapter>>` を消し、`gpu_interop::State` だけが持つ。
- **手順**: 上のとおり（`view/state.rs`・`view/requests.rs`、`platform/lifecycle.rs`、`gpu_interop/share/app_side.rs`・`link.rs`、`view/state.rs` の `adapter` と読む箇所）。
- **状態の点検表**: 0.7 節。
- **試験**: 足さない。
- **unit の確認**: `cargo check -p kivact-core -p kivact-public-api-tests --all-targets --all-features`。`cargo xtask check --no-write --only state-fields --crate kivact-core --module view`。
- **規模**: M。
- **並列**: C-2、L-1、C-3a と同時。共有の file: `kivact-core/src/view/state.rs`（C-3a と）。

### C-4 part の入力と事実の持ち主

- **前提**: C-3a、E-3（`Track`・`Route`、`HoverChanged { target: Option<FocusTarget> }`）。
- **目的**: 設計書 7.5 節。付録 A の SM-C15〜C17。
- **決定済みの形**:
  ```rust
  struct PartInput { press: Press, hover: Hover }   // Signal<PartInput> 一つ
  enum Press { Idle, Pointer(PointerId), Key(Key) }
  enum Hover { Outside, Inside }
  enum RegionEntry { Pending, Adopted(Adopted), Shadowed { by: RegionKey } }
  struct Adopted { owner: Owner, generation: Generation, binding: RegionBinding }
  enum RegionBinding { Unbound, Bound }
  enum PartCapture { None, Captured { target: CaptureTarget, buttons: PointerButtons } }
  enum CaptureTarget { Part(Rc<Target>), Region(RegionId), Fallback { button: MouseButton, target: Rc<Target> } }
  ```
  disabled と hidden は状態ではなく入力（`Value<bool>`、mount の `PartMount` から読む）。disabled か hidden になる遷移が `Press::Idle` と `Hover::Outside` にする。focus の in/out は `NodeRef::focused()`（engine の `FocusChanged`）だけで読み、`previously_focused` の edge の検出を消す。hover は engine の `HoverChanged.target` だけ: node の part は target.node が自分またはその子孫（engine の ancestry）、canvas の region の part は `target == FocusTarget { node, region: Some(region) }` のとき `Inside`。
- **手順**:
  1. `view/part.rs` の `InputState` の 5 つの `Signal<bool>` を `PartInput` にする。pointer の Enter/Leave の observer による hover を消す。`view/mount/regions.rs` の region の hover を `HoverChanged.target` から作る。
  2. `view/common.rs` と `view/mount/regions.rs` の region の登録を `RegionEntry` にする。変化の通知は `Trigger`、重複の報告は `ReportOnceSet`。
  3. `view/common.rs` と `view/mount/part_pointer.rs` の capture を `PartCapture` にする。engine は adopt された pointer の activation を抑えたかを `Routed.activation` に入れて渡し、`Reader::take_adopted_activation` と core の `client/dispatch/pointer.rs` の呼び出しを消す。`Writer::prepare_input` を各入力の dispatch 直前に一度呼ぶ（既存の `Reader::admit_input` bookkeeping は保つ）。`HostEvent::Activate.pointer` は `Option<Routed<PointerSample>>`（AT は None）とし、Up handler 後の採用・抑止を保持する。
- **状態の点検表**: 0.7 節。
- **試験**: 足さない（`parts.rs`、`canvas.rs`、`hover.rs`、`pointer.rs` の E2E が回帰の網）。
- **unit の確認**: `cargo check -p kivact-engine -p kivact-core -p kivact-interaction -p kivact-ui -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- parts:: canvas:: hover:: pointer::`。
- **規模**: M。
- **並列**: なし（C-3a の後）。

### C-5 core の木を side table にする

- **前提**: C-4、E-8（`Display::Contents` の透明の扱い。`Reader::parent(id) -> Option<NodeId>`、`Reader::children(id) -> Vec<NodeId>`（退場中の node を含まない）、`Writer::remove_with_exit(..) -> Vec<NodeId>`（論理的に消した部分木の全 id。退場中で残る node も含む）、退場中の子孫を持つ透明な箱は子孫の退場が終わるまで残る）。
- **目的**: 設計書 17.1 節と 19 章の「fragment は `Display::Contents` の箱」。
- **決定済みの形**: core の fragment は engine の箱（`NodeKind` の box）で、style は `display(Display::Contents)`、semantics なし、`hit_testable(false)`、focus しない。engine に fragment の種類を足さない。
  ```rust
  pub(crate) struct CoreNodes(IdMap<CoreNode>);
  pub(crate) struct CoreNode { owner: Owner, at: Option<&'static Location<'static>>, handlers: Option<Box<Handlers>>,
                               exit_layout: ExitLayout, exit_complete: Option<Handler<()>>,
                               observation: Option<NodeObservation>, origins: Vec<(Prop, EffectId)>,
                               references: Vec<Signal<RefState>>, presence: Option<Signal<Presence>> }
  ```
  C-3/E-9 の参照・presence の lifecycle source は metadata に保持する。所有 service は `ViewNodes { nodes: CoreNodes, next_id, root, engine, future_text_edits, leaving, visibility_revision, live_region_revision }`。engine の既存 `Writer::insert` は再配置にも対応し、detach 前の継承値を比較して旧・新親の layout と a11y を無効化する。
  `parent`・`children`・`kind: Option<NodeType>` を持たない。`Removed` の署名は C-3a のまま（`ids` は `Writer::remove_with_exit` の戻り値）。
- **手順**:
  1. flow（`show`、`dynamic`、`for_each`、`for_index`、`virtual_list` の行、`error_boundary`、`provide*`）が自分の位置に透明な箱を一つ作り、その中へ子を入れる（`Writer::insert(fragment, child, before)`）。
  2. `Tree` を `CoreNodes` にする。`host_parent`、`top_hosts`、`app_view`（root の最初の子にする）、`first_host`、`next_host_after`、`insert`、`attach_writes*`、`move_writes_before`、`remove_unlinked`、`first_host_of`、`set_children`、`delete_records` を消す。list と virtual list の並べ替えは行の箱を `Writer::move_before` で動かす。
  3. parent・children の問い合わせを `Reader::parent`・`Reader::children` にする: `find_up`（`client/dispatch/pointer.rs`）、`client/dispatch/key.rs` の親の walk、`control/container.rs`、`view/node_ref.rs`、`view/overlays.rs`、`Tree::is_within`・`is_visible`・`is_in_live_region`、`client/trace.rs`。
  4. `Inspect::check_trees` と、`kivact-test` の debug の木の一致の検査（`structure.rs` の `core_host_children`）を消す。
- **試験**: 足さない（flow・list・virtual list・overlay・event の bubbling の E2E が回帰の網）。
- **unit の確認**: `cargo check -p kivact-core -p kivact-test -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- flows:: overlay:: focus:: disposal::`。
- **規模**: M（E-8 が L）。
- **並列**: なし。CP4 の後へ遅らせてよい（遅らせたら W4 の後に単独で行い、その後の checkpoint の gate を通す）。

---

## 8. Phase N: 公開 API

この phase は公開 API を設計書 10 章・18 章・19 章と付録 B の形にする。名前と path の変更は全部「付録 B.10」（旧→新の表）が正で、unit の本文では一覧を繰り返さない。B.10 にない item の名前は変えない。

W4 の順: `{N-D1, N-D2}`（定義の段、build 不可でよい） → `{N-F1, N-F2, N-F3, N-F4, N-F5, N-F6}`（fan-out A、build 不可でよい） → orchestrator の `cargo check --workspace --all-targets --all-features` と組ごとの fix → `{N-G1, N-G2, N-G3, N-G4, N-G5}`（fan-out B） → N-8 → CP4。

fan-out の各 unit は、自分の crate の全部の file に Phase N の変更を全部当てる（同じ file を何度も書き直さない）:

1. 付録 B.10 の移動・改名・削除。
2. handler の形（付録 B.4）: payload のない handler は `impl Into<Callback<()>>`、payload のある handler は `impl Fn(E) -> R + 'static` で `R: HandlerResult`。位置引数の callback は handler のメソッドにする。
3. prop の形（付録 B.5）: 実行中に変わる性質は `impl IntoValue<T>`、数値は `impl Into<f64>`・`impl IntoValue<f64>`、範囲は `impl IntoValue<ScalarRange>`、二方向の値は `impl IntoBinding<T>`、選択は `impl IntoSelection<K>`、bound は `ItemKey`・`PropValue`。構造の option は設計書 10.6 節の一覧の引数なしのメソッドだけ。data の record と option の束は plain な値。
4. slot の形（付録 B.9）: 一度だけ mount するものは `impl IntoView`、作り直すものは `impl IntoViewFn<V>` か `impl Fn(A) -> V + 'static`、保存は `View` と `ViewFn<A>`。子を一つずつ包むものは `impl IntoChildren`。
5. modifier: `NodeExt` の import と呼び出しを `Node` の固有メソッドにする。部品の builder の後のメソッドで root を変える箇所は `with_node`。part は `Part::new(state, apply)` と `node.part(p)`。`Part::on_bind` は `Node::on_attach`。
6. `Rc`・`Arc`・`Box<dyn>`・`Location` を公開の署名から消す（付録 B.6）。
7. 一つの root を持つ独自の合成部品は `impl Component` を返す（設計書の「独自部品」）。標準・kit の constructor で固有の fluent option を公開するものは具体的な builder 型を返し、option を隠さない。primitive は `Node`、behavior は `*Api` のまま。

### N-D1 module・facade・prelude・geometry

- **前提**: C-1〜C-4（C-5 は要らない）。
- **目的**: 設計書 7.1・10.1・10.2 節、18.6 節、付録 B.1・B.2。
- **決定済みの形**: facade の module の一覧と各 module の item は付録 B.1、prelude は付録 B.2 の一覧そのもの、移動と改名は付録 B.10。root は `App`、`Result`、`UiError`、`ErrorKind` だけ。`kivact::geometry`（`Point`、`Size`、`Rect`、`Insets`、`Edge`。`kivact-types` の再公開）。
- **手順**:
  1. `kivact-core` の module を設計書 7.1 節の形にする: `control` の中身を `view`（`view::elements`）へ、`primitive` の中身を `custom` へ移し、`control`・`primitive` を消す。`state` module（runtime の handle・setup の関数・`Machine` 一式・`Notifier` と core の `Owner`、`use_app_state`）を作る。`geometry` module を作る。core の `prelude` を消す。
  2. facade（`crates/kivact/src/lib.rs`）を付録 B.1 の module の一覧にする。`ui` による `interaction` の型の再公開（`kivact-ui/src/lib.rs` の `pub use kivact_interaction::{..}`）を消す。`use_inline_edit` を `kivact-interaction` へ、`IntoChildren` を `kivact-core` の `view` へ移す。
  3. `crates/kivact/src/prelude.rs` に付録 B.2 の一覧を一つずつ書く（glob の再公開をしない）。N-D2 が作る名前も書く（build は fan-out の後に通る）。後の phase が作る名前（付録 B.2 に「後の unit が作る」と書いたもの）は書かず、作る unit が一行足す。`dev/xtask/policy/workspace.toml` の prelude の glob の `[[style.exceptions]]` を消す。
  4. 付録 B.10 のうち「削除」の行の item を消す（例: `Transition::as_wire` の残り、`Signal::id`、`Menu::behavior`、`Button::into_node`、`TextInput::into_node`、`F64Signal`）。`IntoValue::into_prop`・`IntoText::into_text` を `#[doc(hidden)]` にし、`Prop` を `kivact::state::__private` からだけ届くようにする（sealed にはしない）。
  5. G3・G4 の中の module の path の参照（`use` と doc link）だけを直す。G5・G6 は直さない（fan-out）。
- **試験**: 足さない。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: M。
- **並列**: N-D2 と同時。共有の file: `kivact-core/src/lib.rs`・`view.rs`・`crates/kivact/src/lib.rs`（N-D1 が持ち主。N-D2 は `mod` と `pub use` の行を足すだけ）。

### N-D2 新しい trait と型の定義

- **前提**: C-3a（`HandlerResult` は四つの impl で完成済み）。
- **目的**: 設計書 10.3〜10.9 節、18.2・18.3・18.7〜18.11 節、19 章の「callback の変換」と「`NodeBuilder` の公開」、付録 B.1。
- **決定済みの形**:
  ```rust
  // kivact-runtime（kivact::state）
  impl<F, R> From<F> for Callback<()> where F: Fn() -> R + 'static, R: HandlerResult { .. }
  // 今の owner（setup、handler、effect のどれでも）の下に作る。owner がなければ API-12 の panic（19 章）
  pub trait ItemKey: Clone + Eq + Hash + 'static {}   impl<T: Clone + Eq + Hash + 'static> ItemKey for T {}
  pub trait PropValue: Clone + PartialEq + 'static {} impl<T: Clone + PartialEq + 'static> PropValue for T {}
  pub trait IntoBinding<T>: Sealed { fn into_binding(self) -> Binding<T>; }   // impl: Signal<T>、Binding<T>
  // `From<Signal<T>> for Binding<T>` は消す
  impl<T> Signal<T> { pub fn project<U>(self, get: impl Fn(&T) -> &U + 'static, get_mut: impl Fn(&mut T) -> &mut U + 'static) -> Binding<U>; }
  impl<T> Binding<T> { pub fn project<U>(..同じ..) -> Binding<U>; }
  #[macro_export] macro_rules! bind { /* bind!(x.a.b) → x.project(|v| &v.a.b, |v| &mut v.a.b) */ }
  impl IntoValue<f64> for i32 / u32 / f32 { .. }        // 定数の prop
  constant_value!(std::time::Duration, std::path::PathBuf);
  // `Signal::field` と `Signal::item` は消す

  // kivact-core（kivact::view）
  pub struct ViewFn<A = ()>(Rc<dyn Fn(A) -> View>);    // Clone、Copy でない
  impl<A, F, V> From<F> for ViewFn<A> where F: Fn(A) -> V + 'static, V: IntoView { .. }
  impl<A> ViewFn<A> { pub fn call(&self, arg: A) -> View; }
  pub trait IntoViewFn<V>: Sealed { #[doc(hidden)] fn into_view_fn(self) -> ViewFn<()>; }
  impl<F: Fn() -> V + 'static, V: IntoView> IntoViewFn<V> for F {}
  pub trait Component: IntoView + NodeBuilder {}       impl<T: IntoView + NodeBuilder> Component for T {}
  impl<F, R> IntoView for F where F: Fn() -> R + 'static, R: Into<String> { .. }   // reactive な文字の子
  #[derive(Clone, Copy)] pub struct Retry { .. }       impl Retry { pub fn run(&self); }
  impl ErrorBoundary { pub fn fallback<V: IntoView>(self, f: impl Fn(UiError, Retry) -> V + 'static) -> Self; }
  pub trait ListSource<T> { type Row; }                // Signal<Vec<T>>・Binding<Vec<T>> は Row = Binding<T>
                                                       // Value<Vec<T>>・Vec<T>・F: Fn() -> Vec<T> は Row = Value<T>
  // for_each(source: impl ListSource<T>, key, row: impl Fn(S::Row) -> V + 'static)
  pub trait NodeBuilder: Sized {
      #[doc(hidden)] fn node_mut(&mut self) -> &mut Node;   // 公開のまま（kit と外部の部品が実装する）
      fn with_node(self, f: impl FnOnce(Node) -> Node) -> Self;
      // 既定のメソッドは現在の集合のまま（設計書 7.4 節の表）
  }
  impl Node { /* 現在の NodeBuilder と NodeExt の全メソッドを by value の固有メソッドとして持つ。on_attach(Fn(PartMount)) */ }
  impl<S: PropValue> Part<S> { pub fn new(state: Value<S>, apply: impl FnOnce(Node) -> Node + 'static) -> Self; }
  // node.part(p) は Modifiers の列に記録するだけで、apply は mount のときに記録の順に当てる

  // kivact-interaction
  pub enum Selection<K> { Single(Binding<Option<K>>), Multiple(Binding<Vec<K>>) }
  pub trait IntoSelection<K>: Sealed { fn into_selection(self) -> Selection<K>; }
  // impl: Signal<Option<K>>、Signal<Vec<K>>、Binding<Option<K>>、Binding<Vec<K>>。From の impl は作らない
  constant_value!(ScalarRange);
  impl IntoValue<ScalarRange> for RangeInclusive<f64> / RangeInclusive<i32> { .. }
  // IntoScalarRange は消す
  ```
  公開の境界の trait（`IntoValue`、`IntoText`、`IntoBinding`、`IntoSelection`、`IntoView`、`IntoViewFn`、`IntoChildren`、`HandlerResult`、`ItemKey`、`PropValue`、`Component`）に `#[diagnostic::on_unimplemented]` を付ける（API-11。例: `IntoBinding` は「slider は f64 を binding する: `signal(40.0)` と書く」、`ItemKey` は「`#[derive(Clone, PartialEq, Eq, Hash)]` を付ける」、`IntoViewFn` は「この内容は開くたびに作り直す: `|| view` と書く」）。blanket の impl を持つ trait は rustdoc の先頭にも同じ hint を書く。
- **手順**: 上の定義を `kivact-runtime`、`kivact-core`（`view`）、`kivact-interaction` に書く。`NodeExt` の全メソッドを `Node` の固有メソッドへ移し、`primitive/node_ext.rs` と `Node` の「Same as `NodeExt::…`」の複製を消す。`Part` の約 40 の modifier の複製（`view/part.rs`）を消す。呼び出し元は直さない。
- **試験**: 足さない。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: M。
- **並列**: N-D1 と同時。

### N-F1 core の追従と試験の API

- **前提**: N-D1、N-D2。
- **主な crate**: `kivact-runtime`、`kivact-style`、`kivact-i18n`、`kivact-editing`、`kivact-core`、`kivact-test`（G3）。定義の所有漏れを避け、`kivact-types/src/motion.rs` の `Transition::decorative(bool)` 削除と `essential()` の直接実装も本 unit が行う（既定値は保つ）。
- **手順**: この節の冒頭の 1〜7 を G3 の全 file に当てる。加えて:
  1. 文脈と ambient の読み出し（付録 B.10 の `use_context`、`try_use_context`、`use_theme`、`use_tokens`、`use_locale`、`use_direction`）は `Value<T>` を返し、setup と、その owner の下で作られた closure（style の closure、handler、effect）の中で呼べる。`view::modifiers()`・`view::pointer_position()`・`platform` の環境の読み出しは `Value<T>` を返し、どこでも呼べる（signal は `ViewState` の構築のときに作る）。`platform::pointer_settings()`・`keyboard_inset()` を足す（`kivact-types::env` の既存の field）。`use_context` の文脈がないときの panic の message は API-12 の形。
  2. theme: `Theme::ext::<T>() -> ThemeRef<T>`（`ThemeRef<T>(Arc<T>)`、`Clone`、`Deref<Target = T>`、`T: Clone` のとき `to_owned(&self) -> T`）。`App::theme(impl IntoValue<Theme>)` は root の owner の下で評価する reactive な値。`ThemeHint` から `ThemeMode` への `From`。
  3. `Recipe<S>` を struct にする契約（`From<impl Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static>`、`Clone`）は維持し、定義が ui にあるため N-F3 が実装し本 unit は G3 caller を移す。`TestFonts::Only(Vec<Asset>)`、`#[track_caller]` で `at` の引数を消す（付録 B.6）。`Node::accessible_name() -> Option<Value<String>>`。`constant()` を消す。
  4. `kivact-test`: `Screen::evidence(name)`（`KIVACT_CATALOG_EVIDENCE` のとき screenshot・支援技術の木・debug の木を保存）、`Screen::assert_layout_clean()`、`Element::read_only` の削除、`invalid` → `is_invalid`、`password` → `is_password`、そのほか付録 B.10 の `kivact::test` の行。
  5. core の内部の試験の helper（`kivact-test` の中）を新しい API に直す。
  6. `#[derive(Clone)] pub enum ActionOutcome<T> { Succeeded(T), Failed(UiError) }`。`Action::on_complete` は既存の完了通知をこの値で渡し、キャンセル時には従来どおり通知しない。
  7. `Node::accessible_disabled`・`accessible_invalid`・`accessible_expanded` は `Option<Value<bool>>`。`TextEditing::new` は `impl IntoText` を受け、Rc への変換は private な使用境界だけで行う。
  8. Scroll.axes と Overlay の placement/align/offset/viewport_margin は IntoValue の表示入力にし実 node を保って更新する。既存 Toast equality のため Callback は app/key identity の PartialEq/Eq、ViewFn は共有 factory identity の PartialEq を持つ。
  9. embed の startup は固定 Theme と Send な private 設定を UI thread に送り、そこで AppOptions を組み立てる。caller-side reactive App theme は明示の Unsupported とし、embedded root factory 内の provide_theme を案内する。AppOptions の Clone と設定の正本一つ、run/test の reactive 評価は保つ。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: L。
- **並列**: N-F2〜N-F6 と同時（crate が別）。

### N-F2 interaction の追従

- **前提**: N-D1、N-D2。
- **主な crate**: `kivact-interaction`。
- **手順**: この節の冒頭の 1〜7 を全 file に当てる。加えて:
  1. hook の handle を付録 B.10 の `*Api` の名前にする（`Toggle` は `SwitchApi`・`ToggleButtonApi` に分け、`use_switch`・`use_toggle_button` が返す。`toaster()`・`announcer()` の handle は `Toaster`・`Announcer`）。
  2. 全 behavior の part の作り方を `Part::new(state, apply)` にする。part は mount のとき host の node の `disabled`・`invalid`・`name`・`labelled_by`・`described_by` を読んで machine の入力にし、`*Api` に状態の読み出し（例: `SwitchApi::control_state() -> Value<ToggleState>`）を足す（設計書 18.10 節）。
  3. `Selection`・`IntoSelection` を一覧系の behavior の入力にする。`IntoValue<ScalarRange>` を scalar 系の入力にする。`use_inline_edit` を受け入れる。`Reorder<K> { key, to }` をここに置く。
  4. `InlineEditApi::field(key, label, draft) -> TextEditApi`。既存 ui の text_draft を interaction に移して TextEditApi と ui TextInput が同じ commit/cancel/blur/cleanup の実装を使う。field の label は control に設定し、ui への依存を作らない。fan-out の描画側は `.on_draft_commit(..).control()` を node に付ける。
  5. 旧 `Part::node_ref()` の呼出し側は明示的な `node_ref()` を作り、Part closure または `with_node` から実 node に束縛する。互換 getter は足さない。共有 drafting は `TextEditApi::return_focus(NodeRef)` も保つ。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: L。
- **並列**: N-F1、N-F3〜N-F6 と同時。

### N-F3 ui と facade の追従

- **前提**: N-D1、N-D2。
- **主な crate**: `kivact-ui`、`kivact`（G4）。
- **手順**: この節の冒頭の 1〜7 を全 file に当てる。加えて: `menu(label, entries).on_activate(..)`、`command_palette(commands, open).on_activate(..)`、`tag_input(label, tags).on_add(..).on_remove(..)`、`disclosure(title, open).content(..)`、`list(label, items).selection(..)`、`tree(label, nodes).selection(..)`、`List::multiple`・`ListSelection` の削除、`number_input` の `impl IntoValue<ScalarRange>`、部品の builder の固有の semantics のメソッド（`Scroll::role`、`Button::selected`・`expanded`）の削除（部品の中で `with_node` か構築時の `Node` で設定する）。名前と引数の順は付録 B.10。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: L。
- **並列**: N-F1、N-F2、N-F4〜N-F6 と同時。

### N-F4 basic・pop・game の追従

- **前提**: N-D1、N-D2。
- **主な crate**: `kivact-ui-basic`、`kivact-ui-pop`、`kivact-ui-game` と、その試験。
- **手順**: この節の冒頭の 1〜7 を全 file に当てる。kit の `impl NodeBuilder` は `node_mut` をそのまま実装する。kit の `slot.rs` を消して `ViewFn` と `IntoViewFn` にする。kit の `GroupItems`・`GroupMembers`・`InputGroupItems`・`IntoListItems`・`IntoMetadataItems` を `IntoChildren` にする。kit の `f32` の引数は `impl Into<f64>`、定数の bool の引数は 3 の規則（実行中に変わるなら `impl IntoValue<bool>`、10.6 節の一覧にあるなら引数なしのメソッド）。付録 C.1 の KIT-6 の「inherent の `disabled`」「option の束」「型の alias」は K-7a〜K-7c。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: L。
- **並列**: N-F1〜N-F3、N-F5、N-F6 と同時。

N-F4 の補足: Calendar の number_of_months/caption/week_start は reactive な表示入力。Drawer.side(Edge)、Sidebar.collapsible(SidebarCollapsible)、InputOtp.groups/pattern は設計書 10.6 の構築 mode/format として plain な値を保つ。IntoChildren への移行で必要な joined style は typed child/recipe が private 構築文脈から読み、任意の View は layout/clipping wrapper で包む。公開 View root mutation や kit 独自 child trait は作らない。

### N-F5 creative の追従

- **前提**: N-D1、N-D2。
- **主な crate**: `kivact-ui-creative` と、その試験。
- **手順**: N-F4 と同じ規則を creative の全 file に当てる。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: L。
- **並列**: N-F1〜N-F4、N-F6 と同時。

### N-F6 soft の追従

- **前提**: N-D1、N-D2。
- **主な crate**: `kivact-ui-soft` と、その試験。
- **手順**: N-F4 と同じ規則を soft の全 file に当てる（soft には `slot.rs` がないので、写しの slot の型があればそれを `ViewFn` にする）。
- **unit の確認**: `cargo fmt` だけ（build 不可でよい）。
- **規模**: L。
- **並列**: N-F1〜N-F5 と同時。

（fan-out A の全部を merge したら、orchestrator が `cargo check --workspace --all-targets --all-features` を回し、残った error を crate の組ごとの fix の unit に配る。G6 の error は fan-out B が直すので、G1〜G5 が通れば fan-out B へ進む。）

N-F6 の補足: StepIndicator::Custom(ViewFn<()>) は plain な slot/config enum。typed children の IntoView は private composition context を読み、任意の View は必要な wrapper を使う（共有 label 測定などの既存挙動を保つ）。

既存 `input_group_with(label, init: impl FnOnce() -> C)`（`C: IntoChildren`）は scoped initializer として残す。group の size 文脈の内側で一度だけ構築する意味があり、作り直す view slot や alias ではない。eager/scoped の入口は一つの private mount 処理へ集約する。

### N-G1 例・catalog・雛形の追従

- **前提**: fan-out A と fix。
- **主な path**: `examples/`、各 kit の catalog、`templates/`。
- **手順**: この節の冒頭の 1〜7 を全 file に当てる。
- **unit の確認**: `cargo check` で該当の package（examples・catalog・template の全部）を `--all-targets --all-features`。
- **規模**: M。
- **並列**: N-G2〜N-G5 と同時。

### N-G2 文書と公開 API の E2E の追従

- **前提**: fan-out A と fix。
- **主な path**: `docs/`、README・README_JP、`dev/tests/docs`（`kivact-docs` の doctest）、`dev/tests/kivact-public-api/tests/e2e/`。
- **分担**: 六つの枠を使うため、本 unit は二つの実装担当に分ける。(a) docs/README/dev/tests/docs と doctest、(b) 既存 public-api E2E の API 移行。file の所有は重ねない。N-G3/N-G5 は新しい試験を追加し、(b) は既存の import/helper/試験本体だけを移す。共有 file の追加行は merge 時に両方を保つ。
- **手順**: この節の冒頭の 1〜7 を全 file に当てる（E2E の書き方を新しい名前と形に直す。試験の意味は変えない）。`docs/for-kivact-user/{en,jp}/how-to-guide/custom-controls.md` に「状態機械で振る舞いを書く」の節を足し（付録 B.7 の例を元に、doctest が通ること）、「part を node に移す前に状態を読む」の記述を消す（18.10 節で不要）。guide の custom component の最初の例で `constant_value!`・`impl Into<Callback<()>>`・`impl Component` を教える（18.2・18.9 節）。
- **unit の確認**: `cargo check -p kivact-docs -p kivact-public-api-tests --all-targets --all-features`。`cargo test -p kivact-docs --doc`。
- **分担した確認**: (a) は docs の check と doctest、(b) は public-api の check。N-G3/N-G5 の追加試験の API は各担当に任せ、同じ修正を重ねない。unit 全体の完了は両方の merge と確認が終わったとき。
- **規模**: L。
- **並列**: N-G1、N-G3〜N-G5 と同時。共有の file: `dev/tests/kivact-public-api/tests/e2e/main.rs`（N-G3・N-G5 が module の行を足す）。

### N-G3 `EditPhase` と `on_edit` の網羅

- **前提**: fan-out A と fix。
- **目的**: 設計書 10.2・11.2 節。
- **決定済みの形**: `EditPhase` は `kivact::view`（N-D1 で移動済み）。値を書く一回は `EditSession` の `Begin` → `Commit` の一組。
- **手順**: core の native text editing（`custom::TextEditing` と text input の node）が `EditPhase` を出し、`ui::text_input` の橋渡しを消す。`use_checkbox`、`use_switch`、`use_toggle_button`、`use_select`、`use_tabs` と、`ui` の `checkbox`、`switch`、`select`、`tabs`、`toggle_button`、`tag_input` に `on_edit(impl Fn(EditPhase) -> R + 'static)` を足す（`EditSession` を使う）。`kivact-ui/src/lib.rs` の「Every component that writes a value reports each edit to `on_edit`」の約束が真になる。
- **試験**: 足す。`dev/tests/kivact-public-api/tests/e2e/interaction_edit.rs` に `every_value_writing_standard_component_reports_on_edit`（6 部品の値をそれぞれ一回変え、`on_edit` が `Begin` と `Commit` を一回ずつ受けること）と `text_input_reports_edit_phases_from_the_native_editor`。
- **unit の確認**: `cargo check -p kivact-core -p kivact-interaction -p kivact-ui -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e interaction_edit::`。
- **規模**: M。
- **並列**: N-G1、N-G2、N-G4、N-G5 と同時。共有の file: `kivact-core/src/view/node.rs`（N-G4 と）。

### N-G4 semantics の patch

- **前提**: fan-out A と fix。
- **目的**: 設計書 7.4 節の semantics の組み立てと 18.21 節。
- **決定済みの形**: `Node` の semantics のメソッドは `Modifiers.semantics: Vec<SemanticsPatch>` に `(層, 位置, patch)` を積む。mount のとき、定数だけの patch は `Semantics<FocusTarget>` に畳んで一度書き、reactive な値を読む patch があるときだけ node ごとに一つの render effect を作って層の順に当て、前回と違えば engine へ書く。patch の `#[track_caller]` の位置を `why` の origin に記録する。`SemanticSources`（`view/common.rs`）を消す。
- **試験**: 足さない（`relations.rs`、`landmark_roles.rs`、`image_semantics.rs`、`parts.rs` の E2E が回帰の網）。
- **unit の確認**: `cargo check -p kivact-core -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- relations:: landmark_roles:: image_semantics:: parts::`。
- **規模**: M。
- **並列**: N-G1〜N-G3、N-G5 と同時。共有の file: `kivact-core/src/view/node.rs`（N-G3 と）。

### N-G5 残りの E2E

- **前提**: fan-out A と fix。
- **手順**: 次の E2E を足す（どれも公開 API だけで書く）:
  - `dev/tests/kivact-public-api/tests/e2e/machine.rs`（新しい module）: `a_machine_renders_its_state_and_runs_effects_after_the_transition`、`a_machine_timer_fires_once_and_is_replaced_by_the_same_event_variant`（同じ event の variant の `Step::after` は前の timer を置き換える。`Screen::advance` で時刻を進める）、`feed_turns_an_input_change_into_one_event`、`a_machine_timer_is_canceled_with_its_owner`、`send_inside_a_transition_effect_is_queued_not_reentrant`。
  - `view.rs`: `closure_child_updates_text`。
  - `flows.rs`: `for_each_row_binding_edits_the_item`。
  - `setup_functions.rs`: `bind_macro_edits_a_nested_field`。
  - `ui_theming.rs`: `use_theme_reads_inside_a_style_closure`、`app_theme_follows_theme_hint`。
    この試験のため G-4c C-20 の theme_hint hook の公開/eager field 登録を本 unit へ前倒しする（From<ThemeHint> は N-F1 済み）。native pointer/fling/snap は G-4c に残す。
  - `reactive_props.rs`（新しい module）: `reactive_component_props_update_without_remounting`。ControlSize の変更が表示に反映され focus を保つことと、ScalarApi の key_step の変更が次の操作に反映されることを公開 API で確かめる（厳密な pixel 比較はしない）。
- **unit の確認**: `cargo check -p kivact-public-api-tests --all-targets --all-features`。`KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- machine:: reactive_props:: closure_child_updates_text for_each_row_binding_edits_the_item bind_macro_edits_a_nested_field use_theme_reads_inside_a_style_closure app_theme_follows_theme_hint`。
- **規模**: S。
- **並列**: N-G1〜N-G4 と同時。

### N-8 公開 API の検査を有効にする

- **前提**: fan-out B。
- **目的**: 設計書 14.1・18.20 節（公開 API の検査だけ。関数の長さと状態の検査は H-G1〜H-G6）。
- **手順**:
  1. `dev/xtask/policy/workspace.toml` の `api-shape`・`canonical-path` の `skip` から `kivact`、`kivact-core`、`kivact-runtime`、`kivact-style`、`kivact-i18n`、`kivact-interaction`、`kivact-ui`、`kivact-test` を外す（api-shape は facade から届く item だけを見る）。`api-shape` に「facade から届く型の公開の `f32` の field は error」を足す。
  2. `[structural_options] list` に設計書 10.6 節の一覧を付録 B.10 の名前で書き、`structural-options` の検査を有効にする。
     この段の scope は上の八 crate。kit は K-7 後の H-G5 で skip を外す。passive な Style/Span/Path/CalendarDate/SortOrder と collection record/本当の Options bundle は UI runtime flag に分類しない。
  3. 出た違反を直す。例外は inline の目印 `// xtask-allow(<検査>): <理由>`（理由は設計書 10.11 節の列挙に限る）。
- **unit の確認**: `cargo xtask check --no-write --only api-shape,canonical-path,structural-options`。`cargo check --workspace --all-targets --all-features`。
- **規模**: M。
- **並列**: なし（W4 の最後。この後 CP4）。

---

## 9. Phase I・U: 振る舞いと標準部品

wave W5。I・U の各 unit は、付録 A の担当の項目を `Machine` で書き直す（一つ目の commit、振る舞いを変えない）。続けて、その unit の behavior に当たる APG の pattern に振る舞いを合わせる（二つ目の commit、振る舞いの変更と E2E）。旧 I-8 は各 unit に分けたので、独立した unit はない。

**全 unit に共通の決定済みの形**（設計書 18.13 節と 19 章。設計書を開かずに書けるように写す）:

```rust
// kivact-runtime（R-6 で公開済み。facade は kivact::state）
pub trait Machine: Sized + 'static {            // Self が、遷移が読む入力の snapshot
    type State: Clone + PartialEq + 'static;
    type Event: 'static;
    type Effect: 'static;
    fn transition(&self, state: &Self::State, event: Self::Event) -> Step<Self::State, Self::Event, Self::Effect>;
}
// Step::stay()、Step::to(next)、.with(effect)、.after(delay, event)（同じ variant の timer を置き換える）、.cancel(event)
pub fn machine<M: Machine>(initial: M::State, input: impl Fn() -> M + 'static, run: impl Fn(MachineCx<M>, M::Effect) + 'static) -> MachineHandle<M>;

// kivact-interaction（crate private。I-0 が作る）
pub(crate) struct OptionsCell<S: Clone + 'static>(Signal<S>);   // Clone・Copy は手で書く
impl<S: Clone + 'static> OptionsCell<S> {
    pub(crate) fn new(initial: S) -> Self;
    #[track_caller]
    pub(crate) fn update(&self, f: impl FnOnce(&mut S));   // 最初に require_setup(..)。mount の後の呼び出しは misuse（API-12）
    pub(crate) fn get(&self) -> S;                         // untracked。machine の入力の snapshot を作るとき
    pub(crate) fn value(&self) -> Value<S>;                // part が mount のときに読む・feed に渡す
}
```

- machine の型は behavior の module の private な子 module `machine` に、`Machine`・`State`・`Event`・`Effect` の名前で置く（公開の `*State`（part の状態）と衝突しない）。
- 状態の enum の型名と variant は付録 A の「解消後」で決まっている。variant の field・event・effect は、今の code の入口（handler、effect、builder の setter）から機械的に作る（設計書 0.3 節の M-1〜M-3）。付録 A の行番号は古いので、名前で探す。
- timer は `Step::after` だけ（`spawn(sleep(..))` と `Task` の handle を消す）。辻褄を合わせる effect（閉じたら戻す、有効に保つ、reset する）は、遷移の規則に入れて消す。
- behavior の option は、その behavior の `XOptions` struct（crate private でよい）を `OptionsCell<XOptions>` に入れる。field が `Value<T>` で、変化で状態を変える必要があるものは `MachineHandle::feed` で event にする。
- **公開の署名を変えない**: 各 `*Api` の公開のメソッドの署名は W5 の間変えない（unit に書いた追加だけを足す）。変える必要が分かったら止めて報告する。
- kit・例・文書の呼び出し元は、unit に書いたものだけを直す。xtask の `skip` の一覧は変えない（H-G4 が外す）。
- 各 unit の報告の前に、触った module ごとに `cargo xtask check --no-write --only state-fields --crate <crate> --module <module>` を回し、違反を 0 にする。

**APG の合わせ方（二つ目の commit）**:

- 基準は WAI-ARIA Authoring Practices Guide の、その pattern の「Keyboard Interaction」と「WAI-ARIA Roles, States, and Properties」。unit に書いた pattern ごとに、interaction の behavior と、それを使う `kivact-ui` の部品（同じ wave でほかの unit が持つ file を除く）を一行ずつ当てる。
- 作者の決定が APG より優先する: 右クリックのメニューは `Capabilities.context_menu_on`（`Press`・`Release`。A-2 が足した）で開き、keyboard で開いたら trigger の位置に出す。閉じた select の矢印は値を変え、Alt+Down で開く。tooltip は約 500 ms の遅延と skip-delay を持ち、一度に一つ。
- APG が決めていない細部（時間、距離）は今の値のまま。
- 違いは APG のとおりに直し、直した違いごとに E2E を `dev/tests/kivact-public-api/tests/e2e/apg_<pattern>.rs` に一つ足す（名前は `<pattern>_<APG の操作>`。例: `tree_view_star_expands_all_siblings`）。`e2e/main.rs` の `mod` の行は足すだけ（merge はオーケストレータが合わせる）。
- 違いの表（pattern、現在、APG、直した内容）を報告に書く。違いがなければ「違いなし」と書き、試験は足さない。

**順と並列**（W5 の中。同時に動かすのは 6 つまで）:

- I-0 は一つで先に行う（ほかの I・U の前提）。
- I-0 の後: I-1、I-2a、I-2b、I-3、I-4a、I-4b、I-5a、I-5b、I-6 は、互いの merge を待たずに並べてよい（公開の署名が固定されているため）。
- U-1 は I-2a の後（`SelectState` の `Surface` を使う）。U-2 は I-3 と I-5a の後。I-7 は E-9a・E-9b、I-1、I-4a、I-6、U-1 の後。
- 9.5 章の T・G・KL は、I・U と crate が重ならないので、同じ wave の空いた枠に入れる。

### I-0 option と policy

- **目的**: 設計書 11.1・18.15 節（19 章で変えた所を含む）。付録 A の SM-I28、SM-I15。
- **決定済みの形**:
  - `OptionsCell<S>`: 上の共通の形。新しい file `crates/kivact-interaction/src/options_cell.rs`。この unit は型を作るだけで、behavior の移し替えは各 unit が行う。
  - `kivact::interaction::InteractionPolicy`（theme の拡張。`Clone`・`Debug`・`PartialEq`・`Default`。全 field が `pub`。FRU で一部を上書きする）:
    ```rust
    pub struct InteractionPolicy {
        pub menu: MenuPolicy,       // { press: MenuPress, drift: PressDrift, submenu: SubmenuTiming, disabled_focus: DisabledMenuFocus }
        pub tooltip: TooltipTiming, // 旧 TooltipConfig { delay: 500 ms, skip_delay: 300 ms }
        pub button: ButtonPolicy,   // { disabled_focus: DisabledButtonFocus }
        pub slider: SliderPolicy,   // { touch: SliderTouchMotion }
        pub switch: SwitchPolicy,   // { drag: SwitchDrag }
        pub toast: ToastPolicy,     // { swipe: ToastSwipe, swipe_slop: f64 = 8.0 }
        pub select: SelectPolicy,   // { sheet_slop: f64 = 10.0 }
    }
    pub struct PressDrift { pub context: f64, pub other: f64 }   // 10.0, 13.0（menu/press.rs の Source::Context とそれ以外）
    ```
    `hover_card` と `drawer` の field は、使う部品ができる K-2a が足す。各 `*Policy` の入れ子の struct と旧い型は `Default` で今の fallback の値を返す。
  - `DisabledButtonFocus`（`kivact-ui/src/button.rs`）、`SliderTouchMotion`（`kivact-ui/src/slider.rs`）、`SwitchDrag`（`kivact-ui/src/switch_drag.rs`）、`ToastSwipe`（`kivact-ui/src/toast_swipe.rs`）の定義を `kivact-interaction` に移す（interaction は ui を読めないため）。公開の path は `kivact::interaction::*` になる。
  - `kivact::ui::UiTokens`（theme の拡張。部品ごとの入れ子の struct。各 struct はその部品の file に置き、`UiTokens` は新しい file `crates/kivact-ui/src/tokens.rs` に置く）。この unit で作る field はこれだけ:
    ```rust
    pub struct UiTokens {
        pub button: ButtonTokens,               // { busy_motion: ButtonBusyMotion }
        pub progress: ProgressTokens,           // { motion: ProgressMotion }
        pub toast: ToastTokens,                 // { row_motion: ToastRowMotion }
        pub toggle_button: ToggleButtonTokens,  // { label_reservation: ToggleLabelReservation }
        pub dialog: DialogTokens,               // { center: DialogCenter }
        pub disclosure: DisclosureTokens,       // { motion: DisclosureMotion }
        pub data_table: DataTableTokens,        // { sticky: DataTableStickyPaint, row_status: DataTableRowStatusPaint }
    }
    pub struct DisclosureMotion { pub animate: bool, pub duration: Duration }  // 既定 false, 200 ms（今の ui::disclosure の既定）
    ```
  - `OverlayViewport`（`kivact-core/src/view/overlays.rs`）を消し、`Theme` の metric `overlay_margin`（`Theme::with_overlay_margin(f64)`、`Theme::overlay_margin() -> f64`。既定は `0.0`。未設定の no-fit を廃止し常に viewport fitting する（本 unit の意図した境界変更））にする。
  - 閾値の literal は policy の既定値にする（値は変えない）: `menu/press.rs` の 10.0・13.0 → `menu.drift`、`kivact-ui/src/select.rs` の touch の 10.0 → `select.sheet_slop`、`kivact-ui/src/toast_swipe.rs` の `SLOP = 8.0` → `toast.swipe_slop`。ほかの literal（switch の drag の 10 px、toast の `FLICK_DISTANCE` など）は名前の付いた private の定数のまま。
  - `Typeahead`（SM-I15）: `enum Typeahead { Idle, Typing { text: String, expires_ns: u64 } }`（窓は今の 500 ms）。I-2a と I-4a が両方使うので、この unit で `list/typeahead.rs` を書き直す。
- **手順**:
  1. `OptionsCell<S>` を作る（`require_setup` は `kivact-core` が再公開している runtime の関数）。
  2. `InteractionPolicy` と入れ子の struct と `PressDrift` を作り、上の 4 型を interaction へ移す。`MenuPress`・`SubmenuTiming`・`DisabledMenuFocus`（`menu/press_state.rs`、`menu/level.rs`）と ui の 11 型の `impl ThemeExtension` を消す。
  3. `TooltipConfig` を `TooltipTiming` と改名し、`try_context::<TooltipConfig>()` の読み出しを `InteractionPolicy.tooltip` の読み出しにする（文脈で渡す経路は消す。使っている所は `tooltip.rs` だけ）。
  4. 各部品の読み出しを直す: interaction と ui は `use_theme().get().ext::<InteractionPolicy>()` と `ext::<UiTokens>()` から読む。
  5. `UiTokens` と入れ子の struct を作り、見た目の動きの 6 型と data table の 2 つの paint の型を field にする。`OverlayViewport` を `Theme::overlay_margin` にする。
  6. soft の theme（`crates/kivact-ui-soft/src/theme.rs` の `.with(..)` の列）を、`InteractionPolicy { menu: .., button: .., slider: .., switch: .., toast: .., ..Default::default() }`、`UiTokens { .. }`、`with_overlay_margin(..)` に直す。拡張を入れている kit は soft だけ。
  7. 拡張を直接入れている試験を直す: `crates/kivact-ui-soft/tests/toggle_label.rs`、`dev/tests/kivact-public-api/tests/e2e/overlay.rs`。
  8. `list/typeahead.rs` を SM-I15 の形にする。
- **試験**: 足さない。APG の commit はない。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui -p kivact-ui-soft -p kivact-public-api-tests --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- overlay:: interaction_menu:: interaction_tooltip:: theme_conformance::`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test menu_press --test disabled_menu --test button_busy --test progress_motion --test toast_swipe --test toast_motion --test slider_touch_motion --test switch_touch --test toggle_label --test table_sticky_columns --test table_row_status`
- **規模**: M。**並列**: T-1、G-1a、G-2、G-4a、G-5、KL-basic と同時に動かしてよい。共有の file は `kivact-interaction/src/lib.rs`・`kivact-ui/src/lib.rs`（足すだけ）。

### I-1 開閉の family

- **目的**: 付録 A の SM-I01（option の部分）、SM-I02、SM-I03、SM-I04、SM-I05。
- **決定済みの形**:
  - SM-I01: `OptionsCell<DisclosureOptions>`。`leaving` の入力と `use_content_reveal` はこの unit では残す（I-7 が消す）。
  - SM-I02 popup:
    ```rust
    enum State {
        Closed,
        Open { opener: Option<NodeToken>, policy: FocusPolicy, focus: FocusRange },
        Closing { reason: PopupCloseReason, opener: Option<NodeToken> },
        ReturningFocus { opener: NodeToken },
    }
    enum FocusRange { NotEntered, Entered, AnchorSeen, AnchorRemoved }
    ```
    `modal` は option。「既に開いているのに開く」は、`Open` で受けた `Open` event を misuse として報告する。
  - SM-I03 delayed: `State::{Hidden, Opening, Shown, Closing, Dismissed}`（task の handle を除く）。開く・閉じるの待ちは `Step::after(delay, Event::OpenDue)` と `Event::CloseDue`。
  - SM-I04 tooltip: hub の状態は `struct Hub { stack: Vec<TooltipId>, warmth: Warmth }`、`enum Warmth { Cold, Warm { until_ns: u64 } }`。`TooltipId` は `Generation`。時間は `InteractionPolicy.tooltip`（I-0）。module の doc を事実（`Delayed` と popup の状態で作る）に直す。
  - SM-I05 dialog: 状態は popup の `State`。`anchored` の bool を消し、private `DialogOptionsMode::Popover { anchor }` の variant にする。公開 `DialogMode::Popover` は trigger を自動で anchor にできる appearance enum のまま保つ。
- **手順**: `disclosure.rs`、`popup.rs`、`delayed.rs`、`tooltip.rs`、`dialog.rs` を書き直す。APG の commit で直すのは `kivact-ui` の `disclosure.rs`・`dialog.rs`・`tooltip.rs`。
- **APG**: Disclosure、Dialog (Modal)、Alert Dialog、Tooltip。
- **試験**: APG の違いごとに `apg_disclosure.rs`、`apg_dialog.rs`、`apg_alert_dialog.rs`、`apg_tooltip.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_delayed:: interaction_dialog:: interaction_tooltip:: overlay:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test overlay_interaction --test text_truncation_tooltip`
- **規模**: M。**並列**: I-0 の後の I・U・T・G・KL と並べてよい（file は重ならない）。

### I-2a combobox・select・palette

- **目的**: 付録 A の SM-I06、SM-I07、SM-I904。
- **決定済みの形**:
  - SM-I06: `ComboboxMode::{Inline, Popup(Binding<bool>)}`（構造の option）。`State { query: String, highlight: Option<K> }`。表示する highlight は「今の rows にある key だけ」を memo で作る（effect で書き戻さない）。
  - SM-I07:
    ```rust
    enum State<K> {
        Closed { typeahead: Typeahead },
        Open { highlight: Option<K>, hovered: Option<K>, opened_by: OpenedBy, surface: Surface, typeahead: Typeahead },
    }
    enum OpenedBy { Pointer, Key(Key), Typeahead }
    enum Surface { Popover, Sheet }
    ```
    `Surface` と、sheet で開く event `OpenSheet` は、この unit で作るだけ。使うのは U-1。`arrow_open`・`keep_trigger_focus` は option。preview の重複を除くのは `Sent<Option<K>>`。
  - SM-I904: palette の `Closed`・`Open` の遷移が、combobox の query と highlight の reset を同時に決める（query の写しを作らない）。
- **手順**: `combobox.rs`、`select.rs`、`palette.rs` を書き直す。`Typeahead` は I-0 の形を使う。APG の commit で直すのは `kivact-ui` の `select.rs`・`command_palette.rs` と combobox の部品。
- **APG**: Combobox、Listbox（select の popup。作者の決定: 閉じた select の矢印は値を変え、Alt+Down で開く）。
- **試験**: APG の違いごとに `apg_combobox.rs`、`apg_listbox.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_combobox:: interaction_select:: interaction_palette:: ui_command_palette:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test selector_keyboard --test selector_focus --test search`
- **規模**: L。**並列**: I-4a と同時に動かしてよい（`Typeahead` は I-0 で固定済み）。I-2b とは file が重ならない。

### I-2b menu・menubar

- **目的**: 付録 A の SM-I08、SM-I09、SM-I10。
- **決定済みの形**:
  ```rust
  enum Session<K> { Closed, Open { opened_by: OpenedBy, levels: Vec<Level<K>>, want: Option<Want<K>>, typeahead: Typeahead } }
  struct Level<K> { parent: Option<K>, highlight: Option<K>, submenu: Submenu<K> }
  enum Submenu<K> { None, Opening { key: K }, Open { key: K }, Closing { key: K } }
  enum PressGesture { Idle, Pending { from: Point, source: PointerId }, Holding { at: Point }, Dragging { at: Point } }
  enum Highlight { Unset, Nothing, Item(NodeRef) }
  ```
  - `menu/session.rs` の三つの effect（閉じたら reset、開いていない level への要求を捨てる、level の切り詰め）は、遷移の規則にする。
  - submenu の開閉の時間は `Step::after` と `InteractionPolicy.menu.submenu`。hold の timer は `Step::after`、drift は `InteractionPolicy.menu.drift`。
  - menubar は同じ `Session` を使う。
- **手順**: `menu/state.rs`、`menu/session.rs`、`menu/press.rs`、`menu/press_state.rs`、`menu/level.rs`、`menu/handle.rs`、`menu/order.rs`、`menubar.rs`。APG の commit で直すのは `kivact-ui/src/menu.rs`。
- **APG**: Menu・Menubar・Menu Button（作者の決定: 右クリックは `Capabilities.context_menu_on`、keyboard で開いたら trigger の位置）。
- **試験**: APG の違いごとに `apg_menu.rs`、`apg_menubar.rs`、`apg_menu_button.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_menu:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test menu_press --test disabled_menu`
- **規模**: L。**並列**: I-2a などと並べてよい。

### I-3 編集・drag・文字編集・toggle

- **目的**: 付録 A の SM-I11、SM-I12、SM-I13、SM-I24。設計書 19 章の `DragApi` の追加。
- **決定済みの形**:
  - SM-I11: `enum EditSessionState<T> { Idle, Editing { initial: T, origin: EditOrigin } }`。U-2 がこの型を crate の中から使えるように `pub(crate)` で置き、`EditSession` handle の公開の署名は変えない。
  - SM-I12: `State::{Idle, Pending { origin: Point, pointer: PointerId }, Active(Active), Releasing { edit: EditOwnership }}`、`enum EditOwnership { Owned, Borrowed }`。source の id は `Generation`。
  - SM-I13:
    ```rust
    enum Control { Unmounted, Mounted { id: ControlId, composition: Composition } }
    enum Composition { Idle, Composing }
    ```
    composition の変化は IME の event で遷移する（`was_composing` の edge の検出を消す）。二つ目の同時の mount は、`Mounted` で受けた `Mount` event を報告する。7 つの settings と callback は `OptionsCell<TextEditOptions>`。
  - SM-I24: `OptionsCell<ToggleOptions>`。
  - `DragApi` に足す（使うのは、今 `use_drag` を使っている部品だけ）:
    ```rust
    impl<T: Clone + 'static> DragApi<T> {
        pub fn slop(self, distance: impl Into<f64>) -> Self;  // 既定 0: 今と同じく押した瞬間に始まる。越えるまで State::Pending
        pub fn axis_lock(self) -> Self;      // 構造の option。slop を越えた最初の向きの軸に固定し、交わる軸の delta を 0 にする
        pub fn track_velocity(self) -> Self; // 構造の option。DragUpdate.velocity を埋める（窓 100 ms）
    }
    pub struct DragUpdate<T> { /* 今の field */ pub velocity: Point }  // px/ms。track_velocity がなければ Point::ZERO
    ```
    `DragVelocity` は公開のまま残す（soft の `date_touch.rs` などが drag なしで使う）。`DragApi` は中でそれを使う。`project_release`・`nearest_snap`・`pull_to_snap`・`resist_overshoot` は今の公開の関数のまま。
- **手順**: `edit.rs`、`drag.rs`、`text_edit/handle.rs`、`text_edit/policy.rs`、`toggle.rs`。APG の commit で直すのは `kivact-ui` の `button.rs`、`toggle.rs`、`toggle_button.rs` と checkbox・switch の部品（`switch_drag.rs` は U-1 が持つので触らない）。
- **APG**: Button、Checkbox（mixed を含む）、Switch。
- **試験**:
  - 足す: `interaction_drag_edit.rs` に `drag_slop_waits_before_starting`、`drag_axis_lock_keeps_the_first_axis`、`drag_track_velocity_reports_pixels_per_ms`。
  - APG の違いごとに `apg_button.rs`、`apg_checkbox.rs`、`apg_switch.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_drag_edit:: interaction_edit:: interaction_toggle:: text_editing:: apg_`
- **規模**: M。**並列**: I-0 の後の I・T・G・KL と並べてよい（KL-creative は I-3 の merge の後）。

### I-4a list・roving・tree・tabs

- **目的**: 付録 A の SM-I14、SM-I16、SM-I26（list の分）。tree・tabs・radio の option の移し替え。
- **決定済みの形**:
  - SM-I14: `keep_focus()` は構造の option で、part の bind（mount）のときに読む（作成の順に依存しない）。`position` は別の memo にする（memo の中で書かない）。`enum Pending<K> { Nothing, Activating { key: K, origin: Origin } }`。option は `OptionsCell<ListOptions>`。
  - SM-I16: `changed` は `Notifier`（公開）。`enum Roving { Unset, Requested(Id), Observed(NodeToken) }`、`enum Restore { Nothing, From(NodeToken) }`。
  - SM-I26: 生存の確認は既存の実体 `Owned<State>::try_get`（pure memo は既存 order の try_get）を使い、`alive: Owned<()>` の token を消す（足す API はない）。
  - `tree.rs`・`tabs.rs` は、option の保存を `OptionsCell` にする（状態の項目はない）。
- **手順**: `list/*`、`roving.rs`、`tree.rs`、`tabs.rs`。APG の commit で直すのは `kivact-ui` の `list.rs`、`radio_group.rs`、`tabs.rs`（`tree.rs` は I-7 が持つので、tree の APG の違いは interaction の `tree.rs` だけで直す。ui の役割と状態の違いは I-7 の後の K-4 で直す）。
- **APG**: Listbox（list）、Tree View、Tabs、Radio Group。
- **試験**: APG の違いごとに `apg_listbox.rs`（I-2a と同じ file に足す。名前が重ならないように `listbox_list_*` で始める）、`apg_tree_view.rs`、`apg_tabs.rs`、`apg_radio_group.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_list_nav:: interaction_tree:: interaction_tabs:: focus:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test consumer_collections --test tab_indicator_motion`
- **規模**: L。**並列**: I-0 の後の I・T・G・KL と並べてよい。`apg_listbox.rs` は I-2a と共有（足すだけ）。

### I-4b table・calendar

- **目的**: 付録 A の SM-I27、SM-I903、SM-I26（calendar・table の分）。
- **決定済みの形**:
  - SM-I27: `OptionsCell<DataTableOptions>`。列の幅の変更は `use_splitter` の状態。並べ替えは `enum Sort { Unsorted, Sorted(Vec<(ColumnKey, SortDirection)>) }`。
  - SM-I903: `enum RangePhase { Start, End { anchor: CalendarDate, before: Option<DateRange> } }`。`ExternalValueChanged` と日付の選択を同じ遷移の入口で受け、binding の更新と phase の reset を一度に決める。
  - SM-I26: facade Owner に is_alive はないため、既存の実体 `Owned<State>::try_get` を使い、`alive` の token を消す。
- **手順**: `table/*`、`calendar/*`。APG の commit で直すのは `kivact-ui/src/data_table/*`。
- **APG**: Grid・Table（data table と calendar の grid）。
- **試験**: APG の違いごとに `apg_grid.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_table:: interaction_calendar:: ui_data_table:: grid:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test grouped_table --test table_tree --test table_sticky_columns --test table_row_status --test table_column_filters`
- **規模**: M。**並列**: I-0 の後の I・T・G・KL と並べてよい。`data_table/filters.rs` は U-2 も触るので、U-2 は I-4b の merge の後。

### I-5a scalar

- **目的**: 付録 A の SM-I17、SM-I18、SM-I901、SM-I902。
- **決定済みの形**:
  - SM-I17: `OptionsCell<ScalarOptions>`。「どの thumb にも focus がない」は thumb の blur の event で遷移する（focus を失ったら drag を止める effect を消す）。
  - SM-I18: `enum TouchSlider { Idle, Pressed { contact: Contact }, Tracking { contact: Contact, anchor: f32 }, Coasting { coast: Coast } }`。`DragApi` を使わず、今の raw pointer の handler を machine の event にする（touch だけ、primary だけ、`suppress_activation` の順は今のまま）。slop の判定は `DragAxis::past_slop`、速度は `DragVelocity`。coast の frame の loop は `Coasting` の間だけ動かす。
  - SM-I901: `enum TrackPhase { Detached, Attached { geometry: Geometry, gesture: TrackGesture } }`、`enum TrackGesture { Idle, Grabbing(Grab) }`。`attached` は memo。phase を書くのは、取り付けと grab の遷移だけ。
  - SM-I902: scalar の machine が `RangeChanged(ScalarRange)` を受け、正規化と値の effect を一つの遷移で決める。range scalar は両端を同じ遷移で計算し、今の書く順を effect の列で保つ。
- **手順**: `scalar/*`、`range_scalar.rs`。
- **APG**: Slider、Multi-Thumb Slider（Spinbutton は U-2）。
- **試験**: APG の違いごとに `apg_slider.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_slider:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test slider_touch_motion --test date_touch`
- **規模**: M。**並列**: I-0 の後の I・T・G・KL と並べてよい。

### I-5b zoom・pan・splitter

- **目的**: 付録 A の SM-I19、SM-I20。`SplitterApi` の collapse と snap（付録 C.3、K-5 が使う）。
- **決定済みの形**:
  - SM-I19: `Gesture::Pan`・`Pinch` の variant が、touch の点と edit の session を持つ。`Idle` に戻る遷移が edit を commit する。option は `OptionsCell<ZoomPanOptions>`。
  - SM-I20: `enum SpaceKey { Up, Held }`。
  - splitter:
    ```rust
    impl SplitterApi {
        pub fn collapsible(self, threshold: impl Into<f64>) -> Self;    // 0.0〜1.0 の割合。下回って離したら閉じる
        pub fn snaps(self, stops: impl IntoValue<Vec<f64>>) -> Self;   // 0.0〜1.0 の割合
    }
    enum PanelSize { Expanded(f64), Collapsed { restore: f64 } }  // SM-K06 の状態。splitter の machine に置く
    ```
    snap は `nearest_snap` を使う。option は `OptionsCell<SplitterOptions>`。
- **手順**: `zoom_pan.rs`、`space_pan.rs`、`splitter.rs`。APG の commit で直すのは `kivact-ui` の `splitter.rs`・`split.rs`。
- **APG**: Window Splitter。
- **試験**:
  - 足す: `interaction_splitter.rs` に `splitter_collapses_below_the_threshold`、`splitter_snaps_to_the_nearest_stop`。
  - APG の違いごとに `apg_window_splitter.rs`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_zoom_pan:: interaction_splitter:: apg_`
- **規模**: M。**並列**: I-0 の後の I・T・G・KL と並べてよい。

### I-6 通知と追従

- **目的**: 付録 A の SM-I21、SM-I22、SM-I23、SM-I25。
- **決定済みの形**:
  - SM-I21: `enum Corridor { Idle, Tracking { origin: Point, previous: Point }, Holding { origin: Point } }`。timer は `Step::after`。`progress` は `Notifier`。`use_hover_surface(..) -> HoverSurfaceApi`（N-F2 で済み）の公開の署名は変えない（K-2a・K-3b が使う）。
  - SM-I22: toaster は二つの machine に分ける。列の machine は、列、`limit`、`struct HoldSet(BTreeSet<HoldId>)` を持つ。toast ごとの machine は次を持ち、期限は `Step::after` で出す。
    ```rust
    enum ToastTimer { Running { deadline_ns: u64 }, Paused { left_ns: u64 }, Expired }
    enum ToastFocus { None, Inside(NodeToken) }
    ```
    toast の id は `Generation`。
  - SM-I23: `enum Follow { Following { expected: Option<f32> }, Detached { offset: f64 }, AwaitingHistory { anchor: f32, revision: Revision } }`。
  - SM-I25: `Notice { id: NoticeId, message }`。id は `Generation`。
- **手順**: `hover_corridor.rs`、`toaster.rs`、`follow_end.rs`、`announcer.rs`。
- **APG**: 当たる pattern はない（toast の表示は I-7、alert は K-6b）。
- **試験**: 足さない。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_toast:: hover::`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test toast_motion --test consumer_chat_scroll`
- **規模**: M。**並列**: I-0 の後の I・T・G・KL と並べてよい。

### U-1 ui の touch の gesture

- **目的**: 付録 A の SM-U01、SM-U02（gesture の状態）、SM-U03。設計書 19 章（touch だけの gesture は `DragApi` でなく machine）。
- **決定済みの形**:
  - 三つとも、今の raw pointer の handler を machine の event にする（`DragApi` は使わない）。受け付ける pointer kind・primary の判定は既存どおり（switch は mouse drag と pen の既存処理も保持）、`suppress_activation` を呼ぶ時と `Handled` の値は今のまま。slop の判定は `DragAxis::past_slop`。値は `InteractionPolicy.toast.swipe_slop`、`InteractionPolicy.select.sheet_slop` と、switch の private の定数 10 px。
  - SM-U01: `enum SwitchGesture { Idle, Pressed { origin: Point, started_on: bool }, Dragging { axis: Axis, visually_on: bool } }`。disable は `Idle` への遷移で、`generation` の counter を消す。
  - SM-U02: `enum Swipe { Rest, Pressed { pointer: PointerId, origin: Point }, Dragging { pointer: PointerId, offset: f64 }, Blocked { pointer: PointerId } }`。閉じる動き（`Visual::Dismiss` と `frame_loop`）はこの unit では残す（I-7 が消す）。
  - SM-U03: `enum SheetGesture { Idle, Pressed { pointer: PointerId, origin: Point }, Moved { pointer: PointerId }, Released(Outcome) }`。sheet の開閉は I-2a の `SelectState::Open { surface: Surface::Sheet, .. }`（`OpenSheet` event）で表し、`sheet_open` の signal を消す。
- **手順**: `kivact-ui/src/switch_drag.rs`、`toast_swipe.rs`、`select.rs`（touch の sheet の部分）。
- **試験**: 足さない（soft の touch の試験が回帰の網）。
- **unit の確認**:
  - `cargo check -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- interaction_select:: ui_controls:: touch_scroll::`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test switch_touch --test toast_swipe --test selector_touch`
- **規模**: M。**並列**: I-2a の merge の後。I-3〜I-6 と並べてよい。

### U-2 ui の編集と入力の draft

- **目的**: 付録 A の SM-U05、SM-U901、SM-U902、SM-U904、SM-U905。
- **決定済みの形**:
  - SM-U05: `enum InlineEdit<K> { Idle, Requested { key: K }, Editing { key: K, draft: String, session: EditSession<String> } }`。条件付き mount の前は Requested とし、field mount で完成した draft/session を渡して Editing へ遷移する。caller の `Binding<Option<K>>` は編集要求と `Editing` の key の写しで、`Sent` で書く。
  - SM-U901: `into_node` で draft と validity を決めてから、callback を引数に取る完成した input を作る。`Rc<OnceCell<..>>` を消し、`enum InputDraft { Immediate, Drafting(Installed) }`。公開の builder のメソッドはそのまま。
  - SM-U902: `Signal<NumberText>` を machine が持ち、`ValueChanged`・`Typed`・`FocusOut` を受ける。数値・文字・`produced` は一つの遷移で決める（今の文字の保持の条件はそのまま）。
  - SM-U904: `enum FilterDraft { Valid { text: String, value: Option<FilterValue> }, Invalid { text: String } }` を一つの signal に置く。valid は memo。外へ書いた値は `Sent<Option<FilterValue>>`。
  - SM-U905: progress は `enum ProgressMotionPhase { Stopped, Running }`。位相は今と同じく clock の絶対時刻から計算する。button は `enum BusyPhase { Idle, Waiting, Revealing, Shown }`。待ちは `Step::after`、frame の loop の start と stop は遷移の effect。
- **手順**: `kivact-ui` の `inline_edit.rs`、`text_draft.rs`、`text_input.rs`、`number_input.rs`、`data_table/filters.rs`、`progress.rs`、`button.rs`。APG の commit で直すのは `number_input.rs`。
- **APG**: Spinbutton（`number_input`）。
- **試験**: APG の違いごとに `apg_spinbutton.rs`。
- **unit の確認**:
  - `cargo check -p kivact-ui --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- ui_field:: ui_controls:: ui_data_table:: ui_standard:: apg_`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test table_column_filters --test button_busy --test progress_motion`
- **規模**: M。**並列**: I-3、I-5a、I-4b の merge の後。U-1・I-6 と並べてよい。

### I-7 出現と退場の一本化

- **目的**: 設計書 17.10 節（19 章で変えた所を含む）。付録 A の SM-I01（presence の部分）、SM-U02（閉じる動き）、SM-U04、SM-U06、SM-U903。
- **決定済みの形**:
  - 退場と出現は engine の presence（E-9a・E-9b: `Style::enter_size(Orientation)`・`exit_size(Orientation)`、`TransitionProp::Size`、`NodeRef::presence() -> Value<Presence>`、`Presence::{Entering, Present, Leaving}`）だけで描く。
  - disclosure は閉じたら内容の node を unmount する。高さの動きは recipe の `enter_size(Orientation::Vertical)`・`exit_size(Orientation::Vertical)` で、時間は `UiTokens.disclosure.motion`。`ui::disclosure(..).animated(bool)` は、その一つの部品の `motion.animate` の上書きとして残す。
  - `DisclosureOptions.leaving` と `interaction::use_content_reveal`・`ContentReveal` を消す。
  - toast の swipe で閉じる動きは、drag の位置から始まる toast の node の退場にする（`Visual::Dismiss` と `frame_loop` を消す）。
  - toast の行（SM-U903）は、行の node の `exit_size(Orientation::Vertical)` で縮める。`toast_rows.rs` の slot の二重の列と世代の counter を消す。
  - tree の行の reveal（SM-U04）は、子の行の node の `enter_size`・`exit_size`。memo の中の `previous_open` の書き込みと `last_height` を消す（`RowReveal` の enum は作らない）。
- **手順**:
  1. interaction の `disclosure.rs`、`content_reveal.rs`（消す）、`lib.rs`。
  2. ui の `disclosure.rs`、`toast.rs`、`toast_swipe.rs`、`toast_rows.rs`（消す）、`tree.rs`、`recipes.rs`（enter・exit の style の field を足す）。
  3. soft の `use_content_reveal` の 4 か所（`code_block.rs`、`collapsible.rs`、`side_nav.rs`、`soft_chat_tools.rs`）を、node の presence と `exit_size` で書き直す。
  4. 5 kit の recipe の disclosure・toast・tree の行に enter・exit の style を足す（今の動きの時間と easing を写す）。
- **振る舞いの変更**: 閉じた disclosure の内容が unmount される。閉じる動きの仕組みが変わる。
- **試験**: 足す。`dev/tests/kivact-public-api/tests/e2e/presence.rs`（新しい file）に `disclosure_unmounts_closed_content_after_its_exit`、`toast_swipe_dismiss_exits_from_the_drag_offset`、`toast_row_shrinks_while_it_exits`、`tree_row_reveal_expands_with_enter_size`。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui -p kivact-ui-basic -p kivact-ui-pop -p kivact-ui-game -p kivact-ui-creative -p kivact-ui-soft --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- presence:: interaction_toast:: interaction_tree:: motion::`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft --test toast_motion --test toast_swipe --test tree_reveal --test consumer_chat_scroll`
- **規模**: M。**並列**: E-9a・E-9b・I-1・I-4a・I-6・U-1 の merge の後。KL-soft とは `soft_chat_tools.rs` を共有するので同時に動かさない。

---

## 9.5 Phase T・G・KL: 拡張口・不足・kit の中の状態

wave W5（I・U と同じ wave）。crate が I・U と重ならないので、9 章の順の空いた枠に入れる。前提はそれぞれの unit に書いた。

### T-1 編集する node の制約の解除と公開の文字編集の API

- **目的**: 設計書 17.12 節（19 章で変えた所を含む）。
- **決定済みの形**:
  - 編集する node に `draw`・`draw_over` と、行の中の inline の子（`inline_view`）を許す。engine の editor の paint は、自分の文字・選択・caret を描いた後に node の draw layer を重ねる。今の `Unsupported` の報告を消す。
  - `TextEditing::highlight` を `line_styles` と改名し、範囲の型を広げる（行をまたぐ状態 `S` の受け渡しは今のまま）:
    ```rust
    pub fn line_styles<T, S, F>(self, content: impl IntoValue<T>, f: F) -> Self
    where T: Clone + PartialEq + 'static, S: PartialEq + Default + 'static,
          F: Fn(&T, &str, &S) -> (Vec<StyledRange>, S) + 'static;
    pub struct StyledRange {                     // 旧 TextHighlight（range, color）
        pub range: Range<usize>,                 // 行の中の UTF-8 の byte。改行を含まない
        pub color: Option<Color>,
        pub background: Option<Color>,
        pub decoration: Option<TextDecoration>,  // G-4a の TextDecorationStyle を含む
    }
    pub fn decorations<T: Clone + PartialEq + 'static>(self, content: impl IntoValue<T>, f: impl Fn(&T) -> Vec<Decoration> + 'static) -> Self;
    pub struct Decoration { pub range: Range<usize> /* 文書の byte */, pub kind: DecorationKind, pub color: Color }
    pub enum DecorationKind { Underline(TextDecorationStyle), Outline, Background }
    ```
  - `kivact-editing::LineColors` は内部の型のまま、範囲ごとに `StyledRange` の内容を運ぶように広げる。
  - `Node::draw_with_layout(impl Fn(&mut Canvas, &TextLayout) + 'static)` を足す（`TextLayout` は編集する node の行の箱、caret、選択の矩形、当たり判定）。
- **手順**: `kivact-core/src/primitive/text_editing.rs`（module の doc の「native leaf」の記述も事実に直す）、`kivact-core/src/view/editing_data.rs`、`kivact-editing/src/line_colors.rs`、engine の editor の paint と `contract/node.rs`。basic の `code_block/view.rs` の `highlight` の呼び出しを `line_styles` に直す。
- **試験**: 足す。`text_editing.rs` に `an_editing_node_draws_its_own_layer_over_the_text`、`line_styles_color_each_line`、`decorations_mark_a_range`。
- **unit の確認**:
  - `cargo check -p kivact-editing -p kivact-engine -p kivact-core -p kivact-ui -p kivact-ui-basic --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- text_editing::`
- **規模**: M。**並列**: 前提は E-4・E-7・G-4a（`TextDecorationStyle`）。I・U と並べてよい。

### T-2 公開 API だけの text field の証明

- **目的**: 設計書 17.12 節の証明。
- **手順**:
  1. `dev/tests/kivact-public-api/tests/e2e/custom_text_field.rs` に、`kivact::custom` と `kivact::interaction` だけで作った code editor の部品を書く。行番号は draw、構文の色は `line_styles`、inline の部品は `inline_view` で作る。
  2. `ui::text_input` の E2E と同じ操作を確かめる: 文字の入力、IME の変換と確定、選択、clipboard、undo の `EditPhase`、支援技術の text と選択。
  3. 足りない口が見つかったら、既存の型（`TextEditing`、`ImeSession`、`NodeRef`、`TextEditApi`）のメソッドなら足し、`DECISIONS.md` に一行書く。新しい型か module が要るときだけ止めて報告する。
- **試験**: 足す（上の file。名前は `custom_text_field_<操作>`）。
- **unit の確認**: `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- custom_text_field::`
- **規模**: M。**並列**: T-1 と I-3 の merge の後。

### G-1a Path

- **目的**: 設計書 17.17 節の Path の部分。
- **決定済みの形**（`kivact-style/src/path/`）:
  ```rust
  impl Path {
      pub fn contains(&self, p: Point, rule: FillRule) -> bool;        // flattened(0.1) の輪郭の winding で決める
      pub fn stroke_contains(&self, p: Point, stroke: &Stroke) -> bool; // stroke_outline(stroke).contains(p, FillRule::NonZero)
      pub fn arc_to(self, p1: Point, p2: Point, radius: impl Into<f64>) -> Self; // canvas の arcTo
      pub fn svg_arc_to(self, radii: (impl Into<f64>, impl Into<f64>), rotation: impl Into<f64>, large_arc: bool, sweep: bool, to: Point) -> Self; // 旧 arc_to
      pub fn parse_svg(d: &str) -> Result<Path, UiError>;               // SVG の path data の全 command
  }
  ```
- **手順**: `geometry.rs`（`contains`、`stroke_contains`）、`commands.rs`（`arc_to`、`svg_arc_to`、`parse_svg`）。旧 `arc_to` の呼び出し元を `svg_arc_to` に直す（kit・例を含む。`grep -rn "\.arc_to(" crates examples dev`）。
- **試験**: 足す。`canvas.rs` に `path_contains_matches_fill_rule`、`canvas_arc_to_rounds_a_corner`、`parse_svg_draws_the_same_path`。
- **unit の確認**:
  - `cargo check --workspace --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- canvas::`
- **規模**: S。**並列**: I・U と並べてよい。

### G-1b snapshot

- **目的**: 設計書 17.17 節の `NodeRef::snapshot`。
- **決定済みの形**: `NodeRef::snapshot(&self) -> impl Future<Output = Result<Pixels, UiError>>`。node の部分木の scene を、Offscreen と同じ CPU の rasterizer（`kivact-engine` の `OffscreenRenderer`）で描く（GPU から読み戻さない）。部分木に GPU 共有の frame（`RenderCmd::External`）があって CPU の画素の代わりがないときは、`ErrorKind::Unsupported` の `UiError` を返す。
- **手順**: engine に部分木の scene を切り出す関数を足し、`kivact-core/src/view/node_ref.rs` に `snapshot` を足す。
- **試験**: 足す。`canvas.rs` に `snapshot_returns_the_node_pixels`、`snapshot_of_a_gpu_frame_is_unsupported`。
- **unit の確認**:
  - `cargo check -p kivact-engine -p kivact-core --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- canvas::`
- **規模**: M。**並列**: I・U と並べてよい。G-2 とは engine の別の module。

### G-2 明示の pointer の capture

- **目的**: 設計書 17.17 節の capture。
- **決定済みの形**: `PointerEvent::capture(&self)`・`release_capture(&self)`。handler の中で呼ぶと、その node にこの pointer を送り続ける（engine の `Route` の capture を使う）。今の「押している間の暗黙の capture」は既定のまま。
- **手順**: `kivact-core/src/view/mount/part_pointer.rs` と engine の `model/pointer/`。
- **試験**: 足す。`pointer.rs` に `captured_pointer_keeps_moving_events_outside_the_node`、`release_capture_returns_to_hit_testing`。
- **unit の確認**:
  - `cargo check -p kivact-engine -p kivact-core --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- pointer::`
- **規模**: S。**並列**: I・U と並べてよい。

### G-3 モバイルの境界

- **目的**: 設計書 17.16 節。V-3 が `Environment.keyboard_inset` と engine の受け渡しを、N が `platform::keyboard_inset() -> Value<Insets>` を作った。この unit は残りを作る。
- **決定済みの形**:
  - `InputPurpose::{Text, Number, Decimal, Email, Url, Phone, Password, Search}`（`kivact-types::input`）。
  - 四つの `password(..)`（`ui::TextInput`、`interaction::TextEditApi`、`custom::TextEditing`、`custom::ImeSession`）を `purpose(impl IntoValue<InputPurpose>)` にする（`password` は消す）。
  - 経路は `TextEditing::purpose` → engine → `ShellWindow::set_ime_purpose`。winit は Normal・Password だけなので、ほかは Normal に写す。
  - `ShellEvent::KeyboardInset(Insets)` を足し、`Environment.keyboard_inset` を更新する。
  - `LifecycleEvent::SaveState`、`platform::save_state(key: &'static str, bytes: Vec<u8>)`、`platform::restored_state(key: &'static str) -> Option<Vec<u8>>`、`Capabilities.state_restoration`。desktop の backend は `false`（保存しない）、`arvane-virtual` は台本で保存と復元を試せる。
- **手順**: types、`arvane-shell`、`arvane-native`（Normal・Password への写し）、`arvane-virtual`（台本に keyboard の inset と状態の保存・復元を足す）、engine、core、interaction の `text_edit/handle.rs`、ui の `text_input.rs`。kit の `password(` の呼び出し元を直す。
- **試験**: 足す。`input_platform.rs` に `text_input_purpose_reaches_the_platform`、`keyboard_inset_is_observable`、`saved_state_is_restored_on_the_next_launch`。
- **unit の確認**:
  - `cargo check --workspace --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- input_platform:: text_editing::`
- **規模**: M。**並列**: I-3 と U-2 の merge の後（`text_edit/handle.rs`、`text_input.rs`）。

### G-4a 文字色・下線の種類・visibility

- **目的**: 設計書 17.18 節の C-05、C-25、C-34。
- **決定済みの形**:
  - C-05: 文字色の補間は、既存の `TransitionProp::Paint` が既に含む（`kivact-style/src/sheet.rs` の `TransitionProp` の doc）。新しい variant は足さず、試験だけを足す。
  - C-25: `pub enum TextDecorationStyle { #[default] Solid, Dashed, Dotted, Wavy }`（`kivact-types::text`）。`TextDecoration` に `pub style: TextDecorationStyle` を足し、`Style::text_decoration_style(impl IntoValue<TextDecorationStyle>)` から paint まで通す。
  - C-34: `Style::visibility(Visibility)`、`enum Visibility { #[default] Visible, Hidden }`。`Hidden` は layout に参加し、paint・支援技術・focus・当たり判定から外れる。
- **手順**: types、style、lower、engine の paint・semantics・focus・hit test。
- **試験**: 足す。`style_paint.rs` に `text_color_transitions_with_paint`、`text_decoration_style_draws_wavy_lines`、`visibility_hidden_keeps_layout_but_hides_the_node`。
- **unit の確認**:
  - `cargo check -p kivact-types -p kivact-style -p kivact-engine -p kivact-core --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- style_paint::`
- **規模**: M。**並列**: I・U と並べてよい。T-1 の前に merge する。

### G-4b layout の補間と transition の retarget

- **目的**: 設計書 17.18 節の C-08、C-10。
- **決定済みの形**:
  - C-08: `Style::layout_transition(Transition)`。layout の結果の位置と大きさの変化を、engine が前の位置から補間する（FLIP。layout は一度だけで、paint の時に前の矩形から今の矩形へ補間する）。
  - C-10: `Transition::retarget(self, Retarget) -> Self`、`enum Retarget { #[default] Restart, Proportional }`。`Proportional` は、途中で目標が変わったら、残りの距離に比例した時間にする。
- **手順**: style、engine の layout と paint、runtime の motion（`Animated` の retarget）。
- **試験**: 足す。`motion.rs` に `layout_transition_moves_from_the_previous_position`、`proportional_retarget_shortens_the_remaining_time`。
- **unit の確認**:
  - `cargo check -p kivact-style -p kivact-engine -p kivact-runtime -p kivact-core --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- motion::`
- **規模**: L。**並列**: I・U と並べてよい。G-4a と同時に動かすときは engine の paint の file を手で合わせる。

### G-4c theme の hint・pointer の種類・scroll の snap と慣性

- **目的**: 設計書 17.18 節の C-20、C-21、C-37（18.11 節で変えた所を含む）。V-3 が `ThemeHint`、`PointerClass`、`PointerSettings.{primary, fling_deceleration, fling_min_velocity}` を作った。
- **決定済みの形**:
  - C-20: `platform::theme_hint() -> Value<ThemeHint>` は N-G5、`From<ThemeHint> for ThemeMode` は N-F1 で前倒し済み。この unit は native 追従を確認する。`ThemeMode::System` は作らない。
  - C-21: backend の判定。Windows は `GetSystemMetrics(SM_MOUSEPRESENT) == 0` かつ `SM_DIGITIZER` が `NID_READY` を含むとき `Coarse`、それ以外は `Fine`。X11・Wayland は `Fine`。`arvane-virtual` は台本で指定する。値は `Environment` の更新で変わる。
  - C-37: `Style::scroll_snap(ScrollSnap)`（`ScrollSnap::{None, Start, Center, End}`。既定 None、ScrollAxes に従い直下の in-flow の子の border box を viewport に整列、Contents は透過）、touch の fling の慣性、`ScrollBehavior::Smooth(Easing)`（今の `Smooth` に easing を持たせる）。慣性は、離したときの速度（`DragVelocity` と同じ計算）から `PointerSettings.fling_deceleration` で減速し、`fling_min_velocity` を下回ったら止める。snap がある scroll は、止まる位置を `nearest_snap` で寄せる。
- **手順**: core の `platform`、`arvane-native`（Windows の判定）、`arvane-virtual`、style、engine の scroll、`kivact-core/src/view/node_ref.rs` の `ScrollBehavior`。
- **試験**: 足す。`environment.rs` に `theme_hint_follows_the_platform`、`pointer_class_is_reported`。`touch_scroll.rs` に `fling_decelerates_and_stops`、`scroll_snap_settles_on_a_stop`、`smooth_scroll_uses_the_easing`。
- **unit の確認**:
  - `cargo check --workspace --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- environment:: touch_scroll:: layout_scroll::`
- **規模**: M。**並列**: I・U と並べてよい。G-4b とは engine の scroll と layout の file が別。

### G-5 非同期の標準部品

- **目的**: 設計書 17.19・18.5 節。
- **決定済みの形**:
  ```rust
  pub fn async_view<T, V: IntoView>(source: impl AsyncSource<T>, ready: impl Fn(T) -> V + 'static) -> AsyncView<T>;
  impl<T> AsyncView<T> {
      pub fn loading<V: IntoView>(self, f: impl Fn() -> V + 'static) -> Self;                  // 既定は ui::spinner の中身
      pub fn fallback<V: IntoView>(self, f: impl Fn(UiError, Retry) -> V + 'static) -> Self;   // 既定は retry の button を持つ表示
  }
  pub trait AsyncSource<T>: sealed::Sealed { .. }  // Resource<T>（retry = refetch）と Action<T, I>（retry = retry）
  ```
  - `IntoValue<AsyncState<T>> for Resource` は作らない。
  - `ui::spinner`・`ui::alert` は K-6a・K-6b で作るので、既定の表示はこの unit では `ui` の private の関数で描く（K-6a・K-6b が中核の部品に差し替える）。
  - facade の feature `tokio`（既定は無効、実装は `kivact-runtime`）。`kivact::task::on_tokio<T: Send + 'static>(fut: impl Future<Output = T> + Send + 'static) -> impl Future<Output = T>` は、App が持つ一つの tokio の runtime のスレッドで `fut` を動かし、終わったら UI を起こす。返した future を drop したら tokio の task を abort する。
- **手順**:
  1. `kivact-runtime`（`on_tokio`、`AsyncSource`）、`kivact-ui` の新しい file `async_view.rs`、facade の `Cargo.toml` の feature。
  2. `examples/kivact/async` を `async_view` で書き直す。
  3. guide の how-to に「HTTP で JSON を読む」を en・jp で足す（`docs/for-kivact-user/{en,jp}/how-to-guide/`。`?` で `UiError` にする）。
- **試験**: 足す。`async_tests.rs` に `async_view_shows_loading_then_ready`、`async_view_fallback_retries`、`on_tokio_future_resolves_and_wakes_the_ui`、`on_tokio_task_is_aborted_with_its_owner`。
- **unit の確認**:
  - `cargo check -p kivact-runtime -p kivact-ui -p kivact -p kivact-example-async --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- async_tests::`
  - `cargo test -p kivact-docs`
- **規模**: M。**並列**: I・U と並べてよい。

### KL-basic basic の中の状態

- **目的**: 付録 A の SM-K914。
- **決定済みの形**: 既存の `Phase::{Asking, Submitted { at, answers }}` を machine の状態にし、`StepChanged`・`AnswersChanged`・`Submit` を遷移の入口にする（提出と失効を一つの所で決める）。
- **手順**: `crates/kivact-ui-basic/src/questionnaire.rs`。
- **試験**: 足さない。
- **unit の確認**:
  - `cargo check -p kivact-ui-basic --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-basic`
- **規模**: S。**並列**: R-6 の後ならいつでもよい。

### KL-pop pop の中の状態

- pop の状態の項目は、全部が中核へ上がる部品の中にある（K-1a・K-1b）。この unit で行う作業はない。サブエージェントを起こさない。

### KL-game game の中の状態

- **目的**: 付録 A の SM-K33 の `reveal_once` の部分（frame の loop と focus の復元は K-4）。
- **決定済みの形**: `enum Reveal { Pending, Done }`。
- **手順**: `crates/kivact-ui-game/src/display.rs`。
- **試験**: 足さない。
- **unit の確認**:
  - `cargo check -p kivact-ui-game --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-game`
- **規模**: S。**並列**: R-6 の後ならいつでもよい。

### KL-creative creative の中の状態

- **目的**: 付録 A の SM-K30、SM-K901、SM-K902、SM-K903、SM-K904、SM-K905、SM-K906、SM-K907、SM-K908、SM-K920、SM-K921。
- **決定済みの形**:
  - SM-K30: drag の開始時の値は `DragApi::source(payload)` の payload（`DragUpdate::payload`）で後の update に渡し、var への写しを消す。`CanvasGesture` の押しただけの状態は `DragApi::slop(..)` の `Pending` を使う（I-3）。
  - SM-K901:
    ```rust
    pub enum PieState { Closed, Open { mode: PieMode, center: Point, hot: Option<usize> } }
    pub enum PieMode { Click, Held }
    ```
    閉じる遷移は `Closed`（公開の型）。
  - SM-K902: blocked は part の値からの memo。drag は完成した controller の構築の引数。無効化と focus の out を編集の machine が受け、cancel を同じ遷移で決める。
  - SM-K903: `Signal<ColorControls>`、`struct ColorControls { channel: Channel, swatch: SwatchSelection, hue: f64 }`、`enum SwatchSelection { None, Selected(usize) }`。無彩色で hue を保つ今の規則は保つ。
  - SM-K904: range の machine が press と `ValueChanged { before, now }` を受け、`RangePart` の変更を遷移で決める。
  - SM-K905: `enum PickerColumn { Empty, Active(usize) }`。`QueryChanged`・`ItemsChanged`・column の移動の遷移で選ぶ。
  - SM-K906: 各 editor の gesture・接続の machine が、focus の out・visibility・無効化・data の変更を受け、取消しと rollback を同じ遷移で決める。
  - SM-K907: target の source の変更を controller の遷移で受け、id の割当てと `Scene` を同時に決める。割当ては `Generation` を包む allocator。
  - SM-K908: `enum MeterClip { Clear, Held }`（`LevelChanged`、`Reset`）。
  - SM-K920: gesture の machine が part の入力の変化を受け、無効化と cancel を同じ遷移で決める。
  - SM-K921: `enum ColorPopupPhase { Closed, Editing { initial: Color } }`。
- **手順**: 二つのサブエージェントに file の組で分けてよい。
  - (a) `pie.rs`、`color_wheel/*`、`gradient/*`、`bezier/*`、`color_popup.rs`、`range.rs`、`numeric.rs`、`feedback.rs`、`panel_group.rs`
  - (b) `editors/*`、`picker.rs`、`audio/meter.rs`
- **試験**: 足さない（creative の consumer の試験が回帰の網）。
- **unit の確認**:
  - `cargo check -p kivact-ui-creative --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-creative`
- **規模**: L（(a)・(b) それぞれ M）。**並列**: I-3 の merge の後。K-4（outliner）と K-8（`viewport_surface.rs`）の file には触らない。

### KL-soft soft の中の状態

- **目的**: 付録 A の SM-K08、SM-K09、SM-K10（`outline.rs` の部分）、SM-K21、SM-K26、SM-K909、SM-K910、SM-K915。
- **決定済みの形**:
  - SM-K08: `enum Stream { Idle, Revealing { shown: usize }, Done }`。進みは `Step::after`。
  - SM-K09: private `enum AppShellMobileState { Desktop, Mobile { nav: MobileNavState } }`、`enum MobileNavState { Closed, Open }`（既存の公開 AppShellMobile/MobileNav と名前を分ける）。
  - SM-K10（outline）: 端で止まる振る舞いのまま、`enum OutlineFocus { Unset, At(String) }`。
  - SM-K21:
    ```rust
    enum PowerSearchState<K> { Closed, Composing { draft: FilterDraft }, Editing { index: usize, original: SearchFilter<K>, draft: FilterDraft } }
    struct FilterDraft { field, operator, value }
    ```
  - SM-K26: `enum OutlineScroll { Idle, Scrolling { target: String } }`。抜けるのは今と同じく 250 ms の `Step::after`。
  - SM-K909: `enum BlockEditPhase { Idle, Focused { target: NodeRef, selection: BlockSelection, composition: BlockComposition }, AwaitingMount { block: usize } }`、`BlockSelection::{Caret, Whole}`、`BlockComposition::{Idle, Composing}`。
  - SM-K910: timer は `enum ElapsedTimer { Running { start: SystemTime, next: Duration } }` が tick を `Step::after` で出す。文字は memo。timestamp も同じ形。
  - SM-K915: `enum SkipFocus { Idle, Requested }`。
  - rich_blocks は Machine が状態を所有し、既存 rich_text toolbar と共有する target/composition port に effect から投影する。Timer/Timestamp は constructor で完成した Machine と lazy memo を作り、構築 option だけの Settings を builder が変更する（表示の後付け初期化 slot を作らない）。
- **手順**: `soft_chat_tools.rs`（streaming text の部分）、`mobile_nav.rs`、`outline.rs`、`power_search.rs`、`rich_blocks.rs`、`timer.rs`、`timestamp.rs`、`app_shell.rs`。
- **試験**: 足さない（soft の試験が回帰の網）。
- **unit の確認**:
  - `cargo check -p kivact-ui-soft --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft`
- **規模**: M。**並列**: R-6 の後。I-0 の `theme.rs` には触らない。I-7 とは `soft_chat_tools.rs` を共有するので同時に動かさない。

---

## 10. Phase K: 公式 kit

wave W6。付録 C の作業。

**K の各 unit に共通の決まり**:

- **二段で行う**:
  1. 中核の段（サブエージェント一つ）: `kivact-interaction` と `kivact-ui` に部品を作り、標準の E2E を `dev/tests/kivact-public-api/tests/e2e/` に書く。
  2. kit の段（関係する kit ごとに一つ、並べてよい）: kit の版を消し、recipe と variant を足し、catalog の page（消さない。kit の look で中核の部品を見せる page にする）と kit の E2E を中核の部品で書き直す。
- **和集合**: 中核の部品は、元の kit の版の機能の和集合を持つ（付録 C.2 の「和集合」の欄）。kit を縮めない。kit の部品を消すときは、その部品の公開の option が中核の部品のどこに当たるかの対応表を報告に書く。
- **名前**: 付録 B.1 と B.10 の名前で作る（B の名前が C.2 の名前より優先する。例: `disclosure_group(..).panel(..)`、`navigation_menu(..).link(..)`、`combobox(..).chips()`、`ui::item_row`、`heading(text, level)`、`pagination(count, page)`、`card(..).on_press(..)`・`.selected(..)`、`StepperItem::new(label)`、`DialogMode`、`DismissOn`、`Snap`、`is_open()`、`is_copied()`、`keep_focus()`、`Notifier`）。引数の型は付録 C.2 の引数の型の表。
- **token**: 中核へ上げた部品ごとに `UiTokens` の入れ子の struct（その部品の file に置く）と recipe の表を作る。token にするのは、元の kit が上書きしていた値と、theme から読む値だけ。ほかの literal は既定の recipe の中に残す。元の kit の値は、その kit の theme の `UiTokens` に入れる。今ある部品の recipe を token に直すことはしない。
- **variant の誤用**: 別の kit の theme の下で kit の variant を使ったら、中核の既定の recipe で描き、`ReportOnce` の diagnostic を一度出す。
- **状態**: 部品の状態は付録 A の担当の項目の「解消後」の形の `Machine`（9 章の共通の形）。
- **APG**: unit に APG の pattern があれば、9 章の APG の合わせ方で合わせ、E2E を足す。
- **文書**: 新しい recipe field を足すときは該当 kit の README の `kivact-recipes` 一覧と `theme_conformance.rs` の実 getter の照合を更新する。中核の部品の rustdoc と、`docs/for-kivact-ui-user/{en,jp}/<kit>/` の該当の page を新しい書き方にする（F-1 は対の確認だけ）。
- **unit の確認**:
  - `cargo check -p kivact-interaction -p kivact-ui -p <触った kit> -p <その catalog> -p kivact-public-api-tests --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e -- <足した試験の module>::`
  - `KIVACT_TEST_MIRROR=off cargo test -p <触った kit> -p <その catalog>`
  - `cargo test -p kivact-docs`

**順と並列**:

- K-0 が先。
- K-0 の後、K-1a、K-1b、K-2a、K-2b、K-3a、K-3b、K-4、K-5、K-6a、K-6b は、中核の段どうしを並べてよい。共有の file は `kivact-ui/src/lib.rs`、`recipes.rs`、`tokens.rs` と、kit の `lib.rs`・`recipes*.rs`・`theme.rs` で、足すだけ。merge はオーケストレータが合わせる。
- K-8 は G-1b・G-2・I-5b の後。K-9b は K-2a の後（`use_copy_feedback`）。K-9a は K-2a の後。
- K-7a〜K-7e は、その kit の K の unit が全部 merge された後。H-G は各 unit に書いた順。

### K-0 kit のための中核の部品

- **目的**: 付録 C.1 の KIT-1・KIT-2、設計書 18.15 節の `UiTokens` の残り。
- **決定済みの形**:
  - `NodeRef::scroll_state() -> Value<ScrollState>`、`enum ScrollState { NotScrollable, AtStart, Middle, AtEnd }`。
  - `NodeRef::image_status() -> Value<ImageStatus>`、`enum ImageStatus { Loading, Ready, Failed }`。
  - `FrameLoop::while_present(self, node: NodeRef) -> Self`（その node が present の間だけ動く）。公開 handle は core の motion 境界で定義し、runtime の単一 loop owner を包む。runtime の gate bridge へ readonly presence を渡し、core 側に rate/state の写しを作らない。
  - `RadioGroupVariant::Segmented`（標準の variant）。
  - kit の `Color` enum を `ColorName` に改名する（basic、pop、game、creative）。soft は `ColorName` enum を新しく足す（`Colors` の Color 型の scalar field 名と一対一。hue・data・syntax の複合 group は `use_tokens` で読む）。
  - game と creative の `ThemeOptions` trait を消し、`theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme`（`ThemeOptions` は struct。game は `{ density }`、creative は `{ accent, density }`。creative の `install()` は `theme_with` に統合する）。
  - kit の `tokens()`（basic と soft の `theme_access.rs`）を消し、`use_tokens::<Tokens>()` を使う。
  - 各 kit の README に、kit が埋めた recipe の表の列挙を書く（KIT-3）。`theme_conformance` の E2E で、その列挙と一致することを確かめる。
- **手順**: 中核の段（core、runtime、ui）の後に、kit の段を kit ごとに一つ（5 つ並べる）。soft の `tokens()` は約 320 か所の機械的な置き換え。
- **試験**: 足す。`node_ref.rs`（新しい file）に `scroll_state_follows_the_scroll_position`、`image_status_reports_a_failed_image`、`frame_loop_runs_only_while_present`。`theme_conformance.rs` に `kit_readme_lists_the_filled_recipes`。
- **unit の確認**: 共通の確認（5 kit と 5 catalog）。
- **規模**: M（中核の段 S、kit の段 S×5）。**並列**: 単独で先に行う。

### K-1a calendar と日付の入力

- **実装時の契約補完**: `calendar(impl Into<CalendarSelection>)`。CalendarSelection は Single/Range の binding descriptor で、Signal/Binding<Option<CalendarDate/DateRange>> の4つの From と恒等変換を受ける。集合用 IntoSelection は変えない。RangePick::Picking は anchor/preview と取消用 before: Option<DateRange> を持つ。time_input.format は TimeStyle、date_time_input は time_format(TimeStyle) を持つ。日付 field は placeholder/format_with/valid を保持し、valid は遷移から出力する。

- **目的**: 付録 C.2 の calendar と date inputs。付録 A の SM-K18、SM-K19、SM-K20、SM-K28（month swipe）、SM-K29、SM-K911、SM-K912、SM-K913。
- **決定済みの形**:
  - calendar:
    - `ui::calendar(selection: impl IntoSelection<CalendarSelection>)`（`Signal<Option<CalendarDate>>` と `Signal<Option<DateRange>>` から作る）。
    - builder: `.month(impl IntoBinding<CalendarMonth>)`、`.today(impl IntoValue<CalendarDate>)`、`.week_start(impl IntoValue<Weekday>)`（既定は locale）、`.min(impl IntoValue<Option<CalendarDate>>)`、`.max(..)`、`.disabled_dates(impl Fn(CalendarDate) -> bool + 'static)`、`.marks(impl Fn(CalendarDate) -> Option<Tone> + 'static)`。
    - 振る舞いは既存の `use_calendar_grid`。
  - 日付の入力:
    - `ui::date_input(label: impl IntoText, value: impl IntoBinding<Option<CalendarDate>>)`、`ui::date_range_input(label, range: impl IntoBinding<Option<DateRange>>)`、`ui::time_input(label, value: impl IntoBinding<Option<TimeOfDay>>)`、`ui::date_time_input(label, value: impl IntoBinding<Option<DateTime>>)`。
    - builder: `.format(impl IntoValue<DateStyle>)`、`.min`、`.max`、`.week_start`、`.presentation(DatePresentation)`。
    - `DatePresentation::{Adaptive, Desktop, BottomSheet}` は soft から中核へ移す（構造の option。`Adaptive` は `Capabilities.touch` で sheet を選ぶ）。sheet の wheel の panel（soft の `date_touch.rs`）は、中核の部品の sheet の中身として `kivact-ui` へ移し、`use_scalar` の上に作り直す（SM-K29。`release` は `Notifier`）。
    - 文字の解釈は `Locale::parse_date`（L-1）。月名・曜日名は `Locale::month_name`・`weekday_name`。kit の月名の表と暦の計算の写し（basic・soft の `MONTHS`、soft の `timestamp.rs` の英語の表と `civil_from_days` の写し）を消す。
  - 状態:
    - SM-K18: `enum DateInputState { Closed { draft: Draft }, Open { opened_by: OpenedBy, draft: Draft } }`、`enum Draft { Empty, Valid(CalendarDate), Invalid(String) }`。
    - SM-K19: `Open { panel: Panel::{Date, Time}, .. }`。
    - SM-K20: range の入力は `CalendarSelection::Range` を使い、start > end を遷移で作らない。
    - SM-K911: `enum TimeDraft { Empty, Valid { text: String, value: TimeOfDay }, Invalid(String) }`。
    - SM-K912: `enum RangePick { Idle, Picking { anchor: CalendarDate, preview: Option<CalendarDate> } }`。月の変更の読み上げは遷移の effect。
    - SM-K913: `enum TypedDate { Following(CalendarDate), Editing { text: String, validation: DateValidation } }`、`DateValidation::{Unchecked, Checked}`。
    - SM-K28（month swipe）: soft の month swipe は touch だけの gesture なので、machine と `DragAxis::past_slop`（6 px の名前の付いた定数）で書き、`sleep(1 ms)` を消す。
  - pop の非 `Option` の binding は `Option` に揃える。`week_starts_on` は `week_start` に揃える。
- **手順**:
  1. 中核の段: `kivact-ui` の `calendar.rs`・`date_input.rs`（新しい file）と wheel の panel。
  2. kit の段: basic（calendar、date_picker、date_range_picker）、pop（calendar、date_input、date_range_input、`form.rs` の typed date）、soft（calendar、4 つの入力、date_touch、month_swipe、timestamp の表）。
- **試験**: 足す。`calendar_selects_a_date_with_the_keyboard`、`calendar_range_picks_start_then_end`、`date_input_parses_typed_text_and_opens_the_calendar`、`date_input_bottom_sheet_picks_with_the_wheel`、`time_input_accepts_hours_and_minutes`（`ui_calendar.rs`、新しい file）。
- **unit の確認**: 共通の確認（basic、pop、soft）。
- **規模**: L（中核の段 L、kit の段 M×3）。**並列**: K-0 の後、ほかの K と並べてよい。

### K-1b carousel と dropzone

- **実装時の契約補完**: Snap::{None,Mandatory}（既定 Mandatory）、Validity::{Valid,Invalid(String)}。FileInfo は types::platform 正準の `{ path: PathBuf }`、From<PathBuf> と純粋な name() を持ち、filesystem を読まない。carousel の items_per_view/gap/padding/buttons/edge_fade は reactive。外部操作は caller の index binding、計測に基づく内部 scroll control は単一 UI owner。

- **目的**: 付録 C.2 の carousel、dropzone。付録 A の SM-K01、SM-K02、SM-K31。
- **決定済みの形**:
  - carousel:
    - `ui::carousel(index: impl IntoBinding<usize>, slides: impl IntoChildren)`。builder: `.looping()`、`.snap(impl IntoValue<Snap>)`、`.orientation(impl IntoValue<Orientation>)`、`.mode(CarouselMode)`（構造の option。`CarouselMode::{Translate, Scroll}`、既定 `Translate`）。
    - `interaction::use_carousel`。状態は `enum CarouselState { Idle { index: usize }, Scrolling { target_index: usize, settle_at_ns: u64 } }`、snap の待ちは `Step::after`（今の 120 ms）。
    - Shift+wheel の横の scroll は `ui::carousel` の wheel の handler に移す（engine の scroll の規則は変えない）。
  - dropzone:
    - `ui::dropzone(label: impl IntoText)`。builder: `.on_files(impl Fn(Vec<FileInfo>))`、`.validate(impl Fn(&FileInfo) -> Option<String> + 'static)`、`.multiple()`、`.pick_on_press()`。
    - `interaction::use_dropzone`。状態は `enum DropzoneState { Idle, DragOver { valid: Validity }, Picking, Failed(UiError) }`。kit は `std::fs` を呼ばない（soft の検証は caller の `validate` へ）。
- **APG**: Carousel。
- **試験**: 足す。`carousel_moves_with_arrows_and_reports_the_index`、`carousel_scroll_mode_snaps_to_a_slide`、`dropzone_accepts_dropped_files_and_rejects_invalid_ones`（`ui_carousel.rs`、`ui_dropzone.rs`）。APG の違いごとに `apg_carousel.rs`。
- **unit の確認**: 共通の確認（basic、pop、game、soft）。
- **規模**: M。**並列**: K-0 の後、ほかの K と並べてよい。

### K-2a disclosure group・hover card・drawer・copy feedback

- **実装時の契約補完**: CopyFeedbackApi.copy(text) は () の owner-scoped command。hover_card.open(binding) は binding を唯一の open authority にし、setter 拒否を尊重する。Disclosure.animated は IntoValue<bool>。独自 trigger は attach と標準 slot で構成し、animate_layout/trigger_view alias を作らない。

- **目的**: 付録 C.2 の disclosure group、hover card、drawer の swipe、copy feedback。付録 A の SM-K03、SM-K04、SM-K14、SM-K15、SM-K24、SM-K28（drawer）。
- **決定済みの形**:
  - disclosure group:
    - `ui::disclosure_group(selection: impl IntoSelection<K>)`（Single は `Signal<Option<K>>`、Multiple は `Signal<Vec<K>>`）と `.panel(key, title: impl IntoText, content: impl Fn() -> V + 'static)`。
    - `interaction::use_disclosure_group`。
    - `ui::disclosure(..).always_open()`（構造の option。soft の「無効なら常に開く」）。
    - basic の `collapsible(open).content(..).attach(trigger)` は `ui::disclosure(..).attach(trigger)`（trigger を caller が持つ形）。
  - hover card:
    - `ui::hover_card(content: impl Fn() -> V + 'static).attach(trigger)`。builder: `.placement(..)`、`.align(..)`、`.open(impl IntoBinding<bool>)`、`.open_delay(Duration)`、`.close_delay(Duration)`（basic の今の option）。
    - `use_hover_surface` の上に作る。状態は `enum HoverCardState { Closed, Opening, Open { by: OpenedBy }, Closing, Dismissed }`。
    - `InteractionPolicy` に `pub hover_card: HoverCardTiming`（`{ open_delay: Duration, close_delay: Duration }`、既定 600 ms・300 ms）を足す。soft の theme は 300 ms・200 ms。
  - drawer:
    - `ui::dialog(..).mode(DialogMode::Drawer(Edge))` に swipe で閉じる振る舞いを足す（`DragApi::track_velocity`）。
    - `InteractionPolicy` に `pub drawer: DrawerPolicy { swipe: DrawerSwipe }`、`pub struct DrawerSwipe { pub close_fraction: f64, pub flick_speed: f64, pub flick_distance: f64 }`（既定 0.3、0.6 px/ms、4.0 px。basic の `drawer.rs` の今の値）を足す。
    - basic の `drawer` は消える。
  - copy feedback: `interaction::use_copy_feedback(duration: Duration) -> CopyFeedbackApi`（`copy(text)`、`is_copied() -> Value<bool>`）。状態は `enum Copied { Idle, Copied }`、戻りは `Step::after`。
- **APG**: Accordion。
- **試験**: 足す。`disclosure_group_single_keeps_one_item_open`、`hover_card_opens_after_the_delay_and_on_focus`、`drawer_closes_on_swipe`、`copy_feedback_resets_after_its_duration`。APG の違いごとに `apg_accordion.rs`。
- **unit の確認**: 共通の確認（basic、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。

### K-2b combobox の family と select の sheet

- **custom chooser の補完**: `ui::selection_field(label:IntoText,value_label:IntoText)` は content(Fn()->V)、presentation(SelectPresentation)、open(binding)、variant を持つ。既存 popup/sheet 境界を使い、ダミーの選択 key や別 selection machine を作らず、caller の検索 binding を保持する。

- **実装時の契約補完**: 公開 ComboboxMode::{Inline,Popup} は unit variant、open(binding) は private presentation descriptor に格納。query は Query::{Local(String),Shared(Binding<String>)} の単一 owner。SelectPresentation::{Popover,BottomSheet,Adaptive}。chips/exclude_selected は structural option。

- **目的**: 付録 C.2 の combobox 系、select の touch の sheet。設計書 17.18 節の C-35。付録 A の SM-K16、SM-K17、SM-K919。
- **決定済みの形**:
  - `ui::combobox(label: impl IntoText, options: impl IntoValue<Vec<ListItem<K>>>, selection: impl IntoSelection<K>)`。builder: `.query(impl IntoBinding<String>)`、`.free_text()`、`.mode(ComboboxMode)`、`.chips()`（複数の選択を chip で見せる）。
  - 状態は `ComboboxState { mode: ComboMode, query, highlight }`。sheet は I-2a の `Surface::{Popover, Sheet}`。
  - C-35: soft の `typeahead.rs`・`selector.rs`・`multi_selector`・`tokenizer`・`calendar.rs` が中核の hook と違う振る舞い（key の操作、範囲の anchor、focus の規則）を、`use_combobox`・`use_select`・`use_calendar_grid` の option にする。名前は soft の今の option の名前。soft の engine を消せるだけの option を足し、それ以外は足さない（足した option の一覧を報告に書く）。
  - soft の `Selector` は `ui::select`（soft の recipe）。`complex_selector`・`power_search` は soft に残し、中は `use_combobox`。creative の `picker.rs` はこの unit で触らない（creative に残す。KL-creative が状態を直した）。
  - SM-K919: `LabelCache<K>` に保持の規則を閉じる。`labels` は source と cache からの memo。
- **APG**: Combobox（中核の部品の追加の分）。
- **試験**: 足す。`combobox_filters_options_and_selects_many_as_chips`、`select_touch_opens_the_sheet`。APG の違いごとに `apg_combobox.rs`。
- **unit の確認**: 共通の確認（basic、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。

### K-3a toolbar・toggle group・button group

- **実装時の契約補完**: ButtonGroupMember は Button から変換でき、solid は IntoValue<bool>、keep_size は構造 option。toggle item recipe は ToggleGroupItemState { toggle, layout, first, last } を受け、列挙から位置を派生させる。

- **目的**: 付録 C.2 の toolbar、toggle group と segmented、button group。付録 A の SM-K10（`button_group.rs`・`tab_list.rs` の部分）、SM-K25、SM-K27。
- **決定済みの形**:
  - `ui::toolbar(label: impl IntoText)`、`.start(impl IntoChildren)`、`.center(..)`、`.end(..)`。roving は中核の `roving_focus`。game の `toolbar(children)` は `.start(children)`。
  - `ui::toggle_group(label, options: impl IntoValue<Vec<ListItem<K>>>, selection: impl IntoSelection<K>)`。segmented は `ui::radio_group(..).variant(RadioGroupVariant::Segmented)`（K-0）。
  - `ListItem` に pressed のときの icon の option を足す（soft の `pressed_icon`）。soft の `ToggleSelection` と basic の `Selection` は中核の `Selection` に。
  - `ui::button_group(label, children: impl IntoChildren)`（中に roving を持つ）。
  - soft の `tab_list.rs` は `use_tabs` と core の `NodeBuilder::roving_focus`。
  - SM-K25: soft の toolbar の key の hint は soft の部品として残す（`enum KeyHint { Hidden, Showing { anchor: NodeToken }, Seen }`、消すのは `Step::after`）。
  - SM-K27: soft の segmented の固有の `disabled` を消し、`NodeBuilder::disabled(impl IntoValue<bool>)`。
- **APG**: Toolbar。
- **試験**: 足す。`toolbar_moves_focus_with_arrows_as_one_tab_stop`、`toggle_group_single_and_multiple_selection`、`button_group_is_one_tab_stop`。APG の違いごとに `apg_toolbar.rs`。
- **unit の確認**: 共通の確認（basic、pop、game、creative、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。

### K-3b pagination・stepper・breadcrumb・navigation menu

- **和集合の補完**: PaginationMode::{Pages,Input,Count,Compact,Dots,None}。PageItem::{First,Previous,Page(usize),Ellipsis,Next,Last}。page binding は0始まり、入力文字列は1始まり。PageCount::{Pages,Items} が count/total_items の authority を選ぶ。NavOpen 遷移が on_open_change を通知する。StepperState.collapsed は rect/count/min_step_width から派生し、StepperTokens に閾値と connector_gap を持つ。

- **目的**: 付録 C.2 の pagination、stepper、breadcrumb、navigation menu。付録 A の SM-K12、SM-K13、SM-K922、SM-K923。
- **決定済みの形**:
  - `ui::pagination(count: impl IntoValue<usize>, page: impl IntoBinding<usize>)`、`.siblings(impl IntoValue<usize>)`、`.item(impl Fn(PageItem) -> V + 'static)`（basic の静的な part の組み立て）。
    - SM-K922: `enum PageDraft { Following(u32), Editing(String) }`。
  - `ui::stepper(items: impl IntoValue<Vec<StepperItem>>, current: impl IntoValue<usize>)`、`StepperItem::new(label).description(..).status(StepStatus)`。
  - `ui::breadcrumb(items: impl IntoValue<Vec<ListItem<K>>>).on_activate(impl Fn(K))`。末尾は `CurrentItem::Page`。
  - `ui::navigation_menu(label)`、`.link(key, label, content: impl Fn() -> V + 'static)`。`interaction::use_navigation_menu`（状態は basic の `NavOpen::{Closed, Hover(usize), Pinned(usize)}`。閉じる effect は遷移へ）。
    - SM-K13: soft の top nav・side nav の flyout も、これを使う。
    - SM-K923: soft の nav heading も、これを使う。
- **APG**: Breadcrumb。
- **試験**: 足す。`pagination_moves_pages_and_keeps_focus`、`stepper_marks_the_current_step`、`breadcrumb_marks_the_current_page`、`navigation_menu_opens_on_hover_and_pins_on_click`。APG の違いごとに `apg_breadcrumb.rs`。
- **unit の確認**: 共通の確認（basic、pop、game、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。

### K-4 collection と creative と game

- **semantics の補完**: TableCell.col_span は正準 Semantics.column_span と Node.column_span、native AccessKit、Element.column_span まで接続する。Tree.header/reveal と marker recipe を保持する。ItemRow の title_content/description_content/header/footer は view slot。

- **目的**: 付録 C.2 の tree の採用、static table、item list。付録 A の SM-K11、SM-K33（frame の loop と focus の復元）、SM-K917。
- **決定済みの形**:
  - tree:
    - game の `tree` と soft の `tree_list` を消し、`ui::tree` と kit の recipe にする。
    - creative の outliner は、列・可視・lock の toggle を `ui::tree` の slot と `use_tree` の上の creative の部品として残す（SM-K11）。
    - SM-K917: `enum OutlinerReveal { Idle, AwaitingRow { target: u64 } }`。
  - `ui::table(..)`、`table_header`、`table_body`、`table_footer`、`table_row`、`table_head`、`table_cell`（揃えは `style::TextAlign`）。game の `TableCell` は `GameCellVariant`。
  - `ui::item_list(children: impl IntoChildren)`、`ui::item_row(title)`（`.media(..)`、`.description(..)`、`.actions(..)`、`.on_press(..)`）。semantics は `Role::List`・`ListItem`。
  - SM-K33: 見えている間だけの loop は `FrameLoop::while_present(node_ref)`（K-0）。game の作り直しのときの focus の復元は `ui::pagination` の中の roving が持つ。
  - creative の編集の部品の zoom・pan の model は `use_zoom_pan` の `ViewTransform` 一つにする。
- **APG**: Tree View（`ui::tree` の役割と状態の分。I-7 で ui の tree の file が空いたため）。
- **試験**: 足す。`static_table_exposes_rows_and_cells`、`item_list_exposes_list_semantics`。APG の違いごとに `apg_tree_view.rs`。
- **unit の確認**: 共通の確認（basic、pop、game、creative、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。KL-creative の file には触らない。

### K-5 soft に残る部品の状態

- **目的**: 付録 C.3。付録 A の SM-K05、SM-K06、SM-K07、SM-K28（bottom sheet）。
- **決定済みの形**:
  - SM-K05: lightbox は soft に残し、`use_dialog` と `use_zoom_pan` の上に作り直す。`enum LightboxState { Closed, Open { index: usize, view: LightboxView } }`、`enum LightboxView { Fit, Zoomed { pan: Point } }`。画像の失敗は `NodeRef::image_status()`。
  - SM-K06: `use_resizable` と `resize_handle` を消し、`ui::splitter` の `collapsible(..)`・`snaps(..)`（I-5b）にする。
  - SM-K07: `use_scroll_state`・`use_scroll_overflow`・`use_overflow` を消し、`NodeRef::scroll_state()`（K-0）にする。overflow の fit は soft に残す（`enum OverflowFit { Measuring, Fitted { visible: usize } }`）。key の scroll は core の既定の key の操作を使う。
  - SM-K28（bottom sheet）: `enum Sheet { Resting { snap: usize }, Dragging { from: usize, offset: f64 }, Settling { to: usize } }`。`DragApi::track_velocity` と `nearest_snap`・`pull_to_snap` を使う。
- **手順**: soft の `lightbox.rs`、`resizable.rs`、`scrollable_area.rs`、`soft_overflow.rs`、`bottom_sheet.rs`。
- **試験**: 足さない（soft の試験が回帰の網）。
- **unit の確認**: 共通の確認（soft）。
- **規模**: M。**並列**: K-0 の後、ほかの K と並べてよい。

### K-6a badge・avatar・skeleton・spinner・kbd

- **見た目の補完**: Avatar.fallback_content(IntoViewFn) は画像なし・最終画像失敗の両経路を共通化する。badge_label recipe で文字だけの transform 等を指定する。Creative KeyGlyph は keyframe 描画の domain enum であり、kbd へ移さない。

- **実装時の契約補完**: kbd_combo(Vec<Key>) に modifiers(IntoValue<Modifiers>) を追加し、既存 Key にない modifier を正準 Modifiers で表す。meta の記号は Capabilities.command_key に従う。

- **目的**: 付録 C.2 の Tier 2 の一部。付録 A の SM-K32、SM-K10（`avatar_group.rs` の部分）、SM-K916。
- **決定済みの形**:
  - `ui::badge(label: impl IntoText)`、`.tone(impl IntoValue<Tone>)`、`.variant(impl Variant)`（kit の `BadgeVariant` は各 kit の `impl Variant`。pop の `Dot`・`Count`・`Hue(Hue)` を含む）。
  - avatar:
    - `ui::avatar(name: impl IntoText)`、`.src(impl IntoValue<Option<Asset>>)`、`.size(impl IntoValue<ControlSize>)`、`.status(impl IntoValue<Option<Tone>>)`。`ui::avatar_group(children: impl IntoChildren)`、`.max(impl IntoValue<usize>)`。
    - 画像の失敗は `NodeRef::image_status()`（SM-K32）。
    - SM-K916: `enum AvatarStatusSource { None, Mounted { label: Value<Option<String>> } }`。
  - `ui::skeleton()`、`.lines(impl IntoValue<usize>)`、`.shape(impl IntoValue<SkeletonShape>)`。reduced motion では止める。
  - `ui::spinner()`、`.size(impl IntoValue<ControlSize>)`。名前は message の key `spinner.loading`。動きは `Animated` 一つ。G-5 の既定の loading の表示をこれに差し替える。
  - `ui::kbd(keys: impl IntoText)`、`ui::kbd_combo(keys: impl IntoValue<Vec<Key>>)`。OS の記号は `Capabilities.command_key`。soft の `kbd.rs` の `cfg!(target_os = "macos")` の分岐を消す。
- **試験**: 足す。`avatar_falls_back_to_initials_when_the_image_fails`、`skeleton_stops_with_reduced_motion`、`kbd_uses_the_platform_command_key`。
- **unit の確認**: 共通の確認（basic、pop、game、creative、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。

### K-6b empty state・alert・card・input group

- **和集合の補完**: Alert content/collapsible/open/dismissible/dismiss_label、EmptyState.heading_level、input_group_with を保持。InputGroupMemberState は列挙位置・方向・実際の focus_within を派生させる。色・密度・elevation は各 kit の Variant と recipe。

- **目的**: 付録 C.2 の Tier 2 の残り。付録 A の SM-K22、SM-K23。
- **決定済みの形**:
  - empty state:
    - `ui::empty_state(title)`、`.description(..)`、`.icon(impl IntoView)`、`.actions(impl IntoView)`、`.announce()`。
    - 一度だけ `announcer` で読み上げる（`enum Announcement { Pending, Spoken }`）。
  - alert:
    - `ui::alert(title)`、`.description(..)`、`.tone(..)`、`.icon(..)`、`.on_dismiss(..)`、`.variant(..)`（banner は `AlertVariant::Banner`）。
    - 閉じた後の focus の復元は core の `restore_focus`（SM-K22: `enum BannerState { Shown, Dismissed }`）。
    - G-5 の既定の失敗の表示をこれに差し替える。
  - card: `ui::card(children: impl IntoView)`、`.header(impl IntoView)`、`.media(impl IntoView)`、`.footer(impl IntoView)`、`.on_press(..)`、`.selected(impl IntoBinding<bool>)`。
  - input group: `ui::input_group(label, children: impl IntoChildren)`、`ui::input_addon(text)`（basic と soft の `input_group_text` は `input_addon`）。
- **APG**: Alert。
- **試験**: 足す。`empty_state_announces_once`、`alert_dismiss_restores_focus`、`card_on_press_activates_with_the_keyboard`。APG の違いごとに `apg_alert.rs`。
- **unit の確認**: 共通の確認（basic、pop、game、creative、soft）。
- **規模**: L。**並列**: K-0 の後、ほかの K と並べてよい。

### K-8 `viewport` を中核へ

- **実装時の契約補完**: ViewTransform は types 正準で interaction が再公開。ZoomPanApi は hidden internal-contract の ViewportTransform port を実装し、viewport.transform(api) に既存の単一 model を接続する。CPU snapshot は context-local Purpose::{Window,Cpu} と DrawCmd::External.cpu_pixels:Option<Pixels> の immutable fallback で処理し、GPU registry や global raster flag を追加しない。

- **目的**: 設計書 17.13 節。付録 A の SM-K918。
- **決定済みの形**:
  - `kivact::view::viewport(pixels: impl IntoValue<Pixels>) -> Viewport`。builder: `.fit(impl IntoValue<ImageFit>)`、`.gpu_frame(..)`（facade の feature `gpu-interop`）、`.transform(ZoomPanApi)`。
  - 座標の変換は `ZoomPanApi::to_content(Point)`・`to_view(Point)`（`ViewportApi` は作らない。付録 B.1）。
  - pointer の capture は G-2。重ね描きは `draw_over`。支援技術の領域は canvas の region。
  - `NodeRef::snapshot` で、viewport は自分の `Pixels` を CPU の代わりとして描く。
  - SM-K918: 最後に報告した大きさは `Sent<SurfaceSize>`。大きさは rect と scale の memo。
- **手順**:
  1. creative の `viewport_surface.rs` を `kivact-core` へ移し、creative の呼び出し元と catalog を直す。
  2. creative の feature `gpu-interop` と、その `cfg` を消す。
  3. creative の `gpu_interop` の試験を public-api へ移す。
- **試験**: 足す。`viewport.rs`（新しい file）に `viewport_maps_points_between_view_and_content`、`viewport_shows_pixels_with_fit`、`viewport_snapshot_draws_its_pixels`。
- **unit の確認**: 共通の確認（creative）と `bash dev/tools/check-features.sh`。
- **規模**: M。**並列**: G-1b・G-2・I-5b の後。ほかの K と並べてよい。

### K-9a chat と code block

- **公開契約の補完**: chat_message(author:IntoText) と sender(ChatSender) を分ける。line_numbers(IntoValue<bool>) に統一し alias は作らない。highlight は T-1 の content と状態付き行 callback と同じ署名。chat_thread.empty(predicate,content)、history_revision は外部 dataset の実 revision をそのまま use_follow_end に渡す。Pop の conversation-preview thread と Soft の音声 dictation は domain に残す。

- **目的**: 設計書 17.15 節の chat と code block。
- **決定済みの形**:
  - chat:
    - `ui::chat_thread(children)`（`Role::Log`、`use_follow_end`）。
    - `ui::chat_message(author)`（`.content(..)`、`.time(..)`、`.status(..)`、`.variant(..)`）。
    - `ui::typing_indicator()`。
    - `ui::chat_composer(value: impl IntoBinding<String>)`（`.on_submit(..)`）。
  - code block:
    - `ui::code_block(code: impl IntoValue<String>)`。builder: `.line_numbers(impl IntoValue<bool>)`、`.highlight(..)`（引数は T-1 の `line_styles` と同じ形）、`.wrap(impl IntoValue<bool>)`。copy の button は `use_copy_feedback`。
    - 言語ごとの構文の色の関数は kit に残す（basic の `rust_lines` は `highlight` に渡す関数になる）。
  - soft の `streaming_text`・`chat_tool_calls`・`chat_token` は soft に残す。
- **試験**: 足す。`chat_thread_is_a_log_and_follows_the_end`、`code_block_copies_and_numbers_lines`。
- **unit の確認**: 共通の確認（basic、pop、soft）。
- **規模**: L。**並列**: K-2a の後。ほかの K と並べてよい。

### K-9b progress の variant・stat・heading・description list・aspect ratio

- **公開契約の補完**: Trend::{Up,Down,Neutral}、delta(IntoText)、icon/chart/value_content/label_content は view slot。Description.term_content は term の accessible text を保ち可視内容だけ差し替える。独立 sparkline や caller Animated を合成し、stat に別の数値 owner を作らない。

- **目的**: 設計書 17.15 節の残りの行。
- **決定済みの形**:
  - progress: `ProgressVariant::{Bar, Ring, Gauge}`（標準の variant）。
  - stat: `ui::stat(label, value: impl IntoText)`（`.delta(..)`、`.trend(impl IntoValue<Trend>)`、`.icon(..)`）。
  - heading: `ui::heading(text: impl IntoText, level: u8)`（`Role::Heading` と `level`）。
  - key value: `ui::description_list(children)`、`ui::description(term: impl IntoText, detail: impl IntoView)`。creative の編集できる `property_row` は creative に残し、表示の部分を中核の部品で組む。
  - aspect ratio: kit の部品を消し、`Style::aspect_ratio` を使う（basic、soft）。
- **試験**: 足す。`progress_ring_reports_its_value`、`stat_exposes_label_and_value`、`heading_has_its_level`、`description_list_pairs_terms_and_details`。
- **unit の確認**: 共通の確認（basic、pop、game、creative、soft）。
- **規模**: M。**並列**: K-0 の後、ほかの K と並べてよい。

### K-7a〜K-7e kit の規約の残り（kit ごと: a basic、b pop、c game、d creative、e soft）

- **目的**: 付録 C.1 の表のその kit の列の残り全部、C.4 の改名、C.6。
- **手順**（その kit の列にある行だけ）:
  1. slot（KIT-5）、children の trait、variant（KIT-6）、固有の `disabled`、option の束、型の alias、message の key（KIT-8）、literal の色（KIT-2）、RTL（soft は `use_direction()`）、breakpoint、`provide((), ..)` の回避策、soft の残る公開 item の rustdoc を直す。
  2. C.4 の改名を行う。
  3. C.6 を行う: kit の試験を一つの binary に（soft の test file を `tests/consumer.rs` と `tests/consumer/<topic>.rs` に）、試験の helper を中核の API に（`Screen::evidence`、`RenderOptions::with_theme(..).reduced_motion()`、`Screen::assert_layout_clean()`）、catalog の font を `examples/shared-assets/fonts/` に一つずつ（`THIRD-PARTY-NOTICES.md` を直す）。
  4. pop と game の recipe の file の構成を揃える（file 名と並び。中身の token 化はしない）。
  5. その kit を `dev/xtask/policy/workspace.toml` の `name_collision`・`kit_literal_color`・`os_branch`・`api_shape` の `skip` から外し、出た違反を直す。
- **試験**: 足さない（試験の移動だけ）。
- **unit の確認**:
  - `cargo check -p <kit> -p <その catalog> --all-targets --all-features`
  - `KIVACT_TEST_MIRROR=off cargo test -p <kit> -p <その catalog>`
  - `cargo xtask check --no-write --crate <kit>`（`--only` は一つしか取れないので、crate の全検査を回す）
- **規模**: M（soft は L）。**並列**: 5 つを並べてよい。その kit の K の unit が全部 merge された後。

### H-G1〜H-G6 crate の組ごとの検査の有効化

- **目的**: 設計書 18.20 節（関数の長さ、後から埋める初期化、状態の検査群）。crate の組（0.10 節）ごとに一つの unit。
  - H-G1: engine
  - H-G2: arvane
  - H-G3: core
  - H-G4: ui（`kivact-interaction`、`kivact-ui`、`kivact`）
  - H-G5: kit（kit ごとに分けてよい）
  - H-G6: 外側（examples、catalog、templates、`dev/tests`、`kivact-docs`、xtask）
- **手順**:
  1. 組の library crate の `lib.rs`（binary は `main.rs`）に `#![deny(clippy::too_many_lines, clippy::disallowed_types)]` を足す（閾値は `clippy.toml` の 80）。
  2. 組の crate を `dev/xtask/policy/workspace.toml` の `state_fields`・`effect_writes` の `skip` から外す。H-G1〜H-G4 と H-G6 は `os_branch` の `skip` からも外す（kit の `os_branch` は K-7）。
  3. 出た違反を直す。
     - 長い関数は分ける（振る舞いを変えない）。
     - `OnceCell` などは構築の時に値を入れる形にする。
     - 状態の field の違反は付録 A の形か設計書 18.14 節の規則で直す。直し方が付録 A にない状態を見つけたら、`SM-<組>9xx` として付録 A に足してから直す。
  4. 例外は inline の目印 `// xtask-allow(<検査>): <理由>` か `#[expect(.., reason = "..")]` だけ。理由は「状態でないことの説明」か設計書 10.11 節の列挙に限る。
- **試験**: 足さない。
- **unit の確認**:
  - `cargo clippy -p <組の crate> --all-targets --all-features -- -D warnings`（この unit だけは clippy を回す）
  - `cargo xtask check --no-write --crate <crate>`（crate ごとに。`--only` は一つしか取れない）
  - `KIVACT_TEST_MIRROR=off cargo test -p <組の crate>`
- **規模**: 各 M（H-G3・H-G4・H-G5 は L になりうる。そのときは crate ごとに分ける）。
- **並列**:
  - H-G1・H-G2 は W6 の始めから K と並べてよい（K は engine と arvane に触らない）。
  - H-G3 は K-0 と K-8 の後。
  - H-G4 は K の中核の段が全部 merge された後。
  - H-G5 はその kit の K-7a〜K-7e の後。
  - H-G6 は H-G5 の後。

---

## 10.5 Phase X: 横断の整理

### X-1 feature の閉じ込め

- **目的**: 設計書 17.3 節（DEP-4）と 19（X-1 GPU の型）。以後の unit が `cfg(feature)` を二通りで保たなくてよいように、W2 の engine の unit の前に一度だけ行う。
- **決定済みの形**:
  - 最後の feature の表:

    | crate | feature |
    |---|---|
    | `arvane-render`（実装） | `gpu`、`gpu-interop`（今のまま） |
    | `arvane-native`（実装） | `embed`、`test` |
    | `arvane-virtual`（実装） | `mirror` |
    | `kivact-engine`（実装: 試験 host） | `testing` |
    | `kivact-core`（公開 API） | `embed = []`、`gpu-interop = []`、`test = ["kivact-engine/testing"]` |
    | `kivact`（facade） | `default = ["gpu"]`、`gpu = ["dep:arvane-render", "arvane-render/gpu"]`、`gpu-interop = ["gpu", "arvane-render/gpu-interop", "kivact-core/gpu-interop"]`、`embed = ["dep:arvane-native", "arvane-native/embed", "kivact-core/embed"]`、`test = ["kivact-core/test"]` |
    | `kivact-ui-creative` | `gpu-interop`（K-8 まで残す） |

    `arvane`・`kivact-engine` の `gpu`・`gpu-interop`・`embed`、`kivact-style` の `gpu-interop`、`kivact-core` の `gpu` を消す。facade の `arvane-render`・`arvane-native` は optional の直接の依存（feature を有効にするためだけ。code から使わない）。
  - 間の crate（`arvane`、`kivact-engine`、`kivact-style`、`kivact-core` の内部）は経路を常に compile する。実装がないときは実行時に `Unsupported` か「GPU なし」になる。
  - `arvane-render` は feature `gpu` がなくても `GpuContext`・`GpuSurface`・`GpuEnv` を同じ名前で公開する: `gpu` なしでは `pub enum GpuContext {}`・`pub enum GpuSurface {}`（値がない型。メソッドは `match *self {}`）、`GpuContext::new(..)` は `Err(RenderError::NoAdapter("built without the gpu feature".into()))`、`GpuEnv::from_env(..)` は値を読まない空の struct。`pub const GPU_BUILT: bool = cfg!(feature = "gpu");`（arvane の `Capabilities.gpu` は今の `cfg!(feature = "gpu")` の代わりにこれを読む）。外部の texture の import（`gpu-interop`）も同じく、feature なしでは `Unsupported` の `Failure` を返す関数を常に出す。
  - render の外部 texture import の既存 `Result<(), RenderError>` は維持し、未対応は `RenderError::External`。`arvane` の公開境界は既存の `FailureKind::Unavailable` と未対応の理由を返す（`FailureKind` に `Unsupported` は足さない）。
  - `arvane-native` の `embed` なし: `arvane_native::embed::open` は常にあり、`Unsupported` を返す。
  - core の `gpu-interop`・`embed` の feature は公開 API（`kivact::gpu_interop`、`view::viewport(..).gpu_frame(..)`、`App::embed`、`kivact::embed`）の `cfg` だけを持つ。core の中の engine を呼ぶ経路は常に compile する。
- **手順**:
  1. `arvane-render`: 値のない GPU の型と `GPU_BUILT`、import の関数。
  2. `arvane`: `cfg(feature = "gpu")`・`cfg(feature = "gpu-interop")`・`cfg(feature = "embed")` を全部消す。
  3. `kivact-engine`・`kivact-style`: `cfg(feature = "gpu-interop")`・`cfg(feature = "embed")` を消す。
  4. `kivact-core`: 公開 API の item の cfg だけを残し、内部の cfg を消す。`Cargo.toml` の feature を上の表に。
  5. facade の `Cargo.toml`。`arvane-native` の `embed` なしの `open`。
  6. `dev/tools/check-features.sh` の一覧を上の表にする。policy の `visible_to` の `arvane-render` と `arvane-native` に `kivact` を足す（facade の code から使わないことは review で確かめる）。
- **試験**: 足さない。
- **unit の確認**: build 可。`cargo check --workspace --all-targets --all-features`、`cargo check --workspace --all-targets --no-default-features`、`bash dev/tools/check-features.sh`。`grep -rn 'feature = "gpu-interop"' crates` が `arvane-render`・`kivact-gpu-share`・`kivact-core` の公開 API の item・`kivact`・`kivact-ui-creative` だけに出ること。
- **規模**: M。**並列**: E-1・A-1b・A-2・A-3・A-4・A-5 の merge を待つ。単独で行う（arvane・engine・core の多くの file に触る）。R-4〜R-6 とは同時でよい（runtime を触らない）。

---

## 11. Phase F: 仕上げ

wave W7。

### F-1 文書

- **目的**: 設計書 18.22 節、付録 B.7。
- **手順**（六つのサブエージェントに分ける）:
  1. **getting-started の en**: `docs/for-kivact-user/en/getting-started/` を付録 B.7 の書き方に書き直す。`coming-from-react-or-solid.md` の concept map は、付録 B.2 の prelude で import なしに書けることを前提にし、「`callback` は `kivact::view` から」などの注記を消し、「`Machine` は `useReducer` に effect と timer を足したもの」を足す。
  2. **getting-started の jp**: `docs/for-kivact-user/jp/getting-started/` を 1 と同じ内容で書き直す。
  3. **how-to の en**: `docs/for-kivact-user/en/how-to-guide/` を書き直す。
     - `custom-components.md`・`custom-controls.md` は `Node` の固有メソッド、`NodeBuilder`、`Part::new`、`Machine` で書く。
     - guide の「Rust の 5 つの詳細」に二つを足す: 整数を返す closure は `IntoValue<f64>` に渡せない（`as f64`）こと。prelude の名前を変数名にすると関数が隠れる（E0618。`text`、`row`、`list`、`field`、`Action`）こと。
  4. **how-to の jp**: 3 と同じ内容。
  5. **`docs/for-kivact-developer/architecture.md`（日本語、新規）と `decisions.md`（日本語、新規）**:
     - `architecture.md` の中身:
       - crate の一覧と境界の根拠（設計書 4.1・4.3 節）、module の向き（4.4 節）
       - 規則（3 章の B・VOC・S・API・DEP、API の 15 の規則、12 章の KIT）
       - 事実の持ち主の表（7.5 節）、`Machine` の書き方（18.13 節）、handler の語彙（10.4・18.2 節）、構造の option の考え方（10.6 節）
       - feature の表（17.3 節）、`Stage` の表と時間の仕組み（6.2・6.3 節）
       - 15 章と 18.23 節の表を「X を足すときに変える場所」の check list にしたもの
     - `decisions.md` は ADR の形（日付、問い、決定、理由）で書く: 16.3 節、17.9 節の APG より優先する決定、17.20 節、`refactoring/DECISIONS.md` の全部、設計書 19 章。
  6. **README・README_JP・`dev/README.md`・`CONTRIBUTING.md`**:
     - README・README_JP は crate の一覧と図。
     - `CONTRIBUTING.md`:
       - 試験の場所の表に `kivact-test` と `arvane-virtual`、新しい検査、`KIVACT_TEST_MIRROR` を足す（CI の記述は変えない）。
       - 「判断の基準」と「コード」の節から `architecture.md` を参照する。
       - 試験の節の「実ウィンドウの E2E」を「`kivact::test` による App 全体の E2E（`arvane-virtual` の上で、手元では実 window に mirror する）」と書き直す。
- `docs/for-kivact-ui-user/` の kit の page は K の各 unit が書き直してある。F-1 は、en と jp が同じ file 名で対になっていることと、中核へ上がった部品の名前が残っていないこと（`grep -rn "basic::calendar\|soft::selector" docs` などの、消えた kit の部品の名前）を確かめる。
- `docs/for-kivact-developer/strategy.md` は変えない。
- **試験**: 足さない。
- **unit の確認**: `cargo test -p kivact-docs`
- **規模**: L（六つの部分それぞれ M）。**並列**: 六つを並べてよい（file が重ならない）。

### F-2 検査の仕上げ

- **目的**: 設計書 18.20 節。
- **手順**:
  1. `dev/xtask/policy/workspace.toml` の全検査の `skip` が空であることを確かめる（空でなければ、残った crate の組の H-G に戻す）。
  2. clippy の `too_many_lines`・`disallowed_types` を workspace の lint（root の `Cargo.toml` の `[workspace.lints.clippy]` で `deny`）に移し、crate ごとの `#![deny(..)]` を消す。CP0 で入れた workspace の `allow` を消す。
  3. `dev/tools/check-features.sh` に `kivact/gpu`、`kivact/gpu-interop`、`kivact/test`、`kivact/tokio` と、新しい crate の feature を足す。
  4. `templates/basic/Cargo.toml` の「CI builds and tests it」の文を消す（CI は既定で無効。`.github/workflows/` は変えない）。雛形に root の `[profile.dev.package]` の engine 側の opt-level の設定を写し、雛形の README に理由（root の profile は workspace の外では効かない）を書く。
- **試験**: 足さない。
- **unit の確認**: `cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo xtask check --no-write`、`bash dev/tools/check-features.sh`。
- **規模**: S。**並列**: F-1 と並べてよい。

### F-3 最後の確認と PR の説明の下書き

- **手順**:
  1. G-final（12 章）を回す。
     - P-1 の全数調査（1〜3）をもう一度回し、残りが 0 であることを `PROGRESS.md` に書く。
     - 12 章の 8 の試しに、paint の性質を一つと標準部品を一つ足す試しを加え、変えた crate の数と file の数を記録する（試しの変更は捨てる）。
  2. `docs/for-kivact-developer/manual-verification.md`（なければ作る）に、作者が手で確かめる項目（IME、screen reader、embed の DAW、GPU の共有、mirror の窓）を、この再設計で変わった経路に絞って書く。
  3. PR の説明の下書きを `.claude/pr/foundation-v3.md`（git の管理の外）に書く。中身:
     - 31 の決定（設計書 1 章と 17 章）と 19 章の改訂
     - crate の変化（4 章）、公開 API の変化の要約（付録 B.3 と B.10 の要点）
     - 消えた暗黙の状態機械の数
     - 振る舞いを変えた点（`key_up` の宛先、`on_edit` の網羅、試験の経路の本番への一致、`blocking` の shutdown の後の error、presence による退場、APG の違いの表）
     - 行わなかったもの（設計書 17.20 節の backend と理由）
     - `DECISIONS.md` の要点、作者の手動の確認の一覧
     - 戦略 8 章「macOS 向けの分岐を入れない」が守られたこと、戦略 5 章の「アプリが描いた絵の表示」が中核の `viewport` になったこと
  4. `refactoring/`、`REFACTORING_DESIGN.md`、`REFACTORING_WORK_ORDER.md` を消す commit を作る。`docs/for-kivact-developer/decisions.md` と `architecture.md` は消さない。`SOFT_UI_WORK_ORDER.md` と `soft-ui-work-order/` は作者の指示がない限り残す。
  5. integration branch を push し、作者に G-final の結果と PR の説明の下書きの path を知らせて止まる。PR の作成と `main` への squash merge は作者が行う。
- **試験**: 足さない。
- **unit の確認**: G-final。
- **規模**: S。**並列**: F-1・F-2 の後。

---

## 12. 完了条件

全部を満たしたら完了とする（F-3 で確かめる）。

1. **重い gate**（0.6 節）と、mirror を有効にした全試験が integration branch で通る（Windows）。WSL で clippy と全試験が Wayland と X11 の両方で通る。
2. **crate の構成** が設計書 4.1 節の表（19 章で変えた所を含む）と一致する: `kivact-wire`・`kivact-reactive`・`kivact-task`・`kivact-motion`・`kivact-text`・`kivact-lower` がなく、`kivact-runtime`・`kivact-test`・`arvane-virtual` がある。`visible_to` が表のとおり。
3. **検査**: 設計書 18.20 節の全検査の `skip` が空で、違反が 0。clippy の `too_many_lines`・`disallowed_types` が workspace の lint で通る。例外は inline の目印と `#[expect(.., reason)]` だけ。
4. **状態機械**: P-1 の全数調査（`state-fields`・`effect-writes` の report と clippy の `disallowed_types`）をもう一度回し、付録 A の「解消しない」以外の項目と新しい違反が 0。
5. **公開 API** が付録 B.1 と一致する（facade の rustdoc の module と item の一覧を比べる）。prelude が付録 B.2 と一致する。
6. **kit**: 付録 C.2 の Tier 1・2・3 の部品が `kivact::ui` にあり、5 kit に重複の版がない。kit の規約（設計書 12 章）を満たす。kit の部品の公開の option が、中核へ上がった部品に全部対応している（K の各 unit の報告の対応表）。
7. **文書**: `docs/` と README の全部の code block が doctest で通る。英語と日本語が対になっている。
8. **戦略の遂行性**（設計書 15 章・18.23 節）: 次の五つについて、足すときに変える file を `docs/for-kivact-developer/architecture.md` の check list に従って列挙し、crate の数が設計書の表のとおりであることを `PROGRESS.md` に一行ずつ書く（code は書かない）: (a) 支援技術の性質 `key_shortcuts`、(b) text style の性質 `text_wrap_style`、(c) OS サービス `beep`、(d) paint の性質一つ、(e) 標準部品一つ。
9. **17 章と 19 章の決定** の各 unit の手順が全部済んでいる（`PROGRESS.md` の unit の表が全部 `merged`）。

---

## 13. unit の一覧

完了した W0・W1 の unit は省く（`PROGRESS.md` にある）。規模は S（〜300 行）・M（〜1000 行）・L（それ以上。一つのサブエージェントで 1 時間を超えうる）。「振る舞い」は意図した振る舞いの変更があり E2E を足す unit。

| wave | unit | 規模 | 振る舞い |
|---|---|---|---|
| W2 | A-0 `Backend` の注入 | S | |
| W2 | A-1a `arvane-virtual` と一つの `Window` | L | 試験の経路が本番に一致 |
| W2 | A-1b engine の試験 host・core の driver・名前 | L | |
| W2 | A-2 サービスと capability | M | |
| W2 | A-3 native の runner と shell | M | |
| W2 | A-4 embed の lifecycle | M | |
| W2 | A-5 GPU の状態 | M | |
| W2 | A-6a arvane-native の port と key・cursor・提示 | L | 生の入力の E2E |
| W2 | A-6b IME と DnD の pipeline | M | 生の入力の E2E |
| W2 | E-1 `kivact-text` と `kivact-lower` を engine に入れる | M | |
| W2 | E-2 host の状態機械 | M | |
| W2 | E-3 pointer と key | M | `key_up` の宛先、canvas の region の hover |
| W2 | E-4 IME と編集 | M | |
| W2 | E-5a layout の持ち主と settle | M | |
| W2 | E-5b presence と状態の動き | S | |
| W2 | E-6 支援技術 | M | |
| W2 | E-7 paint | S | |
| W2 | E-8 engine の fragment と `Display::Contents` | M | あり |
| W2 | E-9a 出現と退場の大きさ | M | あり |
| W2 | E-9b presence の値 | S | あり |
| W2 | X-1 feature の閉じ込め | M | |
| W2 | R-1 `kivact-runtime` を作る | M | |
| W2 | R-2 host との境界と turn の状態 | M | |
| W2 | R-3 reactive の内部の状態 | M | |
| W2 | R-4 時間と動きの状態 | M | 同じ安定化の中で完了と再開が起きた animation は `on_complete` を呼ばない |
| W2 | R-5 非同期の状態 | M | queue の待機仕事が残れば完了通知中も Pending |
| W2 | R-6 `Machine` | S | |
| W3 | C-1 起動の経路と `kivact-test` | L | |
| W3 | C-2 i18n と style の純粋化 | M | |
| W3 | L-1 ICU4X による地域書式 | M | 全 locale の書式 |
| W3 | C-3a view・flow・木の状態と `HandlerResult` | L | |
| W3 | C-3b focus の要求・lifecycle・gpu-interop の状態 | M | |
| W3 | C-4 part の入力と事実の持ち主 | M | |
| W3 | C-5 core の木を side table にする | M | |
| W3/W5 | KL-basic・KL-game・KL-soft（W3）、KL-creative（I-3 後の W5）kit の中の状態 | S・S・M・L | |
| W4 | N-D1 module・facade・prelude・geometry | M | |
| W4 | N-D2 新しい trait と型の定義 | M | |
| W4 | N-F1〜N-F6 fan-out A（core、interaction、ui、basic・pop・game、creative、soft） | 各 L | |
| W4 | N-G1 例・catalog・雛形の追従 | M | |
| W4 | N-G2 文書と公開 API の E2E の追従 | L | |
| W4 | N-G3 `EditPhase` と `on_edit` の網羅 | M | あり |
| W4 | N-G4 semantics の patch | M | |
| W4 | N-G5 残りの E2E | S | |
| W4 | N-8 公開 API の検査を有効にする | M | |
| W5 | I-0 option と policy | M | |
| W5 | I-1 開閉の family | M | APG |
| W5 | I-2a combobox・select・palette | L | APG |
| W5 | I-2b menu・menubar | L | APG |
| W5 | I-3 編集・drag・文字編集・toggle | M | APG |
| W5 | I-4a list・roving・tree・tabs | L | APG |
| W5 | I-4b table・calendar | M | APG |
| W5 | I-5a scalar | M | APG |
| W5 | I-5b zoom・pan・splitter | M | APG |
| W5 | I-6 通知と追従 | M | |
| W5 | U-1 ui の touch の gesture | M | |
| W5 | U-2 ui の編集と入力の draft | M | APG |
| W5 | I-7 出現と退場の一本化 | M | あり |
| W5 | T-1 編集する node の制約の解除と公開の文字編集の API | M | あり |
| W5 | T-2 公開 API だけの text field の証明 | M | あり |
| W5 | G-1a Path | S | あり |
| W5 | G-1b snapshot | M | あり |
| W5 | G-2 明示の pointer の capture | S | あり |
| W5 | G-3 モバイルの境界 | M | あり |
| W5 | G-4a 文字色・下線の種類・visibility | M | あり |
| W5 | G-4b layout の補間と transition の retarget | L | あり |
| W5 | G-4c theme の hint・pointer の種類・scroll の snap と慣性 | M | あり |
| W5 | G-5 非同期の標準部品 | M | あり |
| W6 | K-0 kit のための中核の部品 | M | あり |
| W6 | K-1a calendar と日付の入力 | L | あり |
| W6 | K-1b carousel と dropzone | M | あり |
| W6 | K-2a disclosure group・hover card・drawer・copy feedback | L | あり |
| W6 | K-2b combobox の family と select の sheet | L | あり |
| W6 | K-3a toolbar・toggle group・button group | L | あり |
| W6 | K-3b pagination・stepper・breadcrumb・navigation menu | L | あり |
| W6 | K-4 collection と creative と game | L | あり |
| W6 | K-5 soft に残る部品の状態 | M | |
| W6 | K-6a badge・avatar・skeleton・spinner・kbd | L | あり |
| W6 | K-6b empty state・alert・card・input group | L | あり |
| W6 | K-8 `viewport` を中核へ | M | あり |
| W6 | K-9a chat と code block | L | あり |
| W6 | K-9b progress の variant・stat・heading・description list・aspect ratio | M | あり |
| W6 | K-7a〜K-7e kit の規約の残り（kit ごと） | 各 M（soft は L） | |
| W6 | H-G1〜H-G6 crate の組ごとの検査の有効化 | 各 M | |
| W7 | F-1 文書（六つ） | 各 M | |
| W7 | F-2 検査の仕上げ | S | |
| W7 | F-3 最後の確認と PR の説明の下書き | S | |

### 実装で確定した公開境界の補足
- I-3/KL: `DragApi::source_with(impl Fn() -> T)` は accepted press で一度だけ snapshot を取得する。`source(payload)` は定数データ用で同じ pipeline。`active_payload() -> Value<Option<T>>` は Pending/Active の投影で、このメソッドだけ `T: PartialEq`。editor は既存の zero slop を保持し、domain の click 意図だけを Machine に持つ（pointer/origin/数値 snapshot は Drag が所有）。
- KL: `custom::Generation` と `Generation::as_u64()` で numeric region ID allocator を組む。追加 counter/map/hash は不要。
- U-1: `SelectApi::open_sheet()`・`is_sheet_open() -> Value<bool>`・`trigger_with_sheet(impl Fn() -> bool)` を足す。最後の chooser は activation の単一処理内で Released outcome を消費し、Popup の default toggle と二重に実行しない。on_press の Handled は callback 境界で消えるため、これで抑止しない。
- T-1: `Canvas` は既存名 `Draw`。`StyledRange`・入力用 `TextDecoration` は style、`Decoration`・`DecorationKind`・`TextLayout` は custom。入力用 decoration は Color/TextLength、backend へ一度 resolve する。
