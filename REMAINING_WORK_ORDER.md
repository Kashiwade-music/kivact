# 作業指示書: Foundation v3 の残りの課題

- 発行: 2026-11-06
- 対象: `refactor/kashiwade/foundation-v3` @ `b76cf06e`
- 実装担当: ChatGPT Codex（主エージェント）と、Codex が起動するサブエージェント

この branch は、Foundation v3 の再設計と、その後のレビューの修正を済ませた状態にある。本書は残った課題だけを定める。全部を終えたら、**本書を消す commit を作って push し**、作者に知らせて止まる。`main` への PR と squash merge は作者が行う。本書は公開リポジトリに残さない。

---

## 0. 進め方

- **規則:** `CONTRIBUTING.md` と `docs/for-kivact-developer/architecture.md` に従う。互換の wrapper・alias・旧 API を残さない。試験は `kivact::test` の E2E だけで、不具合の再現だけの試験は足さない。既存の試験の assertion を弱めない。CI（`.github/workflows/`）は変えない。force push はしない。
- **並べ方:** 課題は互いに file が重ならない。サブエージェントは同時に 6 つまで使い、課題ごとに worktree を作る（`git worktree add .claude/worktrees/<id> -b wt/<id> refactor/kashiwade/foundation-v3`）。新しい worktree では `git submodule update --init` を先に回す（`vendor/xtask-core`）。
- **build の資源:** C: の空きは少ない（作業の開始の時点で約 55 GB）。全部の worktree で一つの target dir を共有する（`CARGO_TARGET_DIR=C:/w/kashiwade/kivact/target-wt`）。共有すると cargo が lock で順番待ちになるが、受け入れる。全部の cargo に `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` を付ける。空きが 30 GB を切ったら、動いていない target dir を `cargo clean --target-dir <dir>` で消す。linker が LNK1285（壊れた PDB）を出したら、その `.pdb` を消して回し直す。
- **試験の回し方:**
  - 公開 API の E2E は、**workspace 全体の feature で** 回す: `KIVACT_TEST_MIRROR=off cargo test --workspace --all-features --exclude gpu-interop-producer --test e2e -- <名前>`。`-p kivact-public-api-tests` だけで回すと feature の組み合わせが変わり、課題 5 の試験が落ちる。
  - 文書の doctest は、並列の compile が `can't find crate for kivact`（E0463）や link.exe の error で落ちることがある。`RUSTDOCFLAGS="--test-args=--test-threads=4"` を付ける。
  - GPU の試験は直列に回す: `cargo test -p gpu-interop-producer --all-features -- --test-threads=1`。
- **記録:** 決めたことは commit message に書く。記録の file は作らない。

---

## 1. 課題

### R-1 engine の presence の clip を閉じる

- **不具合:** 一覧の node（listed）が出現・退場の大きさの動きの最中だと、paint の walker は presence の clip を `PushClip` で積む（`crates/kivact-engine/src/paint/frame_scene.rs:184`）。ところが、閉じる `Entry::Close` を出す条件 `closes()`（`crates/kivact-engine/src/paint/list/walk.rs:24`）に presence の clip が入っていない。そのため、overflow の clip を持たない node が大きさの動きの最中だと、後ろの兄弟と overlay が全部その node の rect で切られる。
- **経緯:** `closes()` に `|| step.presence_clip.is_some()` を足す修正を入れたところ、`presence::toast_swipe_dismiss_exits_from_the_drag_offset`（`dev/tests/kivact-public-api/tests/e2e/presence.rs:153`）が落ちたので取り消した（`b3bbe10c`）。指を離した後も toast が残る。swipe の release が届いていないか、届いても dismiss に至っていないと考えられる。
- **作業:**
  1. 修正を入れ直し、なぜ toast の swipe の dismiss が止まるのかを突き止める。疑う所は、push と pop の数が合わなくなって、hit test か `frame_scene.rs:301` の側の pop が二重になることである。
  2. clip の漏れと toast の試験の両方を正しく直す。試験の期待（指を離したら toast は消え、drag の位置から退場の動きを始める）は変えない。
- **確認:** 上の E2E を workspace の feature で回し、`presence::` を全部通す。`cargo test -p kivact-engine --all-features`。

### R-2 toast は条件を満たす swipe の後だけ滑らせて消す

- **不具合:** `crates/kivact-ui/src/toast_swipe/handle.rs` が、swipe の滑らせて消す `exit` を、どの状態でも付けている。そのため soft の theme では、時間切れや閉じる button で消える toast まで横に滑って消える。`main` では、条件を満たす swipe の後だけ付けていた（`git show main:crates/kivact-ui/src/toast_swipe.rs` の 221〜231 行）。
- **経緯:** `Swipe::Released` のときだけ `exit` を付け、dismiss を一つ後の display frame にずらす修正（`8eabdd2d`）を入れたところ、R-1 と同じ試験が落ちたので取り消した（`e971bbd3`）。
- **作業:**
  1. R-1 を先に終える（同じ試験を使う）。
  2. `8eabdd2d` を入れ直す（`git cherry-pick 8eabdd2d`）。
  3. 試験が落ちる理由を調べて直す。一つ frame を遅らせると、指を離した直後には toast がまだ残る。直後に消えていることを試験が確かめているなら、frame を遅らせずに、release の同じ turn で exit の target を commit してから dismiss する形にする。
  4. 時間切れと閉じる button の toast は、kit の theme の元の退場の形のままであること。
- **確認:** `presence::` の E2E を workspace の feature で回す。`cargo test -p kivact-ui-soft --all-features`。

### R-3 試験の driver で engine の自己検査を有効にする

- **背景:** `main` では、試験の driver が settle のたびに、debug の build で `host.self_check()` を呼んでいた。今は core と engine の木の一致だけを確かめている（`crates/kivact-test/src/screen/driver.rs:96-112` の `self_check` と `Inspect::check_tree`）。
- **不具合:** `host.self_check()` を戻すと、公開 API の E2E が 468 件中 347 件落ちる。message はどれも `computed text node N: kept … resolved …` で、engine が保持している text の計算済みの style が古くなっている。
- **作業:**
  1. engine の text style の cache が古くなる原因を直す。疑う所は、継承した値が変わったときに子の text の計算値を作り直していないことである。`crates/kivact-engine/src/model/text.rs` と `model/apply.rs` の text の継承の走査、`host/testing/self_check.rs` の検査の中身を読む。
  2. `driver.rs` の `self_check` に engine の自己検査も足す。
  3. 公開 API の E2E を全部通す。
- **確認:** `cargo test --workspace --all-features --exclude gpu-interop-producer`。

### R-4 中核の部品が足りないため kit で戻せなかった二つ

kit は外部の利用者がいるので縮めない（`main` にあった機能は、名前が変わっても残す）。

1. **soft の avatar の既定の tooltip:** `main` の soft の avatar は、名前の tooltip を既定で出していた（`git show main:crates/kivact-ui-soft/src/avatar.rs` の 20 行と 108 行）。中核の `ui::avatar`（`crates/kivact-ui/src/avatar.rs:31`）に、tooltip の有無を選ぶ option と、その既定値の theme の token を足す。soft の theme はその既定を「出す」にする。soft の README の、avatar を `tooltip(name).attach(..)` で包む回避の記述を消す。
2. **`TableRow::selected`:** basic の表の行の `selected` の状態（`main` の basic の table）を戻す。中核の `ui::table_row`（`crates/kivact-ui/src/table.rs:51`）に `selected(impl IntoValue<bool>)` を足す。行の semantics の selected と recipe の見た目の両方に効かせる。basic の README の回避の記述（`.with_node(|n| n.selected(..))`）を消す。
- **試験:** 公開の振る舞いが増えるので、E2E を一つずつ足してよい: `avatar_shows_its_name_tooltip_by_default_under_a_theme_that_asks_for_it`、`table_row_selected_is_exposed_and_styled`。
- **文書:** `docs/for-kivact-ui-user/{en,jp}/` の該当の page と、kit の README の「Migrating from the previous release」の表。

### R-5 公開 API の試験が feature の組み合わせに頼っている

- **不具合:** `presence::toast_swipe_dismiss_exits_from_the_drag_offset` は、`cargo test -p kivact-public-api-tests --all-features` では落ちる。workspace 全体の feature では通る（R-1・R-2 の前から。`86c3ae5f` でも同じ）。試験の package の `Cargo.toml` が、通るのに要る feature を自分で有効にしていない。
- **作業:**
  1. workspace の feature の和で有効になり、package だけでは無効になる feature を突き止める（`cargo tree -e features -p kivact-public-api-tests` を二通りで比べる）。
  2. `dev/tests/kivact-public-api/Cargo.toml` が、その feature を明示的に有効にする。
  3. `-p kivact-public-api-tests --all-features` だけで全部の E2E が通ることを確かめる。

### R-6 名前の直し

1. **`DataTable::initially_expanded`:** 実際は、tree の列の見出しに「全部開く・全部閉じる」の control を出す option である（`crates/kivact-ui/src/data_table/builder.rs:196`）。名前が意味と合わないので、`expand_all_control()` に改名する。呼び出し元は次の 4 つ。
   - `crates/kivact-ui-soft/tests/consumer/table_tree.rs:32`
   - `examples/kivact-ui-soft/src/docs/collections.rs:465` と 467 行の文字列
   - `dev/xtask/policy/workspace.toml:432` の構造の option の一覧
   - `crates/kivact-ui-soft/README.md:266`
2. **`kivact::interaction::OpenedBy`**（`crates/kivact-interaction/src/hover_card.rs:8`）: hover card だけの型なのに、crate の root に一般的な名前で出ている。`HoverCardOpenedBy` に改名する。
3. **`Toaster`**（`crates/kivact-interaction/src/toaster/api.rs:122`）: interaction の handle には `*Api` の接尾辞を付ける規則がある（`docs/for-kivact-developer/decisions.md` の作者の決定）。`toaster()` が返す handle なので、`ToasterApi` に改名する。

- 呼び出し元（kit、例、文書、試験）は全部直す。kit の README の移行の表に行を足す。`cargo xtask check` の `api-shape`・`canonical-path`・`name-collision` を通す。

### R-7 `virtual_list` の行の形を `for_each` に揃える

- **不一致:** `for_each` の行の関数は、`ListSource` の `S::Row` を一つ受け取る（書ける行は `Binding`）。`virtual_list` はまだ `(K, Value<T>)` を受け取り、`Binding` の行もない（`crates/kivact-core/src/view/flow/virtual_list.rs`）。二つの keyed list の書き方が違うので、利用者が迷う。
- **作業:**
  1. `virtual_list` も `ListSource` を受け取り、行の関数が `S::Row` を一つ受け取る形にする（`for_each` の `list.rs` の作りを読んで、同じ trait を使う）。
  2. 呼び出し元と文書（`docs/for-kivact-user/{en,jp}/how-to-guide/lists-and-tables.md` と `layout.md`）を直す。
- **確認:** `virtual_list` の E2E と、kit の一覧の試験。

---

## 2. 行わないこと（決定済み）

- **calendar の範囲選択の最初の書き込み:** 戻さない。最初の選択では何も書かない、が現在の契約である。公開 API の試験 3 件がこれを確かめており、文書も合わせてある。
- **位置が分からないうちのホイールの event:** 捨てたままにする。3 つの OS の実装が一貫して捨てている。届けるには位置を作り上げる必要があり、それがマウスの位置として記録されてしまう。
- **creative の Cargo の feature `gpu-interop`:** 戻さない。kit の README の移行の表に書いてある。
- **soft の dialog の、粗い pointer での reduced motion の fade:** 振る舞いの定義がないので作らない。

---

## 3. 終わりの確認

全部の課題を merge したら、integration の checkout で次を回し、全部 exit 0 にする。

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
KIVACT_TEST_MIRROR=off RUSTDOCFLAGS="--test-args=--test-threads=4" cargo test --workspace --all-features --no-fail-fast --exclude gpu-interop-producer
KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --all-features
cargo test -p gpu-interop-producer --all-features -- --test-threads=1
KIVACT_TEST_MIRROR=off cargo test --workspace --no-default-features --no-fail-fast --exclude gpu-interop-producer
bash dev/tools/check-features.sh
cargo xtask check
wsl -d Ubuntu-24.04 -- bash -lc 'cd /mnt/c/w/kashiwade/kivact && CARGO_TARGET_DIR=$HOME/kivact-target cargo clippy --workspace --all-targets --all-features -- -D warnings'
```

- `cargo xtask check` が `architecture/` を作り直したら、それも commit する。
- **公開の確認:** 次が何も出さないこと。

  ```text
  git grep -nIE "SM-[A-Z]+[0-9]|作業手順書|REFACTORING_|orchestrator|オーケストレータ|Codex|subagent|サブエージェント|Foundation v3" -- . ':!REMAINING_WORK_ORDER.md'
  ```

  （`dev/tests/kivact-public-api/src/gpu_interop.rs` の定数 `W2` のような偽の当たりは除く。）
- **最後:** 本書（`REMAINING_WORK_ORDER.md`）を消す commit を作り、push する。課題の worktree と `wt/*` の branch を消す。作者には次を知らせる。
  - 各課題の結果（直した内容と、直せなかったものの理由）
  - 上の確認の結果
  - 作者が行う後始末: squash merge の後に、remote の `docs/kashiwade/foundation-v3-refactoring-design` と `refactor/kashiwade/foundation-v3` を消すこと（どちらも履歴に計画の文書を含む）
