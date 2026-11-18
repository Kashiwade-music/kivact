# 未解決・差戻し（Orchestrator が更新）

## 基点の失敗
（W0 の基点の gate で失敗した試験。改修の失敗ではない）

- `kivact-ui-soft` の `--test consumer`：`chat_keyboard::composer_inserts_shift_enter_and_recalls_the_last_submission`（`crates/kivact-ui-soft/tests/consumer/chat_keyboard.rs:58`）。left=`Some("Another message")`、right=`Some("First line\nSecond line")`。基点 `2b8df5ac` で既に失敗。直さない。
- 1 回目の `cargo test --workspace` はこの binary で止まった（cargo の既定の fail-fast）。`--no-fail-fast` で全件を再実行した結果：`--all-features` も `--no-default-features` も失敗は上の `chat_keyboard` の 1 件だけ（81・79 binary を実行）。
- GPU E2E（`kivact-public-api-tests --test e2e`、KIVACT_TEST_GPU=1、751 件中 4 件失敗、基点で既に失敗）：
  - `app_handle::close_during_setup_exits_after_the_first_turn`：`crates/kivact-test/src/screen/driver.rs:50`、「GPU device did not start within five seconds」（GPU の起動の遅れ。環境の差の疑い）
  - `compositor::idle_window_redraws_whole_after_release`（`dev/tests/kivact-public-api/tests/e2e/compositor.rs:561`）：first frame after release rasters 4000 px
  - `frame_sources::incremental_frames_match_a_fresh_render`（`.../e2e/frame_sources.rs:264`）：20 partial frames が fresh frame と一致しない
  - `layout_scroll::a_long_list_draws_what_reaches_its_viewport_from_rows_outside_it`（`.../e2e/layout_scroll.rs:187`）：layout pass 後の pixel が fresh frame と一致しない
  - 直さない。改修後の同じ試験の結果と比べる基準にする。

## 差戻し
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **S01**（W1a・S lane）、付属書 S の「## S01：」の手順 2・3。`crates/kivact-types/src/cell.rs` の `Deferred::new`（構成式 `Self(std::cell::OnceCell::new())`、337 行）と `SharedDeferred::new`（`Self(std::sync::OnceLock::new())`、371 行）が `clippy::disallowed_types`（`clippy.toml`）で error になる。付属書の `#[expect(clippy::disallowed_types, ...)]` は struct の field にだけ付き、構成式の path は覆わない（手順 3 の位置の変更でも消えない）。clippy と `cargo xtask check --crate kivact-types` の 2 件が失敗する。Orchestrator は expect・allow を足さない（作業指示書 0.2 の 4）。設計者に、構成式の書き方か覆う印の文面を決めてほしい。
  - 状態：commit `b0e84c65` は `wt/rw-W1a-S` に残し、統合 branch には merge しない。W1a の S lane は止まり、S01 の `Deferred` を前提にする unit（S02〜、R、U の一部など）も止まる。
  - 「`cargo test -p kivact-types cell`」は通っている（2 件）。

- **T01**（W1a・T0 lane、commit `5beb3a38`、`wt/rw-W1a-T0` に残す）、付属書 T の「## T01：」の完了条件。確認コマンドはすべて通る（xtask の check・clippy・test 53 件）。完了条件の「`rg` で `state-fields|state_fields|report-audit|state-audit` が `dev/xtask/src` で何も出さない」は満たせない。残るのは仕様のコードそのもの（`dev/xtask/src/state/check.rs:12` の `OBSOLETE` 定数と `:129` の検査文面、`dev/xtask/src/state/tests.rs:101` の試験）。旧い印を検出するため仕様が要求する文字列で、rg の条件と仕様のコードが食い違う。Orchestrator は文字列を別の形に置き換えない。設計者に、rg の条件を「旧い印の検出のための文字列を除く」にするか、検出の文字列の書き方を決めてほしい。
  - 状態：T0 lane は T02 へ進む（T01 の機能は要件を満たす）。W1a の merge は S01 の差戻しが解けるまで行わない。

- **T02**（W1a・T0 lane、commit `e2879bee`、`wt/rw-W1a-T0` に残す）、付属書 T の「## T02：」。確認はすべて通る（xtask の clippy・test 54 件、全体の check は ok）。付属書の手順が `dev/xtask/policy/workspace.toml` の `[visible_to]` の `kivact-devtools-protocol` 行から `kivact-devtools-client` を消すよう求めるが、`kivact-devtools-client` は T13 まで存在して protocol に依存する。消すと全体の検査が「`kivact-devtools-client` has a normal dependency on `kivact-devtools-protocol`」で error になる。そのため行は元に戻してある（現在の行に `kivact-devtools-client` が残っている）。設計者に、この名前を消すのを T13 の client の削除に合わせるかを決めてほしい。
  - 注記：`policy/workspace.toml` の commit の diff は、CRLF と LF の正規化で 1044 行の変更として見える（内容の差は少ない）。merge の前に行末を確かめる。

- **T04**（W1a・T0 lane、commit `a75087ab`、`wt/rw-W1a-T0` に残す）、付属書 T の「## T04：」。確認はすべて通る（xtask 56 件、protocol の試験、全体の check は ok）。完了条件の二つが満たせない。
  - `rg -n "devtools-codegen|devtools_codegen" dev crates` が 1 件出る：`dev/README.md:41`（「`kit-literal-color`・`os-branch`・`structural-options`・`devtools-codegen` を選べます」）。この file は付属書で T21（W1b・T6 lane）の所有で、T04 の所有外のため変えていない。T21 で直す。
  - `cargo xtask check --no-write --only docs-pairs --strict` が 4 件出る。付属書の想定の canvas.md・windows.md は含む。残り 2 件は `docs/for-kivact-ui-user/jp/basic/getting-started/02-build-the-settings-form.md` と `03-dark-mode-and-feedback.md` の Rust block 1 が en 側と字句で違う（jp 02 は `signal(Some("ja"))`、en は `Some("en")` など）。この 2 件は付属書 T17 の所有に無い。設計者に、どの unit が直すかを決めてほしい。`docs-pairs` は `[rework.pending]` にあるため通常の `--only local` は失敗しない。

## KPI の後退
## KPI の未達
## 台帳
