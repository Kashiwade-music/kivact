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

## KPI の後退
## KPI の未達
## 台帳
