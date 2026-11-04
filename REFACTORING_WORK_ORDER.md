# 作業手順書: Kivact 基盤再設計（Foundation v3）

- 発行: 2026-11-04（Claude Code が作成）
- 実装担当: ChatGPT Codex（主エージェント = **オーケストレータ**）と、オーケストレータが起動する**サブエージェント**
- 設計: [`REFACTORING_DESIGN.md`](REFACTORING_DESIGN.md)（以下「設計書」）、付録 [`refactoring/A-state-machines.md`](refactoring/A-state-machines.md)（付録 A）、[`refactoring/B-public-api.md`](refactoring/B-public-api.md)（付録 B）、[`refactoring/C-kits.md`](refactoring/C-kits.md)（付録 C）
- 起点: `main` @ `a3536ef`

本書は **順序・作業の単位（unit）・各 unit の手順・検査・完了条件** を定める。**作業の速度を優先する**（作者の方針、2026-11-04）: サブエージェントは同時に 6 つまで使い、integration branch は最後に作者が `main` への PR を立てて squash merge するので、途中の commit と途中の integration branch で build が壊れていてよい。検査は wave の区切り（checkpoint）でまとめて回す（0.6 節）。**何をどの形にするか** は設計書が決める。本書と設計書が矛盾したら設計書に従い、本書を直す。

---

## 目次

0. [進め方の規則](#0-進め方の規則)
1. [全体の順序](#1-全体の順序)
2. [Phase P: 準備](#2-phase-p-準備)
3. [Phase V: 語彙](#3-phase-v-語彙)
4. [Phase A: Arvane](#4-phase-a-arvane)
5. [Phase E: エンジン](#5-phase-e-エンジン)
6. [Phase R: ランタイム](#6-phase-r-ランタイム)
7. [Phase C: 中核](#7-phase-c-中核)
8. [Phase N: 公開 API](#8-phase-n-公開-api)
9. [Phase I・U: 振る舞いと標準部品](#9-phase-iu-振る舞いと標準部品)
10. [Phase K: 公式 kit](#10-phase-k-公式-kit)
11. [Phase F: 仕上げ](#11-phase-f-仕上げ)
12. [完了条件](#12-完了条件)
13. [unit の一覧](#13-unit-の一覧)

---

## 0. 進め方の規則

### 0.1 役割

| 役割 | する | しない |
|---|---|---|
| オーケストレータ（Codex 主エージェント） | 本書の順に unit を進める。unit ごとに worktree を作り、サブエージェントに 0.4 節の形で指示する。差分を点検し、gate を回し、integration branch へ merge し、push する。設計書 0.3 節の M-1〜M-8（機械的な導出）の範囲で、サブエージェントへの指示を具体化する。`refactoring/PROGRESS.md` を更新する | 設計書の決定を変える。M-1〜M-8 の外の判断（公開の名前・公開の型と署名・振る舞い・crate と依存・新しい概念）をする。これらに当たる問いは作者（Kashiwade）に聞き、答えを決定記録に書いてから進める。PR を作る・`main` へ merge する・`main` へ push する（作者が行う） |
| サブエージェント | 指示された unit の手順を、指示された worktree の中だけで実行する。0.6 節の「unit の確認」だけを回して報告する（workspace の全体の gate は回さない） | 設計を決める（指示と code が食い違ったら、止めて報告する）。指示の範囲の外の file を変える。互換の wrapper・alias・旧 API を残す。push する |

設計の判断をサブエージェントに任せない。サブエージェントへの指示には、変える file、型と署名、data の形、振る舞いの規則、試験の名前と確かめる内容、直す文書を書き、選択肢を残さない。指示が code と合わないとサブエージェントが報告したら、オーケストレータが決めて返事をする。

### 0.2 守る規則

- `CONTRIBUTING.md` の全部（module の `//! Role:` 行、`expect("invariant: ..")`、lint の `reason`、glob import の禁止、panic を隔離しない、など）。
- **互換を残さない。** 旧い名前・旧い経路・二重の mode・「呼び出し元が壊れないように」の既定の実装を作らない。作業の途中の一時的な二重の経路（移行のための旧 API の併存）も作らない。途中で build が壊れてよいので、旧い経路を残して build を保つ必要もない。workspace の中の呼び出し元（crate、例、雛形、kit、試験、文書）は、同じ unit か、0.10 節の追従の段（Phase N では N-9）で新しい形に移し、その wave の checkpoint までに旧い形を残さない。
- **試験は実 window の E2E だけ**（`kivact::test`）。過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden を足さない。例外は `dev/xtask` の検査の自分の試験（道具の試験）だけ。
- **状態機械の unit は振る舞いを変えない**（設計書 16.2 節）。振る舞いを意図して変える unit（本書で「振る舞いの変更」と書いたもの）だけが E2E を足す。
- window を開くコマンド（mirror を有効にした `cargo test`）は必ず `bash dev/tools/window-lock.sh [--timeout 秒] -- <command>` を通す。`KIVACT_TEST_MIRROR=off` の `cargo test`（A-1b の後）は window を開かないので lock を通さない。終了コード 124 は時間切れ。出力が空でも終了コードを確かめる。
- Linux の gate（WSL、distro `Ubuntu-24.04`）は checkpoint CP2 と最後の F-3 でだけ回す（0.6 節）。unit ごとには回さない。
- GUI の OS 入力の harness、実機の IME・screen reader・browser の手動の確認は作者が行う。エージェントは行わない。
- commit は conventional commits で目的ごとに分ける。公開 API を壊す commit は `!` を付ける（例: `refactor(core)!: ...`）。commit が build できる状態である必要はない（integration branch は最後に作者が squash merge する）。
- `vendor/xtask-core` は変えない（検査は `dev/xtask` に足す）。
- CI（`.github/workflows/`）は既定で無効にしてあり、変えない。CI に job・step・設定を足さない。

### 0.3 branch と worktree

- integration branch: `refactor/kashiwade/foundation-v3`（`main` から作る）。全 unit の merge 先。checkpoint を通したら origin へ push する（force push はしない）。全部が終わったら、オーケストレータは F-3 で PR の説明の下書きを書き、integration branch を push して作者に知らせ、止まる。**`main` への PR を作るのと squash merge は作者（人間）が行う。** エージェントは PR を作らず、`main` へ merge も push もしない。
- unit ごとに worktree を作る:
  ```text
  git worktree add .claude/worktrees/<unit-id> -b wt/<unit-id> refactor/kashiwade/foundation-v3
  ```
  サブエージェントにはその絶対 path を渡す。`wt/*` は push しない。merge したら worktree と branch を消す。
- **同時に動かすサブエージェントは最大 6 つ。** 1 章の wave の表のとおりに並べる。
- merge は `git merge wt/<unit-id>`（fast-forward でよい。最後に作者が squash するので merge の形は問わない）。unit が終わったらすぐ merge する（wave の終わりを待たない）。
- **生成物を unit で作らない:** サブエージェントは `architecture/` を書き換えない（`cargo xtask check` を write の mode で回さない。回すときは `--no-write`）。`architecture/` は checkpoint でオーケストレータが一度だけ作り直す。これで SVG の merge の衝突が起きない。
- **build の cache:** 各 worktree は自分の target dir を使う（既定）。`sccache` が入っていれば `RUSTC_WRAPPER=sccache` を全 worktree で使う。サブエージェントは `cargo build` ではなく `cargo check` を使う（試験を回す unit だけ `cargo test`）。

### 0.4 サブエージェントへの指示の型

次の形で書く（unit の節の「指示に含める」を埋める）。オーケストレータは、サブエージェントが読む設計書の節を必要な範囲に絞り、決定済みの形（型・署名）は指示に直接写す（サブエージェントが設計書を通して読まなくてよいように）。

```text
あなたは Kivact の実装担当です。設計はしません。指示と code が食い違ったら、作業を止めて報告してください。

- worktree: <絶対 path>（この中だけを変更する。branch は wt/<unit-id>）
- unit: <unit-id> <名前>
- 読む: REFACTORING_DESIGN.md の <節>、<付録の節>、REFACTORING_WORK_ORDER.md の <unit の節>
- 変更する crate・file: <列挙>
- 変更しない: <列挙（ほかの unit が同時に触る file）>
- 決定済みの形: <型・署名・data の形・規則。設計書から写すか参照する>
- 手順: <番号付き>
- 状態の点検表: 0.7 節（状態機械の unit のとき）
- 試験: <足す E2E の名前と確かめる内容。足さないなら「足さない」>
- 文書: <直す rustdoc・guide・README>
- 規則: CONTRIBUTING.md、互換を残さない、push しない、`architecture/` を書き換えない、読むのは「読む」に挙げた節だけ
- build の期待: <「build 可」（この unit の後で <crate の列> が check を通るはず）か「build 不可」（ほかの並列の unit が終わるまで通らない）>
- 完了の前に回す確認: 0.6 節の「unit の確認」（<crate の列>、<回す試験の名前>）
- 報告: 0.8 節の形
```

### 0.5 記録

オーケストレータは integration branch に次の二つの file を置き、checkpoint ごとに更新する（unit の merge ごとには更新しない）。

- `refactoring/PROGRESS.md`: unit の表（ID、状態 `todo`/`doing`/`merged`、merge の commit、gate の結果、時間、残った問題）。中断したら、ここから再開する。
- `refactoring/DECISIONS.md`: (1) 作者に聞いた問いと答え（日付、unit、問い、答え）、(2) M-1〜M-8 の導出のうち後の unit が知るべきもの（例: 状態機械の event の一覧）。付録 A にない暗黙の状態機械を見つけたら、設計書 13.2 節の「解消の型」の表の一行に当てはまるものはその形で解消して付録 A に項目を足し、ここに一行書く。当てはまらないもの（公開の型が変わる、振る舞いが変わる）は作者に聞く。

全 unit が終わったら、長く使う規則と根拠（crate の境界の表と根拠、S・API・VOC・DEP・KIT・B の規則、`Machine` の書き方、handler の語彙）を F-1 で `docs/for-kivact-developer/architecture.md` と `CONTRIBUTING.md` に移してから、`refactoring/` の 5 つの file（付録 A〜C、PROGRESS、DECISIONS）と設計書・本書を消す（F-3）。

### 0.6 確認と gate

**unit の確認**（サブエージェントが自分の worktree で、報告の前に回す。workspace の全体は回さない）:

```text
cargo fmt -p <crate> [-p <crate> ...]                       # 書き換える（--check ではない）
cargo check -p <crate> [-p <crate> ...] --all-targets --all-features   # 指示が「build 可」のときだけ
KIVACT_TEST_MIRROR=off cargo test -p <crate> <試験の名前>               # 試験を足した・回すと指示された unit だけ（A-1b の後）
cargo xtask check --no-write --only state-fields --crate <crate> --module <module path>   # 状態機械の unit だけ
```

- 指示が「build 不可」の unit（fan-out の並列の一つ、定義の段の unit など。0.10 節）は、`cargo fmt` だけを回して報告する。build の失敗は checkpoint で見る。
- A-1b より前に試験を足した unit は、試験を書くだけにして回さない（checkpoint で回す）。
- clippy は unit では回さない（checkpoint で回す）。

**checkpoint の gate**（オーケストレータが、wave の全 unit を merge した後に integration branch で回す。1 章の CP0〜CP8）:

```text
cargo fmt --all -- --check
cargo xtask check                      # architecture/ をここで作り直して commit する
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off cargo test --workspace --all-features          # A-1b の後。前は window-lock.sh を通す
KIVACT_TEST_MIRROR=off cargo test --workspace --no-default-features
bash dev/tools/check-features.sh
KIVACT_TRACE=stage,semantics,host_events KIVACT_TEST_MIRROR=off cargo test --workspace --all-features   # 前の checkpoint の trace と比べる
```

- **trace の比較:** 振る舞いを変えない unit を含む wave では、前の checkpoint の trace（`target/kivact-trace/`、commit しない）と今の trace を比べる。差は、その wave の unit が意図した差（13 章の表の「振る舞いの変更」の列、名前の変更）だけであること。意図しない差があれば、差の出た試験の名前から原因の unit を特定する。
  - GPU など実機の非同期完了では空の poll turn の回数が揺れる。event・semantics・write・rerun・op がなく、`next_wake_ns: None`・`exit: false` の turn だけは、回数と file 名を記録して比較から外し、それに伴う turn 番号の差を正規化してよい。raw log は保存し、残る payload と順序は厳密に比較する（CP0 で確認した実機の揺らぎ。データを変更する turn は除外しない）。
- **失敗したとき:** オーケストレータは失敗を crate ごとにまとめ、0.10 節の crate の組ごとに直す unit（`fix/<cp>-<組>`）を作って、最大 6 つ並べて直させる。全部が通るまで次の wave に進まない。
- **mirror と WSL:** CP2（Phase A・R・E の後）と F-3 では、上に加えて mirror を有効にした試験（`bash dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --all-features`）と、WSL の gate（Wayland と X11 を分けて記録）を回す:

```text
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target cargo clippy --workspace --all-targets --all-features -- -D warnings'
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target WAYLAND_DISPLAY=wayland-0 cargo test --workspace --all-features'
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target WAYLAND_DISPLAY= DISPLAY=:0 cargo test --workspace --all-features'
```

（WSL の distro を聞かれたら `KIVACT_WSL_DISTRO=Ubuntu-24.04`。）

checkpoint を通したら push し、`PROGRESS.md` を更新する。本書の unit の節の「完了条件」に書いた G-unit・G-phase は、それぞれ「unit の確認」と「その unit を含む wave の checkpoint の gate」と読む。

**G-final**: 12 章（F-3 で回す）。

### 0.7 状態の点検表（状態機械を扱う unit の完了の前に、サブエージェントが確かめて報告に書く）

変えた型・module のそれぞれについて:

1. 状態の field に `bool`・`Cell<bool>`・`AtomicBool`・`Signal<bool>`・`Var<bool>` が残っていないか（option の束を除く。S-2）。
2. `Some`/`None` が別の field に依存する `Option` が残っていないか（S-3）。
3. 同じ事実を二つの場所に持っていないか。派生できる値を保存していないか（S-4）。
4. effect が別の signal を後から直していないか（S-5）。
5. 「X の前に Y」の呼び出し順の要求が doc に残っていないか（S-6）。
6. 後から一度だけ埋める field が残っていないか（S-7）。
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
指示との違い・見つけたこと: <なし / 一覧（付録 A にない状態機械、設計書と code の食い違い、ほかの unit に影響すること）>
```

### 0.9 crate の作成・統合・削除のときに一緒に直すもの

crate を作る・統合する・消す unit は、次を全部直す（大きな unit は 0.10 節の追従の段に分けてよい。その wave の checkpoint までに揃える）。

- root の `Cargo.toml` の `members` と、`[profile.dev.package.<name>]`（engine 側の crate は opt-level 3 を付け直す。`kivact-runtime` は付けない: debug のまま）
- `dev/xtask/policy/workspace.toml` の `visible_to`、`module_rules`、`api_packages`、`options_structs`、`[[style.exceptions]]` の path
- `dev/tools/check-features.sh` の feature の一覧
- 各 crate の `Cargo.toml` の依存と feature の伝播（DEP-1: feature の名前を全層で揃える）
- `THIRD-PARTY-NOTICES.md`（依存が移る場合）
- README・README_JP・`docs/` の crate 名の言及
- `architecture/`（unit では直さない。checkpoint でオーケストレータが `cargo xtask check` で作り直す）

### 0.10 速く進めるための分け方

**fan-out（定義の段と追従の段）:** 多くの crate にまたがる変更（型の移動、改名、公開 API の形の変更）は、一つの unit を二段に分ける。

1. **定義の段**（サブエージェント一つ、または定義が別々の file にあれば複数）: 型・trait・関数の定義を変える。呼び出し元は直さない（build 不可でよい）。
2. **追従の段**（下の crate の組ごとに一つ、最大 6 つを並べる）: それぞれの組の中の呼び出し元を、新しい定義に合わせて直す。並べている間は互いの結果を待たない（build 不可でよい）。全部を merge したら、オーケストレータが `cargo check --workspace --all-targets --all-features` を回し、残った error を組ごとの fix の unit に配る。

**crate の組**（追従の段と、checkpoint の fix の分け方）:

| 組 | crate |
|---|---|
| G1 engine | `kivact-engine`、`kivact-layout`、`kivact-lower`（`kivact-text` は統合まで） |
| G2 arvane | `arvane`、`arvane-shell`、`arvane-native`、`arvane-render`、`arvane-text`、`arvane-asset`、`arvane-virtual`、`kivact-gpu-share`、`gpu-interop-producer` |
| G3 core | `kivact-types`、`kivact-runtime`（統合までは reactive・task・motion）、`kivact-style`、`kivact-i18n`、`kivact-editing`、`kivact-core`、`kivact-test` |
| G4 ui | `kivact-interaction`、`kivact-ui`、`kivact`（facade） |
| G5 kit | 5 つの kit（kit ごとに分けてもよい: 一つの kit を一つのサブエージェントに） |
| G6 外側 | `examples/`、catalog、`templates/`、`dev/tests/`（public-api、docs）、`docs/`、README |

**公開の署名を固定して並べる:** 同じ crate の中の別々の module を並べるとき（E-2〜E-7、I-1〜I-6 など）、各 unit は、ほかの unit が使う公開の型・メソッドの署名を変えない（変えるのは中の状態の持ち方だけ）。署名を変える必要が分かったら、作業を止めて報告する（オーケストレータが順を決め直す）。

**呼び出し元の追従をまとめる:** Phase N（公開 API）では、各 unit は中核の crate（G3・G4）だけを直し、kit・例・文書・試験（G5・G6）の追従は Phase N の最後の一度の追従の段（N-9）でまとめて行う（kit と文書を 8 回書き直さない）。

**file の持ち主:** 並べる unit の指示には「変更しない」の欄に、同時に動くほかの unit の file を書く。共有の file（各 crate の `lib.rs` の `mod`・`pub use`、`dev/xtask/policy/workspace.toml`、`Cargo.toml` の `members`）は足すだけにし、merge の衝突はオーケストレータが手で合わせる。

---

## 1. 全体の順序（wave）

同時に動かすサブエージェントは最大 6 つ。wave の中の `{..}` は並べてよい組（6 つまで）。`→` は前の unit の merge を待つ。wave の終わりの CP が checkpoint の gate（0.6 節）。

| wave | 内容（順と並列） | 並列の数 | 終わり |
|---|---|---|---|
| W0 準備 | `{P-0, P-2}` → P-1 | 2 | CP0（出発点の gate は P-0 の前に一度、2 章） |
| W1 語彙 | V-1a → V-1b（定義の段） → V-1b（追従の段 G1〜G6） → `{V-2, V-3, V-4, V-5, V-6}`（定義の段。`kivact-types` の別々の module） → V-2〜V-6 の追従の段（G1〜G6 を一度にまとめて） | 6 | CP1 |
| W2 下層と runtime | `{R-1, A-0}` → `{A-1a, R-2}` → `{A-1b, R-3, R-4, R-5}` → `{A-2, R-6}` → `{A-3, A-4, A-5, R-7}` → A-6 → E-1 → X-1 → `{E-2, E-3, E-4, E-5, E-6, E-7}` → E-8 → E-9 | 6 | CP2（mirror と WSL も） |
| W3 中核 | C-1 → `{C-2, C-3}` → `{C-4, L-1}` → C-5 | 2 | CP3 |
| W4 公開 API | N-1（定義の段） → `{N-2, N-6}` → N-3 → `{N-4, N-5}` → N-7 → N-9（kit・例・文書・試験の追従の段。G5 を kit ごとに 5 つ ＋ G6 の 1 つ） → N-8 | 6 | CP4 |
| W5 振る舞い | I-0 → `{I-1, I-2, I-3, I-4, I-5, I-6}`（公開の署名を固定して並べる） → `{U-1, U-2, I-7}` → I-8 | 6 | CP5 |
| W6 拡張口と不足 | T-1 → `{T-2, G-1, G-2, G-3, G-4, G-5}` | 6 | CP6 |
| W7 kit | K-0 → `{K-1, K-2, K-3, K-4, K-6, K-8}` → K-9 → K-5 → K-7 | 6 | CP7 |
| W8 仕上げ | F-1（文書を 6 つに分ける: getting-started の en・jp、how-to の en、how-to の jp、kit の文書、`architecture.md`・`decisions.md`、README・CONTRIBUTING） → F-2 → F-3 | 6 | G-final |

- W2 の中の A の列と R の列は互いに待たない（触る crate が違う）。`{..}` の中に A と R が混ざっているのは、同じ時に動いてよいという意味で、並列の数の上限 6 を超えないように詰めたもの。
- I-1〜I-6 を並べてよいのは、各 behavior の `*Api` の公開のメソッドの署名を W5 の間変えないから（0.10 節）。I-5 が I-3 の drag の `Pending`（slop）を使う所は、I-3 が `DragApi` に足すメソッドの署名（設計書 11.1 節の `slop`・`axis_lock`・`track_velocity`）が決まっているので、I-5 は I-3 の merge を待たずにその署名で書く（build 不可でよい）。
- L-1 は `kivact-i18n` の中だけなので、C-2 の後の空いている枠に入れる。
- 13 章の表の「依存」と「並列」の列は、この表が置き換える（13 章の表は unit の一覧と規模として読む）。

---

## 2. Phase P: 準備

### P-0 準備

- **目的**: integration branch、記録の file、検査の枠を用意し、出発点の gate の結果を記録する。
- **手順**:
  1. `git switch -c refactor/kashiwade/foundation-v3 main`、`git submodule update --init`。
  2. checkpoint の gate（0.6 節。この時点では window を開く試験は `window-lock.sh` を通す）を一度回し、結果（各コマンドの終了コード、試験の件数、時間）を `refactoring/PROGRESS.md` の「出発点」の節に書く。失敗があれば記録し、直さない（作者に知らせる）。ただし後の unit の gate を妨げる失敗（build が通らないなど）は、オーケストレータが最小の修正をして `DECISIONS.md` に書く。
  3. `refactoring/PROGRESS.md`（13 章の表を写して状態の列を足したもの）と `refactoring/DECISIONS.md`（空の表）を作る。
  4. 検査を設計書 18.20 節の表のとおりに用意する（14.1 節・17 章の検査の一覧は 18.20 節が置き換える）。`dev/xtask` に `state-fields`、`effect-writes`、`api-shape`（公開の field の `f32` を含む）、`canonical-path`、`name-collision`、`kit-literal-color`、`os-branch`、`structural-options` を実装し、各検査に `--report`（違反を error にせず一覧を出す）を付ける（P-1 が使う）。検査の対象は「全 crate から policy の `skip` の一覧を除いたもの」で、P-0 では全 crate を `skip` に入れる。例外は inline の目印 `// xtask-allow(<検査>): <理由>`（設計書 18.14 節）。各検査の module の rustdoc に規則と理由を書く。clippy の `too_many_lines`（`clippy.toml` に `too-many-lines-threshold = 80`）と `disallowed_types`（`OnceCell`、`OnceLock`、`LazyCell`、`LazyLock`）は、crate ごとに `lib.rs` の `#![deny(..)]` で有効にする（各 crate の最後の構造の unit が足す。F-2 で workspace の lint に移し、crate の `#![deny]` を消す）。
     - `dev/xtask/src/` に検査ごとの leaf module を足す（`state_fields.rs` など）。`syn` で crate の source を読む（既存の `module_rules.rs` と同じ読み方）。
     - policy に検査ごとの節を足す。
       ```toml
       [state_fields]
       skip = ["<P-0 では全 crate>"]
       role_suffixes = ["Options", "Event", "Phase", "Reason", "Input", "Report", "Tokens", "Recipes"]
       record_types = ["ListItem", "MenuEntry", "Command", "TreeNode", "DataColumn", "ToastOptions", "CalendarOptions", "StepperItem"]
       value_modules = ["kivact-engine::contract", "kivact-types::semantics", "kivact-types::env", "kivact-types::layout", "kivact-types::paint", "kivact-types::text", "kivact-types::scene", "kivact-types::render", "kivact-types::input", "kivact-types::platform", "kivact-types::embed", "kivact-types::check"]
       [effect_writes]
       skip = ["<P-0 では全 crate>"]
       [api_shape]
       skip = ["<P-0 では全 crate>"]
       [canonical_path]
       skip = ["<P-0 では全 crate>"]
       [name_collision]
       skip = ["<P-0 では全 crate>"]
       [kit_literal_color]
       skip = ["<P-0 では 5 kit>"]
       allowed_files = ["tokens.rs", "theme.rs", "color.rs"]
       [os_branch]
       skip = ["<P-0 では全 crate>"]
       allowed_paths = ["crates/arvane-native/", "crates/kivact-gpu-share/", "crates/arvane-render/src/gpu/", "crates/kivact-types/src/gpu_interop.rs", "crates/arvane-asset/", "crates/arvane-text/"]
       [structural_options]
       skip = ["<P-0 では全 crate>"]
       list = []   # N-4 が設計書 10.6 節の一覧を書く
       ```
       `NodeBuilder` を実装する型と `*Api` は、`state_fields` が型の実装を見て対象外にする。
     - 対象と例外は設計書 18.14・18.20 節のとおり（`skip` の一覧と inline の目印）。policy に data の record の一覧（`record_types`）と構造の option の一覧（`structural_options`）を置く。`disallowed_types` は設定だけでも既定の警告になるため、移行中は workspace lint を `allow` にし、各 crate の `#![deny]` で有効にする。F-2 で workspace の `deny` に置き換える。
     - CLI: `cargo xtask check --only <検査> [--crate <crate> [--module <module path>]]`（一つの検査を一つの crate と module の部分木だけに回す。`--crate` を付けたときは `skip` に入っている crate も検査する。`--only` がないときは今までどおり全部）。
     - 以下、`state-fields`・`effect-writes` を「状態の検査群」と呼ぶ。
     - 各検査の自分の試験を `dev/xtask` に足す（小さな source の文字列を与えて、違反を見つけること・見つけないことを確かめる。window を開かない道具の試験）。
  5. `CONTRIBUTING.md` の「merge 前の検査」の節に、新しい検査が `cargo xtask check` に含まれることを一文足す。
- **完了条件**: CP0 の gate が出発点と同じ結果。新しい検査は全 crate が `skip` に入っているので何も報告しない。xtask の試験が通る。
- **指示に含める**: 4 の検査の仕様は設計書 18.14・18.20 節と、14.1 節の表のうち 18.20 節が残した検査の行を写す。

### P-1 状態の全数調査

- **目的**: 設計書 17.8 節。付録 A を「全部」にする。
- **手順**:
  1. P-0 の状態の検査群（`state-fields`・`effect-writes`）と clippy の `disallowed_types` を、全 crate に `--report`（clippy は `-W`）で回す。
  2. 設計書 17.8 節の 1 の検索の一覧を全 crate（`crates/`、`examples/`、`templates/`）に回す。
  3. 1 と 2 の結果を一件ずつ見て、設計書 13.1 節の 8 種類に当たり、付録 A にないものを付録 A に足す（番号は接頭辞のあとに 9 から始める。例: SM-E901）。「解消後」は 13.2 節の表の型で書き、「unit」は付録 A の同じ crate・module の項目と同じ unit にする。同じ module の項目がなければ、13 章の表で「主な crate」がその crate の unit のうち、phase の順で最初の状態機械の unit にする。13.2 節の表に当たらないもの（公開の型が変わる、振る舞いが変わる）は作者に聞く。
  4. 当たらないと判断したもの（option の束、公開する値など）は、理由を `refactoring/DECISIONS.md` に一行ずつ書く。
- **完了条件**: 付録 A の追加の commit。`DECISIONS.md` の除外の一覧。追加の件数を `PROGRESS.md` に書く。
- **18 章の追加**: 状態の検査は `state-fields`・`effect-writes` の二つ（設計書 18.20 節）。18.14 節の規則（状態の field が二つ以上）で数える。

### P-2 比較用の trace

- **目的**: checkpoint の trace の比較の道具（0.6 節）。
- **手順**: 現在の `kivact::diagnostics` の `TurnRecord` を元に、環境変数 `KIVACT_TRACE`（値は `stage`・`semantics`・`host_events` を `,` で並べる）で、turn の段階、node ごとの semantics の変化、host との event を一行ずつ text で `target/kivact-trace/<試験の名前>.txt` に書く機能を `kivact-core` に足す（debug build だけ）。試験の名前ごとに file を分け、並べ替えずに書く。
- **完了条件**: `KIVACT_TRACE=stage,semantics,host_events` で public-api の E2E を回し、file ができる。unit の確認（core、public-api）。

---

## 3. Phase V: 語彙

この phase は公開 API の名前をほとんど変えない（`kivact::*` の path は facade の再公開で保つ）。中の crate の path が変わる。大きいが機械的な unit が多い。

### V-1a 名前の衝突を先に解く

- **目的**: V-1b の前に、`kivact-types` の `NodeId`（支援技術の node の id、`kivact-types/src/geometry.rs:7`）を `SemanticsId` と改名し、`TextRunId` を newtype にする（V-1b の「名前の衝突の解消」の箇条を先に行う）。wire の `NodeId` はそのまま。
- **完了条件**: unit の確認（types、engine、arvane 系、core）。

### V-1b `kivact-wire` を解体する

- **目的**: 設計書 5.1・5.2 節。`kivact-wire` の値を `kivact-types` へ、closure を含む契約を `kivact-engine::contract` へ移し、`kivact-wire` を消す。
- **決定済みの形**:
  - `kivact-types` に移す（型の名前はこの unit では変えない。ただし下の衝突だけは解く。module の配置は設計書 5.1 節の表のとおりにする: wire の module の名前を残さない）: wire の `check`、`drawing`、`grid`、`id_hash`、`ids`、`motion`、`paint`、`paint_vec`、`scene`、`semantics`、`style`、`text`、`values` の全部。`events.rs` のうち入力の値（`ButtonPress`、`FilePhase`、`Key`、`KeyCode`、`KeyPhase`、`Modifiers`、`MouseButton`、`PointerButtons`、`PointerEvent`、`PointerId`、`PointerKind`、`PointerPhase`、`ThemeHint`、`HostEnv`、`PlatformResult`、`ExternalImport`）。`node.rs` のうち値（`FileDialogRequest`、`PlatformRequest`、`OverlayAt`、`OverlayFit`、`OverlayProps`、`IntrinsicSize`、`EditLineLayout`（editing の型を含まないことを確かめる）、`ImageData`、`BoxData`、`TextData`）。`measure.rs` のうち値（`AvailableSpace`、`Baseline`、`LeafSize`、`MeasureRequest`）。（`Capabilities` は既に `kivact-types/src/platform.rs:154` にあり、wire は再公開しているだけ: `kivact-wire/src/host.rs:9`。再公開を消す）。
  - `kivact-engine/src/contract/` に移す: `node.rs` の残り（`NodeProperties`、`NodeKind`、`NodeType`、`EditingData`、`ImeSession`、`RovingFocus`、`SceneLayer`、`NodeSemantics`、`NodeInput`）、`events.rs` の `HostEvent`、`host.rs` の `HostInfo`・`TurnOutput`・`HostError`、`measure.rs` の `LeafMeasure`・`LeafTextMeasurer`・`TextShaper`。kivact-editing の型を field に持つ値は全部 contract へ。
  - `kivact-engine` の `bridge.rs`・`native.rs`・`layout.rs`（再公開だけの module）は消す。`contract` は値と trait だけを持ち、engine のほかの module に依存しない。core が使う handle（`Writer`・`Reader`・`Client`・`Boundary`・`HostWaker`・`Offscreen`（`OffscreenRenderer` と改名）・`TextMeasurer`・`WriteStats`）は定義の module に置いたまま、新しい再公開だけの module `kivact_engine::api` から使う（設計書 4.4・5.2 節）。contract の `ImeSession` は `ImeSessionProps` と改名する（core の公開の `ImeSession` と名前を分ける）。
  - **名前の衝突の解消（この unit で行う）**: `kivact-types` の既存の `NodeId`（支援技術の node の id、`kivact-types/src/geometry.rs:7`）を `SemanticsId` と改名する。wire の `TextRunId`（`kivact_types::NodeId` の alias、`kivact-wire/src/lib.rs:54`）は `kivact-types::ids::TextRunId` の newtype にする。wire の `NodeId` は `kivact_types::NodeId` になる。
  - **lower の `HostEvent` の使用**: `kivact-lower` は engine の contract に依存できない（循環）。lower が `HostEvent` を作っている asset の報告は、lower の型 `kivact_lower::AssetEvent`（variant は今の `HostEvent` の asset の variant の中身）の列を返す形にし、engine が `HostEvent` に包む（設計書 8.3 節の `LowerReport.asset_events` の先取り）。
  - `kivact-layout` の trait `LeafMeasure` を `LeafMeasurer` と改名する（contract の struct `LeafMeasure` と名前を分ける、設計書 5.2 節）。
  - **実コードに基づく訂正（2026-11-04）**: 上の値の一覧の `BoxData` は `measure: Option<LeafMeasure>` に closure を持つため `kivact-engine::contract` に移す。types から contract への依存は作らない。wire の grid の `Repeat` は `kivact_types::layout::GridRepeat` と改名し、motion の `Repeat` と区別する。旧い alias は残さず、追従の段で全呼び出し元を直す。
  - **共有の問い合わせ trait の訂正（2026-11-04）**: `TextShaper` は style の描画 recorder も使うため、上の contract の一覧から除き `kivact-types::text_shaping` へ移す（types root で再公開）。一つの trait を style・engine・core が使い、style から engine への依存や第二の trait は作らない。`LeafTextMeasurer` と `LeafMeasure` は contract に残す。
  - `kivact-core` は `kivact-types` に直接依存する（現在は wire の再公開で受け取っている）。
  - policy: `visible_to` から `kivact-wire` の行を消す。`kivact-types` の `visible_to` を外す（制限なし）。`kivact-engine` の `visible_to` は `["kivact-core"]` のまま。
- **手順**（一つの worktree で順に。途中で workspace が build できない状態があってよいのは、この unit の commit の中だけ）:
  1. `kivact-types` に module を作り、file を移す（`git mv` で履歴を残す）。module の `//! Role:` を書く。`kivact-types` の root の公開を足す。
  2. `kivact-engine/src/contract/` を作り、契約を移す。`bridge`・`native`・`layout` を統合する。
  3. 依存の順に直す: `kivact-editing` → `kivact-types` →（`kivact-layout`、`kivact-lower`、`kivact-style`、`kivact-motion`）→ `kivact-engine` → `kivact-core` → `kivact-interaction`・`kivact-ui` → 試験と例。`use kivact_wire::X` を `use kivact_types::X` か `use kivact_engine::contract::X` に置き換える。
  4. `crates/kivact-wire` を消す。0.9 節の一覧を直す（`[profile.dev.package.kivact-wire]` を消す）。
  5. `dev/xtask/policy/workspace.toml` の `[module_rules.kivact-types.uses]` に新しい module を足す（値の module は互いに参照してよい範囲を、現在の wire の内部の参照の向きから写す）。
- **試験**: 足さない。
- **完了条件**: unit の確認（`-p kivact-types -p kivact-engine -p kivact-core -p kivact-public-api-tests -p kivact-docs` と、直した全 crate の build）。`grep -r kivact_wire crates dev examples templates docs` が 0 件。
- **規模**: 大。機械的。0.10 節の fan-out で行う: 手順 1・2・4・5 を定義の段（サブエージェント一つ）、手順 3 の `use` の置き換えを crate の組 G1〜G6 ごとの追従の段（最大 6 つを並べる。手順 3 の依存の順は待たない）。

### V-2 semantics を一つの型にする

- **目的**: 設計書 5.3 節。`SemanticSources`（core）・wire の `SemanticsProperties`（`Option`）・protocol の `SemanticsProperties`（bool）の三つを `kivact_types::semantics::Semantics<R>` 一つにする。
- **決定済みの形**: 設計書 5.3 節の struct と、その「field の型の規則」。`derive(Clone, Debug, PartialEq, Hash)`（geometry を含む `SemanticsText` の既存比較を保つため `Eq` は付けない）、`Default` は手で書く、`f64` の field は `Finite`。`Finite::new` は符号付きゼロを正規化する。text snapshot の `Hash` は設計書 5.3 節の訂正に従う。`map_relations`。field の一覧を報告に書く。
- **手順**:
  1. `kivact-types` に `Semantics<R>` を作り、wire の `SemanticsProperties` と protocol の `SemanticsProperties` を消す。
  2. engine: node の semantics を `Semantics<FocusTarget>` で持つ（region への既存の関係を保持するため、当初の NodeId 指定を訂正）。`a11y/hash.rs` の field ごとの手書きの hash を、`Semantics<SemanticsId>` の `Hash` で置き換える。`a11y/build/*` は `map_relations` で関係を写す。
  3. `arvane-shell`（`a11y/validation.rs`、`a11y/state.rs`）と `arvane-native`（`a11y/conversion.rs`）を `Semantics<SemanticsId>` にする。
  4. core: `SemanticSources` は残し（N-3 で消す）、mount のときに `Semantics<FocusTarget>` を作って engine へ書く形にする。
- **試験**: 足さない。既存の支援技術の E2E（`dev/tests/kivact-public-api/tests/e2e/` の accessibility 系、`crates/arvane/tests/e2e/accessibility.rs`）が回帰の網。
- **完了条件**: unit の確認（`-p kivact-types -p kivact-engine -p arvane -p arvane-shell -p arvane-native -p kivact-core -p kivact-public-api-tests`）。

### V-3 環境・設定・option・platform の request を一つにする

- **目的**: 設計書 5.1 節の `env`・`platform`・`embed`・`check`。
- **決定済みの形**:
  - `Environment`（設計書 5.5 節）が protocol の `Environment`、wire の `HostEnv`、`VirtualShell` の field の役割を全部持つ。`surface_active` は持たない（E-2 で engine の状態から派生させるまでは、engine が core への event に別の field として足す）。`keyboard_inset` を足す（`arvane-native` は `Insets::ZERO`）。
  - `PointerSettings` が protocol の `MouseSettings`（ns・論理 px）と `arvane-native/src/native_input/state.rs:13-29` の `MouseSettings`（ms・物理 px）を一つにする。単位は ns と論理 px。`drag_slop`（論理 px）を足し、`arvane-native` は Windows で `SM_CXDRAG`/`SM_CYDRAG`、ほかは 4.0 を入れる。
  - `FileDialog` が core の `platform::FileDialog`、wire の `FileDialogRequest`、shell の `ShellFileDialog` を一つにする（core の `to_wire`、engine の `pump.rs:471-478` の `dialog()` を消す）。
  - `EmbedOptions` が engine・arvane・shell の三つの純粋な設定値を一つにする（`size`、`scale`、`prefer_gpu`、`gpu_preference`）。callback と panic slot は shell 所有の `ChildHooks` に分け、embed 入口・`Backend::open_child` に options と並べて明示的に渡す。既存の callback/panic の受け渡しを保つ。
  - `LayoutRule`・`LayoutLimit`・`LayoutFinding` は `kivact-types::check` の一つ。core の `test/screen/results.rs:189-220` の二つ目と変換を消す。
  - `ThemeHint` と `dark: bool` の変換（`events.rs:388-393`）を消し、`ThemeHint` だけにする。
- **試験**: 足さない。
- **完了条件**: unit の確認（engine、arvane 系、core、public-api）。
- **18 章の追加**: 失敗の型を `kivact_types::Failure { kind: FailureKind, detail: String }` 一つにする（`ShellError::Failed(String)`、`PlatformResult::Failed`。設計書 18.21 節）。

### V-4 共通の部品と定数

- **目的**: 設計書 5.1 節の `cell`・`process`・`defaults`・`text_utf`。付録 A の冒頭の部品と SM-R20。
- **決定済みの形**:
  - `kivact_types::cell`: `ReportOnce`、`ReportOnceSet<K>`、`Sent<T>`、`Generation`、`Mailbox<T>`（`Mutex`）、`LocalMailbox<T>`（`Cell`）、`Registry<T: ?Sized>`。付録 A の冒頭の署名。それぞれ短い rustdoc で「どの S-9 の例外か」を書く。
  - `kivact_types::process::DetachedThreads`。`kivact-task/src/detached.rs:4`、`arvane/src/shutdown.rs:9`、`arvane-native/src/detached.rs:3`、`arvane-asset/src/detached.rs:4` の四つの static を消して使う。
  - `kivact_types::defaults`: `FRAME_PERIOD_NS`（`arvane-shell/src/window.rs:14`、`protocol/window.rs:89`、`test_display/virtual_window.rs:82` の三つ）、`DEFAULT_WINDOW_SIZE`（960×720 の五つ: `arvane-shell/src/types.rs:52-56`、`child.rs:86-89`、`arvane/src/embed.rs:31-35`、`kivact-engine/src/host/launch.rs:14-17`、`kivact-engine/src/embed.rs:75-78`）、`DOUBLE_CLICK`（`arvane-shell/src/clicks.rs:23`、`native_input/state.rs:20-27`、`protocol/window.rs:49-58`）。
  - `kivact_types::text_utf`: `floor_char_boundary`、`utf16_offset`、`byte_offset_from_utf16`。五つの重複（`kivact-editing/src/edit_state.rs:69,158,165`、`editing/geometry.rs:55`、`kivact-engine/src/model/edit/session.rs:126,148`、`model/interact/text_geometry.rs:278`、`kivact-text/src/pieces.rs:151`、`chunks.rs:326`、`excerpt.rs`）を置き換える。
  - premultiplied → straight の変換（`kivact-engine/src/offscreen.rs:162-170`、`kivact-gpu-share/src/pool.rs:192`）を `kivact_types::color::unpremultiply` 一つに。
  - この unit では部品を作り、上に挙げた重複だけを置き換える。各所の `Cell<bool>` の latch を `ReportOnce` に置き換えるのは各 phase の状態機械の unit が行う。
- **完了条件**: unit の確認（types と置き換えた crate）。
- **18 章の追加**: facade から届く型の公開の `f32` の field を全部 `f64` にする（`api-shape` の report で一覧を作り報告に書く。`Point`・`Size`・`Rect`・`Insets`・`Transform`・`Spring` など、設計書 18.4 節）。taffy の adapter と GPU の code の境界で f32 に変換する。

### V-5 入力 event を一つの形にする

- **実コードとの照合による訂正**: `PointerSample.pressure` は未取得を保持する `Option<f64>`、`is_primary: bool` も保持する。`Routed<T>` は `input: T`、`target: Option<NodeId>`、`region: Option<RegionId>`、`local`、`captured`、`activation` を持つ。window 宛て key は target が `None`、pointer/wheel は `Some`。下の当初の `target: NodeId` の例よりこの訂正を優先する。

- **目的**: 設計書 5.4 節。pointer の 4 つの形と key の 2 つの形を一つにする。
- **決定済みの形**: `PointerSample`、`KeyInput`、`WheelInput`、`ImeInput`、`FileDrop`（`kivact-types::input`）。engine の `Routed<T> { target: NodeId, local: Point, captured: bool, activation: Option<Activation> }`（`kivact-engine::contract`）。`Activation` は、現在 `Reader::take_adopted_activation`（`kivact-core/src/client/dispatch/pointer.rs:178-184` が呼ぶ）が返す値をそのまま包む struct で、C-4 が使う。この unit では engine は常に `None` を入れる（core は今までどおり `take_adopted_activation` を呼ぶ。C-4 で置き換える）。core の `PointerEvent`・`KeyEvent`・`WheelEvent` は `Routed<..>` を包む型にし、field を写さない。`KeyEvent.region` の field を `region()` のメソッドにする。
- **手順**: shell（`Mouse`・`ClickCounter`・`Contacts` が `PointerSample` を作る）→ arvane（`ShellEvent` の pointer・key の variant が新しい型を運ぶ）→ arvane-native の三つの経路（winit、Win32 embed、X11 embed）→ engine（`model/pointer/shell.rs` が `PointerSample` を受ける。`PointerSource` は `Track` の中の状態として残す）→ core（dispatch と event の型）→ interaction・ui・kit の event の field の読み方。
- **完了条件**: unit の確認（全 crate に及ぶので その wave の checkpoint の gate（unit の確認は crate の check まで））。

### V-6 描画の記録・render scene・画素・text style の名前と形

- **目的**: 設計書 5.1 節の `scene`・`render`、5.6 節。
- **決定済みの形**:
  - 作成側の記録: wire の `Scene`/`SceneCmd` → `kivact_types::scene::{DrawList, DrawCmd}`。
  - Arvane の入力: protocol の `Scene`/`DrawCommand` → `kivact_types::render::{RenderScene, RenderCmd}`。`protocol` module を `render` と改名する。
  - 画素: `wire::Pixels`、`arvane_asset::Pixels`（id 付き）、`protocol::ImageData` を `kivact_types::scene::Pixels`（直線 alpha の RGBA8、`width`、`height`、`Arc<[u8]>`）一つにする。`arvane-asset` の id は `AssetImage { id, pixels: Pixels }` で包む。`kivact::draw::RgbaImage` は `Pixels` に統合する（公開の名前の変更は N-1 で facade に反映する）。
  - text style: `kivact_types::text::{TextStyle, ComputedTextStyle}` と `ComputedTextStyle::inherit`。wire の `TextProps`・`TextMore`・`ComputedText`、protocol の `TextStyle`、`kivact-engine/src/measure/inherited_text.rs` の継承、`kivact-lower/src/conversion.rs:47-141` の写しを消す。`arvane-text` は `ComputedTextStyle` を受け取る。
  - `DrawingShape::Path { even_odd: bool }` を `FillRule` にする（`scene_shape.rs:11-17` の変換を消す）。`path::transform([f32; 6])` の変換を消し、`Transform` 一つにする。`kivact_lower::conversion::insets`・`sanitize::color` の 1:1 の変換は、sanitize（範囲の clamp）だけの関数として `kivact-types` の型のメソッドにする。
- **試験**: 足さない。text の E2E（`crates/arvane/tests/e2e/text*.rs`、public-api の text 系）が回帰の網。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **18 章の追加**: VOC-5（設計書 18.16 節）: `motion::Transition` は types の型の再公開にする。types の側の path の記録は `PathData` と名付ける。


---

## 4. Phase A: Arvane

### A-0 `Backend` trait と注入

- **実コード照合による署名・所有権の補足**: `run` は `Box<dyn ShellHandler + '_>` で既存の借用 handler を維持する。`open_child` は純粋 `EmbedOptions` と `ChildHooks` に加え既存の Send `MakeHandler` を受け、UI thread 内で handler を生成する。消費される backend の終了通知は各実装の session guard が所有し、handler/presenter を先に破棄する（起動失敗・unwind も同じ）。embedded guard は実 UI handler の終了に置く。既存 leak/panic 境界での module 保持を失わない。生成前 capability は上限、生成後の ShellWindow capability を確定値とする。詳細は設計書 9.1 の補足。

- **目的**: 設計書 9.1 節の前半。
- **決定済みの形**: 設計書 9.1 節の `Backend`（`capabilities`、`services`、`run`、`open_child`、`shutdown`）。`arvane::spi` は `arvane-shell` の `Backend`・`ShellWindow`・`ShellHandler`・`ShellError` を再公開する（engine はここから使う）。`SteppedBackend`・`SteppedLoop`・`StepOutcome` は A-1、`Services` は A-2 が作り、そのとき `arvane::spi` に足す。`services` は A-2 で足す（この unit の `Backend` は `services` 以外の 4 つのメソッドを持つ）。`arvane::Shell::new(backend: Box<dyn Backend>)`。`arvane_native::backend() -> Box<dyn Backend>`。`kivact_engine::run(backend, client, options)` と `kivact_engine::native_backend()`（`arvane_native::backend()` を返すだけ。core が arvane-native の名前を知らないため）。`arvane::embed::open` は `backend.open_child(..)` を呼ぶ（`arvane_native::embed::open` の直接の呼び出し、`arvane/src/embed.rs:96` を消す）。
- **手順**: `arvane-shell` に trait、`arvane-native` に impl（既存の `Shell::new` の中身を `run` に、`embed::open` を `open_child` に）、`arvane` の `Shell` と `embed`、engine の `host/launch.rs:31` と `embed.rs:70-110`、core の `platform/window_loop.rs`（`run_native` → `kivact_engine::run(kivact_engine::native_backend(), ..)`）。policy の `visible_to`: `arvane-shell` に `arvane-virtual` を足す（A-1 のため先に）、`arvane-native` に `kivact-engine` を足す。arvane はまだ arvane-native に依存している（`retain_module`、`detached_threads`、`test_display`、`TestParent`）。依存を外すのは A-1。
- **完了条件**: unit の確認（arvane 系、engine、core、public-api）。

### A-1a `arvane-virtual` と試験の経路

- **分け方**: 下の手順の 1〜5（crate、`SteppedBackend`・`start_stepped`、`Window` の具体型化と `VirtualWindow`・`Foreground` の削除、engine の試験 host、core の driver）を A-1a で行う。環境変数の整理、feature の `testing` → `test` の改名、公開の `WindowMode` → `MirrorMode` の改名、`Script::respond`、native の smoke の試験は A-1b で行う（A-1a の時点では今の環境変数と名前のまま動く。旧い経路の併存ではなく、改名を後に回すだけ）。

### A-1 の内容（A-1a と A-1b で分けて行う）

- **目的**: 設計書 9.2・9.3 節。付録 A の SM-A15。試験が本番と同じ `Driver`・`Presenter` を通るようにし、`arvane::Window` trait を消す。
- **決定済みの形**:
  - 新しい crate `crates/arvane-virtual`（`//! Role: Virtual Arvane backend with scripted input, a virtual clock and optional mirror windows.`）。
  - `arvane-shell` の `SteppedBackend`・`SteppedLoop`・`StepOutcome`（設計書 9.1 節）と、`arvane::Shell::start_stepped(backend) -> SteppedShell`（`run` と同じ `Driver` で handler を包む）。
  - arvane の arvane-native への依存を外す: `retain_module`・`detached_threads`（`arvane/src/lib.rs:74`、`shutdown.rs:18,25`、`RetainOnDrop(bool)` の `:21`）は `Backend::shutdown(detached)` と `DetachedThreads` に（付録 A の SM-A16）。`TestParent`（`arvane/src/test_display.rs:58`）は `kivact_engine::testing::TestParent`（engine が arvane-native から再公開する）に。policy の `visible_to` の `arvane-native` を `["kivact-engine", "arvane-virtual"]` にする。
  - `VirtualBackend` は `Backend` と `SteppedBackend` の両方を実装する（`run` は仮想の時計を次の締切へ進めながら handler が終わるまで `step` を繰り返す）。`VirtualOptions { size, scale, capabilities, gpu: GpuMode, mirror: MirrorMode }`。`Script`（設計書 9.3 節）。`open_child` は `ShellError::Unsupported`。`ShellWindow` の実装は CPU の frame を持ち、screenshot はそれを読む。GPU の試験は mirror の実 window の surface で `Presenter` の GPU の経路を通し、mirror がないときは `gpu_unavailable()` が理由を返して skip する（設計書 9.3 節）。
  - feature `mirror`: `arvane-native/src/test_display.rs`（実 window への mirror、process に一つの UI スレッド）をこの crate へ移し、frame と支援技術の木を写す。OS の入力は無視する（現状どおり）。`kivact-engine` の feature `test` が `arvane-virtual/mirror` を有効にする。
  - native の smoke の試験 `crates/arvane-native/tests/smoke.rs`（native の window を開き、frame を一つ出し、閉じる。display がない環境では理由を出して return する）を足す（設計書 9.3 節）。
  - 消すもの: `arvane/src/test_display/`（`VirtualWindow`、`VirtualShell`）、`arvane::Window` trait。`NativeWindow` を `arvane::Window`（struct）と改名する。`Presenter` の `State::Foreground`（試験専用）、`defers_accessibility`。engine の `W: arvane::Window` の generic（`host/pump.rs`、`host/events.rs`）。
  - engine の試験 host（`host/testing/test_host.rs` の `ArvaneTestHost`）は `SteppedShell` と `VirtualBackend` の上に作り直す。入力の注入は `Script` を通す（`ShellEvent` を host の入口に直接入れない）。`woken: AtomicBool` は `Mailbox<()>`。
  - 環境変数は `KIVACT_TEST_MIRROR`（`off`/`auto`/`required`、既定 `auto`）、`KIVACT_TEST_GPU`、`KIVACT_TEST_HOLD_MS` の三つ（設計書 9.3 節）。`ARVANE_TEST_HEADLESS`・`KIVACT_TEST_HEADLESS`・`KIVACT_TEST_LIVE` を消す。`kivact::test::WindowMode` は `MirrorMode` と改名する（公開 API の名前の変更。この unit で行い、文書も直す）。
  - サービスの応答は `Script::respond(kind, f)`。登録のない種類は `Unsupported`。現在の `Screen` の台本の API（`controls.rs:180-193` など）はこの上に作り直す。
  - feature の名前（DEP-1）: `arvane`・`arvane-native`・`kivact-engine` の `testing` を `test` と改名する。
- **手順**:
  1. crate を作り、`VirtualBackend`・`Script`・`ShellWindow` の実装・時計を書く（`arvane/src/test_display/` の code を移して `ShellWindow` の実装に直す）。
  2. `arvane` の `Shell::start_stepped`、`Window` の具体型化、`Presenter` の一本化。
  3. `arvane/tests/e2e`（33 試験）を `VirtualBackend` で回るように直す。
  4. engine の試験 host を作り直す。engine の generic を外す。
  5. `kivact-core/src/test` の driver（`test/screen/driver.rs`）を新しい試験 host に合わせる（`kivact-test` への分離は C-1）。
  6. 0.9 節の一覧を直す。
- **注意**: 本番の挙動と試験の挙動の差（click の数え方、focus を失ったときの mouse の `Cancel`、支援技術の公開の時点）が出て試験が失敗したら、本番の挙動を正として試験を直す（設計書 16.2 節）。直した試験の一覧を報告する。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）（全 E2E が新しい経路で回る）。`KIVACT_TEST_MIRROR=off` でも全 E2E が回る。
- **規模**: 大。サブエージェントは一つ。手順 1〜3 と 4〜6 で commit を分ける。

### A-1b 試験の経路の名前と残り

- **目的**: A-1 の内容のうち A-1a で行わなかったもの（上の「分け方」）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。

### A-2 サービスと capability

- **生成時点と owner の訂正（この指定を優先）**: service は生成済み window ごとの resource とし、`ShellWindow::services(&self) -> Rc<dyn Services>` で取得する。`Backend` と `SteppedBackend` に生成前の getter を足さない。native handle・child resize 交渉 state を UI thread の構築時に渡し、後埋め window slot と global current-window を作らない。
- **request の境界**: A-2 で engine の `HostRequest::{Platform(PlatformRequest), RegisterFont(Asset), ReducedMotion(Option<bool>)}` を作り、後二つを pure OS `PlatformRequest` から外す。font 登録は既存の Writer 処理時点、ReducedMotion override は Host の単一 owner を保つ。OS request だけを `Services::perform` へ送る。完了時は実 window 環境を同期してから `EffectResult` を配送し、既存の順序と `Unsupported`／`Failed(Failure)` を維持する。

- **目的**: 設計書 9.1 節の後半。付録 A の SM-A09。
- **決定済みの形**: `ShellWindow` から clipboard・4 つの dialog・open・reveal を外し、`Services::perform(request: PlatformRequest, reply: Box<dyn FnOnce(PlatformResult)>)` 一つにする（reply は UI スレッドで呼ぶ）。service は上記の訂正どおり生成済み `ShellWindow::services` から取る。`ShellError { unsupported: bool }` → `ShellError::Unsupported` と `ShellError::Failed(String)`。`Capabilities` を設計書 9.1 節の一覧にする（現在の 8 つの field に、`clipboard_text`、`file_dialogs`、`open_path`、`reveal_path`、`fullscreen`、`min_size`、`command_key: CommandKey::{Ctrl, Command}`、`context_menu_on: ContextMenuTrigger::{Press, Release}` を足す。これ以外は足さない）。embed の子 window の no-op（X11 の `set_min_size`・`set_soft_keyboard`・`set_ime_text`、Win32 の `set_min_size`・`set_soft_keyboard`、両方の `set_fullscreen`）は `false` を報告する。engine の `host/pump.rs:132-178` の request ごとの match は、`PlatformRequest` をそのまま `Services::perform` へ渡す一本にする。CPU の提示は `ShellWindow::cpu_format() -> CpuFormat` を足し、renderer がその形式で書く（`Xrgb8888` だけを今は定義する）。`Clipboard::{Unopened, Open, Failed}`（SM-A09）。
- **完了条件**: unit の確認（arvane 系、engine、core、public-api の platform services の E2E）。

### A-3 native の runner と shell

- **目的**: 付録 A の SM-A07・A08・A10・A13。設計書 7.5 節の modifier とマウスの位置の持ち主、9.4 節の OS の分岐の移動。
- **手順**:
  1. `Runner.cursor`・`Runner.shift` を消す（SM-A07）。shell に `ModifierState`（正規化）を足し、winit・X11 の xkb mask・Win32 の `GetKeyState` はそこへ入力する。
  2. `NativeInput` の mailbox を `LocalMailbox<T>`、`Subclass` enum にする（SM-A08）。
  3. `ShellEvent::Wake(WakeReasons)`（SM-A10）。`Driver` は `GpuStep` のときだけ GPU を進める。
  4. shell の支援技術: engine が作る更新を検証して適用するだけにし、二つ目の diff を消す（SM-A13）。`ShellEvent::AccessibilityActivated`・`AccessibilityDeactivated` を足す（engine の使い方は E-6）。
  5. OS の分岐の移動（DEP-3）: `kivact-interaction/src/menu/handle.rs:295` の macOS の分岐を `Capabilities.context_menu_on` に、`arvane-native/src/runtime.rs:260` の `touch: !cfg!(target_os = "macos")` は arvane-native の中なのでそのまま、`kivact-ui-soft/src/kbd.rs:14` は K-6 で `ui::kbd` に移すときに `Capabilities.command_key` にする（この unit では interaction の分だけ）。
- **完了条件**: unit の確認。

### A-4 embed の lifecycle

- **目的**: 付録 A の SM-A01〜A06。設計書 9.4 節。
- **手順**: `embed/child.rs` に `ChildLifecycle` と `HandlerSlot` を置き、`embed/session.rs` の `HandlerState`、Win32 の `Phase`、X11 の `opening: bool`・`exists: AtomicBool` を置き換える。Win32 の `Reentry`、`Utf16Decoder`、`Uia`、`WakeCoalescer`、`OpeningGuard`。X11 の `ChildRedraw`、`WindowFocus`、XIM の `ImeAllowance` と `Connection` の enum。engine 側の `closing` は E-2 で `CloseSignal` にする（この unit では arvane-native の中だけ）。
- **完了条件**: unit の確認（`-p arvane-native -p arvane -p kivact-public-api-tests --features embed`）。Windows の embed の E2E（`dev/tests/kivact-public-api` の embed 系、`crates/arvane/tests/e2e` の embed 系）。

### A-5 GPU の状態

- **目的**: 付録 A の SM-A11・A12・A14。
- **手順**: `FrameHealth` に CPU への fallback の段階を入れ、`Device.failures` を消す。`DeviceSurface`、`WorkerState`。arvane-render の `DeviceLost`、`Offscreen`、`Retain`、`LastConfigure`。gpu-share の `SlotState`、`PoolHealth`、`ExternalPass<'_>`。
- **完了条件**: unit の確認（`--features gpu-interop` を含む: `-p arvane -p arvane-render -p kivact-gpu-share -p gpu-interop-producer -p kivact-public-api-tests`）。GPU のない環境の skip の扱いは変えない。
### A-6 arvane-native の port と pipeline

- **目的**: 設計書 17.2 節。
- **決定済みの形**: 設計書 17.2 節の module の構成と `RawInput`・`PlatformKey`・`ImeDriver`・`ImePort`・`DropSession`・`DropPort`。policy に `[module_rules.arvane-native.uses]` を足す: `pipeline = []`、`port = ["pipeline"]`（port は pipeline を使い、pipeline は port を知らない）。
- **手順**:
  1. `pipeline/keys` に正規化と三つの写像表を集める（`key_code.rs`、`embed/windows/input.rs` の `keyboard`、`embed/x11/keyboard.rs` の `logical`・`physical` から表を移す）。
  2. `pipeline/ime` に `ImeDriver` を作り、winit＋Windows の subclass、IMM32、XIM の三つを `ImePort` の実装にする（XIM の handshake の状態は SM-A06 の `Connection` のまま port に残す）。
  3. `pipeline/dnd` に `DropSession` を作り、winit の `Dnd`、Win32 の `WM_DROPFILES`、X11 の Xdnd を `DropPort` にする。
  4. `pipeline/present`: Win32 の子 window の CPU の提示を softbuffer にし、`embed/windows/cpu.rs`（GDI）を消す。
  5. `pipeline/cursor` に cursor の写像表を集める。
  6. a11y の conversion を `pipeline/a11y` へ移す。
- **試験**: 足さない（入力・IME・DnD・embed の既存の E2E と、A-1 の smoke の試験が回帰の網）。
- **完了条件**: unit の確認（arvane 系、engine、public-api の embed・input・IME 系）。Windows の embed の E2E。
- **18 章の追加**: OS を知らない pipeline（`ImeDriver`、`DropSession`、key の正規化、VK・keysym の写像表、cursor の写像表、`RawInput`）は `arvane-shell` の `pipeline` に置き、`arvane-native/port` には OS の呼び出しと winit からの写像だけを残す。`arvane-virtual` の台本に `Script::raw(RawInput)` を足し、key・IME・DnD の E2E を `Script::raw` で書く（`raw_key_mapping_reaches_handlers`、`raw_ime_composition_commits`、`raw_file_drop_reaches_dropzone`）（設計書 18.18 節）。


---

## 5. Phase E: エンジン

### E-1 `kivact-text` の統合と `kivact-lower` の公開面

- **目的**: 設計書 4.3・8.1・8.3 節。付録 A の SM-E33。
- **手順**:
  1. `crates/kivact-text` を `kivact-engine/src/text_input/` へ移す（`git mv`）。`sanitize::color` の一つの依存は V-6 で types のメソッドになっている。crate を消し、0.9 節の一覧を直す。
  2. `kivact-lower` の公開を設計書 8.3 節の `Lowerer`・`LowerCx`・`LowerReport`・`shape` に絞る。engine の 19 file の import を `Lowerer` のメソッドの呼び出しに置き換える。`SceneOut`・`Tint`・`glyphs`・`conversion`・`sanitize` は `pub(crate)`。`Context` の latch（`error`、`dropped`、`failures`）を消し、`LowerReport` を返す。`ArvaneAssets` の `pending: Vec<HostEvent>` は V-1 の `AssetEvent` の列を `LowerReport.asset_events` として返す形にする。
  3. `Model` は host の start で本物の `AssetStore` を受け取って作る（`AssetStore::inline()` で始めて差し替える処理、`host/launch.rs:62-67`・`events.rs:110` を消す）。Offscreen は inline の store で作る（変わらない）。
  4. policy: `kivact-lower` の `visible_to` を `["kivact-engine"]`、`arvane` の `visible_to` を `["kivact-engine", "kivact-lower"]`。
- **完了条件**: unit の確認（engine、lower、core、public-api の offscreen と text 系）。
- **18 章の追加**: `ArvaneAssets` と当たり判定の `shape` を engine へ移し、`BoxShape` を `kivact-types::paint` へ移す。lower の公開は `Lowerer`・`LowerCx`・`LowerReport`・`SceneCache` だけ（設計書 18.17 節）。

### E-2 host の状態機械

- **V-3 の環境 field の追従**: E-1 で移した font collection の唯一の持主から `Environment.fonts: Generation` を派生させる。font 登録成功で世代を進め、更新後の環境を通知する。native の環境再取得で世代が初期値へ戻らないことを確かめる。W1 は field の導入に留め、既存の `fonts_added` と `EffectResult` の順を変えない。

- **目的**: 設計書 8.2 節。付録 A の SM-E01〜E07、E18。
- **手順**: `HostPhase`・`Run`・`AppPresence`・`Surface` を `host/` に置き、`Host` の field を置き換える（`HostPhase::Exiting` は締切を持たない。設計書 8.2 節）。`drawable()` を一つの関数にし、model の `Clock` と core への event の `surface_active` をそこから派生させる（V-3 の一時的な field を消す）。`CloseSignal`（`kivact_engine::contract`。embed の `AtomicBool` を包む）を pump の入口で一度だけ読み、立っていれば `HostEvent::CloseRequested` を core へ送る（14 か所の `closing()` を消す）。core の `client/dispatch.rs` に `CloseRequested` の arm を一つ足して `Runtime::request_exit`（R-2 が作る）を呼び、`AppClient.embed_closing` の読み出しを消す（core を変えるのはこの二点だけ。`ClientMode::Embedded` への整理は C-1）。`SurfaceLost` の往復をやめる。`FrameRequests`、`SmoothScroll`、`TickHistory`、`Host.last_frame` を `LastPresent` と改名。`PlatformRequest::RegisterFont` を engine の内部の request に分ける。`soft_keyboard`・`environment`・`adapter` を `Sent<T>`、`accessibility_noted` を `ReportOnce`。
- **状態の点検表**: 0.7 節。
- **完了条件**: unit の確認（engine、core、public-api の lifecycle・window・embed 系）。
- **並列の注意**: この unit は core の `client/dispatch.rs` を一行変えるので、同時に動くほかの unit が同じ file を触るときは merge で手で合わせる。

### E-3 pointer と key

- **目的**: 付録 A の SM-E10〜E17。
- **手順**: `Track::{Mouse, Touch, Pen}` と variant ごとの gesture の enum（SM-E11）、`Route` の enum（SM-E10）、`MouseGesture::SelectText`（SM-E14）、`PointerState.mouse` の削除（SM-E13）、`Admission` の整理（SM-E15）、`KeyDownTargets`（SM-E16）、modifier の `Sent`（SM-E17）。
- **振る舞いの変更**: `key_up` を `key_down` を受けた node へ送る。E2E を `dev/tests/kivact-public-api/tests/e2e/keyboard.rs`（なければ input の module）に足す: `key_up_goes_to_the_node_that_received_key_down`（node A で key を押し、押している間に focus を node B へ移し、離したとき A の `on_key_up` が呼ばれ B のは呼ばれないこと）。
- **完了条件**: unit の確認。足した試験は、A-1b の後なら unit の確認で名前を指定して回し、前なら checkpoint で回す。

### E-4 IME と編集

- **目的**: 付録 A の SM-E19〜E22、E32。
- **手順**: `ControlledValue`、`InputStart`、`ImeTarget`、`Sent<ImeAllowed>`・`Sent<SoftKeyboard>`、`EditState::take_damage(&mut self)`、`max_length` の明示の setter、editor の索引を一つに、`EditLines` の借用の失敗を `expect("invariant: ..")`。
- **完了条件**: unit の確認（engine、editing、core、public-api の text input・IME 系）。

### E-5 layout と presence

- **目的**: 設計書 8.4 節。付録 A の SM-E27〜E31。
- **手順**: `kivact-layout` の型状態の pass（`Edit`・`Pending`・`Computed`）、`RecordState`。engine の `LayoutPending`、`Model::settle` を一つにして `handle` の 5 か所の `layout()` と `sync` の二重の `pack_child_orders` を消す、`Sent<LayoutReport>`・`Sent<ScrollReport>`・`Stacking`、`Presence` の enum と索引、`ColorChannels`。`kivact-layout/src/lib.rs:11-23` の呼び出し順の doc を消す（型が表す）。
- **完了条件**: unit の確認（layout、engine、core、public-api の layout 系と `LayoutCheck` を使う試験）。

### E-6 支援技術

- **目的**: 付録 A の SM-E23〜E26。
- **手順**: `Dirt` に直接積む（`Changes` の a11y の field を消す）、`A11yPublish`、`SemanticsIdAllocator`、A-3 の activation の event で遷移、`Focused` の `FocusVisibility`・`Reveal`、`A11yState.focus` を消す。
- **完了条件**: unit の確認（engine、arvane 系、public-api の accessibility 系）。

### E-7 paint

- **目的**: 付録 A の SM-E08・E09。
- **手順**: `Redraw`、`ListState`、`ink_gen` の `Generation`。
- **完了条件**: unit の確認。
### E-8 engine の fragment

- **目的**: 設計書 17.1 節の engine 側。
- **主な crate**: engine、layout、style（`Display::Contents` の公開の variant）。
- **手順**: `NodeKind::Fragment` を足す。`NodeTree::flat_children(id)`（topology の変化で無効になる cache）を作り、`layout/adapter.rs`、paint の walker、a11y の build、hit test、focus の navigation がこれだけを使うようにする。`Display::Contents` の box も `flat_children` で透明に扱う（支援技術では自分の semantics を出さない）。`Reader::parent(id)`・`Reader::children(id)` を足す。
- **試験**: 足す。`dev/tests/kivact-public-api/tests/e2e/layout.rs`（なければ作る）に `display_contents_lays_out_children_in_the_parent`（`Display::Contents` の box の子が親の flex の項目として並ぶこと）。fragment そのものは core が C-5 で使い始めるので、この unit では core の flow を変えない。
- **完了条件**: unit の確認（engine、layout、public-api）。
- **18 章の追加**: engine の `tree` の生の `children` を module の外に出さず、`flat_children` だけを出す（`module_rules` で強制。設計書 18.21 節）。

### E-9 出現と退場の大きさ

- **目的**: 設計書 17.10 節の engine 側。
- **手順**: `Style::enter_size(Axis)`・`exit_size(Axis)` を足し、engine の presence の遷移（`NodePresence`）が node の transition でその軸の大きさを 0 から・0 へ動かし、layout がその大きさを使う。`NodeRef::presence() -> Value<Presence>`（公開の `kivact::view::Presence`）を engine の presence の event から作る。
- **試験**: 足す。`presence.rs`（public-api の E2E）に `exit_size_collapses_the_node_before_it_is_removed`（閉じた内容の高さが途中の frame で 0 と元の間にあり、退場の完了で node が消えること。`Screen::advance` で時刻を進める）と `presence_reports_entering_present_leaving`。
- **完了条件**: unit の確認（engine、style、core、public-api）。


---

## 6. Phase R: ランタイム

### R-1 `kivact-runtime` を作る

- **公開と macro の移設方針**: `reactive`・`task`・`motion` は実装 module として private にし、root が旧三つの `lib.rs` の公開集合を明示的に再公開する。`constant_value!` は `macro_export` により新 crate root へ出るので、既存の `$crate::{IntoValue, Prop}` を root の再公開で解決する。macro のためだけに実装 module を public 化しない。core の正準 macro 入口は `kivact_runtime::constant_value` を再公開する。

- **目的**: 設計書 4.1・6 章。`kivact-reactive`・`kivact-task`・`kivact-motion` を一つの crate にする。この unit は移すだけで、振る舞いも内部の形も変えない。
- **手順**:
  1. `crates/kivact-runtime` を作り、三つの crate の `src` を module として移す（`git mv`）: reactive の file は `reactive/`、task は `task/`、motion は `motion/`。`handler`・`turn`・`time`・`machine`・`host` の module（設計書 4.4 節）は R-2・R-4・R-6 が作る。三つの crate の `lib.rs` の Role と公開を root にまとめる。`[module_rules.kivact-runtime.uses]` はこの unit では `reactive`・`task`・`motion` の三つだけを書き（`task = ["reactive"]`、`motion = ["reactive", "task"]`）、後の unit が module を足すたびに設計書 4.4 節の行を足す。
  2. motion の純粋な計算が `kivact-types::motion`（V-1 で wire から移ったもの）を使っていることを確かめる。V-6（18.16 節）で `Transition` は既に types の単一型の再公開となり、`as_wire` は削除済みなので、古い wrapper や変換メソッドを復活させない。
  3. 依存を直す: `kivact-style`・`kivact-i18n`・`kivact-core` が `kivact-runtime` に依存する。facade の `kivact::task`・`kivact::motion` などの再公開の path は変えない（core の再公開の module の中身だけ直す）。
  4. 0.9 節の一覧（三つの crate を消し、`kivact-runtime` を足す）。`visible_to` の `kivact-runtime` は `["kivact-style", "kivact-i18n", "kivact-core"]`。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）（runtime を使う全部）。

### R-2 host との境界と turn の状態機械

- **目的**: 設計書 6.1・6.2 節。付録 A の SM-R04〜R06、R09、R10、R19、SM-C02、C05、C06。
- **決定済みの形**: 設計書 6.1 節の `Host`（`wake` だけ）・`ThreadWaker`・`RuntimeConfig`・`Runtime::new(host, waker, config, build)`（root owner を作ってから `build` で extension を作る）、6.2 節の `TurnClient`・`TurnInput`（時刻は `now` だけが運ぶ）・`TurnItem`・`TurnReport`・`ExitState`・`TurnState`・`Stage` と、現在の code と `Stage` の対応の表。
- **手順**:
  1. runtime に `Runtime::turn` を書く。順序は現在の `kivact-core/src/client/app_client.rs:271-360`、`client/phases.rs`、`client/settle.rs` を、上から下への並びのまま一つの関数へ移す（上限の 1000 round と 100 round も同じ。順序の表は作らない）。設計書 6.2 節の対応の表で各部分に `Stage` を付け、段階の中で許されない操作を `misuse` にする（例: `Stage::Commit` の中の signal の書き込み）。
  2. `on_mount` の待ちを二つの型付きの queue にする（SM-R05）。後の報告の配送を `Stage::Reports` にする（SM-R06）。
  3. `Driver` trait、`driver: OnceCell`、`root_owner: Cell<Option>`、`turn_requested: Cell<bool>` を消す。
  4. core: `HostLink` を消し、`ClientHost`（`kivact_runtime::Host` の実装。`wake` は engine の `HostWaker` を呼ぶ）を作る。engine から turn ごとに受け取る時刻は `TurnInput.now` に入れる。`AppClient` は `TurnClient` を実装し、`dispatch` と `commit` だけを持つ。`client/phases.rs`・`client/settle.rs` を消す。`ExitPhase` を消し `TurnReport.exit` を使う。
  5. runtime を `Launch::Started` の構築のときに作る（現在 `AppClient::new` で作っているのをやめる。extension も同時に入れる）。この unit では `Launch` の形（`AppConfig` を持つ）を保ち、`AppSettings` への統合は C-1。
  6. executor の waker を `ThreadWaker` から構築時に渡す（SM-R19）。
- **状態の点検表**: 0.7 節。
- **試験**: 足さない。全 E2E が回帰の網（turn の順序を変えていないことの確認）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **規模**: 大。サブエージェントは一つ。手順 1〜3（runtime の中）と 4〜6（core）で commit を分ける。

### R-3 reactive の内部の状態

- **目的**: 設計書 6.4 節。付録 A の SM-R01〜R03、R07、R08、R11、R12。
- **手順**: `CompState`、`PassRuns`、`OwnerPhase`、`Frame`/`FrameKind`（`writer` を含む）、graph の `NodeData::Watcher` と、それで書き直した `effect_on`、`SelectorCache`。
- **状態の点検表**: 0.7 節。
- **完了条件**: unit の確認（runtime、core、public-api の reactive 系・flow 系）。
- **18 章の追加**: `FrameKind::Pure`（memo・selector の評価）の中の signal の書き込みを misuse の panic にする（設計書 18.20 節の memo の検査）。

### R-4 時間の仕組みを一つにする

- **目的**: 設計書 6.3 節。付録 A の SM-R16、R22〜R25、SM-C23（motion の Weak の表）。
- **決定済みの形**: `Scheduler { timers, frames, cancel: CancelQueue }`、`SleepState`、`FrameClock`/`FramePhase`/`ClockOffset`、`next_frame()`（runtime の内部の future）、`frame_loop` を `next_frame().await` を繰り返す task にし `Signal<LoopState>` を持つ、`AnimatedCell` と `AnimPhase`（`running` は memo、完了は callback の列、初回の skip は `Watcher`）、`RuntimeConfig.reduced_motion` と `Runtime::set_reduced_motion`（`MotionEnv` と `attach_env` を消す）。`TurnReport.wants_frame` は「frame を待つ task があるか」。
- **試験**: 足さない。animation・frame loop・timer の E2E が回帰の網。
- **完了条件**: unit の確認（runtime、core、interaction、ui、public-api の animation・motion・task 系、kit の動きの試験）。

### R-5 非同期の状態

- **目的**: 設計書 6.6 節。付録 A の SM-R13〜R15、R17、R18、R21。
- **手順**: `AsyncCell`/`AsyncSnapshot`、`ActionPhase`/`Drain`（`Detached` は `Runner::Queue|All` からだけ作れる型にする）、`Latest`、`TaskCell::new(task)`、`PoolState`（`ShutDown` の後の `blocking` は `UiError`、`ErrorKind::Shutdown` を足す）、drain の queue の移動と `Registry<dyn Drain>`。
- **完了条件**: unit の確認（runtime、core、public-api の async・task 系、例の async）。

### R-6 `Machine` と `Trigger` の公開

- **目的**: 設計書 6.5 節。
- **決定済みの形**: 設計書 6.5 節の `Machine`（`State`・`Event`・`Effect`・`Input`・`Timer`）・`Step`・`machine(initial, input, run)`・`MachineHandle`（`send`、`state`、`get`、`feed`。`Clone`・`Copy` は手で書く）・`MachineCx`（`send`、`after`、`cancel`）。`send` の再入は queue に積んで直列に処理する。timer は owner の破棄で消える。`HandlerResult`・`Handled` を `kivact-runtime::handler` に移す（core の `view::input` から。facade の正準パスは N-1 で `kivact::view`）。`Machine` 一式と `Trigger`・`trigger()` は、この unit では `kivact-runtime` の `pub` にするだけで facade から出さない（facade の `kivact::state` は N-1 で作り、そこから一度だけ出す。仮の公開の path を作らない）。この unit の E2E は `kivact_core` を通して使う（`dev/tests/kivact-public-api` は facade だけに依存するので、E2E の追加は N-1 の後に回し、この unit では runtime の API と core の接続だけを作る。E2E は N-1 の unit が足す）。
- **試験**: N-1 で足す（上の理由）。N-1 の手順に次の E2E を `dev/tests/kivact-public-api/tests/e2e/machine.rs` として足すことを含める:
  - `a_machine_renders_its_state_and_runs_effects_after_the_transition`（button を押すと状態が変わり、effect が新しい状態の後に走ること）
  - `a_machine_timer_fires_once_and_is_replaced_by_the_same_key`（`after` を同じ `Timer` の値で二回呼ぶと、後の締切で一回だけ event が届くこと。`Screen::advance` で時刻を進める）
  - `feed_turns_an_input_change_into_one_event`（`feed` に渡した signal を変えると event が一回届き、初期値では届かないこと）
  - `a_machine_timer_is_canceled_with_its_owner`（`show` の分岐を閉じると timer の event が届かないこと）
  - `send_inside_a_transition_effect_is_queued_not_reentrant`
- **文書**: rustdoc だけ。guide の節（`custom-controls.md` の「状態機械で振る舞いを書く」）は N-1 で足す。
- **完了条件**: unit の確認（runtime、core、public-api）。
- **18 章の追加**: `Machine` は設計書 18.13 節の形（`State`・`Event`・`Effect` の三つ、`transition(&self, ..)`、`Step::after`・`cancel`）。E2E の `a_machine_timer_fires_once_and_is_replaced_by_the_same_key` は「同じ event の variant の `Step::after` は前の timer を置き換える」を確かめる。

### R-7 内部の handler の型

- **目的**: 設計書 17.5 節と 17.11 節の `AppStates`。
- **手順**: `kivact_runtime::handler::Handler<E>` を作り、設計書 17.5 節の一覧の内部の slot を置き換える（core の `view/tree.rs`・`handler_layers.rs`・`editing_data.rs` は C-1 の後に触るので、この unit では runtime の中（motion の frame loop の関数、task の `DoneFn`）だけを置き換え、core の分は C-3 で行う）。`use_app_state` の保存を runtime の `AppStates`（型ごとの slot）にする。
- **完了条件**: unit の確認（runtime、core、public-api）。
- **18 章の追加**: `AppStates` は別の map にせず、`Extensions` に `use_app_state` が最初の呼び出しで入れる（設計書 18.21 節）。


---

## 7. Phase C: 中核

### C-1 起動の経路と `kivact-test`

- **目的**: 設計書 7.1〜7.3 節。付録 A の SM-C01、C03、C04、C08、C28。
- **決定済みの形**: 設計書 7.2 節の `AppSettings`・`Launch::new(settings, mode, root)`・`ClientMode`（`Window`、`Embedded`、`Test(TestTiming)`）。`kivact_core::launch` は公開 module（`kivact-test` が使う。facade は再公開しない）。`launch::Inspect<'_>` は設計書 7.3 節の規則（現在 `kivact-core/src/test` が読んでいる core の内部の値ごとに読み出しだけのメソッドを一つ、と `check_trees`）で作り、メソッドの一覧を報告に書く。`kivact::test::Script`（engine の試験 host の台本の包み）と `native_unavailable()`。
- **手順**:
  1. `App` の field を `AppSettings` にまとめる。`AppConfig`（`client/app_client.rs:59-71`）と `AppSettings`（`view/root.rs:37-42`）を消す。`check_window_sizes` を `AppSettings` のメソッドにする。
  2. `Launch` と `LaunchHandle`（`into_engine_client` が返す。`inspect` を持つ）、`TestTiming`（設計書 7.2 節）を作り、`App::run`・`App::embed` が通る。`ClientMode::Embedded` は E-2 の `HostEvent::CloseRequested` を受けて `Runtime::request_exit` を呼ぶ（SM-C03）。`Embedded.host` の `Option` を消す（SM-C04）。`EmbeddedState`（SM-C28）。
  3. `ViewState` を `Launch::Started` の構築のときに完成させる（SM-C08）。
  4. `crates/kivact-test` を作り、`kivact-core/src/test/` の公開 API を移す。core の内部を読む箇所は `launch::Inspect` と engine の試験 host の API に置き換える。`RenderOptions` は `AppSettings` を包む。`render` は `ClientMode::Test` で `Launch` を通る（window の大きさ・最小の大きさ・blocking のスレッドの数を無視しない）。
  5. `kivact-core/src/test/screen/*_tests.rs` の内部の試験（10 file、37 試験）の扱い: 公開 API で書けるものは `dev/tests/kivact-public-api/tests/e2e/` の該当の module へ移す。core と engine の木の一致など内部の不変条件は `kivact-test` の debug の自己検査（現在 pump の debug にある検査）へ入れる。どちらでもないもの（`real_window_tests.rs:398` の `#[ignore]` の測定）は消し、消した試験の名前を報告に書く。embed の E2E（`TestParent`）は `native_unavailable()` で skip できるようにする。
  6. facade の `kivact` の feature `test` は `dep:kivact-test` を有効にし、`kivact::test` を `kivact_test` の再公開にする。`kivact-core` の feature `test` と `#[cfg(feature = "test")]` を消す。0.9 節の一覧。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。`grep -rn 'feature = "test"' crates/kivact-core` が 0 件。
- **規模**: 大。サブエージェントは一つ。手順 1〜3 と 4〜6 で commit を分ける。
- **18 章の追加**: `AppSettings` を `AppOptions` と改名する（設計書 18.12 節）。

### C-2 i18n と style の純粋化

- **目的**: 設計書 7.6 節。
- **手順**:
  1. 文脈の設置（`install_locale`、`install_messages`、`install_layered_messages`、`LocaleCtx`、`MessagesCtx`）を `kivact-core/src/view/boundary/` へ移す。`kivact-i18n` の runtime への依存は `constant_value!`（`kivact-i18n/src/locale.rs:142`、`messages.rs:69` と、足す暦の型）のためだけに残す（設計書 7.6 節）。`message`・`message_with` は core が実装し、`kivact::i18n` から出す。
  2. `kivact-i18n` に `TimeOfDay`、`DateTime`、`HourCycle`、`NumberStyle`、`DateStyle`、`NameWidth` と、`Locale` の `format_number`、`format_date`、`format_time`、`parse_date`、`month_name`、`weekday_name`、`hour_cycle` を足す。表は en-US だけ（ほかの locale は en-US に fallback する。rustdoc にそう書く）。`kivact-i18n/src/date.rs` の `civil_from_days`・`days_from_civil` を公開の暦の計算の根にする（soft の写し `timestamp.rs:489,498` は K-1 で消す）。
  3. `kivact-ui/src/number_input.rs:149-161` の step の精度の書式を `Locale::format_number(value, NumberStyle { fraction_digits, grouping })` と `parse_number` にする（設計書 7.6 節の `NumberStyle`）。
  4. `kivact-style` の host 側の口（`Theme::install`、`Theme::of`、`Style::compile`、`Style::merge`、`Draw::record`）を `kivact_style::internal` module に移し、`kivact-core` だけが使う。`kivact-core` の facade の再公開は `internal` を出さない。`Style::display_grid`・`with_scroll_axes` は `internal` へ（公開の経路は `view::grid` と `scroll(..).axes(..)` だけ）。
  5. `static THEMED`（`kivact-style/src/theme/value.rs:527`）を `Theme` の値の中の表にする。
  6. `Theme` の文脈への設置を core の `view::boundary` が行う。`kivact_runtime::Owner` が `kivact-style` の公開の署名に出ないようにする。
- **試験**: 足す。`dev/tests/kivact-public-api/tests/e2e/i18n.rs` に `en_us_formats_numbers_dates_and_names`（`Locale` の書式の関数の結果を text に出して確かめる）。
- **完了条件**: unit の確認（i18n、style、core、ui、public-api、docs）。

### C-3 view の状態

- **目的**: 付録 A の SM-C07、C09〜C14、C18〜C27。
- **手順**: 付録 A の各項目の「解消後」のとおり。`ReportOnce`・`ReportOnceSet`・`Sent`・`Generation`・`Registry` は V-4 の部品を使う。core の内部の callback の slot（`view/tree.rs:64-99` の `Handlers`、`PartActions`、`EventHandlers<E>`、`SessionHandler`）を R-7 の `Handler<E>` にする。gpu-interop の状態の二つ目の置き場所（`ViewState.Started.adapter`）を消し、`gpu_interop::State` だけにする（設計書 17.11 節）。
- **状態の点検表**: 0.7 節。
- **完了条件**: unit の確認（core、public-api、gpu-interop の feature を含む）。

### C-4 part の入力と事実の持ち主

- **前提**: E-3（engine の `Track` と `Route`）。
- **目的**: 設計書 7.5 節。付録 A の SM-C15〜C17。
- **手順**:
  1. `PartInput`（`Press`・`Hover`）。disabled になる遷移で `Press::Idle`。focus の in/out は engine の既存の `HostEvent::FocusChanged`（現在 `kivact-wire/src/events.rs:181`、V-1 の後は `kivact_engine::contract`）で受ける。hover は engine の hover chain だけ（`view/part.rs:222-223` の observer を消す）。
  2. `RegionEntry`・`Adopted`・`RegionBinding`、変化の通知は `Trigger`。
  3. `PartCapture`・`CaptureTarget`。engine が activation を `Routed.activation` で渡し、`take_adopted_activation`（`Reader` の変更の呼び出し）を消す。
- **完了条件**: unit の確認（core、engine、interaction、ui、public-api の input・canvas region 系、kit の試験）。
### C-5 core の木を side table にする

- **前提**: E-8。
- **目的**: 設計書 17.1 節の core 側。
- **手順**: core の flow が自分の位置に engine の fragment を作り、その中へ子を入れるようにする。`Tree` を `CoreNodes(IdMap<NodeId, CoreNode>)` にし、parent・children・順序を消す。`find_up` と挿入の位置を `Reader::parent`・`Reader::children` にする。`launch::Inspect::check_trees` と、試験の driver の二つの木の一致の検査を消す。
- **試験**: 足さない（flow・list・virtual list・overlay・event の bubbling の既存の E2E が回帰の網）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。


---

## 8. Phase N: 公開 API

この phase は公開 API を設計書 10 章・18 章と付録 B の形にする。N-1〜N-7 の各 unit は中核の crate（0.10 節の G3・G4）の定義と呼び出し元だけを直し、kit・例・catalog・雛形・試験・文書（G5・G6）は N-9 でまとめて直す。CP4 の `-p kivact-docs` の doctest が漏れを見つける。

### N-1 module と正準パス

- **目的**: 設計書 10.1・10.2 節、付録 B.1〜B.3。
- **手順**:
  1. `kivact-core` の module を設計書 7.1 節の形にする: `control` と `primitive` を `view::elements` と `custom` へ移す。`kivact::state` に当たる module を core に作り（`kivact_core::state`）、runtime の handle と setup の関数と `Machine` と `Trigger` を出す。
  2. facade（`crates/kivact/src/lib.rs`）を付録 B.1 の module の一覧にする。root は `App`、`Result`、`UiError`、`ErrorKind` だけ。
  3. 付録 B.3 の行のうち、この unit で行うもの: module の移動（root の再公開、`view` から `state` への移動、`control`・`primitive` の削除、`custom` の新設）、`Asset` の `style` への移動、`DrawText`、`Pixels`、`ErrorKind` の root への移動、`Reactive` の `custom` への移動、`Part`・`app_state` の移動（`app_state` は名前を `use_app_state` にする）、`interaction` の再公開の削除、`use_inline_edit` と `IntoChildren` の移動、`KeyEvent::region()`、`__private`。N-5 で行うもの: 文脈と ambient の読み出しの改名（`use_context`、`try_use_context`、`use_theme`、`use_locale`、`use_direction`、`modifiers`、`pointer_position`）と戻り値の `Value<T>` 化。N-7 で行うもの: `*Api` の改名、試験の API。
  4. `ui` から `interaction` の型の再公開を消す（`kivact-ui/src/lib.rs:170-177`）。`use_inline_edit` を interaction へ、`IntoChildren` を view へ移す。
  5. prelude を付録 B.2 の固定の一覧にする（`crates/kivact/src/prelude.rs` に一つずつ書く。core の prelude は消し、facade の prelude だけにする。`[[style.exceptions]]` の prelude の glob の例外を消す）。
  6. `Transition::as_wire`、`Signal::id`、`Menu::behavior`、`Button::into_node`、`TextInput::into_node`、`F64Signal` を消す（付録 B.3）。`IntoValue::into_prop`・`IntoText::into_text` を `#[doc(hidden)]` にし、`Prop` を `kivact::state::__private` から届くようにする（付録 B.3。sealed にはしない）。
  7. 全部の呼び出し元を直す。docs の import を直す。
  8. R-6 で作った `Machine` の E2E（`machine.rs` の 5 試験）と、`docs/for-kivact-user/{en,jp}/how-to-guide/custom-controls.md` の「状態機械で振る舞いを書く」の節（付録 B.7 の例を元に。doctest が通ること）を足す。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。足した試験は、A-1b の後なら unit の確認で名前を指定して回し、前なら checkpoint で回す。
- **18 章の追加**: prelude は付録 B.2 の新しい一覧（設計書 18.6 節）。`kivact::geometry`（`Point`、`Size`、`Rect`、`Insets`、`Edge`）を新しく作り、`Asset` を `kivact::view` へ、`Reorder` を `kivact::interaction` へ移す（付録 B.8）。

### N-2 handler

- **目的**: 設計書 10.4 節、付録 B.4。
- **手順**: `HandlerResult` を一つにして `kivact::view` に置く（`KeyHandlerResult` を消す）。payload のない event の handler の引数を `impl IntoAction<M>` にし、closure と `Callback<()>` の両方を受ける。payload のある event は `impl Fn(E) -> R + 'static` のまま（closure の引数の型の推論のため。設計書 17.6 節）。付録 B.4 の表の全部の handler の名前・引数・戻り値を直す。位置引数の callback を handler に（`menu`、`command_palette`、`tag_input`）。`a11y_action`、`on_dismiss_default`、`Action::on_complete` と `ActionOutcome`、`Reorder<K>`、`on_text_change`、`on_composition` と `Composition`、`leave_to`、`on_draft_commit`。
- **試験**: 足さない（既存の E2E の書き方を新しい名前に直す）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **18 章の追加**: payload のない handler の引数は `impl Into<Callback<()>>`（`From<F: Fn() -> R> for Callback<()>` を足す）。`IntoAction` は作らない（設計書 18.2 節）。

### N-3 modifier の語彙

- **目的**: 設計書 7.4 節。
- **手順**:
  1. `NodeExt` の全メソッドを `Node` の固有メソッドへ移し、`NodeExt` を消す。`Node` の「Same as `NodeExt::…`」の複製を消す。
  2. `NodeBuilder` を呼び出し側に安全な部分（設計書 7.4 節の表）にし、`with_node` を足す。`node_mut` は sealed にする。部品の builder の固有の semantics のメソッド（`Scroll::role`、`Button::selected`・`expanded`）を消し、部品の中で `with_node` か構築時の `Node` で設定する。
  3. `Part<S>` を `Part::new(state, apply: impl FnOnce(Node) -> Node + 'static)` にし、約 40 の modifier の複製（`view/part.rs:390-742`）を消す。interaction の全 behavior の part の作り方を直す。`Part::on_bind` を `Node::on_mount` に。
  4. semantics を patch で組み立てる（設計書 7.4 節）。`SemanticSources`（`view/common.rs:267-289`）を消す。
  5. kit の builder（basic 43、pop 18、game 9、creative 23、soft 100 の `impl NodeBuilder`）と、`use kivact::primitive::NodeExt;` の全部の import を直す。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **18 章の追加**: part は mount のとき host の node の `disabled`・`invalid`・`name`・`labelled_by`・`described_by` を読んで machine の入力にし、`*Api` に状態の読み出し（`control_state()` など）を足す（設計書 18.10 節）。semantics の patch は、定数だけなら mount のときに畳み、reactive な patch があるときだけ render effect を作り、patch の位置を `why` の origin に記録する。`Node::on_mount(Fn(PartMount))` は `on_attach` と改名する。

### N-4 prop の形

- **目的**: 設計書 10.5・10.6 節、付録 B.5。
- **手順**: 付録 B.5 の表の全行。`IntoScalarRange` を消し、`ScalarRange` に `constant_value!`。構造の option を設計書 10.6 節の表のとおり引数なしのメソッドにする（既定が「する」だった `keep_focused` は `release_offscreen_focus()`、`decorative` は `essential()`）。`IntoValue<f64>` を `i32`・`u32`・`f32` に、`IntoValue<ScalarRange>` を `RangeInclusive<f64>`・`RangeInclusive<i32>` に足す（`From<RangeInclusive<f64>>` は足さない）。kit の同じ形の引数（`f32`、定数の bool）もこの unit で同じ規則で直す（付録 C.1 の表の KIT-6 の行の「inherent の `disabled`」「option の束」「型の alias」は K-7）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **18 章の追加**: `IntoBinding<T>`（`IntoSelection<K>` は `Selection` を作る N-5）、`ItemKey`・`PropValue`、`Duration`・`PathBuf` の定数の prop、構造の option の新しい名前（`release_offscreen_focus`、`keep_focus`、`initially_expanded`、`track_velocity` など、付録 B.8）を行う（設計書 18.7 節）。policy の `structural_options` に 10.6 節の一覧を書き、`structural-options` の検査を有効にする。

### N-5 子・slot・選択・文脈・回復

- **目的**: 設計書 10.3・10.7〜10.9 節、付録 B.6。
- **手順**:
  1. `ViewFn<A>` を足す。ui の slot（`Dialog::content` など）を API-6 の規則で直す（一度だけの mount は `impl IntoView`、作り直すものは `impl Fn(..) -> V`）。`disclosure(title, open, content)` を `.content(..)` に。
  2. `Selection<K>` と `IntoSelection<K>`（設計書 18.7 節の impl の組。`From` の impl は作らない）。`List::multiple`、`ListSelection`、`tree` の位置引数の selection を `.selection(..)` に。
  3. `ErrorBoundary::fallback(Fn(UiError, Retry) -> V)`。
  4. 文脈と ambient の読み出しの改名と戻り値（`use_context`、`try_use_context`、`use_theme`、`use_locale`、`use_direction` と、App 全体の `view::modifiers`、`view::pointer_position`。`platform` の環境の読み出しは全部 `Value<T>`）。`use_context` の文脈がないときの panic の message は設計書 3.4 節の API-12。公開の境界の trait の `#[diagnostic::on_unimplemented]`（API-11）もこの unit で付ける。`platform::pointer_settings`、`keyboard_inset` を足す。
  5. 付録 B.6 の表（`Rc`・`Arc`・`Box<dyn>`・`Location` を署名から消す。`Ext<T>`、`Recipe<S>` の struct、`TestFonts::Only(Vec<Asset>)`、`#[track_caller]`）。
  6. `constant()` を消す。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **18 章の追加**: `use_*` の読み出しを owner の下の closure から呼べるようにする（設計書 18.3 節）、`use_tokens`、`bind!`・`Signal::project`、`for_each` の行の `Binding<T>`（`Signal::item` の削除）、closure の子の `IntoView`、`Component`、`IntoViewFn`、`Theme::ext`・`ThemeRef`・reactive な `App::theme`、`Retry::run`（設計書 18.3・18.7〜18.9・18.11 節）。E2E を足す: `closure_child_updates_text`、`for_each_row_binding_edits_the_item`、`bind_macro_edits_a_nested_field`、`use_theme_reads_inside_a_style_closure`、`app_theme_follows_theme_hint`。

### N-6 `EditPhase` と `on_edit` の網羅

- **目的**: 設計書 10.2・11.2 節。
- **手順**: `EditPhase` を `kivact::view` へ移す。core の native text editing が `EditPhase` を出し、`ui::text_input` の橋渡しを消す。`use_checkbox`、`use_switch`、`use_toggle_button`、`use_select`、`use_tabs` と、`ui` の `checkbox`、`switch`、`select`、`tabs`、`toggle`、`tag_input` に `on_edit` を足す（`EditSession` を使う。値の書き込みの一回が `Begin`→`Commit` の一組）。`kivact-ui/src/lib.rs:37-39` の約束が真になる。
- **振る舞いの変更**: E2E を `dev/tests/kivact-public-api/tests/e2e/edit_phase.rs`（既存の edit の試験の module があればそこ）に足す:
  - `every_value_writing_standard_component_reports_on_edit`（`checkbox`・`switch`・`select`・`tabs`・`toggle`・`tag_input` の値をそれぞれ一回変え、`on_edit` が `Begin` と `Commit` を一回ずつ受けること）
  - `text_input_reports_edit_phases_from_the_native_editor`
- **完了条件**: unit の確認（core、interaction、ui、kit、public-api、docs）。足した試験は、A-1b の後なら unit の確認で名前を指定して回し、前なら checkpoint で回す。

### N-7 interaction の名前と試験の API

- **目的**: 付録 B.1 の interaction と test、B.3。
- **手順**: interaction の handle を全部 `*Api` に（`Toggle` は `SwitchApi`・`ToggleButtonApi` に分ける）。`Element::read_only` を消し、`invalid` → `is_invalid`、`password` → `is_password`。`render_with_handle` を `render_with` に統合。`Screen::evidence`、`Screen::assert_layout_clean`、`RenderOptions::with_theme`・`reduced_motion` を足す（kit の写しの置き換えは K-7）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。
- **18 章の追加**: 付録 B.8 の改名の表のうち、この時点で存在する item の行を行う（`ViewportApi`、`pagination`、`heading`、`card`、`calendar`、`StepperItem`、`kbd_combo`、`StateTokens` のように後の unit（K-0〜K-9、G-*）が作る item は、その unit が B.8 の名前で作る）。

### N-9 kit・例・文書・試験の追従

- **目的**: Phase N の公開 API の変更（N-1〜N-7）を、G5（5 つの kit）と G6（例、catalog、雛形、`dev/tests`、`docs/`、README）に一度にまとめて当てる（0.10 節）。
- **分け方**: kit ごとに一つ（basic、pop、game、creative、soft）と、G6 に一つの、6 つのサブエージェントを並べる。どれも build 不可から始めてよい。全部を merge したら、オーケストレータが `cargo check --workspace --all-targets --all-features` を回し、残った error を組ごとに直す。
- **完了条件**: CP4 の gate。

### N-8 公開 API の検査を有効にする

- **目的**: 設計書 14.1 節。
- **手順**: `api-shape`・`canonical-path` の `skip` から `kivact`、`kivact-core`、`kivact-runtime`、`kivact-style`、`kivact-i18n`、`kivact-interaction`、`kivact-ui`、`kivact-test` を外す（api-shape は facade から届く item だけを見る）。状態の検査群の `skip` から、`kivact-interaction`・`kivact-ui`・kit を除く全 library crate（`kivact-types`、`kivact-runtime`、`kivact-i18n`、`kivact-editing`、`kivact-style`、`kivact-core`、`kivact-test`、`kivact-engine`、`kivact-lower`、`kivact-layout`、`arvane`、`arvane-shell`、`arvane-native`、`arvane-virtual`、`arvane-text`、`arvane-render`、`arvane-asset`、`kivact-gpu-share`）を外し、同じ crate の `lib.rs` に clippy の `#![deny(clippy::too_many_lines, clippy::disallowed_types)]` を足して違反を直す（関数を分けるのは M-4）。`os-branch` の `skip` から kit を除く全 library crate を外す。出た違反を直す（例外は inline の目印 `// xtask-allow(<検査>): <理由>` か `#[expect(.., reason = "..")]`。理由は状態でないことの説明か、10.11 節の列挙に限る）。
- **完了条件**: その wave の checkpoint の gate。

---

## 9. Phase I・U: 振る舞いと標準部品

各 unit は、付録 A の該当の項目を `Machine`（設計書 6.5・11.1 節）で書き直す。**振る舞いを変えない**（例外は unit に書く）。既存の E2E（public-api の interaction 系、ui 系、kit の試験）が回帰の網。各 unit の完了の前に、その unit が触った module だけに `cargo xtask check --only state-fields --crate <kivact-interaction か kivact-ui> --module <module>` を回し、違反がないことを確かめる（状態の検査群の `skip` から crate 全体を外すのは I-6 と U-2 の後。そのとき `lib.rs` に clippy の `#![deny(clippy::too_many_lines, clippy::disallowed_types)]` も足す）。

### I-0 option と policy

- **目的**: 設計書 11.1 節と 18.15 節の `OptionsCell<S>`、`InteractionPolicy`（`UiPolicy` は作らない）、gesture の閾値。付録 A の SM-I28。
- **手順**:
  1. `kivact-interaction` に `Settings<S>`（`Signal<S>` の薄い包み）を作る。
  2. `kivact::interaction::InteractionPolicy` を、設計書 18.15 節の behavior ごとの入れ子の struct で作る。旧い theme の拡張の型（interaction の `MenuPress`・`SubmenuTiming`・`DisabledMenuFocus`、ui の `UiRecipes` を除く 11 型、`TooltipConfig`）は、振る舞いのものは入れ子の struct の field に、見た目の動きのもの（`ButtonBusyMotion`、`ProgressMotion`、`ToastRowMotion`、`ToggleLabelReservation`）は `UiTokens` の field にして、`ThemeExtension` の impl を外す。見た目の値（`DataTableStickyPaint`・`DataTableRowStatusPaint`）は recipe の表へ。`OverlayViewport` は `Theme::overlay_margin`。
  3. gesture の閾値の literal（`menu/press.rs:96-99` の 10.0・13.0 px、`ui/select.rs:258` の 10 px、`ui/toast_swipe.rs:11` の 8.0 px）を、設計書 11.1 節の `Policy` の field（`InteractionPolicy.menu.drift`、`InteractionPolicy.select.sheet_slop`、`InteractionPolicy.toast.swipe_slop`。設計書 18.15 節）の既定値に移す。値は変えない（振る舞いを変えない）。`platform::pointer_settings()` はこの unit では使わない。
  4. soft の theme が入れていた 15 の拡張（`kivact-ui-soft/src/theme.rs:70-167`）を、`InteractionPolicy`・`UiPolicy` の値と recipe の表と `Theme::overlay_margin` に直す（この unit で行う。kit の他の規約は K-7）。
- **完了条件**: unit の確認（interaction、ui、5 kit、public-api）。
- **18 章の追加**: `UiPolicy` を作らず、振る舞いの knob は全部 `InteractionPolicy` の behavior ごとの入れ子の struct に、見た目の動きは `UiTokens` に置く（設計書 18.15 節）。`Settings<S>` は `OptionsCell<S>` と名付け、behavior の `*Settings` は `*Options` にする。`*Api` の option の builder は setup の中だけにする（mount の後は API-12 の misuse）。

### I-1 開閉の family

- 付録 A の SM-I01〜I05（disclosure、popup、delayed、tooltip、dialog）。tooltip の doc を事実に直す（SM-I04）。

### I-2 menu と選択の family

- 付録 A の SM-I06〜I10（combobox、select、menu の session・press・level）。menubar と command palette も同じ `MenuSession`・`ComboboxState` を使うように直す。

### I-3 編集・drag・文字編集・toggle

- 付録 A の SM-I11〜I13、I24。`DragSession::Pending`（slop）と軸の固定・速度の option を `DragApi` に足す（`DragApi::slop(impl Into<f64>)`、`axis_lock()`、`velocity()`（設計書 11.1 節）。速度は既存の `drag_velocity` を `DragApi` の中へ移す。`drag_settle` の snap の関数は公開の関数のまま）。
- **振る舞いの変更**: slop の既定は 0（今と同じく押した瞬間に始まる）。slop を使う呼び出し元は U-1 と kit。

### I-4 一覧の family

- 付録 A の SM-I14〜I16、I26、I27（list、typeahead、roving、tree、table、生存の token）。`Owner::is_alive` を足す。

### I-5 値の family

- 付録 A の SM-I17〜I20（scalar、touch の slider、zoom・pan、space の pan）。touch の slider を `DragApi` の上に作り直す。`SplitterApi` に `collapsible(threshold: impl Into<f64>)` と `snaps(impl IntoValue<Vec<f64>>)`（0.0〜1.0 の割合）を足す（付録 C.3、soft の `use_resizable` の行き先。K-5 が使う）。

### I-6 通知と追従の family

- 付録 A の SM-I21〜I23、I25（hover corridor、toaster、follow end、announcer）。
- **18 章の追加**: toaster は toast ごとの machine（期限の timer を持つ）と、列を持つ machine に分ける（設計書 18.13 節: timer は一つの machine の中で event の variant ごとに一つ）。

### U-1 ui の gesture

- 付録 A の SM-U01〜U03（switch の drag、toast の swipe、select の touch の sheet）。どれも `DragApi`（slop、軸の固定、速度）の上に作り直す。

### U-2 ui の開閉と編集

- 付録 A の SM-U04〜U06（tree view の reveal、inline の編集、disclosure の動き）。

### I-7 出現と退場の一本化（interaction・ui）

- **前提**: E-9、I-1、U-1。
- **目的**: 設計書 17.10 節。付録 A の SM-I01、SM-U02、SM-U06 の新しい形。
- **手順**: `use_content_reveal` を消し、`disclosure`・`tree` の行の reveal・toast の swipe の閉じる動きを engine の presence（`enter_size`・`exit_size`、drag の位置からの退場）で書き直す。recipe の表に enter・exit の style を足す（5 kit の recipe も直す）。
- **振る舞いの変更**: 閉じた disclosure の内容が unmount される、閉じる動きの仕組みが変わる。E2E を `dev/tests/kivact-public-api/tests/e2e/presence.rs` に足す: `disclosure_unmounts_closed_content_after_its_exit`、`toast_swipe_dismiss_exits_from_the_drag_offset`、`tree_row_reveal_expands_with_enter_size`。
- **完了条件**: unit の確認（interaction、ui、5 kit、public-api の disclosure・toast・tree 系）。足した試験は、A-1b の後なら unit の確認で名前を指定して回し、前なら checkpoint で回す。

### I-8 振る舞いを APG に合わせる

- **前提**: I-1〜I-7、U-1、U-2。
- **目的**: 設計書 17.9 節。
- **手順**: 設計書 17.9 節の pattern ごとに、APG の Keyboard Interaction と Roles・States・Properties を、interaction と ui の対応する behavior と部品に一行ずつ当てる。違いは APG のとおりに直す（作者の決定は 17.9 節のとおり優先）。違いの表（pattern、現在、APG、直した内容）を報告に書く。
- **振る舞いの変更**: 直した違いごとに E2E を `dev/tests/kivact-public-api/tests/e2e/apg_<pattern>.rs` に足す（名前は `<pattern>_<APG の操作>`。例: `tree_view_star_expands_all_siblings`）。
- **完了条件**: その wave の checkpoint の gate（unit の確認は crate の check まで）。足した試験は、A-1b の後なら unit の確認で名前を指定して回し、前なら checkpoint で回す。

**I・U の各 unit の共通の完了条件**: unit の確認（interaction、ui、5 kit、public-api）。0.7 節の点検表。I-6 と U-2 の後に状態の検査群の `skip` から `kivact-interaction` と `kivact-ui` を外し（clippy の `#![deny]` も足す）、G-phase。push。

---

## 9.5 Phase T・G・L: 拡張口・不足・地域書式

### T-1 編集する node の制約の解除と公開の文字編集の API

- **目的**: 設計書 17.12 節。
- **手順**: engine の editor の paint の後に node の draw layer を重ね、編集する node に `draw`・`draw_over`・inline の子を許す（`Unsupported` の報告を消す）。`LineColors` を `TextEditing::line_styles` にし、`TextEditing::decorations` と `Node::draw_with_layout` を足す。`kivact-core/src/primitive/text_editing.rs:12-14` の「native leaf」の doc を事実に直す。
- **試験**: 足す。`text_editing.rs`（public-api の E2E）に `an_editing_node_draws_its_own_layer_over_the_text`、`line_styles_color_each_line`、`decorations_mark_a_range`。
- **完了条件**: unit の確認（engine、core、interaction、ui、public-api）。

### T-2 公開 API だけの text field の証明

- **目的**: 設計書 17.12 節の証明。
- **手順**: `dev/tests/kivact-public-api/tests/e2e/custom_text_field.rs` に code editor の部品を `kivact::custom` と `kivact::interaction` だけで書き、`ui::text_input` の E2E と同じ操作（文字の入力、IME の変換と確定、選択、clipboard、undo の `EditPhase`、支援技術の text と選択）を確かめる試験を書く。足りない口が見つかったら、作業を止めて作者に聞く（新しい公開 API の概念になるため）。
- **完了条件**: unit の確認（public-api、docs）。

### G-1 Path と snapshot

- 設計書 17.17 節の `Path::contains`・`stroke_contains`・`arc_to`（canvas の形。旧 `arc_to` は `svg_arc_to`）・`parse_svg` と `NodeRef::snapshot`。E2E: `path_contains_matches_fill_rule`、`canvas_arc_to_rounds_a_corner`、`parse_svg_draws_the_same_path`、`snapshot_returns_the_node_pixels`（`dev/tests/kivact-public-api/tests/e2e/canvas.rs`。なければ作る）。

### G-2 明示の pointer の capture

- 設計書 17.17 節の `PointerEvent::capture`・`release_capture`。E2E: `captured_pointer_keeps_moving_events_outside_the_node`。

### G-3 モバイルの境界

- 設計書 17.16 節の `InputPurpose`（四つの `password` の統合を含む）、`ShellEvent::KeyboardInset`、状態の復元の API と `Capabilities.state_restoration`。`arvane-virtual` の台本に keyboard の inset と状態の保存・復元を足す。E2E: `text_input_purpose_reaches_the_platform`（台本が受け取った purpose を確かめる）、`keyboard_inset_is_observable`、`saved_state_is_restored_on_the_next_launch`（virtual backend）。

### G-4 soft kit が求めた中核の不足

- 設計書 17.18 節の表の全行（C-38 の `Display::Contents` は E-8 が行う）。各行に E2E を一つ足す（名前は `<API>_<確かめること>`）。C-35 の option の名前と型は `soft-ui-work-order/components/02-*.md` の「Kivact 実装方針」の節の記述のとおり。記述がない option は作者に聞く。
- **18 章の追加**: C-20 は `platform::theme_hint()` だけを足し、`ThemeMode::System` は作らない（設計書 18.11 節）。

### G-5 非同期の標準部品

- 設計書 17.19 節の `ui::async_view` と、17.6 節の `IntoValue<AsyncState<T>>` for `Resource`・`Action`。E2E: `async_view_shows_loading_then_ready`、`async_view_error_retries`。例 `examples/kivact/async` を `async_view` で書き直す。
- **18 章の追加**: `async_view(source: impl AsyncSource<T>, ready)`（`Resource` と `Action`）、`.loading(..)`、`.fallback(..)`。`IntoValue<AsyncState<T>> for Resource` は作らない（設計書 18.5 節）。facade の feature `tokio` と `task::on_tokio` も足す（実装は `kivact-runtime`。E2E: `on_tokio_future_resolves_and_wakes_the_ui`、`on_tokio_task_is_aborted_with_its_owner`）。guide に「HTTP で JSON を読む」の how-to を en・jp で足す。

### L-1 ICU4X による地域書式

- **前提**: C-2。
- **目的**: 設計書 17.14 節。
- **手順**: `kivact-i18n` に ICU4X 2 系の crate（作業の時点の最新の 2.x）を足し、C-2 の書式の関数を ICU4X で実装する。手書きの表（`locale.rs:120-140` と C-2 の en-US の表）を消す。`plural_category`、`compare`、`Messages::with_plural` と `message_with` の plural を足す。`THIRD-PARTY-NOTICES.md` に ICU4X（Unicode License）を足す。全 locale の compiled data を含める（設計書 16.3 節の Q-07）。L-1 の前後で、`examples/kivact/counter` の release build の実行ファイルの大きさを測り、増えた大きさを `PROGRESS.md` に書く。
- **試験**: 足す。`i18n.rs` に `ja_jp_and_de_de_format_numbers_and_dates`、`plural_forms_follow_the_locale`、`collation_sorts_by_locale`。
- **完了条件**: unit の確認（i18n、core、ui、kit、public-api、docs）。

---

## 10. Phase K: 公式 kit

付録 C の作業。どの unit も、中核へ上げる部品を **先に中核（`kivact-interaction` と `kivact-ui`）に作り**、標準の E2E を `dev/tests/kivact-public-api/tests/e2e/` に書き、それから各 kit の版を消して recipe と variant を足し、catalog の page と kit の E2E を中核の部品で書き直す。中核の部品は、元の kit の版の機能の和集合を持つ（付録 C.2 の「和集合」の欄）。kit の部品を消すときは、その部品の公開の option が中核の部品のどこに当たるかの対応表を unit の報告に書く（機能が落ちていないことの確認）。

### K-0 kit のための中核の部品

- **手順**:
  1. `use_tokens::<T>() -> ThemeRef<T>` は N-5 で作ってある。kit が自分の `tokens()` を消してこれを使う。
  2. `UiRecipes::filled()`。`theme_conformance` の E2E に、kit ごとの README の列挙との一致の確認を足す（KIT-3 の規約。列挙はこの unit で各 kit の README に書く）。
  3. `NodeRef::scroll_state() -> Value<ScrollState>`、`NodeRef::image_status() -> Value<ImageStatus>`、`FrameLoop::while_present(node_ref)`。
  4. `ui::RadioGroupVariant::Segmented`（標準の variant）。
  5. kit の `Color` enum を `ColorName` に改名する（5 kit。soft は新しく作る）。game・creative の `ThemeOptions` trait を `theme_with(mode, ThemeOptions)` にする（付録 C.1 の KIT-1）。kit の `tokens()` を消す。
- **試験**: 足す。`dev/tests/kivact-public-api/tests/e2e/node_ref.rs`（なければ作る）に `scroll_state_follows_the_scroll_position`、`image_status_reports_a_failed_image`。
- **完了条件**: unit の確認（core、ui、5 kit、catalog、public-api、docs）。
- **18 章の追加**: 旧 K-10（recipe の token 化）をここで行う: `UiTokens` を部品ごとの入れ子の struct にし、token にするのは「現在いずれかの kit が上書きしている値」と「theme から読む値」だけにする（設計書 18.15 節）。pop と game の recipe の file を `UiTokens` の値と形の違う recipe だけにする。別の kit の theme の下の kit の variant は中核の既定の recipe で描き、`ReportOnce` の diagnostic を出す（18.11 節）。

### K-1 日付・carousel・dropzone

- 付録 C.2 の calendar、date inputs、carousel、dropzone。付録 A の SM-K01、K02、K18〜K20、K28（month swipe）、K29、K31。kit の月名の表と暦の計算の写しを消す（C-2 の `kivact-i18n` を使う）。
- **標準の E2E**（足す）: `calendar_selects_a_date_with_the_keyboard`、`calendar_range_picks_start_then_end`、`date_input_parses_typed_text_and_opens_the_calendar`、`time_input_accepts_hours_and_minutes`、`carousel_moves_with_arrows_and_reports_the_index`、`dropzone_accepts_dropped_files_and_rejects_invalid_ones`。

### K-2 開閉と hover と選択の入力

- 付録 C.2 の disclosure group、hover card、combobox 系、select の touch の sheet、drawer の swipe、copy feedback。付録 A の SM-K03、K04、K14〜K17、K24、K28（drawer）。
- **標準の E2E**: `disclosure_group_single_keeps_one_item_open`、`hover_card_opens_after_the_delay_and_on_focus`、`combobox_filters_options_and_selects_many_as_tokens`、`drawer_closes_on_swipe`、`copy_feedback_resets_after_its_duration`。

### K-3 navigation と focus の部品

- 付録 C.2 の toolbar、toggle group と segmented、button group、pagination、stepper、breadcrumb、navigation menu。付録 A の SM-K10、K12、K13、K27。
- **標準の E2E**: `toolbar_moves_focus_with_arrows_as_one_tab_stop`、`toggle_group_single_and_multiple_selection`、`pagination_moves_pages_and_keeps_focus`、`stepper_marks_the_current_step`、`breadcrumb_marks_the_current_page`、`navigation_menu_opens_on_hover_and_pins_on_click`。

### K-4 collection と creative と game

- 付録 C.2 の tree の採用、static table、item list。creative の outliner・drag の写し・zoom・pan の model、game の frame の loop。付録 A の SM-K11、K30、K33。
- **標準の E2E**: `static_table_exposes_rows_and_cells`、`item_list_exposes_list_semantics`。

### K-5 soft に残る状態と hook

- 付録 C.3 と、付録 A の SM-K05〜K09、K21、K25、K26、K28（bottom sheet）。soft の E2E が回帰の網。

### K-6 見た目の部品（Tier 2）

- 付録 C.2 の Tier 2（badge、avatar、skeleton、spinner、empty state、alert と banner、card、kbd、input group）。付録 A の SM-K23、K32。soft の `kbd.rs:14` の macOS の分岐を `Capabilities.command_key` に（DEP-3）。
- **標準の E2E**: `avatar_falls_back_to_initials_when_the_image_fails`、`empty_state_announces_once`、`alert_dismiss_restores_focus`、`kbd_uses_the_platform_command_key`。

### K-8 `viewport` を中核へ

- 設計書 17.13 節。creative の `viewport_surface` を `kivact::view::viewport` に移し、creative の呼び出し元と catalog を直す。E2E: `viewport_maps_points_between_view_and_content`、`viewport_shows_pixels_with_fit`（GPU の frame の試験は既存の creative の `gpu_interop` の試験を public-api へ移す）。

### K-9 Tier 3 の部品を中核へ

- 設計書 17.15 節の表の全行。付録 C.5 の各 kit の「消す（Tier 3、中核へ）」を行い、kit は recipe と variant を足す。E2E: `chat_thread_is_a_log_and_follows_the_end`、`code_block_copies_and_numbers_lines`、`progress_ring_reports_its_value`、`stat_exposes_label_and_value`、`heading_has_its_level`、`description_list_pairs_terms_and_details`。

### K-7 kit の規約の残り

- 付録 C.1 の表の残り全部（slot、children の trait、variant、固有の `disabled`、option の束、型の alias、message の key、literal の色、RTL、breakpoint、`provide((), ..)`、soft の rustdoc）と、付録 C.4 の改名、付録 C.6（kit の試験を一つの binary に、試験の helper を中核の API に、catalog の font を `examples/shared-assets/fonts/` に）。
- `name-collision`・`kit-literal-color`・`api-shape`・状態の検査群・`os-branch` の `skip` から 5 kit を外し、kit の `lib.rs` に clippy の `#![deny(clippy::too_many_lines, clippy::disallowed_types)]` を足して、違反を直す。
- **完了条件**: その wave の checkpoint の gate。push。

**K の各 unit の共通の完了条件**: unit の確認（interaction、ui、触った kit とその catalog、public-api、docs）。中核へ上げた部品ごとに `UiTokens` の入れ子の struct（設計書 18.15 節）と recipe の表を作り、元の kit が上書きしていた値を kit の theme の `UiTokens` に入れる。中核の部品の rustdoc と、`docs/for-kivact-ui-user/{en,jp}/<kit>/` の該当の page を新しい書き方にする。

---

## 10.5 Phase X: 横断の整理

### X-1 feature の閉じ込め

- **目的**: 設計書 17.3 節（DEP-4）。
- **手順**: 設計書 17.3 節の表のとおり、feature を実装の crate と公開 API の crate だけに残す。`gpu`・`gpu-interop`・`embed` について、間の crate（`kivact-style`、`kivact-engine`、`kivact-lower`、`arvane`）の `cfg(feature = ..)` と feature の定義を消す（creative kit の `gpu-interop` は、`viewport_surface` が中核へ移る K-8 で消す）（`test` は表の実装の crate の連鎖のまま）。間の crate では経路を常に compile する（実装がないときは `Unsupported`）。facade の `Cargo.toml` に anchor の optional の依存を足す。`dev/tools/check-features.sh` の一覧を新しい feature にする。policy の `visible_to` に anchor を足す（anchor の依存を code から使わないことは review で確かめる。`anchor-deps` の検査は作らない）。
- **完了条件**: その wave の checkpoint の gate。`bash dev/tools/check-features.sh` が通る。`grep -rn 'feature = "gpu-interop"' crates` が実装と公開 API の crate と creative kit（K-8 まで）だけに出る。
- **18 章の追加**: この unit は E-1 の後に単独で行う（以後の unit が `cfg(feature)` を二通りで保たなくてよいように。設計書 18.20 節の前提）。

---

## 11. Phase F: 仕上げ

### F-1 文書

- **手順**:
  1. `docs/for-kivact-user/{en,jp}/getting-started/` と `how-to-guide/` を付録 B.7 の書き方に書き直す。`coming-from-react-or-solid.md` の concept map を付録 B.2 の prelude で import なしに書けることを前提に直し、「`callback` は `kivact::view` から」などの注記を消す。`custom-components.md`・`custom-controls.md` は `Node` の固有メソッド・`NodeBuilder`・`Part::new`・`Machine` で書き直す。
  2. `docs/for-kivact-ui-user/{en,jp}/` の 5 kit の page を新しい部品（中核へ上がったものは `kivact::ui` と kit の variant）で書き直す。
  3. README・README_JP の crate の一覧と図。`dev/README.md`。`CONTRIBUTING.md`（試験の場所の表に `kivact-test` と `arvane-virtual`、新しい検査、`KIVACT_TEST_MIRROR`。CI の記述は変えない）。
  4. `docs/for-kivact-developer/architecture.md`（日本語、新規）を書く。中身は設計書から長く使うものだけ: crate の一覧と境界の根拠（設計書 4.1・4.3 節）、module の向きの考え方（4.4 節）、規則（3 章の B・VOC・S・API・DEP、12 章の KIT）、事実の持ち主の表（7.5 節）、`Machine` の書き方（6.5・11.1 節）、handler の語彙（10.4 節）、構造の option の一覧（10.6 節）。`CONTRIBUTING.md` の「判断の基準」と「コード」の節から、この文書を参照する。CONTRIBUTING の試験の節の「実ウィンドウの E2E」を「`kivact::test` による App 全体の E2E（`arvane-virtual` の上で、手元では実 window に mirror する）」と書き直す。
  5. `docs/for-kivact-developer/strategy.md` は変えない。ただし 8 章「macOS 向けの分岐を入れない」が守られたこと、5 章の「アプリが描いた絵の表示」が中核の `viewport` になったことは、F-3 の PR の説明の下書きに書く。
- **完了条件**: unit の確認（`-p kivact-docs`）。英語と日本語が同じ file 名で対になっている。
- **18 章の追加**: `docs/for-kivact-developer/decisions.md`（作者の決定の ADR の記録: 16.3 節、17.9 節の APG より優先する決定、17.20 節、`DECISIONS.md` の全部）を作る。`architecture.md` に、設計書 17.3 節の feature の表、6.2 節の `Stage` の表と 6.3 節の時間の仕組み、15 章と 18.23 節の表を「X を足すときに変える場所」の check list にしたもの、API の 15 の規則（3.4 節）を書く。guide の「Rust の 5 つの詳細」に、整数を返す closure は `IntoValue<f64>` に渡せない（`as f64`）ことと、prelude の名前を変数名にすると関数が隠れる（E0618。`text`、`row`、`list`、`field`、`Action`）ことを足す。coming-from-react に「`Machine` は `useReducer` に effect と timer を足したもの」を足す。

### F-2 検査の仕上げ

- **手順**:
  1. xtask の全検査の `skip` が空であることを確かめる（設計書 18.20 節。`policy-coverage` は作らない）。
  3. `dev/tools/check-features.sh` に `kivact/gpu`、`kivact/gpu-interop`、`kivact/test` と新しい crate の feature を足す。
  4. `templates/basic/Cargo.toml` の「CI builds and tests it」の文を事実に合わせて消す（CI は既定で無効。`.github/workflows/` は変えない）。雛形に root の `[profile.dev.package]` の engine 側の opt-level の設定を写し、雛形の README に理由を書く（root の profile は workspace の外では効かないため）。
- **完了条件**: その wave の checkpoint の gate。
- **18 章の追加**: N-8・I-6/U-2・K-7 で clippy の `#![deny]` を足していない crate（構造の unit が触らなかった crate。例: 例・catalog・雛形・試験の crate）の違反を直してから、clippy の `too_many_lines`・`disallowed_types` を workspace の lint（`[workspace.lints.clippy]`）に移し、crate ごとの `#![deny(..)]` を消す。xtask の全検査の `skip` が空であることを確かめる。

### F-3 最後の確認と PR の説明の下書き

- **手順**:
  1. G-final（12 章）。P-1 の全数調査（1〜3）をもう一度回し、残りが 0 であることを `PROGRESS.md` に書く（設計書 17.8 節の 3）。
  2. `docs/for-kivact-developer/manual-verification.md`（なければ作る）に、作者が手で確かめる項目（IME、screen reader、embed の DAW、GPU の共有、mirror の窓）を、この再設計で変わった経路に絞って書く。
  3. `refactoring/` と設計書・本書を消す前に（長く使う規則は F-1 で `docs/for-kivact-developer/architecture.md` に移してある）、PR の説明の下書きを `.claude/pr/foundation-v3.md`（git の管理の外。`/.claude/` は `.gitignore` 済み）に書く。中身: 31 の決定（設計書 1 章と 17 章）、crate の変化（4 章）、公開 API の変化の要約（付録 B.3 の表の要点）、消えた暗黙の状態機械の数、振る舞いを変えた点（`key_up` の宛先、`on_edit` の網羅、試験の経路の本番への一致、`blocking` の shutdown の後の error）、行わなかったもの（設計書 17.20 節の backend と理由）、`DECISIONS.md` の要点、作者の手動の確認の一覧。
  4. `refactoring/`、`REFACTORING_DESIGN.md`、`REFACTORING_WORK_ORDER.md` を消す commit を作る（`SOFT_UI_WORK_ORDER.md` と `soft-ui-work-order/` は別の作業の文書なので、作者の指示がない限り残す）。
  5. integration branch を push し、作者に、G-final の結果と PR の説明の下書きの path を知らせて止まる。PR を作るのと `main` への squash merge は作者が行う（エージェントは行わない）。
- **18 章の追加**: `docs/for-kivact-developer/decisions.md` と `architecture.md` は消さない（消すのは `refactoring/`、設計書、本書だけ）。12 章の試しに、paint の性質と標準部品を一つずつ足す試しを加え、crate の数と file の数を記録する（設計書 18.23 節）。

---

## 12. 完了条件

全部を満たしたら完了とする。

1. **checkpoint の gate**（mirror の有無の両方）が integration branch で通る（Windows）。WSL で clippy と全試験が Wayland と X11 の両方で通る。
2. **crate の構成** が設計書 4.1 節の表と一致する（`kivact-wire`・`kivact-reactive`・`kivact-task`・`kivact-motion`・`kivact-text` がなく、`kivact-runtime`・`kivact-test`・`arvane-virtual` がある）。`visible_to` が表のとおり。
3. **検査**: 設計書 18.20 節の全検査の `skip` が空で、違反が 0。clippy の `too_many_lines`・`disallowed_types` が workspace の lint で通る。例外は inline の目印と `#[expect(.., reason)]` だけ。
4. **付録 A の項目（169 と P-1 が足したもの）**と、作業中に追記した項目が全部「解消後」の形になっている（`PROGRESS.md` の unit ごとの点検表の報告で確かめる）。
5. **公開 API** が付録 B.1 と一致する（facade の rustdoc の module の一覧と item の一覧を比べる）。prelude が付録 B.2 の一覧と一致する。
6. **kit**: 付録 C.2 の Tier 1・2・3 の部品が `kivact::ui` にあり、5 kit に重複の版がない。kit の規約（設計書 12 章）を満たす。kit の部品の公開の option が、中核へ上がった部品に全部対応している（K の各 unit の対応表）。
7. **文書**: `docs/` と README の全部の code block が doctest で通る。英語と日本語が対になっている。
8. **戦略の遂行性の確認**（設計書 15 章）: 次の三つを、実際に試しの branch で一つずつ足してみて（試しの変更は捨てる）、変えた crate の数が設計書の表のとおりであることを `PROGRESS.md` に記録する: (a) 支援技術の性質（例: `aria-keyshortcuts` に当たる `key_shortcuts`）、(b) text style の性質（例: `text_wrap_style`）、(c) OS サービス（例: `beep`）。
9. **17 章の決定（D-13〜D-31）** の各 unit の完了条件が満たされている。木は engine の一つ、arvane-native は port と pipeline、feature は設計書 17.3 節の表の crate だけ、退場の仕組みは presence 一つ、clippy の `too_many_lines` の違反が 0。
10. **終わりの全数調査** で暗黙の状態機械の残りが 0。

---

## 13. unit の一覧

「依存」と「並列」の列は 1 章の wave の表が置き換える。この表は unit の一覧と規模として読む。

| ID | 名前 | 依存 | 並列 | 主な crate | 振る舞いの変更 | 規模 |
|---|---|---|---|---|---|---|
| P-0 | 準備 | — | — | dev/xtask | なし | 中 |
| P-1 | 状態の全数調査 | P-0 | — | 全部（付録 A に足すだけ） | なし | 中 |
| P-2 | 比較用の trace | P-1 | — | core | なし | 小 |
| V-1a | 名前の衝突を先に解く | P-2 | — | types、engine、arvane 系、core | なし | 中 |
| V-1b | `kivact-wire` の解体 | V-1a | — | types、engine、全部 | なし | 大 |
| V-2 | semantics を一つに | V-1b | — | types、engine、arvane 系、core | なし | 中 |
| V-3 | 環境・設定・option・request | V-2 | — | types、engine、arvane 系、core | なし | 中 |
| V-4 | 共通の部品と定数 | V-3 | — | types、各所 | なし | 小 |
| V-5 | 入力 event を一つに | V-4 | — | 全部 | なし | 大 |
| V-6 | 描画の記録・render・画素・text style | V-5 | — | types、style、lower、engine、arvane 系 | なし | 大 |
| A-0 | `Backend` と注入 | R-1 | R-2〜R-6 と | arvane 系、engine、core | なし | 中 |
| A-1a | `arvane-virtual` と試験の経路 | A-0 | R-* と | arvane 系、engine、core の test | 試験の経路が本番に一致 | 大 |
| A-1b | 試験の経路の名前と残り | A-1a | R-* と | arvane 系、engine、core の test | なし | 中 |
| A-2 | サービスと capability | A-1b | R-* と | arvane 系、engine、core | なし | 中 |
| A-3 | native の runner と shell | A-2 | R-* と | arvane-shell、arvane-native、interaction | なし | 中 |
| A-4 | embed の lifecycle | A-3 | R-* と | arvane-native | なし | 中 |
| A-5 | GPU の状態 | A-4 | R-* と | arvane、arvane-render、gpu-share | なし | 中 |
| A-6 | arvane-native の port と pipeline | A-5 | R-* と | arvane-native | なし | 大 |
| E-1 | text の統合と lower の公開面 | A-6 | R-* と | engine、lower | なし | 中 |
| X-1 | feature の閉じ込め | E-1 | — | 全部の Cargo.toml | なし | 中 |
| E-2 | host の状態機械 | X-1、R-2 | R-3〜R-6 と | engine | なし | 中 |
| E-3 | pointer と key | E-2 | R-* と | engine | `key_up` の宛先 | 中 |
| E-4 | IME と編集 | E-3 | R-* と | engine、editing | なし | 中 |
| E-5 | layout と presence | E-4 | R-* と | layout、engine | なし | 中 |
| E-6 | 支援技術 | E-5、A-3 | R-* と | engine | なし | 中 |
| E-7 | paint | E-6 | R-* と | engine | なし | 小 |
| E-8 | engine の fragment | E-7 | R-* と | engine、layout | `Display::Contents` | 中 |
| E-9 | 出現と退場の大きさ | E-8 | — | engine、style、core | 新しい API | 中 |
| R-1 | `kivact-runtime` を作る | V-6 | — | runtime、style、i18n、core | なし | 中 |
| R-2 | host との境界と turn | R-1 | A-*・E-* と | runtime、core | なし | 大 |
| R-3 | reactive の内部 | R-2 | A-*・E-* と | runtime | なし | 中 |
| R-4 | 時間の仕組み | R-3 | A-*・E-* と | runtime、core | なし | 中 |
| R-5 | 非同期の状態 | R-4 | A-*・E-* と | runtime | `blocking` の shutdown 後は error | 中 |
| R-6 | `Machine` と `Trigger` | R-5 | A-*・E-* と | runtime、core、docs | 新しい API | 中 |
| R-7 | 内部の handler の型 | R-6 | A-*・E-* と | runtime | なし | 小 |
| C-1 | 起動の経路と `kivact-test` | R-6、A-1b、E-2 | E-3〜E-7 と | core、test、facade | 試験が App の設定を使う | 大 |
| C-2 | i18n と style の純粋化 | C-1 | — | i18n、style、core、ui | 書式の関数 | 中 |
| C-3 | view の状態 | C-2、R-7 | — | core | なし | 中 |
| C-4 | part の入力と事実の持ち主 | C-3、E-3 | — | core、engine | なし | 中 |
| C-5 | core の木を side table に | C-4、E-8 | — | core | なし | 大 |
| N-1 | module と正準パス | C-5、E-9、A-6 | — | 全部 | 公開 API | 大 |
| N-2 | handler | N-1 | — | 全部 | 公開 API | 大 |
| N-3 | modifier の語彙 | N-2 | — | core、interaction、ui、kit | 公開 API | 大 |
| N-4 | prop の形 | N-3 | — | 全部 | 公開 API | 中 |
| N-5 | 子・slot・選択・文脈・回復 | N-4 | — | 全部 | 公開 API | 大 |
| N-6 | `EditPhase` と `on_edit` | N-5 | — | core、interaction、ui | `on_edit` の網羅 | 中 |
| N-7 | interaction の名前と試験の API | N-6 | — | interaction、ui、test、kit | 公開 API | 中 |
| N-9 | kit・例・文書・試験の追従 | N-7 | — | kit、例、docs、dev/tests | なし | 大 |
| N-8 | 公開 API の検査 | N-9 | — | dev/xtask、全部 | なし | 中 |
| I-0 | option と policy | N-8 | — | interaction、ui、kit | なし | 中 |
| I-1 | 開閉の family | I-0 | I-4 と | interaction | なし | 中 |
| I-2 | menu と選択の family | I-1、I-4 | I-3 と | interaction | なし | 大 |
| I-3 | 編集・drag・文字編集・toggle | I-1、I-4 | I-2 と | interaction | なし | 中 |
| I-4 | 一覧の family | I-0 | I-1 と | interaction | なし | 大 |
| I-5 | 値の family | I-3 | I-6 と | interaction | なし | 中 |
| I-6 | 通知と追従の family | I-1、I-4 | I-5 と | interaction | なし | 中 |
| U-1 | ui の gesture | I-3 | U-2 と | ui | なし | 中 |
| U-2 | ui の開閉と編集 | I-1、I-3 | U-1 と | ui | なし | 小 |
| I-7 | 出現と退場の一本化 | I-1、U-1、E-9 | — | interaction、ui、kit | 動きの仕組み | 中 |
| I-8 | 振る舞いを APG に合わせる | I-7、U-2 | — | interaction、ui | APG との違い | 大 |
| T-1 | 編集する node の制約の解除 | I-8 | — | engine、core、interaction | 新しい API | 中 |
| T-2 | 公開 API だけの text field | T-1 | — | public-api の試験 | なし | 中 |
| G-1 | Path と snapshot | T-2 | G-* と | style、core、engine | 新しい API | 中 |
| G-2 | 明示の pointer の capture | T-2 | G-* と | core、engine | 新しい API | 小 |
| G-3 | モバイルの境界 | T-2 | G-* と | types、arvane 系、engine、core、ui | 新しい API | 中 |
| G-4 | soft kit が求めた中核の不足 | T-2 | G-* と | style、engine、core、motion、interaction | 新しい API | 大 |
| G-5 | 非同期の標準部品 | T-2 | G-* と | runtime、ui | 新しい API | 小 |
| L-1 | ICU4X による地域書式 | C-2 | I-* と | i18n | 全 locale の書式 | 中 |
| K-0 | kit のための中核の部品 | G-1〜G-5、L-1 | — | core、ui、kit | 新しい API | 中 |
| K-1 | 日付・carousel・dropzone | K-0 | K-2〜K-4・K-6 のうち 1 つと | interaction、ui、kit | 中核の部品 | 大 |
| K-2 | 開閉と hover と選択の入力 | K-0 | 同上 | interaction、ui、kit | 中核の部品 | 大 |
| K-3 | navigation と focus の部品 | K-0 | 同上 | interaction、ui、kit | 中核の部品 | 大 |
| K-4 | collection と creative と game | K-0 | 同上 | ui、kit | 中核の部品 | 中 |
| K-5 | soft に残る状態と hook | K-9、I-5 | — | soft | なし | 中 |
| K-6 | 見た目の部品 | K-0 | 同上 | ui、kit | 中核の部品 | 大 |
| K-8 | `viewport` を中核へ | K-1〜K-4、K-6 | — | core、creative | 中核の部品 | 中 |
| K-9 | Tier 3 の部品を中核へ | K-8 | — | ui、kit | 中核の部品 | 大 |
| K-7 | kit の規約の残り | K-5 | — | kit、catalog、dev/xtask | 公開 API | 大 |
| F-1 | 文書 | K-7 | — | docs | なし | 大 |
| F-2 | 検査の仕上げ | F-1 | — | dev/xtask、templates、例・catalog | なし | 中 |
| F-3 | 最後の確認と PR の説明の下書き | F-2 | — | — | なし | 小 |
